//! AC-1, AC-5 and AC-11 — what the command does, seen from outside.

use std::path::{Path, PathBuf};
use std::process::Command;

fn stidcart() -> Command {
    Command::new(env!("CARGO_BIN_EXE_stidcart"))
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

/// A scratch directory unique to the calling test, so cases never see each other's output.
fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("stidcart-cli-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn ac_1_cli_build_writes_an_image_and_reports_success() {
    let dir = scratch("ac1");
    let out = dir.join("out.crt");

    let status = stidcart()
        .args(["build", "-e", "a cart", "-o"])
        .arg(&out)
        .arg(fixtures().join("hello.prg"))
        .status()
        .unwrap();

    assert!(status.success(), "build should exit successfully");
    let bytes = std::fs::read(&out).expect("an image should have been written");
    assert_eq!(
        &bytes[..16],
        b"C64 CARTRIDGE   ",
        "output is not a cartridge image"
    );
}

#[test]
fn ac_1_cli_entry_name_comes_from_the_file_stem() {
    let dir = scratch("ac1-name");
    let src = dir.join("My Game v2.prg");
    std::fs::copy(fixtures().join("hello.prg"), &src).unwrap();
    let out = dir.join("out.crt");

    assert!(stidcart()
        .args(["build", "-o"])
        .arg(&out)
        .arg(&src)
        .status()
        .unwrap()
        .success());

    // Stored in the machine's own encoding, so an uppercase-looking string is expected.
    let bytes = std::fs::read(&out).unwrap();
    let needle = b"mY gAME V2";
    assert!(
        bytes.windows(needle.len()).any(|w| w == needle),
        "entry name derived from the file stem was not found in the image"
    );
}

#[test]
fn ac_5_cli_oversized_input_fails_without_writing_an_image() {
    let dir = scratch("ac5");
    let out = dir.join("too-big.crt");

    // Comfortably larger than the cartridge, so this cannot be placed.
    let huge = dir.join("huge.prg");
    std::fs::write(&huge, vec![0x42; 1024 * 1024 + 16]).unwrap();

    let result = stidcart()
        .args(["build", "-o"])
        .arg(&out)
        .arg(&huge)
        .output()
        .unwrap();

    assert!(!result.status.success(), "an oversized build must fail");
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("free") || stderr.contains("bytes"),
        "the error should say what went wrong, got: {stderr}"
    );
    assert!(
        !out.exists(),
        "no image should be left behind by a failed build"
    );
}

#[test]
fn ac_11_cli_empty_input_list_matches_the_reference_behaviour() {
    // Measured on the reference tool: it exits reporting success and writes no file. Pinned here
    // so the behaviour is a decision rather than an accident.
    let dir = scratch("ac11");
    let out = dir.join("nothing.crt");

    let status = stidcart().args(["build", "-o"]).arg(&out).status().unwrap();

    assert!(status.success(), "an empty request should report success");
    assert!(!out.exists(), "an empty request should write no file");
}

#[test]
fn ac_5_cli_a_missing_input_file_is_reported_not_ignored() {
    let dir = scratch("ac5-missing");
    let out = dir.join("out.crt");

    let result = stidcart()
        .args(["build", "-o"])
        .arg(&out)
        .arg(dir.join("does-not-exist.prg"))
        .output()
        .unwrap();

    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("does-not-exist.prg"));
    assert!(!out.exists());
}
