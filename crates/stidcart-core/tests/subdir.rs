//! AC-47 and AC-50 of the **subdir-entries** work-item — the CrtUsage subdirectory bit.
//!
//! The reference tool carries the bit on an xbank entry and rejects it on any other cartridge kind
//! (format spec, "Subdirectories"). Type `0x1A` stays refused; that is pinned by AC-7 in `read.rs`.

use std::path::{Path, PathBuf};
use stidcart_core::directory::{self, DirEntry, EntryType, Slot};
use stidcart_core::read::{self, ReadError};
use stidcart_core::{crt, reserved, CartBuilder};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(fixtures_dir().join(name)).unwrap_or_else(|e| panic!("fixture {name}: {e}"))
}

/// A one-bank cartridge entry of `kind` at bank 1, well-formed apart from whatever a test sets.
fn a_cartridge(kind: EntryType) -> DirEntry {
    DirEntry {
        name: "probe".into(),
        kind,
        bank: 1,
        offset: 0,
        size: 0x2000,
        hidden: false,
        align_64k: false,
        subdir: false,
        other_off: 0,
        other_num: 0,
    }
}

/// AC-47 (none): an xbank entry carrying the subdirectory bit decodes, loads and saves back with the
/// bit set, and a load-and-save of a cart holding one is byte-identical.
#[test]
fn ac_47_a_subdirectory_flagged_xbank_is_carried_through_load_and_save() {
    for kind in [
        EntryType::Xbank8,
        EntryType::Xbank16,
        EntryType::XbankUltimax,
    ] {
        let entry = DirEntry {
            subdir: true,
            ..a_cartridge(kind)
        };
        let raw = directory::encode(&entry);
        assert_ne!(
            u16::from_le_bytes([raw[19], raw[20]]) & directory::USAGE_SUBDIR,
            0,
            "{kind:?}: encode writes the bit"
        );
        assert_eq!(
            directory::decode(&raw),
            Slot::Cartridge(entry),
            "{kind:?}: decode inverts encode, bit included"
        );
    }

    let input = fixture("in_cxb16sub.crt");
    let info = read::inspect(&input).expect("inspect describes a subdirectory-flagged xbank");
    let described = &info.entries[1];
    assert_eq!(described.kind, EntryType::Xbank16);
    assert!(described.subdir, "inspect carries the bit");

    let cart = read::load(&input).expect("load accepts a subdirectory-flagged xbank");
    let loaded = cart.entries()[1]
        .as_cartridge()
        .expect("entry 1 is the xbank");
    assert!(loaded.subdir, "load carries the bit into the model");
    assert_eq!(
        cart.build().unwrap(),
        input,
        "a load-and-save reproduces the cart byte for byte, bit included"
    );
}

/// AC-50 (none): the subdirectory bit on a cartridge kind other than xbank is refused as malformed
/// by both read paths, naming the slot.
#[test]
fn ac_50_the_subdirectory_bit_on_a_non_xbank_cartridge_is_malformed() {
    for kind in EntryType::CARTRIDGES.into_iter().filter(|k| !k.is_xbank()) {
        let mut raw = directory::encode(&a_cartridge(kind));
        let word = u16::from_le_bytes([raw[19], raw[20]]) | directory::USAGE_SUBDIR;
        raw[19..21].copy_from_slice(&word.to_le_bytes());

        // Slot 0 is a well-formed xbank, so the refusal has to name the slot that is wrong.
        let mut img = CartBuilder::new().build_image().unwrap();
        for (slot, bytes) in [
            directory::encode(&a_cartridge(EntryType::Xbank8)),
            raw,
            directory::end_marker(),
        ]
        .iter()
        .enumerate()
        {
            img.write_at(
                reserved::DIRECTORY_OFFSET + slot * directory::ENTRY_LEN,
                bytes,
            );
        }
        let bytes = crt::write(&img, "probe");

        for (path, got) in [
            ("inspect", read::inspect(&bytes).map(|_| ())),
            ("load", read::load(&bytes).map(|_| ())),
        ] {
            let err = got.expect_err(&format!("{kind:?}: {path} refuses the bit"));
            assert_eq!(
                err,
                ReadError::SubdirectoryBitMisplaced { slot: 1 },
                "{kind:?}: {path} names it malformed at its slot"
            );
            assert!(
                err.to_string().contains("slot 1"),
                "{kind:?}: the message names the slot: {err}"
            );
        }
    }
}
