# JOURNAL — custom-menu

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the *entire*
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->
## NOW
- **Lane:** Deep
- **Goal:** Our own C64 menu, first slice: it lists a cart's entries and boots them, is chosen opt-in at build/edit, and load/edit/GUI recognise it (D-037)
- **Next action:** done; PR #36 green (gate, gui, audit, claude-review) and awaits the user's review and merge
- **Branch:** feat/custom-menu  ·  **Updated:** 2026-11-07

## STEPS
> Mirrors `tasks.md` T1..T45 by block, and that file is the detail.
- [x] S0 — triage, spec, plan, tasks (D-037, D-038)
- [x] S1 — A: T1 version 0.9.0 · T2–T6 CI spike (harness, install script, ci.yml); the Linux confirmation is S8
- [x] S2 — B: T7 format spec facts · T8 MAP
- [x] S3 — C: T9–T12 toolchain, skeleton menu, embedding (AC-93)
- [x] S4 — D: T13–T24 `--menu`, refusals, recognition, `list` line, goldens (AC-81..85 CLI, AC-95)
- [x] S5 — E: T25–T30 GUI DTO, window, save parity (AC-85 RENDER, AC-86)
- [x] S6 — F: T31–T42 markers, boot tests, the 6502 menu (AC-87..92)
- [x] S7 — G: T43–T45 docs
- [x] S8 — Linux CI green · T-audit · T-final

## DECISIONS (this work-item)
- Scope forks (Ocean refused, GUI recognise-only, VICE in CI, no hardware gate) → D-037
- Menu as builder property, GETIN input, EasyLoader-order PRG start, $D000 refusal, CI tool build → D-038

## OPEN QUESTIONS
- none blocking; headless VICE's build deps and `script` behaviour come from the CI spike

