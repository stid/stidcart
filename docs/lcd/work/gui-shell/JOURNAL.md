# JOURNAL — gui-shell

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the *entire*
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Deep
- **Goal:** First GUI slice: a read-only desktop viewer that opens a `.crt` (chooser or drop) and shows what `list` shows, with refusals as readable errors; macOS first, portable toolkit.
- **Next action:** run /lcd:tasks gui-shell
- **Branch:** feat/gui-shell  ·  **Updated:** 2026-10-11 13:40

## STEPS
- [x] S1 — triage (Deep), recon on toolkits
- [x] S2 — spec.md (AC-51..AC-58)
- [x] S3 — toolkit decision → D-030 (Tauri 2.x, separate workspace + required GUI CI job)
- [x] S4 — plan.md (listing in core D-032; GUI workspace + required `gui` CI job; host adapter + golden JSON contract)
- [ ] S5 — /lcd:tasks gui-shell  ← next

## DECISIONS (this work-item)
- Platforms: macOS first, toolkit must stay portable since CI builds on Ubuntu (user, 2026-10-11)
- First slice is a read-only viewer, no editing (user, 2026-10-11)
- GUI ACs use the existing `RENDER` surface token (MAP: a GUI widens RENDER, it doesn't open a new surface)

## OPEN QUESTIONS
- none (DTO contract → golden JSON; frontend built before cargo in the GUI job)

## EDIT BOUNDARY (paths this work may touch)
- `docs/lcd/work/gui-shell/`, `docs/lcd/DECISIONS.md`, `docs/lcd/MAP.md`, `docs/lcd/SPEC.md` (reconcile)
- `Cargo.toml`, `Cargo.lock`, `.gitignore`, `CLAUDE.md`, `README.md`, `.claude/rules/lcd-conventions.md`
- `crates/stidcart-core/src/listing.rs`, `crates/stidcart-core/src/lib.rs`, `crates/stidcart-core/tests/listing.rs`
- `crates/stidcart-cli/src/main.rs`, `crates/stidcart-cli/tests/list_golden.rs`, `crates/stidcart-cli/tests/golden/`
- `gui/` (frontend, `src-tauri/`, tests; see plan File structure)
- `.github/workflows/ci.yml`
<!-- /lcd-resume -->

---

## Acceptance criteria

See `spec.md`: AC-51..AC-58.

## LOG (append-only; not read on resume)

- 2026-10-11 12:11 — Recon on toolkits (crates.io API + web, Context7 unavailable). All MSRVs ≤ pinned 1.97.1.
  - eframe/egui 0.36.2 (MIT/Apache, MSRV 1.95): headless tests via egui_kittest Harness (accesskit tree, no GPU); `dropped_files`; direct core calls. Ubuntu CI: libxcb-render0/shape0/xfixes0-dev, libxkbcommon-dev, libssl-dev. Keep kittest `wgpu`/`snapshot` features off, since `--all-features` would turn on GPU snapshots on ubuntu-latest.
  - Tauri 2.12.3 (MIT/Apache, MSRV 1.90): builds .app/.dmg itself, native drop; 3.0 alphas since 2026-09; CI needs webkit2gtk-4.1 + 5 packages; mock runtime covers only the Rust side, JS tests sit outside `cargo test`; IPC means serde in core or DTOs.
  - Slint 1.18.1 (GPL-3.0-only OR royalty-free-2.0 OR commercial): the royalty-free terms add attribution and API restrictions, so in practice it's GPLv3 and the binary ships GPLv3. Dropping files from Finder is unconfirmed.
  - iced 0.14.0 (MIT, MSRV 1.88): iced_test Simulator, FileDropped; no release in about 10 months.
  - Dioxus desktop / gpui / Xilem / Floem are not contenders (webview cost, no standalone release, immature).
  - rfd 0.17.2 (MIT) for native dialogs; its default features need no GTK. Packaging without Tauri: cargo-packager.
  - Licence hygiene: choose the MIT side of dual licences, consider cargo-deny.
  - Recon recommendation: egui + egui_kittest (Harness only) + rfd.
- 2026-10-11 12:55 — /stid-decide council (3 analysts, one revision round). User relaxed tests-in-cargo-test to tests-automated-in-CI; ruled small-footprint = gate only, GUI job required, Linux = portability only. Two analysts converged on Tauri 2.12 + separate workspace; the skeptic held egui because Tauri's style control is established and egui's isn't, and weighting polish first decides it → D-030. Skeptic's stress-test (after the user's rulings) upheld Tauri: council unanimous. Corrections folded into D-030: re_ui's polish layer is ~1.4k lines, not 20k; no written 2.x support policy. Measured: Tauri apt step 27–47 s on ubuntu-latest (tauri runs 38139467117/38139467273), nothing preinstalled; `frontendDist` must exist at compile time unless devUrl in debug.

- 2026-10-11 13:40 — plan.md written. User chose the listing in core (D-032). Verified in scratch: an `exclude`d nested workspace is not reached by `cargo test --workspace`. Frontend tests use a fake host adapter instead of mockIPC (refines D-030). No bits-ui/TanStack in this slice.

## DEEP PIPELINE (Deep lane only)

> Phase tracker for the Deep-lane pipeline. Files live beside this JOURNAL.

`spec.md ✅  plan.md ✅  tasks.md ⬜  tests ⬜  red-green ⬜  audit.md ⬜`
<!-- mark ✅ done · ⏳ in progress · ⬜ not started -->
