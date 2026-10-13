# Plan: Desktop cartridge editor (GUI, second slice)

> **Deep lane · Phase 2 of 6 (Plan).** This document owns **how**. Specify ≠ Plan: if you're
> about to write a user story or acceptance criterion, that belongs in `spec.md`.

**Slug:** `gui-edit`
**Spec:** [`spec.md`](./spec.md) · **Journal:** [`JOURNAL.md`](./JOURNAL.md)
**Created:** 2026-10-11
**Status:** approved <!-- draft | approved | implemented | superseded -->

---

## Goal

The Tauri viewer becomes an editor. It offers the full `edit` set plus New, keeps changes in
memory with undo/redo and an unsaved-changes indicator and prompt, saves on request through
⌘S/⌘⇧S, and every save is byte-identical to `edit`/`build` for the same request.

## Architecture

**The document is a stack of saved images, held in Rust (D-035).** A new `Document` type in the
GUI crate holds:
- `path: Option<PathBuf>`
- `history: Vec<Snapshot { bytes: Vec<u8>, listing: Listing }>`
- the cursor `at`
- `saved: Option<usize>`
- `read_only: Option<String>`

Each snapshot's `bytes` is exactly what saving at that point writes, and its `listing` is
`listing::describe(&bytes)`, so the window always shows what a save would produce (AC-71).

`Document::open` handles three outcomes:
- It reads the file and describes it. A refusal becomes the error state, as today.
- It tries `read::load`. On success the cart is editable, and snapshot 0 is the file's own bytes,
  so an opened cart shows positions as stored (AC-59, AC-51).
- When `load` refuses, the document is view-only and carries the refusal text `edit` prints
  (AC-60).

`Document::new()` takes snapshot 0 from `CartBuilder::new().build()`, so it starts with the
default names `build` uses (AC-77).

Every edit goes through one path:
1. `read::load(current.bytes)`.
2. Apply the operation to the builder.
3. `build()`, then `describe`.
4. On success: truncate the redo tail, push the snapshot, advance `at`. If `saved` pointed into the
   discarded tail, it becomes `None`.

Any `Err` returns the text `edit` would print and touches nothing, so a refused edit leaves the
listing, the dirty state and the history as they were (AC-70).

Undo and redo move `at`, and `dirty = saved != Some(at)` (AC-72, AC-73).

Save writes `history[at].bytes` with a temp file and a rename, then sets `saved = Some(at)`. Save
does nothing when the document is clean, so a clean cart is never rewritten (AC-74). Save As does
the same at a new path and adopts it (AC-75). Saving a cart that was opened and never edited, with
Save As, therefore writes a verbatim copy, never a re-placement.

Reloading every step is only sound if `load(build(b)).build() == build(b)`. A property test checks
that for every editable fixture and every state the tests reach. Memory is one image per undo step,
at most about 1 MB and usually 30–700 KB. No cap is set, because AC-73 asks for every edit since
opening.

**The Tauri layer stays thin.**
- `lib.rs` holds `Mutex<Option<Document>>` as managed state. There is one window, and several
  windows are out of scope.
- The commands are `open_document`, `new_document`, `apply_edit(Edit)`, `undo`, `redo`,
  `save_document`, `save_document_as(path)` and `allow_exit`. Each returns a `DocumentDto` or the
  error text.
- Every index is bounds-checked in the command layer, with the CLI's `no entry at index …` text,
  before any core mutator runs. Core mutators panic on a bad index, and a panic while the mutex is
  held would poison it for every later command.
- `Edit` is a serde-tagged enum: `Add { paths, aligned }`, `Remove`, `Rename`, `Replace`,
  `SetHidden`, `SetAligned`, `SetCartName` and `SetHeaderName`. A multi-file `Add` reads and
  classifies every file first, then pushes them all and builds once: all or none, one undo step
  (AC-63).

**Input classification moves into the core (D-034).** `input_entry` in the CLI holds format
knowledge the MAP keeps out of front ends: the CRT signature check, `Cartridge::from_crt`, and the
alignment checks for an added file. It becomes
`Entry::from_input(name, data, aligned) -> Result<Entry, InputError>` in core. That function does
no I/O and holds no path.
- `InputError`'s Display reproduces the CLI's alignment messages word for word.
- Both front ends wrap the cartridge variant as `'<path>': <reason>`, as the CLI does today.
- The existing CLI tests (`edit.rs`, `carts.rs`, `cli.rs`) pin that text, which is the
  no-downgrade check.
