# Firmware: EasyLoader and EAPI (zlib licensed)

Everything in this directory is under the **zlib license** (`COPYING` here), not the GPL that
covers the rest of the repository. It is the C64-side code that every EasyFlash cart built by
stidcart carries, written by other people:

- **EasyFlash and EAPI**: (c) 2009-2010 Thomas 'skoe' Giesel
- **EasyLoader**: (c) 2009-2012 ALeX Kazik
- **Modifications to EasyLoader and its boot code**: (c) 2011-2012 Hannu Nuotio, for ndefpack

This is not the original EasyLoader. It is the altered version that ships with ndefpack 1.0.1.
Hannu Nuotio translated it from KickAssembler to ACME, filled unused space with `$FF` instead of
`$00`, removed some redundant instructions, and added a short delay at cart start so a held
joystick fire button is released first. The boot code merges EasyLoader's launcher with the
key test from skoe's banking-test example. Each source file's header lists its own changes.

## What is here

| File                          | Size | Role                                         | Lands at                         |
| ----------------------------- | ---- | -------------------------------------------- | -------------------------------- |
| `eapi_am29f040.prg`           | 770  | EAPI, the flash-write API games call to save | `0x81800`, 0x300 bytes           |
| `easyloader_nrm.prg`          | 8194 | EasyLoader menu, normal mode                 | `0x00000` (bank 0 ROML, `$8000`) |
| `easyloader_ocm.prg`          | 8194 | EasyLoader menu, Ocean mode                  | `0x82000` (bank 1 ROMH, `$A000`) |
| `easyloader_launcher_nrm.prg` | 88   | boot code, normal mode                       | `0x81b18` region, at `$FFAA`     |
| `easyloader_launcher_ocm.prg` | 88   | boot code, Ocean mode                        | `0x81b18` region                 |
| `src/`                        |      | the source these were assembled from         |                                  |

Each `.prg` is two bytes of little-endian load address followed by the data. They are
byte-identical to the ones in ndefpack 1.0.1, and stidcart embeds them as data: nothing here is
compiled when you build stidcart.

EAPI and the boot code go into every cart, whichever menu it boots. The EasyLoader menu goes in
only when it is chosen with `--menu easyloader` or the app's menu picker; otherwise the loader
slot holds our own menu from `../menu/`, which is GPL and contains none of EasyLoader's code.

**The boot region is not just the launcher.** It is 0x4E8 bytes based at `$FB18`, filled with
`0xFF` and then overlaid with the launcher at `start_addr - $FB18`. For the normal launcher that
start address is `$FFAA`, so only the last 86 bytes are not `0xFF`, and they end in the 6502
vectors: RESET at `$FFFC` points to `$FFAA`. Reproducing the `0xFF` fill is required for byte
parity.

## The source

`src/` holds the source files from ndefpack 1.0.1's `src/fw/`, copied unchanged from the tagged
release `ndefpack-1.0.1` in [ndefpack's repository on SourceForge](https://sourceforge.net/p/ndefpack/code/).
The original EasyLoader was published at `bitbucket.org/skoe/easyflash`, as the file headers say;
that repository no longer exists.

- `*.asm` assemble with [ACME](https://sourceforge.net/projects/acme-crossass/). The `_nrm` and
  `_ocm` files are the top-level ones; they set the mode and include `easyloader.asm` or
  `easyloader_launcher.asm`. `eapi_am29f040.asm` includes `eapi_defs.asm`.
- `easyloader.asm` also includes generated files: `easyloader_font.bin` and
  `easyloader_sprites.bin`, made from the two PNGs by `generate_font.php` and
  `generate_sprites.php`, and `easyloader_screen.bin`, `easyloader_cols.bin` and
  `easyloader_screendefs.asm`, made by `generate_screen.php`. `generate_screen.php` relies on a
  `preg_replace` feature PHP removed in 7.0, so it needs an old PHP or a small port.

ndefpack's build glue (its C wrapper, Makefile and header generator) is left out, because
stidcart reads the `.prg` files directly.

## The boundary

Don't move code across it. The rest of this repository is GPL-2.0-or-later (`../COPYING`). These
zlib files can sit inside a GPL project as they are, but GPL code copied in here would stop them
being zlib, and zlib code copied out should keep its notice. Our own menu replaces the EasyLoader
menu by default; EAPI and the boot code are still these.
