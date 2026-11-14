# JOURNAL — release-workflow

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the *entire*
> cold-start payload. `/lcd:resume` reads MAP.md + this block + DECISIONS headers and nothing else.

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Standard
- **Goal:** A `v*` tag builds the command for macOS universal and Linux x86_64 and the macOS app, and attaches them with checksums to a draft GitHub release; "Run workflow" does the same build as a dry run with no release (step 1 of the release roadmap, D-072).
- **Next action:** write `.github/workflows/release.yml` from the recon digest (S3).
- **Branch:** feat/release-workflow  ·  **Updated:** 2026-11-14 21:00

## STEPS
- [x] S1 — `release-version.sh` and its test: the tag must name the version, and the app's manifest must agree
- [x] S2 — scope rule and test for `release.yml`
- [ ] S3 — `release.yml`: version job, command archives, macOS app, checksums, draft release; signing skipped without secrets  ← next
- [ ] S4 — dry run on the branch ("Run workflow"), read the artifacts: archive contents, universal binaries, version, checksums
- [ ] S5 — docs: CLAUDE.md scope table, CONTRIBUTING (how to cut a release), MAP entry
- [ ] S6 — PR

## DECISIONS (this work-item)
- The version check is a script under `docs/lcd/ci/` with a cargo test, so the one rule a release must not break is tested on every gate run that touches it.
- `release.yml` is scoped to no tests: nothing reads it, and its check is the dry run.

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
