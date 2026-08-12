# JOURNAL — read-path

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the *entire*
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Deep
- **Goal:** Load a cartridge image back into the description that would produce it, and prove it by rebuilding the frozen reference images byte-identically.
- **Next action:** nothing — work-item CLOSED, PR open. Do NOT re-run the pipeline on this slug. Next work would be a CLI surface for reading, or entry editing; run `lcd:triage` for it.
- **Branch:** feat/read-path  ·  **Updated:** 2026-08-11 00:55

## STEPS
- [x] S1 — settle the divergence policy (→ D-007) and the version question (→ D-008), commit 43792b7
- [x] S2 — write spec.md (AC-1..AC-12; two branch points put to the maintainer)
- [x] S3 — write plan.md (decoders beside encoders; two spec amendments called out)
- [x] S4 — write tasks.md (T1..T25 + T-audit + T-final)
- [x] S5 — T1: version bump 0.2.0 → 0.3.0, first source-touching commit
- [x] S8 — T10..T21: one failing test per AC, all twelve `AC-N (none)` literals present, both targets red
- [x] S6 — T2..T7: decoders beside their encoders (directory, crt, reserved)
- [x] S7 — T8..T9: read.rs contract + lib.rs wiring
- [x] S9 — T22..T24: load implemented; 97 tests green, fmt and clippy clean
- [x] S10 — T25: round-trip closes on all four fixtures + the empty base case
- [x] S11 — T-audit PASS (vacuous — see audit.md), T-final green: 97 tests, fmt and clippy clean

## DECISIONS (this work-item)
- Divergence decided per behaviour from the format spec's own classification, not by a blanket
  bug-compatible rule (→ D-007)
- Crate stays at 0.2.0; the container header default keeps tracking the crate version (→ D-008)
- Hidden entries and mismatched reserved firmware are refused as unsupported, not loaded lossily
  (spec open questions 1 and 2, answered by the maintainer 2026-08-10)

## OPEN QUESTIONS
- none — all four spec questions resolved before planning

