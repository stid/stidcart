# Plan: Desktop cartridge viewer (GUI, first slice)

> **Deep lane · Phase 2 of 6 (Plan).** This document owns **how**. Specify ≠ Plan: if you're
> about to write a user story or acceptance criterion, that belongs in `spec.md`.

**Slug:** `gui-shell`
**Spec:** [`spec.md`](./spec.md) · **Journal:** [`JOURNAL.md`](./JOURNAL.md)
**Created:** 2026-10-11
**Status:** approved <!-- draft | approved | implemented | superseded -->

---

## Goal

A Tauri 2 + Svelte 5 desktop window that opens one `.crt` (chooser or drop) and shows the same
listing `list` prints, from one structured description in the core, with refusals shown as
readable errors and the file never written.

## Architecture

**One listing, two renderers (D-032).** The values `list` prints are computed today by private
helpers in `stidcart-cli/src/main.rs` (`kind_label`, `layout`, the flag letters, `printable`, and
the free figure from `load` with an `inspect` fallback). They move into a new
`stidcart-core::listing` module. `describe(&[u8]) -> Result<Listing, ReadError>` returns the cart
name, the filtered producing-tool name, one `Row` per entry with each value as displayed, and the
free figure. `list` keeps only column widths, the `(no entries)` line and pluralisation. Its output
does not change: a golden snapshot of `list` (stdout, stderr and exit code) for every fixture, including the ones it refuses, is committed **before** the move and
must still pass after it (no-downgrade).

**The GUI is its own Cargo workspace (D-030).** It lives at `gui/`, using Tauri's standard layout:
`gui/` holds the Vite + Svelte 5 + TypeScript frontend and `gui/src-tauri/` the Rust crate
`stidcart-gui`. That crate has its own `[workspace]` table and a path dependency on
`stidcart-core`. The root `Cargo.toml` adds `exclude = ["gui/src-tauri"]`, so the existing `gate`
(`--workspace`) never compiles webkit; this was checked in a scratch workspace (JOURNAL LOG). The
root `rust-toolchain.toml` binds the nested crate too, because rustup walks up the parent
directories.

**The Rust side is thin and testable without a window.**
- `stidcart_gui::open(path: &Path) -> Result<ListingDto, String>` reads the file and calls
  `listing::describe`.
- On failure it formats the error exactly as `list` does: `reading '<path>': <io error>`, or
  `ReadError`'s `Display`.
- The `#[tauri::command] open_cartridge` is a one-line wrapper around `open`, so tests call `open`
  directly rather than going through the unstable `tauri::test` module.
- `ListingDto` derives `Serialize` and lives only in the GUI crate. The core gains no serde.

**The frontend talks to Tauri through one adapter module.** `src/lib/host.ts` holds the only code
that touches `@tauri-apps/*`:
- `pickFile()` uses `@tauri-apps/plugin-dialog`'s `open`, filtered to `.crt`, and returns `null`
  when the chooser is dismissed.
- `onDrop(cb)` uses `getCurrentWebview().onDragDropEvent`, passing the paths on `drop`.
- `openCartridge(path)` uses `invoke('open_cartridge')`.

`App.svelte` receives the host as a prop. View state is a small discriminated union:
`empty | listing | error` plus a transient notice for AC-56. The view logic lives in
`src/lib/viewer.svelte.ts`, a rune-based store, and the components only render it. Vitest
component tests pass a fake host, so neither a webview nor `mockIPC` is needed. This refines
D-030's "Vitest with `mockIPC`" to fakes at the adapter boundary, which is simpler. `mockIPC`
remains available if a later slice needs to test real `invoke` payloads.

**The Rust↔TS contract is pinned by a golden JSON file.**
- A Rust test serializes `ListingDto` for `two.crt` and one refusal, and asserts the result equals
  the committed `gui/src/lib/fixtures/*.json`. Setting `UPDATE_GOLDEN=1` rewrites the files.
- The TS type `Listing` in `src/lib/types.ts` is checked against those same JSON files by
  `svelte-check`, which assigns the imported JSON to the type, and the component tests render from
  them.
- A field renamed on either side fails a test, with no code generator involved (the JOURNAL
  question: closed without tauri-specta).

