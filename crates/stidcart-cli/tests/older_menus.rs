//! older-menus: a cart carrying an earlier build of our menu, through the binary (AC-129, D-049).

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use stidcart_core::crt;
use stidcart_core::menu::STIDCART_MENU;

/// The menu loader's slot: bank 0 ROML, the first 8 KiB of the image.
const SLOT: std::ops::Range<usize> = 0..0x2000;

/// The build at 5b54e81, the one carts made by stidcart 0.15 carry.
const AT_5B54E81: &[u8] = include_bytes!("../../../menu/past/5b54e81.bin");

fn fx(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(name)
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("stidcart-older-menus-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn stidcart(args: &[&OsStr]) -> Output {
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

fn s(text: &str) -> &OsStr {
    OsStr::new(text)
}

fn image(path: &Path) -> Vec<u8> {
    crt::parse(&std::fs::read(path).unwrap())
        .unwrap()
        .image
        .as_bytes()
        .to_vec()
}

/// Two programs built with our menu, then given the 5b54e81 build in the loader slot, as a cart
/// made by stidcart 0.15 is.
fn older(dir: &Path) -> PathBuf {
    let built = dir.join("built.crt");
    ok(&stidcart(&[
        s("build"),
        s("--menu"),
        s("stidcart"),
        s("-e"),
        s("older"),
        s("-o"),
        built.as_os_str(),
        fx("hello.prg").as_os_str(),
        fx("ramp.prg").as_os_str(),
    ]));
    let mut container = crt::parse(&std::fs::read(&built).unwrap()).unwrap();
    container.image.write_at(SLOT.start, AT_5B54E81);
    let out = dir.join("older.crt");
    std::fs::write(&out, crt::write(&container.image, &container.header_name)).unwrap();
    out
}

// AC-129 (CLI): list names an earlier build of our menu
#[test]
fn ac_129_cli_list_names_an_older_build() {
    let dir = scratch("list");
    let listed = ok(&stidcart(&[s("list"), older(&dir).as_os_str()]));
    assert!(
        listed.contains("menu: stidcart (older build)\n"),
        "{listed}"
    );
}

// AC-129 (CLI): edit --menu stidcart writes the current build and changes nothing else
#[test]
fn ac_129_cli_edit_menu_stidcart_writes_the_current_build() {
    let dir = scratch("menu");
    let cart = older(&dir);
    let out = dir.join("upgraded.crt");
    ok(&stidcart(&[
        s("edit"),
        cart.as_os_str(),
        s("-o"),
        out.as_os_str(),
        s("--menu"),
        s("stidcart"),
    ]));
    let (before, after) = (image(&cart), image(&out));
    assert_eq!(&after[SLOT], STIDCART_MENU);
    assert_eq!(
        &before[SLOT.end..],
        &after[SLOT.end..],
        "something outside the slot moved"
    );
    let listed = ok(&stidcart(&[s("list"), out.as_os_str()]));
    assert!(listed.contains("menu: stidcart\n"), "{listed}");
}

// AC-129 (CLI): any other edit writes the current build too
#[test]
fn ac_129_cli_any_edit_writes_the_current_build() {
    let dir = scratch("rename");
    let cart = older(&dir);
    ok(&stidcart(&[
        s("edit"),
        cart.as_os_str(),
        s("--rename"),
        s("0"),
        s("renamed"),
    ]));
    assert_eq!(&image(&cart)[SLOT], STIDCART_MENU);
    let listed = ok(&stidcart(&[s("list"), cart.as_os_str()]));
    assert!(listed.contains("menu: stidcart\n"), "{listed}");
    assert!(listed.contains("renamed"), "{listed}");
}
