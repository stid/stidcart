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
- **Next action:** run lcd:redgreen-loop (first red: `cargo test --workspace --test cartridge`, T4–T7)
- **Branch:** feat/cart-entries  ·  **Updated:** 2026-09-19 14:10

## STEPS
- [x] S1 — triage → Deep, spec.md (AC-31..AC-46), approved; D-022
- [x] S2 — plan.md; D-023, D-024
- [x] S3 — tasks.md (T1..T34 + audit/final)
- [x] S4 — T1–T3 foundation: 0.6.0, make_inputs.py + 10 inputs, 11 ndeft images + MANIFEST
- [x] S5 — test-gen: 16 failing tests across cartridge/cart_placement/ocean (core) and carts/cartlist (cli)
- [ ] S6 — T4–T7 directory: cart kinds, CrtUsage, geometry (AC-31, AC-32)  ← next
- [ ] S7 — T8–T10 cartridge.rs from_crt + crt::packets (AC-39)
- [ ] S8 — T11–T16 placement + Entry model + image helpers (AC-33, AC-34, AC-38)
- [ ] S9 — T17–T18 read path describes/loads every kind (AC-35)
- [ ] S10 — T19–T22 Ocean mode (AC-36, AC-37)
- [ ] S11 — T23–T24 parity + roundtrip on new fixtures
- [ ] S12 — T25–T30 CLI (AC-40..AC-44)
- [ ] S13 — T31–T33 RENDER (AC-45, AC-46)
- [ ] S14 — T34 README; T-audit; T-final gate + reconcile; PR

## DECISIONS (this work-item)
- recon skipped: the format is fully specified in `docs/format/easyflash-cart-format.md` and no external library is involved
- refine (test-gen): the nine cart kinds join `EntryType` (it already carries the type values) instead of a separate `CartKind`, so `DirEntry.kind` keeps its type and every existing test compiles; `Cartridge.kind: EntryType` is a cart variant by construction of `from_crt`. A dedicated `ReadError::BadGeometry { slot }` names the slot for AC-32. `push`/`with_entry`/`replace` take `impl Into<Entry>` so `push(Program)` keeps compiling.
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

- 2026-09-19 14:10 — test-gen: 16 red tests, one per AC (none) / (AC × surface). Four core/cli targets fail to compile against the planned API (`cartridge` module, `Entry`, `with_cartridge`, `entries`, `EntryType` cart variants, `DirEntry` CrtUsage fields, `reserved::Mode`/`LOADER_OCM_OFFSET`, `BuildError::SecondOcean`, `ReadError::BadGeometry`); `cartlist` runs and fails on the cart-entry refusal.
- 2026-09-19 13:45 — T1–T3 done. `ndeft list` of every new image (Type / Bank / Offs / Size), input order preserved in every directory:
  - c8k: hello PRG $01 $2000 $66 · in_8k 8k $01 $0000 $2000 — **gap search stops below the top bank**, so bank 1 ROMH is top space and hello is interleaved there, not confined (T12 corrected)
  - c16k: ramp PRG $02 $0000 · in_16k 16k $01 $4000
  - cult8: hello PRG(ROML) $01 $0000 · in_ult8 Ultimax 8k $01 $2000
  - cult16: hello PRG $02 $0000 · in_ult16 Ultimax 16k $01 $4000
  - cocean (mode Ocean, loader bank 1 ROMH): big PRG(ROML) $10 $0000 · in_ocean Ocean $00 size $40000, ROMH run at +16 (banks 16–31)
  - cocean512 (mode Ocean): hello PRG(ROMH) $02 $2000 · in_ocean512 Ocean 512k $00 size $80000 (all 64 ROML banks)
  - cxb16: hello PRG $05 $0000 · in_xb16 xbank 16k $01 size $10000
  - cxb16a: aligned xbank 16k $08 (A flag) · hello PRG(ROML) $01 $0bba · ramp PRG(ROML) $01 $0000
  - cxbult: hello PRG $03 $0000 · in_xbult xbank Ult. $01 size $8000
  - cmixed: in_8k 8k $07 · in_16k 16k $05 · in_ult16 Ultimax 16k $06 · in_xb8 xbank 8k $01 (size $8000) · hello PRG(ROMH) $03 $39dc · ramp PRG(ROMH) $03 $2e22 · big PRG(ROMH) $01 $2000 — weight ties (16k vs ult16) keep input order; find_free_gap's bank cursor persists across rounds
  - cxbwide: written by ndeft (681328 bytes), `ndeft list` refuses it: `size $0a0000 > $80000` (D-022 write-side pin)
- 2026-09-19 13:20 — tasks.md: 34 tasks + audit/final, fixtures first (T2/T3) since every placement AC is pinned by them.
- 2026-09-19 13:00 — plan.md written: Entry enum + two-run Cartridge model, reference placement reproduced in the one Placer, Ocean as derived mode, CLI grammar, 10 new ndeft fixtures with a checked-in input generator. Decisions → D-023, D-024.
- 2026-09-19 12:30 — spec approved: Ocean in, xbank bounded by flash geometry (→ D-022), alignment exposed on the CLI, replace may swap kinds, cartridge inputs recognised by content.
- 2026-09-19 12:00 — triage: Deep (5 signals, hard: cross-cutting model + public API + CLI/RENDER; risk: multi-session). Recon skipped (in-tree format spec is the source). spec.md drafted with five open questions, each carrying a recommendation.

## DEEP PIPELINE (Deep lane only)

> Phase tracker for the Deep-lane pipeline. Files live beside this JOURNAL.

`spec.md ✅  plan.md ✅  tasks.md ✅  tests ✅  red-green ⬜  audit.md ⬜`
<!-- mark ✅ done · ⏳ in progress · ⬜ not started -->
