# JOURNAL — replace-loader

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the _entire_
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->

## NOW

- **Lane:** Standard
- **Goal:** An explicit menu switch replaces a loader slot that holds no known menu, through `edit --menu` and the window's picker on a view-only cart (D-050)
- **Next action:** done; the PR awaits CI and the user's merge. The window's picker on a view-only cart is left to the user's live check
- **Branch:** feat/replace-loader · **Updated:** 2026-11-08

## STEPS

- [x] S1 — version 0.16.1 → 0.17.0 (first commit on the branch)
- [x] S2 — triage, D-050, AC-132..AC-135 claimed, this JOURNAL
- [x] S3 — failing tests: `edit --menu` on an unknown loader (AC-132, AC-133), the window's document (AC-134, AC-135), the picker and the viewer
- [x] S4 — `read::load_with_menu`; `edit` takes it with `--menu`; read-only per history step and `menuReplaceable` in the document; the picker on a view-only cart
- [x] S5 — contract golden, gates, live check (left to the user), audit, reconcile, PR

## DECISIONS (this work-item)

- the slot is replaced only by an explicit choice; EAPI, boot and the magic still held exact → D-050

## OPEN QUESTIONS

- none

## EDIT BOUNDARY (paths this work may touch)

- `crates/stidcart-core/src/read.rs`
- `crates/stidcart-core/src/reserved.rs`
- `crates/stidcart-core/src/lib.rs`
- `crates/stidcart-core/tests/`
- `crates/stidcart-cli/src/lib.rs`
- `crates/stidcart-cli/tests/`
- `gui/src-tauri/src/document.rs`
- `gui/src-tauri/tests/`
- `gui/src/App.svelte`
- `gui/src/lib/Listing.svelte`
- `gui/src/lib/Listing.test.ts`
- `gui/src/lib/viewer.svelte.ts`
- `gui/src/lib/viewer.test.ts`
- `gui/src/lib/types.ts`
- `gui/src/lib/fixtures/`
- `docs/lcd/SPEC.md`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/triage-log.md`
- `docs/lcd/work/replace-loader/JOURNAL.md`
- `.claude/rules/lcd-conventions.md`

<!-- /lcd-resume -->

---

## Acceptance criteria

**AC-132** (surfaces: CLI): Given a cart whose loader slot holds no known menu and whose other
reserved firmware is as embedded, when `edit --menu easyloader` or `--menu stidcart` is given,
then the slot holds that menu afterwards, the other operations in the same call apply, and with
`--menu` alone nothing outside the slot changes. Without `--menu`, `edit` refuses as before,
naming the menu loader.

**AC-133** (surfaces: CLI): Given a cart whose EAPI, boot region or EF-Name magic differs from what
this tool embeds, then `edit --menu` still refuses it, naming that region.

**AC-134** (surfaces: RENDER): Given a cart whose loader slot holds no known menu, when the window
opens it, then it is view-only with `edit`'s reason and offers the menu picker. Choosing a menu is
one undoable edit, after which the cart is editable and saves. Undoing returns it to view-only,
where saving and other edits are refused.

**AC-135** (surfaces: RENDER): Given a cart refused for any reason other than its loader slot, then
the window offers no menu picker and refuses a menu switch.

## LOG (append-only; not read on resume)

- 2026-11-08 — split from older-menus at the user's "Both": earlier builds of our menu shipped in
  #47 (0.16.1); this item replaces a loader the tool doesn't recognise. Triage: Standard, 3 signals
  (files, two surfaces on one path, the wire contract). D-050. AC-132..AC-135 claimed, next free
  AC-136.
- 2026-11-08 — red as expected: three of four `replace_loader` CLI cases (the refusal without
  `--menu` already held), the document tests (no `menu_replaceable`), the picker and the viewer.
  Green after S4: root 249/0 with STIDCART_REQUIRE_TOOLS=1, GUI 41/0 cargo + 59/0 vitest. The
  contract golden gained `menuReplaceable` and nothing else.
- 2026-11-08 — review found one state no test reached: switch, save, undo leaves the window at the
  cart as opened, unsaved and view-only, where Save in the unsaved-changes prompt did nothing
  silently (no loop: the prompt returns, and Don't Save works). Save now explains in a notice; a
  viewer test pins it. README, menu README and `edit --menu`'s help now say the switch replaces
  whatever loader the cart carries. 60/0 vitest.
- 2026-11-08 — no live check in the app was run: the picker on a view-only cart and the banner's
  hint are covered by `Listing.test.ts` and the document tests only, and are left to the user.
- 2026-11-08 — audit: PASS (test-presence). `AC-132 (CLI)` and `AC-133 (CLI)` in `crates/stidcart-cli/tests/replace_loader.rs`, `AC-134 (RENDER)` and `AC-135 (RENDER)` in `gui/src-tauri/tests/menu.rs`, `Listing.test.ts` and `viewer.test.ts`, green at 37dcc08. Closeout line appended.
- 2026-11-08 — reconciled: folded AC-132..AC-135 → SPEC.md/CLI (the `edit` and `edit` refusal rows), RENDER (the window's menu row) and none (the `inspect`/`load` row), each rewritten in place.
