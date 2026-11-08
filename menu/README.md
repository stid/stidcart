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
| CRSR ↓ / CRSR ↑ | move the selection. The list scrolls past 16 rows          |
| RETURN          | start the selected entry                                   |
| letters, digits | filter: list only the entries whose name contains the text |
| DEL             | remove the last character of the filter text               |
| CLR/HOME        | clear the filter text                                      |
| F1              | switch between the cart's own order and A-Z                |
| RUN/STOP        | leave to BASIC with the cartridge switched off             |

The screen is black (D-041) and wears the stidcart brand (D-048): the cartridge mark, its label in
five colour bands, sits left of a white block-letter logo, and a strip of the same five bands runs
under the cart name. The cart name sits above 16 list rows. Each
entry shows a badge for its kind, in the kind's colour: "PRG" for a program, "8K" and "16K" for
those cartridges, "ULT" for Ultimax, and "XB" for multi-bank. The selected entry sits in an orange
bar. When the list is longer than the screen, the divider to its right becomes a scroll bar: the
thumb's length is the share of the list in view, and its place is where the view is. The pane on the
right describes the selection: its type, size, load address (program) or first bank (cartridge),
and how it starts. Below the list are the position ("17 of 40") and the order in use, then the
filter text in yellow with the number of matches out of the visible entries, then a grey bar naming the
keys. Hidden entries are not listed. The filter and A-Z both ignore case. The filter takes up to 16
characters, any printable one except `"`. Every boot starts in
the cart's own order, which is the order they were built in (D-039). A program that loads at or below `$0801` is started with `RUN`,
and any other with `SYS` to its load address. It finds the C64's own colours and upper-case
character set, as after a reset. The tool refuses to put this menu on a cart with an
Ocean entry, or with a program that would load into `$D000-$DFFF` (D-038).

## How it starts

The boot code in bank 0 ROMH hands over through the first word of the loader slot, `jmp ($8000)`,
in 16K mode at bank 0, with the KERNAL not initialised. The full contract is in
`../docs/format/easyflash-cart-format.md`, under "How the boot code hands over to the menu loader".

- `src/menu.s` runs from ROM. It copies `src/launch.s` to the cartridge's I/O2 RAM at `$DF02` and
  initialises the KERNAL from there with the cartridge off, so the top of memory comes out right.
  It copies the ROM's mixed-case character set to `$3800` and adds the menu's own glyphs to it in
  codes `$60-$7F`: the badges (with their inverses at `$E0` and up, for the selection bar), the rule,
  the scroll bar and the logo's quarter-block cells. The cartridge mark's nine multicolour cells go
  in `$EA-$F2`, reversed codes that names never reach. It then scans the EasyFS2 directory at `$A000`
  and draws straight into screen and colour RAM, writing each cell once per update. A cursor move
  that doesn't scroll redraws only the two rows it changes, and drawing starts below the visible
  screen, so nothing flickers. CHROUT isn't used: a full 40-column row links into the next line,
  and the last cell of row 24 scrolls the screen.
- A raster interrupt draws the brand once a frame (D-048). From line 46 it turns multicolour on for
  the logo rows, where only the mark's cells have colours 8-15, and rewrites `$D022` every three
  lines so the mark's label shows the five bands. It turns multicolour off after row 2, then sets
  the background for one line each on 84-88, the strip. It replaces the CIA timer's interrupt and
  runs the KERNAL's own work (`$EA31`) itself, so the keyboard is scanned 50 times a second and
  nothing can hold the bars up. Entered too late in a frame, it skips that frame's bars.
- The interrupt handler runs from the cartridge's ROM. So interrupts are masked whenever the menu
  maps another bank in, and every way out (a program, a cartridge, RUN/STOP) first puts back the
  KERNAL's interrupt: raster interrupt off, `$0314` at `$EA31`, the CIA timer on. The boot tests
  watch every write to `$DE00` and `$DE02` for a live interrupt (AC-127), and read VICE
  screenshots for what screen RAM can't show (AC-124..AC-126).
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
