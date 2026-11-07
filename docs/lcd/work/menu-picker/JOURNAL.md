# JOURNAL — menu-picker

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the _entire_
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->

## NOW

- **Lane:** Standard
- **Goal:** Choose the cart's C64 menu (EasyLoader or stidcart) in the desktop app, as an undoable edit with `edit --menu`'s refusals (D-042)
- **Next action:** S5, live check in the app, audit, reconcile, PR
- **Branch:** feat/menu-picker · **Updated:** 2026-11-07

## STEPS

- [x] S1 — version 0.12.0 → 0.13.0 (first commit on the branch)
- [x] S2 — triage, D-042, AC-112..AC-114 claimed, this JOURNAL
- [x] S3 — failing tests: Rust `gui/src-tauri/tests/menu.rs` (AC-112, AC-113), vitest `Listing.test.ts` (AC-112, AC-114)
- [x] S4 — `Edit::SetMenu` in `document.rs`, `setMenu` in `types.ts`, the picker in `Listing.svelte`
- [ ] S5 — README desktop-app section, GUI gate, live check in the app, audit, reconcile, PR

## DECISIONS (this work-item)

- The picker is a `<select>` on editable carts; a switch is one edit through the core's refusals → D-042

## OPEN QUESTIONS

- none

## EDIT BOUNDARY (paths this work may touch)

- `gui/src-tauri/src/document.rs`
- `gui/src-tauri/tests/menu.rs`
- `gui/src/lib/Listing.svelte`
- `gui/src/lib/Listing.test.ts`
- `gui/src/lib/types.ts`
- `README.md`
- `Cargo.toml`
- `Cargo.lock`
- `gui/src-tauri/Cargo.toml`
- `gui/src-tauri/Cargo.lock`
- `docs/lcd/work/menu-picker/JOURNAL.md`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/SPEC.md`
- `docs/lcd/triage-log.md`
- `.claude/rules/lcd-conventions.md`

<!-- /lcd-resume -->

---

## Acceptance criteria

**AC-112** (surfaces: RENDER): Given an editable cart, when the window shows it, then the menu is a
choice between EasyLoader and stidcart with the cart's own selected. Choosing the other applies it
as one edit: the listing names the new menu, the cart is marked unsaved, undo gives back the
previous menu, and saving writes exactly what `edit --menu` writes.

**AC-113** (surfaces: RENDER): Given a cart our menu cannot run (an Ocean entry, or a program that
loads into `$D000-$DFFF`), when stidcart is chosen, then the window shows `edit`'s refusal naming
the entry, and the cart is unchanged: same menu, not marked unsaved, nothing new to undo.

**AC-114** (surfaces: RENDER): Given a cart the window shows view-only, or a listing shown without
edits, then the menu is shown as text and offers no choice.

## LOG (append-only; not read on resume)

- 2026-11-07 — Triage: Standard (2 signals). The user asked how to choose the menu in the app
  (the box only shows it, D-037) and then to start this slice. The core already does the switch
  (`CartBuilder::set_menu`) and the refusals (`build`), so the window adds one edit kind and a
  picker → D-042. AC-112..AC-114 claimed, next free AC-115.
- 2026-11-07 — Red run: Rust didn't compile (no `Edit::SetMenu`), vitest failed AC-112 and AC-113
  (no picker), and AC-114 passed, since the text-only box was already that behaviour. Green after
  one pass. The Ocean refusal in the window contains the exact text `edit --menu` prints. GUI gate
  33/0 cargo + 43/0 vitest, root 225/0. Two snags: the GUI workspace has its own `cargo fmt`, and a
  doc comment on one `Edit` variant made rustfmt expand the whole enum, so the note moved to the
  enum's doc. Prettier's `--write` on `types.ts` would switch the file to double quotes; the repo
  doesn't run Prettier, so that was reverted.

