//! The editor's document, driven without a window (gui-edit).
//!
//! `Document` is what the Tauri commands wrap, so these tests reach the same code the window does.
//! Every refusal is compared with the text the real `edit` prints for the same request, through
//! the CLI's library entry point, so the two cannot drift apart.

use std::path::{Path, PathBuf};

use stidcart_core::{crt, listing, read, reserved, CartBuilder};
use stidcart_gui::document::{Document, Edit};
use stidcart_gui::{open, ListingDto};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn fx(name: &str) -> String {
    fixtures().join(name).to_string_lossy().into_owned()
}

/// A scratch directory unique to the calling test.
fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("stidcart-gui-doc-{tag}"));
    if dir.exists() {
        make_writable(&dir);
        let _ = std::fs::remove_dir_all(&dir);
    }
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[cfg(unix)]
fn make_writable(dir: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o755));
}

/// A copy of a fixture in a scratch directory, so a save never touches `fixtures/`.
fn copy(tag: &str, name: &str) -> PathBuf {
    let path = scratch(tag).join(name);
    std::fs::copy(fixtures().join(name), &path).unwrap();
    path
}

/// What `stidcart edit <cart> <args…>` reports, through the CLI's library entry point.
fn cli_edit(cart: &Path, args: &[&str]) -> Result<Option<String>, String> {
    let mut all = vec!["edit".to_string(), cart.to_string_lossy().into_owned()];
    all.extend(args.iter().map(|a| a.to_string()));
    stidcart_cli::run(&all)
}

/// The refusal `edit` gives for a request on a scratch copy, leaving the original alone.
fn cli_refusal(cart: &Path, args: &[&str]) -> String {
    let out = cart.with_extension("cli-out.crt");
    let mut with_out: Vec<&str> = args.to_vec();
    let out_str = out.to_string_lossy().into_owned();
    with_out.extend(["-o", &out_str]);
    cli_edit(cart, &with_out).expect_err("edit should refuse this request")
}

fn json<T: serde::Serialize>(v: &T) -> serde_json::Value {
    serde_json::to_value(v).unwrap()
}

fn row_names(doc: &Document) -> Vec<String> {
    doc.dto()
        .listing
        .rows
        .iter()
        .map(|r| r.name.clone())
        .collect()
}

/// Reloading a step is lossless: every snapshot is an image that rebuilds to itself (D-035).
fn assert_reloads(doc: &Document) {
    let bytes = doc.bytes();
    let rebuilt = read::load(bytes)
        .expect("every editable snapshot loads")
        .build()
        .unwrap();
    assert_eq!(rebuilt, bytes, "load(build(b)).build() must equal build(b)");
}

/// A cart whose firmware regions differ from the embedded blobs: `list` describes it, `edit`
/// refuses it. Built the way the CLI's AC-25 test builds it.
fn firmware_mismatch(tag: &str) -> PathBuf {
    let mut img = CartBuilder::new().build_image().unwrap();
    img.write_at(reserved::EAPI_OFFSET, &[0x00; 32]);
    let path = scratch(tag).join("mismatch.crt");
    std::fs::write(&path, crt::write(&img, "probe")).unwrap();
    path
}

fn editable_fixtures() -> Vec<PathBuf> {
    let mut crts: Vec<PathBuf> = std::fs::read_dir(fixtures())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "crt"))
        .filter(|p| read::load(&std::fs::read(p).unwrap()).is_ok())
        .collect();
    crts.sort();
    assert!(crts.len() > 5, "expected several editable fixtures");
    crts
}

// ---- Opening and New -------------------------------------------------------------------------

