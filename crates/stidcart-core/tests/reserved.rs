//! AC-6 and AC-8 — the bank-0 reserved region and how names are stored in it.
//!
//! Every offset asserted here was read out of an image built by the reference tool, not derived
//! by arithmetic. Deriving it is how a test in ndefpack filled the wrong half of the image for
//! its entire life.

use stidcart_core::directory::{ENTRY_LEN, RESERVED_FLAG_BITS};
use stidcart_core::firmware;
use stidcart_core::reserved::*;
use stidcart_core::{CartBuilder, Program};

fn one_program_cart(ef_name: &str) -> stidcart_core::FlashImage {
    CartBuilder::new()
        .with_ef_name(ef_name)
        .with_program(Program::new("custom name", vec![0x01, 0x08, 0x42, 0x43]))
        .build_image()
        .expect("a one-program cart fits")
}

#[test]
fn ac_6_none_reserved_region_layout() {
    let img = one_program_cart("weird name");
    let d = img.as_bytes();

    // The menu loader occupies bank 0 ROML in normal mode, and the blob is stored with its
    // two-byte load address stripped.
    let loader = firmware::LOADER_NRM;
    assert_eq!(
        &d[LOADER_NRM_OFFSET..LOADER_NRM_OFFSET + LOADER_SIZE],
        &loader[2..],
        "menu loader is not at bank 0 ROML"
    );

    // The flash-writing interface sits immediately above the directory.
    let eapi = firmware::EAPI;
    assert_eq!(
        &d[EAPI_OFFSET..EAPI_OFFSET + EAPI_SIZE],
        &eapi[2..],
        "EAPI is not at 0x81800"
    );
    assert_eq!(&d[EAPI_OFFSET..EAPI_OFFSET + 4], b"eapi", "EAPI signature");

    // The cartridge name lives behind a magic string, which is literally mixed-case.
    assert_eq!(
        &d[EFNAME_MAGIC_OFFSET..EFNAME_MAGIC_OFFSET + 8],
        EFNAME_MAGIC
    );

    // The boot region is NOT just the launcher blob: it is 0x4E8 bytes of erased flash with the
    // launcher overlaid at its own load address. Writing only the blob leaves ~1100 bytes wrong.
    let launcher = firmware::LAUNCHER_NRM;
    let load_addr = u16::from_le_bytes([launcher[0], launcher[1]]) as usize;
    let overlay_at = BOOT_OFFSET + (load_addr - BOOT_BASE_ADDR as usize);
    assert_eq!(
        &d[BOOT_OFFSET..overlay_at],
        vec![0xff; overlay_at - BOOT_OFFSET].as_slice(),
        "boot region below the launcher must be left erased"
    );
    assert_eq!(
        &d[overlay_at..BOOT_OFFSET + BOOT_SIZE],
        &launcher[2..],
        "launcher is not overlaid at its own load address"
    );

    // The reset vector at $FFFC points at the launcher's start address. This is the assertion
    // that would catch an off-by-one-bank error in the whole map above.
    let reset = u16::from_le_bytes([d[0x81ffc], d[0x81ffd]]) as usize;
    assert_eq!(
        reset, load_addr,
        "reset vector does not point at the launcher"
    );
}

#[test]
fn ac_6_none_reserved_region_is_contiguous_and_fills_the_bank() {
    // directory | EAPI | name magic | name | boot — end to end, no holes and no overlap.
    assert_eq!(DIRECTORY_OFFSET + DIRECTORY_SLOTS * ENTRY_LEN, EAPI_OFFSET);
    assert_eq!(EAPI_OFFSET + EAPI_SIZE, EFNAME_MAGIC_OFFSET);
    assert_eq!(EFNAME_MAGIC_OFFSET + EFNAME_MAGIC.len(), EFNAME_OFFSET);
    assert_eq!(EFNAME_OFFSET + EFNAME_LEN, BOOT_OFFSET);
    assert_eq!(BOOT_OFFSET + BOOT_SIZE, 0x82000);
}

#[test]
fn ac_8_none_entry_name_encoding_and_reserved_flag_bits() {
    let img = one_program_cart("weird name");
    let d = img.as_bytes();

    // The name is stored in the machine's own encoding, which for an all-lowercase input means
    // uppercase bytes, and is NUL-padded to the full 16-byte field.
    let slot = &d[DIRECTORY_OFFSET..DIRECTORY_OFFSET + ENTRY_LEN];
    let mut expected = b"CUSTOM NAME".to_vec();
    expected.resize(16, 0);
    assert_eq!(&slot[0..16], expected.as_slice(), "entry name field");

    // Bits 5-6 of the flags byte are reserved and must be set. A past ndefpack bug wrote them
    // as zero, so this is format-relevant rather than cosmetic.
    assert_eq!(
        slot[16] & RESERVED_FLAG_BITS,
        RESERVED_FLAG_BITS,
        "reserved flag bits 0x60 are not set"
    );
    assert_eq!(slot[16] & 0x80, 0, "entry should not be marked hidden");

    // Same encoding rules for the cartridge name, also NUL-padded to its field width.
    let mut want = b"WEIRD NAME".to_vec();
    want.resize(EFNAME_LEN, 0);
    assert_eq!(
        &d[EFNAME_OFFSET..EFNAME_OFFSET + EFNAME_LEN],
        want.as_slice()
    );
}

#[test]
fn ac_8_none_omitted_cartridge_name_still_stores_a_placeholder() {
    // Measured: the reference tool always writes the name region, using a fixed placeholder when
    // none is given. A build that skips it will not match byte-for-byte.
    let img = CartBuilder::new()
        .with_program(Program::new("x", vec![0x01, 0x08, 0x00]))
        .build_image()
        .unwrap();
    let d = img.as_bytes();

    assert_eq!(
        &d[EFNAME_MAGIC_OFFSET..EFNAME_MAGIC_OFFSET + 8],
        EFNAME_MAGIC
    );
    let mut want = stidcart_core::petscii::ascii_to_petscii(DEFAULT_EF_NAME);
    want.resize(EFNAME_LEN, 0);
    assert_eq!(
        &d[EFNAME_OFFSET..EFNAME_OFFSET + EFNAME_LEN],
        want.as_slice()
    );
}
