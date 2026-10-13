//! Classifying an input file as the entry it is (D-034, gui-edit).
//!
//! `edit --add`, `edit --replace`, `build` and the GUI all decide what a file is through
//! `Entry::from_input`, so they cannot disagree about it. These tests pin the decision and the
//! refusal text, which is the text the command line has always printed.

use std::path::{Path, PathBuf};

use stidcart_core::cartridge::Cartridge;
use stidcart_core::directory::EntryType;
use stidcart_core::{Entry, InputError};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(fixtures_dir().join(name)).unwrap_or_else(|e| panic!("fixture {name}: {e}"))
}

#[test]
fn a_file_without_the_crt_signature_is_a_program_under_the_given_name() {
    let data = fixture("hello.prg");
    let entry = Entry::from_input("hello", data.clone(), false).unwrap();
    let program = entry.as_program().expect("a .prg is a program");
    assert_eq!(program.name, "hello");
    assert_eq!(program.data, data);
    assert!(!program.hidden);
}

#[test]
fn a_cartridge_image_becomes_the_cartridge_it_derives_to_under_the_given_name() {
    for (file, kind) in [
        ("in_8k.crt", EntryType::K8),
        ("in_16k.crt", EntryType::K16),
        ("in_ocean.crt", EntryType::Ocean),
        ("in_xb16.crt", EntryType::Xbank16),
    ] {
        let data = fixture(file);
        let entry = Entry::from_input("given", data.clone(), false).unwrap();
        let cart = entry
            .as_cartridge()
            .unwrap_or_else(|| panic!("{file} is a cartridge"));
        assert_eq!(cart.kind, kind, "{file}");
        assert_eq!(
            cart.name, "given",
            "{file}: the caller's name wins over the header's"
        );
        assert!(!cart.align_64k, "{file}");
        assert_eq!(
            *cart,
            Cartridge::from_crt(&data).unwrap().with_name("given"),
            "{file}: classification is exactly from_crt"
        );
    }
}

#[test]
fn alignment_requested_on_an_xbank_is_recorded() {
    let entry = Entry::from_input("x", fixture("in_xb16.crt"), true).unwrap();
    assert!(entry.as_cartridge().unwrap().align_64k);
}

#[test]
fn a_refused_cartridge_image_carries_the_derivation_error() {
    let mut bad = fixture("in_8k.crt");
    bad.truncate(0x50);
    let err = Entry::from_input("bad", bad.clone(), false).unwrap_err();
    let expected = Cartridge::from_crt(&bad).unwrap_err();
    assert!(
        matches!(err, InputError::Cartridge(ref e) if *e == expected),
        "{err:?}"
    );
    assert_eq!(err.to_string(), expected.to_string());
}

#[test]
fn alignment_on_a_program_is_refused_with_the_command_line_text() {
    let err = Entry::from_input("hello", fixture("hello.prg"), true).unwrap_err();
    assert_eq!(
        err.to_string(),
        "'hello' is a program, not an xbank cartridge, so 64 KB alignment does not apply to it"
    );
}

#[test]
fn alignment_on_a_cartridge_that_is_not_an_xbank_is_refused_with_the_command_line_text() {
    let err = Entry::from_input("in_8k", fixture("in_8k.crt"), true).unwrap_err();
    assert_eq!(
        err.to_string(),
        "'in_8k' is not an xbank cartridge, so 64 KB alignment does not apply to it"
    );
}
