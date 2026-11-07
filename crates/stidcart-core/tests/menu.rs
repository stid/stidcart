//! custom-menu: the menu put into carts is what its source assembles to (AC-93).
//!
//! The assembled image is committed, so `cargo` never needs the assembler. This test is what keeps
//! the committed bytes honest: with 64tass present it reassembles the source and compares.

use std::path::{Path, PathBuf};
use std::process::Command;

use stidcart_core::menu::STIDCART_MENU;

fn menu_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../menu")
}

/// Whether `name` is on PATH. Absent, the test skips, unless `STIDCART_REQUIRE_TOOLS=1` (as CI
/// sets it), where an absent tool is a failure. A local copy of the CLI tests' helper: the core's
/// tests cannot reach `stidcart-cli/tests/support`.
fn require_tool(name: &str) -> bool {
    let present = Command::new("sh")
        .args(["-c", &format!("command -v {name} >/dev/null 2>&1")])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !present && std::env::var("STIDCART_REQUIRE_TOOLS").as_deref() == Ok("1") {
        panic!("{name} is required (STIDCART_REQUIRE_TOOLS=1) but is not on PATH");
    }
    present
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("stidcart-menu-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Run `build.sh` on the source tree at `src_root` (a directory holding `build.sh` and `src/`),
/// writing into `out`.
fn assemble(src_root: &Path, out: &Path) -> std::process::Output {
    Command::new("sh")
        .arg(src_root.join("build.sh"))
        .arg(out)
        .current_dir(src_root)
        .output()
        .unwrap()
}

// AC-93: the embedded menu is what its source assembles to, and fits the loader slot
#[test]
fn ac_93_the_embedded_menu_is_what_its_source_assembles_to_and_fits() {
    assert_eq!(
        STIDCART_MENU.len(),
        0x2000,
        "the menu image must fill the 8 KiB loader slot exactly"
    );
    assert_eq!(
        STIDCART_MENU,
        std::fs::read(menu_dir().join("menu.bin"))
            .unwrap()
            .as_slice(),
        "the embedded menu is not the committed menu.bin"
    );

    if !require_tool("64tass") {
        eprintln!("AC-93: skipping the reassembly half — 64tass is not installed");
        return;
    }

    let out = scratch("ac93");
    let run = assemble(&menu_dir(), &out);
    assert!(
        run.status.success(),
        "build.sh failed: {}",
        String::from_utf8_lossy(&run.stderr)
    );
    for file in ["menu.bin", "menu.lbl"] {
        assert_eq!(
            std::fs::read(out.join(file)).unwrap(),
            std::fs::read(menu_dir().join(file)).unwrap(),
            "the committed {file} is not what the source assembles to; rerun menu/build.sh menu/"
        );
    }

    // A source that outgrows the slot must not assemble: copy the tree and overfill it.
    let tree = scratch("ac93-overfill");
    std::fs::create_dir_all(tree.join("src")).unwrap();
    std::fs::copy(menu_dir().join("build.sh"), tree.join("build.sh")).unwrap();
    for entry in std::fs::read_dir(menu_dir().join("src")).unwrap() {
        let path = entry.unwrap().path();
        std::fs::copy(&path, tree.join("src").join(path.file_name().unwrap())).unwrap();
    }
    let launch = tree.join("src/launch.s");
    let mut text = std::fs::read_to_string(&launch).unwrap();
    text.push_str(
        "\n        .fill $2000, $ea   ; AC-93 negative control: more than the slot holds\n",
    );
    std::fs::write(&launch, text).unwrap();
    let overfilled = assemble(&tree, &tree.join("out"));
    assert!(
        !overfilled.status.success(),
        "a menu larger than the loader slot assembled without error"
    );
}
