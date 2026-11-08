# JOURNAL — menu-keys

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the _entire_
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->

## NOW

- **Lane:** Standard
- **Goal:** Our C64 menu gains SHIFT+RETURN load-without-run, page keys, HOME/CLR to the ends, an F3 kind filter, an F5 random pick and an F7 help screen (D-051)
- **Next action:** S3 — failing boot tests for AC-136..AC-141
- **Branch:** feat/menu-keys · **Updated:** 2026-11-08

## STEPS

- [x] S1 — version 0.17.0 → 0.18.0 (first commit on the branch)
- [x] S2 — triage, D-051, AC-136..AC-141 claimed, this JOURNAL
- [ ] S3 — failing boot tests in `menu_boot.rs` for AC-136..AC-141
- [ ] S4 — implement in `menu/src/menu.s` (and `launch.s` for AC-136), rebuild `menu.bin`/`menu.lbl`, the replaced build's fingerprint joins `EARLIER_STIDCART_BUILDS`
- [ ] S5 — `menu/README.md` key table and help bar, gates with `STIDCART_REQUIRE_TOOLS=1`
- [ ] S6 — audit, reconcile, closeout, PR

## DECISIONS (this work-item)

- keys and behaviours → D-051

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
- `docs/lcd/work/menu-keys/JOURNAL.md`
- `.claude/rules/lcd-conventions.md`

<!-- /lcd-resume -->

---

## Acceptance criteria

**AC-136** (surfaces: none): Given a program selected, when SHIFT+RETURN is pressed, then the program
is in memory at its load address, the C64 is at `READY.` with nothing typed after it, BASIC's
pointers end the program so `LIST` and `RUN` work for a BASIC program, and it is left as RETURN
leaves a started program: cart off, the C64's own colours and upper-case set, the IRQ vector at
`$EA31` with the raster interrupt off. Given a cartridge selected, SHIFT+RETURN changes nothing.

**AC-137** (surfaces: none): Given more entries listed than fit the screen, when CRSR → is pressed,
then the selection moves 16 entries down, or to the last entry if fewer remain, and the view scrolls
to show it; CRSR ← moves 16 up, or to the first entry. On a list that fits, they go to the last and
first entry.

**AC-138** (surfaces: none): Given filter text, HOME and CLR still clear it (D-039). Given no filter
text, HOME selects the first entry listed and CLR the last, scrolling the view to show it.

**AC-139** (surfaces: none): Given the menu, when F3 is pressed, then the kind filter steps all →
programs → cartridges → all; only entries of that kind are listed, together with the text filter;
row 22 names the kind beside the order; the match count counts what is listed; the selection goes to
the first match. A kind with no match says so as the text filter does.

**AC-140** (surfaces: none): Given several entries listed, when F5 is pressed, then the selection is
one of the entries listed (with the text and kind filters applied), the view shows it, and nothing
is started; across repeated presses more than one entry is chosen. With one entry or none listed,
F5 changes nothing.

**AC-141** (surfaces: none): Given the menu, when F7 is pressed, then a full screen of help names every
key the menu takes (this item's and the existing ones); the next key press returns to the menu with
the same selection, filter text, kind filter and order, and that key does nothing else. Row 24's
help bar names F7. The raster strip and mark need not show on the help screen, and AC-127's rule
(no bank switch with the raster interrupt live) holds through it.

## LOG (append-only; not read on resume)

- 2026-11-08 — Triage: Standard (3 signals: files, cold pickup, a binary rebuild). The user asked
  for every menu improvement from the brainstorm, orchestrated with agents; this is the first item.
  Spec approval for it is delegated to me, so the keys are decided in D-051. AC-136..AC-141
  claimed, next free AC-142.
