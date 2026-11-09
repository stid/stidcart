# JOURNAL — folder-editor

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the _entire_
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->

## NOW

- **Lane:** Deep
- **Goal:** Folders replace groups: rename Group → Folder (window, CLI `--folder`, import key), add entry re-order (`edit --move`, window), and a folder tree in the window with drag and drop.
- **Next action:** user approves spec.md, then run /lcd:plan folder-editor
- **Branch:** feat/folder-editor · **Updated:** 2026-11-08

## STEPS

- [x] S1 — triage (Deep) and spec.md
- [ ] S2 — plan.md ← next
- [ ] S3 — tasks.md
- [ ] S4 — failing tests (AC-173..AC-184)
- [ ] S5 — red-green
- [ ] S6 — live check of the window, audit, PR

## DECISIONS (this work-item)

- Folders replace groups, no label field; tree order on tree operations; new folder made with an entry; tree whenever a folder exists; `edit --move` once per call → D-060

## OPEN QUESTIONS

- none (spec awaiting the user's approval)

## EDIT BOUNDARY (paths this work may touch)

- `docs/lcd/work/folder-editor/`
- `docs/lcd/DECISIONS.md`

<!-- /lcd-resume -->

---

## Acceptance criteria

See `spec.md` (AC-173..AC-184).

## LOG (append-only; not read on resume)

- 2026-11-08 — Triage: Deep (5 signals, hard trigger: CLI and window surfaces). Recon on
  Tauri/Svelte drag and drop started in a subagent.
- 2026-11-08 — Spec written; four branch points answered by the user (→ D-060).
- 2026-11-08 — Recon (Tauri 2.12.3 / wry 0.57.0 source, tauri#14373): with `dragDropEnabled`
  on (needed for Finder drops' paths), HTML5 dragover/drop never reach the page on macOS, so
  HTML5-DnD libraries are out and `dragDropEnabled: false` is out. Use pointer-event DnD
  (setPointerCapture + elementFromPoint, three-band hit test per row: above / into folder /
  below); fallback library @dnd-kit/svelte 0.5.3 (pre-1.0, drop-onto-row unverified).
  svelte-dnd-action has no drop-onto-item. Rows need `-webkit-user-drag: none; user-select:
none` and preventDefault on dragstart, or WKWebView starts a native drag Tauri swallows.
  WCAG 2.2 SC 2.5.7 → the menu move commands (AC-182) are the non-drag path; wire them through
  the native menu path too.

## DEEP PIPELINE (Deep lane only)

> Phase tracker for the Deep-lane pipeline. Files live beside this JOURNAL.

`spec.md ✅  plan.md ⬜  tasks.md ⬜  tests ⬜  red-green ⬜  audit.md ⬜`
<!-- mark ✅ done · ⏳ in progress · ⬜ not started -->
