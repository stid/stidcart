# JOURNAL — easyfs1-refusal

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the *entire*
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Standard
- **Goal:** Refuse an old-format (EasyFS1) directory with a typed `Unsupported` error instead of
  misreading it as EasyFS2 — closing the gap between D-007's decision and `SPEC.md:58`.
- **Next action:** done — PR #20 is open with `gate`, `audit` and `claude-review` all green. Merge
  is the maintainer's call.
- **Branch:** feat/easyfs1-refusal  ·  **Updated:** 2026-08-12

## STEPS
- [x] S1 — Corrected the format spec in the private notes: the reserved bits `0x60` are the
      EasyFS1/EasyFS2 version discriminator, not a tolerate-both quirk. Misattribution fixed (Draco
      Cart Maker, `README:82` — not an ndefpack bug, not `NEWS`), writer guidance kept, single-byte
      bank added. D-017 logged. The private notes are not in a git repo, so that edit is not in any commit.
- [x] S2 — Failing test for AC-14: `inspect`/`load` refuse a first entry with the bits clear.
- [x] S3 — `directory::is_old_format()` + `Unsupported::OldFormatDirectory`. Replaced
      `decode_tolerates_the_reserved_bits_being_clear` and fixed the doc comments that had copied
      the wrong claim in from the format spec.
- [x] S4 — AC-15: slots disagreeing are `ReadError::DirectoryFormatDisagrees`, a malformed variant
      of its own. `read_directory` carries a `seen_easyfs2` flag; `BadEntry` was rejected for this
      because its message names an undefined *type value*, which is not the fault here.
- [x] S5 — AC-13: `list` exits non-zero with the reason on stderr. Verified on a real cart built by
      the tool, not only in-process.
- [x] S6 — AC-16 regression guard: the four frozen fixtures still round-trip byte-identically.
- [x] S7 — Gate green, audit PASS (test-presence, first run), SPEC folded, closeout logged.
- [x] S8 — PR #20 open; `gate`, `audit` and `claude-review` all pass, no review findings.

## DECISIONS (this work-item)
- The reserved bits are the version discriminator; "tolerate both on read" was unsatisfiable —
  you cannot both ignore a byte and use it to detect the format. → DECISIONS.md as D-017.
