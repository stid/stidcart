# JOURNAL — menu-folder-up

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the _entire_
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->

## NOW

- **Lane:** Standard
- **Goal:** Inside a folder our menu lists a `..` row first; RETURN or fire on it goes back to the top level, so the joystick can leave a folder (D-058)
- **Next action:** S3 — failing boot tests for AC-170..AC-172
- **Branch:** feat/menu-folder-up · **Updated:** 2026-11-08

## STEPS

- [x] S1 — version 0.23.0 → 0.24.0 (first commit on the branch)
- [x] S2 — triage, D-058, AC-170..AC-172 claimed, this JOURNAL
- [ ] S3 — failing boot tests in `menu_boot.rs` for AC-170..AC-172
- [ ] S4 — implement in `menu/src/menu.s`, rebuild `menu.bin`/`menu.lbl`, the replaced build's fingerprint joins `EARLIER_STIDCART_BUILDS`
- [ ] S5 — `menu/README.md`, the F7 help, gates with `STIDCART_REQUIRE_TOOLS=1`
- [ ] S6 — audit, reconcile, closeout, PR (merge only on the user's yes)

## DECISIONS (this work-item)

- `..` row first in a folder, always listed, not an entry, folder opens on the first entry → D-058

## OPEN QUESTIONS

- none

## EDIT BOUNDARY (paths this work may touch)

- `menu/src/menu.s`
- `menu/menu.bin`
- `menu/menu.lbl`
- `menu/README.md`
- `crates/stidcart-core/src/menu.rs`
- `crates/stidcart-core/tests/menu_builds.rs`
- `crates/stidcart-cli/tests/menu_boot.rs`
- `crates/stidcart-cli/tests/meta_boot.rs`
- `crates/stidcart-cli/tests/support/`
- `Cargo.toml`
- `Cargo.lock`
- `gui/src-tauri/Cargo.toml`
- `gui/src-tauri/Cargo.lock`
- `docs/lcd/SPEC.md`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/triage-log.md`
- `docs/lcd/work/menu-folder-up/JOURNAL.md`
- `.claude/rules/lcd-conventions.md`

<!-- /lcd-resume -->

---

## Acceptance criteria

**AC-170** (surfaces: none): Given a folder opened in our menu, then its first row is `..` with the
folder badge and the selection is on the first entry below it; RETURN, or fire on the joystick in
port 2, with `..` selected goes back to the top level with that folder selected; the details pane
on `..` says it goes back.

**AC-171** (surfaces: none): Given a folder with filter text that matches nothing, with A-Z on, and
with each kind filter, then `..` is still the first row and can still be chosen. HOME selects it;
CLR selects the last entry.

**AC-172** (surfaces: none): Given a folder, then the position and match counts on rows 22-23 leave
`..` out, F5 never selects it, and starting an entry and soft-resetting restores that entry, never
`..`. DEL with no text and RUN/STOP still go back up, as in AC-167.

## LOG (append-only; not read on resume)

- 2026-11-08 — Triage: Standard (2 signals: a menu change with a binary rebuild, may span a
  session). The user tried the folders on their own cart in VICE (`~/Desktop/stidcollection.crt`)
  and found no way back up with a joystick, then asked for a top item that goes up a level. ESC in
  VICE is RUN/STOP, which at the top level leaves to BASIC as designed. That stays, since the user
  asked only about going up. AC-170..AC-172 claimed, next free AC-173.
