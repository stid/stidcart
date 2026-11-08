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
- **Next action:** S3 — failing boot tests for AC-142..AC-146
- **Branch:** feat/menu-joystick · **Updated:** 2026-11-08

## STEPS

- [x] S1 — version 0.18.0 → 0.19.0 (first commit on the branch)
- [x] S2 — triage, D-052, AC-142..AC-146 claimed, this JOURNAL
- [ ] S3 — `support/vice.rs` takes extra emulator arguments; failing boot tests in `menu_boot.rs` for AC-142..AC-146
- [ ] S4 — implement in `menu/src/menu.s`, rebuild `menu.bin`/`menu.lbl`, the replaced build's fingerprint joins `EARLIER_STIDCART_BUILDS`
- [ ] S5 — `menu/README.md`, gates with `STIDCART_REQUIRE_TOOLS=1`
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