**The look comes from a design-token stylesheet.** `src/styles/tokens.css` defines CSS custom
properties: colour, spacing, radius and type scale, with light and dark sets switched by
`prefers-color-scheme`. Text uses the system UI stack (`-apple-system, BlinkMacSystemFont, …`) and
`ui-monospace` (SF Mono) for the numeric columns. There are no bundled fonts, which keeps the look
native-pro on macOS with no licence files. The window chrome uses the macOS overlay title bar
(`titleBarStyle: "Overlay"`), as Zed and VS Code do, with a drag region in the header. The table
is a plain `<table>` with sticky headers, tabular numerals and hover/selection states. bits-ui
(D-031) and TanStack Table are **not** added in this slice, since nothing here needs a headless
dialog, a menu or virtualisation. They come in with the editing slice.

**CI gets a second required job, `gui`, on ubuntu-latest.**
- Steps: checkout → `rustup toolchain install` → apt (`libwebkit2gtk-4.1-dev` plus whatever else
  the first run proves the build needs; the Tauri prerequisites list is the ceiling) →
  `actions/setup-node` with an npm cache → `npm ci` → `npm run check` (svelte-check) →
  `npm test` (vitest run) → `npm run build`, which produces `frontendDist` for the
  compile-time check in `generate_context!` → `cargo fmt --check`, `cargo clippy --all-targets -D
  warnings` and `cargo test` in `gui/src-tauri`, with `Swatinem/rust-cache` keyed on that workspace.
- Building the frontend before cargo was chosen over setting `devUrl` in debug builds, because it
  also proves the frontend build itself.
- After the job is green on the PR, `gui` is added to the `main` branch protection's required
  checks. That is an outward-facing change, so it gets a confirmation at that step.

**Rejected:**
- A `stidcart-view` crate, because a module does the same job (D-032).
- The GUI re-deriving values, because two implementations would drift (D-032).
- The GUI as a member of the root workspace with `--exclude` in the gate, because that edits the
  gate string in three places for no gain over `exclude` (JOURNAL LOG).
- Tests through `tauri::test`, because that module is unstable and `open` is callable directly.
- tauri-specta, because the golden JSON pins the contract without a new dependency.
- Bundled Inter or JetBrains Mono, because the system stack is native on the shipping platform.

## Constitution check

| Rule | Applies? | Compliance note |
|---|---|---|
| `testing.md` (tests first) | yes | Each AC test is written failing first. Rust: `cargo test` in core and in `gui/src-tauri`. Frontend: `vitest run`, never watch mode. Full suite (root gate, GUI job) after the multi-file move |
| `no-downgrade.md` | yes | Moving `list`'s helpers into core must leave its output byte-identical. A golden snapshot of `list` stdout for every fixture is committed before the move. No CLI option, column or error text changes |
| `no-overengineering.md` | yes | No bits-ui, TanStack, routing, state library, bundled fonts, menus, packaging or code generator. One adapter module, one store, three components |
| `refinement-protocol.md` | yes | The spec is approved. Any drift found mid-build goes through `lcd:refine`, and changes of intent are escalated |
| `versioning.md` | yes | `feat/` branch → minor bump. Done as its own commit (`1f7d4f1`, `0.6.0 → 0.7.0`, base checked with `git merge-base`), with the root gate green, parity included. The GUI crate's `Cargo.toml` holds the GUI version (`0.7.0`). `tauri.conf.json` omits `version`, so Tauri falls back to Cargo, and `package.json` is `private` with no version |
| `commits.md` | yes | Conventional commits, one per red→green flip in the loop, pushed (user rule) |
| `ac-convention.md`, EVAL coverage | no | No scorer, ranker or LLM output. The viewer mirrors deterministic values, and the golden JSON plus per-fixture equality are presence/equality tests, not quality thresholds |
| MAP invariant: no global state or I/O policy in the core | yes | `listing::describe` takes bytes and returns data. File reading, path formatting and dialogs stay in the CLI and GUI crates. No serde in core |
| MAP invariant: describing vs rebuilding (D-011) | yes | `describe` uses `inspect` for rows, and `load` only for the free figure with the existing fallback. An image `load` refuses is still listed (AC-51 covers such fixtures) |
| MAP invariant: byte parity / crate version in produced bytes | yes | The viewer never writes (AC-58). The version bump is the only byte change, handled as above |
| MAP invariant: zlib/GPL boundary at `firmware/` | yes | Untouched. GUI dependencies are MIT/Apache (D-030). License field `GPL-2.0-or-later` on `stidcart-gui` and in `package.json` |
| MAP invariant: format facts from `docs/format/` | no | No format fact changes. Labels and layout are moved, not redefined |
| `CLAUDE.md`: gate green before every commit; never commit to `main` | yes | The root gate is unchanged and still runs. GUI commits also run the GUI job's steps locally. Work stays on `feat/gui-shell` with a PR, and the required checks become `gate`, `gui` and `claude-review` |
| `CLAUDE.md`: toolchain pin through rustup shims (D-010) | yes | The nested workspace inherits `rust-toolchain.toml`. Check `cargo --version` in `gui/src-tauri` before trusting a result |
| `.claude/rules/lcd-conventions.md` | yes | ACs AC-51..58 claimed (next free AC-59). The machine block knows only the root cargo workspace, but this work adds two more runners, so the keys change in the scaffold task, followed by `/lcd:doctor`. The audit only greps for `AC-N (SURFACE)` tokens and runs no tests (`docs/lcd/ci/audit-crosspath.sh`), so vitest files satisfy it as long as they carry the token. New values: |
| ↳ `test-discovery-glob` | | `crates/*/tests/*.rs` + inline `#[cfg(test)]` + `gui/src-tauri/tests/*.rs` + `gui/src/**/*.test.ts` |
| ↳ `scoped-test` / `bail` | | `docs/lcd/ci/run-test.sh {path}`. A plain stem → `cargo test --workspace --test {stem}`; `gui/src-tauri/tests/<stem>.rs` → `cargo test --manifest-path gui/src-tauri/Cargo.toml --test <stem>`; `*.test.ts` → `npm --prefix gui exec -- vitest run <file>` |
| ↳ `single-test` | | `cargo test --workspace {name}`, unchanged for root. The GUI form is recorded in the notes: `cargo test --manifest-path gui/src-tauri/Cargo.toml {name}` / `vitest run -t {name}` |
| ↳ `gate` | | root gate `&& docs/lcd/ci/gui-gate.sh`. The script runs the GUI job's steps locally (npm ci when the lockfile changed, check, test, build, then fmt/clippy/test in `gui/src-tauri`), and CI's `gui` job calls the same script so the two can't drift. The `test-placement` notes add `gui`, and the stem note says GUI stems live in their own workspace |

