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
- **Next action:** redgreen block F: T34 `gui/src/lib/rowDrag.ts` (rowDrag.test.ts); write `.ts` through the shell (auto-format hook)
- **Branch:** feat/folder-editor · **Updated:** 2026-11-08

## STEPS

- [x] S1 — triage (Deep), spec.md, plan.md, tasks.md
- [x] S2 — T1 version 0.26.0
- [x] S3 — T2–T19 rename group → folder (core, CLI, GUI; both gates green before commit)
- [x] S4 — T20–T23 `edit --move` and `move_entry`
- [x] S5 — T24–T29 `Edit::Arrange` in the Document
- [x] S6 — T30–T32 tree model (`tree.ts`)
- [ ] S7 — T33–T34 pointer drag (`rowDrag.ts`) ← next
- [ ] S8 — T35–T38 EntryMenu moves, FolderMenu
- [ ] S9 — T39–T41 viewer reports success
- [ ] S10 — T42–T44 tree in the listing
- [ ] S11 — T45–T46 README, conventions stem list
- [ ] S12 — T-live, T-audit, T-final, T-close

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
- `crates/stidcart-core/src/cartridge.rs`
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
- 2026-11-08 — tasks.md written: 46 numbered tasks + T-live, T-audit, T-final, T-close.
- 2026-11-08 — test-gen: 30 failing tests (5 in folder_cli.rs, 5 in GUI folders.rs, 20 in vitest across tree, rowDrag, FolderMenu, EntryMenu, MetaPanel, Listing), every AC × surface token present. Existing tests still pass; the AC-158 Listing tests that name "group" change in T12/T44. Test-defined contracts: EntryMenu `onmove` ({kind: folder|noFolder|newFolder|up|down}, props `folders`, `canUp`, `canDown`); FolderMenu props `name`, `canUp`, `canDown`, `onrename(name)`, `onmove(name, ±1)`, trigger "Actions for folder <name>"; rowDrag `dropTarget(kind, over, rect, y)` → {before|after|into|end|last}, `dragRow(source, ondrop)` attachment; rows carry `data-entry` (+ `data-folder`), folder rows `data-folder-row`; tree.ts exports buildTree, moveEntry, moveFolder, newFolder, renameFolder, stepEntry, stepFolder, sameTree, toArrange, indexAfter.
- 2026-11-08 — refine: EDIT BOUNDARY gains `crates/stidcart-core/src/cartridge.rs` (one doc comment names the metadata fields; the plan's rename missed it).
- 2026-11-08 — Block B green as one atomic commit (the rename can't compile one workspace at a time): AC-173..AC-175 (CLI), AC-175 (RENDER), and AC-184's Rust test. The global auto-format hook ran default Prettier on the new `.ts` tests (double quotes, 80 cols) because the repo has no Prettier config; the test files were restored to the repo style (single quotes, width 120), and `.ts` edits now go through the shell, which the hook doesn't watch.

## DEEP PIPELINE (Deep lane only)

> Phase tracker for the Deep-lane pipeline. Files live beside this JOURNAL.

`spec.md ✅  plan.md ✅  tasks.md ✅  tests ✅  red-green ⬜  audit.md ⬜`
<!-- mark ✅ done · ⏳ in progress · ⬜ not started -->
