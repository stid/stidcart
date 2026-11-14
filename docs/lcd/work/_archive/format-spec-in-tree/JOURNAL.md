# JOURNAL — format-spec-in-tree

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the *entire*
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Standard
- **Goal:** Move the EasyFlash cart format spec from the author's private notes into this repo at
  `docs/format/easyflash-cart-format.md`, so the document every offset in `stidcart-core` derives
  from is reviewable, gateable and blameable (GitHub issue #21).
- **Next action:** done — PR open against `main`, closes #21.
- **Branch:** docs/format-spec-in-tree  ·  **Updated:** 2026-08-14

## STEPS
- [x] S1 — Copy the format spec into `docs/format/`, cleaned of note markup: drop the
      YAML frontmatter, resolve the two `[[wikilinks]]`, keep every byte of the substance.
- [x] S2 — Update the in-repo pointers: `CLAUDE.md`'s notes-references block, `MAP.md:34` (docs
      zone splits in two), `MAP.md:64` + `:71` (format-facts and D-007 invariants),
      `.claude/rules/lcd-conventions.md:76`, and `README.md` (path + one stale EasyFS1 claim).
- [x] S3 — Confirm the licence question the issue raises: the spec is a derivative work of
      ndefpack's GPL C by the same argument as D-004, so record that in the moved file.
- [x] S4 — Replace the private note with a stub pointing at the repo copy (no second source of truth).
- [x] S5 — Log D-018 (supersedes the format-facts invariant's private-notes clause).
- [x] S6 — Gate green, close the JOURNAL, PR (closes #21).

## DECISIONS (this work-item)
- **`docs/format/`, not `spec/` or bare `docs/`** — chosen by the user at triage. Keeps prose under
  `docs/` and leaves `docs/lcd/` as the untouched LCD artifact root (D-014); the docs zone row
  splits into two rather than losing its anti-sprawl clause. → D-018.
- **The cleanup is mechanical, not content editing.** Frontmatter and `[[wikilinks]]` are private-notes
  transport, and both render as noise on GitHub. The issue's "not in scope: rewriting the spec's
  content" holds — no format claim changes.

## OPEN QUESTIONS
- none

## EDIT BOUNDARY (paths this work may touch)
- `docs/format/easyflash-cart-format.md` (new)
- `docs/lcd/MAP.md`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/work/format-spec-in-tree/JOURNAL.md`
- `docs/lcd/triage-log.md`
- `CLAUDE.md`
- `README.md`
- `.claude/rules/lcd-conventions.md`
- the private-notes copy of the format spec (S4 only, replaced by a stub)
<!-- /lcd-resume -->

---

## Acceptance criteria

None. No surface: this work-item touches no code and declares no CLI/RENDER behaviour, so the
audit's test-presence check does not apply and no AC ids are claimed (next free stays AC-17).

## LOG (append-only; not read on resume)

- 2026-08-14 — Triaged Standard (3 signals: 3–8 files, contained architecture impact, one-commit
  revert; no hard trigger, no risk signal). Work came from issue #21, filed 2026-08-13, the only
  open item on `stid/stidcart`; no open PRs.
- 2026-08-14 — **The verbatim claim nearly shipped with unrunnable evidence.** The first PR body
  offered a `diff` command against a path that does not exist — S4 had already overwritten the private notes
  original, and the private notes are not versioned, so nothing on a reviewer's machine reproduces it.
  Replaced with the actual diff output, captured before the overwrite, and a sentence saying why it
  cannot be re-run. Reconstructed the pre-stub note from `dc7deec` plus the three exact edit strings
  and kept it in the session scratchpad until #22 merges: reverting `docs/format/` is trivial,
  restoring a destroyed private note is not, and review could still move the file.
- 2026-08-14 — Checked that a docs-only PR still trips the required checks — it is the first here
  with nothing under `crates/`. Neither `ci.yml` nor `claude-review.yml` carries a `paths:` filter,
  so `gate` and `audit` both ran and passed and `claude-review` queued normally. Had `gate` been
  filtered, a required check with `enforce_admins: true` would have left the PR unmergeable by
  anyone. Nothing to change; recording it so nobody re-derives it.
- 2026-08-14 — Closeout. Gate green (fmt, clippy `-D warnings`, 13 test targets) on the pinned
  toolchain — `cargo 1.97.1` from `/opt/homebrew/opt/rustup/bin`, checked per D-010 before trusting
  it. No audit: no surface was declared, so the test-presence check has nothing to check.
- 2026-08-14 — **`lcd:reconcile` deliberately not run, though `living-spec: on`.** `SPEC.md` says
  what the *tool does*, and this work-item changes no behaviour — folding "the format spec lives in
  `docs/format/`" into a capability index would put a fact about the repo's layout in the one file
  whose whole discipline is that it holds behaviour only. The move's durable record is D-018 and the
  MAP zone row, which is where a layout fact belongs. A grep confirmed no `SPEC.md` row cites the
  private notes. (The stale `README.md:25` claim this item fixed was never a `SPEC.md` row — `SPEC.md:59`
  had been folded correctly by the easyfs1-refusal closeout; only the README's prose copy lagged.)
- 2026-08-14 — S4 + S5. The private note is kept as a stub rather than deleted: the project note
  links to it twice (`:41`, `:191`), and deleting the file would break both while removing the one
  place a reader of the notes is told where the spec went. The stub says explicitly not to restore the
  content. D-018 logged; next id is now D-019.
- 2026-08-14 — S2. Five pointers, not four: `README.md:8` also names the spec ("that spec is the
  reference for everything in `stidcart-core`") without a path, which is exactly the pointer this
  move has to make resolvable. Added to the EDIT BOUNDARY via refine before editing — the boundary
  hook denied the first two attempts, and the second denial was mine: it matches **bare paths**, so
  a bullet reading `` `README.md` (added S2 — …) `` does not register. Annotate elsewhere, keep the
  path alone on its line.
- 2026-08-14 — **`README.md:25` was stale and said the opposite of shipped behaviour** — "nothing
  detects [EasyFS1 directories], so they are walked as EasyFS2 and misread rather than refused".
  D-017 made that false on 2026-08-12 and the closeout folded `SPEC.md` without catching the
  README's narrative copy of the same fact. Corrected here rather than left for a later item: the
  line is two files away from the ones this work-item is already rewriting, and the whole argument
  for moving the spec in-tree is that a claim and its source should be diffable together.
- 2026-08-14 — S1 + S3. Copied with `tail -n +8` rather than retyped, then diffed against the private notes
  original: the only differences are the dropped frontmatter, the two de-wikilinked references, a
  header block saying the file is the source for `stidcart-core`'s offsets, and the licence
  paragraph. Every format claim is byte-identical, which is what "not in scope: rewriting the
  spec's content" has to mean if it is to be checkable.
- 2026-08-14 — Licence question (the issue's "worth checking"): resolved, not assumed. The spec was
  written by reading ndefpack's GPL C, so it is a derivative work by exactly D-004's argument, and
  GPL-2.0-or-later is where it already belonged. The repo going public changes nothing — the
  obligation travels with the licence, and `COPYING` is already in the tree. `firmware/` is
  untouched: the zlib boundary runs through that directory only.
- 2026-08-14 — Placement settled at `docs/format/` (user's call among the three the issue names).
  `MAP.md:34`'s "anything non-LCD directly in `docs/`" clause becomes "directly in `docs/lcd/`" and
  the zone table gains a `format` row — the guardrail moves down one level rather than being dropped.