## Cross-path behavior matrix

The listing is shown by two renderers, `list` (CLI text) and the viewer (GUI), and AC-51 requires
the same values from both. The spec tags every GUI AC `RENDER`, so each row's handler is the file
that renders it.

| AC | Surface | Path |
|---|---|---|
| `AC-51` | RENDER | `gui/src/lib/Listing.svelte` |
| `AC-52` | RENDER | `gui/src/lib/viewer.svelte.ts` |
| `AC-53` | RENDER | `gui/src/lib/viewer.svelte.ts` |
| `AC-54` | RENDER | `gui/src/lib/ErrorState.svelte` |
| `AC-55` | RENDER | `gui/src/lib/viewer.svelte.ts` |
| `AC-56` | RENDER | `gui/src/lib/viewer.svelte.ts` |
| `AC-57` | RENDER | `gui/src/lib/EmptyState.svelte` |

AC-58 (`none`) is covered by `gui/src-tauri/tests/open.rs`.

**Test scope:** `crates/stidcart-core/tests/listing.rs`, `crates/stidcart-cli/tests/list_golden.rs`, `gui/src-tauri/tests/`, `gui/src/lib/`

How each AC is proven:
- **AC-51:**
  - `listing.rs` checks the values against the CLI's pre-move golden.
  - `gui/src-tauri/tests/open.rs` checks that, for **every** fixture, `open` equals the DTO of
    `describe`.
  - `Listing.test.ts` checks that every DTO field from the golden JSON appears in the rendered
    row.
- **AC-54:**
  - `open.rs` checks that the error text equals `list`'s reason for crafted EasyFS1, truncated,
    non-CRT and other-hardware inputs (built the way `cartridge.rs`/`read.rs` tests build them).
  - `viewer.test.ts` and `ErrorState.test.ts` check that the state clears the previous cart.
- **AC-52, AC-53, AC-55, AC-56, AC-57:** `viewer.test.ts` and component tests using a fake host.
- **AC-58:** `open.rs` hashes the file before and after opening it, for a clean open and for a
  refused one.
- **Live check (RENDER):** `npm run tauri dev`, then open a fixture, drop one, drop two and open a
  refused file. Each state is captured with `screencapture` and reviewed. claude-in-chrome does not
  apply here, because the UI runs in WKWebView, not Chrome.