- Detection state lives in `read.rs`'s walk, the byte test in `directory.rs`, matching the existing
  split (one entry's bytes vs the directory walk). Not a new `Slot` variant: old-format-ness is
  orthogonal to entry type, and a variant would make `decode` stop being a true inverse of `encode`.
- Case 3 (slots disagree) is in scope though D-007 only asked for case 2 — the byte is being read
  anyway, and letting disagreement fall through to "old format" is a wrong message on a malformed
  cart.

## OPEN QUESTIONS
- none

## EDIT BOUNDARY (paths this work may touch)
- the private-notes copy of the format spec (S1 only)
- `crates/stidcart-core/src/directory.rs`
- `crates/stidcart-core/src/read.rs`
- `crates/stidcart-core/tests/read.rs`
- `crates/stidcart-cli/tests/list.rs`
- `docs/lcd/DECISIONS.md` · `docs/lcd/SPEC.md`
<!-- /lcd-resume -->

---

## Acceptance criteria

> **Numbered from AC-13, not AC-1.** `AC-1..AC-12 (none)` and `AC-1..AC-5 (CLI)` are already live
> tokens in this workspace's test comments, left over from the pre-D-013 pipeline. Reusing one would
> let the audit's test-presence check pass against an unrelated orphan test. See the corrected note
> in `.claude/rules/lcd-conventions.md`.

**AC-13** (surfaces: CLI): `list` on a `.crt` whose first directory entry carries the reserved flag
bits clear exits non-zero and prints the old-format refusal on stderr, instead of printing a
listing decoded under the wrong format.

**AC-14** (surfaces: none): both `inspect` and `load` refuse such an image with
`ReadError::Unsupported(Unsupported::OldFormatDirectory { .. })`. Refusing to *describe* it is
consistent with D-011 rather than an exception to it: the bytes cannot be decoded correctly at all
under EasyFS2, so there is no accurate description to give.

**AC-15** (surfaces: none): a directory whose slots disagree — a first entry with the reserved bits
set followed by one with them clear — is refused as malformed, not as old-format.

**AC-16** (surfaces: none): every frozen fixture still round-trips byte-identically. An EasyFS2
directory, whose slots all carry the bits set, is unaffected by the new check.

## LOG (append-only; not read on resume)

- 2026-08-12 — Triaged Standard (6 signals, all Standard-level; no hard trigger, no risk signal).
  No backlog existed: `gh issue list` is empty on `stid/stidcart` and `docs/lcd/work/` was bare.
  This item was chosen as the only decided-but-unbuilt gap in the log.
- 2026-08-12 — Format research (the reason this looked blocked, and why it isn't). The discriminator
  is the reserved bits `0x60` of the flags byte at offset 16 — `efs_check_entry`
  (`ndefpack/src/libndef/easyfs.c:371-389`): bits not both set → old EasyFS. The first non-end slot
  decides for the whole directory, and a later slot disagreeing is a hard error, not a
  reclassification (`easyfs.c:382-385`). `efs_parse_all` (`easyfs.c:831`) threads that state.
  EasyFS1's structural difference for the entries this tool represents is a **single-byte bank** at
  offset 17 — `data[18]` is not part of the field (`easyfs.c:255-260`) — so an EasyFS1 directory read
  as EasyFS2 gets a corrupted bank number, which is why misreading is silent rather than loud.
- 2026-08-12 — The private-notes spec was wrong, in a way that changed the work. Its reserved-bits bullet
  reads "A past ndefpack bug wrote them as 0 (`NEWS`); anything reading these carts should tolerate
  both". Wrong tool, wrong citation, and an unsatisfiable prescription: `README:82` is a comparison
  against **Draco Cart Maker**, a different 2012 tool, and `NEWS` says nothing about reserved bits.
  ndefpack's own writer ORs `EFS_RESERVED_MASK` unconditionally (`easyfs.c:581`), so clear bits are
  unambiguously "not written by ndefpack" — the discriminator is clean. A Draco-written cart is
  refused as old-format, which is exactly what ndefpack does with it.
- 2026-08-12 — The wrong claim was copied into this repo: `directory.rs:12-13` and `:67-69` both
  repeat "a past bug in the reference implementation", and the test
  `decode_tolerates_the_reserved_bits_being_clear` (`directory.rs:250`) pins the behaviour this
  work-item removes. The writer half is correct and stays — `encode` sets the bits
  (`directory.rs:107`) and must keep doing so.
- 2026-08-12 — `SPEC.md:58` ("Old-format (EasyFS1) directories are **not detected**") is the row this
  work-item makes false. It is what `lcd:reconcile` rewrites at closeout.
- 2026-08-12 — **The conventions file was wrong about AC-id collisions, and it would have produced a
  false green.** `.claude/rules/lcd-conventions.md` claimed the orphan `ac_N_*` tests "cannot
  false-green a coverage check" because the old *function names* lack the literal `AC-N (SURFACE)`.
  Their doc comments carry it verbatim — `grep -rn "AC-[0-9]* (" crates/` lists `AC-1..AC-12 (none)`
  and `AC-1..AC-5 (CLI)` across six files. Had this work-item numbered from AC-1 as that note
  advised, the audit's test-presence check would have passed against unrelated pre-existing tests.
  Corrected the note, recorded the next free id, and numbered this item AC-13..AC-16. Renaming the
  orphans was rejected as a six-file sweep for no behaviour change, which the note itself advises
  against.
- 2026-08-12 — Tests validated by mutation rather than by trusting a green run, per the project's own
  standard. Forcing `is_old_format` to always report EasyFS2 fails AC-13, AC-14 and the unit test
  while AC-16 correctly stays green; separately forcing `seen_easyfs2` to stay false fails AC-15
  alone, confirming it is not riding on AC-14's coverage. Note that `cargo test` fails fast at target
  granularity, so the first mutation run reported only the lib target until each target was run
  individually — a mutation check here has to name the target, or it under-reports.
- 2026-08-12 — reconciled: folded AC-13 → SPEC.md/CLI, AC-14..AC-16 → SPEC.md/Internal. The
  EasyFS1 row was a `D-015 (backfill)` seed asserting the opposite behaviour; it decays into an
  AC-pinned row citing D-017. No supersede entry — D-017 implements what D-007 decided rather than
  overturning it, so D-007 stays active.
- 2026-08-12 — **Audit PASS (test-presence, first run).** Standard lane has no `spec.md`/`plan.md`,
  so the cross-path script does not apply; the test-presence half ran instead. `AC-13 (CLI)` →
  `crates/stidcart-cli/tests/list.rs:250`, `AC-14 (none)` → `crates/stidcart-core/tests/read.rs:505`,
  `AC-15 (none)` → `:540`, `AC-16 (none)` → `:564`; all four pass and the suite is green across 13
  targets. `closeout-evaluator: off`, so no independent evaluator was dispatched.
- 2026-08-12 — Drove the real binary end to end: built a cart with `build`, cleared the reserved bits
  of slot 0 in its bank-0 ROMH packet, and confirmed `list` prints
  `stidcart: the directory is in the old EasyFS format (slot 0 …)` and exits 1, while the untouched
  cart still lists normally.
