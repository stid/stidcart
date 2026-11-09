# Plan: Folders replace groups, with a folder tree in the window

> **Deep lane · Phase 2 of 6 (Plan).** This document owns **how**. Specify ≠ Plan: if you're
> about to write a user story or acceptance criterion, that belongs in `spec.md`.

**Slug:** `folder-editor`
**Spec:** [`spec.md`](./spec.md) · **Journal:** [`JOURNAL.md`](./JOURNAL.md)
**Created:** 2026-11-08
**Status:** draft <!-- draft | approved | implemented | superseded -->

---

## Goal

Rename the metadata group to Folder on every surface, add `edit --move`, and give the window a
folder tree whose drags and menu commands become a single edit that writes the cart in tree order.

## Architecture

**The rename stays above the bytes.** `meta::Field::Group` becomes `Field::Folder`, named `folder`.
`Field::from_name` also accepts `group`, so `--group`, the import key `group` and any old caller
keep working through one alias. The listing row's `group` becomes `folder` in the core, the GUI DTO
and `types.ts`. The stored layout and the format doc keep "group table" (that's the byte layout,
D-018), and so do the menu source and its boot tests. They are untouched.

**One core primitive for order.** `CartBuilder::move_entry(from, to)` (remove + insert, metadata and
flags travel with the entry) is the only new core API. `edit --move FROM TO` is one `Op::Move`.
After parsing it is refused when given twice, or next to `--add`/`--add-aligned`/`--remove`. FROM
and TO are bounds-checked against the cart as given, along with every other index. It applies after
the in-place changes, which address the cart as given (AC-27 holds), and before the build. A cart
whose metadata is kept in an unknown layout is already refused on a re-order by the name-prefix
check (`BuildError::MetadataKept`), so `--move` adds no new refusal there.

**The window sends a tree, and Rust flattens it.** A tree operation becomes one new edit,
`Edit::Arrange { folders: [{ name, entries }], loose }`, where the indices are the listing's. The
Document validates it:

- every entry appears exactly once;
- every folder has a non-empty name;
- no two folders share a name ("a folder named 'X' already exists"). This is where AC-182 and
  AC-183 refuse a name another folder has.

