# Credits

stidcart stands on other people's work. This lists who made what, and under which license.

## Shipped with stidcart

- **EasyLoader**, the menu EasyFlash carts have booted since 2009, by ALeX Kazik (2009-2012),
  in the version Hannu Nuotio altered for ndefpack (2011-2012). zlib license. In `firmware/`,
  with its source.
- **EAPI**, the flash-write API, and the boot code's key test, by Thomas 'skoe' Giesel
  (2009-2010). zlib license. In `firmware/`, with its source.
- **[Geist and Geist Mono](https://vercel.com/font)**, the desktop app's fonts, by The Geist
  Project Authors (2024). SIL Open Font License 1.1. In `gui/src/fonts/`.
- **Rust crates and npm packages** the CLI and the desktop app are built from, by their authors.
  Each one's license is in `Cargo.lock`, `gui/src-tauri/Cargo.lock` and `gui/package-lock.json`;
  all of them are permissive (MIT, Apache-2.0, BSD, ISC, Zlib, MPL-2.0, Unicode).

## Built on

- **[ndefpack](https://ndefpack.sourceforge.net/)** by Hannu Nuotio (2011-2012),
  GPL-2.0-or-later. stidcart's knowledge of the EasyFlash cart format was extracted from
  ndefpack's sources into
  `docs/format/easyflash-cart-format.md`, and the reference carts in `fixtures/` were built with
  its `ndeft`.
- **[EasyFlash](http://skoe.de/easyflash/)**, the cartridge and its file system, by Thomas
  'skoe' Giesel.
- **EasyLoader's start sequences**: our own menu (`menu/`) starts programs and cartridges the way
  EasyLoader does. It contains none of EasyLoader's code, and its source cites the technique where
  it is used.
- **[Catppuccin](https://catppuccin.com/)**, the colour palette the desktop app's light and dark
  themes use (MIT).

## Used to test, not shipped

- **[VICE](https://vice-emu.sourceforge.io/)**, whose `x64sc` runs the menu's boot tests.
- **[64tass](https://tass64.sourceforge.net/)**, which assembles our menu.
