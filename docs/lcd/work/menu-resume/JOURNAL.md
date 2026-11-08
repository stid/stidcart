# JOURNAL — menu-resume

> **The resume anchor.** The block between the `lcd-resume:v1` markers below is the _entire_
> cold-start payload — everything a fresh session needs to continue, in <~40 lines. `/lcd:resume`
> reads MAP.md + this block + DECISIONS headers and nothing else. Keep NOW and STEPS current as
> you work (the build loop and `lcd:refine` update them as part of their normal edit/commit
> step). Everything below the `---` is history/detail, NOT read on resume.

<!-- lcd-resume:v1 -->

## NOW

- **Lane:** Standard
- **Goal:** After a soft reset our C64 menu comes back with the entry, order and kind filter it was left with, kept in `$DFF6-$DFFF` behind a magic, a cart check and a checksum (D-053)
- **Next action:** S6 — audit, reconcile into SPEC, closeout, PR (orchestrator)
- **Branch:** feat/menu-resume · **Updated:** 2026-11-08

## STEPS

- [x] S1 — version 0.19.0 → 0.20.0 (first commit on the branch)
- [x] S2 — triage, D-053, AC-147..AC-152 claimed, this JOURNAL
- [x] S3 — failing boot tests in `menu_boot.rs` for AC-147..AC-152, and the assembly cap in `menu_builds.rs`
- [x] S4 — implement in `menu/src/menu.s` and `launch.s` (cap `$DFF5`), rebuild `menu.bin`/`menu.lbl`, the replaced build's fingerprint joins `EARLIER_STIDCART_BUILDS`
- [x] S5 — `menu/README.md`, gates with `STIDCART_REQUIRE_TOOLS=1`
- [ ] S6 — audit, reconcile, closeout, PR

## DECISIONS (this work-item)

- state layout, when written, when restored, cap tightened → D-053

## OPEN QUESTIONS

- none

## EDIT BOUNDARY (paths this work may touch)

- `menu/src/menu.s`
- `menu/src/launch.s`
- `menu/menu.bin`
- `menu/menu.lbl`
- `menu/README.md`
- `crates/stidcart-core/src/menu.rs`
- `crates/stidcart-core/tests/menu_builds.rs`
- `crates/stidcart-cli/tests/menu_boot.rs`
- `crates/stidcart-cli/tests/support/`
- `Cargo.toml`
- `Cargo.lock`
- `gui/src-tauri/Cargo.toml`
- `gui/src-tauri/Cargo.lock`
- `docs/lcd/SPEC.md`
- `docs/lcd/DECISIONS.md`
- `docs/lcd/triage-log.md`
- `docs/lcd/work/menu-resume/JOURNAL.md`
- `.claude/rules/lcd-conventions.md`

<!-- /lcd-resume -->

---

## Acceptance criteria

**AC-147** (surfaces: none): Given an entry started from our menu (a program by RETURN, a cartridge by
RETURN, a program by SHIFT+RETURN), when the C64 is soft-reset, then the menu boots with that entry
selected and in view, the same order and kind filter as when it started, and no filter text.

**AC-148** (surfaces: none): Given RUN/STOP from the menu and then a soft reset, the menu boots with
the entry that was selected, its order and its kind filter.

**AC-149** (surfaces: none): Given a hard reset (the power-on pattern in `$DF00`), state bytes changed
after the menu wrote them, or state written for another cart (a different entry count, or a
different name in that slot), the menu boots as it does without state: first entry, directory
order, all kinds.

**AC-150** (surfaces: none): Given state whose kind filter would hide its entry, the menu boots with
that entry selected and the kind filter at all kinds.

**AC-151** (surfaces: none): Given any start, the launch routines copied to `$DF02` leave
`$DFF6-$DFFF` as the menu wrote them (assembly refuses routines past `$DFF5`), and AC-127/AC-128 hold
through restoring and saving the state.

