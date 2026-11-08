# SPEC — stidcart (living current-state)

> **What this system DOES now — a capability index, not documentation.** One line per
> capability; the detail lives in the cited `D-NNN` (history) and, until pruned, the work-item
> delta. **Current-state only**: a replaced capability is rewritten in place — its history is
> kept in `DECISIONS.md`, never here. Folded at closeout by `lcd:reconcile`. Read only the
> section for the surface/area you're about to touch, not the whole file. Opt-in: this file
> exists only when `living-spec: on` in `.claude/rules/lcd-conventions.md`.

**Last compacted:** 2026-11-08 (menu-raster)

---

> ⚠️ Rows marked `D-015 (backfill)` were derived from `MAP.md` and the code on 2026-08-12, not
> AC-verified. Provisional seed; each decays into an AC-pinned row as work touches it. See D-015.

## Capabilities by surface

> One row per current capability. **Capability** = one terse line of what the system does now
> (behaviour, not implementation). **ACs** = the acceptance-criteria IDs that pin it. **Provenance**
> = the durable `D-NNN` that records the why (the work-item slug in parens is a breadcrumb — it may
> dangle after `/lcd:tidy` prunes the work-item, and that's fine). A `· stale: <date>` suffix
> on Provenance means Quick-lane work changed this behaviour after the last fold — treat the row as
> approximate until the next `reconcile` clears it. Delete any surface section that has no
> capabilities.

### CLI

| Capability (current behaviour, one line) | ACs | Provenance |
|---|---|---|
| `build` writes a `.crt` from PRG inputs and cartridge images, recognising a cartridge by its content; `-e` sets the menu name, `-o` the output path, `--hide INPUT` marks that input hidden, `--align INPUT` requests 64 KB alignment for an xbank, `--menu stidcart\|easyloader` picks the C64 menu the cart boots into (EasyLoader by default; ours changes only the loader slot), and each entry name is the input's file stem with case and spacing preserved. A build of programs only, marking nothing hidden and naming no menu, produces the bytes it produced before any of these existed | AC-28, AC-40, AC-42, AC-81, AC-82 | D-020 (cart-edit) · D-026 (cart-entries) · D-037 (custom-menu) |
| A `build` request naming no input programs succeeds and writes nothing, matching the reference tool rather than erroring | — | D-015 (backfill) · D-002 |
| `list` prints what a `.crt` contains, with positions taken from the image as stored rather than from a rebuild that would re-place every entry, and a `menu:` line naming the menu the loader slot holds: `EasyLoader`, `stidcart`, or `unrecognised` | AC-85 | D-038 (custom-menu) · D-011 |
| Failures print `stidcart: <reason>` on stderr and exit non-zero; an unknown command or option prints usage | — | D-015 (backfill) |
| `list` refuses an old-format (EasyFS1) cartridge, naming the format on stderr, rather than printing entries decoded under the wrong layout | AC-13 | D-017 (easyfs1-refusal) |
| `edit` changes an existing cartridge: `--add` appends a program or cartridge image (`--add-aligned` with 64 KB alignment), `--remove` / `--rename` / `--replace` / `--hide` / `--show` / `--align` / `--unalign` act on the entry at the index `list` prints whatever its kind, `--replace` takes the kind of the new data, `-e` / `--header` set the menu and container names independently, and `--menu stidcart\|easyloader` switches the C64 menu (a round trip gives back the original bytes); without `--menu` the cart keeps the menu it carries | AC-17..AC-22, AC-41..AC-43, AC-82, AC-83 | D-019 (cart-edit) · D-026 (cart-entries) · D-037 (custom-menu) |
| An edit writes over the cartridge it was given unless `-o` names a destination, in which case the input is left byte-identical | AC-23 | D-019 (cart-edit) |
| Every index in one invocation refers to the cartridge as given, not to the state a previous operation in the same invocation left | AC-27 | D-019 (cart-edit) |
| A failed edit — a bad index, a set that does not fit, an unreadable program — reports the reason, exits non-zero and leaves every file it was given byte-identical, with no partial cartridge left behind | AC-24 | D-019 (cart-edit) |
| `edit` refuses what it could not rebuild unchanged, naming the reason: a type `0x1A` EasyFlash entry (the one kind still unmodelled), firmware regions differing from the embedded blobs (the loader slot may hold either menu; one holding neither is refused), or an old-format (EasyFS1) directory. A subdirectory-flagged xbank is edited like any entry, and the result is byte-for-byte what `ndeft` writes for the same rename | AC-25, AC-26, AC-43, AC-48, AC-85 | D-019 (cart-edit) · D-011, D-017 · D-026 (cart-entries) · D-029 (subdir-entries) · D-038 (custom-menu) |
| A cartridge image `build` or `edit --add` cannot represent — an EasyFlash multicart, a subdirectory, an unlisted id — is refused naming what it is, exit non-zero, every file left byte-identical; alignment on a non-xbank entry is refused naming the entry | AC-42, AC-44 | D-026 (cart-entries) |
| A request that would leave our menu on a cart it cannot run — one with an Ocean entry, or a program whose loaded extent touches `$D000-$DFFF` — is refused naming the entry, through `build`, `edit --menu`, `--add` or `--replace`, and nothing is written | AC-84, AC-95 | D-037 · D-038 (custom-menu) |

### RENDER

| Capability (current behaviour, one line) | ACs | Provenance |
|---|---|---|
| The listing shows the cart name, producing tool and C64 menu, then one row per entry — index, name, flags (`H` hidden, `A` 64 KB aligned, `S` subdirectory bit), kind label, bank, layout (a program's offset, or a cartridge's bank runs per half), size — closing with the entry count and the free bytes. Sizes and the free figure read as bytes under 1 KB, then 1024-based KB with one decimal (whole for an exact multiple), then MB from 1024 KB; `list --bytes` gives exact byte counts instead. The free figure is the size a re-placement of the entries plus one more program accepts, so a program of exactly that size can be added and one byte more cannot, in Ocean mode too. For an image `load` refuses (and `edit` therefore cannot touch), the figure is the contiguous run at the top of the image as stored | AC-45, AC-46, AC-49, AC-51, AC-85, AC-115, AC-116 | D-026 (cart-entries) · D-027 (ocean-free-space) · D-029 (subdir-entries) · D-032 (gui-shell) · D-038 (custom-menu) · D-044 (readable-sizes) |
| Every kind the walk accepts — nine cartridge kinds, interleaved and both half-confined program kinds — has one distinct label, so no entry renders as `?` | AC-45 | D-026 (cart-entries) |
| Control characters in the container header name are replaced with `.` before printing, because a `.crt` is untrusted input and the field is free-form host text | — | D-015 (backfill) |
| On opening, the desktop app shows the same listing as `list`, value for value: file name, cart name, producing tool, menu, every row (index, name, flags, kind, bank, layout, size), entry count and free bytes, for every image `list` describes. Both render one core description. Hovering a size or the free figure shows its exact byte count, and each type badge is tinted in our C64 menu's colour for its kind's family (programs green, 8K yellow, 16K red, Ultimax purple, multi-bank blue, others neutral), in a light and a dark pair | AC-51, AC-85, AC-115, AC-117, AC-118 | D-032 (gui-shell) · D-030 · D-038 (custom-menu) · D-044 · D-045 (readable-sizes) |
| The desktop app opens a `.crt` picked with Open…/⌘O (dismissing the chooser changes nothing), or one dropped while no editable cart is open (empty, an error, or view-only); several dropped then open none and it says to drop one. Before any file it shows an empty state inviting either. With an editable cart open, every drop (`.prg` or `.crt`) adds instead, never opens | AC-52, AC-53, AC-56, AC-57, AC-79 | D-030 (gui-shell) · D-033 (gui-edit) |
| The window wears the stid design system, following the OS appearance: Catppuccin Latte in light and Mocha in dark, the peach accent (`#fe640b` / `#fab387`) with dark text on it, stid's radii, text in Geist and figures, paths and codes in Geist Mono. Both faces ship inside the app (OFL-1.1, `gui/src/fonts/`) and no stylesheet loads anything from the network. The type badges keep their own colours | AC-119, AC-120 | D-046 · D-047 (raster-brand) |
| The stidcart mark, a cartridge whose label is five raster bands (red, peach, yellow, green, blue), sits before the title in the title bar and heads the empty state, and is the app icon at every bundled size. The same five bands, in the theme's Catppuccin shades, run as a strip under the title bar whether the window is empty, shows a cart or shows a refusal | AC-121, AC-122 | D-046 (raster-brand) |
| A file `list` refuses shows the reason `list` gives and no rows, replacing whatever cart was shown; the next valid open clears it | AC-54, AC-55 | D-032 (gui-shell) |
| A cart `edit` accepts opens editable and clean, whichever menu it carries, and its saves keep that menu as `edit` does; one `list` describes but `edit` refuses (firmware differing from the embedded blobs) opens view-only, with `edit`'s reason and no edit commands. On an editable cart the menu is a picker between EasyLoader and stidcart; choosing the other is one undoable edit that saves what `edit --menu` writes, and a cart with an Ocean entry or a program loading into `$D000-$DFFF` refuses stidcart with `edit`'s reason, unchanged. A view-only cart shows the menu as text | AC-59, AC-60, AC-86, AC-112..AC-114 | D-033 · D-035 (gui-edit) · D-037 (custom-menu) · D-042 (menu-picker) |
| The window edits what `edit` does — add (picked or dropped; several at once all-or-none as one step; Add Aligned for an xbank), remove, rename (inline, shown as stored), replace (name, hidden and place kept, kind follows the file), hide/show, align/unalign (offered only on an xbank), cart and header names — plus New, an empty untitled cart with `build`'s default names. Each new entry is named by its file stem and classified as `edit --add` classifies it | AC-61..AC-69, AC-77 | D-033 · D-034 (gui-edit) |
| An edit `edit` would refuse shows `edit`'s reason and changes nothing — listing, unsaved state and undo history stay as they were | AC-70 | D-035 (gui-edit) |
| After an edit the listing describes the cart as saving would write it: the whole set is re-placed, so an untouched entry may show a new position, and the free bytes are recomputed | AC-71 | D-035 (gui-edit) · D-011 |
| Edits stay in memory: undo/redo walk every edit since opening (in a focused text field they act on its text), an Edited marker shows when the cart differs from the state last saved or opened, ⌘S replaces the file only when there are changes, ⌘⇧S writes elsewhere and adopts that file, an untitled cart saves through Save As, and a failed save leaves the destination whole and the changes unsaved | AC-72..AC-77 | D-033 · D-035 (gui-edit) |
| Closing the window, quitting from the app (⌘Q, its Quit item), opening another cart or New with unsaved changes asks Save / Don't Save / Cancel; only the first two go ahead, and a dismissed or failed save stops. macOS ending the app (Dock Quit, logout) does not ask | AC-78 | D-033 · D-036 (gui-edit) |

### Internal (surface: none)

> Invariants, calculations, and behaviours with no external surface — including the one-line
> capability `lcd:reconcile` folds from a surface-less work-item's `Goal`.

| Capability (current behaviour, one line) | ACs | Provenance |
|---|---|---|
| The whole library API is one owned value: `CartBuilder` accumulates entries (programs and cartridges), mutates them in place by index, and returns bytes, touching no filesystem and holding no global state. The index-addressed mutators panic on an out-of-range index, as `Vec` indexing does | — | D-015 (backfill) · D-001, D-019, D-023 |
| An edited cartridge is byte-for-byte a fresh build of the entry set the edits produced: saving runs the one placement path, so entries the edit never mentioned may land at different positions than the input had | AC-30 | D-019 (cart-edit) |
| Placement puts every cartridge before any program (Ocean first, then 64 KB-aligned ones stepping eight banks, then by height, first fit in both halves), then programs largest first: one fitting a gap below the top bank goes there confined to one half, the rest interleaved at the top; the directory keeps the caller's input order | AC-33, AC-34 | D-026 (cart-entries) · D-025 |
| Every build rewrites bank 0's reserved regions: EAPI and the boot code from the zlib blobs embedded as-is, the loader slot with the chosen menu (EasyLoader's blob, or our GPL menu from `menu/`), plus the `ef-nAME:` magic and the caller's cartridge name. The mode is derived from the entries: an Ocean entry owns bank 0 ROML and switches to the Ocean-mode boot and loader (loader in bank 1 ROMH), which only EasyLoader has; a second Ocean entry is a typed build error naming it | AC-36, AC-37, AC-81, AC-84 | D-005 · D-024 (cart-entries) · D-037 (custom-menu) |
| The CRT container header name and the EF-Name are separate caller-supplied fields at different offsets; the header defaults to `stidcart <crate-version>` | — | D-015 (backfill) · D-006, D-008 |
| Names cross the PETSCII boundary in both directions | — | D-015 (backfill) |
| `inspect` describes any image it can walk; `load` layers the rebuild preconditions on top and refuses what it could not reproduce byte-identically — firmware regions differing from the blobs the derived mode implies (normal or Ocean), a loader slot holding neither embedded menu, and type `0x1A` entries. `inspect` reports which menu the slot holds without refusing; `load` carries it into the builder | AC-36, AC-43, AC-85 | D-011, D-007 · D-026 (cart-entries) · D-029 (subdir-entries) · D-038 (custom-menu) |
| The hidden flag is part of the entry model on both sides, so a cartridge carrying hidden entries loads, edits and saves back with names, order, flags and program data intact — it was refused for rebuilding only while the description had nowhere to keep the flag | AC-29 | D-020 (cart-edit) · D-011 |
| Both read paths describe and load every cartridge kind and half-confined programs (same-half, next-bank continuation); a load-and-save of a reference fixture holding them is byte-identical. The CrtUsage subdirectory bit is carried on an xbank through decode, load and save, and refused as malformed at its slot on any other cartridge kind, as the reference does. Only type `0x1A` entries are refused, as unsupported | AC-35, AC-43, AC-47, AC-50 | D-026 (cart-entries) · D-029 (subdir-entries) |
| Every cartridge directory entry round-trips through encode and decode with all fields, and one whose bank runs reach past the top of flash in either half, or whose opposite-half count exceeds its size, is refused as malformed at that slot | AC-31, AC-32 | D-026 (cart-entries) · D-017 |
| A cartridge image becomes an entry by its own header and packet layout — generic → 8k/16k/Ultimax, Ocean → Ocean/Ocean 512k, EasyFlash xbank → the matching xbank kind — banks mapped into the two halves in order | AC-39 | D-026 (cart-entries) · D-023 |
| An xbank is bounded by flash geometry, not a flat 512 KiB: one wider than 32 banks is placed, written, described and loaded, though the reference tool refuses to read it back | AC-38 | D-022 (cart-entries) |
| Old-format (EasyFS1) directories are refused by both read paths with a typed `Unsupported` error — the reserved flag bits `0x60` of the first non-end slot are the format's only version signal — while well-formed EasyFS2 directories are unaffected | AC-14, AC-16 | D-017 (easyfs1-refusal) · D-007 |
| A directory whose slots disagree about the format is refused as malformed rather than as old-format: the first non-end slot settles the format for all of them, so a later slot contradicting it is corruption | AC-15 | D-017 (easyfs1-refusal) |
| Build fails with a typed error — not a panic — when the entry count exceeds the directory or the image runs out of space, naming the program and the shortfall | — | D-015 (backfill) |
| An empty EasyFS2 directory (end marker in slot 0) loads as a cartridge with no programs, since the tool writes exactly that from an empty request; the reference tool refuses it by a bug the format spec classifies as an implementation artifact | — | D-015 (backfill) · D-007 |
| Every CHIP packet's marker and total-length field are validated on read, and a packet whose bank, geometry or length the format does not allow is refused as malformed at its file offset; the reference tool checks none of these | — | D-015 (backfill) · D-007 |
| The CRT header length field is honoured on read: the packet walk starts at the offset it states, a value below the fixed `0x40` header is refused as malformed, and one past the end of the file is a truncation. The writer still emits `0x40` | — | D-021 (crt-header-length) |
| Produced bytes match the frozen `ndeft` fixtures for every case the parity test covers, and each covered case's originating command line is asserted present in `fixtures/MANIFEST.md` (the converse is not checked — a manifest image with no case is not caught) | — | D-015 (backfill) · D-002 |
| Viewing a cartridge in the GUI never writes to it: the file is byte-identical and unmodified afterwards, opened or refused | AC-58 | D-030 (gui-shell) |
| A cart saved from the window after edits is byte-identical to what `edit` writes for the equivalent request on the same input, and a New cart saved after adding files is byte-identical to `build` of the same files in order — held against the real CLI over the fixtures | AC-80 | D-035 (gui-edit) |
| Booted, our menu shows the cart name and the visible entries in directory order (hidden ones absent, the first selected in reverse), scrolls past 16 rows on CRSR, starts the selection on RETURN — a program loaded with the cart switched out and started by `RUN` (load address ≤ `$0801`) or `SYS`, or any non-Ocean cartridge kind by bank, mode and reset — says so when nothing is visible, and leaves to BASIC with full memory on RUN/STOP. Proven by VICE boots, not the reference tool | AC-87..AC-92 | D-037 · D-038 (custom-menu) · D-041 (menu-details) |
| In our menu, typed characters (up to 16, not `"`) filter the list to visible entries whose name contains them, ignoring case. DEL removes one, CLR/HOME clears them, and a filter matching nothing says so. F1 toggles A-Z with stable ties (sorted on the first press) and back to directory order, keeping the text. The text shows in yellow on row 23 with the matches out of the visible entries; row 22 shows the selection's position among those listed and the order. Proven by VICE boots | AC-96..AC-101, AC-110 | D-039 (menu-search-sort) · D-041 (menu-details) |
| Our menu draws from a RAM copy of the ROM's mixed-case set at `$3800` carrying its own glyphs in codes `$60-$7F`, on a black screen: a white block-letter STIDCART logo from column 7 (rows 0-2) beside the cartridge mark in multicolour cells (columns 2-4, codes `$EA-$F2`: dark grey shell, yellow pins, a label of five three-line bands), the cart name (row 3), a raster strip of five one-scanline bands (light red, orange, yellow, light green, light blue; scanlines 84-88) across row 4, a dark grey rule (row 21), 16 list rows in columns 1-20 with a two-character badge per kind in its colour (PRG, 8K, 16K, ULT, XB) and the selection in an orange reverse bar, a scroll bar in column 21 whose thumb shows the share and place of the view, a pane describing the selection (type, size in KB, load address or first bank, and `RUN`, `SYS` and its address, or `reset`), and a grey reverse key-help bar (row 24), empty cart included. The strip and the mark come from a raster interrupt that replaces the CIA timer's and stays steady while the list redraws; no bank is switched while it can run from the cart's ROM, and every way out puts the IRQ vector back at `$EA31` with the raster interrupt off. A cursor move that doesn't scroll redraws only its two rows. A program it starts, and BASIC after RUN/STOP, get the C64's own colours and upper-case set, and nothing it starts finds SHIFT+C= locked. Proven by VICE boots and screenshots | AC-104..AC-111, AC-123..AC-128 | D-040 (menu-look) · D-041 (menu-details) · D-048 (menu-raster) |
| The menu embedded in carts is exactly what `menu/build.sh` assembles from `menu/src/` and fits the 8 KiB loader slot; a check reassembles and compares | AC-93 | D-037 (custom-menu) |
| The required CI gate builds the pinned 64tass and VICE and sets `STIDCART_REQUIRE_TOOLS=1`, so the boot and reassembly checks fail rather than skip when a tool is missing | AC-94 | D-038 (custom-menu) |

<!--
What this file is FOR (and what it is NOT):
  - FOR: the single place to learn "what does this system do now" when scoping new work, so you
    don't replay the chain of closed work-items. Read the relevant surface section before
    planning a change to that surface.
  - NOT a changelog (that's the append-only DECISIONS.md), NOT documentation (no prose, no
    examples), NOT a structural map (that's MAP.md — where things live).
Leanness rules:
  - One line per capability. If a capability needs a paragraph, the detail belongs in its D-NNN.
  - Rewrite a superseded capability in place; never keep a graveyard of old behaviour here.
  - Surface sections use the fixed vocabulary {CLI, HTTP, MCP, RENDER, DB, EVAL, none}; delete
    any section with no rows. HTTP, MCP, DB and EVAL are deleted here: no such surface exists, and
    EVAL is n/a because output is byte-exact images diffed against fixtures, never scored.
This file is NOT read on cold start (resume reads MAP + JOURNAL block + DECISIONS headers) — it
is on-demand, consulted when planning new work.
-->
