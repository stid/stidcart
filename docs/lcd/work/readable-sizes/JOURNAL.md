# JOURNAL — readable-sizes

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the _entire_
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->

## NOW

- **Lane:** Standard
- **Goal:** Sizes and the free figure read as KB (bytes under 1 KB, MB from 1024 KB) in `list` and the window, with exact bytes on hover and through `list --bytes` (D-044)
- **Next action:** S6, the user's live check in the dev app, audit, reconcile, PR; deploy locally after merge
- **Branch:** feat/readable-sizes · **Updated:** 2026-11-08

## STEPS

- [x] S1 — version 0.13.0 → 0.14.0 (first commit on the branch)
- [x] S2 — triage, D-044, AC-115..AC-117 claimed, this JOURNAL
- [x] S3 — failing tests: core `readable_size` cases, `list` readable and `--bytes` (cmixed), the window's hover
- [x] S4 — core `readable_size`, `Row.size_bytes`, `free_text`; `list --bytes`; DTO `sizeBytes`; hover titles
- [x] S5 — regenerate `list` goldens and the GUI fixtures, reviewing the diff; AC-46 reads `--bytes`
- [ ] S6 — README, both gates, deploy the app locally for the user, audit, reconcile, PR

## DECISIONS (this work-item)

- KB with one decimal, 1024-based, bytes under 1 KB, MB from 1024 KB; exact bytes on hover and with `--bytes` → D-044

## OPEN QUESTIONS

- none

## EDIT BOUNDARY (paths this work may touch)

- `crates/stidcart-core/src/listing.rs`
- `crates/stidcart-core/tests/listing.rs`
- `crates/stidcart-cli/src/lib.rs`
- `crates/stidcart-cli/tests/list.rs`
- `crates/stidcart-cli/tests/cartlist.rs`
- `crates/stidcart-cli/tests/list_golden.rs`
- `crates/stidcart-cli/tests/golden`
- `gui/src-tauri/src/lib.rs`
- `gui/src-tauri/tests/contract.rs`
- `gui/src/lib/types.ts`
- `gui/src/lib/Listing.svelte`
- `gui/src/lib/Listing.test.ts`
- `gui/src/lib/fixtures`
- `README.md`
- `Cargo.toml`
- `Cargo.lock`
- `gui/src-tauri/Cargo.toml`
- `gui/src-tauri/Cargo.lock`
- `docs/lcd/work/readable-sizes/JOURNAL.md`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/SPEC.md`
- `docs/lcd/triage-log.md`
- `.claude/rules/lcd-conventions.md`

<!-- /lcd-resume -->

---

## Acceptance criteria

**AC-115** (surfaces: CLI, RENDER): Given a cartridge, when it is listed by `list` or shown in the
window, then each entry's size and the free figure read as text: under 1024 bytes as bytes ("652
bytes", "1 byte"); otherwise in 1024-based KB with one decimal rounded half up ("1.1 KB", "10.0
KB"), whole for an exact multiple ("8 KB"); and in MB the same way from 1024 KB. Both show the
same text.

**AC-116** (surfaces: CLI): Given a cartridge, when it is listed with `list --bytes`, then each
size and the free figure are the exact byte counts in decimal.

**AC-117** (surfaces: RENDER): Given the window showing a cartridge, then each size and the free
figure carry their exact byte count, shown on hover.

## LOG (append-only; not read on resume)

- 2026-11-08 — Triage: Standard (3 signals). The user found the hex sizes hard to read in the app.
  They chose both `list` and the window (one core description, D-032) and KB with one decimal.
  The format they picked notes exact bytes on hover and through `list --bytes`, so the precision
  of the hex isn't lost → D-044. AC-115..AC-117 claimed, next free AC-118.
- 2026-11-08 — Red run: the core and CLI tests didn't compile (no `readable_size`), and the GUI
  tests had no `sizeBytes`. Green after the core formatter, `Row.size_bytes`, `list --bytes`, the
  DTO field and the hover titles. The regenerated goldens differ only in the size column (now 10
  wide), its header and the free line. The fixtures gained `sizeBytes`, with "1005.0 KB" for
  two.crt's 1,029,088 free bytes, as specified (MB from 1024 KB). Two snags: clippy wants
  `is_multiple_of`, and list.rs's AC-1 split columns on whitespace, which "102 bytes" now breaks,
  so it rejoins the size words. Root 229/0, GUI 35/0 cargo + 44/0 vitest.

