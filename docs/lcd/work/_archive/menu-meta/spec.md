# Spec: Entry metadata and groups

> **Deep lane · Phase 1 of 6 (Specify).** This document owns **what** and **why**. No tech
> stack, no file paths, no function names. Move all of that to `plan.md` in Phase 2.

**Slug:** `menu-meta`
**Created:** 2026-11-08
**Related issue:** none
**Status:** approved <!-- the user approved the draft on 2026-11-08 -->

---

## Problem

A cart entry today is a name of at most 16 characters and nothing else, so "Impossible Mission II"
becomes "IMPOSSIBLE MISSI" and a cart of 200 games is one flat list with nothing to say what each
one is. EasyLoader has nowhere to keep more. stidcart writes both the cart and its C64 menu, so it
can store a full title, year, author, a note and a group for each entry, and show them. The user
asked for it, with folders for groups and three ways to edit it: flags on `edit`, an import of
many entries from one file, and fields in the window (2026-11-08). This work-item covers storing,
round-tripping, editing and listing the metadata. Showing it in the C64 menu, folders included,
is the follow-on `menu-folders`.

## User stories

- As a collector, I want each entry to carry its full title, year, author and a note, so that the
  cart says what each program is instead of a 16-character code.
- As a collector, I want to put entries into groups such as Games, Demos and Tools, so that the menu
  can show a big cart as folders.
- As a collector with a big cart, I want to import titles and groups for many entries from one
  file, so that I don't edit them one at a time.
- As a user of the desktop app, I want to edit an entry's metadata in the window with undo, so
  that it works like every other edit there.
- As a user of either C64 menu, I want carts carrying metadata to keep working under EasyLoader,
  so that switching menus never loses or breaks anything.

## Acceptance criteria

Fields and limits: title up to 48 characters, year up to 4, author up to 24, note up to 48, group
up to 16. Text must map to PETSCII as entry names do. An entry is in one group or none.

**AC-153** (surfaces: none): Given a cart whose entries carry metadata, when it is loaded and saved,
when other entries are removed or re-ordered, when the entry itself is renamed, and when the menu
is switched to EasyLoader and back, then every entry keeps its own metadata. Given a cart where no
entry carries metadata, the tool writes exactly the bytes it writes today, so every fixture and
golden still holds.

**AC-154** (surfaces: none): Given stored metadata whose records no longer match the cart's
entries (another tool re-ordered or renamed them), when it is loaded, then only the records whose
entry still matches are attached, the rest are dropped, and the listing says how many were
dropped. Given stored metadata in a layout version the tool doesn't know, it is described and
kept, and any edit that would have to rewrite it is refused naming the reason.

**AC-155** (surfaces: none): Given a cart with metadata, the stored metadata takes one directory
slot and is never listed as an entry or shown by either menu. A set that would need more slots than
the directory has is refused naming the shortfall, as today. Our menu's rule against programs that
load into the I/O area never refuses the metadata itself.

**AC-156** (surfaces: CLI): Given a cart, when `edit` is given a title, year, author, note or group
for the entry at an index `list` prints, then that field is set on that entry, an empty value
clears it, indexes refer to the cart as given (as for every other edit), and an over-long or
unmappable value is refused naming the field, with every file left as it was.

**AC-157** (surfaces: CLI): Given a cart and a file naming entries by their stored name (ignoring
case) with any of the five fields for each, when `edit` imports it, then every named entry gets the
given fields, fields not given are left alone, and an empty value clears. A name matching no entry
or more than one entry, or any invalid value, refuses the whole import naming it, and nothing is
written.

**AC-158** (surfaces: RENDER): Given a cart with metadata, then `list` prints each entry's metadata on
an indented line under it, only for entries that have some, so a cart without metadata lists exactly
as before; and the window shows each entry's title under its name and its group in a column.

**AC-159** (surfaces: RENDER): Given an editable cart in the window, when an entry is selected, then
its five fields can be edited; each change is one undoable edit, an invalid value shows `edit`'s
reason and changes nothing, and saving writes exactly what `edit` writes for the same values,
held against the command line as the window's other edits are.

## Out of scope

- Showing metadata in our C64 menu, titles in its list, the details pane, folders and searching by
  title. They are the follow-on work-item `menu-folders` (its ACs are AC-166..AC-169).
- Folders inside folders, and an entry in more than one group.
- Editing metadata on the C64.
- Fetching metadata from online databases.
- Importing at `build` time. Only `edit` imports, so there is one import path.
- Any change to EasyLoader, which skips the stored metadata like any hidden entry.

## Open questions

- [x] Fields, limits, groups as folders and the three ways to edit — answered by the user,
      2026-11-08.
- [x] Where it is stored — a hidden directory entry, chosen in the recon of 2026-11-08 over spare
      space in the loader slot, bank 0 and a fixed bank (D-054).
- [x] Import file format — TOML, as in the approved draft.
