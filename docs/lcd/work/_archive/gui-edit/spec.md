# Spec: Desktop cartridge editor (GUI, second slice)

> **Deep lane · Phase 1 of 6 (Specify).** This document owns **what** and **why**. No tech
> stack, no file paths, no function names. Move all of that to `plan.md` in Phase 2.

**Slug:** `gui-edit`
**Created:** 2026-10-11
**Related issue:** none
**Status:** approved <!-- draft | approved | implemented | abandoned -->

---

## Problem

The viewer (gui-shell) shows what a cart holds, but every change still has to be typed as an
`edit` command line: an entry index per option, a file path per addition, and no way to see the
result before it is written over the original. That works for a test harness. It doesn't work for
someone building a menu cart out of a folder of programs, who wants to drag files in, fix names,
hide a few entries and check the free space as they go. The viewer slice has shown that the window,
file opening and the shared description hold up, and the user has confirmed that on a real cart. So
the next step is the operations that rewrite files. Two properties matter most: nothing reaches disk
until the user asks for it, and a cart saved from the window is the same bytes `edit` would have
written.

## User stories

- As a C64 user assembling a menu cart, I want to drop programs and cart images onto the open cart,
  so that building one doesn't take a command line per file.
- As a user tidying an existing cart, I want to rename, hide, remove, replace and align entries and
  set its names in the window, so that I can see each change as I make it.
- As a user who makes a mistake, I want to undo it, and I want the original file left alone until I
  save, so that experimenting costs nothing.
- As a user about to close or switch carts, I want to be warned about unsaved changes, so that I
  don't lose work by accident.
- As a user starting from nothing, I want to create an empty cart and fill it, so that I don't need
  the command line to begin one.
- As the maintainer, I want a cart saved from the window to be byte-identical to what `edit` or
  `build` writes for the same request, so that the GUI never becomes a second, divergent writer of
  the format.

## Acceptance criteria

**AC-59** (surfaces: RENDER): Given a cartridge `edit` accepts, when it is opened (picked or dropped
with nothing open), then it is shown as AC-51 describes, its edit commands are available, and it
has no unsaved changes. Opening it still writes nothing (AC-58).

**AC-60** (surfaces: RENDER): Given a cartridge `list` describes but `edit` refuses (firmware
regions that differ from the embedded ones, the one rebuild precondition `list` does not apply),
when it is opened, then its listing is
shown as AC-51 describes, no edit command is available, and the window states the reason `edit`
gives for refusing it.

**AC-61** (surfaces: RENDER): Given an editable cart is open, when the user adds a file, either
through the add command or by dropping it on the window, then a new last entry appears whose name
is the file name without directory or extension. Its kind comes from the file's content (a program,
or a cartridge image of a kind `edit --add` accepts), as `edit --add` decides it.

**AC-62** (surfaces: RENDER): Given an editable cart is open, when the user adds a cartridge image
with 64 KB alignment requested, then the new entry is shown with the `A` flag. When the file is not
an xbank cartridge, the addition is refused as AC-70 describes, with the reason `edit --add-aligned`
gives.

**AC-63** (surfaces: RENDER): Given an editable cart is open, when several files are added at once
(chosen together or dropped together), then they are added in the order given, all of them or none.
If any one cannot be added (unreadable, unrepresentable, or the set does not fit), none is, and the
reason is shown. An accepted multi-file addition is a single step for undo.

**AC-64** (surfaces: RENDER): Given an editable cart is open, when the user removes an entry, then
it is gone from the listing and the entries after it move up one index.

**AC-65** (surfaces: RENDER): Given an editable cart is open, when the user renames an entry, then
the listing shows the new name as the cart will store it. A name longer than the format holds is
shown shortened, as the saved file will hold it.

**AC-66** (surfaces: RENDER): Given an editable cart is open, when the user replaces an entry's data
with a chosen file, then the entry keeps its name, its hidden state and its place in the order. Its
kind follows the new file, as `edit --replace` does.

**AC-67** (surfaces: RENDER): Given an editable cart is open, when the user hides a visible entry or
shows a hidden one, then the entry's `H` flag appears or disappears accordingly.

**AC-68** (surfaces: RENDER): Given an editable cart is open, when the user requests or clears 64 KB
alignment on an xbank entry, then its `A` flag appears or disappears. On an entry of any other kind,
alignment is either not offered or refused as AC-70 describes, with the reason `edit --align` gives.

**AC-69** (surfaces: RENDER): Given an editable cart is open, when the user changes the cart name or
the container header name, then the window shows the new value for that one, and the other is
unchanged.

**AC-70** (surfaces: RENDER): Given an editable cart is open, when an edit is one `edit` would
refuse (the entries do not fit, the directory is full, a second Ocean cartridge, an unreadable or
unrepresentable file, alignment on a non-xbank), then the window shows the reason `edit` gives. The
listing, the unsaved-changes state and the undo history are exactly as they were before the edit
was attempted.

