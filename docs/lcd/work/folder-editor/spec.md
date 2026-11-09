# Spec: Folders replace groups, with a folder tree in the window

> **Deep lane · Phase 1 of 6 (Specify).** This document owns **what** and **why**. No tech
> stack, no file paths, no function names. Move all of that to `plan.md` in Phase 2.

**Slug:** `folder-editor`
**Created:** 2026-11-08
**Related issue:** none
**Status:** draft <!-- draft | approved | implemented | abandoned -->

---

## Problem

An entry's metadata carries a "group", and our C64 menu always shows groups as folders (D-057).
The name says "label" while the behaviour says "folder", so a user who fills it in as information
(a publisher, a genre) gets folders on the C64 without having asked for them. The window doesn't
show the structure either: the group is a column and a text field, so the only way to see how a
cart is organised is to boot it. Nothing in the window or the CLI can change an entry's position,
so a user can't put a folder's entries in the order they want, and EasyLoader's flat list keeps
whatever order the entries were added in. The user decided (2026-11-08) that this field is a
folder, full stop. A separate label field would make the metadata too heavy, and the note field
already covers labels.

## User stories

- As a cart author, I want the field called what it does (a folder), so that I don't create
  folders on the C64 by accident when I meant to add a label.
- As a cart author, I want the window to show my cart as the folders and entries our menu will
  show, so that I can see how it's organised without booting it.
- As a cart author, I want to drag entries into and out of folders and into a new position, so
  that I can organise a cart directly instead of typing folder names into a field.
- As a cart author scripting carts, I want to move an entry to a new position from the command
  line, so that what the window can do, `edit` can do too.

## Acceptance criteria

**AC-173** (surfaces: CLI): Given a cart `edit` accepts, when `edit --folder N TEXT` runs, then
entry N gets folder TEXT with the limits `--group` had (at most 16 characters, PETSCII-mappable,
an empty TEXT clears it), and the result is byte-identical to what `edit --group N TEXT` writes
for the same values. `--group` is still accepted and the usage text names `--folder`.

**AC-174** (surfaces: CLI): Given an import file, when an `[[entry]]` table sets `folder`, then
the entry gets that folder, byte-identical to the same table setting `group`, which is still
accepted. A table setting both is refused naming the entry, and nothing is written.

**AC-175** (surfaces: CLI, RENDER): `list`'s metadata line labels the field `folder`, the window
lists no Group or Folder column (the tree shows the folder, AC-178), and the entry panel's field is
labelled "Folder". No stored byte changes: a load and save of a cart whose entries have groups is
byte-identical to the input. *(Amended 2026-11-08, D-062: was "the window's column is headed
Folder".)*

**AC-176** (surfaces: CLI): Given a cart `edit` accepts, when `edit --move FROM TO` runs, then the
entry at index FROM of the cart as given sits at index TO of the result, every other entry keeps
its relative order, the moved entry keeps its name, flags, kind, data and metadata, and `list` of
the result shows it at TO.

**AC-177** (surfaces: CLI): `edit --move` with an index outside the cart, given more than once in
one invocation, or combined with `--add`, `--add-aligned` or `--remove`, is refused with the reason,
exits non-zero and leaves every file byte-identical. Any other operation in the same invocation
names entries by their index in the cart as given.

**AC-178** (surfaces: RENDER): Given a cart where at least one entry has a folder, whichever menu it
carries, the window lists one folder card per folder (a header with its name and how many entries
it holds) in the order our menu lists folders, each holding its entries in directory order, then
the entries with no folder under a "not in a folder" label. Each entry row shows the same values
as before, its number inside the indent with its name. A folder's header collapses and expands
its entries, and folders open expanded. A cart with no folder shows the flat list as before.
*(Amended 2026-11-08, D-062: was "one folder row per folder", the number in its own column.)*

**AC-179** (surfaces: RENDER): Given an editable cart, dragging an entry onto a folder row puts it
in that folder, dragging it to the top level clears its folder, and dragging it between rows moves
it to that position, in the tree or in the flat list. Each drop that changes something is one
undoable edit. A drop that changes nothing adds no undo step and doesn't mark the cart Edited.

**AC-180** (surfaces: RENDER): After any tree operation (AC-179, AC-181, AC-182, AC-183), the
directory holds the entries in the tree's order: each folder's entries together, folders in the
tree's order, then the entries with no folder. So our menu lists the folders as the tree shows
them, and EasyLoader lists the entries in the same order. Saving writes byte-for-byte what `edit`
writes for the equivalent `--folder` and `--move` edits. A cart opened and saved with no tree
operation keeps its directory order.

**AC-181** (surfaces: RENDER): Dragging a folder row among the folder rows moves the folder and all
its entries to that position, as one undoable edit. A folder row can't be dropped into a folder,
and an entry can't be dropped deeper than one folder: such a drop is not accepted and nothing
changes.

**AC-182** (surfaces: RENDER): An entry's menu offers "Move to folder" listing the existing folders,
"No folder" when the entry is in one, and "New folder…", plus "Move up" and "Move down", so every
drag is available without dragging. "New folder…" takes a name with the folder rules and moves the
entry into a new folder of that name. An invalid name, or one another folder already has, is
refused with the reason and nothing changes. Each command is one undoable edit.

**AC-183** (surfaces: RENDER): Renaming a folder from its row renames it for every entry in it, as
one undoable edit. An empty or invalid name, or one another folder already has, is refused with the
reason and nothing changes.

**AC-184** (surfaces: RENDER): The entry panel's Folder field stays a metadata edit: changing it
sets that entry's folder as `edit --folder` does and moves no entry in the directory. The tree
shows the entry under its new folder.

## Out of scope

- A separate label, genre or tag field next to the folder (decided with the user, 2026-11-08: too
  much metadata; the note field covers labels).
- Nested folders. Our menu has one level, and so does the tree.
- Storing an empty folder. A folder exists only while an entry is in it.
- Any change to our C64 menu or to the stored metadata layout. The folder is the existing group
  field under a new name.
- Folders in EasyLoader. It keeps its flat list; only its order follows the tree (AC-180).
- Re-ordering by `edit --folder`, the import or the entry panel. Only tree operations write tree
  order.
- Dragging several entries at once, and CLI commands to rename or re-order whole folders (a folder
  rename is a series of `--folder` edits, a folder move a series of `--move` edits).
- Re-ordering on carts the window opens view-only.

## Open questions

- [x] Directory order after a drag → tree order (user, 2026-11-08; AC-180).
- [x] CLI re-order → `edit --move FROM TO` (user, 2026-11-08; AC-176, AC-177).
- [x] "New folder" → always made by moving an entry into it, never empty (user, 2026-11-08; AC-182).
- [x] When the tree shows → whenever any entry has a folder, whichever menu (user, 2026-11-08; AC-178).
- [x] One `--move` per invocation, refused next to `--add`/`--remove`: decided here, since a second
      move or a shifting operation would make "the index in the cart as given" (AC-27) ambiguous for TO.
- [x] In-window dragging alongside the file drop from Finder: feasible (recon, 2026-11-08); the
      how belongs to plan.md.

---

<!--
Quality checks before approving this spec:
  - No tech stack named (no language / framework / database appears anywhere)
  - No file paths or function names
  - Every acceptance criterion follows `**AC-N** (surfaces: <CSV>): <body>`
  - Surface tokens are exactly from {CLI, HTTP, MCP, RENDER, DB, EVAL, none}
  - Every acceptance criterion is testable (you can describe an assertion that proves it)
  - Out-of-scope is filled in, even if just one bullet
  - Open questions either all answered, or explicitly flagged for /lcd:plan to resolve
-->
