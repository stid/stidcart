//! AC-31, AC-32 and AC-39 of the **cart-entries** work-item — the directory encoding of cartridge
//! kinds, the geometry a directory may not describe, and how a cartridge is derived from an image.
//!
//! All three are `surfaces: none`: library behaviour with no command-line surface of its own, so
//! these tests are the coverage rather than the audit being it.

use std::path::{Path, PathBuf};

use stidcart_core::cartridge::{Cartridge, CartridgeError};
use stidcart_core::directory::{self, DirEntry, EntryType, Slot, ENTRY_LEN};
use stidcart_core::image::Half;
use stidcart_core::read::{self, ReadError};
use stidcart_core::{crt, reserved, CartBuilder};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(fixtures_dir().join(name)).unwrap_or_else(|e| panic!("fixture {name}: {e}"))
}

/// The nine cartridge kinds, with the type value the format assigns each (`easyfs.c:64-80`).
const CART_KINDS: [(EntryType, u8); 9] = [
    (EntryType::K8, 0x10),
    (EntryType::K16, 0x11),
    (EntryType::Ultimax16, 0x12),
    (EntryType::Ultimax8, 0x13),
    (EntryType::Ocean512, 0x14),
    (EntryType::Ocean, 0x15),
    (EntryType::Xbank8, 0x1c),
    (EntryType::Xbank16, 0x1d),
    (EntryType::XbankUltimax, 0x1e),
];

fn a_cart_entry(kind: EntryType) -> DirEntry {
    DirEntry {
        name: "some cart".into(),
        kind,
        bank: 5,
        offset: 0,
        size: 6 * 0x2000,
        hidden: true,
        align_64k: true,
        other_off: 2,
        other_num: 2,
    }
}

/// AC-31 (none): every cartridge kind round-trips through encode and decode with all its fields.
#[test]
fn ac_31_every_cartridge_kind_round_trips_through_encode_and_decode() {
    for (kind, type_value) in CART_KINDS {
        let e = a_cart_entry(kind);
        let raw = directory::encode(&e);
        assert_eq!(
            raw[16] & directory::TYPE_MASK,
            type_value,
            "{kind:?} must encode as type {type_value:#04x}"
        );
        // CrtUsage: MSB always set, alignment at bit 13, opposite offset in bits 8-12, count low.
        let usage = u16::from_le_bytes([raw[19], raw[20]]);
        assert_eq!(
            usage,
            0x8000 | 0x2000 | (2 << 8) | 2,
            "CrtUsage word for {kind:?}"
        );

        let Slot::Cartridge(back) = directory::decode(&raw) else {
            panic!(
                "{kind:?} should decode as a cartridge, got {:?}",
                directory::decode(&raw)
            );
        };
        assert_eq!(back, e, "{kind:?} did not survive the round trip");
        // The primary half's bank count is derived, and must agree with what went in: 6 banks
        // total, 2 in the opposite half.
        assert_eq!(back.size / 0x2000 - u32::from(back.other_num), 4);
    }

    // The primary half depends on the kind: bit 1 of the type value says the opposite half is
    // ROML, i.e. the primary is ROMH — the three Ultimax kinds.
    for (kind, type_value) in CART_KINDS {
        let want = if type_value & 2 != 0 {
            Half::RomH
        } else {
            Half::RomL
        };
        assert_eq!(kind.primary(), want, "primary half of {kind:?}");
    }
}

/// Put raw slot bytes into the leading directory slots of an otherwise valid empty cart.
fn cart_with_slots(slots: &[[u8; ENTRY_LEN]]) -> Vec<u8> {
    let mut img = CartBuilder::new()
        .build_image()
        .expect("an empty image builds");
    for (n, slot) in slots.iter().enumerate() {
        img.write_at(reserved::DIRECTORY_OFFSET + n * ENTRY_LEN, slot.as_slice());
    }
    crt::write(&img, "probe")
}

