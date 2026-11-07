# The stidcart menu

Our own C64-side menu (D-037). It lists a cart's entries and starts the one picked: a program, or a
cartridge of any kind except Ocean. It is opt-in. `build --menu stidcart` puts it into a cart, and
`edit --menu stidcart|easyloader` switches an existing one. Without the option, carts keep
EasyLoader, the zlib menu in `../firmware/`.

GPL-2.0-or-later, like everything outside `firmware/` (see `../COPYING`). It is written fresh: the
start sequences follow EasyLoader's technique, cited where they are used, and contain none of its
code.

## Using it

| Key             | Does                                                       |
| --------------- | ---------------------------------------------------------- |
| CRSR ↓ / CRSR ↑ | move the selection. The list scrolls past 20 rows          |
| RETURN          | start the selected entry                                   |
| letters, digits | filter: list only the entries whose name contains the text |
| DEL             | remove the last character of the filter text               |
| CLR/HOME        | clear the filter text                                      |
| F1              | switch between the cart's own order and A-Z                |
| RUN/STOP        | leave to BASIC with the cartridge switched off             |

Hidden entries are not listed. The filter and A-Z both ignore case. The filter takes up to 16
characters, any printable one except `"`, and shows them on the row above the list. The bottom row
shows how many entries match out of those listed, and which order is in use. Every boot starts in
the cart's own order, which is the order they were built in (D-039). A program that loads at or below `$0801` is started with `RUN`,
and any other with `SYS` to its load address. The tool refuses to put this menu on a cart with an
Ocean entry, or with a program that would load into `$D000-$DFFF` (D-038).

## How it starts

The boot code in bank 0 ROMH hands over through the first word of the loader slot, `jmp ($8000)`,
in 16K mode at bank 0, with the KERNAL not initialised. The full contract is in
`../docs/format/easyflash-cart-format.md`, under "How the boot code hands over to the menu loader".

- `src/menu.s` runs from ROM. It copies `src/launch.s` to the cartridge's I/O2 RAM at `$DF02` and
  initialises the KERNAL from there with the cartridge off, so the top of memory comes out right.
  It then scans the EasyFS2 directory at `$A000` and draws the list through CHROUT.
- Keys come only from GETIN, the KERNAL keyboard buffer, because that is the only place VICE can
  inject keys for the boot tests. `menu_idle` is the point where the menu waits with an empty
  buffer, and the tests break there.
- `src/launch.s` holds what has to run while the cartridge or its bank is switched away:
  - leaving
  - starting a cartridge: bank and mode, the entry's first bank left in `$DF00/$DF01` for xbank
    carts, then reset
  - starting a program: a BASIC cold start with CHRIN trapped. The trap copies the program bank by
    bank and returns to the menu ROM, which sets BASIC's pointers and types `RUN` or `SYS`.

  `$DF00-$DFFF` belongs to the menu once it is entered.

## Rebuilding

```sh
menu/build.sh menu/
```

This assembles `src/menu.s` with 64tass (1.60.3243; the flags are in the script) into `menu.bin`,
the raw 8 KiB slot, and `menu.lbl`, the VICE labels. Commit both with any source change. They are
committed so building the Rust crates never needs the assembler. `stidcart-core`'s AC-93 test
reassembles the source and fails if either file differs, or if the menu outgrows the slot. In CI,
64tass is required, so that check always runs.

The evidence that the menu works is VICE, because the reference tool knows nothing of it:
`crates/stidcart-cli/tests/menu_boot.rs` boots carts and reads the screen back (AC-87..AC-92).