// AC-59 (RENDER): an editable cart opens clean, listed as stored, and opening writes nothing.
#[test]
fn ac_59_render_an_editable_cart_opens_clean_and_listed_as_stored() {
    let path = copy("ac59", "two.crt");
    let before = std::fs::read(&path).unwrap();
    let doc = Document::open(&path).unwrap();
    let dto = doc.dto();
    assert_eq!(dto.read_only, None);
    assert!(!dto.dirty && !dto.can_undo && !dto.can_redo);
    assert!(!dto.untitled);
    assert_eq!(dto.file, "two.crt");
    assert_eq!(json(&dto.listing), json(&open(&path).unwrap()));
    assert_eq!(
        doc.bytes(),
        before.as_slice(),
        "snapshot 0 is the file itself"
    );
    assert_eq!(
        std::fs::read(&path).unwrap(),
        before,
        "opening writes nothing"
    );
}

// AC-60 (RENDER): a cart `list` describes but `edit` refuses opens view-only with edit's reason.
#[test]
fn ac_60_render_a_cart_edit_refuses_opens_view_only_with_its_reason() {
    let path = firmware_mismatch("ac60");
    let before = std::fs::read(&path).unwrap();
    let reason = cli_refusal(&path, &["-e", "x"]);

    let mut doc = Document::open(&path).unwrap();
    let dto = doc.dto();
    assert_eq!(dto.read_only.as_deref(), Some(reason.as_str()));
    assert_eq!(json(&dto.listing), json(&open(&path).unwrap()));
    assert_eq!(
        doc.apply(Edit::SetCartName { name: "x".into() }),
        Err(reason.clone())
    );
    assert_eq!(doc.save_as(&path.with_extension("copy.crt")), Err(reason));
    assert!(!doc.dto().dirty);
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

// AC-77 (RENDER): New is an untitled, clean, empty cart with the default names `build` uses.
#[test]
fn ac_77_render_new_is_an_untitled_clean_empty_cart() {
    let doc = Document::new();
    let dto = doc.dto();
    assert!(dto.untitled && !dto.dirty && !dto.can_undo);
    assert_eq!(dto.file, "Untitled");
    assert_eq!(dto.read_only, None);
    assert!(dto.listing.rows.is_empty());
    // `build` naming no menu gives ours (D-064), and so does New.
    let empty = CartBuilder::new()
        .with_menu(stidcart_core::menu::Menu::Stidcart)
        .build()
        .unwrap();
    assert_eq!(doc.bytes(), empty.as_slice());
    assert_eq!(
        json(&dto.listing),
        json(&ListingDto::from(&listing::describe(&empty).unwrap()))
    );
}

#[test]
fn rows_line_up_with_the_loaded_entries_and_only_xbanks_are_alignable() {
    for path in editable_fixtures() {
        let doc = Document::open(&path).unwrap();
        let dto = doc.dto();
        let builder = read::load(&std::fs::read(&path).unwrap()).unwrap();
        let entries = builder.entries();
        assert_eq!(dto.listing.rows.len(), entries.len(), "{path:?}");
        assert_eq!(dto.rows.len(), entries.len(), "{path:?}");
        for (i, e) in entries.iter().enumerate() {
            assert_eq!(dto.listing.rows[i].name, e.name(), "{path:?} row {i}");
            let xbank = e.as_cartridge().is_some_and(|c| c.kind.is_xbank());
            assert_eq!(dto.rows[i].alignable, xbank, "{path:?} row {i}");
        }
    }
}

// ---- Edits -------------------------------------------------------------------------------------

// AC-61 (RENDER): an added file becomes the last entry, named by its stem, its kind by content.
#[test]
fn ac_61_render_an_added_file_becomes_the_last_entry_named_by_its_stem() {
    let mut doc = Document::open(&copy("ac61", "two.crt")).unwrap();
    doc.apply(Edit::Add {
        paths: vec![fx("hello.prg")],
        aligned: false,
    })
    .unwrap();
    doc.apply(Edit::Add {
        paths: vec![fx("in_xb16.crt")],
        aligned: false,
    })
    .unwrap();
    let rows = doc.dto().listing.rows;
    assert_eq!(rows.len(), 4);
    assert_eq!(rows[2].name, "hello");
    assert!(rows[2].kind.starts_with("PRG"), "{}", rows[2].kind);
    assert_eq!(rows[3].name, "in_xb16");
    assert_eq!(rows[3].kind, "XB16");
    assert_reloads(&doc);
}

// AC-62 (RENDER): an aligned add marks an xbank `A`; on anything else it is refused as edit does.
#[test]
fn ac_62_render_an_aligned_add_marks_an_xbank_and_refuses_anything_else() {
    let path = copy("ac62", "two.crt");
    let mut doc = Document::open(&path).unwrap();
    for file in ["in_8k.crt", "hello.prg"] {
        let reason = cli_refusal(&path, &["--add-aligned", &fx(file)]);
        let before = json(&doc.dto());
        assert_eq!(
            doc.apply(Edit::Add {
                paths: vec![fx(file)],
                aligned: true
            }),
            Err(reason),
            "{file}"
        );
        assert_eq!(json(&doc.dto()), before);
    }
    doc.apply(Edit::Add {
        paths: vec![fx("in_xb16.crt")],
        aligned: true,
    })
    .unwrap();
    let rows = doc.dto().listing.rows;
    assert!(rows[2].flags.contains('A'), "{}", rows[2].flags);
    assert_reloads(&doc);
}

// AC-63 (RENDER): several files are added all or none, as a single undo step.
#[test]
fn ac_63_render_several_files_are_added_all_or_none_as_one_step() {
    let path = copy("ac63", "two.crt");
    let mut doc = Document::open(&path).unwrap();
    let missing = path
        .with_file_name("missing.prg")
        .to_string_lossy()
        .into_owned();
    let before = json(&doc.dto());
    assert!(doc
        .apply(Edit::Add {
            paths: vec![fx("hello.prg"), missing, fx("in_8k.crt")],
            aligned: false,
        })
        .is_err());
    assert_eq!(json(&doc.dto()), before, "nothing was added");

    doc.apply(Edit::Add {
        paths: vec![fx("hello.prg"), fx("in_8k.crt")],
        aligned: false,
    })
    .unwrap();
    assert_eq!(row_names(&doc), ["hello", "ramp", "hello", "in_8k"]);
    assert!(doc.undo());
    assert_eq!(row_names(&doc), ["hello", "ramp"], "one undo step");
    assert!(!doc.dto().can_undo);
}

// AC-64 (RENDER): removing an entry drops it and moves the later ones up an index.
#[test]
fn ac_64_render_removing_an_entry_moves_the_later_ones_up() {
    let mut doc = Document::open(&copy("ac64", "two.crt")).unwrap();
    doc.apply(Edit::Remove { index: 0 }).unwrap();
    let rows = doc.dto().listing.rows;
    assert_eq!(rows.len(), 1);
    assert_eq!((rows[0].index, rows[0].name.as_str()), (0, "ramp"));
    assert_reloads(&doc);
}

// AC-65 (RENDER): a rename shows the name as stored, shortened when it is too long.
#[test]
fn ac_65_render_a_rename_shows_the_name_as_stored() {
    let mut doc = Document::open(&copy("ac65", "two.crt")).unwrap();
    doc.apply(Edit::Rename {
        index: 0,
        name: "greetings".into(),
    })
    .unwrap();
    assert_eq!(row_names(&doc)[0], "greetings");

    let long = "a much longer name than fits";
    doc.apply(Edit::Rename {
        index: 1,
        name: long.into(),
    })
    .unwrap();
    assert_eq!(row_names(&doc)[1], &long[..16]);
    assert_reloads(&doc);
}

// AC-66 (RENDER): replacing keeps the name, hidden state and place; the kind follows the file.
#[test]
fn ac_66_render_replace_keeps_name_hidden_and_place_and_takes_the_new_kind() {
    let mut doc = Document::open(&copy("ac66", "two.crt")).unwrap();
    doc.apply(Edit::SetHidden {
        index: 1,
        hidden: true,
    })
    .unwrap();
    doc.apply(Edit::Replace {
        index: 1,
        path: fx("in_8k.crt"),
    })
    .unwrap();
    let rows = doc.dto().listing.rows;
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1].name, "ramp");
    assert!(rows[1].flags.contains('H'), "{}", rows[1].flags);
    assert_eq!(rows[1].kind, "8K");
    assert_reloads(&doc);
}

