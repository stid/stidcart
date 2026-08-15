# JOURNAL — format-spec-in-tree

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the *entire*
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Standard
- **Goal:** Move the EasyFlash cart format spec from the Obsidian vault into this repo at
  `docs/format/easyflash-cart-format.md`, so the document every offset in `stidcart-core` derives
  from is reviewable, gateable and blameable (GitHub issue #21).
- **Next action:** S2 — update the four in-repo pointers.
- **Branch:** docs/format-spec-in-tree  ·  **Updated:** 2026-08-14

## STEPS
- [x] S1 — Copy `reference/easyflash-cart-format.md` into `docs/format/`, de-vaulted: drop the
      Obsidian frontmatter, resolve the two `[[wikilinks]]`, keep every byte of the substance.
- [ ] S2 — Update the in-repo pointers: `CLAUDE.md`'s vault-references block, `MAP.md:34` (docs
      zone splits in two), `MAP.md:64` (format-facts invariant), `.claude/rules/lcd-conventions.md:76`.  ← next
- [x] S3 — Confirm the licence question the issue raises: the spec is a derivative work of
      ndefpack's GPL C by the same argument as D-004, so record that in the moved file.
- [ ] S4 — Replace the vault note with a stub pointing at the repo copy (no second source of truth).
- [ ] S5 — Log D-018 (supersedes the format-facts invariant's vault clause).
- [ ] S6 — Gate green, close the JOURNAL, PR (closes #21).

## DECISIONS (this work-item)
- **`docs/format/`, not `spec/` or bare `docs/`** — chosen by the user at triage. Keeps prose under
  `docs/` and leaves `docs/lcd/` as the untouched LCD artifact root (D-014); the docs zone row
  splits into two rather than losing its anti-sprawl clause. → D-018.
- **De-vaulting is mechanical, not content editing.** Frontmatter and `[[wikilinks]]` are vault
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
- `.claude/rules/lcd-conventions.md`
- `~/Documents/obsidian/stid/reference/easyflash-cart-format.md` (vault — S4 only, replaced by a stub)
<!-- /lcd-resume -->

---

## Acceptance criteria

None. No surface: this work-item touches no code and declares no CLI/RENDER behaviour, so the
audit's test-presence check does not apply and no AC ids are claimed (next free stays AC-17).

## LOG (append-only; not read on resume)

- 2026-08-14 — Triaged Standard (3 signals: 3–8 files, contained architecture impact, one-commit
  revert; no hard trigger, no risk signal). Work came from issue #21, filed 2026-08-13, the only
  open item on `stid/stidcart`; no open PRs.
- 2026-08-14 — S1 + S3. Copied with `tail -n +8` rather than retyped, then diffed against the vault
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
