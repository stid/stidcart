//! AC-36 and AC-37 of the **cart-entries** work-item — Ocean as a mode of the whole image.
//!
//! Both are `surfaces: none`. Ocean is the one cartridge kind that must own bank 0, which is where
//! the normal-mode menu loader lives; the reference tool answers by moving the loader and swapping
//! the boot code (D-024), and the frozen `cocean.crt` / `cocean512.crt` pin the result.

use std::path::{Path, PathBuf};

use stidcart_core::cartridge::Cartridge;
use stidcart_core::image::{FlashImage, Half};
use stidcart_core::{crt, firmware, read, reserved, BuildError, CartBuilder, Program};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(fixtures_dir().join(name)).unwrap_or_else(|e| panic!("fixture {name}: {e}"))
}

fn cart(stem: &str) -> Cartridge {
    Cartridge::from_crt(&fixture(&format!("{stem}.crt")))
        .unwrap_or_else(|e| panic!("{stem}.crt: {e:?}"))
        .with_name(stem)
}

/// AC-36 (none): an Ocean entry puts the image in Ocean mode — the cart owns bank 0 ROML, the
/// Ocean-mode loader and boot code are written — and such an image loads back under that mode.
#[test]
fn ac_36_an_ocean_entry_puts_the_image_in_ocean_mode_and_loads_back_under_it() {
    let ocean = cart("in_ocean");
    let img = CartBuilder::new()
        .with_ef_name("cocean")
        .with_program(Program::new("big", fixture("big.prg")))
        .with_cartridge(ocean.clone())
        .build_image()
        .expect("the Ocean set builds");
    let d = img.as_bytes();

    assert_eq!(
        img.bank(0, Half::RomL),
        &ocean.roml[..0x2000],
        "bank 0 ROML belongs to the Ocean cartridge, not the loader"
    );
    assert_eq!(
        &d[reserved::LOADER_OCM_OFFSET..reserved::LOADER_OCM_OFFSET + reserved::LOADER_SIZE],
        firmware::body(firmware::LOADER_OCM),
        "the Ocean-mode loader sits at bank 1 ROMH"
    );
    assert_eq!(
        reserved::LOADER_OCM_OFFSET,
        FlashImage::offset(1, Half::RomH)
    );
    let launcher = firmware::LAUNCHER_OCM;
    let start = reserved::BOOT_OFFSET
        + (firmware::load_addr(launcher) as usize - reserved::BOOT_BASE_ADDR as usize);
    assert_eq!(
        &d[start..start + firmware::body(launcher).len()],
        firmware::body(launcher),
        "the Ocean-mode launcher is the boot code"
    );

    // The container still says "EasyFlash cart, Ultimax mode" — the mode is inside the image.
    let bytes = crt::write(&img, "probe");
    assert_eq!(&bytes[0x16..0x1a], &[0x00, 0x20, 0x01, 0x00]);

    // And the frozen Ocean images load: the firmware check is made against the Ocean-mode blobs.
    for name in ["cocean.crt", "cocean512.crt"] {
        let original = fixture(name);
        let loaded = read::load(&original).unwrap_or_else(|e| panic!("{name}: load: {e}"));
        assert!(loaded
            .entries()
            .iter()
            .any(|e| e.as_cartridge().is_some_and(|c| c.kind.is_ocean())));
        assert!(
            loaded.build().expect("rebuilds") == original,
            "{name}: load-and-save is byte-identical"
        );
    }
    // A normal-mode image still loads under the normal-mode check.
    assert!(read::load(&fixture("two.crt")).is_ok());
}

/// AC-37 (none): a second Ocean entry is refused by name — only one entry can own bank 0.
#[test]
fn ac_37_a_second_ocean_entry_is_refused_by_name() {
    let err = CartBuilder::new()
        .with_cartridge(cart("in_ocean").with_name("first"))
        .with_cartridge(cart("in_ocean").with_name("second"))
        .build()
        .expect_err("two Ocean carts cannot both own bank 0");
    assert_eq!(
        err,
        BuildError::SecondOcean {
            name: "second".into()
        }
    );
}