// AC-67 (RENDER): hiding and showing toggle the `H` flag.
#[test]
fn ac_67_render_hide_and_show_toggle_the_h_flag() {
    let mut doc = Document::open(&copy("ac67", "two.crt")).unwrap();
    doc.apply(Edit::SetHidden {
        index: 0,
        hidden: true,
    })
    .unwrap();
    assert_eq!(doc.dto().listing.rows[0].flags, "H");
    doc.apply(Edit::SetHidden {
        index: 0,
        hidden: false,
    })
    .unwrap();
    assert_eq!(doc.dto().listing.rows[0].flags, "-");
    assert_reloads(&doc);
}

// AC-68 (RENDER): alignment toggles `A` on an xbank and is refused, as edit does, on anything else.
#[test]
fn ac_68_render_alignment_toggles_on_an_xbank_and_is_refused_elsewhere() {
    let path = copy("ac68", "cxb16.crt");
    let mut doc = Document::open(&path).unwrap();
    assert!(doc.dto().rows[1].alignable && !doc.dto().rows[0].alignable);

    doc.apply(Edit::SetAligned {
        index: 1,
        aligned: true,
    })
    .unwrap();
    assert!(doc.dto().listing.rows[1].flags.contains('A'));
    doc.apply(Edit::SetAligned {
        index: 1,
        aligned: false,
    })
    .unwrap();
    assert!(!doc.dto().listing.rows[1].flags.contains('A'));

    let reason = cli_refusal(&path, &["--align", "0"]);
    let before = json(&doc.dto());
    assert_eq!(
        doc.apply(Edit::SetAligned {
            index: 0,
            aligned: true
        }),
        Err(reason)
    );
    assert_eq!(json(&doc.dto()), before);
    assert_reloads(&doc);
}

