# JOURNAL — hex-viewer

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the *entire*
> cold-start payload. Keep NOW and STEPS current as you work. Everything below the `---` is
> history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Standard
- **Goal:** a read-only hex viewer for the open cart in the desktop app, in its own window (issue #66)
- **Next action:** done; merged as PR #70
- **Branch:** feat/hex-viewer  ·  **Updated:** 2026-11-13

## STEPS
- [x] S1 — core `hexmap`: rows of 16 bytes in bank order with bank, half, C64 address and a text column; ownership regions (loader, directory, EAPI, name, boot, entries, metadata entry). Tests first
- [x] S2 — GUI Rust `hex.rs`: DTOs and pure `info` / `lines` over a `Document`, golden JSON contract, tests in `gui/src-tauri/tests/hex.rs`
- [x] S3 — GUI Rust wiring: commands, the `hex` window, View ▸ Hex Viewer, change and goto events, `capabilities/hex.json`, main window closing takes the hex window with it
- [x] S4 — frontend `HexView.svelte` (virtualised rows, tints, owner readout, jump select) with tests; `main.ts` mounts it in the `hex` window
- [x] S5 — main window: selection reaches the hex window, entry menu gets Show in Hex Viewer
- [x] S6 — live check, SPEC + MAP + guide, D-067, audit, PR
- [x] S7 — after the user tried it: a Whole cart / Selected entry switch; the entry view shows one entry's bytes from offset `$0000` (AC-205, AC-206)

## DECISIONS (this work-item)
- User's answers (2026-11-13, question dialog): whole image with a jump from the selected entry; rows addressed by bank + C64 address; a separate window; text column, entry ownership and reserved bank-0 marking all in the first version.
- Rows run bank by bank, ROML then ROMH of each (Claude's choice): an interleaved program is then one contiguous run. The image's own order is all ROML halves, then all ROMH.
- ROMH reads `$A000` throughout, as the issue's example does, although Ultimax carts and the boot code see it at `$E000` (Claude's choice; one fixed address per half keeps a row's label independent of who owns it).
- Addresses, the text column and ownership come from the core; the frontend only draws them (MAP: no format knowledge in `gui/`).
- The hex window shows the current step, which is what saving would write, and follows edits, undo and open.
- User, 2026-11-13, after trying the app ("so this is the entire cart catalog not the individual carts"): add an entry view beside the whole image, rows addressed by offset in the entry (question dialog). A cartridge entry reads in bank order, ROML then ROMH of each bank (Claude's choice). The window still opens on the whole cart (Claude's choice).
- No caching: each row request re-parses the container (a 1 MiB copy).

## OPEN QUESTIONS
- none

## EDIT BOUNDARY (paths this work may touch)
- `crates/stidcart-core/src/hexmap.rs`
- `crates/stidcart-core/src/lib.rs`
- `crates/stidcart-core/src/petscii.rs`
- `crates/stidcart-core/tests/hexmap.rs`
- `gui/src-tauri/src/`
- `gui/src-tauri/tests/`
- `gui/src-tauri/capabilities/`
- `gui/src/`
- `docs/guide/app.md`
- `docs/lcd/`
- `.claude/rules/lcd-conventions.md`
<!-- /lcd-resume -->

---

## Acceptance criteria

**AC-200** (surfaces: RENDER): View ▸ Hex Viewer opens a second window showing the open cart's flash image, 16 bytes a row, bank by bank with ROML before ROMH, each row labelled with its bank, half and C64 address (`$8000` up for ROML, `$A000` up for ROMH). The bytes are the ones saving would write. Nothing in the window changes a byte.

**AC-201** (surfaces: RENDER): each row carries a text column, one character per byte through the PETSCII mapping the names use, with `.` for anything unprintable.

**AC-202** (surfaces: RENDER): bytes are marked by owner: the menu loader, the directory, EAPI, the cart name, the boot code, each entry (in its kind family's colour) and the metadata entry; unowned bytes are plain. Pointing at a byte names its owner.

**AC-203** (surfaces: RENDER): selecting an entry in the listing moves an open hex window to that entry's first byte, and Show in Hex Viewer on an entry opens the window there.

**AC-204** (surfaces: RENDER): the hex window follows the cart: after an edit, undo, redo, open or New it shows the new bytes and owners. Only the rows in view are fetched and drawn.

**AC-205** (surfaces: RENDER): the hex window has a Whole cart / Selected entry switch, starting on the whole cart. Selected entry shows only the bytes of the entry selected in the main window, first to last, rows counted from `$0000`, with its name and size in the top bar and no owner tints, key or jump list; a program reads as the file it was added from and a cartridge in bank order. Switching back shows the whole cart at that entry.

**AC-206** (surfaces: RENDER): the entry view follows the selection and the cart: selecting another entry shows that one from its first byte, an edit or undo shows the new bytes, and with no entry selected it asks for one.

## LOG (append-only; not read on resume)

- 2026-11-13 — S7 built test-first after the user's feedback; audit: PASS (test-presence) for AC-205 and AC-206, both gates green; reconciled into the SPEC row. Not live-checked by Claude: the user is trying it in the dev app.

- 2026-11-13 — reconciled: folded AC-200..AC-204 → SPEC.md/RENDER
- 2026-11-13 — audit: PASS (test-presence). AC-200..AC-204 (RENDER) each have tests carrying the token; root gate and gui gate green.
- 2026-11-13 — live check in the dev app on a scratch copy of `cmixed.crt` (dark theme): View ▸ Hex Viewer opens the window at bank 00 ROML with the loader tinted; selecting `in_16k` moved it to bank 05 ROML; Show in Hex Viewer on `hello` brought it forward at `03 H $B9D0` with the start mark at `$B9DC`; pointing at a byte read `03 H $BB3F · unused`; Remove emptied hello's bytes in place and ⌘Z brought them back; ⌘W on the main window with the hex window open ended the app. Light theme and the no-cart message were not looked at live.
- 2026-11-13 — S1 to S5 built test-first; both gates green. A view-only cart has no entry menu, so there Show in Hex Viewer is not offered: select the entry and use View ▸ Hex Viewer.
- 2026-11-13 — triaged Standard (3 signals). The user then chose a separate window and all three extras, which is more than scored; kept Standard: one surface, reversible, no new dependency.
