//! custom-menu in the window: the listing says which menu a cart carries (AC-85), and a cart with
//! our menu opens editable and saves what `edit` writes (AC-86).

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
