# JOURNAL — gui-edit

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the *entire*
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Deep
- **Goal:** Second GUI slice: the viewer edits carts with the full `edit` set plus New, held in memory with undo, an unsaved-changes indicator and prompt, ⌘S/⌘⇧S, and saves byte-identical to `edit`/`build`.
- **Next action:** run /lcd:plan gui-edit (spec awaiting user approval)
- **Branch:** feat/gui-edit  ·  **Updated:** 2026-10-11 18:30

## STEPS
- [x] S1 — triage (Deep) + recon dispatched + spec.md
- [ ] S2 — /lcd:plan gui-edit  ← next

## DECISIONS (this work-item)
- ACs AC-59..AC-80 claimed (next free AC-81); GUI ACs use `RENDER`, byte parity `none`
- Product choices → D-033

## OPEN QUESTIONS
- none blocking; undo storage is a plan question

## EDIT BOUNDARY (paths this work may touch)
- `docs/lcd/work/gui-edit/`
- `docs/lcd/DECISIONS.md`
- `.claude/rules/lcd-conventions.md`
<!-- /lcd-resume -->

---

## Acceptance criteria

See `spec.md` (AC-59..AC-80).

## LOG (append-only; not read on resume)

- 2026-10-11 18:10 — triage → Deep (5 signals, hard: new stateful editing module; risk: multi-session). Recon (Tauri state, menus, close interception, save dialog, undo, bits-ui) dispatched.
- 2026-10-11 18:20 — user: full edit set; explicit save; drops on an open cart add (incl. .crt); multi-drop adds atomically; New in scope → D-033.
- 2026-10-11 18:25 — found: core `build()` already refuses alignment on non-xbank (`AlignNotXbank`); names truncate silently to field width (16 entry, 32 header). `input_entry`/`entry_name` are CLI-private and must be shared for AC-61/AC-80 (plan).

## DEEP PIPELINE (Deep lane only)

> Phase tracker for the Deep-lane pipeline. Files live beside this JOURNAL.

`spec.md ✅  plan.md ⬜  tasks.md ⬜  tests ⬜  red-green ⬜  audit.md ⬜`
<!-- mark ✅ done · ⏳ in progress · ⬜ not started -->
