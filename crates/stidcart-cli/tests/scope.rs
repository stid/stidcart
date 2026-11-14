//! test-scopes (D-071): `docs/lcd/ci/scope.sh` turns the paths a change touches into the test
//! groups the gate runs. These pin the rules, because a rule that is too narrow passes a change
//! without the tests that would have caught it.

#[path = "support/bash.rs"]
mod bash;

use std::io::Write;
use std::path::Path;
use std::process::Stdio;

use bash::bash;

/// The scope of a change touching `paths`, as `(rust, vice, gui)`.
fn scope(paths: &[&str]) -> (bool, String, bool) {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/lcd/ci/scope.sh");
    let mut child = bash()
        .arg(script)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let mut input = paths.join("\n");
    input.push('\n');
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "scope.sh failed for {paths:?}");
    let text = String::from_utf8(out.stdout).unwrap();
    let value = |key: &str| {
        text.lines()
            .find_map(|l| l.strip_prefix(&format!("{key}=")))
            .unwrap_or_else(|| panic!("no {key}= line in {text:?}"))
            .to_string()
    };
    (
        value("rust") == "true",
        value("vice"),
        value("gui") == "true",
    )
}

fn all() -> (bool, String, bool) {
    (true, "full".into(), true)
}

fn nothing() -> (bool, String, bool) {
    (false, "none".into(), false)
}

#[test]
fn prose_runs_nothing() {
    assert_eq!(scope(&["README.md"]), nothing());
    assert_eq!(
        scope(&[
            "CONTRIBUTING.md",
            "docs/guide/menu.md",
            "docs/guide/img/menu.png",
            "docs/lcd/DECISIONS.md",
            "docs/lcd/work/test-scopes/JOURNAL.md",
            "docs/format/easyflash-cart-format.md",
            "menu/README.md",
            "gui/README.md",
            ".claude/rules/lcd-conventions.md",
            "scripts/install.sh",
        ]),
        nothing()
    );
}

#[test]
fn no_change_runs_nothing() {
    assert_eq!(scope(&[]), nothing());
}

#[test]
fn the_menu_runs_every_boot_and_not_the_app() {
    let menu = (true, "full".to_string(), false);
    assert_eq!(scope(&["menu/src/menu.asm"]), menu);
    assert_eq!(scope(&["menu/menu.bin", "menu/menu.lbl"]), menu);
    assert_eq!(scope(&["menu/past/5b54e81.bin"]), menu);
    assert_eq!(scope(&["menu/build.sh", "README.md"]), menu);
}

#[test]
fn the_builder_runs_the_cart_boots_and_the_app() {
    let cart = (true, "cart".to_string(), true);
    assert_eq!(scope(&["crates/stidcart-core/src/place.rs"]), cart);
    assert_eq!(scope(&["crates/stidcart-cli/src/lib.rs"]), cart);
    assert_eq!(scope(&["Cargo.lock"]), cart);
    assert_eq!(scope(&["rust-toolchain.toml"]), cart);
    assert_eq!(scope(&["firmware/easyloader_nrm.prg"]), cart);
}

#[test]
fn the_builder_and_the_menu_together_run_every_boot_and_the_app() {
    assert_eq!(
        scope(&["crates/stidcart-core/src/menu.rs", "menu/menu.bin"]),
        all()
    );
}

#[test]
fn a_root_test_runs_the_root_tests_alone() {
    let rust = (true, "none".to_string(), false);
    assert_eq!(scope(&["crates/stidcart-core/tests/placement.rs"]), rust);
    assert_eq!(scope(&["crates/stidcart-cli/tests/golden/list.txt"]), rust);
    // The parity test reads the manifest, so prose under fixtures/ is not prose.
    assert_eq!(scope(&["fixtures/MANIFEST.md"]), rust);
}

#[test]
fn a_boot_test_or_its_harness_runs_every_boot() {
    let boots = (true, "full".to_string(), false);
    assert_eq!(scope(&["crates/stidcart-cli/tests/menu_boot.rs"]), boots);
    assert_eq!(scope(&["crates/stidcart-cli/tests/boot.rs"]), boots);
    assert_eq!(scope(&["crates/stidcart-cli/tests/meta_boot.rs"]), boots);
    assert_eq!(scope(&["crates/stidcart-cli/tests/support/vice.rs"]), boots);
}

#[test]
fn the_app_runs_the_app_alone() {
    let gui = (false, "none".to_string(), true);
    assert_eq!(scope(&["gui/src/lib/Listing.svelte"]), gui);
    assert_eq!(scope(&["gui/src-tauri/src/lib.rs"]), gui);
    assert_eq!(scope(&["gui/package-lock.json", "docs/guide/app.md"]), gui);
}

#[test]
fn the_gate_itself_runs_everything() {
    assert_eq!(scope(&[".github/workflows/ci.yml"]), all());
    assert_eq!(scope(&["docs/lcd/ci/scope.sh"]), all());
    assert_eq!(scope(&["docs/lcd/ci/gui-gate.sh"]), all());
    assert_eq!(scope(&["docs/lcd/ci/install-menu-tools.sh"]), all());
}

#[test]
fn a_path_no_rule_names_runs_everything() {
    assert_eq!(scope(&["some/new/place.xyz"]), all());
    assert_eq!(scope(&["README.md", "Makefile"]), all());
}

#[test]
fn all_is_everything() {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/lcd/ci/scope.sh");
    let out = bash().arg(script).arg("--all").output().unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        "rust=true\nvice=full\ngui=true\n"
    );
}

#[test]
fn an_unknown_base_fails_instead_of_reading_as_no_change() {
    let script = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/lcd/ci/scope.sh");
    let out = bash()
        .arg(script)
        .arg("stidcart-no-such-ref")
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(out.stdout.is_empty(), "a scope was printed for a bad base");
}
