# JOURNAL — raster-brand

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the _entire_
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->

## NOW

- **Lane:** Standard
- **Goal:** The window wears the stid design system (Catppuccin Mocha/Latte, peach, Geist bundled) with the "raster" cartridge as its mark, its icon and a five-band strip under the title bar (D-046, D-047)
- **Next action:** done; the PR awaits CI and the user's merge, then deploy the app locally
- **Branch:** feat/raster-brand · **Updated:** 2026-11-08

## STEPS

- [x] S1 — version 0.14.0 → 0.15.0 (first commit on the branch)
- [x] S2 — triage, D-046, D-047, AC-119..AC-122 claimed, this JOURNAL
- [x] S3 — failing tests: tokens and fonts (`tokens.test.ts`), the strip (`App.test.ts`), the mark (Toolbar, EmptyState)
- [x] S4 — `tokens.css` to stid values, `fonts.css` and the four woff2 files with OFL; MAP licence invariant names `gui/src/fonts/`
- [x] S5 — `Mark.svelte`; the strip in `App.svelte`; the mark in the title bar and the empty state
- [x] S6 — app icon: `icon-source.svg` on the macOS grid, regenerate the six bundled files with `tauri icon`
- [x] S7 — both gates, the user's live check, audit, reconcile, PR; deploy locally after merge

## DECISIONS (this work-item)

- stid tokens, peach, raster mark and strip; badges keep VIC-II → D-046
- fonts copied from Fontsource into `gui/src/fonts/`, not an npm dependency → D-047

## OPEN QUESTIONS

- none

## EDIT BOUNDARY (paths this work may touch)

- `gui/src/styles/`
- `gui/src/fonts/`
- `gui/src/main.ts`
- `gui/src/App.svelte`
- `gui/src/App.test.ts`
- `gui/src/lib/Mark.svelte`
- `gui/src/lib/Toolbar.svelte`
- `gui/src/lib/Toolbar.test.ts`
- `gui/src/lib/EmptyState.svelte`
- `gui/src/lib/EmptyState.test.ts`
- `gui/src/lib/Listing.svelte`
- `gui/src/lib/EntryMenu.svelte`
- `gui/src/lib/EditableText.svelte`
- `gui/src/lib/ErrorState.svelte`
- `gui/src-tauri/icons/`
- `gui/vite.config.ts`
- `gui/tsconfig.json`
- `docs/lcd/MAP.md`
- `docs/lcd/SPEC.md`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/triage-log.md`
- `docs/lcd/work/raster-brand/JOURNAL.md`
- `.claude/rules/lcd-conventions.md`
- `README.md`

<!-- /lcd-resume -->

---

## Acceptance criteria

**AC-119** (surfaces: RENDER): Given the window in either theme, then its surfaces, text and accent
are the stid system's: Catppuccin Mocha with the peach accent `#fab387` in dark, Latte with
`#fe640b` in light, text on the accent dark in both. The type badges keep their colours from D-045.

**AC-120** (surfaces: RENDER): Given the window, then its text is set in Geist and its figures,
paths and codes in Geist Mono. Both faces ship inside the app, and no stylesheet loads anything
from the network.

**AC-121** (surfaces: RENDER): Given the window in any state (empty, a cart, a refusal), then a
strip of five equal bands, red, peach, yellow, green and blue in the theme's Catppuccin shades,
runs under the title bar across the full width.

**AC-122** (surfaces: RENDER): Given the window, then the stidcart mark (the raster cartridge)
shows in the title bar whether or not a cart is open, and heads the empty state. The app icon
draws the same cartridge at every bundled size.

## LOG (append-only; not read on resume)

- 2026-11-08 — the user reviewed four directions on the canvas
  https://claude.ai/artifact/AQKoiqAq5WusphAAhWqXmu and picked "raster" (option A), with peach and
  bundled Geist. Triage: Standard, 4 signals. D-046, D-047. AC-119..AC-122 claimed, next free AC-123.
- 2026-11-08 — the app icon's pixels have no automated test: AC-122's icon clause is checked live
  (the Dock and Finder at each size), the in-app mark by `Toolbar.test.ts` and `EmptyState.test.ts`.
- 2026-11-08 — red, as expected: no `fonts.css`, no strip, no mark. Vitest hands a `?raw` CSS import
  back empty unless it processes CSS, so `vite.config.ts` processes `src/styles/*.css` in tests only.
  Green: vitest 54/0, svelte-check clean, GUI gate passing; the four woff2 files land in `dist/assets`.
- 2026-11-08 — S6: `icon-source.svg` draws the mark on the macOS grid (824 px tile on 1024).
  `tauri icon` regenerated the set; only the six files `tauri.conf.json` lists were kept. Looked at
  the 256 and 32 px PNGs: the label reads as bands at 256 and as a colour block at 32, as expected.
- 2026-11-08 — live check (dev app, dark appearance, cmixed.crt copy): the empty state and the
  listing render in Mocha with Geist and Geist Mono, the five-band strip under the title bar, the mark
  in the title bar and heading the empty state, the peach Open… button, badges in their D-045 colours.
  Light mode and the Dock icon are left for the user: switching the system appearance is theirs.
- 2026-11-08 — the user checked light mode and the Dock icon: "this is good".
- 2026-11-08 — audit: PASS (test-presence). `AC-119 (RENDER)` and `AC-120 (RENDER)` are in `gui/src/styles/tokens.test.ts`, `AC-121 (RENDER)` in `App.test.ts` and `tokens.test.ts`, `AC-122 (RENDER)` in `Toolbar.test.ts` and `EmptyState.test.ts`, green at ec3577c (root 230/0, GUI 37/0 cargo + 54/0 vitest). Closeout line appended.
- 2026-11-08 — reconciled: folded AC-119..AC-122 → SPEC.md/RENDER (two new rows).