/// AC-32 (none): a cartridge entry reaching past flash, or whose opposite half exceeds its size, is
/// refused as malformed at its slot — never read as a smaller entry, never through an underflow.
#[test]
fn ac_32_a_cartridge_entry_with_impossible_geometry_is_refused_at_its_slot() {
    // Primary run past the top: 4 primary banks starting at bank 62.
    let mut past_primary = a_cart_entry(EntryType::K16);
    past_primary.bank = 62;
    past_primary.other_off = 0;
    past_primary.other_num = 0;
    past_primary.size = 4 * 0x2000;

    // Opposite run past the top: starts at bank 60, opposite half at +3 for 2 banks → bank 65.
    let mut past_other = a_cart_entry(EntryType::Xbank16);
    past_other.bank = 60;
    past_other.other_off = 3;
    past_other.other_num = 2;
    past_other.size = 3 * 0x2000;

    // Opposite half larger than the whole entry: the subtraction the reference tool got wrong.
    let mut oversized_other = a_cart_entry(EntryType::K8);
    oversized_other.bank = 1;
    oversized_other.other_off = 0;
    oversized_other.other_num = 3;
    oversized_other.size = 0x2000;

    for (what, entry) in [
        ("primary run past bank 63", past_primary),
        ("opposite run past bank 63", past_other),
        ("opposite half larger than the entry", oversized_other),
    ] {
        let image = cart_with_slots(&[directory::encode(&entry)]);
        let err = read::inspect(&image)
            .err()
            .unwrap_or_else(|| panic!("{what}: should be refused"));
        assert!(
            matches!(err, ReadError::BadGeometry { slot: 0 }),
            "{what}: expected BadGeometry at slot 0, got {err:?}"
        );
        assert!(
            !matches!(err, ReadError::Unsupported(_)),
            "{what}: malformed, not unsupported"
        );
    }
}

/// Bank body byte `i` of input file `k`, bank `b`, half `h` — the formula `fixtures/make_inputs.py`
/// records, so the bytes a cartridge yields can be checked against what the generator wrote.
fn body(k: usize, b: usize, h: usize) -> Vec<u8> {
    (0..0x2000)
        .map(|i| ((i * 7 + 3 + 5 * k + 2 * b + h) % 251) as u8)
        .collect()
}

fn banks(k: usize, range: std::ops::Range<usize>, h: usize) -> Vec<u8> {
    range.flat_map(|b| body(k, b, h)).collect()
}

/// A `.crt` of any id, for the refusal cases the checked-in inputs do not cover.
fn crt(id: u16, exrom: u8, game: u8, packets: &[(u16, u16, Vec<u8>)]) -> Vec<u8> {
    let mut out = vec![0u8; 0x40];
    out[..16].copy_from_slice(b"C64 CARTRIDGE   ");
    out[0x10..0x14].copy_from_slice(&0x40u32.to_be_bytes());
    out[0x14..0x16].copy_from_slice(&0x0100u16.to_be_bytes());
    out[0x16..0x18].copy_from_slice(&id.to_be_bytes());
    out[0x18] = exrom;
    out[0x19] = game;
    out[0x20..0x25].copy_from_slice(b"probe");
    for (bank, addr, data) in packets {
        out.extend_from_slice(b"CHIP");
        out.extend_from_slice(&(0x10 + data.len() as u32).to_be_bytes());
        out.extend_from_slice(&0u16.to_be_bytes());
        out.extend_from_slice(&bank.to_be_bytes());
        out.extend_from_slice(&addr.to_be_bytes());
        out.extend_from_slice(&(data.len() as u16).to_be_bytes());
        out.extend_from_slice(data);
    }
    out
}

