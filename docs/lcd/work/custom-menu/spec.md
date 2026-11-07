# Spec: Our own C64 menu, first slice

> **Deep lane · Phase 1 of 6 (Specify).** This document owns **what** and **why**. No tech
> stack, no file paths, no function names. Move all of that to `plan.md` in Phase 2.

**Slug:** `custom-menu`
**Created:** 2026-11-07
**Related issue:** none (D-005, D-037)
**Status:** approved <!-- draft | approved | implemented | abandoned -->

---

## Problem

Every cart the tool builds boots into EasyLoader, a third-party menu embedded unchanged. The
features wanted on the C64 side (a new look, search and sort, folders) all need a menu this project
owns and can change. EasyLoader can't be extended here: it ships as fixed blobs on the far side of a
licence boundary, and cart verification only accepts those exact bytes. The first step is a menu of
our own that does EasyLoader's basic job, listing a cart's entries and starting the one picked, and
that the tool can put into a cart, recognise and preserve. It has to be optional. The frozen
reference carts and every existing user's carts depend on EasyLoader, and the reference tool can't
vouch for a cart carrying a menu it has never seen. For those carts, booting them in an emulator is
the only evidence.

## User stories

- As a C64 user building a menu cart, I want to choose our menu instead of EasyLoader, so that
  the cart boots into the menu this project will keep improving.
- As a user at the C64, I want the menu to list the cart's entries and start the one I pick, a
  program or a cartridge, so that the cart is usable without EasyLoader.
- As a user with a cart that already carries our menu, I want to list and edit it from the command
  line and the window like any other cart, so that choosing the menu doesn't make the cart
  read-only.
- As a user who doesn't ask for anything new, I want my builds and edits to produce exactly what
  they did before, so that the new menu costs me nothing.
- As the maintainer, I want every change under review to boot a cart with our menu in an emulator,
  so that a broken menu can't merge on the strength of tests that were skipped.

## Acceptance criteria

**AC-81** (surfaces: CLI): Given programs and cartridge images `build` accepts, none of them an
Ocean cartridge, when `build` is asked for our menu, then the produced cart carries our menu in the
menu loader's place. Every other byte is the same as the same `build` without the request.

**AC-82** (surfaces: CLI): Given no menu is requested, when `build` runs, then it produces the bytes
it produced before this capability existed: EasyLoader, matching every frozen fixture. When `edit`
runs without a menu request, the cart keeps the menu it already carries, either one.

**AC-83** (surfaces: CLI): Given a cart `edit` accepts, when `edit` is asked for our menu or for
EasyLoader, then the cart carries that menu afterwards and nothing else changes. Switching a cart to
our menu and back gives back the original bytes.

**AC-84** (surfaces: CLI): Given a request that would leave our menu on a cart with an Ocean entry
(`build` with our menu and an Ocean input, `edit` asking for our menu on a cart holding one, or
`edit` adding an Ocean image to a cart carrying our menu), when it runs, then it is refused with an
error naming that entry, and nothing is written.

**AC-85** (surfaces: CLI, RENDER): Given a cart carrying either menu, when it is listed, then the
listing says which menu the cart carries, in `list`'s output and in the window. A cart whose menu
regions match neither is described and refused for editing as it is today.

**AC-86** (surfaces: RENDER): Given a cart carrying our menu, when it is opened in the window, then
it opens editable. Edits and Save keep our menu, and the saved bytes are the same as `edit`
writes for the same changes.

**AC-87** (surfaces: none): Given a cart carrying our menu and some hidden entries, when it boots in
the emulator, then the screen shows the cart's name and every visible entry's name in directory
order, with no hidden entry and the first entry selected.

**AC-88** (surfaces: none): Given a cart carrying our menu with more visible entries than fit on
one screen, when the selection moves past the last row shown, then the list scrolls, every visible
entry can be reached, and the last one starts when picked.

**AC-89** (surfaces: none): Given a cart carrying our menu with program entries, when a program is
picked, then it is in memory at its load address with the cart switched out of the memory map. A
program loading at or below the start of BASIC is started as `RUN` would start it, and one loading
higher is started at its load address. Either way, the program's own output appears on screen.

**AC-90** (surfaces: none): Given a cart carrying our menu with one entry of each cartridge kind
other than Ocean (8K, 16K, both Ultimax sizes, and the three xbank kinds), when each is picked, then
that cartridge starts as it would from its own `.crt`, and its own output appears on screen.

**AC-91** (surfaces: none): Given a cart carrying our menu, when the user presses the leave key,
then the C64 is at the BASIC `READY.` prompt with the cart switched out and the full BASIC memory
free.

**AC-92** (surfaces: none): Given a cart carrying our menu with no visible entries, when it boots,
then the menu says that the cart has no entries, doesn't hang, and the leave key still works.

**AC-93** (surfaces: none): The menu put into carts is exactly what its source produces and fits
the space the menu loader has. A check fails when the two differ or the menu outgrows that space.

**AC-94** (surfaces: none): In the checks required on every change under review, the emulator
boot checks run rather than skip. An emulator missing there is a failure.

**AC-95** (surfaces: CLI): Given a program that, once loaded, would occupy any address from `$D000`
to `$DFFF`, when a request would leave it on a cart carrying our menu (`build` with our menu,
`edit` asking for our menu, or `edit` adding or replacing such a program on a cart carrying it),
then the request is refused with an error naming that entry, and nothing is written.
*(Added 2026-11-07 during planning, after approval; the user was told. The menu starts programs
from that area of memory, so it can't load one over it.)*

## Out of scope

- Ocean-mode carts with our menu. They are refused (AC-84) until a later item.
- Starting programs that load into `$D000`–`$DFFF` from our menu. They are refused at build time
  (AC-95) until a later item needs them.
- Choosing the menu in the window. The window recognises it, shows it and keeps it (AC-85, AC-86).
  Switching stays a command-line action.
- A designed look: custom font, colours, sprites or animation. A readable text listing is enough
  here, and look and feel is the next item (D-037).
- Search, sort, filter and jumping to an entry by typing.
- Subdirectory navigation, the `0x4000` bit, and type `0x1A` entries. The menu treats a
  subdirectory-flagged xbank as the plain xbank it is today.
- Joystick control and EasyLoader's other keys and screens beyond listing, starting and leaving.
- Replacing the boot launcher or the flash-write API. Both stay EasyLoader's, unchanged.
- Making our menu the default.
- A boot on real EasyFlash 3 hardware. The emulator is the evidence for this slice (user's
  choice). A hardware check comes later.

## Open questions

All four forks below were resolved with the user on 2026-11-07 and are recorded in D-037. None
blocks `/lcd:plan`.

- [x] Ocean mode in this slice? No, refuse it (AC-84).
- [x] How much of the window? Recognise, show and preserve, without choosing (AC-85, AC-86).
- [x] What proves a boot? The emulator, running in the required checks rather than skipping
  (AC-94). Hardware isn't required for this slice.
- [x] Default or opt-in? Opt-in (AC-82).

Left for `plan.md`, not blocking:

- Which key leaves to BASIC, and which keys move and pick (AC-88, AC-91).
- How the boot checks drive keys and read the screen. The recon digest feeds this.

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
