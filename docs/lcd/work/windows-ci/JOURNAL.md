# JOURNAL — windows-ci

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the *entire*
> cold-start payload. Everything below the `---` is history, not read on resume.

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Standard
- **Goal:** a `windows` CI job runs the root workspace tests on `windows-latest`, and they pass, so a Windows build of the command can ship later (release roadmap step 2).
- **Next action:** push the branch with the new job and read its result with `gh`.
- **Branch:** ci/windows-tests  ·  **Updated:** 2026-11-14 00:00

## STEPS
- [ ] S1 — add the `windows` job to `ci.yml` (scope from the Scope step, pinned toolchain, `cargo test --workspace`, no `STIDCART_REQUIRE_TOOLS`)  ← next
- [ ] S2 — open the PR and read the Windows job's failures
- [ ] S3 — fix each failure, one cause per commit
- [ ] S4 — Windows job green; `gate` and `gui` still green

## DECISIONS (this work-item)
- Not a required check, and the emulator boots and 64tass reassembly skip on Windows: the job says whether the command works there, not whether the boots do.

## OPEN QUESTIONS
- none

## EDIT BOUNDARY (paths this work may touch)
- `.github/workflows/ci.yml`
- `.gitattributes`
- `crates/stidcart-cli/tests/`
- `crates/stidcart-core/tests/`
- `docs/lcd/work/windows-ci/`
<!-- /lcd-resume -->

---

## LOG (append-only; not read on resume)

- 2026-11-14 — triage: Standard (CI job plus whatever portability fixes it turns up; no CLI or GUI surface change, so no ACs).
