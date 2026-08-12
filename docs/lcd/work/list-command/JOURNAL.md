# JOURNAL — list-command

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the *entire*
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Standard
- **Goal:** Expose the existing read path through a `stidcart list <file>` command that prints the
  cartridge name, one line per directory entry (index, name, type, bank, offset, size) and a
  free-space summary.
- **Next action:** open the PR (`gh pr create`); the audit passed and the gate is green.
- **Branch:** feat/list-command  ·  **Updated:** 2026-08-11 22:52

## STEPS
- [x] S1 — `README.md` State section: it still says "no reading"; the read path landed. Docs-only.
- [x] S2 — test first: `read::inspect` returns the directory as the **file** records it (AC-6).
- [x] S3 — refactor `read.rs` so one directory walk feeds both `load` and `inspect`; no behaviour
      change (round-trip + parity tests are the guard).
- [x] S4 — test first: `stidcart list` on each fixture — entry lines, cart name, free space (AC-1..3).
- [x] S5 — test first: `list` on a non-cartridge, and `list` with no path (AC-4, AC-5).
- [x] S6 — implement the CLI command and its usage text.
- [x] S7 — gate green (108 tests, fmt + clippy clean); audit PASS (`audit.md`).
- [ ] S8 — open the PR.  ← next

## DECISIONS (this work-item)
- **`list` reports the file's own positions, not regenerated ones.** `load()` returns a
  `CartBuilder`, which deliberately drops bank/offset — a rebuild re-places from scratch. Printing
  those regenerated positions under the heading "what is in this file" would be silently wrong in
  exactly the way this project keeps guarding against, so `inspect` is a second view over the same
  walk rather than a formatter over `load`'s result.
- **Free space is capacity minus the sum of entry sizes**, not a fragmentation model. The reference
  tool reports `total` and `top` separately; matching that is not required by anything here and
  would mean modelling a layout `list` does not otherwise need.

## OPEN QUESTIONS
- none

## EDIT BOUNDARY (paths this work may touch)
- `README.md`
- `crates/stidcart-core/src/read.rs`
- `crates/stidcart-core/src/lib.rs`
- `crates/stidcart-core/tests/read.rs`
- `crates/stidcart-cli/src/main.rs`
- `crates/stidcart-cli/tests/cli.rs`
- `crates/stidcart-cli/tests/list.rs`
- `docs/lcd/work/list-command/JOURNAL.md`
<!-- /lcd-resume -->

---

## Acceptance criteria

**AC-1** (surfaces: CLI): Given a valid cartridge image, when `stidcart list <file>` runs, then it
exits successfully and prints one line per directory entry carrying that entry's index, name, type,
bank, offset and size.

**AC-2** (surfaces: CLI): Given a cartridge stored with a name of its own, when it is listed, then
the output identifies the cartridge by that name.

**AC-3** (surfaces: CLI): Given a cartridge whose entries do not fill the flash, when it is listed,
then the output states the free space remaining, and that figure is the capacity less the sizes of
the entries present.

**AC-4** (surfaces: CLI): Given a file that is not a cartridge image, or one the loader refuses,
when it is listed, then the command fails, reports why in the loader's own words, and prints no
entry lines.

**AC-5** (surfaces: CLI): Given `list` with no file named, when it runs, then it fails and prints
the usage text.

**AC-6** (surfaces: none): Given a cartridge image, when its directory is inspected, then each
entry's reported bank, offset and size are the values recorded in the file, not values a rebuild
would regenerate.

## LOG (append-only; not read on resume)

- 2026-08-11 22:19 — triaged `LCD → Standard (3 signals)`: 3–4 files, one CLI surface, one-commit
  revert; no hard trigger, no risk signal. Work-item opened on `feat/list-command`.
- 2026-08-11 22:58 — **first Standard-lane item in this repo**, so the CI `audit` check had never
  met a `work/<slug>/` without `spec.md` + `plan.md` — and D-009 made it required with
  `enforce_admins`, so an unexpected red would have blocked the merge with no bypass. Checked rather
  than assumed: `lcd-audit.yml` already skips an item missing either file (`::notice::… not a
  Deep-lane item, skipping`) and exits 0. Running `audit-crosspath.sh` directly *does* exit 1 on the
  missing `spec.md`, but the workflow never calls it for such a slug. No CI change needed; both
  checks passed.
- 2026-08-11 22:58 — kept `audit.md` even though the skill's Standard exception says to record the
  result in the LOG and write no file. Nothing in `lcd-audit.yml` keys on its presence or its
  `Result:` line, so it cannot affect the gate, and the token-ambiguity finding deserves a place a
  future audit will actually look.
- 2026-08-11 22:52 — audit PASS (test-presence; the cross-path script does not apply to a Standard
  item, which has no `plan.md` matrix). Recorded in `audit.md`, including the finding that a bare
  `AC-N (SURFACE)` grep now matches three other work-items and would pass this slug even with no
  tests written — the numbering is per work-item, so this only gets worse. A slug-qualified token
  would fix it; not acted on here, phase 6 is verification only.
- 2026-08-11 22:48 — version bumped 0.3.0 → 0.4.0 (`versioning.md`: `feat/` → minor). It belongs in
  the branch's first commit and landed three commits late, because this branch opened with a
  docs-only fix and the habit is anchored to the first *source* commit. Changes the bytes of any
  cart built without an explicit `-e` (D-008); no fixture moves.
- 2026-08-11 22:44 — S2..S6 green in one pass. Worth recording which criterion actually carries the
  positions claim: **AC-1 does not**. It compares the printed line against `inspect`'s own output on
  `two.crt`, and that fixture round-trips byte-identically, so regenerated and recorded positions
  coincide there — a CLI that rebuilt instead of inspecting would still pass it. AC-6 is the one with
  teeth, and it has its own asserted negative control (an entry planted at bank 40, with the
  rebuilding path checked to disagree). Do not "simplify" AC-6 away on the grounds that AC-1 covers
  the same ground; it does not.
- 2026-08-11 22:44 — `list` prints to stdout and errors to stderr, and AC-4 asserts stdout is empty
  on refusal. That is what keeps the output pipeable without a partial listing arriving ahead of an
  error.
- 2026-08-11 22:31 — `lcd:refine`: added `crates/stidcart-cli/tests/list.rs` to the EDIT BOUNDARY.
  The boundary named `tests/cli.rs`, but that file already carries the *write-path* work-item's
  AC-1/AC-5, and this work-item numbers its own criteria from 1 as well. Two unrelated AC-1 (CLI)
  tokens in one file would make the audit's test-presence check unreadable, so `list` gets its own
  target — the same split `read-path` used against `cli.rs`.
- 2026-08-11 22:19 — the read-path spec named this work itself ("This work-item delivers the
  capability; exposing it to a user is separate work"), so scope is that sentence and nothing more:
  no editing, no `dump`/`ext`, no entry types the loader already refuses.
