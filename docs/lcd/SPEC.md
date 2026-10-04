# SPEC — stidcart (living current-state)

> **What this system DOES now — a capability index, not documentation.** One line per
> capability; the detail lives in the cited `D-NNN` (history) and, until pruned, the work-item
> delta. **Current-state only**: a replaced capability is rewritten in place — its history is
> kept in `DECISIONS.md`, never here. Folded at closeout by `lcd:reconcile`. Read only the
> section for the surface/area you're about to touch, not the whole file. Opt-in: this file
> exists only when `living-spec: on` in `.claude/rules/lcd-conventions.md`.

**Last compacted:** 2026-10-04 (cart-entries)

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
| `build` writes a `.crt` from PRG inputs and cartridge images, recognising a cartridge by its content; `-e` sets the menu name, `-o` the output path, `--hide INPUT` marks that input hidden, `--align INPUT` requests 64 KB alignment for an xbank, and each entry name is the input's file stem with case and spacing preserved. A build of programs only, marking nothing hidden, produces the bytes it produced before either capability existed | AC-28, AC-40, AC-42 | D-020 (cart-edit) · D-026 (cart-entries) |
| A `build` request naming no input programs succeeds and writes nothing, matching the reference tool rather than erroring | — | D-015 (backfill) · D-002 |
| `list` prints what a `.crt` contains, with positions taken from the image as stored rather than from a rebuild that would re-place every entry | — | D-015 (backfill) · D-011 |
| Failures print `stidcart: <reason>` on stderr and exit non-zero; an unknown command or option prints usage | — | D-015 (backfill) |
| `list` refuses an old-format (EasyFS1) cartridge, naming the format on stderr, rather than printing entries decoded under the wrong layout | AC-13 | D-017 (easyfs1-refusal) |
| `edit` changes an existing cartridge: `--add` appends a program or cartridge image (`--add-aligned` with 64 KB alignment), `--remove` / `--rename` / `--replace` / `--hide` / `--show` / `--align` / `--unalign` act on the entry at the index `list` prints whatever its kind, `--replace` takes the kind of the new data, and `-e` / `--header` set the menu and container names independently | AC-17..AC-22, AC-41..AC-43 | D-019 (cart-edit) · D-026 (cart-entries) |
| An edit writes over the cartridge it was given unless `-o` names a destination, in which case the input is left byte-identical | AC-23 | D-019 (cart-edit) |
| Every index in one invocation refers to the cartridge as given, not to the state a previous operation in the same invocation left | AC-27 | D-019 (cart-edit) |
| A failed edit — a bad index, a set that does not fit, an unreadable program — reports the reason, exits non-zero and leaves every file it was given byte-identical, with no partial cartridge left behind | AC-24 | D-019 (cart-edit) |
| `edit` refuses what it could not rebuild unchanged, naming the reason: an EasyFlash subdirectory entry (the one kind still unmodelled), firmware regions differing from the embedded blobs, or an old-format (EasyFS1) directory | AC-25, AC-26, AC-43 | D-019 (cart-edit) · D-011, D-017 · D-026 (cart-entries) |
| A cartridge image `build` or `edit --add` cannot represent — an EasyFlash multicart, a subdirectory, an unlisted id — is refused naming what it is, exit non-zero, every file left byte-identical; alignment on a non-xbank entry is refused naming the entry | AC-42, AC-44 | D-026 (cart-entries) |

### RENDER

| Capability (current behaviour, one line) | ACs | Provenance |
|---|---|---|
| The listing shows the cart name and producing tool, then one row per entry — index, name, flags (hidden, 64 KB aligned), kind label, bank, layout (a program's offset, or a cartridge's bank runs per half), size — closing with the entry count and the free bytes: the contiguous run at the top. In normal mode a program of exactly that size can be added and one byte more cannot; in Ocean mode it is a lower bound, since a larger ROMH-confined program can still fit a gap (follow-up) | AC-45, AC-46 | D-026 (cart-entries) |
| Every kind the walk accepts — nine cartridge kinds, interleaved and both half-confined program kinds — has one distinct label, so no entry renders as `?` | AC-45 | D-026 (cart-entries) |
| Control characters in the container header name are replaced with `.` before printing, because a `.crt` is untrusted input and the field is free-form host text | — | D-015 (backfill) |

### Internal (surface: none)

> Invariants, calculations, and behaviours with no external surface — including the one-line
> capability `lcd:reconcile` folds from a surface-less work-item's `Goal`.

| Capability (current behaviour, one line) | ACs | Provenance |
|---|---|---|
| The whole library API is one owned value: `CartBuilder` accumulates entries (programs and cartridges), mutates them in place by index, and returns bytes, touching no filesystem and holding no global state. The index-addressed mutators panic on an out-of-range index, as `Vec` indexing does | — | D-015 (backfill) · D-001, D-019, D-023 |
| An edited cartridge is byte-for-byte a fresh build of the entry set the edits produced: saving runs the one placement path, so entries the edit never mentioned may land at different positions than the input had | AC-30 | D-019 (cart-edit) |
| Placement puts every cartridge before any program (Ocean first, then 64 KB-aligned ones stepping eight banks, then by height, first fit in both halves), then programs largest first: one fitting a gap below the top bank goes there confined to one half, the rest interleaved at the top; the directory keeps the caller's input order | AC-33, AC-34 | D-026 (cart-entries) · D-025 |
| Every build rewrites bank 0's reserved regions from the zlib blobs embedded as-is, plus the `ef-nAME:` magic and the caller's cartridge name. The mode is derived from the entries: an Ocean entry owns bank 0 ROML and switches to the Ocean-mode boot and loader (loader in bank 1 ROMH); a second Ocean entry is a typed build error naming it | AC-36, AC-37 | D-005 · D-024 (cart-entries) |
| The CRT container header name and the EF-Name are separate caller-supplied fields at different offsets; the header defaults to `stidcart <crate-version>` | — | D-015 (backfill) · D-006, D-008 |
| Names cross the PETSCII boundary in both directions | — | D-015 (backfill) |
| `inspect` describes any image it can walk; `load` layers the rebuild preconditions on top and refuses what it could not reproduce byte-identically — firmware regions differing from the blobs the derived mode implies (normal or Ocean), and subdirectory entries | AC-36, AC-43 | D-011, D-007 · D-026 (cart-entries) |
| The hidden flag is part of the entry model on both sides, so a cartridge carrying hidden entries loads, edits and saves back with names, order, flags and program data intact — it was refused for rebuilding only while the description had nowhere to keep the flag | AC-29 | D-020 (cart-edit) · D-011 |
| Both read paths describe and load every cartridge kind and half-confined programs (same-half, next-bank continuation); a load-and-save of a reference fixture holding them is byte-identical. Only subdirectory entries are refused | AC-35, AC-43 | D-026 (cart-entries) |
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
