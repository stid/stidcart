# JOURNAL — custom-menu

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the *entire*
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Deep
- **Goal:** Our own C64 menu, first slice: it lists a cart's entries and boots them, is chosen opt-in at build/edit, and load/edit/GUI recognise it (D-037)
- **Next action:** user approves spec.md, then run /lcd:plan custom-menu (fold the recon digest from LOG into the plan)
- **Branch:** feat/custom-menu  ·  **Updated:** 2026-11-07

## STEPS
- [x] S1 — triage (Deep), D-037, spec.md
- [ ] S2 — /lcd:plan custom-menu  ← next
- [ ] S3 — /lcd:tasks, /lcd:test-gen, red-green, audit

## DECISIONS (this work-item)
- Scope forks (Ocean refused, GUI recognise-only, VICE in CI, no hardware gate) → D-037

## OPEN QUESTIONS
- none blocking; key bindings and the boot-check key injection are left for plan.md

## EDIT BOUNDARY (paths this work may touch)
- `menu/` (new), `crates/stidcart-core/`, `crates/stidcart-cli/`, `gui/`, `.github/workflows/ci.yml`, `docs/`, `README.md`
<!-- /lcd-resume -->

---

## Acceptance criteria

AC-81..AC-94 live in `spec.md`.

## LOG (append-only; not read on resume)

- 2026-11-07 — Triaged Deep (5 signals, hard trigger: new module + assembler dependency + produced
  bytes). User picked opt-in, GPL `menu/`, 64tass, and all four goals split into slices → D-037.
  Scope forks for this slice: Ocean refused, GUI recognise/show only, VICE installed in CI, VICE
  is enough without a hardware check. Spec written: AC-81..AC-94.

## DEEP PIPELINE (Deep lane only)

> Phase tracker for the Deep-lane pipeline. Files live beside this JOURNAL.

`spec.md ✅  plan.md ⬜  tasks.md ⬜  tests ⬜  red-green ⬜  audit.md ⬜`
<!-- mark ✅ done · ⏳ in progress · ⬜ not started -->
