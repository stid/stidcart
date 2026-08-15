# Spec: Editing an existing cart

> **Deep lane · Phase 1 of 6 (Specify).** This document owns **what** and **why**. No tech
> stack, no file paths, no function names. Move all of that to `plan.md` in Phase 2.

**Slug:** `cart-edit`
**Created:** 2026-08-14
**Related issue:** none
**Status:** draft <!-- draft | approved | implemented | abandoned -->

---

## Problem

A cart can be built from a set of programs and it can be read back and described, but nothing
between those two ends exists: changing one entry means rebuilding the whole cart from the original
inputs, and whoever holds the cart may no longer have them. Every change — dropping a program that
turned out not to boot, fixing a name that reads wrong in the menu, hiding an entry, swapping in a
newer build of one program — currently costs a full rebuild from sources that may be gone. The read
path already reconstructs the exact state a cart was built from, so the missing capability is the
mutation and the write-back, not the understanding of the cart.

A second, smaller problem rides along. The hidden flag is a real part of the format and the listing
already reports it, but nothing here can produce or preserve it: a cart containing a hidden entry is
refused when read for rebuilding, purely because the model cannot express it. That refusal exists to
prevent silent data loss, and it stops being needed the moment the model can carry the flag.

## User stories

- As someone maintaining a multicart, I want to add, remove and rename entries in a cart I already
  have, so that I can keep it current without owning the original program files.
- As someone whose cart holds a program that has since been updated, I want to swap that entry's
  program data while keeping its name and its place in the menu, so that the menu my users know does
  not change under them.
- As someone preparing a cart for release, I want to hide entries and set the names the menu and the
  container show, so that the cart presents the way I intend without rebuilding it.
- As someone editing a cart I cannot afford to lose, I want a failed edit to leave my cart exactly as
  it was, so that a mistake costs me nothing.

## Acceptance criteria

**AC-17** (surfaces: CLI): Given a cart and one or more program files to add, when the user edits the
cart with those additions, then the saved cart contains every entry it had before plus one entry per
added program, each named after its input's file stem.

**AC-18** (surfaces: CLI): Given a cart, when the user removes an entry by the index the listing
shows for it, then the saved cart contains every other entry, unchanged in name and relative order,
and does not contain the removed one.

**AC-19** (surfaces: CLI): Given a cart, when the user renames an entry by index, then that entry's
name in the saved cart is the new name and no other entry's name changes.

**AC-20** (surfaces: CLI): Given a cart, when the user sets the hidden flag on an entry by index,
then the saved cart reports that entry as hidden and every other entry as it was; clearing the flag
on that entry returns it to visible.

**AC-21** (surfaces: CLI): Given a cart and a replacement program file, when the user replaces an
entry's program data by index, then that entry's name and its position in the entry order are
unchanged and its stored program data is the replacement's.

**AC-22** (surfaces: CLI): Given a cart, when the user sets the cart's container name, then the saved
cart carries that container name and its menu name is unchanged — and setting the menu name leaves
the container name unchanged.

**AC-23** (surfaces: CLI): Given a cart, when the user edits it without naming a destination, then
the cart file itself holds the result; when a destination is named, the result goes there and the
input file's bytes are identical to what they were before the edit.

**AC-24** (surfaces: CLI): Given an edit that cannot be completed — an index that names no entry, a
resulting set that does not fit, an unreadable program file — when the user runs it, then the
command reports the reason on stderr, exits non-zero, and every file it was given is byte-identical
to what it was before; no partial cart is left behind.

**AC-25** (surfaces: CLI): Given a cart holding something this tool cannot rebuild unchanged — an
entry kind it cannot represent, or firmware regions that differ from the ones it carries — when the
user tries to edit it, then the edit is refused with the reason named, rather than saving a cart
that differs from the original in ways the user did not ask for.

**AC-26** (surfaces: CLI): Given an old-format (EasyFS1) cart, when the user tries to edit it, then
the edit is refused naming the format, consistent with how listing such a cart is refused.

**AC-27** (surfaces: CLI): Given a cart and several operations in one invocation, when any of them
address entries by index, then every index refers to the listing of the cart as it was given, not to
any intermediate state produced by the other operations in the same invocation.

**AC-28** (surfaces: CLI): Given a set of programs being built into a new cart, when the user marks
one of them hidden, then the built cart reports that entry as hidden; and a build that marks nothing
hidden produces the same bytes it produced before this capability existed.

**AC-29** (surfaces: none): Given a cart containing hidden entries, when it is read into editable
state and saved back with no operations applied, then the saved bytes are identical to the original
— the hidden flag survives the round trip, and the former refusal of such carts is retired.

**AC-30** (surfaces: none): Given any completed edit, when the result is saved, then its bytes are
exactly those a fresh build of the resulting entry set would produce — entries the user did not
touch may sit at different positions than they did in the input.

## Out of scope

- **Reordering entries.** The order is what the input cart had, with additions appended; there is no
  operation to move an entry within it.
- **Preserving the input cart's placement.** AC-30 settles this deliberately: an edit re-places
  everything, so an edited cart is not byte-comparable to its input outside the untouched round trip
  of AC-29.
- **Converting an old-format cart.** EasyFS1 carts are refused (AC-26), not upgraded.
- **Editing entry kinds the tool cannot represent** — cartridge entries and entries confined to one
  half of a bank. A cart containing them is refused whole (AC-25); there is no partial edit that
  leaves them alone.
- **Extracting program data out of a cart** to a file. Replacement writes data in; nothing reads it
  back out.
- **Changing the firmware a cart carries.** The embedded blobs are what a save writes, and a cart
  whose regions differ is refused rather than rewritten.
- **Merging two carts** into one.
- **Any graphical surface.** This is the command-line tool only.

## Open questions

- [x] How are existing entries addressed? **By index only**, as shown in the listing — unambiguous
  when two entries share a name.
- [x] Is replacing an entry's program data in scope? **Yes** (AC-21).
- [x] Does building gain a way to mark an entry hidden? **Yes** (AC-28), now that the model can carry
  the flag.
- [x] Does the hidden-entry read refusal go away? **Yes** (AC-29) — it exists only because the model
  was lossy.
- [x] In place or to a named destination? **In place by default**, destination redirects (AC-23),
  with a failed edit leaving every input untouched (AC-24).
- [ ] None blocking `/lcd:plan`.

---

<!--
Quality checks before approving this spec:
  - No tech stack named (no language / framework / database appears anywhere)
  - No file paths or function names
  - Every acceptance criterion follows `**AC-N** (surfaces: <CSV>): <body>`
  - Surface tokens are exactly from {CLI, RENDER, none} — the surfaces MAP.md records in use
  - Every acceptance criterion is testable (you can describe an assertion that proves it)
  - Out-of-scope is filled in, even if just one bullet
  - Open questions either all answered, or explicitly flagged for /lcd:plan to resolve
-->
