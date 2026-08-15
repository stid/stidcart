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
- **Next action:** run `/lcd:tasks cart-edit`
- **Branch:** feat/cart-edit  ·  **Updated:** 2026-08-14 22:45

## STEPS
- [x] S1 — triage → Deep (6 signals, hard:yes risk:yes); logged in `docs/lcd/triage-log.md`
- [x] S2 — `/lcd:plan cart-edit` — mutators on `CartBuilder`, hidden in `Program`, one `edit`
  command with repeatable op flags, atomic write in the CLI
- [ ] S3 — `/lcd:tasks cart-edit` — TDD-ordered checklist  ← next
- [ ] S4 — `/lcd:test-gen cart-edit` — one failing test per (AC × surface), AC-17..AC-30
- [ ] S5 — `lcd:redgreen-loop` — per-fix commits until green
- [ ] S6 — `/lcd:audit cart-edit`, then PR

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
- none blocking `/lcd:plan`

## EDIT BOUNDARY (paths this work may touch)
- `crates/stidcart-core/src/lib.rs` — `Program.hidden`, `CartBuilder` mutators, `build_image` writes
  the flag
- `crates/stidcart-core/src/read.rs` — `load` carries the flag; the `HiddenEntry` refusal is removed
- `crates/stidcart-cli/src/main.rs` — the `edit` command + atomic write; `build --hide`; `USAGE`
- `crates/stidcart-core/tests/mutate.rs` (new) — AC-29, AC-30
- `crates/stidcart-cli/tests/edit.rs` (new) — AC-17..AC-27
- `crates/stidcart-cli/tests/cli.rs` — AC-28
- `Cargo.toml` — 0.4.0 → 0.5.0 in the first implementation commit (`feat/` branch, versioning rule)
- `README.md` — the State section still says "It does not edit a cart"
- `docs/lcd/work/cart-edit/`, `docs/lcd/DECISIONS.md`, `docs/lcd/SPEC.md` (closeout only)
- **Not touched:** `docs/format/` — no format fact changes; `directory.rs` already encodes bit 7
<!-- /lcd-resume -->

---

## Acceptance criteria

See `spec.md` — **AC-17 … AC-30** (CLI: 17–28; none: 29–30). Numbering starts at AC-17 per
`.claude/rules/lcd-conventions.md`; that file's "Next free" line is bumped to AC-31 by this
work-item.

## LOG (append-only; not read on resume)

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

`spec.md ✅  plan.md ✅  tasks.md ⬜  tests ⬜  red-green ⬜  audit.md ⬜`
<!-- mark ✅ done · ⏳ in progress · ⬜ not started -->

<!--
Resume contract (why the block above is fenced):
  /lcd:resume extracts ONLY the lcd-resume:v1 block. If NOW/STEPS are stale, resume
  rebuilds a false context — so update them in the same edit where you change the work.
  /lcd:resume also cross-checks `Updated` + `Branch` against `git log` and warns if this
  JOURNAL is older than HEAD before trusting it.
-->
