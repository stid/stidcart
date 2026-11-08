//! The reserved region of the image, and what goes where in it.
//!
//! Bank 0 ROMH is reserved end to end: directory, EAPI, cartridge name, boot code. Bank 0 ROML
//! holds the menu loader in normal mode; in Ocean mode that bank belongs to the Ocean cartridge
//! and the loader moves to bank 1 ROMH, with its own launcher in the boot region (D-024). Every
//! offset here was confirmed by dumping an image built by the reference tool.

use crate::firmware;
use crate::image::{FlashImage, Half, BANK_SIZE};
use crate::menu::{fingerprint, Menu, Recognised, EARLIER_STIDCART_BUILDS, STIDCART_MENU};
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
/// The menu loader in Ocean mode: bank 1 ROMH, seen at `$A000` (`loader.c:38`).
pub const LOADER_OCM_OFFSET: usize = FlashImage::offset(1, Half::RomH);
pub const LOADER_SIZE: usize = BANK_SIZE;

/// Which firmware an image carries, and where. A property of the entry set — Ocean if any entry
/// is an Ocean cartridge — never declared (D-024).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Normal,
    Ocean,
}

impl Mode {
    pub fn loader_offset(self) -> usize {
        match self {
            Mode::Normal => LOADER_NRM_OFFSET,
            Mode::Ocean => LOADER_OCM_OFFSET,
        }
    }

    /// The loader-slot bytes of `menu` in this mode, or `None` for our menu in Ocean mode, which
    /// has no build (D-037: `build_image` refuses the pairing before anything is written).
    fn loader(self, menu: Menu) -> Option<&'static [u8]> {
        match (self, menu) {
            (Mode::Normal, Menu::EasyLoader) => Some(firmware::body(firmware::LOADER_NRM)),
            (Mode::Ocean, Menu::EasyLoader) => Some(firmware::body(firmware::LOADER_OCM)),
            (Mode::Normal, Menu::Stidcart) => Some(STIDCART_MENU),
            (Mode::Ocean, Menu::Stidcart) => None,
        }
    }

    fn launcher(self) -> &'static [u8] {
        match self {
            Mode::Normal => firmware::LAUNCHER_NRM,
            Mode::Ocean => firmware::LAUNCHER_OCM,
        }
    }
}

/// What the reference tool stores when no cartridge name is given. The region is never absent.
pub const DEFAULT_EF_NAME: &str = "(no EF-Name)";

pub fn write_eapi(img: &mut FlashImage) {
    img.write_at(EAPI_OFFSET, firmware::body(firmware::EAPI));
}

pub fn write_loader(img: &mut FlashImage, mode: Mode, menu: Menu) {
    let loader = mode
        .loader(menu)
        .expect("build_image refuses our menu on an Ocean cart before writing anything");
    img.write_at(mode.loader_offset(), loader);
}

/// Which menu the loader slot holds, or `None` when it is neither. Never refuses: describing an
/// image reports this, and only `load` turns `None` into a refusal (D-011).
pub fn detect_menu(img: &FlashImage, mode: Mode) -> Option<Recognised> {
    let at = mode.loader_offset();
    let slot = &img.as_bytes()[at..at + LOADER_SIZE];
    let current = [Menu::EasyLoader, Menu::Stidcart]
        .into_iter()
        .find(|&m| mode.loader(m) == Some(slot))
        .map(|menu| Recognised {
            menu,
            older_build: false,
        });
    // Our menu has no Ocean build, so only a normal-mode slot can hold an earlier one (D-049).
    current.or_else(|| {
        (mode == Mode::Normal && EARLIER_STIDCART_BUILDS.contains(&fingerprint(slot))).then_some(
            Recognised {
                menu: Menu::Stidcart,
                older_build: true,
            },
        )
    })
}

