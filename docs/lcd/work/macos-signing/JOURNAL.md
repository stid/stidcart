# JOURNAL — macos-signing

<!-- lcd-resume:v1 -->

## NOW

- **Lane:** Standard
- **Goal:** `release.yml` signs the macOS app and command with a Developer ID, notarizes them and staples the `.dmg`, and still builds ad hoc when the secrets are absent.
- **Next action:** read the recon digest, then write the signing steps into the `app` and `command` jobs (S3).
- **Branch:** ci/macos-signing · **Updated:** 2026-11-16 10:00

## STEPS

- [x] S1 — triage, branch, JOURNAL
- [ ] S2 — recon: Tauri's signing and notarization variables, what it staples, the bare-binary path ← next
- [ ] S3 — `release.yml`: sign, notarize and staple in `app`; sign and notarize the macOS `command`; both skipped without secrets; checks (`codesign`, `spctl`, `stapler validate`) run only when signed
- [ ] S4 — owner steps written into `CONTRIBUTING.md` (certificate, API key, `gh secret set`)
- [ ] S5 — owner sets the secrets; PR dry run goes green with signing on
- [ ] S6 — owner downloads the dry-run `.dmg` through a browser: opens with no Gatekeeper prompt, `spctl -a -vv` reports a notarized Developer ID
- [ ] S7 — decision logged, roadmap step 3 ticked, SPEC and README mention updated if they say "unsigned", closeout

## DECISIONS (this work-item)

- The standalone macOS command is signed and notarized too (open in the roadmap): a browser download is quarantined, and the certificate is already there. → D-NNN at S7.

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