## EDIT BOUNDARY (paths this work may touch)
> From plan.md File structure. The red-green loop may modify only these. Keep every bullet a bare
> path — the boundary hook matches the whole bullet, so a trailing note like `(new)` makes the path
> unmatchable and denies the edit. The first three are new files; `Cargo.toml` is the version bump only.
- `crates/stidcart-core/src/read.rs`
- `crates/stidcart-core/tests/read.rs`
- `crates/stidcart-core/tests/roundtrip.rs`
- `crates/stidcart-core/src/crt.rs`
- `crates/stidcart-core/src/directory.rs`
- `crates/stidcart-core/src/reserved.rs`
- `crates/stidcart-core/src/lib.rs`
- `Cargo.toml`
- `docs/lcd/work/read-path`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/MAP.md`
<!-- /lcd-resume -->

---

## Acceptance criteria

See `spec.md` — AC-1 through AC-12. **Every AC is `surfaces: none`**: this work-item delivers a
library capability and deliberately exposes no command-line surface (see spec Out of scope). The
write-path audit recorded that the audit script BLOCKs without a cross-path matrix while the plan
template omits one for single-surface specs; the same degenerate-matrix workaround applies here, and
`plan.md` should carry it from the start rather than discovering it at the gate.

## LOG (append-only; not read on resume)

- 2026-08-10 23:30 — read CLAUDE.md, MAP, DECISIONS and the extracted format spec in the vault before
  taking any position, per the project's "format facts come from the spec" invariant.
- 2026-08-10 23:35 — pushed back on the proposed blanket bug-compatible read policy and it was not
  adopted. Two arguments carried it. Parity compares produced bytes and "rejected" has no bytes, so
  accept/reject was never under the parity oracle; and all four reference images are well-formed
  PRG-only carts that cannot trigger any of the quirks, so bug-compatibility buys no oracle strength
  on the tests this work-item writes. Recorded as D-007.
- 2026-08-10 23:36 — the decisive evidence was measured, not argued: a throwaway probe confirmed
  `CartBuilder::new().build()` succeeds and emits a 16480-byte cart with the end marker in directory
  slot 0 — exactly the empty directory the reference implementation rejects. Bug-compatibility would
  have meant the library could not read what it writes. Probe deleted; the finding became AC-4.
- 2026-08-10 23:38 — D-008 closes the version question write-path-skeleton left open since
  2026-08-10: 0.2.0 stays, default header name keeps deriving from the crate version. The read path
  never re-stamps it (AC-8), which is what would have made cross-version churn bite.
- 2026-08-10 23:40 — triaged Deep (5 signals, hard trigger on architecture and on public API
  irreversibility). Recon skipped for the same reason as the write path: standard-library-only work
  against a format already extracted to a written spec, no uncertain library territory.
- 2026-08-10 23:42 — spec.md written. Two material branch points went to the maintainer rather than
  being inferred — hidden entries and mismatched reserved firmware — both answered "refuse as
  unsupported", which is also what the divergence policy already does for cartridge entries and the
  older directory format. The other two open questions were resolvable from the format spec and the
  existing encoder, and are recorded with their reasoning.
- 2026-08-10 23:42 — encoding trap pinned as AC-9 before it could be hit: the container's
  producing-tool name is not in the machine's encoding, while entry names and the cartridge name are.
  A reader that applies the case swap to all three turns `ndefpack 1.0.1` into `NDEFPACK 1.0.1` and
  breaks parity on every reference image. D-006 exists because these fields were conflated once.

- 2026-08-11 00:10 — tasks.md written, T1..T25 + T-audit + T-final. Two orderings are deliberate and
  worth not "fixing" on a resume. T1 bumps the version first because `versioning.md` puts the bump in
  the branch's first code commit and the branch base was verified at 0.2.0. T8 lands the read contract
  (error types + a `load` signature that returns an error) *before* the AC tests, because in Rust a
  test referencing a function that does not exist is a build failure rather than a legible red — the
  stub is what makes every AC test fail for the right reason.
- 2026-08-11 00:12 — session interrupted mid-phase by a machine sleep. Nothing was lost: the last
  commit was the plan (732bb2c) and tasks.md had not been written, so the phase simply re-ran from
  its gate. Recorded because the recovery path (check the tracker and the last commit, not the
  transcript) is the reusable part.

- 2026-08-11 00:25 — twelve failing tests written, one per AC, split across `tests/read.rs` (the
  refusals) and `tests/roundtrip.rs` (the oracle). Both targets are red as a build error, which is
  the correct phase-4 red in Rust: the tests reference a module that does not exist yet, and T8's
  stub is what converts that into twelve legible assertion failures.
- 2026-08-11 00:26 — the boundary hook denied the first write of `tests/read.rs` even though the
  path was listed. Cause: the hook matches the whole EDIT BOUNDARY bullet, so the ` (new)`
  annotation made the path unmatchable. Bullets are now bare paths with the annotations moved into
  the note above them. Same class of matcher limitation the write-path item hit with `lcd-doctor`
  and prose inside a machine-block value — worth remembering that these blocks are parsed strictly.
- 2026-08-11 00:27 — T1 version bump 0.2.0 → 0.3.0 applied with this, the first source-touching
  commit on the branch (`versioning.md`: `feat/` → minor, branch base verified at 0.2.0). Checked
  first that nothing asserts the full string — `tests/container.rs:76` asserts only the
  `stidcart ` prefix, and parity pins the reference tool's name, so no fixture moves.
- 2026-08-11 00:28 — recorded for the audit phase: **this work-item's audit gate is vacuous.**
  `audit-crosspath.sh` skips every `none` surface, and all twelve ACs are `none`, so it will emit an
  empty table and exit 0 without checking anything. The real gates are T-final and the round-trip.
  The plan's matrix note originally claimed the matrix was kept to avoid a BLOCK; that was wrong —
  the write-path BLOCKed on its CLI rows — and has been corrected in `plan.md`.

- 2026-08-11 00:45 — red-green: green in 1 iteration, 0 reverts. All twelve ACs passed on the first
  full run, including the round-trip on all four fixtures and the empty base case. The only build
  failures were two mechanical ones (`DirEntry` needed `PartialEq`/`Eq` for `Slot` to derive them;
  the EF-Name magic was being compared against an expected image that never had it written).
- 2026-08-11 00:46 — a first-run pass is weak evidence, so the oracle was checked against a
  deliberate negative control rather than trusted: decoding the 32-byte container header name with
  the machine's case swap makes AC-8 and AC-9 fail immediately. That is the exact conflation D-006
  was written about, and it confirms those two criteria are load-bearing rather than vacuous. File
  restored from a scratchpad copy and the suite re-run green.
- 2026-08-11 00:47 — one design point worth keeping: entry extents are bounds-checked *before*
  reading, not during. Stepping past the last bank does not run off the end of the buffer — the
  interleave wraps the address back to bank 0 ROMH, which is the directory — so an over-long entry
  would have returned the directory as though it were program data. Silently wrong rather than
  merely broken, and the reason `check_extent` tracks bank and half instead of a flat offset.

## DEEP PIPELINE (Deep lane only)

> Phase tracker for the Deep-lane pipeline. Files live beside this JOURNAL.

`spec.md ✅  plan.md ✅  tasks.md ✅  tests ✅  red-green ✅  audit.md ✅`
<!-- mark ✅ done · ⏳ in progress · ⬜ not started -->

<!--
Resume contract (why the block above is fenced):
  /lcd:resume extracts ONLY the lcd-resume:v1 block. If NOW/STEPS are stale, resume
  rebuilds a false context — so update them in the same edit where you change the work.
  /lcd:resume also cross-checks `Updated` + `Branch` against `git log` and warns if this
  JOURNAL is older than HEAD before trusting it.
-->
