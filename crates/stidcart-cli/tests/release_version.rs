//! release-workflow (D-072): `docs/lcd/ci/release-version.sh` names the version a release is built
//! from, and refuses a tag that names another. The version reaches the bytes of every cart the
//! released tools write, so a binary published under the wrong tag is a wrong binary.

#[path = "support/bash.rs"]
mod bash;

use bash::bash;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Output;

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn run(args: &[&str]) -> Output {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/lcd/ci/release-version.sh");
    bash().arg(script).args(args).output().unwrap()
}

fn stdout(out: &Output) -> String {
    String::from_utf8(out.stdout.clone()).unwrap()
}

/// A stand-in checkout holding the two manifests the script reads, each with a `version` key
/// in another table that must not be taken for the package's.
fn tree(tag: &str, workspace: &str, app: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("stidcart-release-version-{tag}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("gui/src-tauri")).unwrap();
    fs::write(
        dir.join("Cargo.toml"),
        format!(
            "[workspace]\nmembers = [\"a\"]\n\n[workspace.dependencies]\nversion = \"9.9.9\"\n\n\
             [workspace.package]\nedition = \"2021\"\nversion = \"{workspace}\"\n"
        ),
    )
    .unwrap();
    fs::write(
        dir.join("gui/src-tauri/Cargo.toml"),
        format!(
            "[package]\nname = \"app\"\nversion = \"{app}\"\n\n\
             [dependencies]\ntauri = {{ version = \"9.9.9\" }}\n"
        ),
    )
    .unwrap();
    dir
}

#[test]
fn without_a_tag_it_prints_the_version_of_this_checkout() {
    let out = run(&[]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(stdout(&out), format!("{VERSION}\n"));
}

#[test]
fn a_tag_naming_this_version_passes() {
    let out = run(&[&format!("v{VERSION}")]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(stdout(&out), format!("{VERSION}\n"));
}

#[test]
fn a_tag_naming_another_version_fails() {
    let out = run(&["v0.0.1"]);
    assert!(!out.status.success());
    assert!(
        out.stdout.is_empty(),
        "a version was printed for a wrong tag"
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("v0.0.1") && err.contains(VERSION), "{err}");
}

#[test]
fn a_tag_without_the_v_fails() {
    let out = run(&[VERSION]);
    assert!(!out.status.success());
    assert!(out.stdout.is_empty());
}

#[test]
fn it_reads_the_package_version_and_no_other() {
    let dir = tree("decoys", "1.2.3", "1.2.3");
    let out = run(&["--root", dir.to_str().unwrap(), "v1.2.3"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(stdout(&out), "1.2.3\n");
}

/// A Windows checkout can hold the manifests with CRLF line ends.
#[test]
fn it_reads_manifests_with_crlf_line_ends() {
    let dir = tree("crlf", "1.2.3", "1.2.3");
    for manifest in ["Cargo.toml", "gui/src-tauri/Cargo.toml"] {
        let text = fs::read_to_string(dir.join(manifest)).unwrap();
        fs::write(dir.join(manifest), text.replace('\n', "\r\n")).unwrap();
    }
    let out = run(&["--root", dir.to_str().unwrap(), "v1.2.3"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(stdout(&out), "1.2.3\n");
}

#[test]
fn an_app_at_another_version_fails_with_or_without_a_tag() {
    let dir = tree("app-lags", "1.2.3", "1.2.2");
    let root = dir.to_str().unwrap();
    for args in [vec!["--root", root], vec!["--root", root, "v1.2.3"]] {
        let out = run(&args);
        assert!(!out.status.success(), "passed with {args:?}");
        assert!(out.stdout.is_empty());
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(err.contains("1.2.3") && err.contains("1.2.2"), "{err}");
    }
}

#[test]
fn a_manifest_without_a_version_fails() {
    let dir = tree("no-version", "1.2.3", "1.2.3");
    fs::write(dir.join("Cargo.toml"), "[workspace]\nmembers = [\"a\"]\n").unwrap();
    let out = run(&["--root", dir.to_str().unwrap()]);
    assert!(!out.status.success());
    assert!(out.stdout.is_empty());
}