// AC-69 (RENDER): the cart name and the header name change independently.
#[test]
fn ac_69_render_the_cart_and_header_names_change_independently() {
    let mut doc = Document::open(&copy("ac69", "two.crt")).unwrap();
    let built_by = doc.dto().listing.built_by;
    doc.apply(Edit::SetCartName {
        name: "my menu".into(),
    })
    .unwrap();
    assert_eq!(doc.dto().listing.cart, "my menu");
    assert_eq!(doc.dto().listing.built_by, built_by);

    doc.apply(Edit::SetHeaderName {
        name: "hand made".into(),
    })
    .unwrap();
    assert_eq!(doc.dto().listing.built_by, "hand made");
    assert_eq!(doc.dto().listing.cart, "my menu");
    assert_reloads(&doc);
}

// AC-70 (RENDER): a refused edit shows edit's reason and changes nothing at all.
#[test]
fn ac_70_render_a_refused_edit_gives_edits_reason_and_changes_nothing() {
    let hello = fx("hello.prg");
    let many: Vec<&str> = std::iter::repeat_n(["--add", hello.as_str()], 254)
        .flatten()
        .collect();
    let cases: Vec<(&str, &str, Vec<&str>, Edit)> = vec![
        (
            "space",
            "cxbwide.crt",
            vec!["--add", "IN_XBWIDE"],
            Edit::Add {
                paths: vec![fx("in_xbwide.crt")],
                aligned: false,
            },
        ),
        (
            "full",
            "two.crt",
            many.clone(),
            Edit::Add {
                paths: vec![hello.clone(); 254],
                aligned: false,
            },
        ),
        (
            "ocean",
            "cocean.crt",
            vec!["--add", "IN_OCEAN"],
            Edit::Add {
                paths: vec![fx("in_ocean.crt")],
                aligned: false,
            },
        ),
        (
            "unreadable",
            "two.crt",
            vec!["--add", "/nonexistent/stidcart/x.prg"],
            Edit::Add {
                paths: vec!["/nonexistent/stidcart/x.prg".into()],
                aligned: false,
            },
        ),
        (
            "index",
            "two.crt",
            vec!["--remove", "9"],
            Edit::Remove { index: 9 },
        ),
    ];
    let (in_xbwide, in_ocean) = (fx("in_xbwide.crt"), fx("in_ocean.crt"));
    for (tag, cart, args, edit) in cases {
        let args: Vec<&str> = args
            .into_iter()
            .map(|a| match a {
                "IN_XBWIDE" => in_xbwide.as_str(),
                "IN_OCEAN" => in_ocean.as_str(),
                other => other,
            })
            .collect();
        let path = copy(&format!("ac70-{tag}"), cart);
        let reason = cli_refusal(&path, &args);
        let mut doc = Document::open(&path).unwrap();
        doc.apply(Edit::SetCartName {
            name: "before".into(),
        })
        .unwrap();
        let (dto, bytes) = (json(&doc.dto()), doc.bytes().to_vec());
        assert_eq!(doc.apply(edit), Err(reason), "{tag}");
        assert_eq!(json(&doc.dto()), dto, "{tag}: the view is unchanged");
        assert_eq!(
            doc.bytes(),
            bytes.as_slice(),
            "{tag}: the cart is unchanged"
        );
        assert!(doc.undo(), "{tag}: history still holds the earlier edit");
        assert!(!doc.dto().can_undo, "{tag}: and only that one");
    }
}

