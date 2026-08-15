# JOURNAL — cart-edit

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the *entire*
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Deep
- **Goal:** Edit an existing cart — load it, add/remove/rename/replace entries, toggle hidden, set
  the container and menu names, and write the result back (in place by default, or to a destination).
- **Next action:** run `/lcd:plan cart-edit`
- **Branch:** feat/cart-edit  ·  **Updated:** 2026-08-14 22:26

## STEPS
- [x] S1 — triage → Deep (6 signals, hard:yes risk:yes); logged in `docs/lcd/triage-log.md`
- [ ] S2 — `/lcd:plan cart-edit` — architecture, file map, Constitution check  ← next
- [ ] S3 — `/lcd:tasks cart-edit` — TDD-ordered checklist
- [ ] S4 — `/lcd:test-gen cart-edit` — one failing test per (AC × surface), AC-17..AC-30
- [ ] S5 — `lcd:redgreen-loop` — per-fix commits until green
- [ ] S6 — `/lcd:audit cart-edit`, then PR

## DECISIONS (this work-item)
- Entries are addressed **by index only**, as shown in the listing — unambiguous when two entries
  share a name.
- Replacing an entry's program data is **in** (AC-21); reordering entries is **out**.
- Building gains a way to mark an entry hidden (AC-28), now that the model can carry the flag.
- An edit **re-places the whole cart** — untouched entries may move (→ DECISIONS D-019).
- Saving is **in place by default** (→ DECISIONS D-019); the hidden-entry read refusal is **retired**
  (→ DECISIONS D-020).

## OPEN QUESTIONS
- none blocking `/lcd:plan`

## EDIT BOUNDARY (paths this work may touch)
- `crates/stidcart-core/src/lib.rs` — the entry-set model gains mutation and a hidden flag
- `crates/stidcart-core/src/read.rs` — the hidden-entry refusal is retired
- `crates/stidcart-core/src/directory.rs` — hidden flag on the write side
- `crates/stidcart-cli/src/main.rs` — the `edit` command; `build` gains the hidden marker
- `crates/stidcart-core/tests/cart-edit.rs`, `crates/stidcart-cli/tests/edit.rs` — AC tests
- `docs/lcd/work/cart-edit/`, `docs/lcd/DECISIONS.md`, `docs/lcd/SPEC.md`, `.claude/rules/lcd-conventions.md`
<!-- /lcd-resume -->

---

## Acceptance criteria

See `spec.md` — **AC-17 … AC-30** (CLI: 17–28; none: 29–30). Numbering starts at AC-17 per
`.claude/rules/lcd-conventions.md`; that file's "Next free" line is bumped to AC-31 by this
work-item.

## LOG (append-only; not read on resume)

- 2026-08-14 22:20 — triage: Deep (6 signals, hard:yes risk:yes). Scope confirmed with the user as
  the full edit suite; the minimal add-only variant would have been Standard.
- 2026-08-14 22:26 — spec.md written. Five forks resolved with the user: re-place everything on save,
  in place by default, retire the hidden-entry refusal, address entries by index only, include
  program-data replacement, and let `build` mark entries hidden. → DECISIONS D-019, D-020.

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
