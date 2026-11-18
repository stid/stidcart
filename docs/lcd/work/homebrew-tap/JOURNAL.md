# JOURNAL — homebrew-tap

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the _entire_
> cold-start payload. Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->

## NOW

- **Lane:** Standard
- **Goal:** `brew install` from the project's own tap, `stid/homebrew-tap`, gives a working app and command on a Mac, and each published release updates the tap (roadmap step 7).
- **Next action:** open the PR and read its `tap.yml` dry run. Then the owner sets `TAP_TOKEN` (S4), and the next release shows the push and `brew upgrade` (S7).
- **Branch:** feat/homebrew-tap · **Updated:** 2026-11-17 20:24

## STEPS

- [x] S1 — triage and recon dispatched
- [x] S2 — layout settled: formula `stidcart` and cask `stidcart-app` (D-077)
- [x] S3 — create `stid/homebrew-tap` (owner approved, Claude creates it); recipes written against v0.31.1, checked with `brew style` and `brew audit`, and installed on a clean runner by the tap's own workflow
- [ ] S4 — owner makes a token scoped to the tap and sets it as a repository secret
- [x] S5 — `.github/workflows/tap.yml` updates the tap when a release is published; scope rule and test added
- [x] S6 — README and CONTRIBUTING say how to install with Homebrew and what the release does to the tap; roadmap steps 6 and 7 ticked
- [ ] S7 — check on a clean Mac; `brew upgrade` checked at the next release

## DECISIONS (this work-item)

- The project's own tap, not the official repositories (roadmap, confirmed by the owner 2026-11-17).
- Formula `stidcart` and cask `stidcart-app`, updated by our own job on `release: published` (D-077).

## OPEN QUESTIONS

- none
- The release is created as a draft and published by hand, and a draft's files cannot be downloaded anonymously. So the tap update has to run when the release is published, not in the job that creates the draft.

## EDIT BOUNDARY (paths this work may touch)

- `.github/workflows/release.yml`
- `.github/workflows/tap.yml`
- `README.md`
- `CONTRIBUTING.md`
- `docs/release-roadmap.md`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/MAP.md`
- `CLAUDE.md`
- `docs/lcd/ci/scope.sh`
- `crates/stidcart-cli/tests/scope.rs`
- `docs/lcd/work/homebrew-tap/JOURNAL.md`

<!-- /lcd-resume -->

---

## LOG (append-only; not read on resume)

- 2026-11-17 20:24 — Triage: Standard, 3 signals. State checked: the repository is public, v0.31.1 is published with the signed `.dmg` and the signed command tarball, `stid/homebrew-tap` does not exist. The app bundle does not hold the command (no `externalBin` in `tauri.conf.json`); the command ships as its own notarized tarball. Recon dispatched on tap layout, cask stanzas and the update mechanism.
- 2026-11-17 20:40 — Recon digest (checked against docs.brew.sh for Homebrew 7.0.x and the actions' READMEs):
  - Layout is `Casks/<token>.rb` and `Formula/<name>.rb` in `<owner>/homebrew-tap`. Since Homebrew 6.0.0 (2026-06-11) a third-party tap needs explicit trust: the short name fails after `brew tap` until `brew trust`, while the fully qualified `brew install owner/tap/x` taps and trusts that one item. The README gives the qualified form. https://docs.brew.sh/Tap-Trust
  - A cask and a formula must not share a token: the formula wins with a warning, and the docs say to suffix the cask `-app`. A cask takes one `url`, so it cannot fetch the command's tarball as well. A `binary` stanza into the `.app` is documented but needs the command inside the bundle. https://docs.brew.sh/Cask-Cookbook
  - Quarantine is documented for cask downloads only. A formula installing a prebuilt binary is not forbidden in a personal tap (the from-source rule is homebrew/core's). Both of our macOS files are Developer ID signed and notarized, so either passes Gatekeeper. `--no-quarantine` is gone.
  - Cask stanzas: one `url` and `sha256` (universal, no `arch`), `depends_on macos:`, `zap trash:` for `dev.stid.stidcart` paths, no `auto_updates`. `livecheck` with `url :url` and `strategy :github_latest`, because the tag exists while the release is still a draft. https://docs.brew.sh/Brew-Livecheck
  - Update mechanism with the fewest parts: a hand-written job that checks out the tap with a fine-grained token (Contents: write on that repository only), rewrites `version` and `sha256` from `SHA256SUMS`, commits and pushes. The bump actions (`mislav/bump-homebrew-formula-action` v4, `dawidd6/action-homebrew-bump-formula` v11, `macauley/action-homebrew-bump-cask` v1) want a classic token, open pull requests, or support casks poorly.
  - Checks: `brew style owner/tap`, `brew audit --tap owner/tap --strict --online`, `brew test-bot --only-tap-syntax`, then an install by qualified name. `brew tap-new` scaffolds for bottles built from source; take its tests workflow at most.
  - Avoid: cask `preflight`/`postflight` blocks (deprecated in 7.0.0), `HOMEBREW_NO_REQUIRE_TAP_TRUST`, Intel-only handling, and updating the tap from the tag run. Trigger on `release: published`.
  - Recommendation: cask `stidcart-app` for the `.dmg` and formula `stidcart` for the command tarball, updated by a hand-written job on `release: published`.
- 2026-11-17 20:55 — Owner chose the formula and cask layout and asked Claude to create the tap repository. → D-077.
- 2026-11-17 21:35 — S3: `stid/homebrew-tap` created and pushed. Recipes pass `brew style`, `brew audit --strict --online` and `brew livecheck` locally (Homebrew 7.0.14), and the tap's `Tests` run installed both on `macos-latest`; `spctl` accepted the app. `depends_on macos:` with a version is disabled in 7.0, so the cask says `depends_on :macos`. S5: `tap.yml` written, the rewrite simulated locally against a made-up 0.32.0, full gate green.
- 2026-11-17 22:10 — The cask cannot depend on the formula: installed by its full name from an untrusted tap, Homebrew trusted the cask and refused to load the formula. The dependency is gone and the tap's `Tests` run installs each alone from an untrusted tap; `spctl` reports the app as Notarized Developer ID. The setup action in the tap is pinned to a commit.
- 2026-11-17 22:10 — Owner asked for public-repository hardening. Applied: approval for all outside contributors' workflow runs (both repositories), ruleset "release tags", immutable releases, CodeQL default setup, a ruleset on the tap's `main`. Owner declined a release environment. → D-078. Pinning actions and Dependabot go in their own PR.
