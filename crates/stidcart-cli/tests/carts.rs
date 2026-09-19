//! AC-40..AC-44 of the **cart-entries** work-item — cartridge images as `build` and `edit` inputs,
//! 64 KB alignment, and what is refused, all driven as a binary.
//!
//! Every case works on a copy in its own scratch directory, because `edit` writes over the cart it
//! was given by default.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use stidcart_core::directory::{self, DirEntry, EntryType};
use stidcart_core::{crt, read, reserved, CartBuilder};

fn stidcart() -> Command {
    Command::new(env!("CARGO_BIN_EXE_stidcart"))
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

/// A scratch directory unique to the calling test, so cases never see each other's output.
fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("stidcart-carts-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Copy a fixture cart into the scratch directory and hand back the writable path.
fn cart_copy(tag: &str, fixture: &str) -> PathBuf {
    let dir = scratch(tag);
    let path = dir.join("cart.crt");
    std::fs::copy(fixtures().join(fixture), &path).unwrap();
    path
}

fn bytes(path: &Path) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

/// (name, kind, bank, hidden, aligned) of every entry, as the image records them.
fn entries(path: &Path) -> Vec<(String, EntryType, u16, bool, bool)> {
    let info = read::inspect(&bytes(path)).expect("the cart is readable");
    info.entries
        .iter()
        .map(|e| (e.name.clone(), e.kind, e.bank, e.hidden, e.align_64k))
        .collect()
}

fn edit<I, S>(cart: &Path, args: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    stidcart()
        .arg("edit")
        .arg(cart)
        .args(args)
        .output()
        .unwrap()
}

fn build<I, S>(args: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    stidcart().arg("build").args(args).output().unwrap()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn fx(name: &str) -> PathBuf {
    fixtures().join(name)
}

/// AC-40 (CLI): build accepts a cartridge image among its inputs, recognised by content, and a
/// programs-only build still produces the bytes it produced before.
#[test]
fn ac_40_cli_build_accepts_a_cartridge_image_recognised_by_content() {
    let dir = scratch("build-cart");
    let out = dir.join("out.crt");
    let result = build([
        "-o".as_ref(),
        out.as_os_str(),
        fx("hello.prg").as_os_str(),
        fx("in_8k.crt").as_os_str(),
    ]);
    assert!(result.status.success(), "stderr: {}", stderr(&result));
    let got = entries(&out);
    assert_eq!(got[0].0, "hello");
    assert_eq!(got[0].1, EntryType::Prg);
    assert_eq!(
        (got[1].0.as_str(), got[1].1),
        ("in_8k", EntryType::K8),
        "named after the file stem"
    );

    // The same image under a misleading name is still a cartridge — content decides.
    let disguised = dir.join("disguised.prg");
    std::fs::copy(fx("in_8k.crt"), &disguised).unwrap();
    let out2 = dir.join("out2.crt");
    let result = build(["-o".as_ref(), out2.as_os_str(), disguised.as_os_str()]);
    assert!(result.status.success(), "stderr: {}", stderr(&result));
    assert_eq!(entries(&out2)[0].1, EntryType::K8);
    assert_eq!(entries(&out2)[0].0, "disguised");

    // Programs only: the flash image equals the reference fixture's (the container header name
    // differs by design, D-006, so compare the images the containers describe).
    let out3 = dir.join("two.crt");
    let result = build([
        "-e".as_ref(),
        "two entries".as_ref(),
        "-o".as_ref(),
        out3.as_os_str(),
        fx("hello.prg").as_os_str(),
        fx("ramp.prg").as_os_str(),
    ]);
    assert!(result.status.success(), "stderr: {}", stderr(&result));
    let ours = crt::parse(&bytes(&out3)).unwrap();
    let theirs = crt::parse(&bytes(&fx("two.crt"))).unwrap();
    assert!(
        ours.image.as_bytes() == theirs.image.as_bytes(),
        "a programs-only build is unchanged"
    );
}

/// AC-41 (CLI): `--add` and `--replace` take cartridge images, and the entry's kind follows the data.
#[test]
fn ac_41_cli_add_and_replace_take_cartridge_images_and_the_kind_follows_the_data() {
    let cart = cart_copy("add-cart", "two.crt");
    let result = edit(&cart, ["--add".as_ref(), fx("in_16k.crt").as_os_str()]);
    assert!(result.status.success(), "stderr: {}", stderr(&result));
    let got = entries(&cart);
    assert_eq!(got.len(), 3);
    assert_eq!((got[2].0.as_str(), got[2].1), ("in_16k", EntryType::K16));

    let result = edit(
        &cart,
        [
            "--replace".as_ref(),
            "0".as_ref(),
            fx("in_8k.crt").as_os_str(),
        ],
    );
    assert!(result.status.success(), "stderr: {}", stderr(&result));
    let got = entries(&cart);
    assert_eq!(
        (got[0].0.as_str(), got[0].1),
        ("hello", EntryType::K8),
        "name and index kept, kind follows"
    );
    assert_eq!(got[1].0, "ramp");

    let cart = cart_copy("replace-back", "c8k.crt");
    let result = edit(
        &cart,
        [
            "--replace".as_ref(),
            "1".as_ref(),
            fx("hello.prg").as_os_str(),
        ],
    );
    assert!(result.status.success(), "stderr: {}", stderr(&result));
    let got = entries(&cart);
    assert_eq!(
        (got[1].0.as_str(), got[1].1),
        ("in_8k", EntryType::Prg),
        "a cartridge entry becomes a program"
    );
}

/// AC-42 (CLI): alignment is requested at build or add time and toggled by index, xbank only.
#[test]
fn ac_42_cli_alignment_is_requested_at_build_or_add_time_and_toggled_by_index() {
    let dir = scratch("align");
    let out = dir.join("out.crt");
    let result = build([
        "--align".as_ref(),
        fx("in_xb16.crt").as_os_str(),
        "-o".as_ref(),
        out.as_os_str(),
        fx("hello.prg").as_os_str(),
        fx("in_xb16.crt").as_os_str(),
    ]);
    assert!(result.status.success(), "stderr: {}", stderr(&result));
    let got = entries(&out);
    assert_eq!(
        (got[1].2, got[1].4),
        (8, true),
        "aligned: bank 8 with the request recorded"
    );

    let cart = cart_copy("add-aligned", "two.crt");
    let result = edit(
        &cart,
        ["--add-aligned".as_ref(), fx("in_xb16.crt").as_os_str()],
    );
    assert!(result.status.success(), "stderr: {}", stderr(&result));
    assert_eq!((entries(&cart)[2].2, entries(&cart)[2].4), (8, true));

    let result = edit(&cart, ["--unalign", "2"]);
    assert!(result.status.success(), "stderr: {}", stderr(&result));
    assert_eq!(
        (entries(&cart)[2].2, entries(&cart)[2].4),
        (1, false),
        "unaligned: first free run"
    );

    let result = edit(&cart, ["--align", "2"]);
    assert!(result.status.success(), "stderr: {}", stderr(&result));
    assert!(entries(&cart)[2].4);

    // Alignment on anything but an xbank is refused naming the entry, and nothing is written.
    let out_bad = dir.join("bad.crt");
    let result = build([
        "--align".as_ref(),
        fx("in_8k.crt").as_os_str(),
        "-o".as_ref(),
        out_bad.as_os_str(),
        fx("in_8k.crt").as_os_str(),
    ]);
    assert!(!result.status.success());
    assert!(
        stderr(&result).contains("in_8k"),
        "stderr: {}",
        stderr(&result)
    );
    assert!(!out_bad.exists());

    let before = bytes(&cart);
    let result = edit(&cart, ["--align", "0"]);
    assert!(!result.status.success());
    assert!(
        stderr(&result).contains("hello"),
        "stderr: {}",
        stderr(&result)
    );
    assert_eq!(
        bytes(&cart),
        before,
        "a refused edit leaves the cart byte-identical"
    );
}

/// AC-43 (CLI): every edit operation works on a cartridge entry by index; only the subdirectory
/// kind is still refused.
#[test]
fn ac_43_cli_every_edit_operation_works_on_a_cartridge_entry_by_index() {
    let cart = cart_copy("edit-carts", "cmixed.crt");
    let result = edit(
        &cart,
        ["--rename", "3", "renamed", "--hide", "0", "--remove", "1"],
    );
    assert!(result.status.success(), "stderr: {}", stderr(&result));
    let got = entries(&cart);
    let names: Vec<&str> = got.iter().map(|e| e.0.as_str()).collect();
    assert_eq!(
        names,
        ["in_8k", "in_ult16", "renamed", "hello", "ramp", "big"]
    );
    assert_eq!(got[2].1, EntryType::Xbank8, "renaming keeps the kind");
    assert!(got[0].3, "the 8k cart is hidden");

    let result = edit(&cart, ["--show", "0"]);
    assert!(result.status.success(), "stderr: {}", stderr(&result));
    assert!(!entries(&cart)[0].3);

    // A subdirectory entry (type 0x1A) is the one kind still refused, by name.
    let mut img = CartBuilder::new().build_image().unwrap();
    let mut raw = directory::encode(&DirEntry {
        name: "nested".into(),
        kind: EntryType::K8,
        bank: 1,
        offset: 0,
        size: 0x2000,
        hidden: false,
        align_64k: false,
        other_off: 0,
        other_num: 0,
    });
    raw[16] = 0x1a | directory::RESERVED_FLAG_BITS;
    img.write_at(reserved::DIRECTORY_OFFSET, &raw);
    let path = scratch("subdir").join("cart.crt");
    std::fs::write(&path, crt::write(&img, "probe")).unwrap();
    let before = bytes(&path);
    let result = edit(&path, ["--rename", "0", "x"]);
    assert!(!result.status.success());
    assert!(
        stderr(&result).contains("subdirectory"),
        "stderr: {}",
        stderr(&result)
    );
    assert_eq!(bytes(&path), before);
}

/// AC-44 (CLI): an unsupported cartridge image is refused naming what it is, and every file is left
/// untouched.
#[test]
fn ac_44_cli_an_unsupported_cartridge_image_is_refused_and_files_are_untouched() {
    let dir = scratch("unsupported");
    let out = dir.join("out.crt");
    let result = build([
        "-o".as_ref(),
        out.as_os_str(),
        fx("hello.prg").as_os_str(),
        fx("two.crt").as_os_str(),
    ]);
    assert!(!result.status.success());
    assert!(
        stderr(&result).contains("32"),
        "the id is named: {}",
        stderr(&result)
    );
    assert!(!out.exists(), "nothing is written");

    let cart = cart_copy("unsupported-add", "c8k.crt");
    let before = bytes(&cart);
    let result = edit(&cart, ["--add".as_ref(), fx("two.crt").as_os_str()]);
    assert!(!result.status.success());
    assert!(
        stderr(&result).contains("32"),
        "stderr: {}",
        stderr(&result)
    );
    assert_eq!(bytes(&cart), before);
}
