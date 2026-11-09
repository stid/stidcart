# JOURNAL — default-stidcart-menu

<!-- lcd-resume:v1 -->

## NOW

- **Lane:** Standard
- **Goal:** A new cart gets our stidcart menu unless told otherwise, from `build` and from the window's New Cart; content our menu can't run is refused with a pointer to EasyLoader.
- **Next action:** S4: fix 8 older CLI tests that assumed EasyLoader as build's default: boot ac_4, carts ac_40, cli ac_5, edit ac_28, menu_cli ac_81/82/95/163 (add `--menu easyloader` where the test is about EasyLoader/ndeft; re-read ac_82/ac_95/ac_163, whose intent changes); then run gui tests, D-064, SPEC, README, version 0.29.0, gates, PR
- **Branch:** feat/default-stidcart-menu · **Updated:** 2026-11-08

## STEPS

- [x] S1 — failing tests for AC-197..AC-199
- [x] S2 — `build` defaults to stidcart; window New Cart starts with stidcart
- [x] S3 — refusal text points to EasyLoader
- [ ] S4 — tests that assert EasyLoader/ndeft behaviour name `--menu easyloader` ← next
- [ ] S5 — version, SPEC, README, D-064, gates, PR

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

## DEEP PIPELINE (Deep lane only)

n/a — Standard lane.
