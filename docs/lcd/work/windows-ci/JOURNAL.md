# JOURNAL — windows-ci

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the *entire*
> cold-start payload. Everything below the `---` is history, not read on resume.

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Standard
- **Goal:** a `windows` CI job runs the root workspace tests on `windows-latest`, and they pass, so a Windows build of the command can ship later (release roadmap step 2).
- **Next action:** read the Windows job on PR #85 after the two fixes; fix anything left.
- **Branch:** ci/windows-tests  ·  **Updated:** 2026-11-14 21:00

## STEPS
- [x] S1 — add the `windows` job to `ci.yml` (scope from the Scope step, pinned toolchain, `cargo test --workspace --no-fail-fast`, no `STIDCART_REQUIRE_TOOLS`)
- [x] S2 — open the PR (#85) and read the Windows job's failures
- [x] S3 — fix each failure, one cause per commit: CRLF goldens (`.gitattributes`), WSL `bash.exe` (`tests/support/bash.rs`)
- [ ] S4 — Windows job green; `gate` and `gui` still green  ← next

## DECISIONS (this work-item)
- `--no-fail-fast` on the Windows job: it reports what breaks, so one run shows every failing target.
- `Command::new("bash")` goes through `tests/support/bash.rs`, which sets PATH on the child so std searches it before System32 (WSL's `bash.exe`). A shared helper rather than a job-level fix: no CI setting changes std's search order.
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
- 2026-11-14 — run 1 (fail-fast): every `list_golden` case failed with identical-looking output.
- 2026-11-14 — run 2 (diagnostic step, `--no-fail-fast`): `core.autocrlf=true` from `C:/Program Files/Git/etc/gitconfig`; goldens `i/lf w/crlf`. Four targets failed: `list_golden`, `meta_cli` (ac_158), core `listing` (CRLF goldens) and `scope` (all 12; `where bash` lists `C:\Windows\System32\bash.exe`). Git Bash runs the CRLF scripts fine, so the scripts stay as they are. Everything else passed; boots and 64tass reassembly skipped.
- 2026-11-14 — fixes: `.gitattributes` pins the goldens to LF; `tests/support/bash.rs` for scope.rs. Diagnostic step removed.
