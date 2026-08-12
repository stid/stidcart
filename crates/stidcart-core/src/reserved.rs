//! The reserved region of the image, and what goes where in it.
//!
//! Bank 0 ROMH is reserved end to end: directory, EAPI, cartridge name, boot code. Bank 0 ROML
//! holds the menu loader in normal mode. Every offset here was confirmed by dumping an image
//! built by the reference tool.

use crate::firmware;
use crate::image::{FlashImage, Half, BANK_SIZE};
use crate::petscii;

/// EasyFS2 directory: 256 slots of 24 bytes, filling exactly `0x1800`.
pub const DIRECTORY_OFFSET: usize = FlashImage::offset(0, Half::RomH);
pub const DIRECTORY_SLOTS: usize = 256;

pub const EAPI_OFFSET: usize = 0x81800;
pub const EAPI_SIZE: usize = 0x300;

pub const EFNAME_MAGIC_OFFSET: usize = 0x81b00;
pub const EFNAME_MAGIC: &[u8; 8] = b"ef-nAME:";
pub const EFNAME_OFFSET: usize = 0x81b08;
pub const EFNAME_LEN: usize = 16;

/// The boot region is based at `$FB18` and is mostly erased; the launcher is overlaid inside it
/// at its own load address.
pub const BOOT_OFFSET: usize = 0x81b18;
pub const BOOT_SIZE: usize = 0x4e8;
pub const BOOT_BASE_ADDR: u16 = 0xfb18;

/// The menu loader in normal mode: bank 0 ROML, seen by the machine at `$8000`.
pub const LOADER_NRM_OFFSET: usize = 0;
pub const LOADER_SIZE: usize = BANK_SIZE;

/// What the reference tool stores when no cartridge name is given. The region is never absent.
pub const DEFAULT_EF_NAME: &str = "(no EF-Name)";

pub fn write_eapi(img: &mut FlashImage) {
    img.write_at(EAPI_OFFSET, firmware::body(firmware::EAPI));
}

pub fn write_loader(img: &mut FlashImage) {
    img.write_at(LOADER_NRM_OFFSET, firmware::body(firmware::LOADER_NRM));
}

/// Fill the boot region with erased flash, then overlay the launcher at `load_addr - $FB18`.
///
/// Writing only the launcher blob is the tempting mistake: it leaves roughly 1100 bytes of the
/// region holding whatever was there before.
pub fn write_boot(img: &mut FlashImage) {
    let blob = firmware::LAUNCHER_NRM;
    let mut region = vec![crate::image::ERASED; BOOT_SIZE];
    let start = firmware::load_addr(blob) as usize - BOOT_BASE_ADDR as usize;
    let body = firmware::body(blob);
    region[start..start + body.len()].copy_from_slice(body);
    img.write_at(BOOT_OFFSET, &region);
}

/// Write the magic and the cartridge name, NUL-padded to the full field width.
pub fn write_ef_name(img: &mut FlashImage, name: &str) {
    img.write_at(EFNAME_MAGIC_OFFSET, EFNAME_MAGIC);
    let mut field = petscii::ascii_to_petscii(name);
    field.truncate(EFNAME_LEN);
    field.resize(EFNAME_LEN, 0);
    img.write_at(EFNAME_OFFSET, &field);
}

/// Check that the image's reserved firmware is exactly what this tool embeds.
///
/// Rebuilding always writes the embedded blobs, so an image carrying different ones could be
/// loaded but never rebuilt to the same bytes. Refusing here makes that failure land where its
/// cause is legible instead of surfacing later as an unexplained diff.
pub fn verify_regions(img: &FlashImage) -> Result<(), crate::read::ReadError> {
    let mut expected = FlashImage::erased();
    write_eapi(&mut expected);
    write_boot(&mut expected);
    write_loader(&mut expected);

    let mismatch = |region| {
        Err(crate::read::ReadError::Unsupported(
            crate::read::Unsupported::ReservedRegionMismatch { region },
        ))
    };

    let regions = [
        ("EAPI", EAPI_OFFSET, EAPI_SIZE),
        ("boot", BOOT_OFFSET, BOOT_SIZE),
        ("menu loader", LOADER_NRM_OFFSET, LOADER_SIZE),
    ];
    for (region, at, size) in regions {
        if img.as_bytes()[at..at + size] != expected.as_bytes()[at..at + size] {
            return mismatch(region);
        }
    }

    // The name field itself is caller data, but the magic in front of it is fixed, and its absence
    // means this is not a cartridge whose name we can read.
    if &img.as_bytes()[EFNAME_MAGIC_OFFSET..EFNAME_MAGIC_OFFSET + EFNAME_MAGIC.len()]
        != EFNAME_MAGIC
    {
        return mismatch("EF-Name magic");
    }

    Ok(())
}

