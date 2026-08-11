//! AC-2, container half — the `.crt` wrapper around the flash image.
//!
//! Localizes a parity failure: if these pass and `parity.rs` fails, the wrapper is right and the
//! image inside it is wrong.

use stidcart_core::crt;
use stidcart_core::image::BANK_SIZE;
use stidcart_core::{CartBuilder, Program};

fn a_cart() -> Vec<u8> {
    CartBuilder::new()
        .with_header_name("ndefpack 1.0.1")
        .with_program(Program::new("entry", vec![0x01, 0x08, 0x11, 0x22]))
        .build()
        .unwrap()
}

/// Walk the CHIP packets, returning (bank, load address, data length) for each.
fn chips(bytes: &[u8]) -> Vec<(u16, u16, u16)> {
    let mut out = Vec::new();
    let mut p = crt::HEADER_LEN;
    while p + 16 <= bytes.len() && &bytes[p..p + 4] == b"CHIP" {
        let total = u32::from_be_bytes(bytes[p + 4..p + 8].try_into().unwrap()) as usize;
        out.push((
            u16::from_be_bytes([bytes[p + 10], bytes[p + 11]]),
            u16::from_be_bytes([bytes[p + 12], bytes[p + 13]]),
            u16::from_be_bytes([bytes[p + 14], bytes[p + 15]]),
        ));
        // The length field is the TOTAL packet length, header included. Advancing by
        // `16 + data_len` happens to work; advancing by `8 + total` does not, and that mistake
        // finds one chip in a five-chip file.
        p += total;
    }
    out
}

#[test]
fn ac_2_cli_container_header_fields() {
    let c = a_cart();

    assert_eq!(&c[0..16], b"C64 CARTRIDGE   ", "magic");
    assert_eq!(
        u32::from_be_bytes(c[0x10..0x14].try_into().unwrap()),
        crt::HEADER_LEN as u32,
        "header length"
    );
    assert_eq!(&c[0x14..0x16], &[0x01, 0x00], "version 1.0");
    assert_eq!(
        u16::from_be_bytes([c[0x16], c[0x17]]),
        crt::CARTRIDGE_EASYFLASH,
        "CRT ID"
    );
    // EasyFlash images are written in Ultimax mode: EXROM high, GAME low.
    assert_eq!((c[0x18], c[0x19]), (1, 0), "EXROM/GAME");

    let mut want = b"ndefpack 1.0.1".to_vec();
    want.resize(32, 0);
    assert_eq!(&c[0x20..0x40], want.as_slice(), "header name field");
}

#[test]
fn ac_2_cli_header_name_defaults_to_this_tool_not_the_reference() {
    // The two names are different fields at different offsets and are easy to conflate; the
    // cartridge name never appears in the container header.
    let c = CartBuilder::new()
        .with_ef_name("some cart")
        .with_program(Program::new("entry", vec![0x01, 0x08]))
        .build()
        .unwrap();
    let name = String::from_utf8_lossy(&c[0x20..0x40])
        .trim_end_matches('\0')
        .to_string();
    assert!(
        name.starts_with("stidcart "),
        "unexpected header name {name:?}"
    );
}

#[test]
fn ac_2_cli_chip_packets_cover_the_written_banks_only() {
    let c = a_cart();
    let chips = chips(&c);

    assert!(!chips.is_empty(), "no CHIP packets found");
    for (_, addr, len) in &chips {
        assert_eq!(*len as usize, BANK_SIZE, "every packet carries one 8K bank");
        assert!(
            *addr == 0x8000 || *addr == 0xa000,
            "unexpected load address {addr:#x}"
        );
    }

    // Bank 0 is fully occupied by the loader and the reserved region, so both halves are present.
    assert!(
        chips.contains(&(0, 0x8000, BANK_SIZE as u16)),
        "bank 0 ROML"
    );
    assert!(
        chips.contains(&(0, 0xa000, BANK_SIZE as u16)),
        "bank 0 ROMH"
    );

    // Erased banks are skipped rather than written out as blank, so a nearly-empty cart is small.
    assert!(
        chips.len() < 8,
        "a one-tiny-program cart wrote {} packets; erased banks are not being skipped",
        chips.len()
    );
    assert!(
        c.len() < 128 * 1024,
        "container is {} bytes; erased banks are not being skipped",
        c.len()
    );
}

#[test]
fn ac_2_cli_packets_are_ordered_by_bank_then_half() {
    let c = CartBuilder::new()
        .with_program(Program::new("big", vec![0x42; 0x6000]))
        .build()
        .unwrap();
    let order: Vec<(u16, u16)> = chips(&c).into_iter().map(|(b, a, _)| (b, a)).collect();
    let mut sorted = order.clone();
    sorted.sort();
    assert_eq!(
        order, sorted,
        "packets must be in bank, then ROML-before-ROMH order"
    );
}