/// Fill the boot region with erased flash, then overlay the launcher at `load_addr - $FB18`.
///
/// Writing only the launcher blob is the tempting mistake: it leaves roughly 1100 bytes of the
/// region holding whatever was there before.
pub fn write_boot(img: &mut FlashImage, mode: Mode) {
    let blob = mode.launcher();
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

/// Check that the image's reserved firmware is exactly what this tool embeds, and report which
/// menu the loader slot holds.
///
/// Rebuilding always writes the embedded blobs, so an image carrying different ones could be
/// loaded but never rebuilt to the same bytes. Refusing here makes that failure land where its
/// cause is legible instead of surfacing later as an unexplained diff. The loader slot may hold
/// either menu this crate embeds (D-038), or an earlier build of ours (D-049); anything else is a
/// mismatch.
pub fn verify_regions(img: &FlashImage, mode: Mode) -> Result<Menu, crate::read::ReadError> {
    let mut expected = FlashImage::erased();
    write_eapi(&mut expected);
    write_boot(&mut expected, mode);

    let mismatch = |region| {
        Err(crate::read::ReadError::Unsupported(
            crate::read::Unsupported::ReservedRegionMismatch { region },
        ))
    };

    let regions = [
        ("EAPI", EAPI_OFFSET, EAPI_SIZE),
        ("boot", BOOT_OFFSET, BOOT_SIZE),
    ];
    for (region, at, size) in regions {
        if img.as_bytes()[at..at + size] != expected.as_bytes()[at..at + size] {
            return mismatch(region);
        }
    }
    // An earlier build of our menu loads as ours, so the rebuild writes the current one (D-049).
    let Some(Recognised { menu, .. }) = detect_menu(img, mode) else {
        return mismatch("menu loader");
    };

    // The name field itself is caller data, but the magic in front of it is fixed, and its absence
    // means this is not a cartridge whose name we can read.
    if &img.as_bytes()[EFNAME_MAGIC_OFFSET..EFNAME_MAGIC_OFFSET + EFNAME_MAGIC.len()]
        != EFNAME_MAGIC
    {
        return mismatch("EF-Name magic");
    }

    Ok(menu)
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
        write_boot(&mut img, Mode::Normal);

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
        write_boot(&mut img, Mode::Normal);
        write_loader(&mut img, Mode::Normal, Menu::EasyLoader);
        write_ef_name(&mut img, "anything");
        assert!(verify_regions(&img, Mode::Normal).is_ok());
    }

    #[test]
    fn ocean_mode_puts_the_loader_in_bank_1_romh_and_leaves_bank_0_roml_alone() {
        let mut img = FlashImage::erased();
        write_eapi(&mut img);
        write_boot(&mut img, Mode::Ocean);
        write_loader(&mut img, Mode::Ocean, Menu::EasyLoader);
        write_ef_name(&mut img, "ocean");
        assert_eq!(
            &img.as_bytes()[LOADER_OCM_OFFSET..LOADER_OCM_OFFSET + LOADER_SIZE],
            firmware::body(firmware::LOADER_OCM)
        );
        assert!(img
            .bank(0, Half::RomL)
            .iter()
            .all(|&b| b == crate::image::ERASED));
        assert!(verify_regions(&img, Mode::Ocean).is_ok());
        assert!(
            verify_regions(&img, Mode::Normal).is_err(),
            "the normal-mode check expects the normal loader at bank 0 ROML"
        );
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
            write_boot(&mut img, Mode::Normal);
            write_loader(&mut img, Mode::Normal, Menu::EasyLoader);
            write_ef_name(&mut img, "anything");
            img.write_at(at, &[0x5a; 8]);
            assert!(
                verify_regions(&img, Mode::Normal).is_err(),
                "a change at {at:#x} should be caught"
            );
        }
    }

    #[test]
    fn the_name_field_itself_is_not_part_of_verification() {
        // It is caller data — only the magic in front of it is fixed.
        let mut img = FlashImage::erased();
        write_eapi(&mut img);
        write_boot(&mut img, Mode::Normal);
        write_loader(&mut img, Mode::Normal, Menu::EasyLoader);
        write_ef_name(&mut img, "whatever the user chose");
        assert!(verify_regions(&img, Mode::Normal).is_ok());
    }

    #[test]
    fn verification_reports_which_menu_the_loader_slot_holds() {
        for menu in [Menu::EasyLoader, Menu::Stidcart] {
            let mut img = FlashImage::erased();
            write_eapi(&mut img);
            write_boot(&mut img, Mode::Normal);
            write_loader(&mut img, Mode::Normal, menu);
            write_ef_name(&mut img, "either");
            assert_eq!(
                detect_menu(&img, Mode::Normal),
                Some(Recognised {
                    menu,
                    older_build: false
                })
            );
            assert_eq!(verify_regions(&img, Mode::Normal), Ok(menu));
        }
        let mut img = FlashImage::erased();
        write_loader(&mut img, Mode::Normal, Menu::Stidcart);
        img.write_at(0x100, &[0x5a; 8]);
        assert_eq!(
            detect_menu(&img, Mode::Normal),
            None,
            "a mutated menu is neither"
        );
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
