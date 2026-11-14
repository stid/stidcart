# JOURNAL — windows-app

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the *entire*
> cold-start payload. `/lcd:resume` reads MAP.md + this block + DECISIONS headers and nothing else.

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Standard
- **Goal:** The release workflow builds the command as a `.zip` and the app as an installer for Windows x64, and the window is right on Windows, so the owner can try a dry-run build on a real PC (step 5 of the release roadmap, D-072).
- **Next action:** read the first probe run's screenshots (the window before any change), then write the failing tests for S3.
- **Branch:** feat/release-windows  ·  **Updated:** 2026-11-14 23:20

## STEPS
- [x] S1 — recon: Tauri Windows bundle targets, WebView2, what the macOS title bar keys do on Windows
- [ ] S2 — temporary probe workflow: build both Windows legs, install the app, screenshot it  ← next
- [ ] S3 — the window on Windows: tests first, then the fixes the screenshots call for
- [ ] S4 — `release.yml`: the `windows-x86_64` command leg and the `app-windows` job, both in `release`
- [ ] S5 — PR, dry run green, the `release` artifact read: names and checksums
- [ ] S6 — docs: CONTRIBUTING, the workflow's header; the probe workflow deleted
- [ ] S7 — after the owner's report from a real PC: closeout, step 5 ticked in `docs/release-roadmap.md`

## DECISIONS (this work-item)
- (filled in as they are made)

## OPEN QUESTIONS
- none

## EDIT BOUNDARY (paths this work may touch)
- `.github/workflows/release.yml`
- `.github/workflows/windows-probe.yml`
- `gui/src/App.svelte`
- `gui/src/App.test.ts`
- `gui/src/lib/platform.ts`
- `gui/src/lib/platform.test.ts`
- `gui/src/lib/EmptyState.svelte`
- `gui/src/lib/Toolbar.svelte`
- `gui/src/lib/Listing.svelte`
- `gui/src-tauri/src/menu.rs`
- `gui/src-tauri/src/lib.rs`
- `docs/lcd/work/windows-app/JOURNAL.md`
- `docs/lcd/triage-log.md`
- `docs/lcd/SPEC.md`
- `.claude/rules/lcd-conventions.md`
- `CONTRIBUTING.md`
- `README.md`
<!-- /lcd-resume -->

---

## Acceptance criteria

(added with S3, once the screenshots say what is wrong)

## LOG (append-only; not read on resume)

- 2026-11-14 23:10 — triage: Standard. Recon dispatched (Tauri 2 Windows bundling, WebView2, title bar keys, menu on Windows, static CRT).
- 2026-11-14 23:20 — probe workflow pushed: `windows-probe.yml`, on this branch only, so the macOS jobs of `release.yml` do not run for every look at the window.
