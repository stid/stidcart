//! folder-editor, the Rust side of the window's folder tree: one `arrange` edit carries the whole
//! tree, saves the cart in tree order, equals `edit`'s `--folder`/`--move` result, is one undoable
//! step, and refuses folder names the way `edit` does. Edits arrive as the JSON the frontend sends.

use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use stidcart_gui::document::{Document, Edit};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("stidcart-gui-folders-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn s(v: &str) -> String {
    v.to_string()
}

fn run(args: &[String]) -> Result<Option<String>, String> {
    stidcart_cli::run(args)
}

/// `stidcart edit <cart> <args…>` in place, through the CLI's library entry point.
fn cli_edit(cart: &Path, args: &[&str]) {
    let mut all = vec![s("edit"), cart.to_string_lossy().into_owned()];
    all.extend(args.iter().map(|a| s(a)));
    run(&all).unwrap_or_else(|e| panic!("stidcart {all:?}: {e}"));
}

/// alpha, bravo, charlie, delta (copies of `hello.prg`), with bravo and delta in folder Games.
fn cart(dir: &Path) -> PathBuf {
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let path = dir.join("cart.crt");
    let mut args = vec![
        s("build"),
        s("-e"),
        s("folders"),
        s("-o"),
        path.to_string_lossy().into_owned(),
    ];
    for (i, n) in ["alpha", "bravo", "charlie", "delta"].iter().enumerate() {
        let sub = dir.join(format!("in{i}"));
        std::fs::create_dir_all(&sub).unwrap();
        let p = sub.join(format!("{n}.prg"));
        std::fs::write(&p, &hello).unwrap();
        args.push(p.to_string_lossy().into_owned());
    }
    run(&args).unwrap();
    cli_edit(&path, &["--group", "1", "Games", "--group", "3", "Games"]);
    path
}

/// The edit the frontend sends for a tree.
fn arrange(folders: Value, loose: Value) -> Edit {
    serde_json::from_value(json!({ "kind": "arrange", "folders": folders, "loose": loose }))
        .expect("the window's arrange edit deserializes")
}

/// Each row's name and folder, in directory order.
fn rows(doc: &Document) -> Vec<(String, String)> {
    let json = serde_json::to_value(doc.dto()).unwrap();
    json["listing"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["name"].as_str().unwrap().to_string(),
                r["folder"]
                    .as_str()
                    .unwrap_or("<no folder field>")
                    .to_string(),
            )
        })
        .collect()
}

fn pairs(v: &[(&str, &str)]) -> Vec<(String, String)> {
    v.iter().map(|(a, b)| (s(a), s(b))).collect()
}

// AC-180 (RENDER): an arrange saves the cart in tree order, byte for byte what edit's --folder and --move write
#[test]
fn ac_180_arrange_saves_tree_order_as_edit_does() {
    let dir = scratch("parity");
    let c = cart(&dir);
    let mut doc = Document::open(&c).unwrap();

    // Games holds delta then bravo, a new folder Demos holds charlie, alpha stays loose.
    doc.apply(arrange(
        json!([{ "name": "Games", "entries": [3, 1] }, { "name": "Demos", "entries": [2] }]),
        json!([0]),
    ))
    .unwrap();
    assert_eq!(
        rows(&doc),
        pairs(&[
            ("delta", "Games"),
            ("bravo", "Games"),
            ("charlie", "Demos"),
            ("alpha", ""),
        ]),
        "each folder's entries together, folders in tree order, then the loose entries"
    );
    let saved = dir.join("window.crt");
    doc.save_as(&saved).unwrap();

    // The same through edit: the folder, then one move per position.
    let oracle = dir.join("oracle.crt");
    std::fs::copy(&c, &oracle).unwrap();
    cli_edit(&oracle, &["--folder", "2", "Demos"]);
    cli_edit(&oracle, &["--move", "3", "0"]); // delta alpha bravo charlie
    cli_edit(&oracle, &["--move", "2", "1"]); // delta bravo alpha charlie
    cli_edit(&oracle, &["--move", "3", "2"]); // delta bravo charlie alpha
    assert_eq!(
        std::fs::read(&saved).unwrap(),
        std::fs::read(&oracle).unwrap(),
        "the window saves what edit writes"
    );
}

