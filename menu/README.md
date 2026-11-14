# The stidcart menu

Our own C64-side menu. It lists a cart's entries and starts the one picked: a program, or a
cartridge of any kind except Ocean. A new cart gets it by default; `build --menu easyloader` gives
a cart EasyLoader, the zlib menu in `../firmware/`, and `edit --menu stidcart|easyloader` switches
an existing cart, replacing whatever loader it carries.

Using the menu (its keys, the joystick, folders, what it can't start) is in
[docs/guide/menu.md](../docs/guide/menu.md). This page is about how it works and how to rebuild it.

GPL-2.0-or-later, like everything outside `firmware/` (see `../COPYING`). It is written fresh: the
start sequences follow EasyLoader's technique, cited where they are used, and contain none of its
code.

## How it starts

The boot code in bank 0 ROMH hands over through the first word of the loader slot, `jmp ($8000)`,
in 16K mode at bank 0, with the KERNAL not initialised. The full contract is in
`../docs/format/easyflash-cart-format.md`, under "How the boot code hands over to the menu loader".

- `src/menu.s` runs from ROM. It copies `src/launch.s` to the cartridge's I/O2 RAM at `$DF02` and
  initialises the KERNAL from there with the cartridge off, so the top of memory comes out right.
  It copies the ROM's mixed-case character set to `$3800`, replaces its lower-case letters (codes
  1-26, and their reverses) with small capitals (D-070), and adds the menu's own glyphs to it in
  codes `$60-$7F`: the badges and the folder badge (with their inverses at `$E0` and up, for the
  selection bar), the rule and the scroll bar. The wordmark's eighteen cells go in `$EC-$FD`,
  reversed codes that names never reach, stored with the ink as clear bits. It then
  scans the EasyFS2 directory at `$A000`, reads the metadata entry (below), turns the raster
  interrupt on, and draws straight into screen and colour RAM, writing each cell once per update. A
  cursor move that doesn't scroll redraws only the two rows it changes, and drawing starts below
  the visible screen, so nothing flickers. CHROUT isn't used: a full 40-column row links into the
  next line, and the last cell of row 24 scrolls the screen.
- The label in rows 0-3 is reverse characters (D-069): light grey cells, one cell a row in each
  band's colour, and the wordmark's cells. A reverse cell's ink is the background, so everything on
  the label is black.
- A raster interrupt is the menu's tick, once a frame at line 46 (D-048). It replaces the CIA
  timer's interrupt and runs the KERNAL's own work (`$EA31`) itself, so the keyboard is scanned 50
  times a second and the stick's repeat counts in frames. It draws nothing.
- The interrupt handler runs from the cartridge's ROM. So interrupts are masked whenever the menu
  maps another bank in, and every way out (a program, a cartridge, RUN/STOP) first puts back the
  KERNAL's interrupt: raster interrupt off, `$0314` at `$EA31`, the CIA timer on. The boot tests
  watch every write to `$DE00` and `$DE02` for a live interrupt (AC-127), and read VICE
  screenshots for the label's pixels (AC-207, AC-208).
- Keys come from GETIN, the KERNAL keyboard buffer, which is where VICE injects keys for the boot
  tests. The stick is read in the menu's own loop, once a jiffy, never in the raster interrupt. Port
  A is read with every keyboard column high, so a key held in the matrix can't pull a stick line
  low, and a stick move goes through the same code as the key it stands for. `menu_idle` is the
  point where the menu waits with an empty buffer and the stick at rest, and the tests break there.
  The stick tests plug VICE's "Joyport I/O simulation" device into port 2
  (`-controlport2device 37`) and set its lines from the monitor with `jpdb`.
- `src/launch.s` holds what has to run while the cartridge or its bank is switched away:
  - leaving
  - starting a cartridge: bank and mode, the entry's first bank left in `$DF00/$DF01` for xbank
    carts, then reset
  - starting a program: a BASIC cold start with CHRIN trapped. The trap copies the program bank by
    bank, up to `$D000`, and returns to the menu ROM. That copies the rest, if any (below), sets
    BASIC's pointers and types `RUN` or `SYS`, or nothing when `p_only` is set (SHIFT+RETURN).

  `$DF00-$DFFF` belongs to the menu once it is entered. The routines end by `$DFF5`, and assembly
  fails if they grow further.

