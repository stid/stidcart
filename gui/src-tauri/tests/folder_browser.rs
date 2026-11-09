//! folder-browser, the Rust side of the window's stable folders (D-063): the listing carries the
//! cart's folders in order, empty ones included, and one `arrange` edit that adds, removes, renames
//! or re-orders folders saves what `edit`'s folder flags write.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use stidcart_gui::document::{Document, Edit};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("stidcart-gui-folder-browser-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn s(v: &str) -> String {
    v.to_string()
}

fn cli_edit(cart: &Path, args: &[&str]) {
    let mut all = vec![s("edit"), cart.to_string_lossy().into_owned()];
    all.extend(args.iter().map(|a| s(a)));
    stidcart_cli::run(&all).unwrap_or_else(|e| panic!("stidcart {all:?}: {e}"));
}

/// alpha, bravo, charlie; alpha and bravo in Games, an empty folder Spare after it.
fn cart(dir: &Path) -> PathBuf {
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let path = dir.join("cart.crt");
    let mut args = vec![
        s("build"),
        s("-e"),
        s("browser"),
        s("-o"),
        path.to_string_lossy().into_owned(),
    ];
    for (i, n) in ["alpha", "bravo", "charlie"].iter().enumerate() {
        let sub = dir.join(format!("in{i}"));
        std::fs::create_dir_all(&sub).unwrap();
        let p = sub.join(format!("{n}.prg"));
        std::fs::write(&p, &hello).unwrap();
        args.push(p.to_string_lossy().into_owned());
    }
    stidcart_cli::run(&args).unwrap();
    cli_edit(
        &path,
        &[
            "--folder",
            "0",
            "Games",
            "--folder",
            "1",
            "Games",
            "--new-folder",
            "Spare",
        ],
    );
    path
}

fn arrange(folders: Value, loose: Value) -> Edit {
    serde_json::from_value(json!({ "kind": "arrange", "folders": folders, "loose": loose }))
        .expect("the window's arrange edit deserializes")
}

fn listing(doc: &Document) -> Value {
    serde_json::to_value(doc.dto()).unwrap()["listing"].clone()
}

// AC-189 (RENDER): the window receives the cart's folders in order, empty ones included
#[test]
fn ac_189_the_listing_carries_every_folder_in_order() {
    let dir = scratch("dto");
    let doc = Document::open(&cart(&dir)).unwrap();
    assert_eq!(listing(&doc)["folders"], json!(["Games", "Spare"]));
}

// AC-196 (RENDER): adding, removing, renaming and re-ordering folders in one arrange saves what edit writes
#[test]
fn ac_196_folder_changes_save_as_edit_does() {
    let dir = scratch("parity");
    let c = cart(&dir);
    let mut doc = Document::open(&c).unwrap();

    // New empty "Tools" first, "Spare" deleted, charlie into Games; alpha and bravo stay.
    doc.apply(arrange(
        json!([{ "name": "Tools", "entries": [] }, { "name": "Games", "entries": [0, 1, 2] }]),
        json!([]),
    ))
    .unwrap();
    assert_eq!(listing(&doc)["folders"], json!(["Tools", "Games"]));
    let saved = dir.join("window.crt");
    doc.save_as(&saved).unwrap();

    let oracle = dir.join("oracle.crt");
    std::fs::copy(&c, &oracle).unwrap();
    cli_edit(
        &oracle,
        &[
            "--new-folder",
            "Tools",
            "--move-folder",
            "Tools",
            "0",
            "--remove-folder",
            "Spare",
            "--folder",
            "2",
            "Games",
        ],
    );
    assert_eq!(
        std::fs::read(&saved).unwrap(),
        std::fs::read(&oracle).unwrap(),
        "the window saves what edit writes"
    );

    // A folder survives its last entry moving out, and an arrange may leave it empty.
    doc.apply(arrange(
        json!([{ "name": "Tools", "entries": [] }, { "name": "Games", "entries": [] }]),
        json!([0, 1, 2]),
    ))
    .unwrap();
    assert_eq!(listing(&doc)["folders"], json!(["Tools", "Games"]));
    assert!(doc.undo());
    assert_eq!(listing(&doc)["folders"], json!(["Tools", "Games"]));
}