- The default entry name, the file stem, stays a one-line `Path::file_stem` in each front end. It
  is host policy, not format.

**Parity is proven against the real CLI (D-035).** `stidcart-cli` becomes a library plus a
two-line binary. `run(args)` and its helpers move to `src/lib.rs`, and `main.rs` calls
`stidcart_cli::run` and maps the result to the exit code. The binary's behaviour is unchanged, and
the CLI's tests, which run the binary, pin it.

The GUI crate takes `stidcart-cli` as a **dev-dependency** only. `tests/parity.rs` (AC-80) drives
a table of `(GUI op, CLI args)` scenarios:
- Each GUI step is checked against one chained `edit` call: `run(["edit", prev, <op>, "-o", next])`.
- A multi-file add maps to one call with several `--add`.
- New followed by adds is checked against `build` of the same inputs in the same order.

Chaining one call per step keeps the oracle obviously right. It sidesteps the CLI's within-call
reordering (AC-27), and it re-reads the file each time, exactly as the document reloads per step.

AC-80 covers sequences of one or more edits. A Save As of an unedited cart is a verbatim copy and
needs no CLI oracle.

**The frontend.** The `viewer.svelte.ts` store grows from `empty | listing | error` into an editor.
The `listing` state now carries the whole `DocumentDto`: display name, listing, `readOnly` reason,
`dirty`, `canUndo`, `canRedo` and per-row actions. The store gains `edit`, `undo`, `redo`, `save`,
`saveAs`, `newCart`, and a `guard(action)` that runs the unsaved-changes flow below.

`host.ts` stays the only `@tauri-apps/*` importer. It gains:
- `pickInputs` (several files, `.prg` and `.crt`), `pickSavePath` and `confirmUnsaved`
- `onMenu`, `onCloseRequested` and `onExitRequested`
- the command wrappers

Which actions a row offers comes from the backend: the DTO's `rows[i].alignable` is true only for
an xbank entry of an editable cart. The frontend never infers it from kind labels. AC-68 is met by
not offering alignment on other rows.

UI pieces, built with bits-ui 2.19.5 (D-031 named it, but it was never installed):
- `EntryMenu.svelte`: a ContextMenu on each row plus a "⋯" DropdownMenu, with Rename, Replace…,
  Hide/Show, Align/Unalign (xbank rows only) and Remove.
- `EditableText.svelte`: inline editing for an entry name, the cart name and the header name.
  Enter commits, Esc cancels, blur commits.
- `Toolbar.svelte`: the title-bar contents. It shows the file name or "Untitled" with an "Edited"
  dot when dirty (AC-72), plus Add…, Add Aligned… and Open….

The title bar is the app's own (overlay, hidden title), so the indicator lives there and no
native document-edited call is needed.

**The menu and the unsaved-changes flow.** `menu.rs` builds a native menu with custom items.
- File: New ⌘N, Open… ⌘O, Save ⌘S, Save As… ⌘⇧S, and a predefined Close Window.
- Edit: Undo ⌘Z and Redo ⌘⇧Z, plus predefined Cut/Copy/Paste/Select All, so text fields keep
  native clipboard behaviour.
- `on_menu_event` emits `menu:<id>` to the main window.
- After every command, the Rust side sets Save/Undo/Redo enabled from the document.
- The ⌘O `keydown` handler in `App.svelte` is replaced by the menu accelerator, with the same
  behaviour.

When undo or redo arrives while an editable element has focus, the store leaves the cart alone and
calls `document.execCommand('undo'|'redo')` on the field (AC-73).

**Spike first (task 1):** verify in `tauri dev` that:
- a custom ⌘Z menu item doesn't break undo inside a focused `<input>`, and the fallback above works;
- the menu accelerator, not the webview, gets ⌘O and ⌘S.

If the custom Undo/Redo items take ⌘Z away from text fields in a way the fallback can't fix, the
fallback is to drop them from the menu and handle ⌘Z/⌘⇧Z in the webview's `keydown`, skipping
editable targets. The result is logged in the JOURNAL before the menu work starts.

The guard flow, one place in the store, runs on New, Open, a drop onto a non-editable state, close
and quit (AC-78):

