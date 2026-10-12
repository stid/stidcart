//! What `stidcart list` prints for every frozen fixture, frozen in turn.
//!
//! A characterisation test, not an acceptance criterion. The listing's values are moving into the
//! core (D-032) so the GUI can show the same ones, and nothing about `list`'s output may change on
//! the way, refusals included. Each golden holds stdout, stderr and the exit code. Regenerate with
//! `UPDATE_GOLDEN=1 cargo test --test list_golden` only when a commit means to change the output.

use std::path::{Path, PathBuf};
use std::process::Command;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn goldens() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn listing_of(fixture: &Path) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_stidcart"))
        .arg("list")
        .arg(fixture)
        .output()
        .unwrap();
    // The path appears in no message today, but a refusal that ever names the file must not tie
    // the golden to this checkout's location.
    let stderr = String::from_utf8(out.stderr)
        .unwrap()
        .replace(&fixture.display().to_string(), "<fixture>");
    format!(
        "exit: {}\n-- stdout --\n{}-- stderr --\n{}",
        out.status.code().unwrap(),
        String::from_utf8(out.stdout).unwrap(),
        stderr
    )
}

#[test]
fn list_output_is_unchanged_for_every_fixture() {
    let update = std::env::var_os("UPDATE_GOLDEN").is_some();
    let mut crts: Vec<PathBuf> = std::fs::read_dir(fixtures())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "crt"))
        .collect();
    crts.sort();
    assert!(!crts.is_empty(), "no fixtures found");

    let mut changed = Vec::new();
    for crt in &crts {
        let name = crt.file_stem().unwrap().to_string_lossy().into_owned();
        let golden = goldens().join(format!("{name}.txt"));
        let actual = listing_of(crt);
        if update {
            std::fs::create_dir_all(goldens()).unwrap();
            std::fs::write(&golden, &actual).unwrap();
            continue;
        }
        let expected = std::fs::read_to_string(&golden)
            .unwrap_or_else(|e| panic!("reading golden {}: {e}", golden.display()));
        if actual != expected {
            changed.push(format!(
                "{name}:\n--- expected\n{expected}+++ actual\n{actual}"
            ));
        }
    }
    assert!(
        changed.is_empty(),
        "list output changed:\n{}",
        changed.join("\n")
    );
}
