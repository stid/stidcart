# JOURNAL — menu-details

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the _entire_
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->

## NOW

- **Lane:** Standard
- **Goal:** Layout C for our C64 menu: block logo, type pictograms, details pane, scroll bar, from a RAM character set, without flicker (D-041)
- **Next action:** S7, VICE live run for the user, audit, reconcile, PR
- **Branch:** feat/menu-details (stacked on feat/menu-look, PR #38) · **Updated:** 2026-11-07

## STEPS

- [x] S1 — version 0.11.0 → 0.12.0 (first commit on the branch)
- [x] S2 — triage, D-041, AC-105..AC-111 claimed, this JOURNAL
- [x] S3 — RAM character set at `$3800` with the menu's glyphs and icon inverses (AC-105). Tests: `MIXED_CASE` reads `$1F`
- [x] S4 — the new screen: logo, rules, list with icons in columns 1-20, help bar, write-once rows (AC-106, AC-107). The test row constants move and `listed()` reads columns 0-20 only; AC-102/103 retire
- [x] S5 — details pane (AC-108), scroll bar (AC-109), status and filter rows (AC-110)
- [x] S6 — partial redraw on a cursor move, raster wait (AC-111)
- [ ] S7 — README, VICE screenshots and a live run for the user, full gate, audit, reconcile, PR based on `feat/menu-look`

## DECISIONS (this work-item)

- Layout C, pictograms, RAM charset, position counter with the match count by the filter text, write-once partial redraw → D-041
- New code goes in `menu.s` (ROM), never `launch.s`, which has about 10 bytes left. The pane's load address comes through the existing `read_load`
- No `$DExx/$DFxx` access while CHAREN is clear for the charset copy

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
- `docs/lcd/work/menu-details/JOURNAL.md`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/SPEC.md`
- `docs/lcd/triage-log.md`
- `.claude/rules/lcd-conventions.md`

<!-- /lcd-resume -->

---

## Acceptance criteria

All are proven by booting carts in VICE. The approved bitmaps are in the artifact "Stidcart Menu
Layouts" (https://claude.ai/artifact/Gj6xQWUEDP3uF89k2qn15a), layout C, pictograms.

**AC-105** (surfaces: none): Given our menu, when it boots, then the VIC reads characters from a
RAM copy at `$3800` whose every code outside the menu's own glyphs equals the ROM's mixed-case set,
whose own glyphs are the approved bitmaps, and in which each icon's code+`$80` holds its inverse.

**AC-106** (surfaces: none): Given our menu, when it boots, then rows 0-2 show "STIDCART" in block
letters, white, cyan and light blue by row; row 3 shows the cart name; rows 4 and 21 are dark grey
rules; row 24 is the grey reverse help bar; border and background are black. This holds with no
visible entries too.

**AC-107** (surfaces: none): Given entries of every kind the menu starts, when they are listed,
then each sits in columns 1-20 of rows 5-20 with its kind's icon in column 2 (program light green,
8K yellow, 16K light red, Ultimax purple, multi-bank light blue) and its name from column 4 in
light grey. The selection is a cyan reverse bar over columns 1-20, its icon reversed.

**AC-108** (surfaces: none): Given a selection, when the screen is shown, then the pane shows its
name in white and its type, size in KB, load address (program) or first bank (cartridge), and how
it starts: `RUN` for a load address at or below `$0801`, `SYS` and the address above it, `reset`
for a cartridge. Moving the selection updates the pane.

**AC-109** (surfaces: none): Given more entries listed than the 16 rows, when the screen is shown,
then column 21 shows a thumb of `round(256/n)` cells, at least 1, placed `round((16-len)*top/(n-16))`
rows down the track, where n is the number listed and top the first one on screen. With 16 or
fewer listed it shows the track only.

**AC-110** (surfaces: none): Given the screen, then row 22 shows the selection's position among
those listed ("k of n") and the order in use. With typed text, row 23 shows a grey "find:" label,
the text in yellow, and the number of matches out of the visible entries. Without text, row 23 is
empty.

**AC-111** (surfaces: none): Given a list, when a cursor move doesn't scroll, then list rows other
than the old and the new selection are not rewritten.

AC-102 and AC-103 (menu-look) are superseded by AC-105..AC-110. AC-104 stands.

## LOG (append-only; not read on resume)

- 2026-11-07 — Triage: Standard (3 signals). The user picked layout C from the mockups, asked for
  a scroll bar during review, and said "let's go with this" with pictograms showing → D-041. They
  asked whether search stays: it does, on row 23. Advisor review: keep the match count beside the
  filter text (no downgrade), restrict `listed()` to columns 0-20 because the pane repeats the
  selected name, write the icon inverses at code+$80, keep I/O untouched while CHAREN is clear,
  pad write-once rows to their end column, and prove the partial redraw with a sentinel cell.
  AC-105..AC-111 claimed, next free AC-112.
- 2026-11-07 — Red run: 15 of 29 failed for the expected reasons (no RAM set, nothing at the new
  positions, every row rewritten on a move). The 14 that passed don't depend on the layout.
- 2026-11-07 — Refined: S3-S6 landed as one change, because the row map, the write-once drawing
  and the partial redraw share `draw`. A split would have needed an interim layout that no test
  describes. Green after one fix to the test, not the menu: AC-108 expected `$0801` for
  `basic_prg`, which loads at `$07F0` on purpose (only `RUN` starts it). Root gate 225/0 with
  STIDCART_REQUIRE_TOOLS=1, GUI 31/0 cargo + 40/0 vitest.
- 2026-11-07 — VICE screenshots (boot, scrolled 20 rows, "ra" with A-Z) match the approved mockup.
  The partial redraw is proven by AC-111. Write-once and the raster wait aren't visible in dumps,
  so the live run is the check for flicker.
