# JOURNAL — older-menus

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the _entire_
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->

## NOW

- **Lane:** Standard
- **Goal:** A cart carrying an earlier build of our menu opens editable as "stidcart (older build)", and saving it writes the current build (D-049)
- **Next action:** done; the PR awaits CI and the user's merge. Then the foreign-loader work-item (an explicit menu switch replaces an unrecognised loader)
- **Branch:** fix/older-menus · **Updated:** 2026-11-08

## STEPS

- [x] S1 — version 0.16.0 → 0.16.1 (first commit on the branch)
- [x] S2 — triage, D-049, AC-129..AC-131 claimed, this JOURNAL
- [x] S3 — failing tests: fingerprints (AC-131), `list` and `edit` on an older-build cart (AC-129), the window's picker (AC-130)
- [x] S4 — fingerprints in `menu.rs`, detection in `reserved.rs`, the listing's name, the picker's extra option; `menu/past/5b54e81.bin`
- [x] S5 — live check on a scratchpad copy of the user's `stidcollection.crt`, gates, audit, reconcile, PR

## DECISIONS (this work-item)

- earlier builds by fingerprint, loaded as stidcart, the current build written on save → D-049
- replacing a foreign loader by an explicit menu switch is the next work-item, not this one

## OPEN QUESTIONS

- none

## EDIT BOUNDARY (paths this work may touch)

- `crates/stidcart-core/src/menu.rs`
- `crates/stidcart-core/src/reserved.rs`
- `crates/stidcart-core/src/read.rs`
- `crates/stidcart-core/src/listing.rs`
- `crates/stidcart-core/tests/`
- `crates/stidcart-cli/src/lib.rs`
- `crates/stidcart-cli/tests/`
- `gui/src-tauri/tests/`
- `gui/src/lib/Listing.svelte`
- `gui/src/lib/Listing.test.ts`
- `gui/src/lib/types.ts`
- `menu/past/`
- `menu/README.md`
- `docs/lcd/SPEC.md`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/triage-log.md`
- `docs/lcd/work/older-menus/JOURNAL.md`
- `.claude/rules/lcd-conventions.md`

<!-- /lcd-resume -->

---

## Acceptance criteria

**AC-129** (surfaces: CLI): Given a cart carrying an earlier build of our menu, then `list` reports
`menu: stidcart (older build)`, and `edit` accepts it. `edit --menu stidcart` writes the current
build, and the result differs from the input only inside the loader slot. Any other edit writes the
current build too.

**AC-130** (surfaces: RENDER): Given a cart carrying an earlier build of our menu, when the window
opens it, then it is editable and its menu picker shows "stidcart (older build)". Choosing stidcart
is one undoable edit that writes the current build.

**AC-131** (surfaces: none): Given every committed build of `menu/menu.bin`, then each is
recognised as stidcart, the current build's fingerprint is pinned, and a slot matching no build is
still unrecognised.

## LOG (append-only; not read on resume)

- 2026-11-08 — the user opened `stidcollection.crt` in 0.16.0 and got view-only "menu
  unrecognised". Its loader slot is the menu at `5b54e81`, from before 0.16.0's change. Eight
  committed builds of `menu.bin` exist (`0d3dfba` … `e8eb154`). The user chose to recognise earlier
  builds and to let a menu switch replace a foreign loader. Split in two: this item covers earlier
  builds, the foreign loader follows. Triage: Standard, 2 signals. D-049. AC-129..AC-131 claimed,
  next free AC-132.
- 2026-11-08 — red as expected: `menu_builds` did not compile (no `fingerprint`), the three
  `older_menus` CLI cases failed, and so did the picker test. Green after S4: root 245/0 with
  STIDCART_REQUIRE_TOOLS=1, GUI 39/0 cargo + 55/0 vitest. `menu/past/5b54e81.bin` is the build the
  user's cart carries.
- 2026-11-08 — live check on a scratchpad copy of the user's `stidcollection.crt` (41 entries):
  `list` says `stidcart (older build)`; `edit -o … --menu stidcart` lists `stidcart`, the same 41 rows
  and 578.7 KB free, and the 3915 bytes that differ all sit inside the loader slot. The Desktop
  copy was not touched. The upgraded copy opened in VICE for the user.
- 2026-11-08 — audit: PASS (test-presence). `AC-129 (CLI)` in `crates/stidcart-cli/tests/older_menus.rs`, `AC-130 (RENDER)` in `gui/src-tauri/tests/menu.rs` and `Listing.test.ts`, `AC-131 (none)` in `crates/stidcart-core/tests/menu_builds.rs`, green at 6b33d0d. Closeout line appended.
- 2026-11-08 — reconciled: folded AC-129..AC-131 → SPEC.md/CLI (the `list` and `edit` rows), RENDER (the window's menu row) and none (the `inspect`/`load` row), each rewritten in place.
