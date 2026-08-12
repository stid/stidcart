//! AC-1..AC-5 of the **list-command** work-item — what `stidcart list` shows, seen from outside.
//!
//! The numbering is per work-item, so these are not the AC-1/AC-5 in `cli.rs`; those belong to the
//! write path.

use std::path::{Path, PathBuf};
use std::process::Command;

use stidcart_core::directory::{self, DirEntry, EntryType};
use stidcart_core::{crt, read, reserved, CartBuilder};

fn stidcart() -> Command {
    Command::new(env!("CARGO_BIN_EXE_stidcart"))
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

/// A scratch directory unique to the calling test, so cases never see each other's output.
fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("stidcart-list-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn list(name: &str) -> std::process::Output {
    stidcart()
        .arg("list")
        .arg(fixtures().join(name))
        .output()
        .unwrap()
}

/// AC-1 (CLI): every directory entry is printed with its index, name, type, bank, offset and size.
#[test]
fn ac_1_cli_list_prints_every_entry_with_its_position_and_size() {
    let result = list("two.crt");
    assert!(
        result.status.success(),
        "listing a valid image should succeed"
    );
    let stdout = String::from_utf8_lossy(&result.stdout);

    let info = read::inspect(&std::fs::read(fixtures().join("two.crt")).unwrap()).unwrap();
    assert_eq!(info.entries.len(), 2, "two.crt is the two-entry fixture");

    for (i, entry) in info.entries.iter().enumerate() {
        // The entry's own line, found by name — the facts have to travel together, not merely be
        // present somewhere in the output.
        let line = stdout
            .lines()
            .find(|l| l.contains(&entry.name))
            .unwrap_or_else(|| panic!("no line for entry '{}' in:\n{stdout}", entry.name));

        // Split into columns and compare each one whole. `contains` is not good enough here: "0"
        // already occurs inside `$0bba`, so a substring check on the index passes even when the
        // column is deleted outright, and "PRG" occurs inside "PRG-L", so it cannot tell the
        // interleaved type from a half-confined one.
        // (Both names in `two.crt` are single words, so splitting on whitespace lines up with the
        // columns. A name with a space in it would not, which is why the hidden-entry case below
        // checks its marker rather than reusing this.)
        let cols: Vec<&str> = line.split_whitespace().collect();
        let want = [
            ("index", i.to_string()),
            ("name", entry.name.clone()),
            ("flags", "-".to_string()),
            ("type", "PRG".to_string()),
            ("bank", format!("${:02x}", entry.bank)),
            ("offset", format!("${:04x}", entry.offset)),
            ("size", format!("${:06x}", entry.size)),
        ];
        assert_eq!(
            cols.len(),
            want.len(),
            "expected one column per fact in: {line}"
        );
        for (col, (what, expected)) in cols.iter().zip(want.iter()) {
            assert_eq!(
                col, expected,
                "the {what} column for '{}' is wrong: {line}",
                entry.name
            );
        }
    }
}

/// AC-2 (CLI): the cartridge's own name identifies it in the output.
#[test]
fn ac_2_cli_list_identifies_the_cartridge_by_its_stored_name() {
    let stdout = String::from_utf8_lossy(&list("named.crt").stdout).into_owned();
    assert!(
        stdout.contains("weird name"),
        "the cartridge name stored in the image should appear, got:\n{stdout}"
    );

    // And the placeholder path, so the assertion above is not just finding an echoed argument.
    let placeholder = String::from_utf8_lossy(&list("single.crt").stdout).into_owned();
    assert!(
        !placeholder.contains("weird name"),
        "a different image must not report the other one's name"
    );
}

/// AC-3 (CLI): free space is reported, and it is the capacity less the entries present.
#[test]
fn ac_3_cli_list_reports_free_space_less_the_entries_present() {
    let stdout = String::from_utf8_lossy(&list("spill.crt").stdout).into_owned();

    let info = read::inspect(&std::fs::read(fixtures().join("spill.crt")).unwrap()).unwrap();
    let free = info.free_space();
    assert!(
        free < 63 * 8192 * 2,
        "spill.crt holds an entry, so free space must be below the empty capacity"
    );
    assert!(
        stdout.contains(&free.to_string()) || stdout.contains(&format!("{free:x}")),
        "free space ({free}) should be stated, got:\n{stdout}"
    );
}

/// AC-4 (CLI): a file the loader refuses fails with the loader's own reason and lists nothing.
#[test]
fn ac_4_cli_list_refuses_a_file_that_is_not_a_cartridge() {
    let result = stidcart()
        .arg("list")
        .arg(fixtures().join("hello.prg"))
        .output()
        .unwrap();

    assert!(!result.status.success(), "a .prg is not a cartridge image");
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("not an EasyFlash cartridge"),
        "the loader's own words should reach the user, got: {stderr}"
    );
    assert!(
        String::from_utf8_lossy(&result.stdout).trim().is_empty(),
        "nothing should be listed for an image that would not load"
    );
}

