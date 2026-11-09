# Tasks: Stable folders, a New Folder action and an inspector on the right

> **Deep lane · Phase 3 of 6 (Tasks).** Tests before the code they test.

**Slug:** `folder-browser`
**Spec:** [`spec.md`](./spec.md) • **Plan:** [`plan.md`](./plan.md) • **Journal:** [`JOURNAL.md`](./JOURNAL.md)
**Created:** 2026-11-08
**Status:** in_progress

---

## Conventions

- `[P]` = parallelizable with the previous task.

## Tasks

### A. Version and format (first code commit)

- [ ] **T1**: 0.26.0 → 0.27.0 in `Cargo.toml`, `Cargo.lock`, `gui/src-tauri/Cargo.toml`, `gui/src-tauri/Cargo.lock`
- [ ] **T2**: The folder-table paragraph of `docs/format/easyflash-cart-format.md`: the cart's folders in its order, empty ones allowed, written when a folder exists

### B. Core folder list (AC-188, AC-190)

- [ ] **T3**: Failing tests `AC-190` in `crates/stidcart-core/tests/folders.rs`: table order is the cart's folder order, an empty folder round-trips, a cart whose only metadata is an empty folder stores it, every frozen fixture loads with no folders and saves as before
- [ ] **T4**: `encode(folders, records)`, `Decoded::Records(folders, records)`, `FolderError` in `crates/stidcart-core/src/meta.rs`
- [ ] **T5**: `folders` on `CartBuilder`, `add_folder`/`remove_folder`/`move_folder`/`set_folders`, `set_meta` appends an unknown folder, META written when a folder exists, in `crates/stidcart-core/src/lib.rs`
- [ ] **T6**: Carry the folder table in `crates/stidcart-core/src/read.rs` [P]
- [ ] **T7**: `Listing.folders` in `crates/stidcart-core/src/listing.rs` [P]
- [ ] **T8**: Follow the shapes in `crates/stidcart-core/tests/meta.rs` if needed

### C. CLI (AC-185..AC-189)

- [ ] **T9**: Failing tests `AC-185 (CLI)`..`AC-189 (CLI)` in `crates/stidcart-cli/tests/folder_browser_cli.rs`
- [ ] **T10**: `--new-folder`, `--remove-folder`, `--move-folder`, usage, `folders:` line in `crates/stidcart-cli/src/lib.rs`

### D. C64 menu evidence (AC-191)

- [ ] **T11**: VICE boot test `AC-191` in `crates/stidcart-cli/tests/menu_boot.rs` (no menu change expected)

### E. Window, Rust side (AC-189, AC-196)

- [ ] **T12**: Failing tests in `gui/src-tauri/tests/folder_browser.rs`: `AC-196 (RENDER)` parity with the new flags, `AC-189 (RENDER)` the DTO carries the folders, empty folders through arrange
- [ ] **T13**: arrange sets the folder list in `gui/src-tauri/src/document.rs`; `ListingDto.folders` in `gui/src-tauri/src/lib.rs`; File ▸ New Folder in `gui/src-tauri/src/menu.rs`
- [ ] **T14**: Follow folder stability in `gui/src-tauri/tests/folders.rs` and the DTO in `gui/src-tauri/tests/contract.rs` with `gui/src/lib/fixtures/two.json`, `gui/src/lib/fixtures/document.json`

### F. Window, frontend (AC-189, AC-192..AC-195)

- [ ] **T15**: `Listing.folders`, `MenuId` `new-folder` in `gui/src/lib/types.ts`
- [ ] **T16**: Failing tests in `gui/src/lib/tree.test.ts`: folders from the cart, a folder stays when emptied, add/delete/untitled name
- [ ] **T17**: `gui/src/lib/tree.ts`
- [ ] **T18**: Failing test in `gui/src/lib/rowDrag.test.ts`: the empty-folder row takes a drop into its folder; then `gui/src/lib/rowDrag.ts`
- [ ] **T19**: Failing tests `AC-193 (RENDER)` in `gui/src/lib/FolderMenu.test.ts`; then Delete Folder in `gui/src/lib/FolderMenu.svelte`
- [ ] **T20**: Failing tests `AC-194 (RENDER)` in `gui/src/lib/EntryMenu.test.ts`; then Move to in `gui/src/lib/EntryMenu.svelte`
- [ ] **T21**: Failing tests `AC-195 (RENDER)` in `gui/src/lib/MetaPanel.test.ts`; then the inspector in `gui/src/lib/MetaPanel.svelte`
- [ ] **T22**: Failing test `AC-192 (RENDER)` in `gui/src/lib/Toolbar.test.ts`; then the button in `gui/src/lib/Toolbar.svelte`
- [ ] **T23**: Failing test in `gui/src/lib/viewer.test.ts`: the `new-folder` menu id reaches the listing; then `gui/src/lib/viewer.svelte.ts`
- [ ] **T24**: Failing tests in `gui/src/lib/Listing.test.ts`: `AC-189 (RENDER)` empty cards, `AC-192 (RENDER)` New Folder opens its name, empty-space menu, `AC-193 (RENDER)` delete and empty drop, `AC-195 (RENDER)` inspector beside the list
- [ ] **T25**: `gui/src/lib/Listing.svelte`, wired in `gui/src/App.svelte`

### G. Docs and close

- [ ] **T26**: `README.md`
- [ ] **T-live**: Live check in the app: New Folder three ways, empty folder drop, Delete Folder, Move to, inspector
- [ ] **T-audit**: `/lcd:audit folder-browser`
- [ ] **T-final**: both gates
- [ ] **T-close**: reconcile into `docs/lcd/SPEC.md`, `docs/lcd/MAP.md`; JOURNAL done; PR

## Linkbacks

- `AC-185 (CLI)`, `AC-186 (CLI)`, `AC-187 (CLI)`, `AC-188 (CLI)` → T5, T9, T10
- `AC-189 (CLI)` → T9, T10 · `AC-189 (RENDER)` → T7, T12, T13, T24, T25
- `AC-190` → T2, T3, T4, T5, T6 · `AC-191` → T11
- `AC-192 (RENDER)` → T13, T22, T23, T24, T25 · `AC-193 (RENDER)` → T17, T18, T19, T24, T25
- `AC-194 (RENDER)` → T20 · `AC-195 (RENDER)` → T21, T25 · `AC-196 (RENDER)` → T12, T13
