# The stidcart menu

Our own C64-side menu (D-037). It lists a cart's entries and starts the one picked: a program, or a
cartridge of any kind except Ocean. It is opt-in. `build --menu stidcart` puts it into a cart, and
`edit --menu stidcart|easyloader` switches an existing one, replacing whatever loader it carries.
Without the option, carts keep EasyLoader, the zlib menu in `../firmware/`.

GPL-2.0-or-later, like everything outside `firmware/` (see `../COPYING`). It is written fresh: the
start sequences follow EasyLoader's technique, cited where they are used, and contain none of its
code.

## Using it

| Key             | Does                                                                     |
| --------------- | ------------------------------------------------------------------------ |
| CRSR ↓ / CRSR ↑ | move the selection. The list scrolls past 16 rows                        |
| CRSR → / CRSR ← | move 16 entries down or up, stopping at the last or first                |
| RETURN          | start the selected entry, open the selected folder, or on `..` leave it  |
| SHIFT+RETURN    | load the selected program and stop at `READY.`; on a cartridge, nothing  |
| letters, digits | filter: list only the entries whose name or title contains the text      |
| DEL             | remove the last character of the filter text; with none, leave a folder  |
| HOME / CLR      | clear the filter text; with none, go to the first row / last entry       |
| F1              | switch between the cart's own order and A-Z                              |
| F3              | list all entries, then only programs, then only cartridges               |
| F5              | select a random entry among those listed, never the one already selected |
| F7              | show every key and the stick; the next key press goes back to the menu   |
| RUN/STOP        | leave a folder; at the top level, leave to BASIC with the cartridge off  |

A joystick in port 2 does what the cursor keys and RETURN do (D-052):

| Stick        | Does                                     |
| ------------ | ---------------------------------------- |
| up / down    | as CRSR ↑ / CRSR ↓, one entry            |
| left / right | as CRSR ← / CRSR →, a page               |
| fire         | as RETURN: start, open a folder, or `..` |
| any, on help | go back to the menu, and nothing else    |

A direction acts when it is pushed, and if it is held it repeats after 0.4 s, then ten times a
second. Fire acts once per press. Anything held when the menu starts does nothing until it is let
go, so a stick resting on fire doesn't start the first entry. The same goes for a direction that
closed the help screen. Port 1 isn't read, because its lines are shared with the keyboard.

The screen is black (D-041) and wears the stidcart brand (D-048): the cartridge mark, its label in
five colour bands, sits left of a white block-letter logo, and a strip of the same five bands runs
under the cart name. The cart name sits above 16 list rows. Each
entry shows a badge for its kind, in the kind's colour: "PRG" for a program, "8K" and "16K" for
those cartridges, "ULT" for Ultimax, and "XB" for multi-bank. The selected entry sits in an orange
bar. When the list is longer than the screen, the divider to its right becomes a scroll bar: the
thumb's length is the share of the list in view, and its place is where the view is. The pane on the
right describes the selection: its type, size, load address (program) or first bank (cartridge),
and how it starts. Below the list are the position ("17 of 40"), the order in use and the kind
listed, then the filter text in yellow with the number listed out of the visible entries (inside a
folder, out of the folder's), then a grey bar naming the keys, `return go  f1 sort  f7 help  run/stop`. Hidden entries are not listed.
The filter and A-Z both ignore case. The filter takes up to 16 characters, any printable one except
`"`. A boot starts in the cart's own order, which is the order they were built in (D-039), with all
kinds listed, unless it comes back from a reset (below). The kind filter (F3) combines with the
text, and changing either lists again from the first match.

A move that leaves the view (CRSR, a page, HOME/CLR, F5) scrolls only as far as it takes to show the
selection, so one page down from the top puts it on the last row. F5 mixes the CIA timer, the raster
line and the jiffy clock, which differ at every key press a person makes. The help screen keeps the
logo and the cart name and replaces rows 5-24; the key that ends it, RUN/STOP and RETURN included,
does nothing else (D-051).

A program that loads at or below `$0801` is started with `RUN`, and any other with `SYS` to its load
address. SHIFT+RETURN copies a program the same way and sets BASIC's pointers, but types nothing, so
`LIST`, `RUN` or a `SYS` work from `READY.`. Either way the program finds the C64's own colours and
upper-case character set, as after a reset.

A program can run from below `$D000` up to `$FFFF`. Its bytes from `$D000` on go into the RAM
under I/O and the KERNAL, not into the VIC, SID, CIAs or colour RAM as a KERNAL `LOAD` would put
them (D-056). The tool refuses to put this menu on a cart with an Ocean entry (D-038), or with a
program that the menu can't start (D-056):

- a program that loads at `$D000` or above, since the menu would start it inside I/O or the KERNAL
- a program that runs past `$FFFF`
- a program that reaches `$D000` and loads below `$0150`, over the routine that copies that part

EasyLoader itself won't start programs that reach `$D000` (ndefpack's `easyloader.asm:957-985`).
The tool doesn't check EasyLoader carts for any of this.

