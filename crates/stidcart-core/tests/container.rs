//! AC-2, container half — the `.crt` wrapper around the flash image.
//!
//! Localizes a parity failure: if these pass and `parity.rs` fails, the wrapper is right and the
//! image inside it is wrong.

use stidcart_core::crt;
use stidcart_core::image::BANK_SIZE;
use stidcart_core::read::ReadError;
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

/// AC-2 (CLI): container header fields, the wrapper half of byte identity.
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

/// AC-2 (CLI): the header name defaults to this tool, not the reference (D-006).
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

/// AC-2 (CLI): CHIP packets cover written banks only.
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

/// AC-2 (CLI): packet ordering is bank, then ROML before ROMH.
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

/// Pad the header so the first packet starts at `header_len`, and say so in the field (D-021).
fn with_header_len(cart: &[u8], header_len: usize) -> Vec<u8> {
    let mut out = cart[..crt::HEADER_LEN].to_vec();
    out[0x10..0x14].copy_from_slice(&(header_len as u32).to_be_bytes());
    out.resize(header_len, 0);
    out.extend_from_slice(&cart[crt::HEADER_LEN..]);
    out
}

/// The header length field is the offset of the first packet, not a constant to assume (D-021).
#[test]
fn a_longer_header_is_walked_from_where_the_field_says_the_packets_start() {
    let cart = a_cart();
    let padded = with_header_len(&cart, 0x48);
    assert_ne!(
        cart, padded,
        "the padded file must differ, or the test proves nothing"
    );

    let want = crt::parse(&cart).expect("the reference layout parses");
    let got = crt::parse(&padded).expect("a 0x48-byte header is legal per the container spec");
    assert_eq!(got.image.as_bytes(), want.image.as_bytes());
    assert_eq!(got.header_name, want.header_name);
}

/// The fields the reader has already used end at 0x40, so a smaller header is malformed (D-021).
#[test]
fn a_header_length_below_the_fixed_fields_is_refused() {
    let mut cart = a_cart();
    cart[0x10..0x14].copy_from_slice(&0x20u32.to_be_bytes());
    assert_eq!(
        crt::parse(&cart).err(),
        Some(ReadError::BadHeaderLength { stated: 0x20 })
    );
}

/// A header length past the end of the file is a truncation like any other (D-021).
#[test]
fn a_header_length_past_the_end_of_the_file_is_a_truncation() {
    let mut cart = a_cart();
    let beyond = cart.len() + 1;
    cart[0x10..0x14].copy_from_slice(&(beyond as u32).to_be_bytes());
    assert_eq!(
        crt::parse(&cart).err(),
        Some(ReadError::Truncated {
            need: beyond,
            have: cart.len()
        })
    );
}