/// AC-4 (CLI): a missing path is reported rather than swallowed.
#[test]
fn ac_4_cli_list_reports_a_missing_file() {
    let result = stidcart()
        .args(["list", "no-such-cart.crt"])
        .output()
        .unwrap();

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("no-such-cart.crt"));
}

/// AC-5 (CLI): `list` with no file named fails and prints the usage text.
#[test]
fn ac_5_cli_list_without_a_file_prints_usage() {
    let result = stidcart().arg("list").output().unwrap();

    assert!(!result.status.success(), "list needs a file to list");
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("usage") && stderr.contains("list"),
        "the usage text should name the command, got: {stderr}"
    );
}

/// A hidden entry is listed and marked, rather than making the whole listing fail.
///
/// The refusal belongs to the path that rebuilds a cart, not to the one that describes it — and it
/// used to abort the walk, so an offending entry in slot 0 emptied the listing entirely.
#[test]
fn list_shows_a_hidden_entry_and_the_entries_behind_it() {
    let dir = scratch("hidden");
    let cart = dir.join("hidden.crt");

    let mut img = CartBuilder::new().build_image().unwrap();
    let hidden = DirEntry {
        name: "secret".into(),
        kind: EntryType::Prg,
        bank: 1,
        offset: 0,
        size: 2,
        hidden: true,
    };
    let behind = DirEntry {
        name: "behind".into(),
        bank: 2,
        hidden: false,
        ..hidden.clone()
    };
    img.write_at(reserved::DIRECTORY_OFFSET, &directory::encode(&hidden));
    img.write_at(
        reserved::DIRECTORY_OFFSET + directory::ENTRY_LEN,
        &directory::encode(&behind),
    );
    std::fs::write(&cart, crt::write(&img, "probe")).unwrap();

    let result = stidcart().arg("list").arg(&cart).output().unwrap();
    assert!(
        result.status.success(),
        "a hidden entry is describable, so listing must not fail: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let stdout = String::from_utf8_lossy(&result.stdout);

    let line = stdout
        .lines()
        .find(|l| l.contains("secret"))
        .unwrap_or_else(|| panic!("the hidden entry is missing from:\n{stdout}"));
    assert!(
        line.split_whitespace().any(|c| c == "H"),
        "a hidden entry has to be marked as hidden, not shown as a plain one: {line}"
    );
    assert!(
        stdout.lines().any(|l| l.contains("behind")),
        "the entries after a hidden one must survive it:\n{stdout}"
    );
}

/// A crafted container header name cannot drive the terminal.
///
/// Entry names and the cartridge name are filtered on the way out of the image by the machine's
/// own character mapping, which turns control bytes into `.`. The container header name is not —
/// it is host-encoded text carried through verbatim — and `list` is the first path that puts it in
/// front of a terminal.
#[test]
fn list_does_not_let_a_crafted_header_name_drive_the_terminal() {
    let dir = scratch("escape");
    let cart = dir.join("nasty.crt");

    let img = CartBuilder::new().build_image().unwrap();
    std::fs::write(&cart, crt::write(&img, "\x1b[2J\x1b[Hgotcha\r\x07")).unwrap();

    let result = stidcart().arg("list").arg(&cart).output().unwrap();
    assert!(result.status.success());
    let stdout = String::from_utf8_lossy(&result.stdout);

    assert!(
        !stdout.contains('\x1b') && !stdout.contains('\x07') && !stdout.contains('\r'),
        "no control byte from the image may reach the terminal: {stdout:?}"
    );
    assert!(
        stdout.contains("gotcha"),
        "the printable part is still reported — this is filtering, not discarding: {stdout}"
    );
}
