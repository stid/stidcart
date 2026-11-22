# JOURNAL — listing-filter

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Standard
- **Goal:** A filter box above the app's entry list narrows it to the entries whose name or title contains the typed text, reachable with Edit ▸ Find (⌘F / Ctrl+F).
- **Next action:** S4, failing tests for the `find` menu id in `platform.test.ts` and `App.test.ts`
- **Branch:** feat/listing-filter  ·  **Updated:** 2026-11-22 12:00

## STEPS
- [x] S1 — `filterTree` in `tree.ts`, test first
- [x] S2 — the filter box in `Listing.svelte` (AC-219, AC-220), tests first
- [x] S3 — reordering off while filtering (AC-221), tests first
- [ ] S4 — Edit ▸ Find: `MenuId`, `menu.rs`, `platform.ts`, `App.svelte` (AC-222), tests first  ← next
- [ ] S5 — guide, SPEC fold, conventions AC line, gate, live check, PR

## DECISIONS (this work-item)
- Matches name and title, case-insensitive substring (user, 2026-11-22). Folder names are not matched.
- While a filter is on, drags and Move up/down are off for entries and folders; every other edit stays (user, 2026-11-22).
- An always-visible box plus Edit ▸ Find, Esc clears (user, 2026-11-22).
- The filter is view state in `Listing`: it never reaches the Rust side, and a new cart starts unfiltered (the listing is re-keyed per cart).

## OPEN QUESTIONS
- none

## EDIT BOUNDARY (paths this work may touch)
- `gui/src/lib/tree.ts`
- `gui/src/lib/tree.test.ts`
- `gui/src/lib/Listing.svelte`
- `gui/src/lib/Listing.test.ts`
- `gui/src/lib/types.ts`
- `gui/src/lib/platform.ts`
- `gui/src/lib/platform.test.ts`
- `gui/src/App.svelte`
- `gui/src/App.test.ts`
- `gui/src-tauri/src/menu.rs`
- `docs/guide/app.md`
- `docs/lcd/SPEC.md`
- `.claude/rules/lcd-conventions.md`
- `docs/lcd/work/listing-filter/JOURNAL.md`
<!-- /lcd-resume -->

---

## Acceptance criteria

**AC-219** (surfaces: RENDER): With entries in the cart, a filter box sits above the list. Typing in it shows only the entries whose name or title contains the text, ignoring case and surrounding spaces, and says how many of the entries match; with no match the list says so. Clearing the box, or Esc in it, shows every entry again.

**AC-220** (surfaces: RENDER): On a cart with folders, a filter shows only the folders holding a match, each with its matching entries and their count, open even when collapsed, then the matching entries in no folder.

**AC-221** (surfaces: RENDER): While a filter is on, entries and folders do not drag and their Move up and Move down are disabled; the other edits still act on the entry they are chosen for.

**AC-222** (surfaces: RENDER): Edit ▸ Find (⌘F, Ctrl+F off a Mac) puts the cursor in the filter box; on Windows the shortcut typed in the page is forwarded as that menu item.

## LOG (append-only; not read on resume)

- 2026-11-22 — triaged Standard; the three design forks answered by the user through the question dialog.
- 2026-11-22 — S1 to S3 green: the box filters by name and title, folders without a match drop out, drags and steps are off while filtering.