## EDIT BOUNDARY (paths this work may touch)
- `menu/`
- `crates/stidcart-core/src/menu.rs`
- `crates/stidcart-core/src/lib.rs`
- `crates/stidcart-core/src/reserved.rs`
- `crates/stidcart-core/src/read.rs`
- `crates/stidcart-core/src/listing.rs`
- `crates/stidcart-core/tests/menu.rs`
- `crates/stidcart-core/tests/listing.rs`
- `crates/stidcart-cli/src/lib.rs`
- `crates/stidcart-cli/tests/menu_cli.rs`
- `crates/stidcart-cli/tests/menu_boot.rs`
- `crates/stidcart-cli/tests/boot.rs`
- `crates/stidcart-cli/tests/list.rs`
- `crates/stidcart-cli/tests/list_golden.rs`
- `crates/stidcart-cli/tests/support/`
- `crates/stidcart-cli/tests/golden/`
- `gui/src-tauri/src/lib.rs`
- `gui/src-tauri/src/document.rs`
- `gui/src-tauri/tests/menu.rs`
- `gui/src-tauri/tests/contract.rs`
- `gui/src/lib/types.ts`
- `gui/src/lib/Listing.svelte`
- `gui/src/lib/Listing.test.ts`
- `gui/src/lib/fixtures/`
- `Cargo.toml`
- `Cargo.lock`
- `gui/src-tauri/Cargo.toml`
- `gui/src-tauri/Cargo.lock`
- `.github/workflows/ci.yml`
- `docs/lcd/ci/install-menu-tools.sh`
- `docs/format/easyflash-cart-format.md`
- `docs/lcd/MAP.md`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/work/custom-menu/`
- `.claude/rules/lcd-conventions.md`
- `README.md`
<!-- /lcd-resume -->

---

## Acceptance criteria

AC-81..AC-95 live in `spec.md`.

## LOG (append-only; not read on resume)

- 2026-11-07 — Triaged Deep (5 signals, hard trigger: new module + assembler dependency + produced
  bytes). User picked opt-in, GPL `menu/`, 64tass, and all four goals split into slices → D-037.
  Scope forks for this slice: Ocean refused, GUI recognise/show only, VICE installed in CI, VICE
  is enough without a hardware check. Spec written: AC-81..AC-94.
- 2026-11-07 — Recon digest (web and repo; Context7 not reachable from the agent):
  - 64tass 1.60.3243: `-a -C -Wall -Werror -b --list -l --vice-labels`, `* = $8000` with the entry
    vector first, then `.fill $a000-*, $ff` plus a `.cerror * > $a000` guard so the image is exactly 8 KiB.
    Avoid `-f` and the default cbm-prg output.
  - CI pin: build 64tass from the SourceForge source zip with a committed sha256 and assert
    `--version`. Apt varies (1.59 on 24.04, 1.60 on 26.04, and `ubuntu-latest` is migrating this
    month). Homebrew can't pin.
  - PRG boot: copier and kill code in I/O2 RAM `$DF00-$DFFF`. Copy bank by bank from `$8000-$BFFF`
    with the cart on, then run the KERNAL init with the cart off (`$DE02=$04`). Set the BASIC
    pointers to the end address, `$BA=8`, and put `RUN\r` or `SYS nnnn\r` in `$0277`/`$C6`. A PRG
    overlapping `$D000-$DFFF` can't use a `$DF00` copier as is.
  - Cart boot: from `$DF00`, write the bank to `$DE00` and the mode to `$DE02` (`$05` Ultimax,
    `$06` 8K, `$07` 16K), then `jmp ($FFFC)`. xbank's base bank goes in `$DF00`/`$DF01`
    (EasyLoader's convention, `easyloader.asm:1789`, not a published spec).
  - VICE key injection (`keybuf`, `-keybuf`, binary monitor) only fills `$0277`/`$C6`. Nothing
    presses keys in the CIA matrix. **So the menu must read keys from the KERNAL buffer**, or merge
    the buffer with its own scan. EasyLoader's direct `$DC01` scan could never be driven by a test.
  - VICE 3.10 (2025-12) is current and fixed checkpoint triggering. No EasyFlash changes since 3.8.
    Tests can `ll` the labels and `break .label` instead of boot.rs's ignore-count.
  - `boot.rs` (AC-4) matches EasyLoader's screen codes. It stays valid because EasyLoader stays the
    default, but the new boot tests need their own matcher.
- 2026-11-07 — Spec approved by the user. Plan written → D-038. Reading EasyLoader's actual PRG
  start (`easyloader.asm:2091-2260`) corrected the recon summary's order: the program is copied
  only *after* a BASIC cold start, through a CHRIN trap, not before the KERNAL init. Spec amended
  with AC-95 (programs touching `$D000-$DFFF` refused with our menu), called out to the user. CI:
  the user chose to build VICE 3.10 headless from source, cached, over apt or a macOS job.
- 2026-11-07 — test-gen: 16 AC tokens across `core/tests/menu.rs` (AC-93), `cli/tests/menu_cli.rs`
  (AC-81..85, AC-95 CLI), `cli/tests/menu_boot.rs` (AC-87..92, AC-94), `gui/src-tauri/tests/menu.rs`
  (AC-85, AC-86 RENDER) and `Listing.test.ts` (AC-85 RENDER). The root targets are compile-red on the
  missing `stidcart_core::menu` (T12) and `tests/support` (T3, T31), whose API the boot tests fix:
  `vice::{boot, Step, Target, key, screen_text, find, is_reversed, menu_labels, require_tool[_in]}`
  and `markers::{basic_prg, high_prg, cart, CartKind, Marker{bytes, spin, text}}`. The GUI ones
  are runtime-red. The EDIT BOUNDARY was rewritten one path per bullet, because the hook couldn't
  parse brace globs.
- 2026-11-07 — red-green, in task order with three refinements: markers (T31) came forward with the
  harness, because `menu_boot` declares `mod support`. Block C (skeleton menu + embedding) went
  before the CI push, because CI's clippy compiles every test target. VICE can't attach a stand-alone
  xbank `.crt` (type 33 "NO CART EXISTS"), so the marker control boots only the 4 generic kinds.
- 2026-11-07 — Linux CI spike: 64tass and headless VICE 3.10 built in about 1.5 min. AC-4 then failed
  twice in 0.03s. The headless build has no `+saveres`, `-nativemonitor` or `+confirmonexit` (VICE
  registers them only in `arch/gtk3`/`arch/sdl`), so the harness probes `-help` and passes them only
  when present. It also logs x64sc's output and prints the tail when a dump is missing.
- 2026-11-07 — Menu bugs found in the monitor: (1) the RAM block came to 376 bytes, over the 254 in
  `$DF02-$DFFF`, so all cart-independent work moved to ROM (CIA/VIC quieting, CHRIN patch, size-2,
  BASIC pointers and keys after the copy); it is now `$DF02-$DFF5`. (2) A PRG ran `RUN` on an empty
  line, because the trap returned with X=4 and BASIC's INLIN keeps its buffer index in X across
  CHRIN. The trap now keeps A/X/Y, as EasyLoader's does.
- 2026-11-07 — red-green: all 16 AC tokens green locally (root 209 tests with STIDCART_REQUIRE_TOOLS=1,
  GUI gate green). Took about 9 fix iterations: 2 CI option rounds, 2 menu bugs, plus test-only
  compile and lint fixes, with 0 reverts.
- 2026-11-07 — Linux CI green on 4d77bc2 (gate, gui, audit, claude-review). AC-4 and AC-87..AC-94 ran,
  none skipped. audit: PASS (first run). Closeout line appended.
- 2026-11-07 — reconciled: folded AC-81..AC-95 → SPEC.md. CLI: build, edit, list and the edit-refuses
  row rewritten, plus 1 new refusal row. RENDER: 3 rows rewritten. Internal: 2 rewritten, plus 3 new
  rows. D-005 status notes the menu.

## DEEP PIPELINE (Deep lane only)

> Phase tracker for the Deep-lane pipeline. Files live beside this JOURNAL.

`spec.md ✅  plan.md ✅  tasks.md ✅  tests ✅  red-green ✅  audit.md ✅`
<!-- mark ✅ done · ⏳ in progress · ⬜ not started -->
