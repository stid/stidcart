# JOURNAL — menu-joystick

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the _entire_
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->

## NOW

- **Lane:** Standard
- **Goal:** Our C64 menu takes a joystick in port 2: moves, paging and fire act as the keys do, with repeat while held (D-052)
- **Next action:** S6 — audit, reconcile into SPEC, closeout, PR
- **Branch:** feat/menu-joystick · **Updated:** 2026-11-08

## STEPS

- [x] S1 — version 0.18.0 → 0.19.0 (first commit on the branch)
- [x] S2 — triage, D-052, AC-142..AC-146 claimed, this JOURNAL
- [x] S3 — `support/vice.rs` takes extra emulator arguments; failing boot tests in `menu_boot.rs` for AC-142..AC-146
- [x] S4 — implement in `menu/src/menu.s`, rebuild `menu.bin`/`menu.lbl`, the replaced build's fingerprint joins `EARLIER_STIDCART_BUILDS`
- [x] S5 — `menu/README.md`, gates with `STIDCART_REQUIRE_TOOLS=1`
- [ ] S6 — audit, reconcile, closeout, PR

## DECISIONS (this work-item)

- port 2 only, stick as key presses, repeat while held, fire held at start ignored → D-052

## OPEN QUESTIONS

- none

## EDIT BOUNDARY (paths this work may touch)

- `menu/src/menu.s`
- `menu/src/launch.s`
- `menu/menu.bin`
- `menu/menu.lbl`
- `menu/README.md`
- `crates/stidcart-core/src/menu.rs`
- `crates/stidcart-core/tests/menu_builds.rs`
- `crates/stidcart-cli/tests/menu_boot.rs`
- `crates/stidcart-cli/tests/support/`
- `Cargo.toml`
- `Cargo.lock`
- `gui/src-tauri/Cargo.toml`
- `gui/src-tauri/Cargo.lock`
- `docs/lcd/SPEC.md`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/triage-log.md`
- `docs/lcd/work/menu-joystick/JOURNAL.md`
- `.claude/rules/lcd-conventions.md`

<!-- /lcd-resume -->

---

## Acceptance criteria

**AC-142** (surfaces: none): Given the menu with a stick in port 2 at rest, when it is moved down,
then the selection moves one entry down; up moves one back; right and left move a page as CRSR → and
CRSR ← do (D-051).

**AC-143** (surfaces: none): Given the stick held down, then the selection moves more than one entry
while it is held and stops moving once it is released.

**AC-144** (surfaces: none): Given a program or a cartridge selected, when fire is pressed, then it
starts as RETURN starts it, and AC-128's exit state holds (IRQ vector at `$EA31`, raster interrupt
off). Fire held while the menu starts does nothing until it is released and pressed again.

**AC-145** (surfaces: none): Given the help screen (F7), when the stick moves or fire is pressed, then
the menu returns as it was and that input does nothing else.

**AC-146** (surfaces: none): Given the stick at rest in port 2, then the keyboard works as before, and
AC-127 holds while the stick is used: no write to `$DE00`/`$DE02` with the raster interrupt live and
interrupts unmasked.

## LOG (append-only; not read on resume)

- 2026-11-08 — Triage: Standard (2 signals: files incl. a binary rebuild, may span a session). Second
  item of the user's menu improvements. Recon (2026-11-08, VICE 3.10 verified): `-controlport2device
37` plus monitor `jpdb 1 <value>` (port index 0-based, so 1 is port 2) drives CIA1 port A through
  `read_ciapa` like a real stick. Values are active-low: `1f` rest, `1e` up, `1d` down, `1b` left,
  `17` right, `0f` fire. The device starts with every line pressed, so `jpdb 1 1f` must be the first
  monitor command. It reads bits 5-7 of `$DC00` as 0, so the menu masks with `#$1f`. The levels are
  held while warp runs many frames between breakpoints, so a test should press, break at an exec
  label in the stick handler, then release and break at `menu_idle`. `boot()` in `vice.rs` takes
  fixed arguments, so it needs a way to add the device for these tests only. A recon script that
  worked is at the session scratchpad `recon-vice/work/joy/mon.txt`. AC-142..AC-146 claimed, next
  free AC-147.
- 2026-11-08 — S3. `vice.rs`: `boot_with` (extra args), `Step::Stick` (`jpdb 1 xx`), `stick::*`,
  `STICK_DEVICE`. Tests use frames as the clock (`break exec .irq` + n×`g`), not handler labels:
  press, n frames, release, `menu_idle`. The menu will reach `menu_idle` only with the stick seen at
  rest, so that idle proves the release was read. AC-146's watch scenario uses raw `g`/`jpdb` only
  after the watches, since a `Step::Break` `del` clears them. Red run: AC-142 "down: e02 is not the
  one selected" (still e01); AC-143 "moved 0 entries"; AC-144 held-fire screen dumped, then fire never
  reached the program's spin; AC-145 set-up "e02 is not the one selected"; AC-146 keyboard half green
  (same screens with the device at rest), watch half "only 0 bank or mode writes".
- 2026-11-08 — S4. `stick` in `menu.s`, called from `loop` after an empty GETIN; its key goes
  through `key`, so help, RETURN and the exits need nothing new. Sampled once a jiffy (debounce).
  Port A read with `$FF` in `$DC00` under `sei`, then `$7F` back: the KERNAL leaves `$7F` (checked
  in ROM, `$EB42`), and a column driven low lets SPACE+B ghost PA3 low on real hardware. VICE models
  no such ghosting, so that holds by construction only. Repeat due = `$A2`+20, then +5, signed
  compare. Anything held at the start waits for a release (`joy_last`=`$1f`, `joy_dead` set), so
  a direction held at the start does nothing either. `leave_help` sets `joy_dead`, so a held
  direction that closed the help never repeats. A mutant without that fails AC-145. AC-143 saw 9
  moves in 60 frames, as predicted. AC-145's colour compare masked to 4 bits: colour RAM's high
  nibble is open bus. ROM 4906 → 5041 bytes (ends `$93B1`). 63ba913's build joins
  `EARLIER_STIDCART_BUILDS`. Gate 260/0, GUI gate green.
- 2026-11-08 — S5. README: a stick table under the keys, the timing and the held-at-start rule, and
  how the stick is read and tested. The on-screen F7 help still names keys only. Adding the stick
  there would change `menu.bin` again, and no AC asks for it.
