# Plan: Stable folders, a New Folder action and an inspector on the right

> **Deep lane · Phase 2 of 6 (Plan).** This document owns **how**.

**Slug:** `folder-browser`
**Spec:** [`spec.md`](./spec.md) · **Journal:** [`JOURNAL.md`](./JOURNAL.md)
**Created:** 2026-11-08
**Status:** approved <!-- the user asked for all of it, 2026-11-08 -->

---

## Goal

Give the cart its own ordered folder list, stored even when a folder is empty, edited from the CLI
and from a window that has New Folder, clearer menus and an inspector on the right.

## Architecture

**The cart owns its folders.** `CartBuilder` gains `folders: Vec<String>`, the cart's folders in
order. Every entry's folder is one of them: `set_meta(Folder, name)` appends a folder the cart
doesn't have, so `--folder` and the import keep creating folders on demand (AC-188). Removing an
entry or clearing its folder leaves the list alone, so a folder stays when it empties.

The new operations live on the builder:

- `add_folder` appends, refusing an invalid name or one the cart already has.
- `remove_folder` clears the folder on its entries and drops it.
- `move_folder(name, to)` re-orders.
- `set_folders`, for the window, replaces the order in one go.

Each refuses on a cart whose metadata is kept in an unknown layout, as `set_meta` does.

**Format: the folder table is the cart's folder list.** The format doc changes first (D-018):

- the table holds the cart's folders in its order, empty ones included;
- a record's group number is a position in that list;
- the metadata entry is written when an entry has metadata or the cart has a folder.

No layout version bump: carts from earlier stidcart builds need no compatibility (memory
`compat-scope`). The reader already takes the table as written:

- `meta::decode` now returns the table as well as the records;
- `read_directory` carries it as `CartInfo.folders`;
- `load` gives it to the builder;
- `listing::describe` exposes it as `Listing.folders`.

A legacy cart has no metadata entry, so it gets no folders and its bytes don't move.

**Our C64 menu is unchanged.** It lists folders by group number and skips a group with no visible
entry, so the cart's order and hidden empty folders follow from the bytes. A VICE boot proves it
(AC-191).

**CLI.** `edit` gains:

- `--new-folder NAME`, `--remove-folder NAME` and `--move-folder NAME TO`, applied in the order
  given, among the in-place changes, before an entry `--move`;
- `list` gains a `folders:` line, printed only when the cart has folders, so the legacy goldens
  don't change.

**Window.**

- **The edit:** `Edit::Arrange` already carries `folders: [{name, entries}]`. Empty `entries` are
  now allowed, and the Document calls `set_folders` with the names before setting folders and
  moving entries. So an add, delete, rename or re-order of folders is still one undoable edit, and
  its parity oracle is the new CLI flags (AC-196).
- **The tree:** `tree.ts`'s `buildTree(rows, folders)` starts from the cart's folder list, so
  empty folders show and keep their place. Moving the last entry out no longer drops the folder.
  New helpers `addFolder`, `deleteFolder` and `untitledName`.
- **New Folder:** `Listing` exports `newFolder()`, which adds "untitled folder" (or the first free
  "untitled folder N"), then opens its name once the edit lands. App calls it from three places:
  - the Toolbar button;
  - the native menu item `new-folder` (File ▸ New Folder, ⇧⌘N);
  - a bits-ui `ContextMenu` on the empty space under the list.
- **Empty folders:** an empty folder's card holds a dashed "empty · drag entries here" row
  (`data-folder-drop`), which `rowDrag` reads as dropping into that folder.
- **Menus:** `FolderMenu` gains Delete Folder. `EntryMenu`'s submenu becomes "Move to":
  - a `// folders` label, then every folder with a ✓ on the entry's own;
  - "Not in a folder", checked when it is in none;
  - "New Folder with Entry".

  Choosing what is checked sends nothing. Move up and Move down move into their own group.

- **Inspector:** the details become an inspector `<aside>` to the right of the list, at full height.
  - Its header shows `// entry N`, the name, the kind badge, and the size and bank.
  - The fields are stacked, and Folder is a `<select>` of the cart's folders plus "Not in a
    folder", sending `setMeta`.
  - With nothing selected it says "Select an entry to see its details".

Rejected:

- A layout version 2: there is nothing to stay compatible with (user, 2026-11-08), and v1's
  bytes already allow it.
- Folder list in the frontend only: it would vanish on save.
- Deleting a folder with its entries: destructive, while Finder's model of moving them out is
  safe.
- Hiding the inspector until something is selected: the list would jump.

→ D-063.

## Constitution check