```
action requested ──► dirty? ──no──► do it
                       │yes
                       ▼
             confirmUnsaved() → "Save" | "Don't Save" | anything else (Esc, dismiss) = Cancel
               Save ──► untitled? pickSavePath (null → stop) ──► save (Err → show reason, stop) ──► do it
               Don't Save ──► do it
               Cancel ──► stop (state unchanged)
close:  onCloseRequested → preventDefault → guard(() => allow_exit then destroy window)
quit:   ⌘Q is a custom menu item → emit "menu":"quit" → guard(() => exit_app). (Refined in the
        live check: the predefined Quit sends terminate:, which tao 0.37 does not route to
        ExitRequested, so it skipped the guard — D-036.)
        RunEvent::ExitRequested(code=None) while dirty (closing the last window by an uncleared
        path) → prevent_exit, emit "exit-requested" → guard(() => exit_app). exit_app/allow_exit
        set a flag, so app.exit's own ExitRequested (code=Some) passes without asking again
```

Only the two exact labels go ahead. Anything else counts as Cancel, and a unit test pins that
mapping. The prompt is `message(text, { kind: 'warning', buttons: { yes: 'Save', no: "Don't Save",
cancel: 'Cancel' } })`, from plugin-dialog 2.8.2 (recon: custom buttons return the label).

**Drop routing (AC-79).**
- An editable document is open: the store calls `edit({ kind: 'add', paths, aligned: false })`.
- Otherwise: one path opens, and several are refused with the existing notice.

**Rejected alternatives**, one line each. Durable ones go into D-035.
- Document state in the frontend: the format work and snapshots belong next to the core, and the
  frontend would need the image bytes.
- Snapshots of `CartBuilder` instead of bytes: the same memory, but the listing would no longer
  describe what a save writes, and Save As of an unedited cart would re-place it.
- A per-window `HashMap<label, Document>`: one window is in scope, so `Option` suffices.
- `tempfile` for the atomic save: `NamedTempFile` creates mode 0600, and `persist` would turn every
  saved cart owner-only. The std temp-and-rename the CLI uses keeps umask permissions.
- The objc2 `setDocumentEdited`: the title is hidden, so the in-app indicator is what the user
  sees.
- Committed goldens or core-only parity: ruled out by the user's choice (D-035).

## Constitution check