// AC-179 (RENDER): a tree change is one undoable edit
#[test]
fn ac_179_an_arrange_is_one_undoable_step() {
    let dir = scratch("undo");
    let c = cart(&dir);
    let mut doc = Document::open(&c).unwrap();
    let before = doc.bytes().to_vec();

    // charlie dragged into Games, after delta.
    doc.apply(arrange(
        json!([{ "name": "Games", "entries": [1, 3, 2] }]),
        json!([0]),
    ))
    .unwrap();
    let dto = doc.dto();
    assert!(dto.dirty && dto.can_undo);
    assert_eq!(
        rows(&doc),
        pairs(&[
            ("bravo", "Games"),
            ("delta", "Games"),
            ("charlie", "Games"),
            ("alpha", ""),
        ])
    );

    assert!(doc.undo());
    assert_eq!(doc.bytes(), &before[..], "one undo takes it all back");
    assert!(!doc.dto().dirty);
}

/// The text `edit` refuses a value with, through the CLI's library entry point.
fn edit_refusal(cart: &Path, args: &[&str]) -> String {
    let mut all = vec![s("edit"), cart.to_string_lossy().into_owned()];
    all.extend(args.iter().map(|a| s(a)));
    all.extend([
        s("-o"),
        cart.with_extension("out").to_string_lossy().into_owned(),
    ]);
    run(&all).expect_err("edit refuses it")
}

// AC-183 (RENDER): an empty, invalid or taken folder name is refused, and nothing changes
#[test]
fn ac_183_bad_folder_names_are_refused() {
    let dir = scratch("names");
    let c = cart(&dir);
    let mut doc = Document::open(&c).unwrap();
    let before = doc.bytes().to_vec();

    let long = "g".repeat(17);
    let err = doc
        .apply(arrange(
            json!([{ "name": long, "entries": [1, 3] }]),
            json!([0, 2]),
        ))
        .expect_err("a 17-character folder is refused");
    assert_eq!(
        err,
        edit_refusal(&c, &["--folder", "1", &long]),
        "in edit's words"
    );

    let err = doc
        .apply(arrange(
            json!([{ "name": "", "entries": [1, 3] }]),
            json!([0, 2]),
        ))
        .expect_err("an empty folder name is refused");
    assert!(err.contains("name"), "{err}");

    // Renaming Games to a name another folder has.
    let err = doc
        .apply(arrange(
            json!([{ "name": "Demos", "entries": [1, 3] }, { "name": "Demos", "entries": [2] }]),
            json!([0]),
        ))
        .expect_err("two folders with one name are refused");
    assert!(
        err.contains("a folder named 'Demos' already exists"),
        "{err}"
    );

    // An arrangement that loses or repeats an entry.
    for (folders, loose) in [
        (json!([{ "name": "Games", "entries": [1, 3] }]), json!([0])),
        (
            json!([{ "name": "Games", "entries": [1, 3] }]),
            json!([0, 2, 2]),
        ),
    ] {
        assert!(doc.apply(arrange(folders, loose)).is_err());
    }

    assert_eq!(doc.bytes(), &before[..], "nothing changed");
    assert!(!doc.dto().can_undo, "no step was added");
}

// AC-182 (RENDER): moving an entry to a new folder named like an existing one is refused
#[test]
fn ac_182_a_new_folder_may_not_take_an_existing_name() {
    let dir = scratch("new");
    let c = cart(&dir);
    let mut doc = Document::open(&c).unwrap();
    let before = doc.bytes().to_vec();

    // charlie to "New folder… Games" while Games exists.
    let err = doc
        .apply(arrange(
            json!([{ "name": "Games", "entries": [1, 3] }, { "name": "Games", "entries": [2] }]),
            json!([0]),
        ))
        .expect_err("refused");
    assert!(
        err.contains("a folder named 'Games' already exists"),
        "{err}"
    );
    assert_eq!(doc.bytes(), &before[..]);

    // A new name is a new folder after the others.
    doc.apply(arrange(
        json!([{ "name": "Games", "entries": [1, 3] }, { "name": "Tools", "entries": [2] }]),
        json!([0]),
    ))
    .unwrap();
    assert_eq!(
        rows(&doc),
        pairs(&[
            ("bravo", "Games"),
            ("delta", "Games"),
            ("charlie", "Tools"),
            ("alpha", ""),
        ])
    );
}

// AC-184 (RENDER): the panel's Folder field sets the folder and moves no entry
#[test]
fn ac_184_a_folder_field_edit_moves_nothing() {
    let dir = scratch("field");
    let c = cart(&dir);
    let mut doc = Document::open(&c).unwrap();

    doc.apply(Edit::SetMeta {
        index: 0,
        field: s("folder"),
        value: s("Games"),
    })
    .unwrap();
    assert_eq!(
        rows(&doc),
        pairs(&[
            ("alpha", "Games"),
            ("bravo", "Games"),
            ("charlie", ""),
            ("delta", "Games"),
        ]),
        "the directory order is as it was"
    );
}
