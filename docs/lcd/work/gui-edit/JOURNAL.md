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
- **Next action:** done; PR #34 green (gate, gui, audit, claude-review) and awaits the user's merge
- **Branch:** feat/gui-edit  ·  **Updated:** 2026-10-11 20:10

## STEPS
- [x] S1 — triage (Deep), recon, spec.md
- [x] S2 — plan.md (D-034, D-035)
- [x] S3 — tasks.md
- [x] S4 — /lcd:test-gen gui-edit (failing tests: input.rs, document.rs, parity.rs, contract.rs, viewer/EditableText/EntryMenu/Toolbar/Listing tests)
- [x] S5 — T1 version 0.8.0 (both workspaces, 19b5b12)
- [x] S6 — T2 menu/⌘Z/⌘O/Esc spike → LOG (plan's design holds)
- [x] S7 — T3 CLI lib split
- [x] S8 — T4–T6 Entry::from_input in core (D-034), CLI uses it
- [x] S9 — T7 gui dev-dep on cli
- [x] S10 — T8/T9 Document open/new/dto (AC-59/60/77)
- [x] S11 — T10/T11 Document::apply (AC-61..71, reload property)
- [x] S12 — T12/T13 undo/redo/dirty (AC-72/73)
- [x] S13 — T14/T15 save/save_as (AC-74..76)
- [x] S14 — T16 parity vs CLI (AC-80)
- [x] S15 — T17/T18 DocumentDto golden + commands, exit guard
- [x] S16 — T19/T20 menu.rs + capabilities
- [x] S17 — T21/T22 bits-ui, types.ts
- [x] S18 — T23/T24 editor store (AC-59/61/63/70/73..79)
- [x] S19 — T25–T32 EditableText, EntryMenu, Toolbar, Listing
- [x] S20 — T33–T35 host.ts, App.svelte, tokens
- [x] S21 — T36 live check (one bug found and fixed: ⌘Q)
- [x] S22 — T37/T38 MAP, README
- [x] S23 — T-audit, T-final, reconcile, PR #34

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
- `gui/src/lib/`
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
- 2026-10-11 20:05 — test-gen: spec AC-60 amended. `inspect` refuses type 0x1A, so `list` can't describe it; firmware mismatch is the one describable-but-not-editable case (plan/tasks follow). Boundary bullet for gui/src/lib made plain (hook gotcha). Red confirmed: core `input` (no from_input), GUI cargo (no document module / stidcart_cli dep), vitest 24 failing + 3 missing components. Design pinned by tests: DocumentDto fields pub (file, untitled, listing, rows[].alignable, read_only, dirty, can_undo, can_redo), `Document::bytes()`; SetAligned on a non-xbank refused with the CLI `--align` text (core ignores it on programs); store API `add(aligned)`, `menu(id)`, `requestClose()`, `requestQuit()`, `editable`, `dirty`, `unsavedChoice`; components EditableText{value,label,oncommit}, EntryMenu{row,actions,onedit,onrename,onreplace}, Toolbar{doc,onadd,onopen}, Listing optional {readOnly,rows,onedit,onreplace}.
- 2026-10-11 20:40 — T2 spike (tauri dev, throwaway, reverted): custom Undo item (⌘Z) receives the key; with an <input> focused, `document.execCommand('undo')` from the menu handler undoes the field one step (two edits → one ⌘Z restored the first, no double undo). ⌘O/⌘S: the menu accelerator fires AND the webview keydown still sees meta+key → handle each shortcut only via the menu (remove App's ⌘O keydown). Predefined Select All works in the field. `message()` with 3 custom buttons returns "Cancel" on Esc. Plan design kept; keydown fallback not needed. Live-driving gotcha: another app ("Web App") steals focus — raise with AXRaise and check the frontmost process in the same osascript before any keystroke.
- 2026-10-11 21:30 — block B green: document.rs 20/20, parity 2/2 first run (no fix needed), contract 3/3 with document.json. Refinement: Undo/Redo menu items stay always enabled (a disabled Undo would swallow ⌘Z in a text field); only Save/Save As follow the document. Added `exit_app` command beside `allow_exit` (quit path), `core:window:allow-destroy` for the JS close path.
- 2026-10-11 22:20 — red-green: all green (root gate; gui-gate 38 vitest + 28 GUI cargo). ~14 implement/fix iterations, 0 reverts, no STUCK. Deviations: bits-ui pinned 2.19.3 (2.19.5 only 8 days old); component tests add `afterEach(cleanup)` (vitest without globals); EditableText loads its value only when editing starts (rename from the row menu); T35 tokens not needed (existing tokens cover menu/banner/dot).
- 2026-10-11 22:15 — T36 live check (tauri dev, scratch copies, driven by osascript + a CGEvent click tool; captures in the session scratchpad). Passed: open clean; inline rename with ⌘Z undoing field text only; "⋯" menu offers Align only on the xbank, align re-placed the untouched PRG (AC-71); right-click menu without Align on a PRG; Hide + menu ⌘Z; ⌘S → file byte-identical to `edit --rename 0 greetings --align 1`, Edited cleared; ⌘⇧S → copy with -rw-r--r--, window adopts it, source untouched; Add Aligned with a .prg → edit's exact refusal, no change; New + Add + ⌘S → save panel, file byte-identical to `build hello.prg`; firmware-mismatch cart → view-only banner with edit's text, no edit controls; ⌘W prompt (Save/Don't Save/Cancel), Esc cancels; ⌘O → Don't Save → reopens, file untouched; ⌘W → Don't Save → app exits, file untouched. User: multi-file drop of hello+ramp added both, one ⌘Z removed both; dropped in_8k.crt added as 8K, not opened.
- 2026-10-11 22:15 — BUG found live, fixed c4dff08: predefined Quit sends `terminate:`, which tao 0.37 doesn't intercept (no applicationShouldTerminate) → ⌘Q exited with no ExitRequested and lost an unsaved edit. Quit is now a custom ⌘Q item routed through the store's guard (test added). Known gap: Dock → Quit still sends terminate: and bypasses the prompt (needs an app-delegate hook; out of this slice unless the user wants it). Cosmetic: a focused row shows a blue focus-ring line.
- 2026-10-11 22:30 — audit: PASS (first run), 21/21 RENDER rows OK; AC-80 (none) in parity.rs. Closeout line appended.
- 2026-10-11 22:30 — reconciled: folded AC-59..80 → SPEC.md/RENDER (6 new rows; drop row rewritten for AC-79; opening row reworded) and SPEC.md/none (AC-80). No supersede entry: the drop change is D-033.
- 2026-10-12 00:10 — advisor pre-PR review: added every-editable-fixture parity + reload test (passes on all, incl. Ocean/512/wide/subdir); fixed stale ExitRequested comment and plan guard block (refined: ⌘Q is the custom Quit item). User chose to narrow AC-78 to in-app quit and list macOS-initiated termination (Dock/logout) as out of scope + follow-up → spec amended, SPEC row aligned, D-036.
- 2026-10-12 00:40 — PR #34: gate 26s, gui 2m28s (first Linux run of the new tests, AC-76 included), audit, claude-review (no issues): all pass.

## DEEP PIPELINE (Deep lane only)

> Phase tracker for the Deep-lane pipeline. Files live beside this JOURNAL.

`spec.md ✅  plan.md ✅  tasks.md ✅  tests ✅  red-green ✅  audit.md ✅`
<!-- mark ✅ done · ⏳ in progress · ⬜ not started -->
