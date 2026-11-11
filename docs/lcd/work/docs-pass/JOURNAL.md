# JOURNAL — docs-pass

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the *entire*
> cold-start payload. Keep NOW and STEPS current as you work. Everything below the `---` is
> history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Standard
- **Goal:** make every README and doc page consistent and readable for public C64/EasyFlash hobbyists before the repo goes public (it stays PRIVATE until the user's explicit ok)
- **Next action:** load `anthropic-skills:doc-coauthoring`, re-ask the 8 OPEN QUESTIONS below, then agree the structure (S1)
- **Branch:** docs/docs-pass  ·  **Updated:** 2026-11-10

## STEPS
- [ ] S1 — agree the doc structure with the user (README vs docs/guide vs CONTRIBUTING vs menu/README)  ← next
- [ ] S2 — README.md rewritten section by section with the user; no D-/AC- numbers
- [ ] S3 — user guide pages in docs/ carry the CLI, desktop app and menu detail moved out of the READMEs
- [ ] S4 — CONTRIBUTING.md: build, gate, toolchain pin, LCD process, AI-assisted disclosure, hobby expectations
- [ ] S5 — stale statements fixed: menu/README opt-in/EasyLoader default (D-064), group→folder (D-060), stidcart-cli "test harness", GUI Cargo "viewer" description
- [ ] S6 — reader test with a fresh subagent; consistency sweep across all pages; gate green; PR

## DECISIONS (this work-item)
- Docs only, no behaviour change; no ACs, so no audit.

## OPEN QUESTIONS
Stage 1 of doc-coauthoring, asked 2026-11-10, unanswered. Re-ask these first:
1. Main reader: EF3 owners who only build carts, or also tinkerers? (proposal: users; CONTRIBUTING for tinkerers)
2. What should a reader do after reading: download and try, build from source, or just know it exists?
3. Install: source-only, or prebuilt GitHub downloads (unsigned macOS app)? Which platforms are really used?
4. Voice: first person or neutral; tell the short "why this exists" (ndefpack revival) story?
5. Screenshots: user photo of the menu on the breadbin + app screenshot, or capture in VICE / on the Mac?
6. AI disclosure: one README line + detail in CONTRIBUTING, or more up front?
7. Contributions: PRs welcome, or "issues welcome, PRs maybe, no promises"?
8. Structure OK? README short · docs/guide/{cli,app,menu}.md · CONTRIBUTING.md · menu/README.md internals only · docs/format unchanged

## EDIT BOUNDARY (paths this work may touch)
- `README.md`
- `CONTRIBUTING.md`
- `docs/guide/`
- `menu/README.md`
- `crates/stidcart-cli/Cargo.toml`
- `gui/src-tauri/Cargo.toml`
- `docs/lcd/MAP.md`
- `docs/lcd/work/docs-pass/JOURNAL.md`
<!-- /lcd-resume -->

---

## LOG (append-only; not read on resume)

- 2026-11-10 — triaged Standard (2 signals). Branch cut from main at de881bf (credits PR #64 merged).
