# JOURNAL — macos-signing

<!-- lcd-resume:v1 -->

## NOW

- **Lane:** Standard
- **Goal:** `release.yml` signs the macOS app and command with a Developer ID, notarizes them and staples the `.dmg`, and still builds ad hoc when the secrets are absent.
- **Next action:** done. Merge PR #97 once its checks are green.
- **Branch:** ci/macos-signing · **Updated:** 2026-11-16 10:00

## STEPS

- [x] S1 — triage, branch, JOURNAL
- [x] S2 — recon: Tauri's signing and notarization variables, what it staples, the bare-binary path
- [x] S3 — `release.yml`: sign, notarize and staple in `app`; sign and notarize the macOS `command`; both skipped without secrets; checks (`codesign`, `spctl`, `stapler validate`) run only when signed
- [x] S4 — owner steps written into `CONTRIBUTING.md` (certificate, API key, `gh secret set`)
- [x] S5 — owner sets the secrets; PR dry run goes green with signing on
- [x] S6 — owner downloads the dry-run `.dmg` through a browser: opens with no Gatekeeper prompt, `spctl -a -vv` reports a notarized Developer ID
- [x] S7 — decision logged, roadmap step 3 ticked, SPEC and README mention updated if they say "unsigned", closeout

## DECISIONS (this work-item)

- The standalone macOS command is signed and notarized too (open in the roadmap): a browser download is quarantined, and the certificate is already there. → D-075.

## OPEN QUESTIONS

- none blocking the workflow change; S5 and S6 wait on the owner.

## EDIT BOUNDARY (paths this work may touch)

- `.github/workflows/release.yml`
- `CONTRIBUTING.md`
- `README.md`
- `docs/release-roadmap.md`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/SPEC.md`
- `docs/lcd/work/macos-signing/JOURNAL.md`

<!-- /lcd-resume -->

---

## LOG (append-only; not read on resume)

- 2026-11-16 — Owner reported the Apple Developer account active. Triage → Standard (3 signals). At the start: no code-signing identity in the local keychain, no repository secrets, no tags.
- 2026-11-16 — Recon (tauri-cli 2.12.2, bundler 2.10.2, read from the tagged source and v2.tauri.app):
  - Tauri imports `APPLE_CERTIFICATE` + `APPLE_CERTIFICATE_PASSWORD` into a keychain of its own; `APPLE_API_KEY`, `APPLE_API_ISSUER`, `APPLE_API_KEY_PATH` notarize. An empty variable counts as set, so they are exported only when the secrets exist.
  - Tauri notarizes and staples the app, signs the `.dmg`, and does not notarize the `.dmg`. Missing credentials only log a line and a stapler failure is ignored, so the job asserts with `stapler validate` and `spctl`.
  - A bare binary: sign with `--options runtime --timestamp`, notarize inside a `.zip`, no stapling.
  - Not confirmed: that `spctl` assesses on a GitHub runner; that an individually enrolled account can make a team API key (fallback: Apple ID, app-specific password, team ID); that the app needs no entitlements. The first signed dry run answers the first and third.
- 2026-11-16 — S3, S4 written. `actionlint` clean. A tag now fails in `version` when the secrets are absent.
- 2026-11-16 — A background security review of the first commit flagged key files and an unlocked keychain outliving their step. Fixed: a trap removes them, and `npm ci` got a step with no secret.
- 2026-11-16 — S5: owner set the five secrets. Signed dry run 44026625252: app and `.dmg` notarized `Accepted` and stapled, `spctl` on the runner `source=Notarized Developer ID` for both; command `explicit requirement satisfied`. The three unknowns are settled: `spctl` works on a runner, no entitlements needed, a team key from an individual account is accepted.
- 2026-11-16 — S6: owner downloaded the `.dmg` in Safari and opened the app; the dialog said Apple checked it. `spctl` on the quarantined copy: `accepted`, `Notarized Developer ID`. → D-075. No ACs, so nothing to fold into SPEC. Closeout: audit n/a.
- 2026-11-16 — Run 44029447715: the Linux "Start it" step failed once with "Failed to initialize GTK"; the same job passed before and on a re-run of that job, which made the run green with signing on. Likely cause, not confirmed: Xvfb resetting after `xdpyinfo` disconnects. Added `-noreset` at the owner's choice.