| Rule | Applies? | Compliance note |
|---|---|---|
| `testing.md` (tests first, coverage) | yes | Every AC test is written failing first. Rust: `gui/src-tauri/tests/document.rs` and `parity.rs`, plus the core `from_input` tests. Frontend: vitest with a fake host, `vitest run` only. The full root gate and `gui-gate.sh` run after the multi-file CLI split and before the PR |
| `no-downgrade.md` | yes | Kept as is: every viewer behaviour (AC-51..58; AC-56 is revised by the spec's AC-79, with the user's approval); `list`, `edit` and `build` output and error text, pinned by the existing CLI tests and goldens across the lib split and the D-034 move; ⌘O, now via the menu |
| `no-overengineering.md` | yes | One window, `Option<Document>`. No undo cap or compaction. No new Rust dependency besides bits-ui on the frontend. The file stem stays one line per front end, not a core API. Nothing beyond `edit`'s set |
| `refinement-protocol.md` | yes | The forks (save model, drop semantics, New, the parity oracle) were settled with the user before code. Mid-flight drift goes through `lcd:refine` |
| `versioning.md` | yes | `feat/` → minor. Base version checked with `git show $(git merge-base HEAD main):Cargo.toml` (0.7.0). The bump to 0.8.0 is **the first code commit** (the spec/plan commits precede it), in both the root workspace and `gui/src-tauri/Cargo.toml`, with the root gate (parity included) and `gui-gate.sh` green |
| `commits.md` | yes | Conventional, atomic, one commit per red→green flip. Never on `main`. Push when the user asks (the global rule says always push; the user's standing practice here is a push per checkpoint) |
| `ac-convention.md`: EVAL coverage | no | Byte-exact output against the CLI oracle (MAP: EVAL `n/a`). Nothing scored or silently plausible |
| `CLAUDE.md`: the gate, plus `gui-gate.sh` for GUI changes | yes | Both green before every commit that touches their workspace. `cargo --version` checked through the rustup shim (D-010) |
| `CLAUDE.md`: decisions logged when made | yes | D-033 (product), D-034 (classification in core) and D-035 (document model + CLI-as-library parity), written with this plan |
| `.claude/rules/lcd-conventions.md` | yes | AC-59..80 claimed, next free AC-81 (done). Test tokens `AC-N (RENDER)` / `AC-80 (none)` carried in test comments. The scoped-test dispatcher already covers all three runners |
| MAP invariant: byte parity against the frozen fixtures | yes | The write path is unchanged. AC-80 adds the CLI as an oracle for the GUI. The version bump changes default header bytes, and the root parity test runs green in the bump commit |
| MAP invariant: no global state or I/O policy in the core | yes | `Entry::from_input` takes a name and bytes and does no I/O. File reading, path naming, writing and undo stay in the front ends. The core gains no serde |
| MAP invariant: format facts from `docs/format/` | yes | No new format fact. Classification moves code without changing it |
| MAP invariant: describing versus rebuilding (D-011) | yes | Opening describes as stored. Editing is the rebuild path, and AC-71 states that positions move. View-only carts (load refuses) stay describable (AC-60) |
| MAP invariant: crate version in produced bytes | yes | The New + `build` parity compares both sides on the same core version, so it's version-agnostic. The bump commit re-runs root parity |
| MAP invariant: zlib/GPL boundary at `firmware/` | yes | Untouched. bits-ui is MIT |
| MAP zones (gui: no format knowledge; cli: no format knowledge) | yes | `alignable` comes from the backend (`Cartridge::kind.is_xbank()` on the loaded builder), and classification moves to core. The MAP is updated: cli is lib + bin, gui edits, and gui dev-depends on cli |

## Cross-path behavior matrix

The spec tags every GUI AC `RENDER` (one surface, two renderers of `listing`). The matrix binds
each to the frontend file that implements it, which is what the audit's file-exists check reads.

| AC | Surface | Path |
|---|---|---|
| `AC-59` | RENDER | `gui/src/lib/viewer.svelte.ts` |
| `AC-60` | RENDER | `gui/src/lib/Listing.svelte` |
| `AC-61` | RENDER | `gui/src/lib/viewer.svelte.ts` |
| `AC-62` | RENDER | `gui/src/lib/Toolbar.svelte` |
| `AC-63` | RENDER | `gui/src/lib/viewer.svelte.ts` |
| `AC-64` | RENDER | `gui/src/lib/EntryMenu.svelte` |
| `AC-65` | RENDER | `gui/src/lib/EditableText.svelte` |
| `AC-66` | RENDER | `gui/src/lib/EntryMenu.svelte` |
| `AC-67` | RENDER | `gui/src/lib/EntryMenu.svelte` |
| `AC-68` | RENDER | `gui/src/lib/EntryMenu.svelte` |
| `AC-69` | RENDER | `gui/src/lib/Listing.svelte` |
| `AC-70` | RENDER | `gui/src/lib/viewer.svelte.ts` |
| `AC-71` | RENDER | `gui/src/lib/Listing.svelte` |
| `AC-72` | RENDER | `gui/src/lib/Toolbar.svelte` |
| `AC-73` | RENDER | `gui/src/lib/viewer.svelte.ts` |
| `AC-74` | RENDER | `gui/src/lib/viewer.svelte.ts` |
| `AC-75` | RENDER | `gui/src/lib/viewer.svelte.ts` |
| `AC-76` | RENDER | `gui/src/lib/viewer.svelte.ts` |
| `AC-77` | RENDER | `gui/src/lib/viewer.svelte.ts` |
| `AC-78` | RENDER | `gui/src/lib/viewer.svelte.ts` |
| `AC-79` | RENDER | `gui/src/lib/viewer.svelte.ts` |

AC-80 (`none`) is covered by `gui/src-tauri/tests/parity.rs`.

**Test scope:** `gui/src-tauri/tests/`, `gui/src/lib/`, `crates/stidcart-core/tests/input.rs`

How each AC is proven:
- **Rust, `document.rs`.** Document behaviour, with a test per AC carrying its token:
  - AC-59: clean after open, file hash unchanged.
  - AC-60: a firmware-mismatch image built the way `edit.rs` AC-25 builds it (an
    empty image with the EAPI region zeroed) is view-only with `load`'s text, and edits return it.
  - AC-61..69: each `Edit` variant's effect on the listing.
  - AC-62/68: alignment on a non-xbank is refused and `alignable` is false.
  - AC-63: all-or-none, one step.
  - AC-70: history and dirty state unchanged after an out-of-space, full-directory, second-Ocean,
    unreadable or bad-index edit.
  - AC-71: the listing equals `describe(build())`, and an untouched entry moves.
  - AC-72/73: the dirty and cursor arithmetic, and a redo tail dropped by a new edit.
  - AC-74: save rewrites, and save when clean leaves the mtime alone.
  - AC-75: Save As adopts the path and leaves the source unchanged; an unedited Save As is a
    verbatim copy.
  - AC-76: a read-only directory leaves the file unchanged and the document dirty.
  - AC-77: New equals `CartBuilder::new()`.
  - The reload property `load(build(b)).build() == build(b)` over every editable fixture and every
    reached state.
  - Listing rows and builder entries line up by index across all fixtures.
- **Rust, `parity.rs`.** AC-80 scenarios against `stidcart_cli::run`.
- **Core, `input.rs`.** `Entry::from_input`: a program, each cartridge kind, a refused image, and
  alignment on a program or a non-xbank, with the CLI's text.
- **Frontend, vitest with a fake host:**
  - `viewer.test.ts`: AC-59, 61, 63, 70, 73 (including the editable-focus routing), 74–79, the
    guard's state machine, and the prompt-label mapping.
  - `EntryMenu.test.ts`: AC-64, 66, 67, 68 (actions offered per `alignable`).
  - `EditableText.test.ts`: AC-65 (Enter commits, Esc cancels).
  - `Listing.test.ts`: AC-60 banner, AC-69 names, AC-71 renders the DTO it's given.
  - `Toolbar.test.ts`: AC-62 Add Aligned, AC-72 indicator.
- **Live check (RENDER).** `npm run tauri dev`, then:
  - The spike (⌘Z in a field, ⌘O/⌘S routing).
  - Open a fixture; rename, hide, remove; drop two PRGs; undo/redo; ⌘S; ⌘⇧S.
  - Close with changes and pick each of the three answers; ⌘Q with changes.
  - Open a view-only cart.

  Captured with `screencapture -R`, per the `gui-live-check` memory. Drops need the user.

## Reused primitives

| Existing primitive | Path | Used for |
|---|---|---|
| `read::load`, `read::inspect` (via `describe`), `listing::describe`, `Listing::free_text` | `crates/stidcart-core/src/read.rs`, `listing.rs` | Snapshot listing, editability, the view-only reason |
| `CartBuilder` mutators (`push`, `remove`, `set_name`, `set_hidden`, `set_align_64k`, `replace`, `set_ef_name`, `set_header_name`, `build`) | `crates/stidcart-core/src/lib.rs` | Every `Edit` variant, unchanged |
| `BuildError`, `ReadError`, `CartridgeError` Display | core | The refusal text the window shows, the same as `edit` |
| `input_entry` → `Entry::from_input` | `crates/stidcart-cli/src/main.rs` → core | Moved, not rewritten (D-034) |
| `write_replacing` (temp file + rename) | `crates/stidcart-cli/src/main.rs` | Mirrored in the GUI crate (I/O can't live in core); the comment points back |
| The CLI's `run` (`edit`, `build`) | `crates/stidcart-cli` → `src/lib.rs` | The AC-80 oracle |
| `ListingDto`/`RowDto`, `open()` | `gui/src-tauri/src/lib.rs` | Embedded in `DocumentDto`. `open()` stays for AC-51/54/58 |
| The `Host` adapter, the `viewer.svelte.ts` store, the fake-host test pattern, `Listing.svelte`, `ErrorState`, `EmptyState`, `tokens.css` | `gui/src/` | Extended, not replaced |
| The firmware-mismatch recipe (`build_image` + `write_at(EAPI_OFFSET, zeros)` + `crt::write`) | `crates/stidcart-cli/tests/edit.rs` (AC-25) | Repeated in `document.rs` for AC-60 |
| `run-test.sh`, `gui-gate.sh`, the CI `gui` job | `docs/lcd/ci/`, `.github/workflows/ci.yml` | Unchanged. The new tests are picked up by the existing globs |
| Frozen fixtures, `hello.prg`, `big.prg`, `in_*.crt` | `fixtures/` | Inputs for every Rust test and parity scenario |

## Data model / Contracts

```rust
// stidcart-core (no serde, no I/O)
impl Entry { pub fn from_input(name: &str, data: Vec<u8>, aligned: bool) -> Result<Entry, InputError>; }
pub enum InputError { Cartridge(CartridgeError), AlignProgram { name: String }, AlignNotXbank { name: String } }
// Display: Cartridge → the CartridgeError text (front ends prefix "'<path>': ");
// the Align* variants → the CLI's current messages, verbatim

// stidcart-cli
pub fn run(args: &[String]) -> Result<Option<String>, String>;   // lib.rs; main.rs maps it to ExitCode

// stidcart-gui
pub struct Document { /* path, history: Vec<Snapshot>, at, saved, read_only */ }
impl Document {
    pub fn open(path: &Path) -> Result<Document, String>;   // Err = list's refusal text (error state)
    pub fn new() -> Document;
    pub fn apply(&mut self, edit: Edit) -> Result<(), String>;
    pub fn undo(&mut self) -> bool; pub fn redo(&mut self) -> bool;
    pub fn save(&mut self) -> Result<(), String>;           // no-op when clean; Err when untitled
    pub fn save_as(&mut self, path: &Path) -> Result<(), String>;
    pub fn dto(&self) -> DocumentDto;
}
#[derive(Deserialize)] #[serde(tag = "kind", rename_all = "camelCase")]
pub enum Edit { Add { paths: Vec<String>, aligned: bool }, Remove { index: usize },
                Rename { index: usize, name: String }, Replace { index: usize, path: String },
                SetHidden { index: usize, hidden: bool }, SetAligned { index: usize, aligned: bool },
                SetCartName { name: String }, SetHeaderName { name: String } }
#[derive(Serialize)] #[serde(rename_all = "camelCase")]
pub struct DocumentDto { file: String /* name or "Untitled" */, untitled: bool, listing: ListingDto,
                         rows: Vec<RowActions { alignable: bool }>, read_only: Option<String>,
                         dirty: bool, can_undo: bool, can_redo: bool }
```

```ts
// gui/src/lib/types.ts
type ViewState = { kind: 'empty' } | { kind: 'document'; doc: Document } | { kind: 'error'; file: string; message: string };
type Edit = { kind: 'add'; paths: string[]; aligned: boolean } | { kind: 'remove'; index: number } | …;
type Unsaved = 'save' | 'discard' | 'cancel';   // host maps message() labels; anything unknown → 'cancel'
interface Host {   // existing: pickFile, onDrop
  pickInputs(): Promise<string[]>; pickSavePath(suggested: string): Promise<string | null>;
  confirmUnsaved(file: string): Promise<Unsaved>;
  onMenu(cb: (id: MenuId) => void): void; onCloseRequested(cb: () => Promise<boolean>): void;
  onExitRequested(cb: () => void): void;
  openDocument(path: string): Promise<Document>; newDocument(): Promise<Document>;
  edit(e: Edit): Promise<Document>; undo(): Promise<Document>; redo(): Promise<Document>;
  save(): Promise<Document>; saveAs(path: string): Promise<Document>; exit(): Promise<void>;
}
```

`openCartridge` and the `open_cartridge` command are replaced by `openDocument`, since nothing
else calls them. `ListingDto`/`RowDto` and their golden JSON stay unchanged. A new golden,
`gui/src/lib/fixtures/document.json`, pins `DocumentDto` the same way (`contract.rs`).

## File structure

### New files

```
crates/stidcart-cli/src/lib.rs                 run() and every helper moved from main.rs (input_entry now calls Entry::from_input)
crates/stidcart-core/tests/input.rs            Entry::from_input: kinds, refusals, alignment text (D-034)
gui/src-tauri/src/document.rs                  Document, Snapshot, Edit, DocumentDto, apply/undo/redo/save/save_as, write_replacing mirror
gui/src-tauri/src/menu.rs                      native menu (File/Edit), ids, set_enabled from DocumentDto
gui/src-tauri/tests/document.rs                AC-59..77 Rust side, reload property, row/entry alignment, AC-58 on open
gui/src-tauri/tests/parity.rs                  AC-80: (GUI op, CLI args) scenarios against stidcart_cli::run
gui/src/lib/fixtures/document.json             golden DocumentDto (contract)
gui/src/lib/EntryMenu.svelte                   per-row ContextMenu + "⋯" DropdownMenu (bits-ui)
gui/src/lib/EntryMenu.test.ts                  AC-64/66/67/68
gui/src/lib/EditableText.svelte                inline text edit: Enter commits, Esc cancels, blur commits
gui/src/lib/EditableText.test.ts               AC-65
gui/src/lib/Toolbar.svelte                     title bar: file name/Untitled + Edited dot, Add…, Add Aligned…, Open…
gui/src/lib/Toolbar.test.ts                    AC-62/72
```

### Modified files

- `Cargo.toml` (root): version 0.7.0 → 0.8.0 (first code commit).
- `Cargo.lock` (root): version bump.
- `crates/stidcart-cli/Cargo.toml`: adds a `[lib]` target beside `[[bin]] stidcart`.
- `crates/stidcart-cli/src/main.rs`: reduced to calling `stidcart_cli::run` and the exit-code
  mapping.
- `crates/stidcart-core/src/lib.rs`: `Entry::from_input` and `InputError` (D-034).
- `gui/src-tauri/Cargo.toml`: version 0.8.0; dev-dependency `stidcart-cli` (path).
- `gui/src-tauri/Cargo.lock`: lockfile.
- `gui/src-tauri/src/lib.rs`: managed `Mutex<Option<Document>>`, the new commands replacing
  `open_cartridge`, the menu wiring, `ExitRequested` handling. `open()` and the DTOs are kept.
- `gui/src-tauri/capabilities/default.json`: adds `dialog:allow-save`, `dialog:allow-message`,
  `core:window:allow-destroy` (whatever the close path uses; confirmed in the live check).
- `gui/src-tauri/tests/contract.rs`: also pins `document.json`.
- `gui/src-tauri/tests/open.rs`: kept. It calls `open()`, unchanged; touched only if a helper moves.
- `gui/package.json`, `gui/package-lock.json`: bits-ui 2.19.5.
- `gui/src/lib/types.ts`: `Document`, `Edit`, `Unsaved`, `MenuId`, and the extended `Host` and
  `ViewState`.
- `gui/src/lib/host.ts`: the new host methods (dialog save/message, menu, close and exit events,
  invokes).
- `gui/src/lib/viewer.svelte.ts`: the editor store (edit/undo/redo/save/saveAs/new/guard, drop
  routing, notice).
- `gui/src/lib/viewer.test.ts`: the AC-59/61/63/70/73–79 tests. The existing AC-52..56 tests are
  kept, and AC-56's multi-drop test is scoped to the non-editable case.
- `gui/src/lib/Listing.svelte`: the read-only banner, editable cart and header names, row menus
  and inline rename.
- `gui/src/lib/Listing.test.ts`: AC-60/69/71.
- `gui/src/App.svelte`: the Toolbar in the title bar, menu and close/exit subscriptions; the ⌘O
  `keydown` removed (the menu now does it).
- `gui/src/styles/tokens.css`: tokens for the menu, the inline field and the banner, if needed.
- `docs/lcd/MAP.md`: the cli zone is lib + bin, the gui zone edits, gui dev-depends on cli;
  Surfaces in use unchanged.
- `docs/lcd/DECISIONS.md`: D-034, D-035.
- `docs/lcd/work/gui-edit/`: JOURNAL, tasks, audit.
- `README.md`: one line saying the GUI edits.

## Risks & rejected alternatives

- **⌘Z inside text fields**, unverified per recon. Spiked in task 1, with the `keydown` fallback
  recorded above.
- **What `message()` returns on Esc**, unverified. The label mapping treats anything that isn't
  "Save" or "Don't Save" as Cancel, so the worst case is a safe cancel.
- **The ⌘Q path through `RunEvent::ExitRequested`**, from recon memory and not doc-checked. The
  live check covers ⌘Q, and the `allow_exit` flag prevents a double prompt.
- **A missing capability** fails silently at runtime. The live check exercises Save As, the
  prompt and close.
- **The reload property.** If `load(build(b))` ever drops a field (alignment, subdir bit, a name),
  undo would be silently lossy. The property test runs over every fixture and every reached state.
- **Memory.** Each undo step holds one image. A long session on a large cart could hold hundreds of
  MB, which is accepted for now (out of scope: compaction).
- **`tempfile`**: rejected because it saves with mode 0600 (see Architecture).
- **The GUI depending on the CLI crate at runtime** for the file helpers: rejected. The CLI is the
  test harness (MAP), so the dependency is dev-only. Format logic goes to core, and the eight-line
  write helper is mirrored.
