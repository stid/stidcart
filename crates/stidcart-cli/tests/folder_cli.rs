//! folder-editor AC-173..AC-177 (CLI): the group field is a folder (`--folder`, the import key, the
//! `list` label, `--group` and `group` still accepted), and `edit --move` re-orders an entry.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use stidcart_core::meta::Field;
use stidcart_core::read;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("stidcart-folder-cli-{tag}"));
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

/// `stidcart edit <cart> <args…>`.
fn edit(cart: &Path, args: &[&str]) -> Output {
    let mut all = vec!["edit".to_string(), cart.to_string_lossy().into_owned()];
    all.extend(args.iter().map(|a| a.to_string()));
    stidcart(&all)
}

/// `stidcart edit <cart> <args…> -o <out>`, leaving `cart` as it was.
fn edit_to(cart: &Path, args: &[&str], out: &Path) -> Output {
    let mut all: Vec<&str> = args.to_vec();
    let out = out.to_string_lossy().into_owned();
    all.extend(["-o", out.as_str()]);
    edit(cart, &all)
}

/// The metadata field the window and `edit` call "folder".
fn folder_field() -> Field {
    Field::from_name("folder").expect("the metadata field is named folder")
}

fn folder(cart: &Path, index: usize) -> String {
    let b = read::load(&std::fs::read(cart).unwrap()).unwrap();
    b.entries()[index].meta().get(folder_field()).to_string()
}

fn names(cart: &Path) -> Vec<String> {
    let b = read::load(&std::fs::read(cart).unwrap()).unwrap();
    b.entries().iter().map(|e| e.name().to_string()).collect()
}

// AC-173 (CLI): --folder sets the folder, byte for byte what --group writes
#[test]
fn ac_173_cli_folder_sets_the_folder_as_group_did() {
    let dir = scratch("flag");
    let c = cart(&dir, &["alpha", "bravo"]);
    let by_folder = dir.join("folder.crt");
    let by_group = dir.join("group.crt");

    ok(&edit_to(&c, &["--folder", "0", "Games"], &by_folder));
    ok(&edit_to(&c, &["--group", "0", "Games"], &by_group));
    assert_eq!(folder(&by_folder, 0), "Games");
    assert_eq!(
        std::fs::read(&by_folder).unwrap(),
        std::fs::read(&by_group).unwrap(),
        "--folder and --group write the same bytes"
    );

    // The limits --group had: 16 characters, and an empty value clears.
    let before = std::fs::read(&c).unwrap();
    let err = refused(&edit(&c, &["--folder", "0", &"g".repeat(17)]));
    assert!(err.contains("16"), "names the limit: {err}");
    assert_eq!(
        std::fs::read(&c).unwrap(),
        before,
        "a refusal writes nothing"
    );
    let cleared = dir.join("cleared.crt");
    ok(&edit_to(&by_folder, &["--folder", "0", ""], &cleared));
    assert_eq!(folder(&cleared, 0), "");

    let help = stidcart(&["edit", "--help"]);
    let usage = format!(
        "{}{}",
        String::from_utf8_lossy(&help.stdout),
        String::from_utf8_lossy(&help.stderr)
    );
    assert!(
        usage.contains("--folder N TEXT"),
        "usage names --folder: {usage}"
    );
}

// AC-174 (CLI): the import key folder sets the folder; group still does; both in one table is refused
#[test]
fn ac_174_cli_import_folder_key_and_group_alias() {
    let dir = scratch("import");
    let c = cart(&dir, &["alpha", "bravo"]);
    let by_folder_toml = dir.join("folder.toml");
    let by_group_toml = dir.join("group.toml");
    std::fs::write(
        &by_folder_toml,
        "[[entry]]\nname = \"bravo\"\nfolder = \"Demos\"\n",
    )
    .unwrap();
    std::fs::write(
        &by_group_toml,
        "[[entry]]\nname = \"bravo\"\ngroup = \"Demos\"\n",
    )
    .unwrap();
    let by_folder = dir.join("folder.crt");
    let by_group = dir.join("group.crt");
    ok(&edit_to(
        &c,
        &["--import", &by_folder_toml.to_string_lossy()],
        &by_folder,
    ));
    ok(&edit_to(
        &c,
        &["--import", &by_group_toml.to_string_lossy()],
        &by_group,
    ));
    assert_eq!(folder(&by_folder, 1), "Demos");
    assert_eq!(
        std::fs::read(&by_folder).unwrap(),
        std::fs::read(&by_group).unwrap(),
        "the keys folder and group write the same bytes"
    );

    let both = dir.join("both.toml");
    std::fs::write(
        &both,
        "[[entry]]\nname = \"bravo\"\nfolder = \"Demos\"\ngroup = \"Games\"\n",
    )
    .unwrap();
    let before = std::fs::read(&c).unwrap();
    let err = refused(&edit(&c, &["--import", &both.to_string_lossy()]));
    assert!(err.contains("bravo"), "names the entry: {err}");
    assert!(err.contains("folder"), "says what is set twice: {err}");
    assert_eq!(std::fs::read(&c).unwrap(), before, "nothing is written");
}

