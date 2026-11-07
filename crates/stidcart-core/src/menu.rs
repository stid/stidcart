//! Which C64-side menu a cart boots into (D-037, D-038).
//!
//! EasyLoader is the zlib blob in `firmware/` and stays the default. Our own menu is GPL, built
//! from `menu/src/` by `menu/build.sh`, and committed assembled, so building this crate never
//! needs the assembler. Both occupy the same 8 KiB loader slot.

/// Our menu's loader-slot image, exactly the slot's size.
pub const STIDCART_MENU: &[u8] = include_bytes!("../../../menu/menu.bin");

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
