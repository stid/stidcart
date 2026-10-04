# JOURNAL — ocean-free-space

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the *entire*
> cold-start payload. Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Standard
- **Goal:** `list` reports the free-bytes figure `edit --add` actually accepts on Ocean-mode carts (issue #29), not just the top run.
- **Next action:** done — PR closing #29.
- **Branch:** fix/ocean-free-space  ·  **Updated:** 2026-10-04 21:40

## STEPS
- [x] S1 — failing tests: cocean512 lists `$07a000 (499712)` with exact-fit / one-more-fails; core unit test expects 61 ROMH banks on an Ocean 512k + small program
- [x] S2 — `CartBuilder::free_space()`: search the placer with a probe program appended last; `lo` always fits, `hi` always fails
- [x] S3 — `list` uses the builder figure when `load` succeeds, the replay figure otherwise; `render` takes the figure
- [x] S4 — D-027, SPEC RENDER row, `CartInfo::free_space` doc comment; gate; PR closing #29

## DECISIONS (this work-item)
- Figure = boundary of a search over `Placer::place_all` with a probe, not `build_image` (entry-count refusals would turn "directory full" into "0 bytes free") → D-027
- Placement is greedy, so fit is not proven monotone in size: the figure is a local boundary (fits, +1 fails), not a proven global maximum → D-027

## OPEN QUESTIONS
- none

## EDIT BOUNDARY (paths this work may touch)
- `crates/stidcart-core/src/lib.rs`
- `crates/stidcart-core/src/read.rs`
- `crates/stidcart-cli/src/main.rs`
- `crates/stidcart-cli/tests/cartlist.rs`
- `docs/lcd/DECISIONS.md`, `docs/lcd/SPEC.md`, `docs/lcd/work/ocean-free-space/JOURNAL.md`
<!-- /lcd-resume -->

---

## Acceptance criteria

Re-tightens an existing AC rather than adding one: **AC-46** (surfaces: RENDER), as folded into
`SPEC.md` — the free-bytes figure is a size a further entry could actually be placed at. This
work-item extends the exact-fit / one-more-fails assertion from `c8k`/`cmixed` to `cocean512`.

## LOG (append-only; not read on resume)

- 2026-10-04 21:00 — Triage: Standard (2 signals). Issue #29. Measured on main @ 628ad36: cocean512
  accepts a $07a000-byte program, i.e. ROMH banks 2..62 (61 × $2000); `hello` moves to the top.
  The replay figure (`CartInfo::free_space`) can't reach that number because `edit` re-places
  every entry (D-019), so the figure has to come from a fresh placement.
- 2026-10-04 21:20 — S1–S3: tests red on the $2000 figure, then `CartBuilder::free_space` (probe
  search over `place_all`) and `list` routing it. c8k/cmixed literals held unchanged; cocean512 now
  lists $07a000. Committed 02238a4. Decision → D-027.
- 2026-10-04 21:40 — S4: D-027, SPEC RENDER row, `CartInfo::free_space` doc. Dropped a stray
  `AC-46 (none)` token from the core test (AC-46 is RENDER only). audit: PASS (test-presence).
- 2026-10-04 21:45 — reconciled: folded AC-46 (re-tightened) → SPEC.md/RENDER (free-bytes row rewritten in place, D-027).
