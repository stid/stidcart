# Spec: Desktop cartridge viewer (GUI, first slice)

> **Deep lane · Phase 1 of 6 (Specify).** This document owns **what** and **why**. No tech
> stack, no file paths, no function names. Move all of that to `plan.md` in Phase 2.

**Slug:** `gui-shell`
**Created:** 2026-10-11
**Related issue:** none
**Status:** approved <!-- draft | approved | implemented | abandoned -->

---

## Problem

The only way to see what a cartridge holds is the command-line `list`. That suits a test harness,
not someone sorting a folder of carts. The goal set for this project when it replaced ndefpack is
a usable desktop app, and none of it exists yet. Editing first would put the riskiest operations
(ones that rewrite files) into an untested window. A read-only viewer comes first instead: it puts
a window, file opening, the cartridge description and error reporting through the whole stack,
and it can't damage a file.

## User stories

- As a C64 user with a folder of EasyFlash carts, I want to open one in a window and see its
  entries, so that I know what's on it without using a terminal.
- As a user dragging carts in from the file manager, I want to drop a file onto the window, so
  that opening one is a single gesture.
- As a user who opens something that isn't a usable cart, I want a plain explanation of why it
  can't be shown, so that I know whether the file is old-format, damaged or not an EasyFlash cart
  at all, and I can carry on with another file.
- As the maintainer, I want the window and `list` to agree on every value, so that the GUI never
  becomes a second, divergent description of the format.

## Acceptance criteria

**AC-51** (surfaces: RENDER): Given any cartridge `list` describes (every frozen reference fixture,
including those that can be described but not rebuilt), when it is opened in the viewer, then the
window shows the file's name, the cart name, the producing tool, one row per entry (index, name,
flags `H`/`A`/`S`, kind label, bank, layout, size), the entry count and the free bytes. Each value
equals what `list` prints for the same file, with control characters in the header name replaced
the same way.

**AC-52** (surfaces: RENDER): Given the window is open, when a single cartridge file is dropped
onto it, then that file is opened and shown exactly as AC-51 describes.

**AC-53** (surfaces: RENDER): Given the window is open, when the user picks a file through the
window's open command, then that file is opened and shown exactly as AC-51 describes. Dismissing
the chooser leaves the current view unchanged.

**AC-54** (surfaces: RENDER): Given a file `list` refuses (an old-format EasyFS1 directory, a
malformed or truncated image, a file that is not a cartridge container, or a container for other
cartridge hardware), when it is opened, then the window shows an error carrying the same reason
`list` reports and no entry rows. Any previously shown cartridge is no longer displayed, and the
viewer stays responsive.

**AC-55** (surfaces: RENDER): Given an error is showing, when a valid cartridge is then opened,
then it is shown as AC-51 describes and the error is gone.

**AC-56** (surfaces: RENDER): Given the window is open, when more than one file is dropped at once,
then nothing is opened, a message says to drop one file, and the current view is unchanged.

**AC-57** (surfaces: RENDER): Given the viewer has just started, then before any file is opened it
shows an empty state inviting the user to open or drop a cartridge.

**AC-58** (surfaces: none): Opening and viewing a cartridge never writes to it. The file is
byte-identical after it was viewed, whether it opened cleanly or was refused.

## Out of scope

- Any editing, building or saving: add, remove, rename, hide, align, change names. Those are later
  slices, done after the viewer has shown the shell holds up.
- A distributable app: bundle, icon, code signing, notarisation, installer. This slice runs from
  the developer build.
- Verified support on Linux or Windows. The viewer must build where CI builds, but only macOS is
  exercised by hand.
- Recent files, several carts open at once (tabs or windows), remembering window state.
- Views beyond the listing: hex or bank maps, entry detail panes, PETSCII previews.
- Launching the cart in an emulator.
- Changing what `list` shows. The viewer mirrors the current listing. A new field goes into the
  description both views share, in its own work-item.

## Open questions

- [x] Platforms: macOS first, toolkit portable (user, 2026-10-11).
- [x] First slice scope: read-only viewer (user, 2026-10-11).
- [x] The toolkit: Tauri 2.x (D-030) with a Svelte 5 frontend (D-031).
- [x] How the listing is shared: one structured description in the core, which both `list` and
  the viewer render (D-032).
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
