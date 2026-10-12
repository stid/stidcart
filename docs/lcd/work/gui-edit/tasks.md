# Tasks: Desktop cartridge editor (GUI, second slice)

> **Deep lane · Phase 3 of 6 (Tasks).** A flat, dependency-ordered checklist. Each task names
> the file it touches. Tests come before the code they test (TDD). The granular STEPS in the
> work-item `JOURNAL.md` mirror this list as the resume view; keep them in sync.

**Slug:** `gui-edit`
**Spec:** [`spec.md`](./spec.md) • **Plan:** [`plan.md`](./plan.md) • **Journal:** [`JOURNAL.md`](./JOURNAL.md)
**Created:** 2026-10-11
**Status:** in_progress <!-- in_progress | completed | abandoned -->

---

## Conventions

- `[P]` suffix = parallelizable with the previous task (different file, no shared state).
- Tests precede implementation tasks they cover.
- One file per task where possible; tasks naming multiple files should be split unless the change is genuinely atomic.
- Every AC test carries the literal `AC-N (RENDER)` (or `AC-80 (none)`) in its name or doc comment.
  A RENDER AC is proven in two places: its Rust test checks the document's behaviour and its vitest
  test checks the window. Both carry the token.
- Root gate before any commit touching the root workspace; `docs/lcd/ci/gui-gate.sh` before any
  commit touching `gui/`. Check `cargo --version` through the rustup shim first (D-010).

## Tasks

### A. Groundwork: version, spike, CLI library, classification in core

- [ ] **T1**: Bump the version 0.7.0 → 0.8.0 in root `Cargo.toml`, `Cargo.lock`,
  `gui/src-tauri/Cargo.toml` and `gui/src-tauri/Cargo.lock`. This is atomic, one version across
  both workspaces. First check the base with `git show $(git merge-base HEAD main):Cargo.toml`.
  The root gate (parity included) and `gui-gate.sh` must be green. Commit `chore: bump version to 0.8.0`.
