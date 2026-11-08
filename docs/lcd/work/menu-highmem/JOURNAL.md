# JOURNAL — menu-highmem

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the _entire_
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->

## NOW

- **Lane:** Standard
- **Goal:** Our menu starts programs reaching into `$D000-$FFFF`, copying that part into the RAM under I/O and the KERNAL; with our menu the tool refuses only what still can't start (D-056)
- **Next action:** S3 — failing tests for AC-160..AC-165
- **Branch:** feat/menu-highmem · **Updated:** 2026-11-08

## STEPS

- [x] S1 — version 0.20.0 → 0.21.0 (first commit on the branch)
- [x] S2 — triage, D-056, AC-160..AC-165 claimed, this JOURNAL
- [ ] S3 — failing tests: VICE boots (AC-160..AC-162, AC-165), CLI refusals (AC-163, flipping AC-95's test), the window (AC-164)
- [ ] S4 — the refusal in the core; the split copy in `menu/src/menu.s`/`launch.s`; rebuild `menu.bin`/`menu.lbl`; the replaced build's fingerprint joins `EARLIER_STIDCART_BUILDS`
- [ ] S5 — `menu/README.md`, SPEC-visible refusal text, gates with `STIDCART_REQUIRE_TOOLS=1` and `gui-gate.sh`
- [ ] S6 — audit, reconcile, closeout, PR

## DECISIONS (this work-item)

- split copy at `$D000`, the rest by a routine at `$0100` writing with `$01=$33`; bytes go to RAM, not I/O; what is still refused → D-056

## OPEN QUESTIONS

- none

## EDIT BOUNDARY (paths this work may touch)

- `menu/src/menu.s`
- `menu/src/launch.s`
- `menu/menu.bin`
- `menu/menu.lbl`
- `menu/README.md`
- `crates/stidcart-core/src/lib.rs`
- `crates/stidcart-core/src/menu.rs`
- `crates/stidcart-core/tests/`
- `crates/stidcart-cli/src/`
- `crates/stidcart-cli/tests/`
- `gui/src-tauri/tests/menu.rs`
- `gui/src/lib/Listing.test.ts`
- `README.md`
- `Cargo.toml`
- `Cargo.lock`
- `gui/src-tauri/Cargo.toml`
- `gui/src-tauri/Cargo.lock`
- `docs/lcd/SPEC.md`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/triage-log.md`
- `docs/lcd/work/menu-highmem/JOURNAL.md`
- `.claude/rules/lcd-conventions.md`

<!-- /lcd-resume -->

---

## Acceptance criteria

**AC-160** (surfaces: none): Given a program loading below `$D000` whose bytes run into `$D000-$FFFF`
(for example `$C000` with `$4000` bytes, each byte a function of its address), when our menu starts
it, then every byte is in RAM at its address, `$D000-$FFFF` included, as read back with I/O and ROM
banked out; the I/O chips are not written by the copy (the program's own output still shows on the
screen); and it is started by `SYS` to its load address as today.

**AC-161** (surfaces: none): AC-160 holds when the program's bank boundary falls inside
`$D000-$DFFF`, and when the program is started with SHIFT+RETURN, which stops at `READY.` with the
bytes in place.

**AC-162** (surfaces: none): Given a BASIC program loading at `$0801` whose bytes run past `$D000`,
when our menu starts it, then it runs by `RUN`, with BASIC's pointers set from its end.

**AC-163** (surfaces: CLI): Given `--menu stidcart` (on `build`, or on `edit`, `--add` or
`--replace` with our menu), a program that loads at `$D000` or above, one whose bytes end past
`$FFFF`, and one that reaches `$D000` while loading below the second copy routine's end are each
refused naming the entry, and nothing is written. Any other program reaching into `$D000-$FFFF` is
accepted (AC-95's refusal is gone for it). With EasyLoader, nothing about these programs changes.

**AC-164** (surfaces: RENDER): Given the window, choosing stidcart for a cart holding a program the
CLI still refuses shows that refusal and changes nothing; a cart whose only reason was a program
reaching into `$D000-$FFFF` from below now takes stidcart.

**AC-165** (surfaces: none): Given the second copy, AC-127 and AC-128 hold: no bank or mode write
with the raster interrupt live and interrupts unmasked, and the IRQ vector at `$EA31` with the raster
interrupt off when the program starts. The NMI vector and `$01` are back to the KERNAL's values.

## LOG (append-only; not read on resume)

- 2026-11-08 — Triage: Standard (3 signals: files across core/CLI/GUI tests/menu, a binary rebuild,
  may span a session). Fifth of the user's menu improvements; it goes past EasyLoader, which refuses
  these programs (`ndefpack/src/fw/easyloader.asm:957-985`). Recon (2026-11-08, VICE 3.10):
  - With the cart in 16K mode and `$01=$33`, ROML still reads at `$8000` and `$D000` reads the
    character ROM. Writes to `$D020`, `$DE00` and `$DF00` land in RAM, and the chips, the bank
    register and the cart RAM are untouched. Setting `$01=$37` brings `$DE00` back. This hasn't been
    checked on real hardware.
  - Design: `copy_prg` splits at `$D000`, keeping the tail length in a new `p_tail` (2 bytes of `$DF`
    RAM). menu-resume left `$DFF3-$DFF5` free, so the assembly cap must still pass.
  - The trap copies the head as today. Then `copied` puts about 50 bytes of routine at `$0100` and
    calls it:
    - `sei`, with `$0318` pointed at an `RTI`, so an NMI with CHAREN=0 can't write program RAM.
    - It sets `BANK=p_cbank`. For each byte it does `jsr get_byte` with `$01=$37`, then
      `$01=$33; sta (dst),y; $01=$37`, stepping `dst` and `p_tail`.
    - It restores `$0318` and sets `BANK=0` and `cli`.
  - Today's path then follows. When the end is `$10000`, VARTAB is `$0000`, as after a KERNAL LOAD.
  - `touches_io_area` (`crates/stidcart-core/src/lib.rs:544`) tests `start <= $DFFF && end > $D000`.
    A program loading at `$E000` passes it today, and the menu would `SYS` into the KERNAL ROM: a
    live bug, which AC-163 closes. Nothing limits a program's end to `$FFFF`. The format's limit is
    `$E000` bytes (`prg.c:39`).
  - Test idea: a marker PRG at `$C000`, `$4000` bytes long, ending exactly at `$10000`, filled with
    f(addr). Its code does `sei; $01=$34`, compares, sets `$37` and prints OK or BAD. Separately, the
    monitor's `bank ram` then a dump of `$D000-$FFFF`, checked in Rust (`bsave` follows the monitor
    bank). Add a filler entry sized so the bank wrap falls inside `$D000-$DFFF`, a maximum PRG
    `$0801-$E7FF` started by RUN, and CLI refusals. The AC-95 refusal test
    (`crates/stidcart-cli/tests/menu_cli.rs:275`) flips, and the window's AC-113 case
    (`gui/src-tauri/tests/menu.rs:248`) moves to a program still refused.
  - A recon monitor script and cart are in the session scratchpad (`recon-d000/`).
  - AC-160..AC-165 claimed, next free AC-170 (AC-153..AC-159 are menu-meta's, AC-166..AC-169
    menu-folders').
