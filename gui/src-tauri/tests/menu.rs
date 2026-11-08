//! custom-menu in the window: the listing says which menu a cart carries (AC-85), and a cart with
//! our menu opens editable and saves what `edit` writes (AC-86). menu-picker: switching the menu is
//! an edit that saves what `edit --menu` writes (AC-112), through the same refusals (AC-113).
//! menu-highmem: a program reaching into `$D000-$FFFF` from below no longer refuses it (AC-164).

use std::path::{Path, PathBuf};

use stidcart_gui::document::{Document, Edit};
use stidcart_gui::open;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn fx(name: &str) -> String {
    fixtures().join(name).to_string_lossy().into_owned()
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("stidcart-gui-menu-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn s(v: &str) -> String {
    v.to_string()
}

fn path(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

fn run(args: &[String]) {
    stidcart_cli::run(args).unwrap_or_else(|e| panic!("stidcart {args:?}: {e}"));
}

/// Two programs with our menu, built by the CLI.
fn ours(dir: &Path) -> PathBuf {
    let cart = dir.join("ours.crt");
    run(&[
        s("build"),
        s("--menu"),
        s("stidcart"),
        s("-e"),
        s("ours"),
        s("-o"),
        path(&cart),
        fx("hello.prg"),
        fx("ramp.prg"),
    ]);
    cart
}

// AC-85 (RENDER): the listing the window receives says which menu the cart carries
#[test]
fn ac_85_render_the_listing_dto_carries_the_menu() {
    let dir = scratch("ac85");
    let theirs = serde_json::to_value(open(&fixtures().join("two.crt")).unwrap()).unwrap();
    assert_eq!(theirs["menu"], "EasyLoader");

    let mine = serde_json::to_value(open(&ours(&dir)).unwrap()).unwrap();
    assert_eq!(mine["menu"], "stidcart");
}

// AC-86 (RENDER): a cart with our menu opens editable, and saving after edits writes edit's bytes
#[test]
fn ac_86_render_our_menu_opens_editable_and_saves_what_edit_writes() {
    let dir = scratch("ac86");
    let cart = ours(&dir);

    let mut doc = Document::open(&cart).unwrap();
    let dto = doc.dto();
    assert_eq!(dto.read_only, None, "a cart with our menu opened view-only");
    assert_eq!(
        serde_json::to_value(&dto.listing).unwrap()["menu"],
        "stidcart"
    );

    doc.apply(Edit::Rename {
        index: 0,
        name: s("renamed"),
    })
    .unwrap();
    doc.apply(Edit::SetHidden {
        index: 1,
        hidden: true,
    })
    .unwrap();
    let saved = dir.join("saved.crt");
    doc.save_as(&saved).unwrap();

    let step = dir.join("step.crt");
    let expected = dir.join("expected.crt");
    run(&[
        s("edit"),
        path(&cart),
        s("--rename"),
        s("0"),
        s("renamed"),
        s("-o"),
        path(&step),
    ]);
    run(&[
        s("edit"),
        path(&step),
        s("--hide"),
        s("1"),
        s("-o"),
        path(&expected),
    ]);
    assert_eq!(
        std::fs::read(&saved).unwrap(),
        std::fs::read(&expected).unwrap(),
        "the window's save and edit disagree on a cart with our menu"
    );
}

/// The menu the listing the window receives names.
fn menu_of(doc: &Document) -> String {
    serde_json::to_value(&doc.dto().listing).unwrap()["menu"]
        .as_str()
        .unwrap()
        .to_string()
}

// AC-112 (RENDER): switching the menu is one edit, undoable, saved as `edit --menu` writes it
#[test]
fn ac_112_render_switching_the_menu_is_an_undoable_edit_saving_what_edit_writes() {
    let dir = scratch("ac112");

    let mut doc = Document::open(&fixtures().join("two.crt")).unwrap();
    assert_eq!(menu_of(&doc), "EasyLoader");
    doc.apply(Edit::SetMenu {
        menu: s("stidcart"),
    })
    .unwrap();
    assert_eq!(menu_of(&doc), "stidcart");
    let dto = doc.dto();
    assert!(
        dto.dirty,
        "switching the menu did not mark the cart unsaved"
    );
    assert!(dto.can_undo, "switching the menu left nothing to undo");

    let saved = dir.join("saved.crt");
    doc.save_as(&saved).unwrap();
    let expected = dir.join("expected.crt");
    run(&[
        s("edit"),
        fx("two.crt"),
        s("--menu"),
        s("stidcart"),
        s("-o"),
        path(&expected),
    ]);
    assert_eq!(
        std::fs::read(&saved).unwrap(),
        std::fs::read(&expected).unwrap(),
        "the window's switch to stidcart and edit --menu disagree"
    );

    doc.undo();
    assert_eq!(
        menu_of(&doc),
        "EasyLoader",
        "undo did not give back EasyLoader"
    );

    // And back the other way, by the name the listing shows.
    let ours = ours(&dir);
    let mut back = Document::open(&ours).unwrap();
    back.apply(Edit::SetMenu {
        menu: s("EasyLoader"),
    })
    .unwrap();
    assert_eq!(menu_of(&back), "EasyLoader");
    let saved = dir.join("back.crt");
    back.save_as(&saved).unwrap();
    let expected = dir.join("back-expected.crt");
    run(&[
        s("edit"),
        path(&ours),
        s("--menu"),
        s("easyloader"),
        s("-o"),
        path(&expected),
    ]);
    assert_eq!(
        std::fs::read(&saved).unwrap(),
        std::fs::read(&expected).unwrap(),
        "the window's switch to EasyLoader and edit --menu disagree"
    );
}

/// Apply `edit` and assert it is refused with text containing every one of `words`, leaving the
/// document exactly as it was.
fn assert_refused(doc: &mut Document, edit: Edit, words: &[&str], what: &str) {
    let (bytes, dirty, can_undo) = (doc.bytes().to_vec(), doc.dto().dirty, doc.dto().can_undo);
    let err = doc
        .apply(edit)
        .expect_err(&format!("{what}: stidcart was not refused"));
    for w in words {
        assert!(
            err.contains(w),
            "{what}: the refusal does not say '{w}': {err}"
        );
    }
    assert_eq!(
        doc.bytes(),
        bytes,
        "{what}: the refused switch changed the cart"
    );
    let dto = doc.dto();
    assert_eq!(
        (dto.dirty, dto.can_undo),
        (dirty, can_undo),
        "{what}: the refused switch became a step"
    );
}

// AC-113 (RENDER): a cart our menu cannot run refuses it with edit's reason, and stays unchanged
#[test]
fn ac_113_render_a_cart_our_menu_cannot_run_refuses_it_unchanged() {
    let dir = scratch("ac113");

    // An Ocean entry: the refusal is the one edit --menu prints.
    let mut ocean = Document::open(&fixtures().join("cocean.crt")).unwrap();
    let cli = stidcart_cli::run(&[
        s("edit"),
        fx("cocean.crt"),
        s("--menu"),
        s("stidcart"),
        s("-o"),
        path(&dir.join("never.crt")),
    ])
    .expect_err("edit --menu stidcart accepted an Ocean cart");
    assert!(!dir.join("never.crt").exists());
    assert_refused(
        &mut ocean,
        Edit::SetMenu {
            menu: s("stidcart"),
        },
        &[&cli],
        "an Ocean cart",
    );
    assert_eq!(menu_of(&ocean), "EasyLoader");

    // A program that loads into $D000-$DFFF, added in the window first.
    let mut doc = with_program(&dir, "upper", 0xd000, 0x200);
    assert_refused(
        &mut doc,
        Edit::SetMenu {
            menu: s("stidcart"),
        },
        &["upper", "$D000"],
        "a program loading at $D000",
    );
    assert_eq!(menu_of(&doc), "EasyLoader");
}

/// `<name>.prg` in `dir`, loading at `load` with `len` bytes, as the CLI reads it.
fn write_prg(dir: &Path, name: &str, load: u16, len: usize) -> PathBuf {
    let prg = dir.join(format!("{name}.prg"));
    let mut bytes = load.to_le_bytes().to_vec();
    bytes.resize(2 + len, 0xea);
    std::fs::write(&prg, bytes).unwrap();
    prg
}

/// A new document holding one program, added in the window.
fn with_program(dir: &Path, name: &str, load: u16, len: usize) -> Document {
    let prg = write_prg(dir, name, load, len);
    let mut doc = Document::new();
    doc.apply(Edit::Add {
        paths: vec![path(&prg)],
        aligned: false,
    })
    .unwrap();
    doc
}

/// AC-164 (RENDER): choosing stidcart for a cart holding a program the CLI still refuses (loading
/// at `$D000` or above, or ending past `$FFFF`) shows that refusal and changes nothing; a cart whose
/// only reason was a program reaching into `$D000-$FFFF` from below now takes stidcart.
#[test]
fn ac_164_render_only_what_our_menu_still_cannot_start_refuses_it() {
    let dir = scratch("ac164");

    // Still refused, with the reason edit --menu prints for the same program.
    for (name, load, len) in [
        ("atkernal", 0xe000u16, 0x10usize),
        ("pasttop", 0xc000, 0x4001),
    ] {
        let prg = write_prg(&dir, name, load, len);
        let cart = dir.join(format!("{name}.crt"));
        run(&[s("build"), s("-o"), path(&cart), path(&prg)]);
        let cli = stidcart_cli::run(&[
            s("edit"),
            path(&cart),
            s("--menu"),
            s("stidcart"),
            s("-o"),
            path(&dir.join("never.crt")),
        ])
        .expect_err("edit --menu stidcart accepted a program our menu cannot start");
        assert!(!dir.join("never.crt").exists());
        let mut doc = Document::open(&cart).unwrap();
        assert_refused(
            &mut doc,
            Edit::SetMenu {
                menu: s("stidcart"),
            },
            &[name, &cli],
            name,
        );
        assert_eq!(menu_of(&doc), "EasyLoader");
    }

    // Reaching $D000-$FFFF from below is no longer a reason: stidcart is taken.
    for (name, load, len) in [("iozone", 0xcf00u16, 0x200usize), ("attop", 0xc000, 0x4000)] {
        let mut doc = with_program(&dir, name, load, len);
        doc.apply(Edit::SetMenu {
            menu: s("stidcart"),
        })
        .unwrap_or_else(|e| panic!("{name}: stidcart was refused: {e}"));
        assert_eq!(menu_of(&doc), "stidcart", "{name}");
    }
}

/// Two programs with our menu, then the 5b54e81 build in the loader slot (bank 0 ROML), as a cart
/// made by stidcart 0.15 carries it.
fn older(dir: &Path) -> PathBuf {
    let built = std::fs::read(ours(dir)).unwrap();
    let mut container = stidcart_core::crt::parse(&built).unwrap();
    container
        .image
        .write_at(0, include_bytes!("../../../menu/past/5b54e81.bin"));
    let cart = dir.join("older.crt");
    std::fs::write(
        &cart,
        stidcart_core::crt::write(&container.image, &container.header_name),
    )
    .unwrap();
    cart
}

// AC-130 (RENDER): an earlier build opens editable, and choosing stidcart writes the current build
#[test]
fn ac_130_render_an_older_build_opens_editable_and_upgrades_as_an_edit() {
    let dir = scratch("ac130");
    let mut doc = Document::open(&older(&dir)).unwrap();
    assert_eq!(doc.dto().read_only, None, "an older build opened view-only");
    assert_eq!(menu_of(&doc), "stidcart (older build)");

    doc.apply(Edit::SetMenu {
        menu: s("stidcart"),
    })
    .unwrap();
    assert_eq!(menu_of(&doc), "stidcart");
    let image = stidcart_core::crt::parse(doc.bytes()).unwrap().image;
    assert_eq!(
        &image.as_bytes()[..0x2000],
        stidcart_core::menu::STIDCART_MENU
    );
    assert!(doc.dto().dirty && doc.dto().can_undo);

    doc.undo();
    assert_eq!(menu_of(&doc), "stidcart (older build)");
}

/// Two programs with EasyLoader, then a loader slot this tool doesn't know and `patch` on the image.
fn foreign(dir: &Path, patch: impl Fn(&mut stidcart_core::FlashImage)) -> PathBuf {
    let built = dir.join("built.crt");
    run(&[
        s("build"),
        s("-e"),
        s("foreign"),
        s("-o"),
        path(&built),
        fx("hello.prg"),
        fx("ramp.prg"),
    ]);
    let mut container = stidcart_core::crt::parse(&std::fs::read(&built).unwrap()).unwrap();
    let loader: Vec<u8> = (0..0x2000).map(|i| (i * 13 + 5) as u8).collect();
    container.image.write_at(0, &loader);
    patch(&mut container.image);
    let cart = dir.join("foreign.crt");
    std::fs::write(
        &cart,
        stidcart_core::crt::write(&container.image, &container.header_name),
    )
    .unwrap();
    cart
}

// AC-134 (RENDER): an unknown loader opens view-only; choosing a menu is an undoable edit that
// makes the cart editable
#[test]
fn ac_134_render_choosing_a_menu_replaces_an_unknown_loader() {
    let dir = scratch("ac134");
    let mut doc = Document::open(&foreign(&dir, |_| {})).unwrap();
    let dto = doc.dto();
    assert!(
        dto.read_only
            .as_deref()
            .is_some_and(|r| r.contains("menu loader")),
        "{:?}",
        dto.read_only
    );
    assert!(dto.menu_replaceable);
    assert_eq!(menu_of(&doc), "unrecognised");
    assert!(
        doc.apply(Edit::Rename {
            index: 0,
            name: s("x")
        })
        .is_err(),
        "a view-only cart took an edit other than the menu"
    );

    doc.apply(Edit::SetMenu {
        menu: s("stidcart"),
    })
    .unwrap();
    let dto = doc.dto();
    assert_eq!(
        dto.read_only, None,
        "the cart stayed view-only after the switch"
    );
    assert!(!dto.menu_replaceable && dto.dirty && dto.can_undo);
    assert_eq!(menu_of(&doc), "stidcart");
    doc.apply(Edit::Rename {
        index: 0,
        name: s("renamed"),
    })
    .unwrap();
    let saved = dir.join("saved.crt");
    doc.save_as(&saved).unwrap();
    let listing = stidcart_core::listing::describe(&std::fs::read(&saved).unwrap()).unwrap();
    assert_eq!(listing.menu, "stidcart");

    doc.undo();
    doc.undo();
    let dto = doc.dto();
    assert!(
        dto.read_only.is_some(),
        "undoing the switch left the cart editable"
    );
    assert!(dto.menu_replaceable);
    assert_eq!(menu_of(&doc), "unrecognised");
    assert!(doc.save_as(&dir.join("refused.crt")).is_err());
}

// AC-135 (RENDER): a cart refused for another reason offers no menu switch
#[test]
fn ac_135_render_other_refusals_offer_no_menu_switch() {
    let dir = scratch("ac135");
    let at = stidcart_core::reserved::EAPI_OFFSET;
    let cart = foreign(&dir, |img| {
        let byte = img.as_bytes()[at];
        img.write_at(at, &[byte ^ 0xff]);
    });
    let mut doc = Document::open(&cart).unwrap();
    let dto = doc.dto();
    assert!(dto.read_only.is_some());
    assert!(!dto.menu_replaceable);
    assert!(doc
        .apply(Edit::SetMenu {
            menu: s("stidcart")
        })
        .is_err());
}
