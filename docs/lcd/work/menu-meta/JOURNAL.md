# JOURNAL — menu-meta

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the _entire_
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->

## NOW

- **Lane:** Deep
- **Goal:** Entries carry a title, year, author, note and group, stored in a hidden `STIDCART.META` entry, round-tripped, edited by `edit` flags, a TOML import and the window, and listed (D-054)
- **Next action:** run /lcd:plan menu-meta
- **Branch:** feat/menu-meta · **Updated:** 2026-11-08

## STEPS

- [x] S1 — triage (Deep), D-054, AC-153..AC-159 claimed, spec.md (user-approved draft)
- [ ] S2 — plan.md ← next
- [ ] S3 — tasks.md, failing tests, red-green, audit, reconcile, PR (version bump just before the PR)

## DECISIONS (this work-item)

- hidden `STIDCART.META` entry, carried per entry, versioned, split from `menu-folders` → D-054

## OPEN QUESTIONS

- none

## EDIT BOUNDARY (paths this work may touch)

- `docs/format/easyflash-cart-format.md`
- `crates/stidcart-core/`
- `crates/stidcart-cli/`
- `gui/src-tauri/`
- `gui/src/`
- `Cargo.toml`
- `Cargo.lock`
- `README.md`
- `docs/lcd/SPEC.md`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/MAP.md`
- `docs/lcd/triage-log.md`
- `docs/lcd/work/menu-meta/`
- `.claude/rules/lcd-conventions.md`

<!-- /lcd-resume -->

---

## Acceptance criteria

See `spec.md` (AC-153..AC-159).

## LOG (append-only; not read on resume)

- 2026-11-08 — Triage: Deep (5 signals; hard: a new on-flash layout, a new dependency for the
  import, parallel CLI and RENDER surfaces; risk: multi-session). Fourth of the user's menu
  improvements; the user approved the spec draft on 2026-11-08. Recon (2026-11-08): the metadata
  goes in a hidden program entry, which EasyLoader skips (`ndefpack/src/fw/easyloader.asm:900`,
  `:935-941`) and `ndeft` round-trips (format doc, measured 2026-08-10). The loader slot was
  rejected (about 3 KB spare after the menu keys; whole-slot fingerprint, `reserved.rs:88-107`);
  bank 0 too (`verify()` checks the boot region byte for byte). The menu side is split off as
  `menu-folders` (AC-166..AC-169), so this item runs while menu-resume and menu-highmem hold
  `menu/`. AC-153..AC-159 claimed; AC-160..AC-165 reserved for menu-highmem.

## DEEP PIPELINE (Deep lane only)

> Phase tracker for the Deep-lane pipeline. Files live beside this JOURNAL.

`spec.md ✅  plan.md ⬜  tasks.md ⬜  tests ⬜  red-green ⬜  audit.md ⬜`

<!-- mark ✅ done · ⏳ in progress · ⬜ not started -->