- [ ] **T2**: Spike the menu and keys in `tauri dev`, on a scratch branch that is discarded. Add a
  custom Undo item with ⌘Z and a Save item with ⌘S to a throwaway menu, then check three things:
  - ⌘Z inside a focused `<input>` still undoes text, or `document.execCommand('undo')` from the
    menu handler does;
  - ⌘O and ⌘S reach the menu, not the webview `keydown`;
  - what `message()` with three custom buttons returns on Esc.

  Record the results and the menu design chosen (the plan's design or its `keydown` fallback) in
  the `gui-edit` JOURNAL LOG. No code is committed.
- [ ] **T3**: Split the CLI into a library and a binary. `crates/stidcart-cli/Cargo.toml` gets a
  `[lib]` target, every function and test moves from `crates/stidcart-cli/src/main.rs` into
  `crates/stidcart-cli/src/lib.rs`, and `main.rs` calls `stidcart_cli::run` and maps the result to
  `ExitCode`. This is atomic: a pure move. All the CLI tests and goldens must stay green, along
  with the root gate.
- [ ] **T4**: Write failing tests in `crates/stidcart-core/tests/input.rs` for D-034.
  `Entry::from_input`:
  - classifies a `.prg` as a program;
  - classifies each fixture cartridge image as its kind, with alignment set when asked on an xbank;
  - refuses a malformed image with the `CartridgeError` text;
  - refuses alignment on a program and on a non-xbank with exactly the CLI's two messages.
- [ ] **T5**: Implement `Entry::from_input` and `InputError` (with Display) in
  `crates/stidcart-core/src/lib.rs` so T4 passes.
- [ ] **T6**: Refactor `input_entry` in `crates/stidcart-cli/src/lib.rs` to read the file, take the
  stem and call `Entry::from_input`, prefixing a cartridge refusal with `'<path>': `. The existing
  `edit.rs`, `carts.rs` and `cli.rs` error-text assertions must stay green (no-downgrade), with the
  root gate green.

### B. The GUI document (Rust)

- [ ] **T7**: Add the dev-dependency `stidcart-cli = { path = "../../crates/stidcart-cli" }` to
  `gui/src-tauri/Cargo.toml` and refresh `gui/src-tauri/Cargo.lock`.
- [ ] **T8**: Write failing tests in `gui/src-tauri/tests/document.rs`:
  - `AC-59 (RENDER)`: an editable fixture opens clean, its listing equals `describe(file)`, and
    the file's bytes are unchanged (AC-58 kept).
  - `AC-60 (RENDER)`: a firmware-mismatch image (the `edit.rs` AC-25 recipe) opens view-only with `load`'s refusal text, and every edit returns that text.
  - `AC-77 (RENDER)`: New is untitled and clean, with no entries and the default names, and equals
    `CartBuilder::new().build()`.
  - Listing rows and builder entries line up by index for every editable fixture, and `alignable`
    is true exactly on xbank rows.
- [ ] **T9**: Implement `gui/src-tauri/src/document.rs` so T8 passes: `Document`, `Snapshot`,
  `open`, `new`, `dto`, `DocumentDto` and `RowActions`. Add the one-line `pub mod document;` in
  `gui/src-tauri/src/lib.rs`.
- [ ] **T10**: Write failing tests in `gui/src-tauri/tests/document.rs`:
  - `AC-61 (RENDER)`: adding `hello.prg` and a cart image appends entries named by the file stem,
    of the right kind.
  - `AC-62 (RENDER)`: an aligned add of an xbank gets `A`; an aligned add of a non-xbank is
    refused with `edit --add-aligned`'s text.
  - `AC-63 (RENDER)`: a multi-add is all or none (one unreadable path → nothing added) and is one
    history step.
  - `AC-64 (RENDER)`: remove.
  - `AC-65 (RENDER)`: rename; an over-long name is shown truncated to the stored 16.
  - `AC-66 (RENDER)`: replace keeps the name, the hidden state and the position, and the kind
    follows the file.
  - `AC-67 (RENDER)`: hide and show.
  - `AC-68 (RENDER)`: align and unalign an xbank; alignment on a non-xbank is refused.
  - `AC-69 (RENDER)`: the cart name and the header name change independently.
  - `AC-70 (RENDER)`: out of space, a full directory, a second Ocean, an unreadable file and a bad
    index each return `edit`'s text, and the listing, dirty state and history are unchanged.
  - `AC-71 (RENDER)`: after an edit the listing equals `describe(build())`, and an untouched entry
    shows a re-placed position.
  - The reload property: `load(build(b)).build() == build(b)` holds for every state reached in
    these tests.
- [ ] **T11**: Implement `Edit` and `Document::apply` in `gui/src-tauri/src/document.rs` so T10
  passes. Indexes are bounds-checked with the CLI's `no entry at index …` text.
- [ ] **T12**: Write failing tests in `gui/src-tauri/tests/document.rs`:
  - `AC-72 (RENDER)`: dirty after an edit, clean after undoing back to the saved point, dirty
    again after redo; when the saved point is in a discarded redo tail, the document stays dirty
    until it is saved.
  - `AC-73 (RENDER)`: undo and redo move through every step, a new edit drops the redo tail, and
    a multi-add undoes in one step.
- [ ] **T13**: Implement `undo`, `redo` and dirty tracking in `gui/src-tauri/src/document.rs` so
  T12 passes.
- [ ] **T14**: Write failing tests in `gui/src-tauri/tests/document.rs`:
  - `AC-74 (RENDER)`: save rewrites the file with the current snapshot's bytes and clears dirty;
    a clean save leaves the file's bytes and mtime untouched; saving an untitled document is an
    error.
  - `AC-75 (RENDER)`: Save As writes the new path, adopts it, clears dirty and leaves the source
    unchanged; Save As of an unedited cart is a verbatim copy.
  - `AC-76 (RENDER)`: a save into a read-only directory errors, leaves the old file
    byte-identical and no temp file behind, and keeps the document dirty.
- [ ] **T15**: Implement `save`, `save_as` and the mirrored `write_replacing` (temp file plus
  rename, std only, with a comment pointing at the CLI) in `gui/src-tauri/src/document.rs` so T14
  passes.
- [ ] **T16**: Write `gui/src-tauri/tests/parity.rs`, `AC-80 (none)`: a table of (GUI op, CLI
  args) scenarios over the fixtures. Each GUI step is compared with one chained
  `stidcart_cli::run(["edit", prev, <op>, "-o", next])`, a multi-add with several `--add`, and New
  followed by adds with `build` of the same inputs, all byte-identical. It is expected green
  against T11 and T15. Any red is a `document.rs` fix.
- [ ] **T17**: Write a failing contract test. `gui/src-tauri/tests/contract.rs` also pins
  `gui/src/lib/fixtures/document.json` (a `DocumentDto` for an edited fixture), and
  `UPDATE_GOLDEN=1` writes it. Generate and commit the golden together with T18.
- [ ] **T18**: Implement the commands in `gui/src-tauri/src/lib.rs`:
  - the managed `Mutex<Option<Document>>`;
  - `open_document`, `new_document`, `apply_edit`, `undo`, `redo`, `save_document`,
    `save_document_as` and `allow_exit`, each returning a `DocumentDto` or the error text;
  - the `ExitRequested` guard with its `allow_exit` flag.

  `open_cartridge` is removed and `open()` is kept. T17, `open.rs` and the document tests must be
  green.
- [ ] **T19**: Write `gui/src-tauri/src/menu.rs`, following T2's outcome:
  - File: New ⌘N, Open… ⌘O, Save ⌘S, Save As… ⌘⇧S and the predefined Close Window;
  - Edit: Undo ⌘Z and Redo ⌘⇧Z, plus predefined Cut/Copy/Paste/Select All;
  - `set_enabled` driven from a `DocumentDto`.

  Wire it in `gui/src-tauri/src/lib.rs`: `set_menu`, and `on_menu_event` → `emit("menu", id)`.
  After every command, the enabled state is refreshed.
- [ ] **T20** [P]: Add `dialog:allow-save`, `dialog:allow-message` and the window-destroy
  permission the close path uses to `gui/src-tauri/capabilities/default.json`.

### C. The frontend

- [ ] **T21**: Add bits-ui 2.19.5 (exact, peer svelte ^5.33 checked) to `gui/package.json` and
  `gui/package-lock.json`.
- [ ] **T22**: Extend `gui/src/lib/types.ts` with `Document`, `RowActions`, `Edit`, `Unsaved`,
  `MenuId`, the extended `Host` (per the plan's contract) and `ViewState` (`document` replacing
  `listing`). Add a type-level test that `fixtures/document.json` satisfies `Document`, as the
  existing JSON fixtures are checked.
- [ ] **T23**: Write failing tests in `gui/src/lib/viewer.test.ts` with a fake host:
  - `AC-59 (RENDER)`: opening yields a `document` state, editable and clean.
  - `AC-61 (RENDER)`: the add command sends `add` with the picked paths.
  - `AC-63 (RENDER)`: a multi-file add is sent as one edit.
  - `AC-70 (RENDER)`: a rejected edit shows the reason as a notice and keeps the previous document.
  - `AC-73 (RENDER)`: menu undo and redo call the host when no editable element has focus, and act
    on the field when one does.
  - `AC-74 (RENDER)`: ⌘S saves when dirty and is a no-op when clean.
  - `AC-75 (RENDER)`: Save As picks a path then saves; a dismissed chooser changes nothing.
  - `AC-76 (RENDER)`: a failed save shows the reason and stays dirty.
  - `AC-77 (RENDER)`: New calls `newDocument`; saving while untitled goes through Save As.
  - `AC-78 (RENDER)`: the guard covers each of Save, Don't Save and Cancel, plus a dismissed save
    path and a failed save, for New, Open, close and quit; no prompt when clean; exactly one prompt
    per action. The label mapping: only `Save` and `Don't Save` proceed, and anything else is
    `cancel`.
  - `AC-79 (RENDER)`: a drop onto an editable document adds; onto empty, error or view-only, one
    file opens and several get the notice.

  Narrow the existing AC-56 multi-drop test to the non-editable case. Keep AC-52..55.
- [ ] **T24**: Implement the editor store in `gui/src/lib/viewer.svelte.ts` so T23 passes:
  - `edit`, `undo`, `redo`, `save`, `saveAs`, `newCart` and `guard`;
  - drop routing and the notice;
  - the exported `unsavedChoice(label)`.
- [ ] **T25**: Write failing tests in `gui/src/lib/EditableText.test.ts`, `AC-65 (RENDER)`: Enter
  commits the new text through the callback, Esc restores the old text and commits nothing, and
  blur commits.
- [ ] **T26**: Implement `gui/src/lib/EditableText.svelte` so T25 passes.
- [ ] **T27**: Write failing tests in `gui/src/lib/EntryMenu.test.ts`:
  - `AC-64 (RENDER)`: Remove emits `remove`.
  - `AC-66 (RENDER)`: Replace… picks a file and emits `replace`.
  - `AC-67 (RENDER)`: Hide or Show, according to the row's flags.
  - `AC-68 (RENDER)`: Align/Unalign is present only when `alignable`.
- [ ] **T28**: Implement `gui/src/lib/EntryMenu.svelte` (a bits-ui ContextMenu plus a "⋯"
  DropdownMenu) so T27 passes.
- [ ] **T29**: Write failing tests in `gui/src/lib/Toolbar.test.ts`:
  - `AC-62 (RENDER)`: Add Aligned… sends `add` with `aligned: true`.
  - `AC-72 (RENDER)`: the Edited indicator shows only when dirty, and Untitled shows for a new
    cart.
- [ ] **T30**: Implement `gui/src/lib/Toolbar.svelte` so T29 passes.
- [ ] **T31**: Write failing tests in `gui/src/lib/Listing.test.ts`:
  - `AC-60 (RENDER)`: a read-only document shows the banner with the reason and offers no edit
    affordances.
  - `AC-69 (RENDER)`: the cart and header names are editable and emit the matching edit.
  - `AC-71 (RENDER)`: the rows render the DTO's bank and layout as given.
  - The existing AC-51 tests are kept.
- [ ] **T32**: Update `gui/src/lib/Listing.svelte` so T31 passes: the banner, an `EditableText`
  for the names and entry rename, and `EntryMenu` on each row.
- [ ] **T33**: Implement the new host methods in `gui/src/lib/host.ts`:
  - `pickInputs`, `pickSavePath` and `confirmUnsaved` (`message()` labels mapped via
    `unsavedChoice`);
  - `onMenu`, `onCloseRequested` and `onExitRequested`;
  - the command invokes.

  This is exercised by the live check, as the existing `host.ts` is.
- [ ] **T34**: Wire `gui/src/App.svelte`: the `Toolbar` in the title bar, and the menu, close and
  exit subscriptions to the store. Remove the ⌘O `keydown` (the menu replaces it).
  `gui-gate.sh` must be green.
- [ ] **T35** [P]: Add tokens for the menu, the inline field, the banner and the Edited dot to
  `gui/src/styles/tokens.css`, if the components need them.
- [ ] **T36**: Live check (RENDER) with `npm run tauri dev`:
  - open a fixture; rename, hide, remove, align;
  - Add… and Add Aligned…; undo and redo, including ⌘Z inside the rename field;
  - ⌘S, then ⌘⇧S;
  - close with changes and take each of the three answers; ⌘Q with changes;
  - New, then save while untitled; open a view-only cart; a refused edit (out of space).

  Capture each state with `screencapture -R` (the `gui-live-check` memory). Ask the user to do the
  drops: two PRGs onto an open cart, and a `.crt` onto an open cart. Log the results in the
  JOURNAL.

### D. Docs and closeout

- [ ] **T37**: Update the zones in `docs/lcd/MAP.md`: the cli is library plus binary; the gui
  edits (document, menu) and dev-depends on the cli; the core holds `Entry::from_input`.
- [ ] **T38** [P]: Add one line to `README.md` saying the desktop app edits carts.
- [ ] **T-audit**: Run `/lcd:audit gui-edit`. The resulting `audit.md` must have zero
  MISSING/BLOCKED rows.
- [ ] **T-final**: Run the full gate, all green:
  `cargo fmt --all --check && cargo clippy --workspace --all-targets --all-features -- -D warnings && cargo test --workspace && docs/lcd/ci/gui-gate.sh`.
  Then the closeout: `lcd:reconcile` into `docs/lcd/SPEC.md`, then the PR.

## Checkpoint validation

After every block of related tasks, the suite must stay green. If T3 passes but T5's implementation breaks T1, stop and fix before moving on.

Checkpoints:
- After A (T6), the root gate is green.
- After B (T20), `gui-gate.sh` is green.
- After C (T34), `gui-gate.sh` is green.
- After T-final, both are green.

## Linkbacks

- `AC-59 (RENDER)` → T8, T9, T23, T24
- `AC-60 (RENDER)` → T8, T9, T31, T32
- `AC-61 (RENDER)` → T4, T5, T10, T11, T23, T24
- `AC-62 (RENDER)` → T10, T11, T29, T30
- `AC-63 (RENDER)` → T10, T11, T23, T24
- `AC-64 (RENDER)` → T10, T11, T27, T28
- `AC-65 (RENDER)` → T10, T11, T25, T26, T32
- `AC-66 (RENDER)` → T10, T11, T27, T28
- `AC-67 (RENDER)` → T10, T11, T27, T28
- `AC-68 (RENDER)` → T10, T11, T27, T28
- `AC-69 (RENDER)` → T10, T11, T31, T32
- `AC-70 (RENDER)` → T10, T11, T23, T24
- `AC-71 (RENDER)` → T10, T11, T31, T32
- `AC-72 (RENDER)` → T12, T13, T29, T30
- `AC-73 (RENDER)` → T2, T12, T13, T19, T23, T24
- `AC-74 (RENDER)` → T14, T15, T23, T24
- `AC-75 (RENDER)` → T14, T15, T23, T24
- `AC-76 (RENDER)` → T14, T15, T23, T24
- `AC-77 (RENDER)` → T8, T9, T23, T24
- `AC-78 (RENDER)` → T2, T18, T19, T20, T23, T24, T33, T34, T36
- `AC-79 (RENDER)` → T23, T24, T34
- `AC-80 (none)` → T3, T6, T7, T16
- Version, MAP, docs → T1, T37, T38; DTO contract → T17, T18, T22; wiring → T21, T33, T34, T35, T36

---

<!--
Quality checks before starting implementation:
  - Every AC × non-`none`-surface row in plan.md has at least one test task above
  - Every new file in plan.md "File structure" has at least one task that creates it
  - Tests appear before their implementation
  - T-audit appears before T-final (the audit gate must pass before suite-final)
  - T-final exists (run the suite, lint, type check)
-->