// AC-71 (RENDER): after an edit the listing is the cart as saving writes it, re-placed whole.
#[test]
fn ac_71_render_after_an_edit_the_listing_is_the_cart_saving_writes() {
    let path = copy("ac71", "two.crt");
    let original = std::fs::read(&path).unwrap();
    let mut doc = Document::open(&path).unwrap();
    let hello_was = doc.dto().listing.rows[0].layout.clone();
    doc.apply(Edit::Remove { index: 1 }).unwrap();

    let mut expected = read::load(&original).unwrap();
    expected.remove(1);
    let expected = expected.build().unwrap();
    assert_eq!(doc.bytes(), expected.as_slice());
    assert_eq!(
        json(&doc.dto().listing),
        json(&ListingDto::from(&listing::describe(&expected).unwrap()))
    );
    assert_ne!(
        doc.dto().listing.rows[0].layout,
        hello_was,
        "the untouched entry was re-placed"
    );
    assert_reloads(&doc);
}

// ---- Undo, redo and the unsaved-changes state --------------------------------------------------

fn rename(doc: &mut Document, index: usize, name: &str) {
    doc.apply(Edit::Rename {
        index,
        name: name.into(),
    })
    .unwrap();
}

// AC-72 (RENDER): dirty means "not at the state last saved", whichever way undo and redo move.
#[test]
fn ac_72_render_dirty_tracks_the_state_last_saved() {
    let mut doc = Document::open(&copy("ac72", "two.crt")).unwrap();
    rename(&mut doc, 0, "one");
    assert!(doc.dto().dirty);
    assert!(doc.undo());
    assert!(!doc.dto().dirty, "back at the state it was opened in");
    assert!(doc.redo());
    assert!(doc.dto().dirty);

    doc.save().unwrap();
    assert!(!doc.dto().dirty);
    assert!(doc.undo());
    assert!(doc.dto().dirty, "away from the saved state");
    rename(&mut doc, 1, "two");
    assert!(doc.dto().dirty);
    assert!(doc.undo());
    assert!(
        doc.dto().dirty,
        "the saved state was in the discarded redo tail, so nothing is clean until a save"
    );
}

