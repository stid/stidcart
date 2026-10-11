//! Opening a cartridge from the GUI's Rust side (gui-shell).
//!
//! `stidcart_gui::open` is what the `open_cartridge` command wraps, so these tests reach the same
//! code the window does, with no webview. Equality with `list` is a chain: `listing::describe` is
//! held to `list`'s frozen output in `crates/stidcart-core/tests/listing.rs`, and these tests hold
//! `open` to `describe`.

use std::path::{Path, PathBuf};

use stidcart_core::directory::{self, DirEntry, EntryType};
use stidcart_core::{crt, listing, reserved, CartBuilder};
use stidcart_gui::{open, ListingDto};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn every_fixture() -> Vec<PathBuf> {
    let mut crts: Vec<PathBuf> = std::fs::read_dir(fixtures())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "crt"))
        .collect();
    crts.sort();
    assert!(!crts.is_empty(), "no fixtures found");
    crts
}

/// A scratch directory unique to the calling test.
fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("stidcart-gui-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn json(dto: &ListingDto) -> serde_json::Value {
    serde_json::to_value(dto).unwrap()
}

/// An EasyFS2 image whose first slot claims the old format (D-017), built as `list.rs` builds it.
fn easyfs1_image() -> Vec<u8> {
    let mut img = CartBuilder::new().build_image().unwrap();
    let mut slot = directory::encode(&DirEntry {
        name: "ancient".into(),
        kind: EntryType::Prg,
        bank: 1,
        offset: 0,
        size: 2,
        hidden: false,
        align_64k: false,
        subdir: false,
        other_off: 0,
        other_num: 0,
    });
    slot[16] &= !directory::RESERVED_FLAG_BITS;
    img.write_at(reserved::DIRECTORY_OFFSET, &slot);
    crt::write(&img, "probe")
}

/// The refusals the spec names: old format, malformed/truncated, not a container, other hardware.
fn refused_inputs() -> Vec<(&'static str, Vec<u8>)> {
    let two = std::fs::read(fixtures().join("two.crt")).unwrap();
    vec![
        ("easyfs1", easyfs1_image()),
        ("truncated", two[..two.len() / 2].to_vec()),
        (
            "not-a-crt",
            b"this is not a cartridge image at all".to_vec(),
        ),
        (
            "other-hardware",
            std::fs::read(fixtures().join("in_8k.crt")).unwrap(),
        ),
    ]
}

// AC-51 (RENDER): every fixture opens to the listing list shows
#[test]
fn ac_51_render_every_fixture_opens_to_the_shared_listing() {
    for path in every_fixture() {
        let bytes = std::fs::read(&path).unwrap();
        match listing::describe(&bytes) {
            Ok(expected) => {
                let got = open(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
                assert_eq!(
                    json(&got),
                    json(&ListingDto::from(&expected)),
                    "{}",
                    path.display()
                );
                assert_eq!(
                    json(&got)["freeText"],
                    serde_json::Value::from(expected.free_text()),
                    "{}: the free text is list's",
                    path.display()
                );
            }
            Err(e) => assert_eq!(open(&path).unwrap_err(), e.to_string()),
        }
    }
}

// AC-54 (RENDER): a refused file gives list's reason, and an unreadable path says which path
#[test]
fn ac_54_render_a_refused_file_gives_lists_reason() {
    let dir = scratch("refused");
    for (tag, bytes) in refused_inputs() {
        let path = dir.join(format!("{tag}.crt"));
        std::fs::write(&path, &bytes).unwrap();
        let expected = listing::describe(&bytes).expect_err(tag).to_string();
        assert_eq!(open(&path).unwrap_err(), expected, "{tag}");
    }

    let missing = dir.join("no-such.crt");
    let err = open(&missing).unwrap_err();
    assert!(
        err.starts_with(&format!("reading '{}': ", missing.display())),
        "a read failure names the path the way list does, got: {err}"
    );
}

// AC-58: opening a file never writes to it, whether it opens or is refused
#[test]
fn ac_58_opening_never_writes_to_the_file() {
    let dir = scratch("readonly");
    let mut inputs: Vec<(String, Vec<u8>)> = vec![(
        "two".into(),
        std::fs::read(fixtures().join("two.crt")).unwrap(),
    )];
    inputs.extend(
        refused_inputs()
            .into_iter()
            .map(|(t, b)| (t.to_string(), b)),
    );

    for (tag, bytes) in inputs {
        let path = dir.join(format!("{tag}.crt"));
        std::fs::write(&path, &bytes).unwrap();
        let before = std::fs::metadata(&path).unwrap().modified().unwrap();
        let _ = open(&path);
        assert_eq!(std::fs::read(&path).unwrap(), bytes, "{tag}: bytes changed");
        assert_eq!(
            std::fs::metadata(&path).unwrap().modified().unwrap(),
            before,
            "{tag}: file was rewritten"
        );
    }
}
