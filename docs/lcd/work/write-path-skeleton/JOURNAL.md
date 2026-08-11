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
- **Next action:** nothing — this work-item is CLOSED. PR #1 merged 2026-08-11 (0fd28be); main is green. Do NOT re-run the pipeline on this slug. The next work-item starts by deciding the read-path divergence policy (D-003); run `lcd:triage` for it.
- **Branch:** feat/write-path-skeleton  ·  **Updated:** 2026-08-11 06:25

## STEPS
- [x] S1 — scaffold workspace, LCD spine, firmware blobs (committed 6fbfbd6)
- [x] S2 — write plan.md
- [x] S3 — write tasks.md (T1..T24 + T-audit + T-final)
- [x] S4 — T1..T4: petscii and image
- [x] S5 — T5..T8: firmware blobs and the bank-0 reserved map
- [x] S6 — T9..T11: directory entry encoding
- [x] S7 — T12..T13: placement
- [x] S8 — T14..T16: CRT container, then wire CartBuilder
- [x] S9 — T17..T19: fixtures captured, parity green on all four cases
- [x] S10 — T20..T23: CLI build command
- [x] S11 — T24: booted under x64sc, menu renders both entries
- [x] S12 — T-audit PASS (run 2), T-final green

## DECISIONS (this work-item)
- Write path before read path; the read path's divergence policy stays open (→ D-003)
- Reference tool used as a byte oracle, frozen as fixtures so it survives (→ D-002)
- All four spec open questions resolved by measuring the reference tool, not by assumption

## OPEN QUESTIONS
- Version is `0.2.0`, not `0.1.0`: `versioning.md` bumps minor on a `feat/` branch and the scaffold
  commit had already claimed `0.1.0`. That string is the default CRT header name, so it is inside
  every cart built without an explicit header name, and the test only asserts the `stidcart ` prefix.
  Raised with the user 2026-08-10, still unanswered, and now merged to main at `0.2.0`. Changing it
  is a one-line `Cargo.toml` edit, but it alters the bytes of every cart built without an explicit
  header name — so it needs its own commit saying so, not a drive-by.

## EDIT BOUNDARY (paths this work may touch)
- `crates/stidcart-core/src`
- `crates/stidcart-core/tests`
- `crates/stidcart-cli/src`
- `crates/stidcart-cli/tests`
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

- 2026-08-10 22:56 — byte parity green on all four fixture cases on the FIRST run: single, named, two, spill. The three details that would each have broken it were measured beforehand rather than assumed (boot-region erased fill, always-written cartridge name, CHIP total-length field).
- 2026-08-10 22:57 — full gate green: 63 tests, `cargo fmt --check` clean, `cargo clippy -D warnings` clean.
- 2026-08-10 23:00 — T24 (AC-4): a stidcart-built cart booted under x64sc; the EasyLoader menu rendered both entries with correct sizes (hello 102B, ramp 2.9K). Screenshot kept as `boot-evidence.png`. This is the evidence that does not depend on the oracle.
- 2026-08-10 23:00 — deviation worth recording: `petscii.rs` was written test-and-implementation together rather than test-first, being a pure case-swap whose tests come from measured reference bytes. Every other module had its AC tests written and committed to before implementation.

- 2026-08-10 23:02 — audit run 1 BLOCKED on all six CLI rows, for two artifact reasons rather than missing work: the plan template says single-surface specs omit the cross-path matrix while the audit script BLOCKs without one, and Rust test names cannot contain the literal `AC-N (CLI)`. Fixed with a degenerate matrix and doc-comment literals; run 2 PASS. Written up in audit.md because the next Deep item here will hit the same contradiction.
- 2026-08-10 23:02 — the AC-4 boot test was rewriting its own carts: x64sc defaults to -easyflashcrtwrite and saves emulated flash back over the image, which showed up as a 64-byte broken.crt after the negative control. Added +easyflashcrtwrite. Harmless in a scratch dir, but pointed at a fixture it would have silently corrupted it and broken parity with no obvious cause.
- 2026-08-10 23:03 — T-final green: 66 tests, fmt clean, clippy clean at -D warnings.

- 2026-08-11 06:22 — PR #1 merged (0fd28be). Gate re-run on merged main: 66 tests, fmt and clippy clean. Work-item closed.

## DEEP PIPELINE (Deep lane only)

> Phase tracker for the Deep-lane pipeline. Files live beside this JOURNAL.

`spec.md ✅  plan.md ✅  tasks.md ✅  tests ✅  red-green ✅  audit.md ✅`
<!-- mark ✅ done · ⏳ in progress · ⬜ not started -->
