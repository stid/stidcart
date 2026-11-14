# JOURNAL — release-workflow

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the *entire*
> cold-start payload. `/lcd:resume` reads MAP.md + this block + DECISIONS headers and nothing else.

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Standard
- **Goal:** A `v*` tag builds the command for macOS universal and Linux x86_64 and the macOS app, and attaches them with checksums to a draft GitHub release; "Run workflow" does the same build as a dry run with no release (step 1 of the release roadmap, D-072).
- **Next action:** PR #84 is ready for the owner to merge (after #83, which carries D-072). Then closeout. The tag path (draft release) has not run yet: the first tag on `main` is its first test.
- **Branch:** feat/release-workflow  ·  **Updated:** 2026-11-14 21:40

## STEPS
- [x] S1 — `release-version.sh` and its test: the tag must name the version, and the app's manifest must agree
- [x] S2 — scope rule and test for `release.yml`
- [x] S3 — `release.yml`: version job, command archives, macOS app, checksums, draft release; the app signed ad hoc until `macos-signing`
- [x] S4 — dry run on the PR, artifacts read: archive contents, universal binaries, version, checksums
- [x] S5 — docs: CLAUDE.md scope table, CONTRIBUTING (how to cut a release), MAP entry
- [x] S6 — PR #84
- [ ] S7 — after merge: closeout, tick step 1 in `docs/release-roadmap.md`  ← next

## DECISIONS (this work-item)
- The version check is a script under `docs/lcd/ci/` with a cargo test, so the one rule a release must not break is tested on every gate run that touches it.
- `release.yml` is scoped to no tests: nothing reads it, and its check is the dry run.
- The dry run is a `pull_request` trigger on the workflow's own paths, because "Run workflow" only exists once the file is on `main`.
- No `tauri-action`: recon could not confirm it handles `bundle.active: false` with a `--config` override, and `tauri build` plus `gh release create` is what `scripts/install.sh` already does.
- Linux command is static musl, so the runner's glibc does not matter.
- Archives are named `stidcart-cli-<version>-<platform>.tar.gz`; the app is `stidcart-<version>-macos-universal.dmg`.

## OPEN QUESTIONS
- none

## EDIT BOUNDARY (paths this work may touch)
- `.github/workflows/release.yml`
- `docs/lcd/ci/release-version.sh`
- `docs/lcd/ci/scope.sh`
- `crates/stidcart-cli/tests/release_version.rs`
- `crates/stidcart-cli/tests/scope.rs`
- `docs/lcd/work/release-workflow/JOURNAL.md`
- `docs/lcd/triage-log.md`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/MAP.md`
- `docs/lcd/SPEC.md`
- `CLAUDE.md`
- `CONTRIBUTING.md`
<!-- /lcd-resume -->

---

## LOG (append-only; not read on resume)

- 2026-11-14 20:40 — triage: Standard. Recon dispatched for action versions, Tauri bundling flags, runner labels.
- 2026-11-14 21:00 — S1 and S2 done, tests first (7 in `release_version`, 1 added to `scope`).
- 2026-11-14 21:10 — recon digest: tauri-action is at v1 but unconfirmed for our config; macos-latest is macOS 26 arm64; upload-artifact v7, download-artifact v8; Tauri notarizes the .app but not the .dmg (matters for `macos-signing`); Actions billing is per minute in dollars, macOS about ten times Linux, public repos free.
- 2026-11-14 21:35 — dry run 43637352922 green on the first try, about 3.5 minutes. Downloaded the `release` artifact: checksums verify; both archives hold `stidcart`, `COPYING`, `COPYING.firmware`; the macOS command is x86_64 + arm64 and lists a fixture here; the Linux one is static-pie; the .dmg mounts, the app says 0.30.0, is x86_64 + arm64, ad-hoc signed, and Gatekeeper rejects it, as expected before signing.
- 2026-11-14 21:40 — not exercised: the tag path (`gh release create --draft`, the tag-is-on-main check). Needs a real tag on `main`.
