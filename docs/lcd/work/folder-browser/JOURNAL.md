# JOURNAL — folder-browser

<!-- lcd-resume:v1 -->

## NOW

- **Lane:** Deep
- **Goal:** The cart owns an ordered folder list (empty folders kept and stored), with CLI folder flags, New Folder, clearer menus and an inspector on the right.
- **Next action:** test-gen, then red-green from T1
- **Branch:** feat/folder-browser (off feat/folder-editor; PR #58 awaits the user's merge yes) · **Updated:** 2026-11-08

## STEPS

- [x] S1 — triage, spec, plan, tasks
- [ ] S2 — failing tests ← next
- [ ] S3 — T1–T2 version, format doc
- [ ] S4 — T3–T8 core folder list
- [ ] S5 — T9–T11 CLI, VICE boot
- [ ] S6 — T12–T14 window Rust side
- [ ] S7 — T15–T25 window frontend
- [ ] S8 — T26, live check, audit, gates, close, PR

## DECISIONS (this work-item)

- Cart-owned ordered folder list, empty folders stored, CLI folder flags, New Folder, inspector right → D-063

## OPEN QUESTIONS

- none

## EDIT BOUNDARY (paths this work may touch)

- `Cargo.toml`
- `Cargo.lock`
- `gui/src-tauri/Cargo.toml`
- `gui/src-tauri/Cargo.lock`
- `docs/format/easyflash-cart-format.md`
- `crates/stidcart-core/src/`
- `crates/stidcart-core/tests/`
- `crates/stidcart-cli/src/lib.rs`
- `crates/stidcart-cli/tests/`
- `gui/src-tauri/src/`
- `gui/src-tauri/tests/`
- `gui/src/lib/`
- `gui/src/App.svelte`
- `README.md`
- `docs/lcd/`
- `.claude/rules/lcd-conventions.md`

<!-- /lcd-resume -->

---

## Acceptance criteria

See `spec.md` (AC-185..AC-196).

## LOG (append-only; not read on resume)

- 2026-11-08 — Triage Deep (6 signals; hard: format change, two surfaces). The user approved the plan
  ("implement all of it"); design boards "Folder browser" on the canvas. Spec, plan, tasks written;
  D-063. PR #58 green (review: one misplaced doc comment, fixed here); `gh pr merge 58` was denied
  by the auto-mode classifier, so the merge waits for the user's explicit yes.
- 2026-11-08 — refine: T23 dropped. File ▸ New Folder is routed in App.svelte to the listing's exported `newFolder()` (the listing owns the tree), so `viewer.svelte.ts` needs no change and its test none. Default folder name "new folder N" (names hold 16 characters; "untitled folder 2" is 17).
- 2026-11-08 — Core, CLI, VICE boot (AC-191), window Rust side and frontend green; both gates green.

## DEEP PIPELINE (Deep lane only)

`spec.md ✅  plan.md ✅  tasks.md ✅  tests ✅  red-green ✅  audit.md ⬜`
