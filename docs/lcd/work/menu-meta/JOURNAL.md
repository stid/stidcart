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
- **Next action:** done; PR open, merge when gate, gui and claude-review are green (user authorised). The GUI live check is left for the user (listed in the final summary)
- **Branch:** feat/menu-meta · **Updated:** 2026-11-08

## STEPS

- [x] S1 — triage (Deep), D-054, AC-153..AC-159 claimed, spec.md (user-approved draft)
- [x] S2 — plan.md (layout v1, D-055)
- [x] S3 — format-doc section, its own commit (9d9eecf)
- [x] S4 — tasks.md (T1..T24, T-audit, T-final)
- [x] S5 — failing tests T1..T6
- [x] S6a — core T7..T11
- [x] S6b — CLI T12..T13
- [x] S6c — GUI T14..T22
- [x] S6d — docs T23..T24; both gates green, `STIDCART_REQUIRE_TOOLS=1`
- [x] S7 — audit.md PASS
- [ ] S8 — the user: live check, reconcile, closeout, version bump, PR ← next

## DECISIONS (this work-item)

- hidden `STIDCART.META` entry, carried per entry, versioned, split from `menu-folders` → D-054
- layout v1, 16 KB cap, last-candidate rule, kept-META name-prefix guard → D-055

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
- 2026-11-08 — Plan (approved by delegation). META stays out of `entries()` and is made at build
  by `placed_entries()`. Layout v1: a 13-byte header, a 16-byte group table, then length-prefixed
  records keyed by index + raw slot name. Payload capped at 16 KB (`$4000-$7FFF`). A kept
  unknown-version META refuses a build once the loaded names stop being a prefix, or once meta is
  set. `toml` 1.2.1 in the CLI only, parsed to `toml::Table`. D-055. Worked as local branch
  `wt/menu-meta` (the main checkout holds `feat/menu-meta`), pushed to `feat/menu-meta`.
- 2026-11-08 — Format section committed first (9d9eecf), then failing tests (415256d). The
  pipeline went red to green in three flips: core (2486f1c), CLI (a5a816d), GUI (5230f79).
  - T21 was folded into `gui/src-tauri/tests/meta.rs`, so the parity scenario stays in the
    Test scope.
  - Test fix: the list-golden comparison now reads the goldens' exit/stdout/stderr framing.
  - The edit hook's prettier ran with default options and rewrote `Listing.test.ts` in double
    quotes in 415256d. Restored in 5230f79.
  - `menu_boot` `ac_104` failed once under the full parallel suite, then passed alone and in
    every later full run: VICE timing, unrelated to this item.
  - Gates green with `STIDCART_REQUIRE_TOOLS=1`: root 287, GUI Rust 45, vitest 66. Audit PASS.
  - The GUI live check of selection and the panel is left to the user.
  - D-056 belongs to menu-highmem, so the next free id is D-057.

## DEEP PIPELINE (Deep lane only)

> Phase tracker for the Deep-lane pipeline. Files live beside this JOURNAL.

`spec.md ✅  plan.md ✅  tasks.md ✅  tests ✅  red-green ✅  audit.md ✅`

<!-- mark ✅ done · ⏳ in progress · ⬜ not started -->
- 2026-11-08 — orchestrator: merged main (menu-resume) into the branch; DECISIONS, triage-log,
  conventions and Cargo.lock conflicts resolved by keeping both sides. Fixed the selection risk the
  build reported: the listing (and its selected row) survived opening another cart or New, so the
  panel showed the entry at the same index of the next cart; the viewer now counts documents and
  the window keys the listing on it (`viewer.test.ts`, AC-159). Selection after a remove stays on
  the same row position, which then holds the next entry, as a list cursor does. Version
  0.20.0 → 0.21.0.
- 2026-11-08 — reconciled: folded AC-153..AC-155 → SPEC.md/none, AC-156..AC-157 → SPEC.md/CLI,
  AC-158..AC-159 → SPEC.md/RENDER (three new rows). Closeout line appended. The GUI live check
  (drive the app, select rows, edit fields) was not done by an agent; it is left for the user.
