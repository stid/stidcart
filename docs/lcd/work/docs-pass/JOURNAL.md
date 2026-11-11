# JOURNAL — docs-pass

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the *entire*
> cold-start payload. Keep NOW and STEPS current as you work. Everything below the `---` is
> history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Standard
- **Goal:** make every README and doc page consistent and readable for public C64/EasyFlash hobbyists before the repo goes public (it stays PRIVATE until the user's explicit ok)
- **Next action:** PR is open; take the user's review comments. Merge only on the user's word. The repo stays private
- **Branch:** docs/docs-pass  ·  **Updated:** 2026-11-10

## STEPS
- [x] S1 — agree the doc structure with the user (README vs docs/guide vs CONTRIBUTING vs menu/README)
- [x] S2 — README.md rewritten; no D-/AC- numbers. The user's answers applied, and a pointer to stid.me added at their request
- [x] S3 — user guide pages in docs/guide/ (cli.md, app.md, menu.md)
- [x] S4 — CONTRIBUTING.md: build, gate, toolchain pin, AI disclosure, hobby expectations
- [x] S5 — stale statements fixed
- [x] S6 — reader test, fixes, the four polish items, PR opened  ← the user reads the PR next

## DECISIONS (this work-item)
- Docs only, no behaviour change; no ACs, so no audit.
- User brief (2026-11-10): hobby, non-commercial. Started as a private tool because other EasyFlash tools were a constant struggle to get running, especially on new Macs; shared because it seemed worth sharing. The only aim is a contribution to the Commodore community.
- The community is careful about copyright. The docs answer that with exact credits and licences stated once, plainly, and an open invitation to report a missed credit. No defensive or legalistic tone.

## OPEN QUESTIONS
Answered by the user's general brief (2026-11-10); the readings marked "inferred" are Claude's, not the user's words:
- Q1 reader: people like the user, who want to manage an EasyFlash cart and struggle to get other tools running, especially on new Macs (inferred: users first, CONTRIBUTING for tinkerers).
- Q4 voice: first person, and tell the short story (inferred from how the user told it).
- Q6/credits tone: plain and modest, "nothing to pretend". Credits say: if I missed someone, tell me and I will add them.
Answered by the user 2026-11-10:
- Q3 install: source only (Rust + Node, `scripts/install.sh` on macOS). No promise of prebuilt downloads.
- Platforms: developed and used on macOS; Linux builds in CI but is not used day to day; Windows untried; reports welcome.
- Q5 screenshots: Claude captures both (menu in VICE, app window on the Mac) and leaves a marked spot for a breadbin photo.
- Q7 contributions: issues welcome (bugs, missed credits, compatibility reports); PRs looked at when time allows, no promises.
- Q8 structure and README outline: ok (2026-11-10).
- README checks answered 2026-11-10: AI wording "Most of the code was written by Claude Code... I set the direction, make the decisions and test on real hardware"; flashing is EasyProg from SD or disk; tested on the EF3 only, not an original EasyFlash; the menu's return after reset works with the EF3's RESET button on the breadbin.
- Docs-only commits (Markdown, images) skip the local gate; the Cargo.toml edit and the pre-PR run keep it (told to the user 2026-11-10, who had asked why tests ran for a docs change).

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

- 2026-11-10 — after the PR opened, at the user's request: sizes are KB (1024) throughout cli.md, hide and align described by effect only (user's choice, no use case claimed), EasyFS1/EasyFS2 and the subdirectory terms put in plain words.
- 2026-11-10 — polished menu.md (screen section, menu comparison table), app.md (Folders section), CONTRIBUTING (write-path history in plain terms); README points to stid.me. The user chose to leave CLAUDE.md's gate rule as it is.
- 2026-11-10 — S6 reader test run (fresh subagent). Fixed: unknown-loader contradiction in cli.md, full option list added to cli.md, Ocean needs EasyLoader stated, EAPI/EasyProg/ndeft explained. Left for a later pass: menu.md's long screen-layout paragraph and raw row numbers, no single stidcart-menu-vs-EasyLoader comparison, CONTRIBUTING's "why the write path came first" still in internal terms, app.md's dense folder bullets.
- 2026-11-10 — S3 and S5 done: docs/guide/menu.md split out of menu/README.md, cli.md rewritten, Cargo descriptions fixed, both gates green.
- 2026-11-10 — README first draft; old README detail moved unedited (decision numbers stripped) into docs/guide/cli.md, docs/guide/app.md and CONTRIBUTING.md; menu screenshot captured in VICE from a demo cart of fixture inputs. An attempt to screenshot the app typed into the user's other terminal session when focus moved; stopped, not retried.
- 2026-11-10 — stage 1 context gathered: user brief plus answers on install, platforms, screenshots, contributions. Structure proposed.
- 2026-11-10 — triaged Standard (2 signals). Branch cut from main at de881bf (credits PR #64 merged).