**AC-71** (surfaces: RENDER): Given an editable cart has been changed, then the listing describes
the cart as saving would write it: every entry's bank and layout come from re-placing the whole set,
and the free bytes are recomputed. An entry the user never touched may therefore show a different
position than the file had when it was opened.

**AC-72** (surfaces: RENDER): Given an editable cart, when an edit is accepted, then the window shows
that there are unsaved changes. When undo or redo returns it to the state last saved (or the state
it was opened in, if never saved), the indicator clears. Moving away from that state sets it again.

**AC-73** (surfaces: RENDER): Given one or more accepted edits, when the user undoes, then the most
recent edit is reversed and the listing shows the state before it. When the user redoes, it is
reapplied. Every edit since the cart was opened can be undone. A new edit after an undo discards
whatever could have been redone. While a text field is being edited, undo and redo act on that
field's text, not on the cart.

**AC-74** (surfaces: RENDER): Given an editable cart with unsaved changes that has a file, when the
user saves, then the file is replaced by the edited cart and the unsaved-changes indicator clears.
When there are no unsaved changes, save is not available, so a clean cart is never rewritten.

**AC-75** (surfaces: RENDER): Given an editable cart, when the user chooses Save As and picks a path,
then the edited cart is written there, the window shows that file's name, later saves go to it, and
the indicator clears. The file the cart was opened from is unchanged. Dismissing the chooser changes
nothing.

**AC-76** (surfaces: RENDER): Given a save or Save As that cannot complete (unwritable destination,
disk error), then the window shows the reason, the destination holds either its previous content or
nothing at all and never part of a cart, and the cart still has unsaved changes.

**AC-77** (surfaces: RENDER): Given the window is open, when the user chooses New, then an empty
untitled cart with no entries is shown, editable, with the default cart and header names `build`
uses. Saving an untitled cart behaves as Save As.

**AC-78** (surfaces: RENDER): Given a cart with unsaved changes, when the user closes the window,
quits from the app (⌘Q or its Quit menu item), opens another cart or chooses New, then the window
asks whether to save, discard or cancel.
- Save: saves first, asking for a path when the cart is untitled, then continues.
- Discard: continues, and the changes are lost.
- Cancel, a dismissed path chooser or a failed save: stops, with the cart, its unsaved changes and
  the window all as they were.
Without unsaved changes, none of these asks.

**AC-79** (surfaces: RENDER): Given no editable cart is open (the empty state, an error, or a
view-only cart), when files are dropped, then a single file is opened as before. Several files are
refused with the message to drop one, and the current view is unchanged. Given an editable cart is
open, a drop adds the files as AC-61 and AC-63 describe and never opens them. (This revises AC-56,
which refused every multi-file drop.)

**AC-80** (surfaces: none): A cart saved after a sequence of edits made in the window is
byte-identical to what `edit` writes for the equivalent request on the same input file. A new cart
saved after adding files is byte-identical to what `build` writes for the same files in the same
order. Both hold over the frozen reference fixtures.

## Out of scope

- Reordering entries (drag to reorder, move up/down). `edit` has no reorder either. Placement
  decides positions, and the menu order would be a new capability in both front ends.
- Exporting an entry back out to a `.prg` or `.crt` file.
- Editing a cart `edit` refuses (AC-60): no partial editing, no stripping of the unsupported entry.
- Starting a new cart by dropping programs on the empty window. A single dropped file still opens,
  as today, and New followed by a drop does the same job.
- Several carts open at once (tabs or windows), recent files, remembering window state.
- Undo history that survives closing the cart, autosave, crash recovery.
- A distributable app: bundle, icon, signing, notarisation. Still runs from the developer build.
- Verified support on Linux or Windows: it builds where CI builds, and only macOS is exercised by
  hand.
- Detecting that the open file changed on disk behind the window's back.
- Asking about unsaved changes when macOS itself ends the app: Quit from the Dock, logout, restart
  or shutdown. These bypass the window (D-036), and catching them needs a native termination hook
  of its own. It's a follow-up, found in the live check (amended 2026-10-12, user).
- Anything `edit` can't do. The window offers the CLI's edit set and nothing more, and any new
  editing capability goes into both front ends in its own work-item.

## Open questions

- [x] Scope of the slice: the full `edit` set, plus New (user, 2026-10-11).
- [x] Save model: explicit save, Save As, an unsaved-changes indicator and prompt, undo/redo (user,
  2026-10-11).
- [x] A `.crt` dropped while a cart is open is added as an entry, never opened (user, 2026-10-11).
- [x] A multi-file drop on an open cart adds every file, all or none, as one undo step (user,
  2026-10-11).
- [x] How much undo history to keep in memory, and how. This is a plan question and doesn't block
  the spec: AC-73 requires every edit since opening to be undoable, and the plan decides how to
  hold that cheaply.

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
