//! folder-browser AC-185..AC-189 (CLI): the cart's own folder list from the command line:
//! `--new-folder`, `--remove-folder`, `--move-folder`, folders that stay when they empty, and
//! `list`'s `folders:` line (D-063).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use stidcart_core::meta::Field;
use stidcart_core::read;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("stidcart-folder-browser-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn stidcart<S: AsRef<std::ffi::OsStr>>(args: &[S]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_stidcart"))
        .args(args)
        .output()
        .unwrap()
}

fn ok(out: &Output) -> String {
    assert!(
        out.status.success(),
        "stidcart failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn refused(out: &Output) -> String {
    assert!(!out.status.success(), "expected a refusal");
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// A cart of `hello.prg` copies, one entry per name, in `dir`.
fn cart(dir: &Path, names: &[&str]) -> PathBuf {
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let path = dir.join("cart.crt");
    let mut args = vec![
        "build".to_string(),
        "-e".into(),
        "folders".into(),
        "-o".into(),
        path.to_string_lossy().into_owned(),
    ];
    for (i, n) in names.iter().enumerate() {
        let sub = dir.join(format!("in{i}"));
        std::fs::create_dir_all(&sub).unwrap();
        let p = sub.join(format!("{n}.prg"));
        std::fs::write(&p, &hello).unwrap();
        args.push(p.to_string_lossy().into_owned());
    }
    ok(&stidcart(&args));
    path
}

fn edit(cart: &Path, args: &[&str]) -> Output {
    let mut all = vec!["edit".to_string(), cart.to_string_lossy().into_owned()];
    all.extend(args.iter().map(|a| a.to_string()));
    stidcart(&all)
}

fn folders(cart: &Path) -> Vec<String> {
    read::load(&std::fs::read(cart).unwrap())
        .unwrap()
        .folders()
        .to_vec()
}

fn folder_of(cart: &Path, i: usize) -> String {
    let b = read::load(&std::fs::read(cart).unwrap()).unwrap();
    b.entries()[i].meta().get(Field::Folder).to_string()
}

fn list(cart: &Path) -> String {
    ok(&stidcart(&[
        "list".to_string(),
        cart.to_string_lossy().into_owned(),
    ]))
}

// AC-185 (CLI): --new-folder adds an empty folder after the others; a bad or taken name is refused
#[test]
fn ac_185_cli_new_folder_adds_an_empty_folder() {
    let dir = scratch("new");
    let c = cart(&dir, &["alpha", "bravo"]);
    ok(&edit(&c, &["--folder", "0", "Games"]));
    ok(&edit(&c, &["--new-folder", "Utilities"]));
    assert_eq!(folders(&c), ["Games", "Utilities"]);
    ok(&edit(&c, &["--rename", "1", "bravo2"]));
    assert_eq!(
        folders(&c),
        ["Games", "Utilities"],
        "the empty folder survives a save"
    );

    let before = std::fs::read(&c).unwrap();
    for (name, says) in [
        ("", "name"),
        (&"u".repeat(17) as &str, "16"),
        ("Games", "Games"),
    ] {
        let err = refused(&edit(&c, &["--new-folder", name]));
        assert!(err.contains(says), "{name:?}: {err}");
        assert_eq!(std::fs::read(&c).unwrap(), before, "nothing written");
    }
}

// AC-186 (CLI): --remove-folder removes it and leaves its entries in the cart with no folder
#[test]
fn ac_186_cli_remove_folder_keeps_its_entries() {
    let dir = scratch("remove");
    let c = cart(&dir, &["alpha", "bravo"]);
    ok(&edit(
        &c,
        &["--folder", "0", "Games", "--folder", "1", "Games"],
    ));
    ok(&edit(&c, &["--remove-folder", "Games"]));
    assert!(folders(&c).is_empty());
    assert_eq!(folder_of(&c, 0), "");
    assert_eq!(folder_of(&c, 1), "");
    assert_eq!(
        read::load(&std::fs::read(&c).unwrap())
            .unwrap()
            .entries()
            .len(),
        2
    );

    let before = std::fs::read(&c).unwrap();
    let err = refused(&edit(&c, &["--remove-folder", "Nope"]));
    assert!(err.contains("Nope"), "{err}");
    assert_eq!(std::fs::read(&c).unwrap(), before);
}

// AC-187 (CLI): --move-folder puts a folder at a position of the cart's folder order
#[test]
fn ac_187_cli_move_folder_reorders() {
    let dir = scratch("move");
    let c = cart(&dir, &["alpha"]);
    ok(&edit(
        &c,
        &[
            "--new-folder",
            "A",
            "--new-folder",
            "B",
            "--new-folder",
            "C",
        ],
    ));
    ok(&edit(&c, &["--move-folder", "C", "0"]));
    assert_eq!(folders(&c), ["C", "A", "B"]);

    let before = std::fs::read(&c).unwrap();
    for args in [
        vec!["--move-folder", "Z", "0"],
        vec!["--move-folder", "A", "3"],
    ] {
        refused(&edit(&c, &args));
        assert_eq!(std::fs::read(&c).unwrap(), before, "{args:?}");
    }
}

// AC-188 (CLI): a folder stays when its last entry leaves; --folder adds a folder the cart lacks
#[test]
fn ac_188_cli_folders_stay_when_emptied() {
    let dir = scratch("stay");
    let c = cart(&dir, &["alpha", "bravo", "charlie"]);
    ok(&edit(
        &c,
        &[
            "--folder", "0", "Games", "--folder", "1", "Demos", "--folder", "2", "Tools",
        ],
    ));
    assert_eq!(folders(&c), ["Games", "Demos", "Tools"]);

    ok(&edit(&c, &["--folder", "0", ""]));
    ok(&edit(&c, &["--folder", "1", "Tools"]));
    ok(&edit(&c, &["--remove", "2"]));
    assert_eq!(folders(&c), ["Games", "Demos", "Tools"], "all three stay");

    let toml = dir.join("meta.toml");
    std::fs::write(&toml, "[[entry]]\nname = \"alpha\"\nfolder = \"Extras\"\n").unwrap();
    ok(&edit(&c, &["--import", &toml.to_string_lossy()]));
    assert_eq!(
        folders(&c),
        ["Games", "Demos", "Tools", "Extras"],
        "the import adds it after the others"
    );
}

// AC-189 (CLI): list prints the folders in order with each one's entry count, empty ones included
#[test]
fn ac_189_cli_list_prints_the_folders() {
    let dir = scratch("list");
    let c = cart(&dir, &["alpha", "bravo", "charlie"]);
    assert!(
        !list(&c).contains("folders:"),
        "a cart without folders prints no line"
    );
    ok(&edit(
        &c,
        &[
            "--folder", "0", "Arcade", "--folder", "1", "Arcade", "--folder", "2", "Demos",
        ],
    ));
    ok(&edit(&c, &["--new-folder", "Utilities"]));
    let out = list(&c);
    assert!(
        out.contains("folders: Arcade (2)  Demos (1)  Utilities (0)"),
        "{out}"
    );
}
