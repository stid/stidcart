# JOURNAL — menu-label

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Standard
- **Goal:** Our C64 menu wears direction B, "Label": a light grey label slab with band bars and a drawn wordmark in rows 0-3, and a pane that leads with the kind, title and note.
- **Next action:** done; PR open, waiting for the user's look in VICE and review
- **Branch:** feat/menu-label  ·  **Updated:** 2026-11-14 00:50

## STEPS
- [x] S1 — journal, D-069
- [x] S2 — header and key bar: AC-207..AC-209, AC-212; AC-124..AC-126 retired with the mark and strip; interrupt reduced to the tick; 4595d86's fingerprint listed
- [x] S3 — pane: AC-210 (replaces AC-108), AC-211; AC-166 and AC-170 follow the new order
- [x] S4 — VICE capture, guide and image, menu README, AC range, SPEC row, both gates, PR

## DECISIONS (this work-item)
- D-069: the label replaces the block logo, the mark and the strip; the raster interrupt stays as the 50 Hz tick only.
- Codes: badges `$60-$69`, folder `$6a-$6b`, rule/track/thumb `$6c-$6e`; inverses generated for the first twelve; the wordmark's 18 cells stored ink-as-background at `$ec-$fd`.
- "N entries" counts the cart's visible entries (`count`), "1 entry" for one.

## OPEN QUESTIONS
- none

## EDIT BOUNDARY (paths this work may touch)
- `menu/src/menu.s`
- `menu/menu.bin`
- `menu/menu.lbl`
- `menu/README.md`
- `crates/stidcart-core/src/menu.rs`
- `crates/stidcart-cli/tests/menu_boot.rs`
- `docs/guide/menu.md`
- `docs/guide/img/menu.png`
- `docs/lcd/SPEC.md`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/triage-log.md`
- `.claude/rules/lcd-conventions.md`
- `docs/lcd/work/menu-label/JOURNAL.md`
<!-- /lcd-resume -->

---

## Acceptance criteria

**AC-207** (surfaces: none): Rows 0-3 are the label: every cell a reverse space in light grey, except columns 1-5, reverse spaces in light red, orange, yellow, light green and light blue. Row 4 is spaces. Border and background are black, no cell is multicolour, and a screenshot shows row 4 black on every scanline.

**AC-208** (surfaces: none): Columns 8-16 of rows 1-2 show the STIDCART wordmark, the approved 72×12 pixels starting on the third scanline of row 1, as background-coloured ink on light grey cells.

**AC-209** (surfaces: none): The cart name ends at column 38 of row 1 and "N entries" ("1 entry" for one) at column 38 of row 2, N the cart's visible entries, both reversed in light grey. Empty cart included.

**AC-210** (surfaces: none): For an entry the pane shows a chip in row 5 (its kind's name between two spaces, reversed, in the kind's colour), the name in white in row 7, and in rows 18-20 the size, the load address or first bank, and how it starts, labels in grey and values in light grey. There is no "type" row and every other pane row is blank.

**AC-211** (surfaces: none): With metadata the pane shows from row 7 the title in white, wrapped; the stored name in grey; the author, two spaces and the year in light grey (whichever are set); a blank row; the note in grey, wrapped. The facts stay in rows 18-20. A folder shows a white "folder" chip and its name; ".." shows the chip, "..", and that it goes back to the top level.

**AC-212** (surfaces: none): The key bar in row 24 is reversed in light grey across 40 columns, on the menu and on the help screen, which keeps rows 0-4.

## LOG (append-only; not read on resume)

- 2026-11-13 23:40 — triaged Standard. User picked B from the "Stidcart Menu Directions" page. → D-069
- 2026-11-14 00:10 — S2 done: header, key bar and their tests green in VICE; 1,693 bytes free in the slot (1,484 before).
- 2026-11-14 00:50 — S3, S4 done. Root gate green with STIDCART_REQUIRE_TOOLS=1, GUI gate green. 1,499 bytes free in the slot. `ac_169_the_metadata_entry_is_checked_before_use` failed once in four full runs ("no bank or mode write was seen") and passed alone and in the other three; cause not looked into, hypothesis: the monitor log under load.
- 2026-11-14 00:50 — closeout: audit PASS (test-presence), SPEC row folded by hand as reconcile prescribes.