**AC-152** (surfaces: none): Given the F7 help screen, then it names the joystick in port 2 (move,
page, fire) beside the keys.

## LOG (append-only; not read on resume)

- 2026-11-08 — Triage: Standard (2 signals: files incl. a binary rebuild, may span a session). Third
  item of the user's menu improvements. Recon (2026-11-08, VICE 3.10 verified): a soft reset
  (monitor `reset 0`) keeps the whole `$DF00` page; a hard reset (`reset 1`) refills it with VICE's
  power-on pattern (`ff ff 00 00 ff ff 00 00 ...`), which the checksum must reject. The page is
  visible and writable with the cart off (mode `$04`), so any program can write it, and EAPI uses
  `$DF80-$DFFF` (EasyFlash ProgRef §5.3). The boot path (launcher, RAMTAS, IOINIT, CINT) doesn't
  touch it. `reset` resumes execution, so a test sets the breakpoint first and sends no `g`:
  `Step::Raw("break exec .menu_idle\nreset 0\ndel")`; the existing `Step::Break` emits a `g`, which is
  wrong here. Free bytes are `$DFF6-$DFFF` (10); `.ram_end` is one past the copy. launch.s's
  `.cerror` cap (`> $fe`) must drop so the routines can't grow into the state. On real EF3 hardware
  whether the RESET/MENU button runs EF3's own menu first (which might use `$DF00`) is a
  hypothesis. The F7 help gains the joystick line menu-joystick left out (AC-152). AC-147..AC-152
  claimed, next free AC-153.
- 2026-11-08 — S3. Recon slip in the triage: `.ram_end` is `$DFF7`, so `p_only` sits at `$DFF6` and
  only 9 bytes are free; launch.s must shrink before the cap can drop (D-053 stands). Verified again
  in VICE: soft reset keeps the page, hard reset refills `$DFF7-$DFFF` with the power-on pattern,
  `break exec .entry` + `reset 0` stops before the menu reads anything. Tests: AC-147 (four starts
  incl. xbank), AC-148, AC-149 (power cycle, each of 10 bytes flipped then restored in one run,
  another cart by writing A's bytes at `entry` into B: one more entry / renamed slot, with A's own
  boot as control), AC-150 (beta.prg → beta.crt rebuild), AC-151 (labels, hand-off vs end dumps on
  five exits, AC-127 watch and AC-128 after a restoring boot; cap refusal in `menu_builds.rs`),
  AC-152. Red run: all 7 fail for the right reason — not restored ('p99 prog' not selected,
  'p14' not listed, 'beta' not selected), ram_end `$DFF7`, "help does not name the joystick".
- 2026-11-08 — S4. `leave` now branches into `cart_go`'s tail (`ldy #MODE_OFF`), 4 bytes less:
  routines `$DF02-$DFF2`, cap `.cerror ram_end > STATE`. Layout: `$DFF6-$DFF9` magic "STID" (four
  bytes, so all ten are checked), slot, order, kinds, count (the end marker's slot number, hidden
  entries included), name sum, checksum ($A5 + the nine). Saved in `_cart` and `start_prg` before
  `irq_off` and on RUN/STOP; with nothing listed RUN/STOP spoils the magic. Restore after `scan`,
  slot looked up in `visible` before its name is read. ROM `$8000-$93B0` → `$8000-$94D3` (2860
  free). AC-148's expected position was my slip (p14 is the 14th of 30, not the 15th), fixed.
  Gates: root 267 passed 0 failed with STIDCART_REQUIRE_TOOLS=1, gui-gate exit 0. Fingerprint of
  9f4ca36's build joins the earlier list.
- 2026-11-08 — S5. README: the resume behaviour, the `$DFF6-$DFFF` layout, the `$DFF5` cap, the
  stick on the F7 row, AC-147..AC-152 in the evidence line. Real-EF3 RESET keeping the page stays
  untested and says so.