// AC-73 (RENDER): undo and redo walk every edit; a new edit after an undo discards the redo tail.
#[test]
fn ac_73_render_undo_and_redo_walk_every_edit() {
    let path = copy("ac73", "two.crt");
    let original = std::fs::read(&path).unwrap();
    let mut doc = Document::open(&path).unwrap();
    let mut states = vec![doc.bytes().to_vec()];
    for (i, name) in ["a", "b", "c"].iter().enumerate() {
        rename(&mut doc, i % 2, name);
        states.push(doc.bytes().to_vec());
    }
    for i in (0..3).rev() {
        assert!(doc.undo());
        assert_eq!(doc.bytes(), states[i].as_slice());
    }
    assert_eq!(doc.bytes(), original.as_slice());
    assert!(!doc.undo() && !doc.dto().can_undo);
    for state in &states[1..] {
        assert!(doc.redo());
        assert_eq!(doc.bytes(), state.as_slice());
    }
    assert!(!doc.redo() && !doc.dto().can_redo);

    assert!(doc.undo() && doc.dto().can_redo);
    rename(&mut doc, 0, "d");
    assert!(!doc.dto().can_redo && !doc.redo(), "the redo tail is gone");
}

// ---- Saving ------------------------------------------------------------------------------------

// AC-74 (RENDER): save replaces the file with the edited cart; a clean cart is never rewritten.
#[test]
fn ac_74_render_save_writes_the_edited_cart_and_never_rewrites_a_clean_one() {
    let path = copy("ac74", "two.crt");
    let mut doc = Document::open(&path).unwrap();
    let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
    doc.save().unwrap();
    assert_eq!(
        std::fs::metadata(&path).unwrap().modified().unwrap(),
        modified,
        "a clean save writes nothing"
    );

    rename(&mut doc, 0, "saved");
    doc.save().unwrap();
    assert!(!doc.dto().dirty);
    assert_eq!(std::fs::read(&path).unwrap(), doc.bytes());

    let mut untitled = Document::new();
    untitled
        .apply(Edit::Add {
            paths: vec![fx("hello.prg")],
            aligned: false,
        })
        .unwrap();
    assert!(untitled.save().is_err(), "an untitled cart needs a path");
    assert!(untitled.dto().dirty);
}

// AC-75 (RENDER): Save As writes the new path and adopts it; the source is untouched.
#[test]
fn ac_75_render_save_as_writes_the_new_path_and_adopts_it() {
    let path = copy("ac75", "two.crt");
    let original = std::fs::read(&path).unwrap();
    let mut doc = Document::open(&path).unwrap();

    let verbatim = path.with_file_name("verbatim.crt");
    doc.save_as(&verbatim).unwrap();
    assert_eq!(
        std::fs::read(&verbatim).unwrap(),
        original,
        "an unedited cart is copied, not re-placed"
    );

    rename(&mut doc, 0, "elsewhere");
    let target = path.with_file_name("copy.crt");
    doc.save_as(&target).unwrap();
    assert_eq!(std::fs::read(&target).unwrap(), doc.bytes());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    let dto = doc.dto();
    assert_eq!(dto.file, "copy.crt");
    assert!(!dto.dirty && !dto.untitled);

    rename(&mut doc, 1, "again");
    doc.save().unwrap();
    assert_eq!(
        std::fs::read(&target).unwrap(),
        doc.bytes(),
        "later saves go to it"
    );
    assert_eq!(std::fs::read(&path).unwrap(), original);
}

// AC-76 (RENDER): a save that cannot complete leaves the destination intact and the cart dirty.
#[cfg(unix)]
#[test]
fn ac_76_render_a_failed_save_leaves_the_file_intact_and_the_cart_dirty() {
    use std::os::unix::fs::PermissionsExt;
    let path = copy("ac76", "two.crt");
    let dir = path.parent().unwrap().to_path_buf();
    let original = std::fs::read(&path).unwrap();
    let mut doc = Document::open(&path).unwrap();
    rename(&mut doc, 0, "doomed");

    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o555)).unwrap();
    let result = doc.save();
    let leftovers: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    make_writable(&dir);

    let err = result.expect_err("a read-only directory refuses the save");
    assert!(!err.is_empty());
    assert_eq!(std::fs::read(&path).unwrap(), original);
    assert_eq!(
        leftovers,
        [std::ffi::OsString::from("two.crt")],
        "no temp file left"
    );
    assert!(doc.dto().dirty);
}