## Titles and folders

A cart can carry a title, year, author, note and group for each entry, set with `edit --title`,
`--year`, `--author`, `--note`, `--group` or `--import`, or in the window (D-054). They are stored in a
hidden entry, `STIDCART.META`, which the menu reads once at boot (D-057).

- An entry with a title shows the title in its row instead of its name, cut to the row's 16
  characters. A-Z sorts by what the rows show.
- The pane shows the name as before, then the facts on every row instead of every other row, then
  the title in full, the year, the author and the note. Each starts on a row of its own and wraps
  every 16 characters, so all of them fit at their longest. An entry with no title, year, author
  or note keeps the usual pane.
- Groups are folders. The top level lists one row per group that has a visible entry, with a folder
  badge, the group's name and its number of entries, then the entries without a group. RETURN or
  fire opens a folder, with its first entry selected. Inside, the keys act on the folder's entries,
  and DEL with no text or RUN/STOP goes back to the top level with the folder selected. A name too
  long for the row is cut to leave room for the count.
- Inside a folder the first row is `..`, with the folder badge. RETURN or fire on it goes back to
  the top level as DEL and RUN/STOP do, so the joystick can leave a folder (D-058). It stays first
  under A-Z, the kind filter and filter text, even when nothing matches. It isn't an entry: rows
  22-23 don't count it, so on it the position reads 0, F5 never picks it, and a reset never comes
  back to it. HOME selects it and CLR the last entry.
- Text typed at the top level searches every entry, in every folder, by name or title, and lists
  the matches without folders.
- Folder rows ignore the kind filter and stay in the order the groups were first used, also under
  A-Z.

A cart whose metadata the menu can't use boots as a cart without any: no `STIDCART.META` entry, one
whose payload doesn't start with the magic, a layout version other than 1, a payload over 16 KB, or
a record that runs past the end. A record whose stored name differs from its entry's is ignored, so
a cart re-ordered by another tool shows names, not wrong titles.

## Coming back after a reset

After a soft reset the menu comes back where it was left: the entry last started, or the one
selected at RUN/STOP, is selected and in view, in the same order and kind filter. An entry in a
folder comes back inside that folder, wherever it was started from. The filter text isn't kept.
RUN/STOP with a folder row selected leaves nothing to come back to, so the next boot starts at the
top. If the kind filter would hide that entry, all kinds are listed. A power cycle starts at
the top, and so does a reset after a program that wrote the menu's state (an EAPI or xbank game
may). The state is checked before it is used, so state that was damaged or written for another cart
starts at the top too (D-053). This is proven in VICE; whether an EasyFlash 3's own RESET button
keeps the state on real hardware is untested.

## How it starts

The boot code in bank 0 ROMH hands over through the first word of the loader slot, `jmp ($8000)`,
in 16K mode at bank 0, with the KERNAL not initialised. The full contract is in
`../docs/format/easyflash-cart-format.md`, under "How the boot code hands over to the menu loader".

- `src/menu.s` runs from ROM. It copies `src/launch.s` to the cartridge's I/O2 RAM at `$DF02` and
  initialises the KERNAL from there with the cartridge off, so the top of memory comes out right.
  It copies the ROM's mixed-case character set to `$3800` and adds the menu's own glyphs to it in
  codes `$60-$7F`: the badges (with their inverses at `$E0` and up, for the selection bar), the rule,
  the scroll bar, the logo's quarter-block cells and the folder badge (`$7D-$7E`). The cartridge
  mark's nine multicolour cells go in `$EA-$F2`, reversed codes that names never reach. It then
  scans the EasyFS2 directory at `$A000`, reads the metadata entry (below), turns the raster
  interrupt on, and draws straight into screen and colour RAM, writing each cell once per update. A
  cursor move that doesn't scroll redraws only the two rows it changes, and drawing starts below
  the visible screen, so nothing flickers. CHROUT isn't used: a full 40-column row links into the
  next line, and the last cell of row 24 scrolls the screen.
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
