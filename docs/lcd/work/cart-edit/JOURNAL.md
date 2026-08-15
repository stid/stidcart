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
- **Next action:** open the PR (`gh pr create`); audit PASSED and the gate is green
- **Branch:** feat/cart-edit  ·  **Updated:** 2026-08-14 23:35

## STEPS
> Mirrors `tasks.md` (T1..T23 + T-audit/T-final); read that file for each task's detail.
- [x] S1 — triage → Deep; `spec.md` (AC-17..AC-30); `plan.md` (approach, constitution, matrix)
- [x] S5' — T2, T5, T7..T17, T21: the failing tests exist and are red — `tests/mutate.rs` (AC-29,
  AC-30, red at compile: the mutators do not exist) and `tests/edit.rs` (AC-17..AC-28, red at run:
  `unknown command 'edit'`)
- [x] S2 — T1: `Cargo.toml` 0.4.0 → 0.5.0, in the first implementation commit
- [x] S3 — T3..T4: hidden in the model — `lib.rs` (`Program.hidden`, `build_image`), `read.rs`
  (`load` carries it, `HiddenEntry` deleted), `tests/read.rs` flipped to "it loads" · AC-29
- [x] S4 — T6: `CartBuilder` mutators in `lib.rs` · AC-30
- [x] S6 — T18..T20: `edit` in `main.rs` — grammar + bounds checks, three-phase apply, atomic write
- [x] S7 — T22: `build --hide`, addressed by input path · AC-28
- [x] S8a — T23: README State section and usage block
- [x] S8b — T-audit: PASS first run (12/12 CLI rows); T-final: gate green; SPEC.md reconciled
- [ ] S9 — open the PR  ← next

## DECISIONS (this work-item)
- Entries are addressed **by index only**, as shown in the listing — unambiguous when two entries
  share a name.
- Replacing an entry's program data is **in** (AC-21); reordering entries is **out**.
- Building gains a way to mark an entry hidden (AC-28), now that the model can carry the flag.
- Mutation is **setters on `CartBuilder`**, not a second document type and not a raw `&mut Vec`;
  index-addressed setters panic like `Vec` indexing, because the CLI bounds-checks anyway.
- An edit **re-places the whole cart** — untouched entries may move (→ DECISIONS D-019).
- Saving is **in place by default** (→ DECISIONS D-019); the hidden-entry read refusal is **retired**
  (→ DECISIONS D-020).

## OPEN QUESTIONS
- none blocking `/lcd:test-gen`

## EDIT BOUNDARY (paths this work may touch)
> Bare paths only — the boundary hook matches each bullet as a path, so annotations go here, not on
> the bullets. `lib.rs`: `Program.hidden`, mutators, `build_image` writes the flag. `read.rs`: `load`
> carries it, the `HiddenEntry` refusal goes. `tests/read.rs`: :164 and :439 match that variant and
> flip to "it loads". `main.rs`: the `edit` command, atomic write, `build --hide`, `USAGE`.
> `mutate.rs` and `edit.rs` are new. `Cargo.toml`: 0.4.0 → 0.5.0 in the first implementation commit.
> `README.md`: the State section still says the tool cannot edit. **Not touched:** `docs/format/` —
> no format fact changes, and `directory.rs` already encodes bit 7.
- `crates/stidcart-core/src/lib.rs`
- `crates/stidcart-core/src/read.rs`
- `crates/stidcart-core/tests/read.rs`
- `crates/stidcart-core/tests/mutate.rs`
- `crates/stidcart-cli/src/main.rs`
- `crates/stidcart-cli/tests/edit.rs`
- `crates/stidcart-cli/tests/cli.rs`
- `Cargo.toml`
- `README.md`
- `docs/lcd/work/cart-edit`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/SPEC.md`
<!-- /lcd-resume -->

---

## Acceptance criteria

See `spec.md` — **AC-17 … AC-30** (CLI: 17–28; none: 29–30). Numbering starts at AC-17 per
`.claude/rules/lcd-conventions.md`; that file's "Next free" line is bumped to AC-31 by this
work-item.

## LOG (append-only; not read on resume)

- 2026-08-14 23:50 — audit: PASS, first run, 12/12 CLI rows OK (`audit.md`). Closeout logged in
  `triage-log.md`. Reconciled: folded AC-17..AC-28 → SPEC.md/CLI (4 new rows, `build` rewritten for
  `--hide`) and AC-29/AC-30 → SPEC.md/Internal (1 new row, 2 rewritten). The `load`-refuses row lost
  its "including hidden entries" clause, which D-020 retired; D-011 gained an "Amended by D-020"
  line, since its describe-versus-rebuild split stands and only one item left its precondition list.

- 2026-08-14 23:35 — red-green: green in 2 iterations, 0 reverts, 0 interventions. Iteration 1 took
  the core (hidden in the model + mutators, `tests/read.rs` flipped) — the two flipped tests are the
  behaviour change D-020 authorised, not collateral. Iteration 2 was the `edit` command; AC-25 and
  AC-26 passed without a line of refusal code, because `read::load`'s preconditions already are
  those refusals. Full gate green: fmt, clippy `-D warnings`, `cargo test --workspace`.

- 2026-08-14 23:10 — test-gen: 14 failing tests, one per AC. AC-28's test went into this
  work-item's own `tests/edit.rs` rather than the existing `tests/cli.rs`, so phase 4 added files
  without editing another suite; tasks T21 and the plan's file list follow. The EDIT BOUNDARY block
  was rewritten to bare paths — the boundary hook parses each bullet as a path, so the trailing
  `— what changes` prose made every entry unmatchable and blocked the first write.

- 2026-08-14 22:55 — tasks.md written: 25 tasks (23 + audit + final), one parallel-eligible. Two
  checkpoint hazards recorded there rather than discovered later — `tests/read.rs:164,439` match on
  `Unsupported::HiddenEntry` and stop compiling when T4 deletes it (they flip to "it loads", per
  D-020), and `build_image` is parity-gated, so `--test parity` must stay green through T3.

- 2026-08-14 22:45 — plan.md written. Core shape chosen with the user: mutators on `CartBuilder`,
  not an editable document type and not a raw `&mut Vec`. Two things the plan found by reading rather
  than assuming: `directory.rs` already encodes and decodes the hidden bit (only the model lacked a
  place to keep it), and the `feat/` branch owes a 0.4.0 → 0.5.0 bump whose only byte-level effect is
  the default container header name — no test hardcodes it and `parity.rs:54` sets the reference name
  explicitly, so the fixtures are unaffected.

- 2026-08-14 22:20 — triage: Deep (6 signals, hard:yes risk:yes). Scope confirmed with the user as
  the full edit suite; the minimal add-only variant would have been Standard.
- 2026-08-14 22:35 — spec review caught AC-29 claiming byte-identity for *any* hidden-carrying cart,
  which AC-30 (re-place everything) makes false for a cart laid out differently from this tool's
  placer — and every fixture is placer-compatible, so the test would have passed and hidden it.
  AC-29 narrowed to the entry-set round trip, byte-identity kept only where the stored layout already
  matches a rebuild. D-020 gained the clause separating the hidden refusal (model gap) from the
  firmware refusal (a save genuinely rewrites bank 0).
- 2026-08-14 22:26 — spec.md written. Five forks resolved with the user: re-place everything on save,
  in place by default, retire the hidden-entry refusal, address entries by index only, include
  program-data replacement, and let `build` mark entries hidden. → DECISIONS D-019, D-020.

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
