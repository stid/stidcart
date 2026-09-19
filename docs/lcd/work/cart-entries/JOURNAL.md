# JOURNAL — cart-entries

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the *entire*
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Deep
- **Goal:** Model, place, build, read, list and edit the nine cartridge entry kinds (8k/16k, Ultimax, Ocean, xbank) plus the half-confined program entries the reference placement produces beside them; decide the wide-xbank bound.
- **Next action:** run /lcd:tasks cart-entries
- **Branch:** feat/cart-entries  ·  **Updated:** 2026-09-19 13:00

## STEPS
- [x] S1 — triage → Deep, spec.md drafted (AC-31..AC-46)
- [x] S2 — spec approved 2026-09-19: all five open questions on their recommendation (xbank bound → D-022)
- [x] S3 — /lcd:plan (D-023 entry model + derivation, D-024 Ocean mode derived)
- [ ] S4 — /lcd:tasks  ← next
- [ ] S5 — /lcd:test-gen
- [ ] S6 — red-green
- [ ] S7 — /lcd:audit, PR

## DECISIONS (this work-item)
- recon skipped: the format is fully specified in `docs/format/easyflash-cart-format.md` and no external library is involved
- ACs numbered from AC-31 (next free per lcd-conventions); next free is now AC-47

## OPEN QUESTIONS
- none

## EDIT BOUNDARY (paths this work may touch)
- `Cargo.toml` (version bump)
- `crates/stidcart-core/src/{lib,cartridge,directory,place,read,reserved,crt,image}.rs`
- `crates/stidcart-core/tests/{cartridge,cart_placement,ocean}.rs` (new), `{read,parity,roundtrip}.rs`
- `crates/stidcart-cli/src/main.rs`; `crates/stidcart-cli/tests/{carts,cartlist}.rs` (new), `{list,edit}.rs`
- `fixtures/` (make_inputs.py, in_*.crt inputs, c*.crt reference images, MANIFEST.md)
- `docs/format/easyflash-cart-format.md` (only if a format fact needs correcting — first, per D-018)
- `docs/lcd/` (this work-item, DECISIONS.md, SPEC.md at closeout), `README.md`
<!-- /lcd-resume -->

---

## Acceptance criteria

See `spec.md` (AC-31..AC-46).

## LOG (append-only; not read on resume)

- 2026-09-19 13:00 — plan.md written: Entry enum + two-run Cartridge model, reference placement reproduced in the one Placer, Ocean as derived mode, CLI grammar, 10 new ndeft fixtures with a checked-in input generator. Decisions → D-023, D-024.
- 2026-09-19 12:30 — spec approved: Ocean in, xbank bounded by flash geometry (→ D-022), alignment exposed on the CLI, replace may swap kinds, cartridge inputs recognised by content.
- 2026-09-19 12:00 — triage: Deep (5 signals, hard: cross-cutting model + public API + CLI/RENDER; risk: multi-session). Recon skipped (in-tree format spec is the source). spec.md drafted with five open questions, each carrying a recommendation.

## DEEP PIPELINE (Deep lane only)

> Phase tracker for the Deep-lane pipeline. Files live beside this JOURNAL.

`spec.md ✅  plan.md ✅  tasks.md ⬜  tests ⬜  red-green ⬜  audit.md ⬜`
<!-- mark ✅ done · ⏳ in progress · ⬜ not started -->
