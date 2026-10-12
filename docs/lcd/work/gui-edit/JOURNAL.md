# JOURNAL — gui-edit

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the *entire*
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Deep
- **Goal:** Second GUI slice: the viewer edits carts with the full `edit` set plus New, held in memory with undo, an unsaved-changes indicator and prompt, ⌘S/⌘⇧S, and saves byte-identical to `edit`/`build`.
- **Next action:** run /lcd:test-gen gui-edit
- **Branch:** feat/gui-edit  ·  **Updated:** 2026-10-11 19:25

## STEPS
- [x] S1 — triage (Deep), recon, spec.md
- [x] S2 — plan.md (D-034, D-035)
- [x] S3 — tasks.md
- [ ] S4 — /lcd:test-gen gui-edit  ← next
- [ ] S5 — T1 version 0.8.0 (both workspaces)
- [ ] S6 — T2 menu/⌘Z/⌘O/Esc spike → LOG
- [ ] S7 — T3 CLI lib split
- [ ] S8 — T4–T6 Entry::from_input in core (D-034), CLI uses it
- [ ] S9 — T7 gui dev-dep on cli
- [ ] S10 — T8/T9 Document open/new/dto (AC-59/60/77)
- [ ] S11 — T10/T11 Document::apply (AC-61..71, reload property)
- [ ] S12 — T12/T13 undo/redo/dirty (AC-72/73)
- [ ] S13 — T14/T15 save/save_as (AC-74..76)
- [ ] S14 — T16 parity vs CLI (AC-80)
- [ ] S15 — T17/T18 DocumentDto golden + commands, exit guard
- [ ] S16 — T19/T20 menu.rs + capabilities
- [ ] S17 — T21/T22 bits-ui, types.ts
- [ ] S18 — T23/T24 editor store (AC-59/61/63/70/73..79)
- [ ] S19 — T25–T32 EditableText, EntryMenu, Toolbar, Listing
- [ ] S20 — T33–T35 host.ts, App.svelte, tokens
- [ ] S21 — T36 live check
- [ ] S22 — T37/T38 MAP, README
- [ ] S23 — T-audit, T-final, reconcile, PR

## DECISIONS (this work-item)
- ACs AC-59..AC-80 claimed (next free AC-81); GUI ACs use `RENDER`, byte parity `none`
- Product choices → D-033; classification in core → D-034; document = stack of saved images in Rust, CLI as lib for parity → D-035
- First tasks: version bump 0.8.0 (both workspaces), then the ⌘Z/⌘O menu spike before any menu work

## OPEN QUESTIONS
- none blocking; undo storage is a plan question

## EDIT BOUNDARY (paths this work may touch)
- `Cargo.toml`, `Cargo.lock`, `README.md`
- `crates/stidcart-cli/Cargo.toml`, `crates/stidcart-cli/src/lib.rs`, `crates/stidcart-cli/src/main.rs`
- `crates/stidcart-core/src/lib.rs`, `crates/stidcart-core/tests/input.rs`
- `gui/src-tauri/Cargo.toml`, `gui/src-tauri/Cargo.lock`, `gui/src-tauri/capabilities/default.json`
- `gui/src-tauri/src/lib.rs`, `gui/src-tauri/src/document.rs`, `gui/src-tauri/src/menu.rs`
- `gui/src-tauri/tests/document.rs`, `gui/src-tauri/tests/parity.rs`, `gui/src-tauri/tests/contract.rs`, `gui/src-tauri/tests/open.rs`
- `gui/package.json`, `gui/package-lock.json`, `gui/src/App.svelte`, `gui/src/styles/tokens.css`
- `gui/src/lib/` (types, host, viewer store, Listing, EntryMenu, EditableText, Toolbar, their tests, fixtures/document.json)
- `docs/lcd/MAP.md`, `docs/lcd/DECISIONS.md`, `docs/lcd/SPEC.md`, `docs/lcd/triage-log.md`, `docs/lcd/work/gui-edit/`
- `.claude/rules/lcd-conventions.md`
<!-- /lcd-resume -->

---

## Acceptance criteria

See `spec.md` (AC-59..AC-80).

## LOG (append-only; not read on resume)

- 2026-10-11 18:10 — triage → Deep (5 signals, hard: new stateful editing module; risk: multi-session). Recon (Tauri state, menus, close interception, save dialog, undo, bits-ui) dispatched.
- 2026-10-11 18:20 — user: full edit set; explicit save; drops on an open cart add (incl. .crt); multi-drop adds atomically; New in scope → D-033.
- 2026-10-11 18:25 — found: core `build()` already refuses alignment on non-xbank (`AlignNotXbank`); names truncate silently to field width (16 entry, 32 header). `input_entry`/`entry_name` are CLI-private and must be shared for AC-61/AC-80 (plan).
- 2026-10-11 18:40 — recon (dated 2026-10-11): doc + snapshot undo in Rust behind one managed `Mutex<HashMap<window label, Doc>>`, removed on Destroyed. Rust `MenuBuilder` with custom Open/New/Save/Save As/Undo/Redo, `on_menu_event` → `emit_to` focused window; keep `PredefinedMenuItem` cut/copy/paste/select-all so text fields work. Close: `onCloseRequested` + preventDefault (allowed after awaits); ⌘Q path is `RunEvent::ExitRequested` + `prevent_exit` (verify). 3-button prompt: plugin-dialog `message(..., {buttons:{yes,no,cancel}})` returns the label (≥2.4, have 2.8.2); Esc result unverified. No document-edited API in 2.12 (objc2 `setDocumentEdited` unverified) → title marker fallback. Save: plugin-dialog `save()` + `tempfile` 3.27 `NamedTempFile::new_in(parent)`→sync_all→persist. bits-ui not installed; latest 2.19.5 (peer svelte ^5.33) for row ContextMenu/DropdownMenu.
- 2026-10-11 19:05 — plan written. User picked CLI-as-library for the AC-80 oracle. Advisor: chain one `edit` per GUI step; reload property test; spike ⌘Z/⌘O first; bounds-check indexes (mutex poisoning); reject tempfile (0600); map prompt labels strictly; allow_exit flag against a double prompt.

## DEEP PIPELINE (Deep lane only)

> Phase tracker for the Deep-lane pipeline. Files live beside this JOURNAL.

`spec.md ✅  plan.md ✅  tasks.md ✅  tests ⬜  red-green ⬜  audit.md ⬜`
<!-- mark ✅ done · ⏳ in progress · ⬜ not started -->
