# Spec: Stable folders, a New Folder action and an inspector on the right

> **Deep lane · Phase 1 of 6 (Specify).** This document owns **what** and **why**. No tech
> stack, no file paths, no function names. Move all of that to `plan.md` in Phase 2.

**Slug:** `folder-browser`
**Created:** 2026-11-08
**Related issue:** none (follows folder-editor, PR #58)
**Status:** approved <!-- the user approved the plan and asked for all of it, 2026-11-08 -->

---

## Problem

After trying the folder tree, the user found three things a file browser does better:

- **Folders aren't stable.** A folder exists only while an entry is in it, so moving the last entry
  out makes the folder vanish. New folders can only be made through an entry's submenu, which reads
  as an action on the entry, not on the cart.
- **The menus are hard to decode.** The entry's own folder is greyed at the top of the submenu
  like a heading. Disabled items look the same. Folder, no-folder and new-folder choices sit in one
  flat list, and order moves share a group with folder moves.
- **The details pane is disconnected.** It sits below the list, far from the selected row, and on a
  short window it competes with the list for height and scrolling.

Design reference: the "Folder browser" boards on the stid design canvas
(https://claude.ai/artifact/21eX5WRUmSu77f8zxzMmjY). The project is not public, so carts written by
earlier stidcart builds need no compatibility; carts from other tools must still open and convert
on save.

## User stories

- As a cart author, I want to create a folder before anything is in it, and keep it when it
  empties, so that I can lay out the cart's structure first and fill it later.
- As a cart author, I want New Folder where a file browser has it (a button, a shortcut, a
  right-click on empty space), so that making a folder doesn't depend on picking an entry first.
- As a cart author, I want menus that say clearly where an entry is and what each item does, so
  that I don't misread a disabled item as a heading.
- As a cart author, I want an entry's details beside the list, next to what I selected, so that I
  see which entry I'm editing and the list keeps its full height.
- As a cart author scripting carts, I want to create, remove and order folders from the command
  line, so that the window still edits only what `edit` can do.

## Acceptance criteria

**AC-185** (surfaces: CLI): `edit --new-folder NAME` adds an empty folder after the cart's folders.
An empty or invalid name (the folder rules), or one the cart already has, is refused with the
reason and nothing is written. The folder survives a load and save with no entry in it.

**AC-186** (surfaces: CLI): `edit --remove-folder NAME` removes the folder; its entries stay in the
cart with no folder. A name the cart doesn't have is refused and nothing is written.

**AC-187** (surfaces: CLI): `edit --move-folder NAME TO` puts the folder at position TO of the
cart's folder order, the others keeping theirs. An unknown name or a position outside the order is
refused and nothing is written.

**AC-188** (surfaces: CLI): A folder stays when its last entry leaves it, whether by `--folder N ""`,
`--folder N OTHER` or `--remove`. `--folder N NAME` (and the import) for a folder the cart doesn't
have adds it after the others.

**AC-189** (surfaces: CLI, RENDER): `list` prints the cart's folders in order, each with how many
entries it holds, empty ones included. The window shows every folder in that order; an empty one is
a card reading that it's empty and taking a dragged entry.

**AC-190** (surfaces: none): The stored folder table holds the cart's folders in the cart's order,
empty ones included, and entries point at their folder by that order. A cart whose only metadata is
an empty folder still stores it. A cart from another tool (each frozen fixture) loads and saves as
before, byte for byte where it did.

**AC-191** (surfaces: none): Our C64 menu lists a cart's folders in the cart's folder order and
shows no row for an empty folder. Proven by a VICE boot.

**AC-192** (surfaces: RENDER): New Folder (a button beside Add…, the app menu with ⇧⌘N, and a
right-click on the list's empty space) adds an empty folder named "untitled folder" (or the first
free "untitled folder N") after the others, as one undoable edit, and opens its name for editing.

**AC-193** (surfaces: RENDER): A folder's menu offers Delete Folder, which removes the folder and
moves its entries to the entries in no folder, keeping them in the cart, as one undoable edit. An
entry dragged onto an empty folder's card goes into it. Moving the last entry out of a folder, or
removing it, leaves the folder in place.

**AC-194** (surfaces: RENDER): An entry's menu offers "Move to" listing every folder with a check on
the entry's own, then "Not in a folder" (checked when it is in none), then "New Folder with Entry"
(named inline, as before). Choosing what is already checked changes nothing. Move up and Move down
form their own group.

**AC-195** (surfaces: RENDER): The entry details are an inspector on the right of the list. It shows
the selected entry's number, name, kind and size above its five fields; the folder is chosen from
the cart's folders or none, as a metadata edit that moves no entry. With nothing selected it says
so. The list keeps the window's full height beside it.

**AC-196** (surfaces: RENDER): A tree change that adds, removes, renames or re-orders folders,
including empty ones, is one `arrange`-style edit whose save is byte-for-byte what `edit` writes
for the matching `--new-folder`, `--remove-folder`, `--move-folder`, `--folder` and `--move` calls.

## Out of scope

- Showing an empty folder on the C64: our menu hides it (AC-191), and the menu is unchanged.
- Compatibility with carts written by earlier stidcart builds (not public yet; recorded in memory
  and D-063). Carts from other tools keep loading and converting.
- Nested folders, multi-select, dragging several entries at once.
- A CLI command to rename a folder (renaming is a series of `--folder` edits plus `--remove-folder`).
- Restyling anything else in the window beyond the inspector, New Folder and the menus.

## Open questions

- [x] Delete Folder keeps the entries (moves them out), never deletes them (proposed by Claude,
      accepted with the plan, 2026-11-08).
- [x] Empty folders on the C64 stay hidden; no menu change (2026-11-08).
- [x] No compatibility with intermediate stidcart builds (user, 2026-11-08).
