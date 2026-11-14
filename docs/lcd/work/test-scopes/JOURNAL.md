# JOURNAL — test-scopes

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Standard
- **Goal:** The gate runs only the test groups a change can affect, locally and in CI, with one flag for the whole suite.
- **Next action:** S1 — failing tests for `docs/lcd/ci/scope.sh`
- **Branch:** feat/test-scopes  ·  **Updated:** 2026-11-14 20:00

## STEPS
- [ ] S1 — `crates/stidcart-cli/tests/scope.rs` (failing), then `docs/lcd/ci/scope.sh`: changed paths in, `rust=` `vice=` `gui=` out  ← next
- [ ] S2 — `STIDCART_VICE=none|cart|full` in `support/vice.rs` (test first), cart-set boots marked
- [ ] S3 — `docs/lcd/ci/gate.sh`: scope the local gate, `--all` for everything
- [ ] S4 — `ci.yml`: both jobs always report, steps run by scope, `workflow_dispatch` runs all
- [ ] S5 — docs: D-071, CLAUDE.md, CONTRIBUTING.md, lcd-conventions `gate` key, SPEC/MAP if they say otherwise
- [ ] S6 — gate green, PR

## DECISIONS (this work-item)
- Scoped everywhere (PRs, pushes to main, local); the full suite is manual: `gate.sh --all`, "Run workflow" in CI (user, 2026-11-14)
- core/cli change without a menu change boots the cart-layout set, about 15 boots; a `menu/` change boots all (user, 2026-11-14)
- A path no rule names runs everything
- `gate` and `gui` stay always-running jobs with conditional steps, so the required checks report and a failed scope step fails the job rather than skipping it
- The cart set is marked in the tests (`require_cart_tool`), not listed by name in a script, so a rename cannot drop a boot from it

## OPEN QUESTIONS
- none

## EDIT BOUNDARY (paths this work may touch)
- `docs/lcd/ci/scope.sh`
- `docs/lcd/ci/gate.sh`
- `.github/workflows/ci.yml`
- `crates/stidcart-cli/tests/scope.rs`
- `crates/stidcart-cli/tests/support/vice.rs`
- `crates/stidcart-cli/tests/menu_boot.rs`
- `crates/stidcart-cli/tests/boot.rs`
- `crates/stidcart-cli/tests/meta_boot.rs`
- `CLAUDE.md`
- `CONTRIBUTING.md`
- `.claude/rules/lcd-conventions.md`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/SPEC.md`
- `docs/lcd/MAP.md`
- `docs/lcd/triage-log.md`
<!-- /lcd-resume -->

---

## LOG (append-only; not read on resume)

- 2026-11-14 20:00 — Measured on CI run 43630992169: `gate` Test step 130s, of which 118s is the 70 boots in `menu_boot`; `gui` job 2m20s, about 60s of it the gate script. Everything else in the root workspace is about 5s.
