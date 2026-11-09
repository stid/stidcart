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
- **Next action:** done; the PR is open and awaits CI (gate, gui, claude-review) and the user's merge
- **Branch:** feat/menu-search-sort · **Updated:** 2026-11-07

## STEPS

- [x] S1 — version 0.9.0 → 0.10.0 (first commit on the `feat/` branch)
- [x] S2 — triage, D-039, AC-96..AC-101 claimed, this JOURNAL
- [x] S3 — `vice::key` gets DEL/HOME/CLR/F1. Failing tests AC-96..AC-101 appended to `menu_boot.rs`, each with a negative control
- [x] S4 — menu refactor, no behaviour change: `visible` stays the base list, and a view array with its own count feeds draw/up/down/start. Query on row 1, status on row 23 (no RETURN on the last rows). AC-87..AC-92 stay green
- [x] S5 — filter: typed characters, DEL, CLR/HOME, cap 16, fold, "no matches" (AC-96..AC-99)
- [x] S6 — F1 sort, computed on first press and kept, stable (AC-100, AC-101)
- [x] S7 — `menu/README.md` key table. `menu.bin`/`menu.lbl` reassembled with every menu commit (AC-93)
- [x] S8 — full gate, audit, reconcile SPEC row "Booted, our menu shows…", PR

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
- `crates/stidcart-cli/tests/menu_boot.rs`
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
- 2026-11-07 — Refined: the tests went into `menu_boot.rs` instead of a new `menu_search.rs`, to
  reuse its build, boot and key helpers (about 60 lines that would otherwise be copied). Red run: 6
  failed for the expected reasons (keys ignored, no status line) and the 13 existing tests passed.
  AC-98's inputs were reordered so the unfiltered path would select a non-marker.
- 2026-11-07 — Green: the menu keeps `visible` as the base list, with a `shown` view, a `sorted`
  array and per-slot name address tables in `$C200-$C5FF`. All in ROM, and `launch.s` is unchanged.
  The script-driven edit first cut at `redraw:` instead of `draw:`, and `sort` is a 64tass built-in
  name. Both were caught by the assembler and fixed by restoring the file and re-patching. Root gate
  215/0 with STIDCART_REQUIRE_TOOLS=1, GUI gate 31/0.
- 2026-11-07 — Timing (scratch VICE run, stopwatch from `sort_az` to `refilter`, 200 entries in
  reverse order): plain insertion sort took 6.06M cycles, about 6 s at 1 MHz. Binary insertion
  takes 966k, about 1 s, once per boot. Note that `stidcart` embeds `menu.bin` at compile time, so
  a reassembled menu needs `cargo build` before a hand-built cart carries it.
- 2026-11-07 — audit: PASS (test-presence). All six ACs are surface `none`, so as in custom-menu's audit there is no token row. Their tests are `ac_96`..`ac_101` in `menu_boot.rs`, one each, green at ee3ec0f with STIDCART_REQUIRE_TOOLS=1 (root 215/0, GUI 31/0). Closeout line appended.
- 2026-11-07 — reconciled: folded AC-96..AC-101 → SPEC.md/Internal as 1 new row. The boot row (AC-87..AC-92) still holds, so nothing was superseded.