It then sets each entry's folder through `set_meta(Folder, …)`, which gives the folder-name rules
and the kept-layout refusal in `edit`'s words. Finally it moves the entries into flattened order
with `move_entry`: each folder's entries in turn, then the loose ones. Because the edit can only
express a tree, tree order (AC-180) holds by construction. Because it is literally a series of
folder sets and moves, a cargo test can prove it equals `edit --folder …` plus `edit --move …`
(AC-180's parity). The frontend's job is the view and the gesture:

- `tree.ts` builds the tree from the listing rows: folders in the order of their first row, which
  is the order our menu numbers them (`meta::encode` numbers groups by first use, and the menu
  lists them in that order). Each folder's entries are in index order, then the loose entries.
- `tree.ts` also holds the pure tree operations (move an entry, move to a folder or a new one,
  move a folder, rename a folder, up/down), equality, and `toArrange(tree)`.
- An operation that leaves the tree as it was sends nothing (AC-179's no-op).

**Hand-rolled pointer drag (user, 2026-11-08; recon 2026-11-08).** Tauri's native drag-drop handler
stays on, because the Finder drop's paths come from it. With it on, WKWebView never delivers
HTML5 `dragover`/`drop` to the page (tauri-runtime-wry 2.12.1 `lib.rs:4786-4816`, wry 0.57.0
`wkwebview/drag_drop.rs:45-102`, tauri#14373). So `rowDrag.ts` is a Svelte attachment on
pointer events:

- `pointerdown` on a row, then a 4 px threshold before a drag starts (a click still selects, and a
  double-click still renames), then `setPointerCapture`.
- `document.elementFromPoint` finds the row under the pointer, and its `data-entry` /
  `data-folder` attributes say what the row is.
- `pointerup` turns the target into a tree operation.

Rows carry `-webkit-user-drag: none; user-select: none`, and `dragstart` is cancelled, so
WKWebView never starts a native drag that Tauri would swallow. The hit test is a pure function,
`dropTarget(kind, rowRect, y)`:

- dragging an entry over an entry row means before it or after it, by half;
- over a folder row it means into that folder;
- over the end strip it means out to the top level (the strip shows during a drag, after the
  last row, so a cart with no loose entry still has a top level to drop on).

Loose entries always sit after every folder, so there's no "between folders" target for an entry.
The recon's three bands collapse to two. Dragging a folder row over folder rows means before or
after it, by half. Over a folder's entry rows, it counts as being over that folder's row.

**Non-drag paths (AC-182, WCAG 2.2 SC 2.5.7).**

- `EntryMenu` gains a "Move to folder" submenu (bits-ui `Sub`) listing the existing folders, "No
  folder" and "New folder…", plus "Move up" and "Move down" within the entry's own folder or among
  the loose entries. Each is disabled at its edge.
- A new `FolderMenu` on each folder row offers Rename, Move up and Move down.
- "New folder…" and Rename open the existing `EditableText` inline: on a temporary row for a new
  folder (UI state only; the folder exists only once the edit carrying the entry succeeds), or on
  the folder's name.
- Folders open expanded. Collapse state is per-document view state in `Listing`, reset like the
  selection. The selection follows a moved entry: `Listing` maps the selected index through the
  arrangement once the edit succeeds, so `viewer.edit` reports success.

Rejected, one line each:

- A flat `order` permutation plus a folder per entry from the frontend: it can express a scattered
  result, so AC-180 would rest on frontend discipline.
- Semantic Rust edits (`MoveToFolder`, `MoveFolder`, …): they put the view's tree rules in Rust,
  four edits where one does.
- HTML5 drag and drop or `dragDropEnabled: false`: see the recon.
- @dnd-kit/svelte: pre-1.0, a new dependency, and the recon couldn't confirm dropping onto a row
  (user chose hand-rolled).

→ D-061.

## Constitution check

| Rule                                                                | Applies? | Compliance note                                                                                                                                                                                                                                                                                                                                                    |
| ------------------------------------------------------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `testing.md` (tests first, coverage)                                | yes      | Failing tests per (AC × surface) from `/lcd:test-gen`, then the red-green loop. Cargo targets need no registration; `*.test.ts` run under `vitest run` (no watch). Full gate (both workspaces) before the PR                                                                                                                                                       |
| `no-downgrade.md`                                                   | yes      | `--group` and the import key `group` keep working (alias). The Folder column, the entry panel, the flat list for carts without folders, and every existing EntryMenu action stay. Nothing is removed from `list` output except the label word, which the spec renames (AC-175, user-approved)                                                                      |
| `no-overengineering.md`                                             | yes      | One core primitive (`move_entry`), one new window edit (`Arrange`), no DnD dependency. No native-menu twin of the move commands (the dropdown is keyboard reachable already), no multi-select, no CLI folder commands (spec out of scope). The permutation check is the one guard against a malformed IPC payload, the only path where a bug could scramble a cart |
| `refinement-protocol.md`                                            | yes      | Spec ambiguities settled with the user before planning (five answers, D-060, D-061). A drift found in red-green goes through `lcd:refine`, and an intent change goes back to the user                                                                                                                                                                              |
| `versioning.md`                                                     | yes      | `feat/` → minor: 0.25.0 → 0.26.0 in the branch's first code commit (`Cargo.toml`, `gui/src-tauri/Cargo.toml`, both `Cargo.lock`), as in `c90fa31`. Base version verified on `main`: 0.25.0. The spec/plan docs commits precede it, so the red-green loop's first commit carries the bump                                                                           |
| `commits.md`                                                        | yes      | Conventional commits, one per red→green flip. The project CLAUDE.md overrides "push only when asked": commits are pushed, never to `main`                                                                                                                                                                                                                          |
| `ac-convention.md`, EVAL coverage                                   | no       | No scorer, ranker or LLM output. Results are byte-exact images and a deterministic view, checked against `edit` (parity)                                                                                                                                                                                                                                           |
| MAP invariant: byte parity against the frozen fixtures              | yes      | No fixture changes. A cart with groups load-saves byte-identically (AC-175). `--move` and `Arrange` are new requests, so no existing request's bytes move. The version bump changes the default header name as every bump has; the tests compute it, no fixture pins it                                                                                            |
| MAP invariant: the oracle decays                                    | yes      | `ndeft` has no re-order and no metadata. Evidence is `edit` as the oracle for the window (D-035) and round-trip tests. Our menu is unchanged, so no new VICE boot; tree order equals the menu's folder order through the existing first-use numbering                                                                                                              |
| MAP invariant: format facts come from `docs/format/`                | yes      | No format fact changes. The doc keeps "group table" for the bytes. No code offset changes                                                                                                                                                                                                                                                                          |
| MAP invariant: no global state or I/O policy in the core            | yes      | `move_entry` is a pure builder mutator. The tree, the drag and the refusal for a name another folder has live in the GUI crate and frontend                                                                                                                                                                                                                        |
| MAP invariant: zlib/GPL boundary at `firmware/`                     | no       | Nothing in `firmware/` changes                                                                                                                                                                                                                                                                                                                                     |
| MAP invariant: crate version reaches produced bytes                 | yes      | The 0.26.0 bump flows into the default header name. The tests already derive it from `CARGO_PKG_VERSION`                                                                                                                                                                                                                                                           |
| MAP invariant: describing vs rebuilding (D-011)                     | yes      | `--move` and `Arrange` are rebuild-path edits through `read::load`. The listing (describe) gains no refusal                                                                                                                                                                                                                                                        |
| MAP gui zone: "a second copy of a value `listing` already computes" | yes      | The listing doesn't compute folder order. The tree derives it from row order, which is the same first-use rule `meta::encode` applies. No listing value is recomputed                                                                                                                                                                                              |
| `.claude/rules/lcd-conventions.md` (AC numbering, test keys, gate)  | yes      | ACs 173–184, range claimed. Root tests by stem (`folder_cli`), GUI cargo tests under `gui/src-tauri/tests/`, frontend `*.test.ts`. The gate is the machine block's `gate` (root + `gui-gate.sh`)                                                                                                                                                                   |
| Project CLAUDE.md: never commit to `main`, decision log             | yes      | Branch `feat/folder-editor`, PR with `gate`/`gui`/`claude-review`. D-060 and D-061 logged when decided                                                                                                                                                                                                                                                             |

## Cross-path behavior matrix

| AC       | Surface | Path                                        |
| -------- | ------- | ------------------------------------------- |
| `AC-173` | CLI     | `crates/stidcart-cli/src/lib.rs:"--folder"` |
| `AC-174` | CLI     | `crates/stidcart-cli/src/lib.rs:fn import`  |
| `AC-175` | CLI     | `crates/stidcart-cli/src/lib.rs:r.folder`   |
| `AC-175` | RENDER  | `gui/src/lib/MetaPanel.svelte`              |
| `AC-176` | CLI     | `crates/stidcart-cli/src/lib.rs:"--move"`   |
| `AC-177` | CLI     | `crates/stidcart-cli/src/lib.rs:Op::Move`   |
| `AC-178` | RENDER  | `gui/src/lib/Listing.svelte`                |
| `AC-179` | RENDER  | `gui/src/lib/rowDrag.ts`                    |
| `AC-180` | RENDER  | `gui/src/lib/tree.ts`                       |
| `AC-181` | RENDER  | `gui/src/lib/rowDrag.ts`                    |
| `AC-182` | RENDER  | `gui/src/lib/EntryMenu.svelte`              |
| `AC-183` | RENDER  | `gui/src/lib/FolderMenu.svelte`             |
| `AC-184` | RENDER  | `gui/src/lib/MetaPanel.svelte`              |

**Test scope:** `crates/stidcart-cli/tests/folder_cli.rs`, `gui/src-tauri/tests/folders.rs`, `gui/src/lib/tree.test.ts`, `gui/src/lib/rowDrag.test.ts`, `gui/src/lib/Listing.test.ts`, `gui/src/lib/EntryMenu.test.ts`, `gui/src/lib/FolderMenu.test.ts`, `gui/src/lib/MetaPanel.test.ts`

The behaviour that must match across paths:

- `--folder` and `--group` write the same bytes, and so do the import keys `folder` and `group`.
- `Arrange` saves the bytes of the equivalent `edit --folder`/`--move` sequence (`folders.rs`
  against `stidcart_cli::run`).
- `Arrange` refusals (an invalid folder name, the kept layout) carry `edit`'s text.
- The CLI's `folder:` label and the window's "Folder" name the same field.

## Reused primitives

| Existing primitive                                                   | Path                                                   | Used for                                                                                         |
| -------------------------------------------------------------------- | ------------------------------------------------------ | ------------------------------------------------------------------------------------------------ |
| `meta::Field`, `Field::from_name`, `meta::check`, `Meta::set`        | `crates/stidcart-core/src/meta.rs`                     | Rename and the `group` alias, folder-name rules and their refusal text                           |
| `CartBuilder::set_meta` (with the kept-layout refusal)               | `crates/stidcart-core/src/lib.rs`                      | Setting each entry's folder in `Arrange` and `--folder`                                          |
| `CartBuilder::meta_entry` / `meta::encode` first-use group numbering | `crates/stidcart-core/src/lib.rs`, `meta.rs`           | Folder order on the C64 follows directory order, so tree order is menu order with no menu change |
| `BuildError::MetadataKept` name-prefix check                         | `crates/stidcart-core/src/lib.rs`                      | Refusing a re-order of a cart whose metadata is kept verbatim                                    |
| CLI `Op` resolve/validate/apply pipeline, `index()`                  | `crates/stidcart-cli/src/lib.rs`                       | `Op::Move` parsing, bounds check against the cart as given                                       |
| `listing::describe` / `Row`                                          | `crates/stidcart-core/src/listing.rs`                  | The rows the tree is built from (`folder` field)                                                 |
| `Document::apply`, `check_index`, undo history                       | `gui/src-tauri/src/document.rs`                        | `Edit::Arrange` as one undoable step, refusals leave the document unchanged                      |
| `stidcart_cli::run` as parity oracle (D-035)                         | `gui/src-tauri/tests/meta.rs`, `parity.rs` pattern     | `folders.rs` compares `Arrange` saves to `edit`                                                  |
| `EditableText`                                                       | `gui/src/lib/EditableText.svelte`                      | Inline folder rename and "New folder…" name entry                                                |
| `EntryMenu` (bits-ui DropdownMenu + ContextMenu from one item list)  | `gui/src/lib/EntryMenu.svelte`                         | Move to folder ▸ / Move up / Move down                                                           |
| `viewer.edit` / `attempt` error display                              | `gui/src/lib/viewer.svelte.ts`                         | Showing `Arrange` refusals the way other edits show                                              |
| Listing test harness and fixtures                                    | `gui/src/lib/Listing.test.ts`, `gui/src/lib/fixtures/` | Tree rendering and interaction tests                                                             |

## Data model / Contracts

- **Core:** `meta::Field::Folder` (was `Group`), `name() == "folder"`, and `from_name("group") ==
Some(Folder)`. `listing::Row.folder` (was `group`). `CartBuilder::move_entry(&mut self, from:
usize, to: usize)` panics out of range like the other index mutators.
- **CLI:** `edit --folder N TEXT` (with `--group N TEXT` as an alias) and `edit --move FROM TO`. In
  the import, a table whose keys resolve to the same field twice (`folder` and `group`) is refused:
  `'<file>': entry '<name>' sets the folder twice`. The unknown-key text lists "title, year,
  author, note and folder". `list` prints `folder: X`.
- **GUI IPC:** `Edit::Arrange { folders: Vec<{ name: String, entries: Vec<usize> }>, loose:
Vec<usize> }`, serde tag `arrange`, camelCase. `RowDto.folder`. `types.ts`: `Row.folder`,
  `MetaField` `'folder'`, and the `Edit` union gains `{ kind: 'arrange'; folders: { name: string;
entries: number[] }[]; loose: number[] }`. The fixtures `two.json` and `document.json` are
  re-pinned by `tests/contract.rs`.
- **Frontend:**
  - Types: `Tree = { folders: { name: string; entries: number[] }[]; loose: number[] }`.
  - Operations (pure): `buildTree(rows)`, `moveEntry(tree, index, target)`,
    `moveFolder(tree, name, before|after, other)`, `renameFolder`, `step(tree, index|name, -1|+1)`,
    `sameTree`, `toArrange`.
  - The new index of entry `i` after an arrange is its position in the flattened tree.

## File structure

### New files

```
crates/stidcart-cli/tests/folder_cli.rs   AC-173..AC-177 (CLI) and AC-175's list label: --folder/--group, import keys, --move and its refusals
gui/src-tauri/tests/folders.rs           AC-179..AC-183 Rust side: Arrange parity with edit, tree order, undo, name refusals, permutation check
gui/src/lib/tree.ts                      Tree model built from listing rows, pure tree operations, toArrange
gui/src/lib/tree.test.ts                 AC-178 order, AC-180 flatten, the operations and no-op detection
gui/src/lib/rowDrag.ts                   Pointer-event drag attachment and the pure dropTarget hit test
gui/src/lib/rowDrag.test.ts              AC-179, AC-181: hit test, threshold, drops turned into tree operations
gui/src/lib/FolderMenu.svelte            Folder row actions: Rename, Move up, Move down
gui/src/lib/FolderMenu.test.ts           AC-183, AC-181's non-drag path
```

### Modified files

- `Cargo.toml`, `Cargo.lock`, `gui/src-tauri/Cargo.toml`, `gui/src-tauri/Cargo.lock`: version 0.26.0 (first code commit)
- `crates/stidcart-core/src/meta.rs`: `Field::Group` → `Folder`, name `folder`, `from_name` alias `group`, doc comments
- `crates/stidcart-core/src/lib.rs`: `CartBuilder::move_entry`, comment wording
- `crates/stidcart-core/src/listing.rs`: `Row.group` → `folder`
- `crates/stidcart-core/tests/meta.rs`, `crates/stidcart-core/tests/listing.rs`: follow the rename
- `crates/stidcart-cli/src/lib.rs`: `--folder` (+ `--group` alias), `Op::Move` parse/validate/apply, usage text, import double-key refusal and key list, `list` label `folder`
- `crates/stidcart-cli/tests/meta_cli.rs`: follow the label rename (`folder:` in `list` output)
- `gui/src-tauri/src/document.rs`: `Edit::Arrange` and its validation/apply, the field-list error text
- `gui/src-tauri/src/lib.rs`: `RowDto.folder`
- `gui/src-tauri/tests/meta.rs`, `gui/src-tauri/tests/contract.rs`: follow the rename and re-pin the DTO
- `gui/src/lib/types.ts`: `Row.folder`, `MetaField`, `Edit` `arrange`
- `gui/src/lib/fixtures/two.json`, `gui/src/lib/fixtures/document.json`: `group` → `folder`
- `gui/src/lib/Listing.svelte`: tree rendering (folder rows with count, collapse, `FolderMenu`, end strip during a drag), the "Folder" column header, `rowDrag` wiring, the temporary new-folder row, selection mapped through arrangements
- `gui/src/lib/Listing.test.ts`: AC-178 rendering/collapse, the AC-175 header, AC-179/AC-182 wiring
- `gui/src/lib/EntryMenu.svelte`: Move to folder submenu, Move up/down (needs the tree, or the folder list plus edge flags, as props)
- `gui/src/lib/EntryMenu.test.ts`: AC-182
- `gui/src/lib/MetaPanel.svelte`: the "Folder" label and `folder` field
- `gui/src/lib/MetaPanel.test.ts`: AC-175 label, AC-184 (field edit is `setMeta`, no `arrange`)
- `gui/src/lib/viewer.svelte.ts`: `edit` reports success, so the selection can follow a move
- `gui/src/lib/viewer.test.ts`: covers the success report
- `gui/src/App.svelte`: only if the success report changes the `onedit` wiring
- `README.md`: the group → folder wording, `--move`
- `docs/lcd/work/folder-editor/`: JOURNAL, tasks, audit
- `docs/lcd/DECISIONS.md`: D-061
- `docs/lcd/SPEC.md`, `docs/lcd/MAP.md`: closeout reconcile only
- `.claude/rules/lcd-conventions.md`: stem list gains `folder_cli` if the note is updated

## Risks & rejected alternatives

- **Pointer drag in jsdom**: jsdom has no layout and no `elementFromPoint`. The hit test is pure
  and tested with given rects. The attachment's tests stub `elementFromPoint` and the rects. The
  real gesture is proven by the live check in the running app (memory: GUI live check needs a
  CGEvent click tool, and drags may need the user).
- **bits-ui `DropdownMenu.Sub` inside `ContextMenu`**: `EntryMenu` renders one item list in both.
  If the Sub component doesn't share across them, "Move to folder" falls back to a flat group of
  items under a label in the context menu. That's no loss of function.
- **Renaming `Field::Group` internally**: churn across about eight files, but leaving "group" in
  code while every surface says folder would split the vocabulary. The format doc keeps "group"
  because it names bytes.
- **Moving a folder that holds hidden entries**: they move with it. Our menu doesn't list them,
  but the window does, and the tree shows them in place. Nothing special is needed.
- **Arrange on a cart whose metadata is kept in an unknown layout**: `set_meta` refuses with the
  kept reason before any move. That's `edit`'s refusal, and the cart is unchanged.
