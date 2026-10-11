//! AC-48 and AC-49 of the **subdir-entries** work-item — a subdirectory-flagged xbank through
//! `edit` and `list`, driven as a binary.

use std::path::{Path, PathBuf};
use std::process::Command;

fn stidcart() -> Command {
    Command::new(env!("CARGO_BIN_EXE_stidcart"))
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn bytes(path: &Path) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

/// AC-48 (CLI): `edit --rename` on a cart holding a subdirectory-flagged xbank produces exactly the
/// bytes `ndeft ren` produces for the same request, so the bit survives an edit.
#[test]
fn ac_48_cli_an_edit_carries_the_subdirectory_bit_as_the_reference_does() {
    let dir = std::env::temp_dir().join("stidcart-subdir-rename");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let cart = dir.join("cart.crt");
    std::fs::copy(fixtures().join("in_cxb16sub.crt"), &cart).unwrap();

    let out = stidcart()
        .arg("edit")
        .arg(&cart)
        .args(["--rename", "0", "hello2"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        bytes(&cart) == bytes(&fixtures().join("cxb16sub.crt")),
        "the edited cart differs from ndeft's rename of the same input"
    );
}

/// AC-49 (RENDER): `list` marks a subdirectory-flagged entry with `S` in the flags column.
#[test]
fn ac_49_render_a_subdirectory_flagged_entry_is_marked() {
    let out = stidcart()
        .arg("list")
        .arg(fixtures().join("in_cxb16sub.crt"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    let flags = |entry: &str| -> String {
        stdout
            .lines()
            .find(|l| l.split_whitespace().nth(1) == Some(entry))
            .unwrap_or_else(|| panic!("no row for '{entry}' in:\n{stdout}"))
            .split_whitespace()
            .nth(2)
            .unwrap()
            .to_owned()
    };
    assert_eq!(flags("in_xb16"), "S", "the flagged xbank:\n{stdout}");
    assert_eq!(flags("hello"), "-", "an unflagged program:\n{stdout}");
}