- A program's bytes from `$D000` on (`p_tail` of them) are copied by `high_copy`, which the menu
  ROM puts at `$0100-$014F`, the bottom of the stack page, and calls (D-056). It can't run from
  `$DF00`: that page is I/O, and it is gone while the character ROM is mapped in. For each byte it
  calls `get_byte` with `$01=$37`, so the bank register and the source are there, then writes it
  with `$01=$33`. With the character ROM at `$D000` and the KERNAL at `$E000`, a write lands in the
  RAM under them. Interrupts are masked meanwhile, and the NMI vector points at an `RTI` in the
  routine: the KERNAL's handlers would read CIAs that aren't mapped in and write into that RAM. It
  puts back `$01`, the NMI vector, the bank and the interrupt flag before returning. VICE shows this
  works (AC-160..AC-162, AC-165): writes with `$01=$33` leave the chips, the bank register and the
  cart RAM alone. Real hardware hasn't been tried.

- The metadata entry is read once at boot, before the raster interrupt is on, so interrupts are
  still masked from `entry` (D-057). The menu takes the last hidden program slot named
  `STIDCART.META` and reads it through the program path (`prg_params`, `get_byte`), which follows
  all three program layouts; the boot tests place it interleaved only.
  It needs the load address `$4000` and a payload of at most 16 KB, and copies the payload to
  `$4000-$7FFF` with `high_copy`. It then checks the magic and the version and walks the records,
  stepping by their length bytes. Each record whose name matches its slot is attached to it in
  tables at `$C600-$C9FF`: the record's address and group by slot, and each group's count of
  visible entries. `$4000-$7FFF` isn't shown by VIC bank 0, and a program the menu starts may load
  over it. A full 16 KB takes 1.3 s to copy (1.31 million cycles in VICE, PAL), during which the
  screen stays blank.

- `$DFF6-$DFFF` holds the state a reset comes back to (D-053). The menu writes it when it starts an
  entry and on RUN/STOP: the magic "STID", the selected entry's directory slot, the order, the kind
  filter, the cart's number of directory entries, the 8-bit sum of the entry's name, and a checksum
  of the other nine bytes. At boot it restores them only when the magic and the checksum hold and
  the count, the slot and the name match this cart. A soft reset keeps the page; a power cycle
  fills it with a pattern that fails the check. RUN/STOP with nothing listed, or with a folder row
  selected, spoils the magic. The folder an entry comes back into is its group, so the state holds
  no folder.

## Rebuilding

```sh
menu/build.sh menu/
```

This assembles `src/menu.s` with 64tass (1.60.3243; the flags are in the script) into `menu.bin`,
the raw 8 KiB slot, and `menu.lbl`, the VICE labels. Commit both with any source change. They are
committed so building the Rust crates never needs the assembler. `stidcart-core`'s AC-93 test
reassembles the source and fails if either file differs, or if the menu outgrows the slot. In CI,
64tass is required, so that check always runs.

Carts built before a change keep the build they were made with, and stidcart still edits them (D-049).
Each earlier build is recognised by its fingerprint, listed in `crates/stidcart-core/src/menu.rs`, and
saving such a cart writes the current build. When `menu.bin` changes, the `menu_builds` test fails
until the replaced build's fingerprint is added to that list. `past/5b54e81.bin` is the build
stidcart 0.15 wrote; the tests use it to make a cart that carries an earlier build.

The evidence that the menu works is VICE, because the reference tool knows nothing of it:
`crates/stidcart-cli/tests/menu_boot.rs` boots carts and reads the screen back (AC-87..AC-92,
AC-136..AC-141 for the keys of D-051, AC-142..AC-146 for the stick, AC-147..AC-152 for coming
back after a reset, which the tests do with the monitor's `reset 0` and `reset 1`, and
AC-160..AC-162 and AC-165 for programs reaching `$D000-$FFFF`, read back as RAM with the monitor's
`bank ram`, AC-166..AC-169 for titles, folders and the metadata checks, and AC-170..AC-172 for
the `..` row).
