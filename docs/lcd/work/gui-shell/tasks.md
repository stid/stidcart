# Tasks: Desktop cartridge viewer (GUI, first slice)

> **Deep lane · Phase 3 of 6 (Tasks).** A flat, dependency-ordered checklist. Each task names
> the file it touches. Tests come before the code they test (TDD). The granular STEPS in the
> work-item `JOURNAL.md` mirror this list as the resume view; keep them in sync.

**Slug:** `gui-shell`
**Spec:** [`spec.md`](./spec.md) • **Plan:** [`plan.md`](./plan.md) • **Journal:** [`JOURNAL.md`](./JOURNAL.md)
**Created:** 2026-10-11
**Status:** in_progress <!-- in_progress | completed | abandoned -->

---

## Conventions

- `[P]` suffix = parallelizable with the previous task (different file, no shared state).
- Tests precede implementation tasks they cover.
- One file per task where possible; tasks naming multiple files should be split unless the change is genuinely atomic.
- Every AC test carries the literal `AC-N (RENDER)` (or `AC-58 (none)`) in its name or doc comment.

## Tasks

### A. One listing in the core (D-032), with `list` unchanged

- [ ] **T1**: Characterisation golden: `crates/stidcart-cli/tests/list_golden.rs` runs `stidcart list` on every `fixtures/*.crt` and compares stdout, stderr and exit code to `crates/stidcart-cli/tests/golden/<fixture>.txt`. `UPDATE_GOLDEN=1` writes them. Generate and commit the goldens (atomic: test + data). It is green from the start and is the no-downgrade guard for T4.
- [ ] **T2**: Write failing tests `AC-51 (RENDER): describe yields list's values` in `crates/stidcart-core/tests/listing.rs`. For each listable fixture, `listing::describe` rows, cart, built_by and `free_text()` match the values in the T1 golden. A refused fixture returns the same `ReadError` text that `list` prints. An image `load` refuses still describes, with the inspect free figure.
- [ ] **T3**: Implement `crates/stidcart-core/src/listing.rs` (`Listing`, `Row`, `describe`, `free_text`, with `kind_label`/`layout`/flags/`printable` moved verbatim) plus the one-line `pub mod listing;` in `crates/stidcart-core/src/lib.rs`, to pass T2.
- [ ] **T4**: Refactor `crates/stidcart-cli/src/main.rs`: `list` calls `listing::describe`, `render(&Listing)` keeps only layout and pluralisation, and the moved helpers are deleted. T1 and the existing `list.rs` tests stay green, and the root gate stays green.

### B. GUI scaffold and runner wiring

