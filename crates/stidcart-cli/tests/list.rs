//! AC-1..AC-5 of the **list-command** work-item — what `stidcart list` shows, seen from outside.
//!
//! The numbering is per work-item, so these are not the AC-1/AC-5 in `cli.rs`; those belong to the
//! write path. Assertions here check that the required facts appear, not where on the line they
//! land — the column layout is presentation and should be free to change without a red suite.

use std::path::{Path, PathBuf};
use std::process::Command;

use stidcart_core::read;

fn stidcart() -> Command {
    Command::new(env!("CARGO_BIN_EXE_stidcart"))
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
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

        for (what, needle) in [
            ("index", i.to_string()),
            ("type", "PRG".to_string()),
            ("bank", format!("{:02x}", entry.bank)),
            ("offset", format!("{:04x}", entry.offset)),
            ("size", format!("{:x}", entry.size)),
        ] {
            assert!(
                line.contains(&needle),
                "the line for '{}' is missing its {what} ({needle}): {line}",
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
