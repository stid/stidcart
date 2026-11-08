# JOURNAL — menu-folders

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the _entire_
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->

## NOW

- **Lane:** Standard
- **Goal:** Our C64 menu reads the `STIDCART.META` entry once at boot and shows titles in the list, the full title, year, author and note in the pane, and groups as folders (D-057)
- **Next action:** S3 — failing boot tests for AC-166..AC-169
- **Branch:** feat/menu-folders · **Updated:** 2026-11-08

## STEPS

- [x] S1 — version 0.22.0 → 0.23.0 (first commit on the branch)
- [x] S2 — triage, D-057, AC-166..AC-169 (from the approved menu-meta spec's M-8..M-11), this JOURNAL
- [ ] S3 — failing boot tests in `menu_boot.rs` for AC-166..AC-169 (carts built with `edit --title/--group/...` or the core API)
- [ ] S4 — implement in `menu/src/menu.s` (and `launch.s` if the copy loop lives there), rebuild `menu.bin`/`menu.lbl`, the replaced build's fingerprint joins `EARLIER_STIDCART_BUILDS`
- [ ] S5 — `menu/README.md`, the F7 help (folder keys), gates with `STIDCART_REQUIRE_TOOLS=1`
- [ ] S6 — audit, reconcile, closeout, PR

## DECISIONS (this work-item)

- read once at boot into `$4000-$7FFF`; titles, pane fields, folders, flat top-level search, resume into the folder → D-057

## OPEN QUESTIONS

- none

## EDIT BOUNDARY (paths this work may touch)

- `menu/src/menu.s`
- `menu/src/launch.s`
- `menu/menu.bin`
- `menu/menu.lbl`
- `menu/README.md`
- `crates/stidcart-core/src/menu.rs`
- `crates/stidcart-core/tests/menu_builds.rs`
- `crates/stidcart-cli/tests/menu_boot.rs`
- `crates/stidcart-cli/tests/meta_boot.rs`
- `crates/stidcart-cli/tests/support/`
- `Cargo.toml`
- `Cargo.lock`
- `gui/src-tauri/Cargo.toml`
- `gui/src-tauri/Cargo.lock`
- `docs/lcd/SPEC.md`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/triage-log.md`
- `docs/lcd/work/menu-folders/JOURNAL.md`
- `.claude/rules/lcd-conventions.md`

<!-- /lcd-resume -->

---

## Acceptance criteria

These are M-8..M-11 of `docs/lcd/work/menu-meta/spec.md`, which the user approved on 2026-11-08.

**AC-166** (surfaces: none): Given our menu on a cart with metadata, when it boots, then an entry with a
title shows the title in its list row, cut to the row's width, and the details pane shows the title
in full, wrapped to the pane's width, with the year, the author and the note. An entry without
metadata shows as it does today. A-Z sorts by the text each row shows.

**AC-167** (surfaces: none): Given a cart where some entries have a group, then the top level lists one
folder row per group that has a visible entry, with a folder badge, the group's name and its entry
count, followed by the ungrouped entries.

- RETURN or fire on a folder opens it.
- Inside a folder, DEL with no filter text, or RUN/STOP, goes back to the top level with that folder
  selected. RUN/STOP at the top level still leaves to BASIC.
- The kind filter, A-Z, paging, F5 and HOME/CLR act on the folder's entries.
- After a soft reset, an entry that was started from a folder is restored inside that folder
  (D-053).

**AC-168** (surfaces: none): Given filter text typed at the top level, then the matching visible entries
from every folder and the top level are listed flat. The text matches an entry's name or its title,
ignoring case.

**AC-169** (surfaces: none): Given our menu, it reads the metadata entry once at boot, with interrupts
masked so that AC-127 holds, into `$4000-$7FFF`. The menu boots as for a cart without metadata in
each of these cases:

- the entry is missing;
- its layout version is unknown;
- it is larger than 16 KB;
- a record's length would run past the payload.

A record whose stored name doesn't match its slot is ignored.

## LOG (append-only; not read on resume)

- 2026-11-08 — Triage: Standard (3 signals: a large menu change with a binary rebuild, boot tests, may
  span a session). The last of the user's menu improvements and the C64 half of menu-meta (D-054,
  D-055).
  - **Layout:** `docs/format/easyflash-cart-format.md`, section "stidcart extension: entry metadata
    (`STIDCART.META`)".
  - **Finding the entry:** match on the name, the hidden bit, the program kind and the `STIDMETA`
    magic, taking the last such slot. It is not at a fixed slot.
  - **Bank layout:** the entry may be type `0x01` (interleaved) or `0x02`/`0x03` (confined to one
    half), so the copy follows the layout. It skips the 2-byte load address and puts the payload at
    `$4000-$7FFF`.
  - **Indexes:** a record's index is its entry's position among the slots other than META. With
    META last, that equals the directory slot.
  - **Records:** a length byte, the index, the 16 name bytes, the group number (0 for none), then
    the title, year, author and note, each as a length byte plus PETSCII.
  - **Groups:** G fixed 16-byte NUL-padded names. Names match exactly.
  - **Memory:** the menu's RAM is `$C100-$C3FF` plus the character set at `$3800`. menu-highmem uses
    `$0100-$014F` only while starting a program. So `$4000-$7FFF` is free, VIC bank 0 doesn't show
    it, and a program the menu starts may overwrite it.
  - **ROM:** 2689 bytes free after menu-highmem.
  - **Ids:** AC-166..AC-169 claimed, next free AC-170.
