//! replace-loader: an explicit `edit --menu` replaces a loader slot that holds no known menu
//! (AC-132, AC-133, D-050).

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use stidcart_core::menu::STIDCART_MENU;
use stidcart_core::reserved::{EAPI_OFFSET, EFNAME_MAGIC_OFFSET};
use stidcart_core::{crt, firmware};

/// The menu loader's slot: bank 0 ROML, the first 8 KiB of the image.
const SLOT: std::ops::Range<usize> = 0..0x2000;

fn fx(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(name)
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("stidcart-replace-loader-{tag}"));
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

fn refused(out: &Output) -> String {
    assert!(
        !out.status.success(),
        "stidcart accepted a request it should refuse"
    );
    String::from_utf8_lossy(&out.stderr).into_owned()
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

/// Two programs built with EasyLoader, then `patch` applied to the flash image: a cart whose loader
/// slot holds something this tool doesn't know.
fn foreign(dir: &Path, patch: impl Fn(&mut stidcart_core::FlashImage)) -> PathBuf {
    let built = dir.join("built.crt");
    ok(&stidcart(&[
        s("build"),
        s("-e"),
        s("foreign"),
        s("-o"),
        built.as_os_str(),
        fx("hello.prg").as_os_str(),
        fx("ramp.prg").as_os_str(),
    ]));
    let mut container = crt::parse(&std::fs::read(&built).unwrap()).unwrap();
    let loader: Vec<u8> = (0..SLOT.len()).map(|i| (i * 13 + 5) as u8).collect();
    container.image.write_at(SLOT.start, &loader);
    patch(&mut container.image);
    let out = dir.join("foreign.crt");
    std::fs::write(&out, crt::write(&container.image, &container.header_name)).unwrap();
    out
}

// AC-132 (CLI): without --menu, edit still refuses a loader slot holding no known menu
#[test]
fn ac_132_cli_without_menu_edit_still_refuses() {
    let dir = scratch("refuse");
    let cart = foreign(&dir, |_| {});
    let listed = ok(&stidcart(&[s("list"), cart.as_os_str()]));
    assert!(listed.contains("menu: unrecognised\n"), "{listed}");
    let err = refused(&stidcart(&[
        s("edit"),
        cart.as_os_str(),
        s("--rename"),
        s("0"),
        s("x"),
    ]));
    assert!(err.contains("menu loader"), "{err}");
}

// AC-132 (CLI): edit --menu replaces the slot with that menu and changes nothing else
#[test]
fn ac_132_cli_edit_menu_replaces_the_slot() {
    let dir = scratch("replace");
    let cart = foreign(&dir, |_| {});
    let before = image(&cart);
    let easyloader = firmware::body(firmware::LOADER_NRM);
    for (menu, slot, listed_as) in [
        ("stidcart", STIDCART_MENU, "stidcart"),
        ("easyloader", easyloader, "EasyLoader"),
    ] {
        let out = dir.join(format!("{menu}.crt"));
        ok(&stidcart(&[
            s("edit"),
            cart.as_os_str(),
            s("-o"),
            out.as_os_str(),
            s("--menu"),
            s(menu),
        ]));
        let after = image(&out);
        assert_eq!(
            &after[..slot.len()],
            slot,
            "{menu}: the slot was not replaced"
        );
        assert_eq!(
            &before[SLOT.end..],
            &after[SLOT.end..],
            "{menu}: something outside the slot moved"
        );
        let listed = ok(&stidcart(&[s("list"), out.as_os_str()]));
        assert!(listed.contains(&format!("menu: {listed_as}\n")), "{listed}");
    }
}

// AC-132 (CLI): the other operations in the same call apply
#[test]
fn ac_132_cli_other_operations_apply_with_the_switch() {
    let dir = scratch("with");
    let cart = foreign(&dir, |_| {});
    ok(&stidcart(&[
        s("edit"),
        cart.as_os_str(),
        s("--menu"),
        s("stidcart"),
        s("--rename"),
        s("0"),
        s("renamed"),
    ]));
    let listed = ok(&stidcart(&[s("list"), cart.as_os_str()]));
    assert!(listed.contains("menu: stidcart\n"), "{listed}");
    assert!(listed.contains("renamed"), "{listed}");
}

// AC-133 (CLI): EAPI or EF-Name magic differing is still refused, even with --menu
#[test]
fn ac_133_cli_other_firmware_differing_is_still_refused() {
    let dir = scratch("eapi");
    for (region, at) in [
        ("EAPI", EAPI_OFFSET),
        ("EF-Name magic", EFNAME_MAGIC_OFFSET),
    ] {
        let cart = foreign(&dir, |img| {
            let byte = img.as_bytes()[at];
            img.write_at(at, &[byte ^ 0xff]);
        });
        let err = refused(&stidcart(&[
            s("edit"),
            cart.as_os_str(),
            s("--menu"),
            s("stidcart"),
        ]));
        assert!(err.contains(region), "{region}: {err}");
    }
}
