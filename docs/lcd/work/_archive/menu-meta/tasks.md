# Tasks: Entry metadata and groups

> **Deep lane · Phase 3 of 6 (Tasks).** A flat, dependency-ordered checklist. Each task names
> the file it touches. Tests come before the code they test (TDD). The granular STEPS in the
> work-item `JOURNAL.md` mirror this list as the resume view; keep them in sync.

**Slug:** `menu-meta`
**Spec:** [`spec.md`](./spec.md) • **Plan:** [`plan.md`](./plan.md) • **Journal:** [`JOURNAL.md`](./JOURNAL.md)
**Created:** 2026-11-08
**Status:** completed

---

## Conventions

- `[P]` suffix = parallelizable with the previous task (different file, no shared state).
- Tests precede implementation tasks they cover.
- One file per task where possible.
- T0 (the format section, D-018) is done: commit `9d9eecf`.

## Tasks

Failing tests (one commit, phase 4):

- [x] **T1**: `AC-153 (none)`, `AC-154 (none)`, `AC-155 (none)` core tests in `crates/stidcart-core/tests/meta.rs`. They cover the round trip through load/save, remove, a re-order, rename and a menu switch; a no-metadata set byte-identical to today, and set-then-clear giving the original bytes; dropped records counted; an unknown version kept, described and its rewrites refused; the slot count; not listed; the 16 KB cap; field limits and mapping
- [x] **T2** [P]: `AC-155 (none)` VICE boots in `crates/stidcart-cli/tests/meta_boot.rs`: neither menu lists META
- [x] **T3** [P]: `AC-156 (CLI)`, `AC-157 (CLI)`, `AC-158 (RENDER)` in `crates/stidcart-cli/tests/meta_cli.rs`
- [x] **T4** [P]: `AC-158 (RENDER)` and `AC-159 (RENDER)` Rust side in `gui/src-tauri/tests/meta.rs`: DTO values, `SetMeta` as one undoable step, refusal unchanged, save equals `edit`
- [x] **T5** [P]: `AC-158 (RENDER)` in `gui/src/lib/Listing.test.ts`: the title under the name, the Group column only when used, selection
- [x] **T6** [P]: `AC-159 (RENDER)` in `gui/src/lib/MetaPanel.test.ts`: five fields, one `setMeta` per commit

Implementation (red-green, phase 5):

- [x] **T7**: `crates/stidcart-core/src/meta.rs`: `Field`, `Meta`, `MetaError`, `KeptMeta`, encode/decode v1 (T1)
- [x] **T8**: `crates/stidcart-core/src/cartridge.rs`: `meta` field on `Cartridge` (T1)
- [x] **T9**: `crates/stidcart-core/src/lib.rs`: `meta` on `Program`, `Entry::meta`, `set_meta`, `replace` keeps meta, `placed_entries`, the new build errors (T1)
- [x] **T10**: `crates/stidcart-core/src/read.rs`: the walk finds, lifts and decodes META; `CartInfo` fields; `load` attaches (T1)
- [x] **T11**: `crates/stidcart-core/src/listing.rs`: row fields, dropped count, kept note (T1, T3)
- [x] **T12**: `crates/stidcart-cli/Cargo.toml` and `Cargo.lock`: `toml` 1.2.1 (T3)
- [x] **T13**: `crates/stidcart-cli/src/lib.rs`: metadata flags, `--import`, render lines, usage (T3)
- [x] **T14**: `gui/src-tauri/src/lib.rs`: DTO fields (T4)
- [x] **T15**: `gui/src-tauri/src/document.rs`: `Edit::SetMeta` (T4)
- [x] **T16**: `gui/src-tauri/Cargo.lock`: picks up `toml` through the CLI (T4)
- [x] **T17**: `gui/src/lib/fixtures/two.json` and `gui/src/lib/fixtures/document.json`: regenerated with `UPDATE_GOLDEN=1` (`gui/src-tauri/tests/contract.rs`)
- [x] **T18**: `gui/src/lib/types.ts`: `Row`, `Listing`, `Edit` mirror the DTO (T5, T6)
- [x] **T19**: `gui/src/lib/MetaPanel.svelte`: the selected entry's fields (T6)
- [x] **T20**: `gui/src/lib/Listing.svelte`: the title line, the Group column, selection, the panel (T5)
- [x] **T21**: folded into T4: the save-equals-`edit` scenario sits in `gui/src-tauri/tests/meta.rs` (in the Test scope), on the AC-80 harness pattern
- [x] **T22**: `gui/src/lib/viewer.test.ts`: no change needed (svelte-check clean against the regenerated goldens)
- [x] **T23**: `README.md`: the new `edit` options and the import file
- [x] **T24**: `docs/lcd/MAP.md`: `meta.rs` in the core zone

Close:

- [x] **T-audit**: `LCD_SPECS_DIR=docs/lcd/work docs/lcd/ci/audit-crosspath.sh menu-meta`, saved as `audit.md`, with zero MISSING/BLOCKED rows
- [x] **T-final**: `cargo fmt --all --check && cargo clippy --workspace --all-targets --all-features -- -D warnings && cargo test --workspace` (once with `STIDCART_REQUIRE_TOOLS=1`) and `docs/lcd/ci/gui-gate.sh`, all green

## Checkpoint validation

After every block of related tasks, the suite must stay green. If T3 passes but T5's implementation breaks T1, stop and fix before moving on.

## Linkbacks

- `AC-153 (none)` → T1, T7, T9, T10
- `AC-154 (none)` → T1, T7, T10, T11
- `AC-155 (none)` → T1, T2, T9, T10
- `AC-156 (CLI)` → T3, T7, T9, T13
- `AC-157 (CLI)` → T3, T12, T13
- `AC-158 (RENDER)` → T3, T4, T5, T11, T13, T14, T17, T18, T20
- `AC-159 (RENDER)` → T4, T6, T15, T18, T19, T20, T21
