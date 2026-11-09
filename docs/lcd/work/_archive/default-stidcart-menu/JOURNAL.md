# JOURNAL — default-stidcart-menu

<!-- lcd-resume:v1 -->

## NOW

- **Lane:** Standard
- **Goal:** A new cart gets our stidcart menu unless told otherwise, from `build` and from the window's New Cart; content our menu can't run is refused with a pointer to EasyLoader.
- **Next action:** done; PR open, merge and deploy when the user says
- **Branch:** feat/default-stidcart-menu · **Updated:** 2026-11-08

## STEPS

- [x] S1 — failing tests for AC-197..AC-199
- [x] S2 — `build` defaults to stidcart; window New Cart starts with stidcart
- [x] S3 — refusal text points to EasyLoader
- [x] S4 — tests that assert EasyLoader/ndeft behaviour name `--menu easyloader`
- [x] S5 — version, SPEC, README, D-064, gates, PR

## DECISIONS (this work-item)

- The default changes at the product surfaces (`build`, New Cart), not in the core's `Menu::default()`, which the core's ndeft parity proofs build with → D-064

## OPEN QUESTIONS

- none

## EDIT BOUNDARY (paths this work may touch)

- `Cargo.toml`
- `Cargo.lock`
- `gui/src-tauri/Cargo.toml`
- `gui/src-tauri/Cargo.lock`
- `crates/stidcart-core/src/lib.rs`
- `crates/stidcart-cli/src/lib.rs`
- `crates/stidcart-cli/tests/`
- `gui/src-tauri/src/document.rs`
- `gui/src-tauri/tests/`
- `gui/src/lib/`
- `README.md`
- `docs/lcd/`
- `.claude/rules/lcd-conventions.md`

<!-- /lcd-resume -->

---

## Acceptance criteria

**AC-197** (surfaces: CLI): `build` naming no menu writes our stidcart menu into the loader slot (`list` says `menu: stidcart`); `build --menu easyloader` writes EasyLoader, byte for byte what `ndeft` wrote for the same request.

**AC-198** (surfaces: CLI): `build` naming no menu, given content our menu can't run (an Ocean cartridge, a program loading at `$D000` or above), is refused naming the entry and saying EasyLoader can start it; nothing is written. The same request with `--menu easyloader` builds.

**AC-199** (surfaces: RENDER): The window's New Cart starts with our stidcart menu: its picker shows stidcart, and saving it untouched writes what `build` naming no menu writes for no inputs.

## LOG (append-only; not read on resume)

- 2026-11-08 — Triage Standard. The user asked for stidcart as the default menu, in both `build` and New Cart (Claude's recommendation). Folder carts stay EasyLoader-compatible (META hidden, AC-155).
- 2026-11-08 — AC-197..AC-199 green (build and New Cart default to stidcart; refusals say EasyLoader can start it). Root suite: 8 older CLI tests fail because they assumed EasyLoader as build's default. Session paused at the user's usage limit.
- 2026-11-08 — Resumed. Fixed the eight CLI tests (EasyLoader/ndeft cases name `--menu easyloader`; AC-82 renamed: its "no request means EasyLoader" is superseded by D-064, its edit half stands; AC-28 compares the default build with a core build under our menu) and three GUI tests (AC-77's empty cart has our menu; menu tests' EasyLoader carts are set up as EasyLoader). Both gates green with VICE required. D-064 logged, D-037 marked; SPEC rows, README updated; 0.29.0.
- audit: PASS (test-presence) — AC-197 (CLI), AC-198 (CLI) in `crates/stidcart-cli/tests/default_menu.rs`, AC-199 (RENDER) in `gui/src-tauri/tests/default_menu.rs`; full suites green.

## DEEP PIPELINE (Deep lane only)

n/a — Standard lane.
