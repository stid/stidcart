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
- **Next action:** S1 — correct the reserved-flag-bits bullet in the vault format spec.
- **Branch:** feat/easyfs1-refusal  ·  **Updated:** 2026-08-12

## STEPS
- [ ] S1 — Correct `reference/easyflash-cart-format.md` (vault): the reserved bits `0x60` are the
      EasyFS1/EasyFS2 version discriminator, not a tolerate-both quirk. Fix the misattribution
      (Draco Cart Maker, `README:82` — not an ndefpack bug, not `NEWS`), keep the writer guidance,
      add the single-byte-bank difference. Log D-017 in the same commit.  ← next
- [ ] S2 — Failing test for AC-2: `inspect`/`load` refuse a first entry with the bits clear.
- [ ] S3 — `directory::is_old_format()` + `Unsupported::OldFormatDirectory`; make S2 pass.
      Replace `decode_tolerates_the_reserved_bits_being_clear` (directory.rs:250) — it asserts the
      behaviour being changed — and fix the wrong claim in the doc comments at directory.rs:12,67.
- [ ] S4 — Failing test + code for AC-3: slots disagreeing on the reserved bits are malformed,
      not old-format. Thread `was_old` state through `read_directory`'s walk.
- [ ] S5 — Failing test for AC-1: `list` on such a cart exits non-zero with the message on stderr.
- [ ] S6 — AC-4 regression guard: the four frozen fixtures still round-trip unchanged.
- [ ] S7 — Gate green, `/lcd:audit easyfs1-refusal`, reconcile `SPEC.md:58`, PR.

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
- `~/Documents/obsidian/stid/reference/easyflash-cart-format.md` (vault — S1 only)
- `crates/stidcart-core/src/directory.rs`
- `crates/stidcart-core/src/read.rs`
- `crates/stidcart-core/tests/read.rs`
- `crates/stidcart-cli/tests/list.rs`
- `docs/lcd/DECISIONS.md` · `docs/lcd/SPEC.md`
<!-- /lcd-resume -->

---

## Acceptance criteria

**AC-1** (surfaces: CLI): `list` on a `.crt` whose first directory entry carries the reserved flag
bits clear exits non-zero and prints the old-format refusal on stderr, instead of printing a
listing decoded under the wrong format.

**AC-2** (surfaces: none): both `inspect` and `load` refuse such an image with
`ReadError::Unsupported(Unsupported::OldFormatDirectory { .. })`. Refusing to *describe* it is
consistent with D-011 rather than an exception to it: the bytes cannot be decoded correctly at all
under EasyFS2, so there is no accurate description to give.

**AC-3** (surfaces: none): a directory whose slots disagree — a first entry with the reserved bits
set followed by one with them clear — is refused as malformed, not as old-format.

**AC-4** (surfaces: none): every frozen fixture still round-trips byte-identically. An EasyFS2
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
- 2026-08-12 — The vault spec was wrong, in a way that changed the work. Its reserved-bits bullet
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
