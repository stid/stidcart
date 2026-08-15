//! AC-17..AC-28 of the **cart-edit** work-item — `stidcart edit`, driven as a binary, plus the
//! hidden marker `build` gains alongside it (AC-28).
//!
//! Every case works on a copy in its own scratch directory, because the command's default is to
//! write over the cart it was given: a test that edited a fixture in place would destroy the very
//! thing the parity suite compares against.

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
    let dir = std::env::temp_dir().join(format!("stidcart-edit-{tag}"));
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

/// The entries a cart holds, as the image records them: name, hidden flag and size.
fn entries(path: &Path) -> Vec<(String, bool, u32)> {
    let info = read::inspect(&bytes(path)).expect("the edited cart is readable");
    info.entries
        .iter()
        .map(|e| (e.name.clone(), e.hidden, e.size))
        .collect()
}

fn names(path: &Path) -> Vec<String> {
    entries(path).into_iter().map(|(n, _, _)| n).collect()
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

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

/// Write an image built by the core into the scratch directory, for the cases that need a cart no
/// fixture provides. Built through the public writers rather than by hand-editing offsets, so a
/// layout change moves the test with it.
fn write_cart(tag: &str, image_bytes: Vec<u8>) -> PathBuf {
    let path = scratch(tag).join("cart.crt");
    std::fs::write(&path, image_bytes).unwrap();
    path
}

/// AC-17 (CLI): `--add` appends entries named after their file stems.
#[test]
fn ac_17_cli_add_appends_entries_named_after_their_file_stems() {
    let cart = cart_copy("ac17", "two.crt");
    let out = edit(&cart, ["--add".into(), fixtures().join("big.prg")]);

    assert!(out.status.success(), "adding a program: {}", stderr(&out));
    assert_eq!(
        names(&cart),
        ["hello", "ramp", "big"],
        "the cart keeps every entry it had and gains one named after the input's stem"
    );
}

/// AC-18 (CLI): `--remove` drops exactly the indexed entry.
#[test]
fn ac_18_cli_remove_drops_exactly_the_indexed_entry() {
    let cart = cart_copy("ac18", "two.crt");
    let out = edit(&cart, ["--remove", "0"]);

    assert!(out.status.success(), "removing entry 0: {}", stderr(&out));
    assert_eq!(
        names(&cart),
        ["ramp"],
        "the indexed entry goes and the rest stay, in the order they were in"
    );
}

/// AC-19 (CLI): `--rename` changes one entry's name and no other's.
#[test]
fn ac_19_cli_rename_changes_one_entrys_name_and_no_other() {
    let cart = cart_copy("ac19", "two.crt");
    let out = edit(&cart, ["--rename", "1", "renamed"]);

    assert!(out.status.success(), "renaming entry 1: {}", stderr(&out));
    assert_eq!(names(&cart), ["hello", "renamed"]);
}

/// AC-20 (CLI): `--hide` and `--show` toggle one entry's flag.
#[test]
fn ac_20_cli_hide_and_show_toggle_one_entrys_flag() {
    let cart = cart_copy("ac20", "two.crt");

    let out = edit(&cart, ["--hide", "1"]);
    assert!(out.status.success(), "hiding entry 1: {}", stderr(&out));
    let flags: Vec<bool> = entries(&cart).into_iter().map(|(_, h, _)| h).collect();
    assert_eq!(flags, [false, true], "only the named entry is hidden");

    let out = edit(&cart, ["--show", "1"]);
    assert!(out.status.success(), "showing entry 1: {}", stderr(&out));
    let flags: Vec<bool> = entries(&cart).into_iter().map(|(_, h, _)| h).collect();
    assert_eq!(
        flags,
        [false, false],
        "clearing it returns the entry to visible"
    );
}

/// AC-21 (CLI): `--replace` swaps program data and keeps the name and position.
#[test]
fn ac_21_cli_replace_swaps_program_data_and_keeps_name_and_position() {
    let cart = cart_copy("ac21", "two.crt");
    let replacement = fixtures().join("big.prg");
    let out = edit(&cart, ["--replace".into(), "0".into(), replacement.clone()]);

    assert!(out.status.success(), "replacing entry 0: {}", stderr(&out));

    let after = entries(&cart);
    assert_eq!(
        after.iter().map(|(n, _, _)| n.as_str()).collect::<Vec<_>>(),
        ["hello", "ramp"],
        "the entry keeps its name and its place in the order"
    );
    assert_eq!(
        after[0].2 as usize,
        std::fs::metadata(&replacement).unwrap().len() as usize,
        "the stored size is the replacement's, so the data behind the name changed"
    );

    let reread = read::load(&bytes(&cart)).expect("the edited cart loads");
    assert_eq!(
        reread.programs()[0].data,
        std::fs::read(&replacement).unwrap(),
        "and the bytes are the replacement's, not merely its length"
    );
}

/// AC-22 (CLI): `-e` and `--header` set the menu and container names independently.
#[test]
fn ac_22_cli_menu_and_container_names_are_set_independently() {
    let cart = cart_copy("ac22", "named.crt");
    let before = read::inspect(&bytes(&cart)).unwrap();

    let out = edit(&cart, ["-e", "new menu name"]);
    assert!(
        out.status.success(),
        "setting the menu name: {}",
        stderr(&out)
    );
    let after = read::inspect(&bytes(&cart)).unwrap();
    assert_eq!(after.ef_name, "new menu name");
    assert_eq!(
        after.header_name, before.header_name,
        "setting the menu name leaves the container header name alone"
    );

    let out = edit(&cart, ["--header", "new tool name"]);
    assert!(
        out.status.success(),
        "setting the header name: {}",
        stderr(&out)
    );
    let after2 = read::inspect(&bytes(&cart)).unwrap();
    assert_eq!(after2.header_name, "new tool name");
    assert_eq!(
        after2.ef_name, "new menu name",
        "and setting the container name leaves the menu name alone"
    );
}

/// AC-23 (CLI): the edit lands in place by default, and `-o` leaves the input byte-identical.
#[test]
fn ac_23_cli_edits_land_in_place_by_default_and_o_redirects() {
    let cart = cart_copy("ac23-inplace", "two.crt");
    let before = bytes(&cart);
    let out = edit(&cart, ["--remove", "0"]);
    assert!(out.status.success(), "in-place edit: {}", stderr(&out));
    assert_ne!(
        bytes(&cart),
        before,
        "with no destination the cart itself holds the result"
    );
    assert_eq!(names(&cart), ["ramp"]);

    let cart = cart_copy("ac23-out", "two.crt");
    let before = bytes(&cart);
    let elsewhere = cart.parent().unwrap().join("edited.crt");
    let args: [std::ffi::OsString; 4] = [
        "--remove".into(),
        "0".into(),
        "-o".into(),
        elsewhere.clone().into(),
    ];
    let out = edit(&cart, args);
    assert!(out.status.success(), "redirected edit: {}", stderr(&out));
    assert_eq!(
        bytes(&cart),
        before,
        "a named destination leaves the input exactly as it was"
    );
    assert_eq!(
        names(&elsewhere),
        ["ramp"],
        "and the result is at the destination"
    );
}

/// AC-24 (CLI): a failed edit reports the reason, exits non-zero and leaves every input untouched.
#[test]
fn ac_24_cli_a_failed_edit_changes_nothing_and_says_why() {
    // An index naming no entry.
    let cart = cart_copy("ac24-index", "two.crt");
    let before = bytes(&cart);
    let out = edit(&cart, ["--remove", "7"]);
    assert!(
        !out.status.success(),
        "index 7 does not exist in a two-entry cart"
    );
    assert!(
        stderr(&out).contains("7"),
        "the message should name the index that failed, got: {}",
        stderr(&out)
    );
    assert_eq!(bytes(&cart), before, "a refused edit writes nothing");

    // A program file that cannot be read.
    let cart = cart_copy("ac24-missing", "two.crt");
    let before = bytes(&cart);
    let out = edit(
        &cart,
        ["--add".into(), cart.parent().unwrap().join("absent.prg")],
    );
    assert!(!out.status.success(), "an unreadable input is an error");
    assert!(!stderr(&out).is_empty(), "and it says so");
    assert_eq!(bytes(&cart), before);

    // A resulting set that does not fit: 60 copies of a 20000-byte program overruns the ~1 MB image.
    let cart = cart_copy("ac24-full", "two.crt");
    let before = bytes(&cart);
    let mut args: Vec<std::ffi::OsString> = Vec::new();
    for _ in 0..60 {
        args.push("--add".into());
        args.push(fixtures().join("big.prg").into());
    }
    let out = edit(&cart, args);
    assert!(
        !out.status.success(),
        "the resulting set cannot fit on the cartridge"
    );
    assert!(!stderr(&out).is_empty(), "and the shortfall is reported");
    assert_eq!(
        bytes(&cart),
        before,
        "the cart the user still has is the one they had"
    );
}

/// AC-25 (CLI): a cart that cannot be rebuilt unchanged is refused with the reason named.
#[test]
fn ac_25_cli_a_cart_that_cannot_be_rebuilt_unchanged_is_refused() {
    let mut img = CartBuilder::new()
        .build_image()
        .expect("an empty image builds");
    img.write_at(reserved::EAPI_OFFSET, &[0x00; 32]);
    let cart = write_cart("ac25", crt::write(&img, "probe"));
    let before = bytes(&cart);

    let out = edit(&cart, ["--rename", "0", "whatever"]);
    assert!(
        !out.status.success(),
        "a cart whose firmware differs from the embedded blobs cannot be saved unchanged"
    );
    assert!(
        stderr(&out).contains("rebuilt unchanged"),
        "the refusal should say what it could not do, got: {}",
        stderr(&out)
    );
    assert_eq!(bytes(&cart), before);
}

/// AC-26 (CLI): an EasyFS1 cart is refused by edit, naming the format.
#[test]
fn ac_26_cli_an_old_format_cart_is_refused_naming_the_format() {
    let entry = DirEntry {
        name: "probe".into(),
        kind: EntryType::Prg,
        bank: 1,
        offset: 0,
        size: 2,
        hidden: false,
    };
    let mut slot = directory::encode(&entry);
    slot[16] &= !directory::RESERVED_FLAG_BITS; // what makes it an old-format entry

    let mut img = CartBuilder::new()
        .build_image()
        .expect("an empty image builds");
    img.write_at(reserved::DIRECTORY_OFFSET, slot.as_slice());
    img.write_at(
        reserved::DIRECTORY_OFFSET + directory::ENTRY_LEN,
        &directory::end_marker(),
    );
    let cart = write_cart("ac26", crt::write(&img, "probe"));
    let before = bytes(&cart);

    let out = edit(&cart, ["--remove", "0"]);
    assert!(
        !out.status.success(),
        "an EasyFS1 directory is refused, not converted"
    );
    assert!(
        stderr(&out).contains("EasyFS"),
        "the refusal names the format, as listing one does, got: {}",
        stderr(&out)
    );
    assert_eq!(bytes(&cart), before);
}

/// AC-27 (CLI): every index in one invocation refers to the input listing.
#[test]
fn ac_27_cli_indices_refer_to_the_input_listing() {
    let cart = cart_copy("ac27", "two.crt");
    // Applied in sequence, the rename would land on nothing: after removing entry 0 the cart holds
    // one entry, at index 0. Resolved against the input, it renames what was entry 1.
    let out = edit(&cart, ["--remove", "0", "--rename", "1", "kept"]);

    assert!(
        out.status.success(),
        "both operations apply: {}",
        stderr(&out)
    );
    assert_eq!(
        names(&cart),
        ["kept"],
        "the surviving entry is the one index 1 named in the cart as given"
    );
}

/// AC-28 (CLI): `build --hide` marks that input hidden, and a build marking nothing is unchanged.
#[test]
fn ac_28_cli_build_marks_an_input_hidden_and_changes_nothing_otherwise() {
    let dir = scratch("ac28");
    let hidden_out = dir.join("hidden.crt");
    let plain_out = dir.join("plain.crt");
    let hello = fixtures().join("hello.prg");
    let ramp = fixtures().join("ramp.prg");

    let out = stidcart()
        .arg("build")
        .args(["-e", "marked"])
        .arg("-o")
        .arg(&hidden_out)
        .arg("--hide")
        .arg(&hello)
        .arg(&hello)
        .arg(&ramp)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "building with a marker: {}",
        stderr(&out)
    );
    assert_eq!(
        entries(&hidden_out)
            .into_iter()
            .map(|(n, h, _)| (n, h))
            .collect::<Vec<_>>(),
        [("hello".to_string(), true), ("ramp".to_string(), false)],
        "the marked input is hidden and the other is not"
    );

    let out = stidcart()
        .arg("build")
        .args(["-e", "marked"])
        .arg("-o")
        .arg(&plain_out)
        .arg(&hello)
        .arg(&ramp)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "building without a marker: {}",
        stderr(&out)
    );

    let unmarked = CartBuilder::new()
        .with_ef_name("marked")
        .with_program(stidcart_core::Program::new(
            "hello",
            std::fs::read(&hello).unwrap(),
        ))
        .with_program(stidcart_core::Program::new(
            "ramp",
            std::fs::read(&ramp).unwrap(),
        ))
        .build()
        .expect("the equivalent cart builds");
    assert_eq!(
        bytes(&plain_out),
        unmarked,
        "a build that marks nothing produces the bytes it produced before the marker existed"
    );
}
