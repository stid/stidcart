# JOURNAL — linux-arm64-release

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Standard
- **Goal:** A `v*` tag also builds Linux for 64-bit ARM: the command as a static `.tar.gz`, the app as a `.deb` and an AppImage, all in `SHA256SUMS` (D-080).
- **Next action:** read PR #117's dry run, then the VM check (S5)
- **Branch:** feat/release-linux-arm64  ·  **Updated:** 2026-11-22

## STEPS
- [x] S1 — recon: runner cost and labels, Tauri bundling on ARM, musl for the command
- [x] S2 — `release.yml`: a `linux-aarch64` leg in `command`, `app-linux` as a matrix of two; D-080
- [x] S3 — README install table, RELEASING.md, SPEC.md say the ARM build exists
- [ ] S4 — PR #117; the dry run's `release` artifact holds the three files and `SHA256SUMS` lists them  ← next
- [ ] S5 — in the Linux VM: install the dry run's `.deb`, go through the 2026-11-14 test script, run the AppImage and the command archive
- [ ] S6 — closeout

## DECISIONS (this work-item)
- Named `linux-aarch64`, static musl command, the app as a second leg of `app-linux` on `ubuntu-22.04-arm` → D-080.
- No test is written first: no test reads `release.yml` (scope.sh), so the dry run and the VM run are the check.

## OPEN QUESTIONS
- none blocking. For the owner: both `ubuntu-22.04` images are removed on 2027-04-17, which moves the x86_64 app leg too.

## EDIT BOUNDARY (paths this work may touch)
- `.github/workflows/release.yml`
- `README.md`
- `RELEASING.md`
- `docs/lcd/SPEC.md`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/triage-log.md`
- `docs/lcd/work/linux-arm64-release/JOURNAL.md`
<!-- /lcd-resume -->

---

## LOG (append-only; not read on resume)

- 2026-11-22 — triage: Standard, 3 signals.
- 2026-11-22 — recon digest (installed: tauri 2.12.6, @tauri-apps/cli 2.12.5, Rust 1.100.0):
  - `ubuntu-22.04-arm`, `ubuntu-24.04-arm` and `ubuntu-26.04-arm` are standard runners, free on a public repository (GitHub docs, hosted runners reference).
  - `ubuntu-22.04` and `ubuntu-22.04-arm` are deprecated since 2026-09-17 and removed on 2027-04-17, with brownouts from 2027-03-23 (actions/runner-images#14254).
  - Tauri's docs build the AppImage and `.deb` natively on an ARM runner with no changes; its workflow example uses `ubuntu-22.04-arm` and the same packages. The bundler fetches `linuxdeploy-aarch64` and sets `APPIMAGE_EXTRACT_AND_RUN` itself. Outputs: `_aarch64.AppImage`, `_arm64.deb` with `Architecture: arm64`.
  - `aarch64-unknown-linux-musl` is Tier 2 with host tools; the 1.100.0 manifest ships rust-std for it. The root `Cargo.lock` has no `-sys` crate and no `cc`.
  - Unconfirmed before the dry run: `--appimage-extract` on an aarch64 AppImage; that musl needs no extra package on the ARM runner.
  - Avoid: QEMU emulation (about an hour a build), cross-building the AppImage (linuxdeploy does not support it).
- 2026-11-22 — the test script of 2026-11-14, from the retired roadmap: open, new folder, undo and redo, a drop from the Files app, Save As to a path with spaces, the unsaved-changes question, the hex window, About.
