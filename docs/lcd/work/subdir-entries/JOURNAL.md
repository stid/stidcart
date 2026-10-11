# JOURNAL — subdir-entries

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the *entire*
> cold-start payload. Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Standard
- **Goal:** Carry the CrtUsage subdirectory bit (`0x4000`) on xbank entries through read, list, edit and save, as `ndeft` does; refuse the bit on any other kind as malformed; keep type `0x1A` refused, by name.
- **Next action:** done — PR open; follow-up for type 0x1A needs a VICE boot (D-029).
- **Branch:** feat/subdir-entries  ·  **Updated:** 2026-10-11 16:45

## STEPS
- [x] S1: format spec: type `0x1A` refused by the reference (`easyfs.c:500-502`), bit allowed only on 0x1A/xbank (`easyfs.c:412`), carried only (`easyfs.c:211,590`; `place.c` never reads it), EasyLoader boots `0x1A` as Ultimax (`easyloader.asm:912`). Add to "behaviours to decide on"
- [x] S2: fixture: `make_inputs.py` patches `cxb16.crt` slot 1 with the bit → `in_cxb16sub.crt`; reference `cxb16sub.crt` = `ndeft -q ren` of a copy, renaming entry 0 to `hello2`. MANIFEST rows
- [x] S3: failing tests: directory codec round-trips the bit (AC-47); load+build of `in_cxb16sub.crt` is byte-identical (AC-47); `edit --rename 0 hello2` equals `cxb16sub.crt` (AC-48); `list` flags `S` (AC-49); bit on a non-xbank kind is malformed in both read paths (AC-50); `0x1A` still refused by name (existing AC-43 tests, message reworded)
- [x] S4: model: `subdir` on `DirEntry` and `Cartridge`; `decode` / `encode` / `read_from` / `dir_entry`; new malformed variant; `Slot::Subdirectory` → the 0x1A-only refusal
- [x] S5: CLI `S` flag; README, SPEC rows, D-029; gate; audit; PR

## DECISIONS (this work-item)
- Scope is the `ndeft` oracle: the bit on xbank is carried; `0x1A` stays refused because the reference refuses it and there is no fixture or VICE evidence for it (user chose "carry", 2026-10-11) → D-029
- The bit on a non-xbank kind is malformed, not unsupported: the reference rejects it as an error and EasyLoader never reads it → D-029
- No CLI option sets the bit (carry, not create). `--replace` takes the flag from the new data, like `align_64k`; a `.crt` input never carries it

## OPEN QUESTIONS
- none

## EDIT BOUNDARY (paths this work may touch)
- `docs/format/easyflash-cart-format.md`
- `fixtures/make_inputs.py`
- `fixtures/MANIFEST.md`
- `fixtures/in_cxb16sub.crt`
- `fixtures/cxb16sub.crt`
- `crates/stidcart-core/src/directory.rs`
- `crates/stidcart-core/src/cartridge.rs`
- `crates/stidcart-core/src/read.rs`
- `crates/stidcart-core/src/lib.rs`
- `crates/stidcart-core/tests/subdir.rs`
- `crates/stidcart-core/tests/read.rs`
- `crates/stidcart-core/tests/cartridge.rs`
- `crates/stidcart-cli/src/main.rs`
- `crates/stidcart-cli/tests/subdir_cli.rs`
- `crates/stidcart-cli/tests/carts.rs`
- `crates/stidcart-cli/tests/cartlist.rs`
- `crates/stidcart-cli/tests/edit.rs`
- `crates/stidcart-cli/tests/list.rs`
- `README.md`
- `docs/lcd/SPEC.md`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/triage-log.md`
- `docs/lcd/work/subdir-entries/JOURNAL.md`
- `.claude/rules/lcd-conventions.md`
<!-- /lcd-resume -->

---

## Acceptance criteria

**AC-47** (surfaces: none): an xbank entry whose CrtUsage word carries the subdirectory bit decodes, loads and saves back with the bit set; a load-and-save of a cart holding one is byte-identical.

**AC-48** (surfaces: CLI): `edit --rename` on a cart holding a subdirectory-flagged xbank produces exactly the bytes `ndeft ren` produces for the same request, so the bit survives an edit.

**AC-49** (surfaces: RENDER): `list` marks a subdirectory-flagged entry with `S` in the flags column.

**AC-50** (surfaces: none): the subdirectory bit on a cartridge kind other than xbank is refused as malformed by both read paths, naming the slot, as the reference tool refuses it.

## LOG (append-only; not read on resume)

- 2026-10-11 15:00 — Triage: Standard (5 signals). User picked "carry them" over "carry + create"
  and "research first".
- 2026-10-11 15:20 — Experiment (scratchpad, `ndeft` @ ndefpack build): patched `cult16.crt` slot 1
  to type `0x1A` (with and without the bit), `cxb16.crt` slot 1 with the bit, `cult16.crt` slot 1
  with the bit on Ultimax 16k. `ndeft` refuses both `0x1A` carts ("entry 1 has errors" ←
  `easyfs.c:500-502` "unsupported entry type") and the bit on Ultimax 16k (`easyfs.c:412`). The
  xbank one lists as plain "xbank 16k"; `ndeft ren … 0 hello2` changed one byte (the name), so the
  bit is preserved. Narrowed scope accordingly and told the user.
- 2026-10-11 15:50 — refine: EDIT BOUNDARY rewritten as one full path per bullet (the hook
  read `cartridge.rs` literally) and widened to the two new test targets `tests/subdir.rs` (core)
  and `tests/subdir_cli.rs` (cli), whose stems stay unique across crates.
- 2026-10-11 16:00 — refine: boundary + `tests/edit.rs`, `tests/list.rs` (cli): their `DirEntry`
  literals need the new `subdir` field. Mechanical only.
- 2026-10-11 16:15 — S3+S4: four red tests (core `subdir`, cli `subdir_cli`), then `subdir` on
  `DirEntry`/`Cartridge`, `Slot::MisplacedSubdirBit` → `ReadError::SubdirectoryBitMisplaced`, `S`
  flag. `edit --rename 0 hello2` matches `ndeft ren` byte for byte. No build-side guard for the bit
  on a non-xbank: only `read` sets it, and only on an xbank. Gate green.
- 2026-10-11 16:30 — audit: PASS (test-presence). AC-47..AC-50 tokens present (core `subdir`, cli
  `subdir_cli`); full suite green.
- 2026-10-11 16:40 — reconciled: folded AC-47..AC-50 → SPEC.md/CLI (edit refusal row), RENDER (listing row), none (read-paths and inspect/load rows); all rewrites in place, no new rows. D-026 status notes D-029.
