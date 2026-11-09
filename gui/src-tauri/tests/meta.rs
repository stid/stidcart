//! menu-meta, the window's side of AC-158 and AC-159: the listing it receives carries each
//! entry's metadata, and editing a field is one undoable step that saves what `edit` writes.
//! Refusals are compared with the real `edit`'s text through the CLI's library entry point.

use std::path::{Path, PathBuf};

use stidcart_gui::document::{Document, Edit};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("stidcart-gui-meta-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn s(v: &str) -> String {
    v.to_string()
}

/// `stidcart edit <cart> <args…> -o <out>`, through the CLI's library entry point.
fn cli_edit(cart: &Path, args: &[&str], out: &Path) -> Result<Option<String>, String> {
    let mut all = vec![s("edit"), cart.to_string_lossy().into_owned()];
    all.extend(args.iter().map(|a| s(a)));
    all.extend([s("-o"), out.to_string_lossy().into_owned()]);
    stidcart_cli::run(&all)
}

/// A copy of `two.crt` with a title and folder on entry 0, set by `edit`.
fn annotated(tag: &str) -> PathBuf {
    let dir = scratch(tag);
    let out = dir.join("meta.crt");
    cli_edit(
        &fixtures().join("two.crt"),
        &["--title", "0", "Hello There", "--group", "0", "Games"],
        &out,
    )
    .unwrap();
    out
}

fn set(index: usize, field: &str, value: &str) -> Edit {
    Edit::SetMeta {
        index,
        field: s(field),
        value: s(value),
    }
}

// AC-158 (RENDER): the window receives each entry's metadata with its row
#[test]
fn ac_158_the_listing_carries_each_entrys_metadata() {
    let doc = Document::open(&annotated("listing")).unwrap();
    let json = serde_json::to_value(doc.dto()).unwrap();
    let rows = &json["listing"]["rows"];
    assert_eq!(
        rows.as_array().unwrap().len(),
        2,
        "the metadata entry is not a row"
    );
    assert_eq!(rows[0]["title"], "Hello There");
    assert_eq!(rows[0]["folder"], "Games");
    for f in ["year", "author", "note"] {
        assert_eq!(rows[0][f], "", "{f}");
    }
    assert_eq!(rows[1]["title"], "");
    assert_eq!(json["listing"]["metaDropped"], 0);
    assert!(json["listing"]["metaKept"].is_null());
}

// AC-159 (RENDER): each field change is one undoable step
#[test]
fn ac_159_a_field_change_is_one_undoable_step() {
    let dir = scratch("undo");
    let cart = dir.join("two.crt");
    std::fs::copy(fixtures().join("two.crt"), &cart).unwrap();
    let mut doc = Document::open(&cart).unwrap();
    let title = |d: &Document| d.dto().listing.rows[1].title.clone();

    for (field, value) in [
        ("title", "Second"),
        ("year", "1985"),
        ("author", "Me"),
        ("note", "n"),
        ("folder", "Demos"),
    ] {
        doc.apply(set(1, field, value)).unwrap();
    }
    let dto = doc.dto();
    let row = &dto.listing.rows[1];
    assert_eq!(
        [&row.title, &row.year, &row.author, &row.note, &row.folder],
        ["Second", "1985", "Me", "n", "Demos"]
    );
    assert!(dto.dirty);

    for _ in 0..4 {
        assert!(doc.undo());
    }
    assert_eq!(title(&doc), "Second", "four undos leave the first change");
    assert!(doc.undo());
    assert_eq!(title(&doc), "", "five undos take back all five");
    assert!(!doc.dto().dirty);
    assert!(doc.redo());
    assert_eq!(title(&doc), "Second");
}

// AC-159 (RENDER): an invalid value shows edit's reason and changes nothing
#[test]
fn ac_159_an_invalid_value_is_refused_in_edits_words() {
    let dir = scratch("refuse");
    let cart = dir.join("two.crt");
    std::fs::copy(fixtures().join("two.crt"), &cart).unwrap();
    let mut doc = Document::open(&cart).unwrap();
    let before = doc.bytes().to_vec();

    for (field, flag, value) in [
        ("title", "--title", "T".repeat(49)),
        ("year", "--year", s("12345")),
        ("author", "--author", s("Zoë")),
    ] {
        let window = doc.apply(set(0, field, &value)).unwrap_err();
        let cli = cli_edit(&cart, &[flag, "0", &value], &dir.join("out.crt")).unwrap_err();
        assert_eq!(window, cli, "{field}: the window gives edit's reason");
        assert_eq!(doc.bytes(), before.as_slice(), "{field}: nothing changed");
        assert!(!doc.dto().can_undo, "{field}: no step was added");
    }
    assert!(
        doc.apply(set(0, "colour", "red")).is_err(),
        "an unknown field"
    );
}

// AC-159 (RENDER): saving writes exactly what edit writes for the same values
#[test]
fn ac_159_saved_metadata_is_byte_identical_to_edit() {
    let dir = scratch("parity");
    let cart = dir.join("two.crt");
    std::fs::copy(fixtures().join("two.crt"), &cart).unwrap();
    let mut doc = Document::open(&cart).unwrap();
    let steps: [(Edit, Vec<&str>); 4] = [
        (
            set(0, "title", "Hello There"),
            vec!["--title", "0", "Hello There"],
        ),
        (set(1, "folder", "Games"), vec!["--folder", "1", "Games"]),
        (set(0, "year", "1984"), vec!["--year", "0", "1984"]),
        (set(0, "title", ""), vec!["--title", "0", ""]),
    ];
    let mut prev = cart.clone();
    for (i, (edit, args)) in steps.into_iter().enumerate() {
        let next = dir.join(format!("step{i}.crt"));
        cli_edit(&prev, &args, &next).unwrap();
        doc.apply(edit).unwrap();
        assert_eq!(
            doc.bytes(),
            std::fs::read(&next).unwrap().as_slice(),
            "step {i} ({args:?}): the window and edit disagree"
        );
        prev = next;
    }
    let saved = dir.join("saved.crt");
    doc.save_as(&saved).unwrap();
    assert_eq!(
        std::fs::read(&saved).unwrap(),
        std::fs::read(&prev).unwrap()
    );
}
