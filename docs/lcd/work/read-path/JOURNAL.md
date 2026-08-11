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
- **Next action:** run `/lcd:plan read-path`
- **Branch:** feat/read-path  ·  **Updated:** 2026-08-10 23:42

## STEPS
- [x] S1 — settle the divergence policy (→ D-007) and the version question (→ D-008), commit 43792b7
- [x] S2 — write spec.md (AC-1..AC-12; two branch points put to the maintainer)
- [ ] S3 — write plan.md  ← next
- [ ] S4 — write tasks.md
- [ ] S5 — generate failing tests, one per AC
- [ ] S6 — red-green loop to green
- [ ] S7 — audit gate, then PR

## DECISIONS (this work-item)
- Divergence decided per behaviour from the format spec's own classification, not by a blanket
  bug-compatible rule (→ D-007)
- Crate stays at 0.2.0; the container header default keeps tracking the crate version (→ D-008)
- Hidden entries and mismatched reserved firmware are refused as unsupported, not loaded lossily
  (spec open questions 1 and 2, answered by the maintainer 2026-08-10)

## OPEN QUESTIONS
- none — all four spec questions resolved before planning

## EDIT BOUNDARY (paths this work may touch)
- `crates/stidcart-core/src`
- `crates/stidcart-core/tests`
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

## DEEP PIPELINE (Deep lane only)

> Phase tracker for the Deep-lane pipeline. Files live beside this JOURNAL.

`spec.md ✅  plan.md ⬜  tasks.md ⬜  tests ⬜  red-green ⬜  audit.md ⬜`
<!-- mark ✅ done · ⏳ in progress · ⬜ not started -->

<!--
Resume contract (why the block above is fenced):
  /lcd:resume extracts ONLY the lcd-resume:v1 block. If NOW/STEPS are stale, resume
  rebuilds a false context — so update them in the same edit where you change the work.
  /lcd:resume also cross-checks `Updated` + `Branch` against `git log` and warns if this
  JOURNAL is older than HEAD before trusting it.
-->
