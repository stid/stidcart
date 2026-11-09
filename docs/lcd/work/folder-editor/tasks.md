# Tasks: Folders replace groups, with a folder tree in the window

> **Deep lane · Phase 3 of 6 (Tasks).** A flat, dependency-ordered checklist. Each task names
> the file it touches. Tests come before the code they test (TDD). The granular STEPS in the
> work-item `JOURNAL.md` mirror this list as the resume view; keep them in sync.

**Slug:** `folder-editor`
**Spec:** [`spec.md`](./spec.md) • **Plan:** [`plan.md`](./plan.md) • **Journal:** [`JOURNAL.md`](./JOURNAL.md)
**Created:** 2026-11-08
**Status:** in_progress <!-- in_progress | completed | abandoned -->

---

## Conventions

- `[P]` suffix = parallelizable with the previous task (different file, no shared state).
- Tests precede implementation tasks they cover.
- One file per task where possible; tasks naming multiple files should be split unless the change is genuinely atomic.

## Tasks

### A. Version (first code commit of the branch)

- [ ] **T1**: Bump 0.25.0 → 0.26.0 in `Cargo.toml`, `Cargo.lock`, `gui/src-tauri/Cargo.toml`, `gui/src-tauri/Cargo.lock` (atomic: one version, as `c90fa31`)

### B. Rename group → folder (AC-173, AC-174, AC-175)

- [ ] **T2**: Write failing test `AC-173 (CLI)` in `crates/stidcart-cli/tests/folder_cli.rs`: `--folder N TEXT` equals `--group N TEXT` byte for byte, limits and clearing hold, usage names `--folder`
- [ ] **T3**: Write failing test `AC-174 (CLI)` in `crates/stidcart-cli/tests/folder_cli.rs`: import key `folder` equals `group`; a table setting both is refused naming the entry, file untouched
- [ ] **T4**: Write failing test `AC-175 (CLI)` in `crates/stidcart-cli/tests/folder_cli.rs`: `list` prints `folder: X`; a load and save of a grouped cart is byte-identical
- [ ] **T5**: Rename `Field::Group` → `Folder` (name `folder`, `from_name` alias `group`) in `crates/stidcart-core/src/meta.rs`
- [ ] **T6**: Rename `Row.group` → `folder` in `crates/stidcart-core/src/listing.rs` [P]
- [ ] **T7**: Follow the rename in `crates/stidcart-core/tests/meta.rs`
- [ ] **T8**: Follow the rename in `crates/stidcart-core/tests/listing.rs` [P]
- [ ] **T9**: `--folder` option (`--group` kept), import double-key refusal and key list, `list` label `folder`, usage text in `crates/stidcart-cli/src/lib.rs` (passes T2–T4)
- [ ] **T10**: Follow the label rename in `crates/stidcart-cli/tests/meta_cli.rs`
- [ ] **T11**: Write failing test `AC-175 (RENDER)` in `gui/src/lib/MetaPanel.test.ts`: the field is labelled "Folder" and edits field `folder`
- [ ] **T12**: Write failing test `AC-175 (RENDER)` in `gui/src/lib/Listing.test.ts`: the column is headed "Folder" [P]
- [ ] **T13**: `RowDto.group` → `folder` in `gui/src-tauri/src/lib.rs`
- [ ] **T14**: The field-list error text names `folder` in `gui/src-tauri/src/document.rs` [P]
- [ ] **T15**: Follow the rename in `gui/src-tauri/tests/meta.rs`
- [ ] **T16**: Re-pin `group` → `folder` in `gui/src/lib/fixtures/two.json` and `gui/src/lib/fixtures/document.json`, adjusting `gui/src-tauri/tests/contract.rs` only if it names the key (atomic: the contract test compares the DTO with these files)
- [ ] **T17**: `Row.folder` and `MetaField` `'folder'` in `gui/src/lib/types.ts`
- [ ] **T18**: The "Folder" label and `folder` field in `gui/src/lib/MetaPanel.svelte` (passes T11)
- [ ] **T19**: The "Folder" column header and `row.folder` in `gui/src/lib/Listing.svelte` (passes T12) [P]

### C. Re-order from the CLI (AC-176, AC-177)

