# SPEC — stidcart (living current-state)

> **What this system DOES now — a capability index, not documentation.** One line per
> capability; the detail lives in the cited `D-NNN` (history) and, until pruned, the work-item
> delta. **Current-state only**: a replaced capability is rewritten in place — its history is
> kept in `DECISIONS.md`, never here. Folded at closeout by `lcd:reconcile`. Read only the
> section for the surface/area you're about to touch, not the whole file. Opt-in: this file
> exists only when `living-spec: on` in `.claude/rules/lcd-conventions.md`.

**Last compacted:** 2026-08-14 (cart-edit)

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
| `build` writes a `.crt` from one or more PRG inputs; `-e` sets the menu name, `-o` the output path, `--hide INPUT` marks that input hidden, and each entry name is the input's file stem with case and spacing preserved. A build marking nothing hidden produces the bytes it produced before the marker existed | AC-28 | D-015 (backfill) · D-020 (cart-edit) |
| A `build` request naming no input programs succeeds and writes nothing, matching the reference tool rather than erroring | — | D-015 (backfill) · D-002 |
| `list` prints what a `.crt` contains, with positions taken from the image as stored rather than from a rebuild that would re-place every entry | — | D-015 (backfill) · D-011 |
| Failures print `stidcart: <reason>` on stderr and exit non-zero; an unknown command or option prints usage | — | D-015 (backfill) |
| `list` refuses an old-format (EasyFS1) cartridge, naming the format on stderr, rather than printing entries decoded under the wrong layout | AC-13 | D-017 (easyfs1-refusal) |
| `edit` changes an existing cartridge: `--add` appends a program, `--remove` / `--rename` / `--replace` / `--hide` / `--show` act on the entry at the index `list` prints, and `-e` / `--header` set the menu and container names independently | AC-17..AC-22 | D-019 (cart-edit) |
| An edit writes over the cartridge it was given unless `-o` names a destination, in which case the input is left byte-identical | AC-23 | D-019 (cart-edit) |
| Every index in one invocation refers to the cartridge as given, not to the state a previous operation in the same invocation left | AC-27 | D-019 (cart-edit) |
| A failed edit — a bad index, a set that does not fit, an unreadable program — reports the reason, exits non-zero and leaves every file it was given byte-identical, with no partial cartridge left behind | AC-24 | D-019 (cart-edit) |
| `edit` refuses what it could not rebuild unchanged, naming the reason: an entry kind it cannot represent, firmware regions differing from the embedded blobs, or an old-format (EasyFS1) directory | AC-25, AC-26 | D-019 (cart-edit) · D-011, D-017 |

### RENDER

| Capability (current behaviour, one line) | ACs | Provenance |
|---|---|---|
| The listing shows the cart name and producing tool, then one row per entry — index, name, hidden flag, type, bank, offset, size — closing with the entry count and free bytes | — | D-015 (backfill) |
| Entry kinds the directory walk refuses render as `?` rather than a label, so the output never advertises support no code path has | — | D-015 (backfill) |
| Control characters in the container header name are replaced with `.` before printing, because a `.crt` is untrusted input and the field is free-form host text | — | D-015 (backfill) |

### Internal (surface: none)

> Invariants, calculations, and behaviours with no external surface — including the one-line
> capability `lcd:reconcile` folds from a surface-less work-item's `Goal`.

| Capability (current behaviour, one line) | ACs | Provenance |
|---|---|---|
| The whole library API is one owned value: `CartBuilder` accumulates programs, mutates them in place by index, and returns bytes, touching no filesystem and holding no global state. The index-addressed mutators panic on an out-of-range index, as `Vec` indexing does | — | D-015 (backfill) · D-001, D-019 |
| An edited cartridge is byte-for-byte a fresh build of the entry set the edits produced: saving runs the one placement path, so entries the edit never mentioned may land at different positions than the input had | AC-30 | D-019 (cart-edit) |
| Placement fills largest-first and interleaved across banks, while the directory keeps the caller's input order | — | D-015 (backfill) |
| Every build rewrites bank 0's reserved regions: EAPI, boot and the menu loader from the zlib blobs embedded as-is (normal-mode variants only — the OCM blobs are embedded but unused here), plus the `ef-nAME:` magic and the caller's cartridge name, which are not blobs | — | D-015 (backfill) · D-005 |
| The CRT container header name and the EF-Name are separate caller-supplied fields at different offsets; the header defaults to `stidcart <crate-version>` | — | D-015 (backfill) · D-006, D-008 |
| Names cross the PETSCII boundary in both directions | — | D-015 (backfill) |
| `inspect` describes any image it can walk; `load` layers the rebuild preconditions on top and refuses what it could not reproduce byte-identically — firmware regions differing from the embedded blobs, and entry kinds the builder cannot express | — | D-015 (backfill) · D-011, D-007 |
| The hidden flag is part of the entry model on both sides, so a cartridge carrying hidden entries loads, edits and saves back with names, order, flags and program data intact — it was refused for rebuilding only while the description had nowhere to keep the flag | AC-29 | D-020 (cart-edit) · D-011 |
| Both read paths refuse what the builder cannot represent: cartridge entries (bit 4 of the type value) and entries confined to one half of a bank (types `0x02`/`0x03`) | — | D-015 (backfill) |
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