- [ ] **T5**: Scaffold the Rust crate. `gui/src-tauri/Cargo.toml` (stidcart-gui 0.7.0, GPL-2.0-or-later, own `[workspace]`, path dep `stidcart-core`, tauri 2.12, tauri-plugin-dialog 2.x, serde), `gui/src-tauri/build.rs`, `gui/src-tauri/tauri.conf.json` (overlay title bar, `frontendDist ../dist`, `devUrl`), `gui/src-tauri/capabilities/default.json` (dialog open + core defaults), `gui/src-tauri/icons/`, `gui/src-tauri/src/main.rs` and a stub `gui/src-tauri/src/lib.rs` with `run()`, plus `gui/src-tauri/Cargo.lock`. Root `Cargo.toml` gets `exclude = ["gui/src-tauri"]` (atomic: the crate doesn't resolve without it). Check: root `cargo test --workspace` doesn't list `stidcart-gui`, and `cargo --version` in `gui/src-tauri` is 1.97.1.
- [ ] **T6**: Scaffold the frontend. `gui/package.json` (private, GPL-2.0-or-later, scripts `dev`/`build`/`check`/`test`/`tauri`) with exact versions checked for peer compatibility (`npm view … peerDependencies`), `gui/package-lock.json`, `gui/vite.config.ts` (svelte plugin, Tauri dev-server settings, vitest jsdom + setup), `gui/svelte.config.js`, `gui/tsconfig.json` (strict, resolveJsonModule), `gui/index.html`, `gui/src/main.ts`, a stub `gui/src/App.svelte`, `gui/src/test-setup.ts`, `gui/.gitignore`, and root `.gitignore` if needed. Check: `npm run check`, `npm test` (no tests yet, passWithNoTests) and `npm run build` succeed, and `cargo build` in `gui/src-tauri` compiles against `dist`.
- [ ] **T7**: Write `docs/lcd/ci/gui-gate.sh` (npm ci when needed → check → test → build → cargo fmt/clippy `-D warnings`/test in `gui/src-tauri`). It passes on the scaffold.
- [ ] **T8** [P]: Write `docs/lcd/ci/run-test.sh {path}`, which dispatches a stem to root cargo, `gui/src-tauri/tests/<stem>.rs` to GUI cargo, and `*.test.ts` to vitest.
- [ ] **T9**: Update `.claude/rules/lcd-conventions.md` machine keys (`test-discovery-glob`, `scoped-test`, `bail`, `single-test` notes, `gate` + `gui-gate.sh`, test-placement `gui`, the stem note including the new core stem `listing` and the `list_golden` cli stem), then run `/lcd:doctor` until it's clean.

### C. GUI Rust side

- [ ] **T10**: Write failing tests in `gui/src-tauri/tests/open.rs`:
  - `AC-51 (RENDER)`: for every fixture, `stidcart_gui::open(path)` equals the DTO mapped from `listing::describe`, including `freeText`.
  - `AC-54 (RENDER)`: for crafted inputs (EasyFS1, truncated, not a CRT, other hardware), the error text equals `list`'s reason, and a missing file gives `reading '<path>': …`.
  - `AC-58 (none)`: file bytes are identical before and after `open`, on success and on refusal.
- [ ] **T11**: Implement `gui/src-tauri/src/lib.rs` (`open`, `ListingDto`/`RowDto` serde camelCase, `#[tauri::command] open_cartridge`, `run()` registering the dialog plugin and the command) to pass T10.
- [ ] **T12**: Write the contract test `gui/src-tauri/tests/contract.rs`: the serialized DTO for `two.crt` and for one refusal equals `gui/src/lib/fixtures/two.json` / `refused.json`, and `UPDATE_GOLDEN=1` writes them. Generate and commit the two JSON files (atomic: test + data).

### D. Frontend

- [ ] **T13**: Write `gui/src/lib/types.ts` (`Listing`, `Row`, `ViewState`, `Host`) with a type-level check that the golden JSON is assignable to `Listing`, enforced by `npm run check`.
- [ ] **T14**: Write failing tests in `gui/src/lib/viewer.test.ts` with a fake `Host`:
  - `AC-52 (RENDER)`: a single drop opens and shows the listing.
  - `AC-53 (RENDER)`: picking a file opens it, and dismissing the chooser leaves the state unchanged.
  - `AC-54 (RENDER)`: a refusal replaces the previous listing with an error state.
  - `AC-55 (RENDER)`: a valid open after an error clears the error.
  - `AC-56 (RENDER)`: a multi-file drop opens nothing, sets the "drop one file" notice and leaves the state unchanged.
- [ ] **T15**: Implement `gui/src/lib/viewer.svelte.ts` (rune-based state machine) to pass T14.
- [ ] **T16**: Write failing test `AC-51 (RENDER): every listing value is shown` in `gui/src/lib/Listing.test.ts`. It renders from `fixtures/two.json` and checks the cart, built-by, each row's index/name/flags/kind/bank/layout/size, the entry count, `freeText` and the file name.
- [ ] **T17**: Implement `gui/src/lib/Listing.svelte` to pass T16.
- [ ] **T18** [P]: Write failing test `AC-57 (RENDER): empty state invites open or drop` in `gui/src/lib/EmptyState.test.ts`.
- [ ] **T19**: Implement `gui/src/lib/EmptyState.svelte` to pass T18.
- [ ] **T20** [P]: Write failing test `AC-54 (RENDER): refusal shows the reason and no rows` in `gui/src/lib/ErrorState.test.ts`, using `fixtures/refused.json`.
- [ ] **T21**: Implement `gui/src/lib/ErrorState.svelte` to pass T20.
- [ ] **T22**: Implement `gui/src/lib/host.ts`, the real adapter: plugin-dialog `open` with a `.crt` filter, `onDragDropEvent` drop → paths, and `invoke('open_cartridge')`. It is covered by the live check (T27), not unit tests.
- [ ] **T23**: Write `gui/src/styles/tokens.css`: design tokens, light/dark through `prefers-color-scheme`, the system UI and monospace stacks, tabular numerals.
- [ ] **T24**: Wire `gui/src/App.svelte`: header drag region, file name, Open button, notice, and a switch over `empty | listing | error`. Pass the host as a prop.
- [ ] **T25**: Wire `gui/src/main.ts` to mount `App` with the real host and import `tokens.css`.

### E. CI, live check, docs

- [ ] **T26**: Add the `gui` job to `.github/workflows/ci.yml`: ubuntu-latest, `rustup toolchain install`, apt (`libwebkit2gtk-4.1-dev` plus what the build proves it needs), `actions/setup-node` with an npm cache, `Swatinem/rust-cache` on `gui/src-tauri`, then `docs/lcd/ci/gui-gate.sh`.
- [ ] **T27**: Live check (RENDER). Run `npm run tauri dev` in `gui/`, then cover each state: empty, open `two.crt` through the chooser, drop `cxb16sub.crt`, drop two files, open `in_8k.crt` (refused), then open a valid file. Capture each state with `screencapture` and review against the polish bar, with light and dark. Record the outcome in the JOURNAL LOG. Fixes go back to the owning file's task.
- [ ] **T28**: Update `docs/lcd/MAP.md`: `gui` zone row, entry point, Surfaces (RENDER has two renderers), core zone mentions `listing`.
- [ ] **T29** [P]: Update `CLAUDE.md`: the GUI gate beside the root gate, how to run it locally, and that `frontendDist` must be built before `cargo` in `gui/src-tauri`.
- [ ] **T30** [P]: Update `README.md` with a short GUI section: what it does and how to run `npm run tauri dev`.
- [ ] **T31**: Open the PR and confirm the `gate`, `gui` and `claude-review` checks are green. **Ask the user**, then add `gui` to `main`'s required status checks (branch protection).
- [ ] **T-audit**: Run `/lcd:audit gui-shell`. The resulting `audit.md` has zero MISSING/BLOCKED rows.
- [ ] **T-reconcile**: `lcd:reconcile` folds AC-51..58 into `docs/lcd/SPEC.md` (a RENDER row for the viewer; the CLI `list` row notes the shared listing).
- [ ] **T-final**: Run the root gate (`cargo fmt --all --check && cargo clippy --workspace --all-targets --all-features -- -D warnings && cargo test --workspace`) and `docs/lcd/ci/gui-gate.sh`. All green.

## Checkpoint validation

After every block of related tasks, the suite must stay green. If T3 passes but T5's implementation breaks T1, stop and fix before moving on. Block A ends with the root gate green and T1 unchanged. Block B ends with both gates green on an empty app.

## Linkbacks

- `AC-51 (RENDER)` → T1, T2, T3, T4, T10, T11, T16, T17
- `AC-52 (RENDER)` → T14, T15, T22, T27
- `AC-53 (RENDER)` → T14, T15, T22, T27
- `AC-54 (RENDER)` → T10, T11, T14, T15, T20, T21, T27
- `AC-55 (RENDER)` → T14, T15, T27
- `AC-56 (RENDER)` → T14, T15, T27
- `AC-57 (RENDER)` → T18, T19, T27
- `AC-58 (none)` → T10, T11
- Contract (plan: golden JSON) → T12, T13
- Plan infrastructure (no AC) → T5–T9, T23–T26, T28–T31

---

<!--
Quality checks before starting implementation:
  - Every AC × non-`none`-surface row in plan.md has at least one test task above
  - Every new file in plan.md "File structure" has at least one task that creates it
  - Tests appear before their implementation
  - T-audit appears before T-final (the audit gate must pass before suite-final)
  - T-final exists (run the suite, lint, type check)
-->
