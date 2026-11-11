# JOURNAL — docs-pass

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the *entire*
> cold-start payload. Keep NOW and STEPS current as you work. Everything below the `---` is
> history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Standard
- **Goal:** make every README and doc page consistent and readable for public C64/EasyFlash hobbyists before the repo goes public (it stays PRIVATE until the user's explicit ok)
- **Next action:** take the user's corrections to the README draft (S2). Then S3: write `docs/guide/menu.md` from `menu/README.md`'s user sections and edit `cli.md` / `app.md`, which so far hold the old README's text moved as it was. The app screenshot needs the user's hands off the keyboard, or the user takes it
- **Branch:** docs/docs-pass  ·  **Updated:** 2026-11-10

## STEPS
- [x] S1 — agree the doc structure with the user (README vs docs/guide vs CONTRIBUTING vs menu/README)
- [ ] S2 — README.md rewritten section by section with the user; no D-/AC- numbers  ← next (first full draft committed 2026-11-10, awaiting the user's read; app screenshot missing)
- [ ] S3 — user guide pages in docs/ carry the CLI, desktop app and menu detail moved out of the READMEs
- [ ] S4 — CONTRIBUTING.md: build, gate, toolchain pin, LCD process, AI-assisted disclosure, hobby expectations
- [ ] S5 — stale statements fixed: menu/README opt-in/EasyLoader default (D-064), group→folder (D-060), stidcart-cli "test harness", GUI Cargo "viewer" description
- [ ] S6 — reader test with a fresh subagent; consistency sweep across all pages; gate green; PR

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

- 2026-11-10 — README first draft; old README detail moved unedited (decision numbers stripped) into docs/guide/cli.md, docs/guide/app.md and CONTRIBUTING.md; menu screenshot captured in VICE from a demo cart of fixture inputs. An attempt to screenshot the app typed into the user's other terminal session when focus moved; stopped, not retried.
- 2026-11-10 — stage 1 context gathered: user brief plus answers on install, platforms, screenshots, contributions. Structure proposed.
- 2026-11-10 — triaged Standard (2 signals). Branch cut from main at de881bf (credits PR #64 merged).