// AC-175 (CLI): list labels the field folder, and a cart with groups load-saves byte-identically
#[test]
fn ac_175_cli_list_labels_the_folder_and_bytes_do_not_change() {
    let dir = scratch("label");
    let c = cart(&dir, &["alpha", "bravo"]);
    ok(&edit(
        &c,
        &["--group", "0", "Games", "--title", "1", "Ramp"],
    ));

    let listing = ok(&stidcart(&[
        "list".to_string(),
        c.to_string_lossy().into_owned(),
    ]));
    assert!(listing.contains("folder: Games"), "{listing}");
    assert!(!listing.contains("group:"), "{listing}");

    let saved = dir.join("saved.crt");
    ok(&edit_to(&c, &[], &saved));
    assert_eq!(
        std::fs::read(&saved).unwrap(),
        std::fs::read(&c).unwrap(),
        "no stored byte changes"
    );
}

// AC-176 (CLI): --move FROM TO puts the entry at TO; the others keep their order; it keeps what it carries
#[test]
fn ac_176_cli_move_places_the_entry_at_to() {
    let dir = scratch("move");
    let c = cart(&dir, &["alpha", "bravo", "charlie", "delta"]);
    ok(&edit(
        &c,
        &[
            "--hide", "1", "--title", "1", "Bravo!", "--group", "1", "Games",
        ],
    ));
    let data = |cart: &Path, i: usize| {
        let b = read::load(&std::fs::read(cart).unwrap()).unwrap();
        b.entries()[i].as_program().unwrap().data.clone()
    };
    let bravo_data = data(&c, 1);

    let down = dir.join("down.crt");
    ok(&edit_to(&c, &["--move", "1", "3"], &down));
    assert_eq!(names(&down), ["alpha", "charlie", "delta", "bravo"]);
    let b = read::load(&std::fs::read(&down).unwrap()).unwrap();
    let moved = &b.entries()[3];
    assert!(moved.hidden(), "the hidden flag travels");
    assert_eq!(moved.meta().get(Field::Title), "Bravo!");
    assert_eq!(folder(&down, 3), "Games");
    assert_eq!(data(&down, 3), bravo_data, "the data travels");

    let listing = ok(&stidcart(&[
        "list".to_string(),
        down.to_string_lossy().into_owned(),
    ]));
    let row = listing
        .lines()
        .find(|l| l.contains("bravo"))
        .expect("bravo is listed");
    assert!(
        row.trim_start().starts_with('3'),
        "list shows it at 3: {row}"
    );

    let up = dir.join("up.crt");
    ok(&edit_to(&c, &["--move", "3", "0"], &up));
    assert_eq!(names(&up), ["delta", "alpha", "bravo", "charlie"]);
}

// AC-177 (CLI): --move out of range, twice, or next to --add/--add-aligned/--remove is refused; other indices are the cart as given
#[test]
fn ac_177_cli_move_refusals_and_indices_as_given() {
    let dir = scratch("refuse");
    let c = cart(&dir, &["alpha", "bravo", "charlie"]);
    let hello = fixtures().join("hello.prg");
    let hello = hello.to_string_lossy();
    let before = std::fs::read(&c).unwrap();

    for (args, what) in [
        (vec!["--move", "0", "3"], "3"),
        (vec!["--move", "5", "0"], "5"),
        (vec!["--move", "0", "1", "--move", "1", "2"], "--move"),
        (vec!["--move", "0", "1", "--remove", "2"], "--remove"),
        (vec!["--move", "0", "1", "--add", &hello], "--add"),
        (
            vec!["--move", "0", "1", "--add-aligned", &hello],
            "--add-aligned",
        ),
    ] {
        let err = refused(&edit(&c, &args));
        assert!(err.contains(what), "{args:?} names {what}: {err}");
        assert_eq!(
            std::fs::read(&c).unwrap(),
            before,
            "{args:?} writes nothing"
        );
    }

    // A rename in the same call names the entry by its index in the cart as given.
    let out = dir.join("renamed.crt");
    ok(&edit_to(
        &c,
        &[
            "--move", "0", "2", "--rename", "0", "zulu", "--rename", "2", "yankee",
        ],
        &out,
    ));
    assert_eq!(names(&out), ["bravo", "yankee", "zulu"]);
}
