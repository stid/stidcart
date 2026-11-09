# JOURNAL — menu-raster

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the _entire_
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->

## NOW

- **Lane:** Standard
- **Goal:** Our C64 menu wears the raster brand (D-048): white logo beside the multicolour cartridge mark, a five-scanline raster strip under the cart name, an orange selection bar, the raster interrupt safe on every path
- **Next action:** done; the PR awaits CI and the user's merge (resolve the version line against #45 if it merges first)
- **Branch:** feat/menu-raster · **Updated:** 2026-11-08

## STEPS

- [x] S1 — version 0.15.0 → 0.16.0 (first commit on the branch)
- [x] S2 — triage, D-048, AC-123..AC-128 claimed, this JOURNAL
- [x] S3 — failing tests: the look (AC-123, updating AC-105..AC-107's frame), the strip and the mark from screenshots (AC-124, AC-125), frames (AC-126), bank switches (AC-127), exits (AC-128)
- [x] S4 — port the prototype into `menu/src/menu.s` (irq_off on every start, the late-entry guard), rebuild `menu.bin` and `menu.lbl`
- [x] S5 — README, one NTSC run, VICE live run for the user
- [x] S6 — gates with STIDCART_REQUIRE_TOOLS=1, audit, reconcile, PR

## DECISIONS (this work-item)

- option 3 of the prototypes; strip in row 4 one line a band; mark in multicolour at $EA-$F2; raster IRQ replaces the CIA timer's → D-048

## OPEN QUESTIONS

- none

## EDIT BOUNDARY (paths this work may touch)

- `menu/src/menu.s`
- `menu/src/launch.s`
- `menu/menu.bin`
- `menu/menu.lbl`
- `menu/README.md`
- `crates/stidcart-cli/tests/menu_boot.rs`
- `crates/stidcart-cli/tests/support/`
- `docs/lcd/SPEC.md`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/triage-log.md`
- `docs/lcd/work/menu-raster/JOURNAL.md`
- `.claude/rules/lcd-conventions.md`

<!-- /lcd-resume -->

---

## Acceptance criteria

**AC-123** (surfaces: none): Given our menu, when it boots, then the STIDCART logo is white from
column 7, the selection bar is orange, and row 4 holds spaces where the rule was. Row 21 keeps its
dark grey rule.

**AC-124** (surfaces: none): Given our menu, when a frame is captured, then scanlines 84-88 are each
one colour across the 320 visible pixels, light red, orange, yellow, light green and light blue in
that order, and scanlines 83 and 89 are black.

**AC-125** (surfaces: none): Given our menu, when a frame is captured, then columns 2-4 of rows 0-2
show the cartridge mark: a dark grey shell, yellow pins, and a label of five bands three lines each
in the strip's colours. Nothing else in rows 0-3 shows a colour other than black, white or the
cart name's light grey. Its glyphs sit in codes `$EA-$F2`.

**AC-126** (surfaces: none): Given a long list, when the selection scrolls, the filter text changes
and the order toggles, then the strip and the mark are identical in every captured frame.

**AC-127** (surfaces: none): Given our menu, from boot through browsing, filtering and each way out,
then no write to the bank or mode register (`$DE00`, `$DE02`) happens while the raster interrupt is
enabled and interrupts are not masked.

**AC-128** (surfaces: none): Given our menu, when it starts a program, starts a cartridge or leaves
with RUN/STOP, then the IRQ vector is back at `$EA31` and the raster interrupt is off, as the stock
menu left them.

## LOG (append-only; not read on resume)

- 2026-11-08 — the user ran the option-3 prototype in VICE and picked it. Triage: Standard, 4
  signals. D-048. AC-123..AC-128 claimed, next free AC-129.
- 2026-11-08 — review of the prototype before porting found two of its gaps: `start_prg` reads
  the load address (another bank) with the handler armed, and the cart start left `$0314` in menu
  ROM. Both close with `irq_off` before any start path maps a bank in. The canvas board's "masked
  around that read" was true for the pane only; it will be true for both reads after S4.
- 2026-11-08 — S3 red as expected on the stock menu: AC-106/107/123..127 (AC-126 has no `irq`
  label to break on). AC-127 and AC-128 pass on the stock menu, which has no raster interrupt, so
  both were run against the option-3 prototype too: AC-128 caught its cart start leaving `$0314` in
  menu ROM, and AC-127 caught its program start mapping a bank in with the handler live, once its
  browse scenario was made to end on a program (F1 had put the cart first).
- 2026-11-08 — S4: `irq_off` before each start maps a bank in, a `stop` routine for RUN/STOP, `sei`
  around the pane's load-address read, a late-entry guard in the handler. 4040 of 8192 bytes.
  Green: menu_boot 36/0, root gate 237/0 with STIDCART_REQUIRE_TOOLS=1. VICE's BMP screenshots are
  4-bit with the palette in VIC-II order, so a pixel is its C64 colour number; `Shot::parse` checks
  the palette starts black, white.
- 2026-11-08 — NTSC run (`-ntsc`, the built CLI's menu): the mark, the strip and the pane render as
  on PAL. A capture taken at the first idle showed a pane mixing two entries: VICE's picture of the
  frame in progress still holds the previous frame below the beam. So AC-124 and AC-125 now wait two
  interrupts before their screenshot. README describes the brand and the interrupt. Gate 237/0.
- 2026-11-08 — the user ran the committed menu (`final.crt`, built by the CLI) in VICE: "ok for now".
- 2026-11-08 — audit: PASS (test-presence). All six ACs are surface `none`, so there is no token row. Their tests are `ac_123`..`ac_128` in `menu_boot.rs`, plus the `bank_writes` parser's unit test, green at d12a16d with STIDCART_REQUIRE_TOOLS=1 (root 237/0, GUI gate passing with 54/0 vitest). Closeout line appended.
- 2026-11-08 — reconciled: folded AC-123..AC-128 → SPEC.md/none (the menu-look row rewritten in place); D-040 and D-041 marked superseded in part by D-048.
- 2026-11-08 — CI red on AC-127 only: "no bank or mode write was seen". The headless VICE that CI builds prints each fragment of monitor output on its own `Monitor:` line in the log, so the register lines never reached the parser whole. AC-127 now reads the monitor's own log file (`logname`, `log on`), which reads the same in both builds; CI green at faaf6b3.
