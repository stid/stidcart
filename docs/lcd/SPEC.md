# SPEC — stidcart (living current-state)

> **What this system DOES now — a capability index, not documentation.** One line per
> capability; the detail lives in the cited `D-NNN` (history) and, until pruned, the work-item
> delta. **Current-state only**: a replaced capability is rewritten in place — its history is
> kept in `DECISIONS.md`, never here. Folded at closeout by `lcd:reconcile`. Read only the
> section for the surface/area you're about to touch, not the whole file. Opt-in: this file
> exists only when `living-spec: on` in `.claude/rules/lcd-conventions.md`.

**Last compacted:** 2026-08-12

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
| `build` writes a `.crt` from one or more PRG inputs; `-e` sets the menu name, `-o` the output path, and each entry name is the input's file stem with case and spacing preserved | — | D-015 (backfill) |
| A `build` request naming no input programs succeeds and writes nothing, matching the reference tool rather than erroring | — | D-015 (backfill) · D-002 |
| `list` prints what a `.crt` contains, with positions taken from the image as stored rather than from a rebuild that would re-place every entry | — | D-015 (backfill) · D-011 |
| Failures print `stidcart: <reason>` on stderr and exit non-zero; an unknown command or option prints usage | — | D-015 (backfill) |

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
| The whole library API is one owned value: `CartBuilder` accumulates programs and returns bytes, touching no filesystem and holding no global state | — | D-015 (backfill) · D-001 |
| Placement fills largest-first and interleaved across banks, while the directory keeps the caller's input order | — | D-015 (backfill) |
| Every build rewrites bank 0's reserved regions: EAPI, boot and the menu loader from the zlib blobs embedded as-is (normal-mode variants only — the OCM blobs are embedded but unused here), plus the `ef-nAME:` magic and the caller's cartridge name, which are not blobs | — | D-015 (backfill) · D-005 |
| The CRT container header name and the EF-Name are separate caller-supplied fields at different offsets; the header defaults to `stidcart <crate-version>` | — | D-015 (backfill) · D-006, D-008 |
| Names cross the PETSCII boundary in both directions | — | D-015 (backfill) |
| `inspect` describes any image it can walk; `load` layers the rebuild preconditions on top and refuses what it could not reproduce byte-identically, including hidden entries and mismatched firmware regions | — | D-015 (backfill) · D-011, D-007 |
| Both read paths refuse what the builder cannot represent: cartridge entries (bit 4 of the type value) and entries confined to one half of a bank (types `0x02`/`0x03`) | — | D-015 (backfill) |
| Old-format (EasyFS1) directories are **not detected**: every directory is walked as EasyFS2, so such an image is misread or fails incidentally, never refused for being old | — | D-015 (backfill) · D-007 |
| Build fails with a typed error — not a panic — when the entry count exceeds the directory or the image runs out of space, naming the program and the shortfall | — | D-015 (backfill) |
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
