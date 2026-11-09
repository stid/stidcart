# JOURNAL — folder-editor

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the _entire_
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->

## NOW

- **Lane:** Deep
- **Goal:** Folders replace groups: rename Group → Folder (window, CLI `--folder`, import key), add entry re-order (`edit --move`, window), and a folder tree in the window with drag and drop.
- **Next action:** run /lcd:tasks folder-editor
- **Branch:** feat/folder-editor · **Updated:** 2026-11-08

## STEPS

- [x] S1 — triage (Deep) and spec.md
- [x] S2 — plan.md
- [ ] S3 — tasks.md ← next
- [ ] S4 — failing tests (AC-173..AC-184)
- [ ] S5 — red-green
- [ ] S6 — live check of the window, audit, PR

## DECISIONS (this work-item)

- Folders replace groups, no label field; tree order on tree operations; new folder made with an entry; tree whenever a folder exists; `edit --move` once per call → D-060
- One `Arrange` window edit flattened in Rust via `move_entry`; hand-rolled pointer drag, `dragDropEnabled` stays on → D-061

## OPEN QUESTIONS

- none

## EDIT BOUNDARY (paths this work may touch)
- `Cargo.toml`
- `Cargo.lock`
- `gui/src-tauri/Cargo.toml`
- `gui/src-tauri/Cargo.lock`
- `crates/stidcart-core/src/meta.rs`
- `crates/stidcart-core/src/lib.rs`
- `crates/stidcart-core/src/listing.rs`
- `crates/stidcart-core/tests/meta.rs`
- `crates/stidcart-core/tests/listing.rs`
- `crates/stidcart-cli/src/lib.rs`
- `crates/stidcart-cli/tests/meta_cli.rs`
- `crates/stidcart-cli/tests/folder_cli.rs`
- `gui/src-tauri/src/document.rs`
- `gui/src-tauri/src/lib.rs`
- `gui/src-tauri/tests/meta.rs`
- `gui/src-tauri/tests/contract.rs`
- `gui/src-tauri/tests/folders.rs`
- `gui/src/lib/types.ts`
- `gui/src/lib/fixtures/two.json`
- `gui/src/lib/fixtures/document.json`
- `gui/src/lib/tree.ts`
- `gui/src/lib/tree.test.ts`
- `gui/src/lib/rowDrag.ts`
- `gui/src/lib/rowDrag.test.ts`
- `gui/src/lib/FolderMenu.svelte`
- `gui/src/lib/FolderMenu.test.ts`
- `gui/src/lib/Listing.svelte`
- `gui/src/lib/Listing.test.ts`
- `gui/src/lib/EntryMenu.svelte`
- `gui/src/lib/EntryMenu.test.ts`
- `gui/src/lib/MetaPanel.svelte`
- `gui/src/lib/MetaPanel.test.ts`
- `gui/src/lib/viewer.svelte.ts`
- `gui/src/lib/viewer.test.ts`
- `gui/src/App.svelte`
- `README.md`
- `docs/lcd/work/folder-editor/`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/SPEC.md`
- `docs/lcd/MAP.md`
- `.claude/rules/lcd-conventions.md`
<!-- /lcd-resume -->

---

## Acceptance criteria

See `spec.md` (AC-173..AC-184).

## LOG (append-only; not read on resume)

- 2026-11-08 — Triage: Deep (5 signals, hard trigger: CLI and window surfaces). Recon on
  Tauri/Svelte drag and drop started in a subagent.
- 2026-11-08 — Spec written; four branch points answered by the user (→ D-060).
- 2026-11-08 — Recon (Tauri 2.12.3 / wry 0.57.0 source, tauri#14373): with `dragDropEnabled`
  on (needed for Finder drops' paths), HTML5 dragover/drop never reach the page on macOS, so
  HTML5-DnD libraries are out and `dragDropEnabled: false` is out. Use pointer-event DnD
  (setPointerCapture + elementFromPoint, three-band hit test per row: above / into folder /
  below); fallback library @dnd-kit/svelte 0.5.3 (pre-1.0, drop-onto-row unverified).
  svelte-dnd-action has no drop-onto-item. Rows need `-webkit-user-drag: none; user-select:
none` and preventDefault on dragstart, or WKWebView starts a native drag Tauri swallows.
  WCAG 2.2 SC 2.5.7 → the menu move commands (AC-182) are the non-drag path; wire them through
  the native menu path too.
- 2026-11-08 — Plan written (→ D-061). The user chose hand-rolled pointer drag over
  @dnd-kit/svelte. No native-menu twin of the move commands: the dropdown is keyboard reachable,
  so the recon's native-menu suggestion is dropped as extra scope.

## DEEP PIPELINE (Deep lane only)

> Phase tracker for the Deep-lane pipeline. Files live beside this JOURNAL.

`spec.md ✅  plan.md ✅  tasks.md ⬜  tests ⬜  red-green ⬜  audit.md ⬜`
<!-- mark ✅ done · ⏳ in progress · ⬜ not started -->