/// The inverse of [`write_ef_name`]: the cartridge name the menu shows.
pub fn read_ef_name(img: &FlashImage) -> String {
    petscii::petscii_to_ascii(&img.as_bytes()[EFNAME_OFFSET..EFNAME_OFFSET + EFNAME_LEN])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::directory::ENTRY_LEN;

    #[test]
    fn the_reserved_regions_tile_bank_0_romh_exactly() {
        assert_eq!(DIRECTORY_OFFSET, 0x80000);
        assert_eq!(DIRECTORY_OFFSET + DIRECTORY_SLOTS * ENTRY_LEN, EAPI_OFFSET);
        assert_eq!(EAPI_OFFSET + EAPI_SIZE, EFNAME_MAGIC_OFFSET);
        assert_eq!(EFNAME_MAGIC_OFFSET + EFNAME_MAGIC.len(), EFNAME_OFFSET);
        assert_eq!(EFNAME_OFFSET + EFNAME_LEN, BOOT_OFFSET);
        assert_eq!(BOOT_OFFSET + BOOT_SIZE, FlashImage::offset(1, Half::RomH));
    }

    #[test]
    fn boot_region_keeps_its_erased_fill_around_the_launcher() {
        let mut img = FlashImage::erased();
        img.write_at(BOOT_OFFSET, &vec![0x00; BOOT_SIZE]);
        write_boot(&mut img);

        let start = firmware::load_addr(firmware::LAUNCHER_NRM) as usize - BOOT_BASE_ADDR as usize;
        let d = img.as_bytes();
        assert!(
            d[BOOT_OFFSET..BOOT_OFFSET + start]
                .iter()
                .all(|&b| b == crate::image::ERASED),
            "the fill below the launcher was not restored"
        );
        assert_eq!(
            &d[BOOT_OFFSET + start..BOOT_OFFSET + BOOT_SIZE],
            firmware::body(firmware::LAUNCHER_NRM)
        );
    }

    #[test]
    fn ef_name_is_padded_with_nuls_not_erased_flash() {
        let mut img = FlashImage::erased();
        write_ef_name(&mut img, "abc");
        let field = &img.as_bytes()[EFNAME_OFFSET..EFNAME_OFFSET + EFNAME_LEN];
        assert_eq!(&field[..3], b"ABC");
        assert!(field[3..].iter().all(|&b| b == 0));
    }

    #[test]
    fn read_ef_name_inverts_write_ef_name() {
        for name in ["custom name", "My Game v2", DEFAULT_EF_NAME] {
            let mut img = FlashImage::erased();
            write_ef_name(&mut img, name);
            assert_eq!(read_ef_name(&img), name, "round trip failed for {name:?}");
        }
    }

    #[test]
    fn a_fully_written_image_verifies() {
        let mut img = FlashImage::erased();
        write_eapi(&mut img);
        write_boot(&mut img);
        write_loader(&mut img);
        write_ef_name(&mut img, "anything");
        assert!(verify_regions(&img).is_ok());
    }

    #[test]
    fn a_mutated_reserved_region_does_not_verify() {
        for at in [
            EAPI_OFFSET,
            BOOT_OFFSET,
            LOADER_NRM_OFFSET,
            EFNAME_MAGIC_OFFSET,
        ] {
            let mut img = FlashImage::erased();
            write_eapi(&mut img);
            write_boot(&mut img);
            write_loader(&mut img);
            write_ef_name(&mut img, "anything");
            img.write_at(at, &[0x5a; 8]);
            assert!(
                verify_regions(&img).is_err(),
                "a change at {at:#x} should be caught"
            );
        }
    }

    #[test]
    fn the_name_field_itself_is_not_part_of_verification() {
        // It is caller data — only the magic in front of it is fixed.
        let mut img = FlashImage::erased();
        write_eapi(&mut img);
        write_boot(&mut img);
        write_loader(&mut img);
        write_ef_name(&mut img, "whatever the user chose");
        assert!(verify_regions(&img).is_ok());
    }

    #[test]
    fn an_overlong_ef_name_is_truncated_to_the_field() {
        let mut img = FlashImage::erased();
        write_ef_name(&mut img, "0123456789abcdefghij");
        let field = &img.as_bytes()[EFNAME_OFFSET..EFNAME_OFFSET + EFNAME_LEN];
        assert_eq!(field.len(), EFNAME_LEN);
        assert_eq!(field, b"0123456789ABCDEF");
    }
}
