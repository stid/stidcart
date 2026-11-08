//! menu-meta AC-156..AC-158: setting entry metadata with `edit` flags and an import file, and
//! `list` showing it, driven as a binary.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use stidcart_core::meta::Field;
use stidcart_core::read;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("stidcart-meta-cli-{tag}"));
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
        "meta".into(),
        "-o".into(),
        path.to_string_lossy().into_owned(),
    ];
    for (i, n) in names.iter().enumerate() {
        // A subdirectory each, so two entries may share a name.
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

fn field(cart: &Path, index: usize, f: Field) -> String {
    let b = read::load(&std::fs::read(cart).unwrap()).unwrap();
    b.entries()[index].meta().get(f).to_string()
}

// AC-156 (CLI): edit sets each of the five fields on the entry at an index list prints
#[test]
fn ac_156_cli_edit_sets_each_field_and_an_empty_value_clears() {
    let dir = scratch("set");
    let c = cart(&dir, &["alpha", "bravo"]);
    ok(&edit(
        &c,
        &[
            "--title",
            "1",
            "Bravo Two",
            "--year",
            "1",
            "1987",
            "--author",
            "1",
            "Someone",
            "--note",
            "1",
            "a note",
            "--group",
            "1",
            "Demos",
        ],
    ));
    assert_eq!(field(&c, 1, Field::Title), "Bravo Two");
    assert_eq!(field(&c, 1, Field::Year), "1987");
    assert_eq!(field(&c, 1, Field::Author), "Someone");
    assert_eq!(field(&c, 1, Field::Note), "a note");
    assert_eq!(field(&c, 1, Field::Group), "Demos");
    assert_eq!(
        field(&c, 0, Field::Title),
        "",
        "the other entry is untouched"
    );

    ok(&edit(&c, &["--note", "1", ""]));
    assert_eq!(field(&c, 1, Field::Note), "", "an empty value clears");
    assert_eq!(
        field(&c, 1, Field::Title),
        "Bravo Two",
        "and only that field"
    );
}

// AC-156 (CLI): indexes refer to the cart as given, as for every other edit
#[test]
fn ac_156_cli_indexes_refer_to_the_cart_as_given() {
    let dir = scratch("index");
    let c = cart(&dir, &["alpha", "bravo", "charlie"]);
    ok(&edit(&c, &["--remove", "0", "--title", "2", "Charlie"]));
    let b = read::load(&std::fs::read(&c).unwrap()).unwrap();
    assert_eq!(b.entries()[1].name(), "charlie");
    assert_eq!(b.entries()[1].meta().get(Field::Title), "Charlie");
}

// AC-156 (CLI): an over-long or unmappable value is refused naming the field, nothing written
#[test]
fn ac_156_cli_an_invalid_value_is_refused_naming_the_field() {
    let dir = scratch("invalid");
    let c = cart(&dir, &["alpha"]);
    let before = std::fs::read(&c).unwrap();
    let cases: [(&str, String, &str); 6] = [
        ("--title", "T".repeat(49), "title"),
        ("--year", "19845".into(), "year"),
        ("--author", "A".repeat(25), "author"),
        ("--note", "N".repeat(49), "note"),
        ("--group", "G".repeat(17), "group"),
        ("--author", "Zoë".into(), "author"),
    ];
    for (flag, value, name) in cases {
        let err = refused(&edit(
            &c,
            &[flag, "0", value.as_str(), "--rename", "0", "x"],
        ));
        assert!(err.contains(name), "{flag} {value}: {err}");
        assert_eq!(
            std::fs::read(&c).unwrap(),
            before,
            "{flag}: nothing written"
        );
    }
    // At the limit is fine.
    ok(&edit(
        &c,
        &[
            "--title",
            "0",
            &"T".repeat(48),
            "--group",
            "0",
            &"G".repeat(16),
        ],
    ));
    let err = refused(&edit(&c, &["--title", "5", "x"]));
    assert!(err.contains("no entry at index 5"), "{err}");
}

fn import(dir: &Path, toml: &str) -> PathBuf {
    let p = dir.join("meta.toml");
    std::fs::write(&p, toml).unwrap();
    p
}

// AC-157 (CLI): an import names entries by stored name, ignoring case, and sets what it gives
#[test]
fn ac_157_cli_import_sets_the_named_entries_fields() {
    let dir = scratch("import");
    let c = cart(&dir, &["alpha", "bravo", "charlie"]);
    ok(&edit(
        &c,
        &["--note", "0", "keep me", "--title", "2", "old title"],
    ));
    let file = import(
        &dir,
        r#"
[[entry]]
name = "ALPHA"
title = "Alpha Game"
year = 1984
group = "Games"

[[entry]]
name = "Charlie"
title = ""
author = "Somebody"
"#,
    );
    ok(&edit(&c, &["--import", file.to_str().unwrap()]));
    assert_eq!(field(&c, 0, Field::Title), "Alpha Game");
    assert_eq!(field(&c, 0, Field::Year), "1984");
    assert_eq!(field(&c, 0, Field::Group), "Games");
    assert_eq!(
        field(&c, 0, Field::Note),
        "keep me",
        "a field not given is left alone"
    );
    assert_eq!(field(&c, 2, Field::Title), "", "an empty value clears");
    assert_eq!(field(&c, 2, Field::Author), "Somebody");
    assert_eq!(
        field(&c, 1, Field::Title),
        "",
        "an entry not named is untouched"
    );
}

// AC-157 (CLI): a name matching no entry or several, or an invalid value, refuses the whole import
#[test]
fn ac_157_cli_a_bad_import_is_refused_whole_and_writes_nothing() {
    let dir = scratch("import-bad");
    let c = cart(&dir, &["alpha", "twin", "twin"]);
    let before = std::fs::read(&c).unwrap();
    let cases = [
        ("[[entry]]\nname = \"alpha\"\ntitle = \"ok\"\n[[entry]]\nname = \"nobody\"\ntitle = \"x\"\n", "nobody"),
        ("[[entry]]\nname = \"alpha\"\ntitle = \"ok\"\n[[entry]]\nname = \"TWIN\"\ntitle = \"x\"\n", "twin"),
        (
            "[[entry]]\nname = \"alpha\"\ntitle = \"ok\"\nyear = \"19845\"\n",
            "year",
        ),
        ("[[entry]]\nname = \"alpha\"\ntitel = \"typo\"\n", "titel"),
        ("[[entry]]\nname = \"alpha\"\nauthor = \"Zoë\"\n", "author"),
        ("[[entry]]\nname = \"alpha\"\ntitle = \"a\"\n[[entry]]\nname = \"Alpha\"\nnote = \"b\"\n", "alpha"),
        ("this is not toml", "meta.toml"),
    ];
    for (text, named) in cases {
        let file = import(&dir, text);
        let err = refused(&edit(&c, &["--import", file.to_str().unwrap()]));
        assert!(err.to_lowercase().contains(named), "{text}: {err}");
        assert_eq!(
            std::fs::read(&c).unwrap(),
            before,
            "{text}: nothing written"
        );
    }
}

/// The line `list` prints under a row with metadata.
const META_LINE: &str =
    "       title: Alpha Game  year: 1984  author: Someone  note: a note  group: Games";

// AC-158 (RENDER): list prints metadata on an indented line under each entry that has some
#[test]
fn ac_158_list_prints_metadata_under_the_entries_that_have_some() {
    let dir = scratch("list");
    let c = cart(&dir, &["alpha", "bravo"]);
    ok(&edit(
        &c,
        &[
            "--title",
            "0",
            "Alpha Game",
            "--year",
            "0",
            "1984",
            "--author",
            "0",
            "Someone",
            "--note",
            "0",
            "a note",
            "--group",
            "0",
            "Games",
        ],
    ));
    let out = ok(&stidcart(&["list".as_ref(), c.as_os_str()]));
    let lines: Vec<&str> = out.lines().collect();
    let alpha = lines
        .iter()
        .position(|l| l.contains("  alpha  "))
        .expect("alpha is listed");
    assert_eq!(lines[alpha + 1], META_LINE);
    assert!(
        lines[alpha + 2].contains("  bravo  "),
        "no line under an entry without metadata"
    );
    assert!(
        out.contains("2 entries"),
        "the metadata entry is not counted: {out}"
    );
    assert!(!out.contains("stidcart.meta"), "{out}");

    // Only the fields set are shown.
    ok(&edit(
        &c,
        &["--year", "0", "", "--author", "0", "", "--note", "0", ""],
    ));
    let out = ok(&stidcart(&["list".as_ref(), c.as_os_str()]));
    assert!(
        out.contains("\n       title: Alpha Game  group: Games\n"),
        "{out}"
    );
}

// AC-158 (RENDER): a cart without metadata lists exactly as before
#[test]
fn ac_158_a_cart_without_metadata_lists_as_before() {
    for name in ["two", "cmixed"] {
        let out = ok(&stidcart(&[
            "list".as_ref(),
            fixtures().join(format!("{name}.crt")).as_os_str(),
        ]));
        let golden = std::fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("tests/golden/{name}.txt")),
        )
        .unwrap();
        // A golden holds the exit code, stdout and stderr (list_golden.rs).
        assert_eq!(
            format!("exit: 0\n-- stdout --\n{out}-- stderr --\n"),
            golden,
            "{name}"
        );
        assert!(!out.contains("metadata"), "{name}");
    }
}

// AC-158 (RENDER): list says how many records were dropped
#[test]
fn ac_158_list_says_how_many_records_were_dropped() {
    use stidcart_core::reserved::DIRECTORY_OFFSET;
    let dir = scratch("dropped");
    let c = cart(&dir, &["alpha", "bravo"]);
    ok(&edit(&c, &["--title", "0", "A", "--title", "1", "B"]));
    // Another tool swaps the two directory slots.
    let container = stidcart_core::crt::parse(&std::fs::read(&c).unwrap()).unwrap();
    let mut img = container.image;
    let a = img.as_bytes()[DIRECTORY_OFFSET..DIRECTORY_OFFSET + 24].to_vec();
    let b = img.as_bytes()[DIRECTORY_OFFSET + 24..DIRECTORY_OFFSET + 48].to_vec();
    img.write_at(DIRECTORY_OFFSET, &b);
    img.write_at(DIRECTORY_OFFSET + 24, &a);
    std::fs::write(&c, stidcart_core::crt::write(&img, "other")).unwrap();

    let out = ok(&stidcart(&["list".as_ref(), c.as_os_str()]));
    assert!(out.contains("metadata: 2 records dropped"), "{out}");
    assert!(!out.contains("title:"), "{out}");
}
