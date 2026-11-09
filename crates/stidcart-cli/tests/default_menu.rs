//! default-stidcart-menu AC-197, AC-198 (CLI): a new cart gets our menu unless told otherwise, and
//! content our menu can't run is refused with a pointer to EasyLoader (D-064).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("stidcart-default-menu-{tag}"));
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

fn path(p: &Path) -> String {
    p.to_string_lossy().into_owned()
}

// AC-197 (CLI): build naming no menu writes our menu; --menu easyloader writes EasyLoader
#[test]
fn ac_197_cli_build_defaults_to_our_menu() {
    let dir = scratch("default");
    let ours = dir.join("ours.crt");
    let hello = fixtures().join("hello.prg");
    ok(&stidcart(&["build", "-o", &path(&ours), &path(&hello)]));
    assert!(
        ok(&stidcart(&["list", &path(&ours)])).contains("menu: stidcart"),
        "a build naming no menu gets ours"
    );

    let el = dir.join("el.crt");
    ok(&stidcart(&[
        "build",
        "--menu",
        "easyloader",
        "-o",
        &path(&el),
        &path(&hello),
    ]));
    assert!(ok(&stidcart(&["list", &path(&el)])).contains("menu: EasyLoader"));
}

// AC-198 (CLI): content our menu can't run is refused naming the entry and EasyLoader; --menu easyloader builds it
#[test]
fn ac_198_cli_content_our_menu_cannot_run_points_to_easyloader() {
    let dir = scratch("refuse");
    let out = dir.join("out.crt");
    let ocean = fixtures().join("in_ocean.crt");
    let r = stidcart(&["build", "-o", &path(&out), &path(&ocean)]);
    assert!(
        !r.status.success(),
        "an Ocean cart under our menu is refused"
    );
    let err = String::from_utf8_lossy(&r.stderr);
    assert!(err.contains("in_ocean"), "names the entry: {err}");
    assert!(err.contains("EasyLoader"), "points to EasyLoader: {err}");
    assert!(!out.exists(), "nothing is written");

    ok(&stidcart(&[
        "build",
        "--menu",
        "easyloader",
        "-o",
        &path(&out),
        &path(&ocean),
    ]));
}