/// AC-39 (none): a cartridge's kind and bank runs are derived from its image; anything else is
/// refused naming what it was.
#[test]
fn ac_39_a_cartridge_is_derived_from_its_image_or_refused_by_what_it_is() {
    // (input, generator index, kind, ROML run, ROML offset, ROMH run, ROMH offset)
    let cases: [(
        &str,
        usize,
        EntryType,
        std::ops::Range<usize>,
        u8,
        std::ops::Range<usize>,
        u8,
    ); 10] = [
        ("in_8k.crt", 0, EntryType::K8, 0..1, 0, 0..0, 0),
        ("in_16k.crt", 1, EntryType::K16, 0..1, 0, 0..1, 0),
        ("in_ult8.crt", 2, EntryType::Ultimax8, 0..0, 0, 0..1, 0),
        ("in_ult16.crt", 3, EntryType::Ultimax16, 0..1, 0, 0..1, 0),
        ("in_ocean.crt", 4, EntryType::Ocean, 0..16, 0, 16..32, 16),
        ("in_ocean512.crt", 5, EntryType::Ocean512, 0..64, 0, 0..0, 0),
        ("in_xb8.crt", 6, EntryType::Xbank8, 0..4, 0, 0..0, 0),
        ("in_xb16.crt", 7, EntryType::Xbank16, 0..4, 0, 0..4, 0),
        ("in_xbult.crt", 8, EntryType::XbankUltimax, 0..2, 0, 0..2, 0),
        ("in_xbwide.crt", 9, EntryType::Xbank16, 0..40, 0, 0..40, 0),
    ];
    for (input, k, kind, l, l_at, h, h_at) in cases {
        let c = Cartridge::from_crt(&fixture(input))
            .unwrap_or_else(|e| panic!("{input} should derive a cartridge: {e:?}"));
        assert_eq!(c.kind, kind, "{input}: kind");
        assert_eq!(c.banks(), (l.len(), h.len()), "{input}: bank counts");
        assert_eq!((c.roml_at, c.romh_at), (l_at, h_at), "{input}: run offsets");
        // Ocean numbers its ROMH chips straight on from the ROML ones, so the ROMH bytes are the
        // generator's banks 16..32 of half 1; every other input starts each half at chip 0.
        let (want_l, want_h) = if kind == EntryType::Ocean {
            (banks(k, 0..16, 0), banks(k, 16..32, 1))
        } else {
            (banks(k, l.clone(), 0), banks(k, h.clone(), 1))
        };
        assert_eq!(c.roml, want_l, "{input}: ROML bytes");
        assert_eq!(c.romh, want_h, "{input}: ROMH bytes");
        assert_eq!(
            c.name,
            input.trim_end_matches(".crt"),
            "{input}: name from the header"
        );
        assert!(!c.align_64k && !c.hidden);
    }

    // An EasyFlash multicart is not a cartridge entry, and the refusal names the id.
    assert!(
        matches!(
            Cartridge::from_crt(&fixture("two.crt")),
            Err(CartridgeError::UnsupportedId { id: 32 })
        ),
        "an EasyFlash image (id 32) is refused by id"
    );
    // An unlisted id likewise.
    assert!(matches!(
        Cartridge::from_crt(&crt(7, 0, 0, &[(0, 0x8000, vec![0; 0x2000])])),
        Err(CartridgeError::UnsupportedId { id: 7 })
    ));
    // A short generic packet is not a whole bank (D-023).
    assert!(Cartridge::from_crt(&crt(0, 0, 1, &[(0, 0x8000, vec![0; 0x1000])])).is_err());
    // 64 KiB is not an Ocean size the reference tool lists.
    let ocean64: Vec<(u16, u16, Vec<u8>)> =
        (0..8).map(|b| (b, 0x8000, vec![b as u8; 0x2000])).collect();
    assert!(Cartridge::from_crt(&crt(5, 0, 0, &ocean64)).is_err());
    // An xbank with mode "off" in its header, or whose primary run does not start at bank 0.
    assert!(Cartridge::from_crt(&crt(33, 1, 1, &[(0, 0x8000, vec![0; 0x2000])])).is_err());
    assert!(Cartridge::from_crt(&crt(33, 0, 1, &[(3, 0x8000, vec![0; 0x2000])])).is_err());
    // Not a cartridge at all.
    assert!(matches!(
        Cartridge::from_crt(&fixture("hello.prg")),
        Err(CartridgeError::NotACartridge)
    ));
}