- [ ] **T20**: Write failing test `AC-176 (CLI)` in `crates/stidcart-cli/tests/folder_cli.rs`: `--move FROM TO` places the entry at TO, the others keep relative order, name/flags/kind/data/metadata travel
- [ ] **T21**: Write failing test `AC-177 (CLI)` in `crates/stidcart-cli/tests/folder_cli.rs`: out of range, twice, or next to `--add`/`--add-aligned`/`--remove` is refused with files byte-identical; other operations in the call index the cart as given
- [ ] **T22**: `CartBuilder::move_entry(from, to)` in `crates/stidcart-core/src/lib.rs`
- [ ] **T23**: `Op::Move` parse, validate and apply in `crates/stidcart-cli/src/lib.rs` (passes T20, T21)

### D. The window's Arrange edit, Rust side (AC-179, AC-180, AC-182, AC-183, AC-184)

- [ ] **T24**: Write failing test `AC-180 (RENDER)` in `gui/src-tauri/tests/folders.rs`: `Arrange` saves the bytes of the equivalent `edit --folder`/`--move` sequence, and the directory is in tree order (folders' entries together, folders in order, then loose)
- [ ] **T25**: Write failing test `AC-179 (RENDER)` in `gui/src-tauri/tests/folders.rs`: an `Arrange` is one undoable step; undo restores the bytes
- [ ] **T26**: Write failing test `AC-183 (RENDER)` in `gui/src-tauri/tests/folders.rs`: an empty, invalid (`edit`'s folder text) or duplicate folder name is refused and the document is unchanged; an arrangement missing or repeating an entry is refused
- [ ] **T27**: Write failing test `AC-182 (RENDER)` in `gui/src-tauri/tests/folders.rs`: a new folder named like an existing one is refused (same refusal path, from the entry-move shape)
- [ ] **T28**: Write failing test `AC-184 (RENDER)` in `gui/src-tauri/tests/folders.rs`: `setMeta` on `folder` moves no entry in the directory
- [ ] **T29**: `Edit::Arrange` validation and apply via `set_meta` + `move_entry` in `gui/src-tauri/src/document.rs` (passes T24–T28)

### E. Tree model (AC-178, AC-179, AC-180, AC-181, AC-182, AC-183)

- [ ] **T30**: Write failing tests in `gui/src/lib/tree.test.ts`: `AC-178 (RENDER)` folder order by first row and entries in index order then loose; `AC-180 (RENDER)` `toArrange` flattens to tree order; `AC-179 (RENDER)` moving into/out of a folder and between rows, an unchanged tree reports no change; `AC-181 (RENDER)` moving a folder carries its entries; `AC-182 (RENDER)` move to a new folder, up/down at edges; `AC-183 (RENDER)` rename keeps position
- [ ] **T31**: `Edit` gains `arrange` in `gui/src/lib/types.ts`
- [ ] **T32**: `buildTree`, the tree operations, `sameTree`, `toArrange` and the post-arrange index map in `gui/src/lib/tree.ts` (passes T30)

### F. Pointer drag (AC-179, AC-181)

- [ ] **T33**: Write failing tests in `gui/src/lib/rowDrag.test.ts`: `AC-179 (RENDER)` the hit test (before/after an entry by half, into a folder row, the end strip to top level), the 4 px threshold leaves a click a click, a drop becomes the tree operation; `AC-181 (RENDER)` a folder over folder rows is before/after, never into, and an entry can't land deeper than one folder
- [ ] **T34**: The pointer-drag attachment and pure `dropTarget` in `gui/src/lib/rowDrag.ts` (passes T33)

### G. Menus (AC-181, AC-182, AC-183)

- [ ] **T35**: Write failing test `AC-182 (RENDER)` in `gui/src/lib/EntryMenu.test.ts`: Move to folder lists existing folders, "No folder" only when in one, "New folder…"; Move up/down disabled at the edges; each sends one edit
- [ ] **T36**: Move to folder submenu and Move up/down in `gui/src/lib/EntryMenu.svelte` (passes T35)
- [ ] **T37**: Write failing tests in `gui/src/lib/FolderMenu.test.ts`: `AC-183 (RENDER)` Rename starts inline renaming; `AC-181 (RENDER)` Move up/down among folders, disabled at the edges
- [ ] **T38**: `gui/src/lib/FolderMenu.svelte` (passes T37)

### H. Selection follows a move

- [ ] **T39**: Write failing test in `gui/src/lib/viewer.test.ts`: `edit` reports whether the edit was applied
- [ ] **T40**: `edit` returns success in `gui/src/lib/viewer.svelte.ts` (passes T39)
- [ ] **T41**: Pass the success through `onedit` in `gui/src/App.svelte`, only if T40 changes its wiring

### I. The tree in the listing (AC-178, AC-179, AC-182, AC-184)

- [ ] **T42**: Write failing tests in `gui/src/lib/Listing.test.ts`: `AC-178 (RENDER)` folder rows with name and count then loose entries, collapse/expand, open expanded, flat list without folders; `AC-179 (RENDER)` a drop sends one `arrange` and an unchanged drop sends nothing; `AC-182 (RENDER)` "New folder…" opens an inline name row and its commit sends `arrange`; `AC-184 (RENDER)` after a folder field change the entry shows under its new folder; the selection follows a moved entry
- [ ] **T43**: Write failing test `AC-184 (RENDER)` in `gui/src/lib/MetaPanel.test.ts`: a Folder field change sends `setMeta`, never `arrange` [P]
- [ ] **T44**: Tree rendering, folder rows with `FolderMenu`, collapse state, end strip, `rowDrag` wiring, new-folder row, selection mapping in `gui/src/lib/Listing.svelte` (passes T42, T43)

### J. Docs

- [ ] **T45**: Group → folder wording and `edit --move` in `README.md`
- [ ] **T46**: Add `folder_cli` to the root stem list in `.claude/rules/lcd-conventions.md` [P]

### K. Verify and close

- [ ] **T-live**: Live check in the running app (memory `gui-live-check`): open a cart, see the tree, drag an entry into and out of a folder, move a folder, rename one, check a Finder drop still adds; save and `list` the result. Drags may need the user's hand
- [ ] **T-audit**: Run `/lcd:audit folder-editor`; resulting `audit.md` has zero MISSING/BLOCKED rows
- [ ] **T-final**: `cargo fmt --all --check && cargo clippy --workspace --all-targets --all-features -- -D warnings && cargo test --workspace && docs/lcd/ci/gui-gate.sh`, all green
- [ ] **T-close**: Closeout: `lcd:reconcile` folds AC-173..AC-184 into `docs/lcd/SPEC.md`; `docs/lcd/MAP.md` gets the core's `move_entry` and the GUI's tree/drag modules if its zone text needs them; JOURNAL marked done

## Checkpoint validation

After every block of related tasks, the suite must stay green. If T3 passes but T5's implementation breaks T1, stop and fix before moving on.

Block B is a rename that crosses both workspaces: the root gate goes green at T10 and the GUI gate at T19. Don't commit a block-B flip with the other workspace red.

## Linkbacks

- `AC-173 (CLI)` → T2, T5, T9
- `AC-174 (CLI)` → T3, T5, T9
- `AC-175 (CLI)` → T4, T5, T6, T9, T10
- `AC-175 (RENDER)` → T11, T12, T13, T16, T17, T18, T19
- `AC-176 (CLI)` → T20, T22, T23
- `AC-177 (CLI)` → T21, T23
- `AC-178 (RENDER)` → T30, T32, T42, T44
- `AC-179 (RENDER)` → T25, T29, T30, T32, T33, T34, T42, T44
- `AC-180 (RENDER)` → T24, T29, T30, T31, T32
- `AC-181 (RENDER)` → T30, T32, T33, T34, T37, T38
- `AC-182 (RENDER)` → T27, T29, T30, T35, T36, T42, T44
- `AC-183 (RENDER)` → T26, T29, T30, T37, T38
- `AC-184 (RENDER)` → T28, T29, T42, T43, T44
- Rename support (no new AC) → T7, T8, T14, T15
- Selection follows a move (plan, AC-179's edit) → T39, T40, T41, T44
- Version → T1 · Docs → T45, T46 · Closeout → T-live, T-audit, T-final, T-close

---

<!--
Quality checks before starting implementation:
  - Every AC × non-`none`-surface row in plan.md has at least one test task above
  - Every new file in plan.md "File structure" has at least one task that creates it
  - Tests appear before their implementation
  - T-audit appears before T-final (the audit gate must pass before suite-final)
  - T-final exists (run the suite, lint, type check)
-->
