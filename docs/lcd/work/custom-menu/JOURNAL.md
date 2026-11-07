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
- **Next action:** user approves spec.md, then run /lcd:plan custom-menu (fold the recon digest from LOG into the plan)
- **Branch:** feat/custom-menu  ·  **Updated:** 2026-11-07

## STEPS
- [x] S1 — triage (Deep), D-037, spec.md
- [ ] S2 — /lcd:plan custom-menu  ← next
- [ ] S3 — /lcd:tasks, /lcd:test-gen, red-green, audit

## DECISIONS (this work-item)
- Scope forks (Ocean refused, GUI recognise-only, VICE in CI, no hardware gate) → D-037

## OPEN QUESTIONS
- none blocking; key bindings and the boot-check key injection are left for plan.md

## EDIT BOUNDARY (paths this work may touch)
- `menu/` (new), `crates/stidcart-core/`, `crates/stidcart-cli/`, `gui/`, `.github/workflows/ci.yml`, `docs/`, `README.md`
<!-- /lcd-resume -->

---

## Acceptance criteria

AC-81..AC-94 live in `spec.md`.

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

## DEEP PIPELINE (Deep lane only)

> Phase tracker for the Deep-lane pipeline. Files live beside this JOURNAL.

`spec.md ✅  plan.md ⬜  tasks.md ⬜  tests ⬜  red-green ⬜  audit.md ⬜`
<!-- mark ✅ done · ⏳ in progress · ⬜ not started -->
