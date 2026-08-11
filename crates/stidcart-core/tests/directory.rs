//! AC-7 and AC-9 — what a directory entry records, and how the directory ends.

use stidcart_core::directory::{EntryType, ENTRY_LEN, MAX_ENTRIES};
use stidcart_core::reserved::DIRECTORY_OFFSET;
use stidcart_core::{BuildError, CartBuilder, Program};

/// Pull entry `i` out of a built image and decode the fields the format defines.
struct Slot {
    flags: u8,
    bank: u16,
    offset: u16,
    size: u32,
}

fn slot(img: &stidcart_core::FlashImage, i: usize) -> Slot {
    let d = img.as_bytes();
    let p = &d[DIRECTORY_OFFSET + i * ENTRY_LEN..][..ENTRY_LEN];
    Slot {
        flags: p[16],
        bank: u16::from_le_bytes([p[17], p[18]]),
        offset: u16::from_le_bytes([p[19], p[20]]),
        size: u32::from_le_bytes([p[21], p[22], p[23], 0]),
    }
}

#[test]
fn ac_7_none_entry_geometry_round_trips() {
    // A program's bytes include its two-byte load address, and reading back the location the
    // entry records must yield them unchanged.
    let mut data = vec![0x01, 0x08];
    data.extend((0..500u16).map(|i| (i % 251) as u8));

    let img = CartBuilder::new()
        .with_program(Program::new("round trip", data.clone()))
        .build_image()
        .unwrap();

    let s = slot(&img, 0);
    assert_eq!(s.size as usize, data.len(), "recorded size");
    assert_eq!(
        s.flags & 0x1f,
        EntryType::Prg as u8,
        "an interleaved program entry"
    );

    let read_back = img.read_program(s.bank as usize, s.offset as usize, s.size as usize);
    assert_eq!(read_back, data, "bytes at the recorded location");
    assert_eq!(&read_back[..2], &[0x01, 0x08], "load address preserved");
}

#[test]
fn ac_7_none_entries_do_not_land_in_the_reserved_bank() {
    let img = CartBuilder::new()
        .with_program(Program::new("a", vec![0x01, 0x08, 1, 2, 3]))
        .with_program(Program::new("b", vec![0x01, 0x08, 4, 5, 6]))
        .build_image()
        .unwrap();

    for i in 0..2 {
        assert!(
            slot(&img, i).bank >= 1,
            "entry {i} was placed in bank 0, which is reserved"
        );
    }
}

#[test]
fn ac_9_none_directory_ends_with_an_end_marker() {
    let img = CartBuilder::new()
        .with_program(Program::new("only", vec![0x01, 0x08, 0xaa]))
        .build_image()
        .unwrap();

    let d = img.as_bytes();
    let end = &d[DIRECTORY_OFFSET + ENTRY_LEN..][..ENTRY_LEN];
    assert_eq!(
        end, [0xff; ENTRY_LEN],
        "the slot after the last entry must be an end marker"
    );
    assert_eq!(
        end[16] & 0x1f,
        EntryType::End as u8,
        "an erased slot reads back as the end-marker type"
    );
}

#[test]
fn ac_9_none_too_many_entries_is_refused_not_truncated() {
    let mut b = CartBuilder::new();
    // Each program is tiny, so this hits the entry cap rather than running out of space.
    for i in 0..MAX_ENTRIES {
        b = b.with_program(Program::new(&format!("e{i}"), vec![0x01, 0x08, 0x00]));
    }
    // The cap itself is fine.
    assert!(
        b.clone().build_image().is_ok(),
        "{MAX_ENTRIES} entries should fit"
    );

    let over = b.with_program(Program::new("one too many", vec![0x01, 0x08, 0x00]));
    assert!(
        matches!(over.build_image(), Err(BuildError::TooManyEntries { .. })),
        "exceeding the entry cap must be an error, not a silent truncation"
    );
}
