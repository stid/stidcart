# JOURNAL — menu-look

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the _entire_
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->

## NOW

- **Lane:** Standard
- **Goal:** A designed look for our C64 menu: dark colours, a cyan title bar, a cyan selection bar and a grey help bar (D-040)
- **Next action:** done; the PR is open and awaits CI (gate, gui, claude-review) and the user's merge
- **Branch:** feat/menu-look · **Updated:** 2026-11-07

## STEPS

- [x] S1 — version 0.10.0 → 0.11.0 (first commit on the `feat/` branch)
- [x] S2 — triage, D-040, AC-102..AC-104 claimed, this JOURNAL
- [x] S3 — failing tests AC-102, AC-103 in `menu_boot.rs`: colour RAM and `$D020-$D021` dumps masked to the low nybble, each check with a negative control on a default-colour screen
- [x] S4 — `draw` writes screen and colour RAM directly: title bar, filter row, list with selection bar, status row, help bar. Colours set after `kernal_init`. AC-87..AC-101 stay green
- [x] S5 — AC-104 red (a started program inherits the menu's colours), then the restore in `start_prg`
- [x] S6 — `menu/README.md`, `menu.bin`/`menu.lbl` reassembled with every menu commit (AC-93)
- [x] S7 — VICE screenshot shown to the user, full gate, audit, reconcile SPEC, PR

## DECISIONS (this work-item)

- Colour and layout only, dark scheme with cyan accents, bars top and bottom → D-040
- Rows stay where AC-87..AC-101 pin them; row 24 is the new help bar → D-040
- Mixed-case ROM character set in the menu, upper case restored for programs → D-040
- New code goes in `menu.s` (ROM), never `launch.s`, which has about 10 bytes left

## OPEN QUESTIONS

- none

## EDIT BOUNDARY (paths this work may touch)

- `menu/src/menu.s`
- `menu/menu.bin`
- `menu/menu.lbl`
- `menu/README.md`
- `crates/stidcart-cli/tests/menu_boot.rs`
- `crates/stidcart-cli/tests/support/vice.rs`
- `Cargo.toml`
- `Cargo.lock`
- `gui/src-tauri/Cargo.toml`
- `gui/src-tauri/Cargo.lock`
- `docs/lcd/work/menu-look/JOURNAL.md`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/SPEC.md`
- `docs/lcd/triage-log.md`
- `.claude/rules/lcd-conventions.md`

<!-- /lcd-resume -->

---

## Acceptance criteria

All are proven by booting carts in VICE, as AC-87..AC-101 are. Colours are read from colour RAM
(`$D800-$DBE7`) and `$D020-$D021`, low nybble only, since VICE fills the high one from the bus.
Indices: black 0, cyan 3, yellow 7, light blue 14, blue 6, medium grey 12, light grey 15.

**AC-102** (surfaces: none): Given a cart carrying our menu, when it boots, then the screen is in
the ROM's mixed-case character set, the border and background are black, row 0 is a cyan bar in reverse across all 40 columns carrying "stidcart" and
the cart name, the listed entries are light grey and the selected one sits in a cyan reverse bar,
the count and order on row 23 are medium grey, and row 24 is a medium grey reverse bar across all 40
columns naming the keys. The bars are there when the cart has no visible entries too.

**AC-103** (surfaces: none): Given typed text, when the screen is shown, then the text on row 1 is
yellow after a medium grey label, and with no text row 1 stays empty.

**AC-104** (surfaces: none): Given our menu, when it starts a program or leaves to BASIC, then what
runs next finds the C64's own colours (border light blue, background blue, text colour light blue)
and the upper-case character set. Nothing it starts, a cartridge included, finds SHIFT+C= locked.

## LOG (append-only; not read on resume)

- 2026-11-07 — Triage: Standard (2 signals). User picked colour and layout only (no font, sprites or
  raster), the dark scheme with a cyan accent, and bars top and bottom → D-040. Advisor review:
  keep every row AC-87..AC-101 pins (so the preview's one-line bottom becomes status on row 23 plus
  a help bar on row 24), avoid CHROUT for bars (line links, row-24 scroll), mask colour reads to the
  low nybble, and catch colours leaking into started programs: `basic_go` enters the reset at
  `$FCFE`, after CINT. AC-102..AC-104 claimed, next free AC-105.
- 2026-11-07 — Red run: AC-102 and AC-103 failed on colours (border 14/6, typed text 14); AC-104
  passed, as expected before the menu changed colours. After the new `draw`, AC-104 went red
  (border and background 0 in the started program), then green with the restore in `start_prg`.
  AC-87..AC-101 stayed green without edits.
- 2026-11-07 — Refined: the first VICE screenshot showed capitals in names ("Cave Pilot") as
  graphics. That isn't new: names keep upper case as `$61-$7A`, which the upper-case set draws as
  graphics, and CHROUT did the same. The menu now uses the mixed-case set and restores the
  upper-case one for programs. AC-102 and AC-104 check `$D018`. With the restore removed, AC-104
  failed on `$D018` (23, not 21). Recorded in D-040.
- 2026-11-07 — Screenshot gotchas: VICE's monitor reads `ignore` counts as hex, and checkpoint
  numbers restart at 1 after `del`. A screenshot taken right at `menu_idle` can show a half-drawn
  frame, so wait about 20000 idle hits first.
- 2026-11-07 — audit: PASS (test-presence). All three ACs are surface `none`, so as in menu-search-sort there is no token row. Their tests are `ac_102`..`ac_104` in `menu_boot.rs`, plus the negative control `the_look_checks_reject_the_default_screen`, green at 6b80bb6 with STIDCART_REQUIRE_TOOLS=1 (root 219/0, GUI 31/0). Closeout line appended.
- 2026-11-07 — reconciled: folded AC-102..AC-104 → SPEC.md/Internal as 1 new row. The boot row (AC-87..AC-92) and the filter row (AC-96..AC-101) still hold, so nothing was superseded.
- 2026-11-07 — Advisor review after the PR opened: the menu's SHIFT+C= lock (`$0291=$80`) reached
  started cartridges, whose reset runs before CINT. AC-104 now dumps `$0291` on all three paths.
  It was red on the cart path (128) and green on the other two, where CINT clears it. `_cart` now
  clears it.
