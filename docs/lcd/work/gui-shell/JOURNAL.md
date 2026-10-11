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
- **Next action:** user picks the toolkit (recon in LOG) → log D-030 → run /lcd:plan gui-shell
- **Branch:** feat/gui-shell  ·  **Updated:** 2026-10-11 12:11

## STEPS
- [x] S1 — triage (Deep), recon on toolkits
- [x] S2 — spec.md (AC-51..AC-58)
- [ ] S3 — toolkit decision → D-030  ← next
- [ ] S4 — /lcd:plan gui-shell

## DECISIONS (this work-item)
- Platforms: macOS first, toolkit must stay portable since CI builds on Ubuntu (user, 2026-10-11)
- First slice is a read-only viewer, no editing (user, 2026-10-11)
- GUI ACs use the existing `RENDER` surface token (MAP: a GUI widens RENDER, it doesn't open a new surface)

## OPEN QUESTIONS
- Toolkit (blocks plan)

## EDIT BOUNDARY (paths this work may touch)
- `docs/lcd/work/gui-shell/`
- `docs/lcd/DECISIONS.md`
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

## DEEP PIPELINE (Deep lane only)

> Phase tracker for the Deep-lane pipeline. Files live beside this JOURNAL.

`spec.md ✅  plan.md ⬜  tasks.md ⬜  tests ⬜  red-green ⬜  audit.md ⬜`
<!-- mark ✅ done · ⏳ in progress · ⬜ not started -->