| Rule                                      | Applies? | Compliance note                                                                                                                                                                                                                                                                          |
| ----------------------------------------- | -------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `testing.md`                              | yes      | Failing tests per (AC × surface) first; cargo + vitest; full gates before PR                                                                                                                                                                                                             |
| `no-downgrade.md`                         | yes      | Every folder-editor capability stays. The panel's free-text Folder field becomes a choice of folders: the user asked for stable folders created by New Folder, and a new name typed as free text would still be possible through New Folder; recorded in D-063 as approved with the plan |
| `no-overengineering.md`                   | yes      | No layout version, no menu change, one edit kind reused; CLI flags only for what the window does (parity)                                                                                                                                                                                |
| `versioning.md`                           | yes      | `feat/` → 0.27.0 in the first code commit (base 0.26.0 after PR #58)                                                                                                                                                                                                                     |
| `commits.md`                              | yes      | One commit per flip; pushed; branch `feat/folder-browser`, never `main`                                                                                                                                                                                                                  |
| `ac-convention.md` EVAL                   | no       | No scorer or LLM output                                                                                                                                                                                                                                                                  |
| MAP: byte parity against frozen fixtures  | yes      | Legacy fixtures carry no metadata entry, so no folders; parity tests unchanged and run (AC-190)                                                                                                                                                                                          |
| MAP: the oracle decays                    | yes      | Empty folders and folder order are stidcart-only: evidence is round-trip tests and a VICE boot (AC-191)                                                                                                                                                                                  |
| MAP: format facts from `docs/format/`     | yes      | The folder-table paragraph changes there first, in the first commit of the build                                                                                                                                                                                                         |
| MAP: no global state / I/O policy in core | yes      | Folder ops are pure builder mutators                                                                                                                                                                                                                                                     |
| MAP: describing vs rebuilding (D-011)     | yes      | `Listing.folders` is a description; the folder ops are rebuild-path edits                                                                                                                                                                                                                |
| `.claude/rules/lcd-conventions.md`        | yes      | ACs 185-196 claimed; new root stems `folder_browser_cli`, `folder_boot`; GUI `folder_browser.rs`                                                                                                                                                                                         |
| Memory `compat-scope`                     | yes      | No compatibility code for earlier stidcart carts; legacy fixtures still load and convert                                                                                                                                                                                                 |

## Cross-path behavior matrix

| AC       | Surface | Path                                               |
| -------- | ------- | -------------------------------------------------- |
| `AC-185` | CLI     | `crates/stidcart-cli/src/lib.rs:"--new-folder"`    |
| `AC-186` | CLI     | `crates/stidcart-cli/src/lib.rs:"--remove-folder"` |
| `AC-187` | CLI     | `crates/stidcart-cli/src/lib.rs:"--move-folder"`   |
| `AC-188` | CLI     | `crates/stidcart-core/src/lib.rs:fn set_meta`      |
| `AC-189` | CLI     | `crates/stidcart-cli/src/lib.rs:"folders:"`        |
| `AC-189` | RENDER  | `gui/src/lib/Listing.svelte`                       |
| `AC-192` | RENDER  | `gui/src/lib/Toolbar.svelte`                       |
| `AC-193` | RENDER  | `gui/src/lib/FolderMenu.svelte`                    |
| `AC-194` | RENDER  | `gui/src/lib/EntryMenu.svelte`                     |
| `AC-195` | RENDER  | `gui/src/lib/MetaPanel.svelte`                     |
| `AC-196` | RENDER  | `gui/src-tauri/src/document.rs`                    |

**Test scope:** `crates/stidcart-cli/tests/folder_browser_cli.rs`, `crates/stidcart-core/tests/folders.rs`, `crates/stidcart-cli/tests/folder_boot.rs`, `gui/src-tauri/tests/folder_browser.rs`, `gui/src/lib/tree.test.ts`, `gui/src/lib/Listing.test.ts`, `gui/src/lib/EntryMenu.test.ts`, `gui/src/lib/FolderMenu.test.ts`, `gui/src/lib/MetaPanel.test.ts`, `gui/src/lib/Toolbar.test.ts`, `gui/src/lib/viewer.test.ts`

## Reused primitives

| Existing primitive                                               | Path                                              | Used for                                              |
| ---------------------------------------------------------------- | ------------------------------------------------- | ----------------------------------------------------- |
| `meta::check`, `MetaError` (incl. `Kept`)                        | `crates/stidcart-core/src/meta.rs`                | Folder-name rules and refusals                        |
| `CartBuilder::set_meta`, `move_entry`, `meta_entry`              | `crates/stidcart-core/src/lib.rs`                 | Folder list kept beside entries; META written from it |
| `read_directory` / `load_inner` META walk                        | `crates/stidcart-core/src/read.rs`                | Carrying the folder table into the builder            |
| CLI `Op` pipeline                                                | `crates/stidcart-cli/src/lib.rs`                  | Three new ops                                         |
| `Edit::Arrange` + `arrange()`                                    | `gui/src-tauri/src/document.rs`                   | Folder add/delete/rename/order as one edit            |
| `tree.ts`, `rowDrag.ts`                                          | `gui/src/lib/`                                    | Tree model and drop targets                           |
| `EntryMenu`, `FolderMenu`, `EditableText`, bits-ui `ContextMenu` | `gui/src/lib/`                                    | Menus, inline naming, empty-space menu                |
| VICE boot harness                                                | `crates/stidcart-cli/tests/support/vice.rs`       | AC-191                                                |
| Design canvas boards                                             | https://claude.ai/artifact/21eX5WRUmSu77f8zxzMmjY | Visual reference                                      |

## Data model / Contracts

- Core: `CartBuilder::folders() -> &[String]`, `add_folder(&str) -> Result<(), FolderError>`,
  `remove_folder(&str) -> Result<(), FolderError>`, `move_folder(&str, usize) -> Result<(),
FolderError>`, `set_folders(Vec<String>) -> Result<(), FolderError>`. `FolderError` is
  `Invalid(MetaError) | Exists(String) | Unknown(String) | Position { to, len }`, so its messages
  match `edit`'s. `meta::encode(folders, records)`. `Decoded::Records(folders, records)`.
  `CartInfo.folders`, `Listing.folders: Vec<String>`.
- CLI: `--new-folder NAME`, `--remove-folder NAME`, `--move-folder NAME TO`; `list` line
  `folders: Arcade (2)  Demos (1)  Utilities (0)`.
- GUI: `ListingDto.folders`, `types.ts` `Listing.folders: string[]`, menu id `new-folder`,
  `Listing` export `newFolder()`, EntryMove `{ kind: 'newFolder' }` kept (now "New Folder with
  Entry"), `FolderMenu` `ondelete(name)`, `MetaPanel` props `folders`.

## File structure

### New files

```
crates/stidcart-core/tests/folders.rs        AC-190: folder table order, empty folders stored, legacy fixtures unchanged
crates/stidcart-cli/tests/folder_browser_cli.rs  AC-185..AC-189 (CLI)
crates/stidcart-cli/tests/folder_boot.rs     AC-191: VICE boot, folder order and empty folders hidden
gui/src-tauri/tests/folder_browser.rs        AC-196 parity, empty folders through arrange, AC-189 DTO
```

### Modified files

- `Cargo.toml`, `Cargo.lock`, `gui/src-tauri/Cargo.toml`, `gui/src-tauri/Cargo.lock`: 0.27.0
- `docs/format/easyflash-cart-format.md`: folder table = the cart's folders, empty ones allowed
- `crates/stidcart-core/src/meta.rs`: encode from a folder list, decode returns it, `FolderError`
- `crates/stidcart-core/src/lib.rs`: `folders` on the builder and its operations, META written with folders
- `crates/stidcart-core/src/read.rs`: carry the folder table
- `crates/stidcart-core/src/listing.rs`: `Listing.folders`
- `crates/stidcart-core/tests/meta.rs`: follow the encode/decode shape if it names them
- `crates/stidcart-cli/src/lib.rs`: three ops, usage, `folders:` line
- `gui/src-tauri/src/document.rs`: arrange sets the folder list, empty folders allowed
- `gui/src-tauri/src/lib.rs`: `ListingDto.folders`
- `gui/src-tauri/src/menu.rs`: File ▸ New Folder ⇧⌘N
- `gui/src-tauri/tests/contract.rs`, `gui/src-tauri/tests/folders.rs`: follow the DTO and folder stability
- `gui/src/lib/types.ts`: `Listing.folders`, `MenuId` `new-folder`
- `gui/src/lib/fixtures/two.json`, `gui/src/lib/fixtures/document.json`: `folders: []`
- `gui/src/lib/tree.ts`, `gui/src/lib/tree.test.ts`: folders from the cart, stable folders, add/delete/untitled name
- `gui/src/lib/rowDrag.ts`, `gui/src/lib/rowDrag.test.ts`: the empty-folder drop row
- `gui/src/lib/Listing.svelte`, `gui/src/lib/Listing.test.ts`: empty cards, New Folder, empty-space menu, inspector layout
- `gui/src/lib/EntryMenu.svelte`, `gui/src/lib/EntryMenu.test.ts`: Move to with checks
- `gui/src/lib/FolderMenu.svelte`, `gui/src/lib/FolderMenu.test.ts`: Delete Folder
- `gui/src/lib/MetaPanel.svelte`, `gui/src/lib/MetaPanel.test.ts`: the inspector
- `gui/src/lib/Toolbar.svelte`, `gui/src/lib/Toolbar.test.ts`: New Folder button
- `gui/src/lib/viewer.svelte.ts`, `gui/src/lib/viewer.test.ts`: the `new-folder` menu id
- `gui/src/App.svelte`: wire New Folder to the listing
- `README.md`: folders, the new flags
- `docs/lcd/work/folder-browser/`, `docs/lcd/DECISIONS.md`, `docs/lcd/SPEC.md`, `docs/lcd/MAP.md`, `.claude/rules/lcd-conventions.md`

## Risks & rejected alternatives

- **Free-text folder field → choice:** a new folder name comes from New Folder, not from typing a
  name in the panel. The CLI and the import still create folders by name.
- **Right-click on empty space** only covers the area under the list; rows keep their own menus.
- **Older stidcart builds** reading a new cart would mis-handle an empty folder only if they
  re-saved it; there are none outside this machine.