## Reused primitives

| Existing primitive | Path | Used for |
|---|---|---|
| `read::inspect`, `read::load`, `CartInfo::free_space`, `CartBuilder::free_space` | `crates/stidcart-core/src/read.rs`, `lib.rs` | The rows and the D-027 free figure inside `listing::describe` |
| `kind_label`, `layout`, flag letters, `printable` | `crates/stidcart-cli/src/main.rs` | Moved verbatim into `listing` (not rewritten) |
| `ReadError: Display` | `crates/stidcart-core/src/read.rs` | The refusal text both renderers show (AC-54) |
| `DirEntry`, `EntryType`, `Half`, `is_cartridge`, `primary_num` | `crates/stidcart-core/src/directory.rs`, `image.rs` | Inputs to the row values |
| The fixture-dir helpers and crafted-image builders (`fixture()`, `crt(id, exrom, game, …)`, `cart_with_slots`) | `crates/stidcart-core/tests/cartridge.rs` | Pattern for building refusal inputs in `listing.rs` / `open.rs`. Copied as needed, since test helpers aren't shared across crates |
| Frozen fixtures | `fixtures/*.crt` | AC-51's "every fixture" sweep |
| `rust-toolchain.toml`, `Swatinem/rust-cache`, `actions/checkout@v7` | repo root, `.github/workflows/ci.yml` | The `gui` job mirrors the `gate` job's toolchain handling |

## Data model / Contracts

```rust
// stidcart-core::listing (no serde)
pub struct Listing {
    pub cart: String,        // ef_name, as `cart:` prints it
    pub built_by: String,    // header name with control chars → '.'
    pub rows: Vec<Row>,
    pub free: usize,         // D-027 figure
}
pub struct Row {
    pub index: usize,
    pub name: String,
    pub flags: String,       // "H","A","S" in that order, "-" when none
    pub kind: &'static str,  // "PRG", "XB16", …
    pub bank: String,        // "$0a"
    pub layout: String,      // "$8000" or "L2@0,H2@0"
    pub size: String,        // "$004000"
}
pub fn describe(bytes: &[u8]) -> Result<Listing, ReadError>;
```

```rust
// stidcart-gui (serde, GUI crate only), camelCase on the wire
#[derive(Serialize)] struct ListingDto { cart, builtBy, rows: Vec<RowDto>, free: usize, freeText: String }
// freeText = "$00c000 (49152)", the same text as list's closing line; one formatter in core:
// `Listing::free_text()`
#[tauri::command] fn open_cartridge(path: String) -> Result<ListingDto, String>
```

```ts
// gui/src/lib/types.ts mirrors ListingDto; pinned by gui/src/lib/fixtures/{two,refused}.json
type ViewState = { kind: 'empty' } | { kind: 'listing'; file: string; listing: Listing }
               | { kind: 'error'; file: string; message: string };
interface Host { pickFile(): Promise<string | null>; onDrop(cb: (paths: string[]) => void): void;
                 openCartridge(path: string): Promise<Listing>; }
```

## File structure

### New files

