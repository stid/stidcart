//! AC-10 — a program larger than the space left in its bank continues in the order the format
//! prescribes, and reads back unbroken across the boundary.

use stidcart_core::directory::ENTRY_LEN;
use stidcart_core::image::BANK_SIZE;
use stidcart_core::reserved::DIRECTORY_OFFSET;
use stidcart_core::{CartBuilder, Program};

fn entry_geometry(img: &stidcart_core::FlashImage, i: usize) -> (usize, usize, usize) {
    let d = img.as_bytes();
    let p = &d[DIRECTORY_OFFSET + i * ENTRY_LEN..][..ENTRY_LEN];
    (
        u16::from_le_bytes([p[17], p[18]]) as usize,
        u16::from_le_bytes([p[19], p[20]]) as usize,
        u32::from_le_bytes([p[21], p[22], p[23], 0]) as usize,
    )
}

/// Distinct bytes, so a mis-stitched boundary shows up as wrong data rather than as zeros.
fn ramp(len: usize) -> Vec<u8> {
    let mut v = vec![0x01, 0x08];
    v.extend((0..len - 2).map(|i| (i % 251) as u8));
    v
}

#[test]
fn ac_10_none_program_continues_across_a_bank_boundary() {
    // One byte over a full bank: the continuation is forced and lands in the other half.
    let data = ramp(BANK_SIZE + 1);
    let img = CartBuilder::new()
        .with_program(Program::new("spills", data.clone()))
        .build_image()
        .unwrap();

    let (bank, offset, size) = entry_geometry(&img, 0);
    assert_eq!(size, data.len());
    assert_eq!(
        img.read_program(bank, offset, size),
        data,
        "bytes across the boundary"
    );
}

#[test]
fn ac_10_none_program_spanning_several_banks_reads_back_unbroken() {
    let data = ramp(BANK_SIZE * 3 + 17);
    let img = CartBuilder::new()
        .with_program(Program::new("long", data.clone()))
        .build_image()
        .unwrap();

    let (bank, offset, size) = entry_geometry(&img, 0);
    assert_eq!(img.read_program(bank, offset, size), data);
}

#[test]
fn ac_10_none_second_program_starts_where_the_first_stopped() {
    // Measured against the reference tool: a first program of 0x2002 bytes fills bank 1 ROML and
    // two bytes of bank 1 ROMH, and the next program starts at offset 0x2002 in the same bank —
    // the high bit of the offset word being what selects the other half.
    let first = ramp(0x2002);
    let second = ramp(0x2002);
    let img = CartBuilder::new()
        .with_program(Program::new("first", first.clone()))
        .with_program(Program::new("second", second.clone()))
        .build_image()
        .unwrap();

    assert_eq!(entry_geometry(&img, 0).0, 1, "first entry bank");
    assert_eq!(entry_geometry(&img, 0).1, 0x0000, "first entry offset");
    assert_eq!(entry_geometry(&img, 1).0, 1, "second entry bank");
    assert_eq!(entry_geometry(&img, 1).1, 0x2002, "second entry offset");

    let (b, o, s) = entry_geometry(&img, 1);
    assert_eq!(img.read_program(b, o, s), second);
}

#[test]
fn ac_10_none_larger_programs_are_placed_before_smaller_ones() {
    // Placement is weight-ordered, largest first. Reproducing that ordering is what keeps the
    // output byte-identical; a different but equally valid order would still boot and still fail
    // parity.
    let small = ramp(0x100);
    let large = ramp(0x4000);
    let img = CartBuilder::new()
        .with_program(Program::new("small", small))
        .with_program(Program::new("large", large.clone()))
        .build_image()
        .unwrap();

    // Directory order follows input order; placement order does not.
    let (_, small_off, _) = entry_geometry(&img, 0);
    let (large_bank, large_off, large_size) = entry_geometry(&img, 1);
    assert_eq!(
        (large_bank, large_off),
        (1, 0x0000),
        "the larger program takes the top slot"
    );
    assert!(
        small_off > 0 || entry_geometry(&img, 0).0 > 1,
        "the smaller program follows it"
    );
    assert_eq!(img.read_program(large_bank, large_off, large_size), large);
}
