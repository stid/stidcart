//! Which C64-side menu a cart boots into (D-037, D-038).
//!
//! EasyLoader is the zlib blob in `firmware/` and stays the default. Our own menu is GPL, built
//! from `menu/src/` by `menu/build.sh`, and committed assembled, so building this crate never
//! needs the assembler. Both occupy the same 8 KiB loader slot.
//!
//! A cart built before the last change to `menu/menu.bin` carries an earlier build of our menu. It
//! is recognised by its fingerprint and loads as [`Menu::Stidcart`], so saving it writes the
//! current build (D-049).

/// Our menu's loader-slot image, exactly the slot's size.
pub const STIDCART_MENU: &[u8] = include_bytes!("../../../menu/menu.bin");

/// The FNV-1a 64 fingerprints of every earlier build of `menu/menu.bin`, oldest first, by the
/// commit that last changed it. When the menu changes, the replaced build's fingerprint joins this
/// list (the `menu_builds` test pins the current one), so no cart built with it is stranded.
pub const EARLIER_STIDCART_BUILDS: &[u64] = &[
    0x85f5_5429_73af_46a6, // 0d3dfba
    0x7537_fc2c_cf55_210c, // 2fbc818
    0x18ca_90a5_e0c1_5d17, // ee3ec0f
    0xd28a_8688_0a1c_c6de, // 6b80bb6
    0xe517_0a6f_5851_a075, // 2a1aaa2
    0x37bc_d298_704d_d07f, // 280f474
    0x9548_f328_dfac_82fb, // 5b54e81, stidcart 0.15
    0xeac9_353f_0e59_ab86, // e8eb154
    0xa4cb_0aba_6e81_53e0, // 63ba913
    0xebee_59a8_625e_4c8c, // 9f4ca36
    0x4088_1607_7fb5_9726, // 5bbe7bc
    0xbe42_a63d_5738_a710, // 3aa1b39
    0x7716_0576_a849_b6c4, // 4e8f451
    0x866f_3dcc_8415_53c1, // 7a9a0c1
    0xb8b7_0e40_1467_ea23, // 8b1f570
];

/// One past the last byte of the routine our menu runs at `$0100` to copy a program's bytes from
/// `$D000` on (D-056): a program reaching `$D000` that loads below it would overwrite the routine.
/// It is the label `high_copy_end` in `menu/menu.lbl`, and the `menu_builds` test holds them equal.
pub const HIGH_COPY_END: u16 = 0x0150;

/// FNV-1a 64 over `bytes`: how an earlier build of our menu is recognised in a loader slot.
pub fn fingerprint(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, &b| {
        (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
    })
}

/// A loader slot this build recognises: the menu a rebuild writes there, and whether the slot
/// holds an earlier build of it rather than the one embedded now.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Recognised {
    pub menu: Menu,
    pub older_build: bool,
}

impl Recognised {
    /// The name a listing shows.
    pub fn name(self) -> &'static str {
        if self.older_build {
            "stidcart (older build)"
        } else {
            self.menu.name()
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Menu {
    #[default]
    EasyLoader,
    Stidcart,
}

impl Menu {
    /// The name a listing shows.
    pub fn name(self) -> &'static str {
        match self {
            Menu::EasyLoader => "EasyLoader",
            Menu::Stidcart => "stidcart",
        }
    }

    /// The menu a `--menu` value names, case-insensitively.
    pub fn from_name(name: &str) -> Option<Menu> {
        [Menu::EasyLoader, Menu::Stidcart]
            .into_iter()
            .find(|m| m.name().eq_ignore_ascii_case(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_menu_fills_the_loader_slot() {
        assert_eq!(STIDCART_MENU.len(), crate::reserved::LOADER_SIZE);
    }

    #[test]
    fn names_round_trip_case_insensitively() {
        for m in [Menu::EasyLoader, Menu::Stidcart] {
            assert_eq!(Menu::from_name(m.name()), Some(m));
            assert_eq!(Menu::from_name(&m.name().to_uppercase()), Some(m));
        }
        assert_eq!(Menu::from_name("nonsense"), None);
    }
}
