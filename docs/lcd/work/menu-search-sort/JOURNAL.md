# JOURNAL — menu-search-sort

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the _entire_
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->

## NOW

- **Lane:** Standard
- **Goal:** Search (substring filter as you type) and an F1 sort toggle (directory order ↔ A-Z) in our C64 menu (D-039)
- **Next action:** S3, the failing boot tests in `crates/stidcart-cli/tests/menu_search.rs`
- **Branch:** feat/menu-search-sort · **Updated:** 2026-11-07

## STEPS

- [x] S1 — version 0.9.0 → 0.10.0 (first commit on the `feat/` branch)
- [x] S2 — triage, D-039, AC-96..AC-101 claimed, this JOURNAL
- [ ] S3 — `vice::key` gets DEL/HOME/CLR/F1. Failing tests AC-96..AC-101 in `menu_search.rs`, each with a negative control ← next
- [ ] S4 — menu refactor, no behaviour change: `visible` stays the base list, and a view array with its own count feeds draw/up/down/start. Query on row 1, status on row 23 (no RETURN on the last rows). AC-87..AC-92 stay green
- [ ] S5 — filter: typed characters, DEL, CLR/HOME, cap 16, fold, "no matches" (AC-96..AC-99)
- [ ] S6 — F1 sort, computed on first press and kept, stable (AC-100, AC-101)
- [ ] S7 — `menu/README.md` key table. `menu.bin`/`menu.lbl` reassembled with every menu commit (AC-93)
- [ ] S8 — full gate, audit, reconcile SPEC row "Booted, our menu shows…", PR

## DECISIONS (this work-item)

- Substring filter as you type, F1 view toggle starting in directory order → D-039
- Keys: printable characters `$20-$5F` and `$C1-$DA` type, DEL `$14` erases, HOME `$13` or CLR `$93` clears, F1 `$85` toggles. `"` is not accepted into the query, because CHROUT's quote mode would print the reverse-off code as a glyph
- New code goes in `menu.s` (ROM), never `launch.s`, which has about 10 bytes left in `$DF02-$DFF5`

## OPEN QUESTIONS

- none

## EDIT BOUNDARY (paths this work may touch)

- `menu/src/menu.s`
- `menu/menu.bin`
- `menu/menu.lbl`
- `menu/README.md`
- `crates/stidcart-cli/tests/menu_search.rs`
- `crates/stidcart-cli/tests/support/vice.rs`
- `Cargo.toml`
- `Cargo.lock`
- `gui/src-tauri/Cargo.toml`
- `gui/src-tauri/Cargo.lock`
- `docs/lcd/work/menu-search-sort/JOURNAL.md`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/SPEC.md`
- `.claude/rules/lcd-conventions.md`

<!-- /lcd-resume -->

---

## Acceptance criteria

All are proven by booting carts in VICE, as AC-87..AC-92 are. A test that types a query must not
assert a name equal to the query, because `vice::find` would match it on the query line. It reads
the list from row 2 down.

**AC-96** (surfaces: none): Given a cart carrying our menu with visible and hidden entries, when the
user types text, then only the visible entries whose name contains that text are listed, in the
order in use, matching letters regardless of case. The typed text is shown above the list, the first
match is selected, and the number of matches out of the visible entries is shown. A hidden entry
stays absent even when its name matches.

**AC-97** (surfaces: none): Given typed text, when the user presses DEL, then the last character is
removed and the list shows the matches for what is left. When the user presses CLR/HOME, the text is
cleared and every visible entry is listed again. DEL with no text changes nothing, and text stops
growing at 16 characters.

**AC-98** (surfaces: none): Given typed text that matches several entries, when the user moves the
selection and presses RETURN, then the selection moves through the matches only and the selected
match starts.

**AC-99** (surfaces: none): Given typed text that no entry matches, when the screen is shown, then
the menu says that nothing matches, which is different from the empty-cart message. RETURN does
nothing and doesn't hang, DEL brings matches back, and RUN/STOP leaves to BASIC.

**AC-100** (surfaces: none): Given a cart carrying our menu whose directory order isn't
alphabetical, when it boots, then the entries are in directory order. When the user presses F1, they
are listed A-Z by name, ignoring case, with names that are equal ignoring case kept in directory
order, and the first one selected. F1 again gives back directory order. The order in use is shown.

**AC-101** (surfaces: none): Given typed text, when the user presses F1, then the matches are listed
in the new order and the typed text is kept. Typing with A-Z in use lists the matches A-Z.

## LOG (append-only; not read on resume)

- 2026-11-07 — Triage: Standard (4 signals). User picked type-to-filter (substring) and an F1 toggle
  that starts in directory order → D-039. Advisor review: keep the base list separate from the view
  array, fold all three letter ranges, cover the fold with a mixed-case test name, read assertions
  from row 2 so the query line can't satisfy them, put a negative control in every test, and keep the
  status line off row 24. AC-96..AC-101 claimed, next free AC-102.
