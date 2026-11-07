//! custom-menu in the window: the listing says which menu a cart carries (AC-85), and a cart with
//! our menu opens editable and saves what `edit` writes (AC-86). menu-picker: switching the menu is
//! an edit that saves what `edit --menu` writes (AC-112), through the same refusals (AC-113).

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
    let prg = dir.join("upper.prg");
    let mut bytes = 0xcf00u16.to_le_bytes().to_vec();
    bytes.extend([0xeau8; 0x200]);
    std::fs::write(&prg, bytes).unwrap();
    let mut doc = Document::new();
    doc.apply(Edit::Add {
        paths: vec![path(&prg)],
        aligned: false,
    })
    .unwrap();
    assert_refused(
        &mut doc,
        Edit::SetMenu {
            menu: s("stidcart"),
        },
        &["upper", "$D000-$DFFF"],
        "a program over $D000",
    );
    assert_eq!(menu_of(&doc), "EasyLoader");
}