```
crates/stidcart-core/src/listing.rs        Listing/Row/describe + free_text; moved CLI helpers
crates/stidcart-core/tests/listing.rs      AC-51 core side: describe values per fixture, refusals
crates/stidcart-cli/tests/list_golden.rs   pre-move golden: `list` stdout, stderr and exit code for every fixture, refused ones (`in_*.crt`) included
crates/stidcart-cli/tests/golden/*.txt     the committed `list` outputs (one per fixture)
gui/package.json                           private; scripts dev/build/check/test/tauri; GPL-2.0-or-later
gui/package-lock.json                      npm lockfile
gui/vite.config.ts                         Svelte plugin, Tauri dev-server settings, vitest (jsdom) config
gui/svelte.config.js                       vitePreprocess
gui/tsconfig.json                          strict TS, resolveJsonModule
gui/index.html                             app shell
gui/src/main.ts                            mounts App with the real host
gui/src/App.svelte                         header (drag region, file name, Open button) + state switch
gui/src/styles/tokens.css                  design tokens, light/dark, base typography
gui/src/lib/types.ts                       Listing, Row, ViewState, Host
gui/src/lib/host.ts                        the only @tauri-apps/* importer: pickFile, onDrop, openCartridge
gui/src/lib/viewer.svelte.ts               view state machine (open, drop, multi-drop notice, errors)
gui/src/lib/Listing.svelte                 cart header + entries table + count/free footer
gui/src/lib/EmptyState.svelte              invitation to open or drop
gui/src/lib/ErrorState.svelte              refusal message for a file
gui/src/lib/fixtures/two.json              golden DTO (contract) for a clean listing
gui/src/lib/fixtures/refused.json          golden DTO error for a refusal
gui/src/lib/viewer.test.ts                 AC-52/53/55/56 (+54 state) with a fake host
gui/src/lib/Listing.test.ts                AC-51 render of every golden field
gui/src/lib/EmptyState.test.ts             AC-57
gui/src/lib/ErrorState.test.ts             AC-54 render
gui/src/test-setup.ts                      @testing-library/jest-dom matchers
gui/src-tauri/Cargo.toml                   stidcart-gui 0.7.0, own [workspace], path dep on core, tauri 2.12, plugin-dialog, serde
gui/src-tauri/Cargo.lock                   GUI workspace lockfile
gui/src-tauri/build.rs                     tauri_build::build()
gui/src-tauri/tauri.conf.json              window (overlay title bar, size), frontendDist ../dist, devUrl, identifier
gui/src-tauri/capabilities/default.json    dialog:allow-open + core window/event defaults only
gui/src-tauri/icons/                       default icon set (required by tauri-build)
gui/src-tauri/src/lib.rs                   open(), ListingDto, open_cartridge command, run()
gui/src-tauri/src/main.rs                  calls stidcart_gui::run()
gui/src-tauri/tests/open.rs                AC-51 every-fixture equality, AC-54 error text, AC-58 no writes
gui/src-tauri/tests/contract.rs            golden JSON equality (UPDATE_GOLDEN=1 rewrites)
gui/.gitignore                             node_modules, dist, src-tauri/target
docs/lcd/ci/run-test.sh                    scoped-test dispatcher over the three runners (cargo root, cargo gui, vitest)
docs/lcd/ci/gui-gate.sh                    the GUI gate: same steps locally and in the CI `gui` job
```

### Modified files

- `Cargo.toml`: `exclude = ["gui/src-tauri"]`; version `0.6.0 → 0.7.0`
- `Cargo.lock`: version bump
- `crates/stidcart-core/src/lib.rs`: `pub mod listing;`
- `crates/stidcart-cli/src/main.rs`: `list` calls `listing::describe`; `render` takes `&Listing`;
  helpers removed (moved)
- `.github/workflows/ci.yml`: add the `gui` job (apt + node + rust-cache, then `docs/lcd/ci/gui-gate.sh`)
- `.gitignore`: ignore `gui/node_modules`, `gui/dist`, `gui/src-tauri/target` if not covered by
  `gui/.gitignore`
- `docs/lcd/MAP.md`: `gui` zone row, entry point, Surfaces (RENDER now has two renderers), core
  zone mentions `listing`
- `.claude/rules/lcd-conventions.md`: test-placement `gui`, stems, GUI job commands
- `CLAUDE.md`: the GUI job beside the gate (how to run it locally)
- `README.md`: a short "GUI" section on running `npm run tauri dev` in `gui/`
- `docs/lcd/SPEC.md`: at reconcile only
- `docs/lcd/work/gui-shell/*`: pipeline artifacts

## Risks & rejected alternatives

- **The apt list on ubuntu-latest is unmeasured for our exact crate**, and ubuntu-latest moves to
  26.04 between 2026-10-19 and 2026-11-19. Mitigation: the first CI run sets the list, and webkit2gtk
  4.1 is published for 26.04.
- **Svelte 5 + Vite 8 + vite-plugin-svelte + Vitest 5 version alignment is unverified.**
  Mitigation: the scaffold task pins exact versions that install together, checked with
  `npm view <pkg> peerDependencies` before installing.
- **`generate_context!` needs `frontendDist`.** A cold `cargo test` in `gui/src-tauri` without
  `npm run build` fails at compile time. Documented in CLAUDE.md and the GUI job's order.
- **Overlay title bar on Linux.** `titleBarStyle` is macOS-only and ignored elsewhere. Linux is a
  portability check, so this is acceptable.
- **`describe` returns display strings from the core.** That is a deliberate shift (D-032). Should
  a later renderer need raw numbers, `Row` can gain them beside the strings without breaking either
  renderer.
- **Using `mockIPC` for the frontend tests**: rejected for this slice, because the host adapter
  makes plain fakes enough.
