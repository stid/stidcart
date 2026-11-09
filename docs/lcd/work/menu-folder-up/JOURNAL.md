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
- **Next action:** PR open; merge only on the user's yes
- **Branch:** feat/menu-folder-up · **Updated:** 2026-11-08

## STEPS

- [x] S1 — version 0.23.0 → 0.24.0 (first commit on the branch)
- [x] S2 — triage, D-058, AC-170..AC-172 claimed, this JOURNAL
- [x] S3 — failing boot tests in `menu_boot.rs` for AC-170..AC-172
- [x] S4 — implement in `menu/src/menu.s`, rebuild `menu.bin`/`menu.lbl`, the replaced build's fingerprint joins `EARLIER_STIDCART_BUILDS`
- [x] S5 — `menu/README.md`, the F7 help, gates with `STIDCART_REQUIRE_TOOLS=1`
- [x] S6 — audit, reconcile, closeout, PR (merge only on the user's yes)

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
- 2026-11-08 — S3: AC-170..AC-172 in `menu_boot.rs`, red. AC-171: row 5 is not `..`. AC-172:
  on `..` row 22 does not read `0 of 2` (UP from the first entry stays put). AC-170: UP then
  RETURN starts "demo a", so the menu never idles again (no `..` to go back through). Row 22 on
  `..` reads `0 of N`: the position leaves `..` out, so it is before the first entry.
- 2026-11-08 — S4: `..` is a folder row of group 0 at `shown[0]` inside a folder, so `nfold` is 1
  there and every folder-row path (start, save_state, restore's `_find`) already skips it; `nup`
  ($C141) takes it out of rows 22-23 and F5's range. "no matches" moves to the row under `..`.
  F7 help gains a `..` line. 7a9a0c1's build joins `EARLIER_STIDCART_BUILDS`. AC-167's HOME now
  lands on `..`, and its F5 step starts from the first entry so the pick stays determined. ROM free
  1674 → 1484 bytes. AC-170..AC-172 green; gate 310 passed, GUI gate green.
- 2026-11-08 — S5: README's key and stick tables, the folders section and the test list name
  `..`. Gates as S4 (no code change since).
- 2026-11-08 — Tests tightened: AC-171 titles k8 "!k8" ($21 sorts before "."), so a `..` sorted
  with the entries would land second; AC-172 presses F5 on `..` too (either entry, never `..`).
  Green; gate 310 passed. F5 on `..` with one entry selects it, which reads against AC-140's
  "one entry or none: nothing" if `..` counts as not an entry; left for the reconcile.
- 2026-11-08 — audit: PASS (test-presence). `AC-170 (none)`..`AC-172 (none)` are in
  `crates/stidcart-cli/tests/menu_boot.rs`, green at c624322 with `STIDCART_REQUIRE_TOOLS=1` (root
  310/0, cargo 1.97.1; `gui-gate.sh` green). Closeout line appended; the as-built details and the
  F5-from-`..` call are folded into D-058.
- 2026-11-08 — reconciled: folded AC-170..AC-172 → SPEC.md/none (one new row).
