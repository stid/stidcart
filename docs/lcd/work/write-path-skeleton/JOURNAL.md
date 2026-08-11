# JOURNAL — write-path-skeleton

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the *entire*
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Deep
- **Goal:** Build an EasyFlash cartridge image from program files, byte-identical to the reference tool's output for the same inputs.
- **Next action:** run /lcd:plan write-path-skeleton
- **Branch:** feat/write-path-skeleton  ·  **Updated:** 2026-08-10 22:45

## STEPS
- [x] S1 — scaffold workspace, LCD spine, firmware blobs (committed 6fbfbd6)
- [ ] S2 — write plan.md  ← next
- [ ] S3 — write tasks.md
- [ ] S4 — generate failing tests, one per (AC × surface)
- [ ] S5 — red-green loop to green
- [ ] S6 — capture reference fixtures + their exact request lines
- [ ] S7 — boot a produced image on the emulator
- [ ] S8 — audit, then PR

## DECISIONS (this work-item)
- Write path before read path; the read path's divergence policy stays open (→ D-003)
- Reference tool used as a byte oracle, frozen as fixtures so it survives (→ D-002)
- All four spec open questions resolved by measuring the reference tool, not by assumption

## OPEN QUESTIONS
- none

## EDIT BOUNDARY (paths this work may touch)
- `crates/stidcart-core`
- `crates/stidcart-cli`
- `fixtures`
- `docs/lcd/work/write-path-skeleton`
- `docs/lcd/DECISIONS.md`
- `Cargo.toml`
<!-- /lcd-resume -->

---

## Acceptance criteria

See `spec.md` — AC-1 through AC-11.

## LOG (append-only; not read on resume)

- 2026-08-10 22:33 — scaffolded the workspace: two crates, firmware blobs copied unmodified and kept zlib, GPL for the rest. Committed 6fbfbd6.
- 2026-08-10 22:40 — LCD onboarded (docs/lcd, CI gate on, living-spec off). lcd-doctor 0 FAIL 0 WARN after making the machine-block values bare patterns; the doctor's matcher cannot parse prose inside a value.
- 2026-08-10 22:42 — triaged Deep (4 signals, hard trigger on architecture). Recon skipped deliberately: standard-library-only work against a format already extracted to a written spec, no uncertain library territory.
- 2026-08-10 22:44 — measured the reference tool to close all four spec open questions rather than guessing: name derivation strips directory and extension and preserves case; a placeholder cartridge name is always stored when none is given; duplicate entry names are accepted; an empty input list exits successfully and writes no file.

## DEEP PIPELINE (Deep lane only)

> Phase tracker for the Deep-lane pipeline. Files live beside this JOURNAL.

`spec.md ✅  plan.md ⬜  tasks.md ⬜  tests ⬜  red-green ⬜  audit.md ⬜`
<!-- mark ✅ done · ⏳ in progress · ⬜ not started -->
