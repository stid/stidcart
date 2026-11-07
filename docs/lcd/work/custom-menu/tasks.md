# Tasks: Our own C64 menu, first slice

> **Deep lane · Phase 3 of 6 (Tasks).** A flat, dependency-ordered checklist. Each task names
> the file it touches. Tests come before the code they test (TDD). The granular STEPS in the
> work-item `JOURNAL.md` mirror this list as the resume view; keep them in sync.

**Slug:** `custom-menu`
**Spec:** [`spec.md`](./spec.md) • **Plan:** [`plan.md`](./plan.md) • **Journal:** [`JOURNAL.md`](./JOURNAL.md)
**Created:** 2026-11-07
**Status:** in_progress <!-- in_progress | completed | abandoned -->

---

## Conventions

- `[P]` suffix = parallelizable with the previous task (different file, no shared state).
- Tests precede implementation tasks they cover.
- One file per task where possible; tasks naming multiple files should be split unless the change is genuinely atomic.
- Every AC test carries the literal `AC-N (SURFACE)` token in its doc comment. AC-85 is proven
  three times: CLI in `menu_cli.rs`, RENDER in `gui/src-tauri/tests/menu.rs` (the DTO), and RENDER
  in `Listing.test.ts` (the window).
- Run the root gate before any commit touching the root workspace. Run `docs/lcd/ci/gui-gate.sh`
  before any commit touching `gui/`. Check `cargo --version` through the rustup shim first (D-010).
- A task that changes `menu/src/*.s` reruns `menu/build.sh menu/` and commits the regenerated
  `menu.bin` and `menu.lbl` with it, so AC-93 stays green at every commit.

## Tasks

### A. Groundwork: version, then the CI spike (plan: "task 1")

- [ ] **T1**: Bump the version 0.8.0 → 0.9.0 in the root `Cargo.toml`, `Cargo.lock`,
  `gui/src-tauri/Cargo.toml` and `gui/src-tauri/Cargo.lock`. This is atomic, one version across
  both workspaces. First check the base with `git show $(git merge-base HEAD main):Cargo.toml`.
  Regenerate any golden or fixture expectation that embeds the header name. Both gates must be
  green. Commit `chore: bump version to 0.9.0`.
- [ ] **T2**: Write the failing test `AC-94 (none): CI runs the boot checks instead of skipping` in
  `crates/stidcart-cli/tests/menu_boot.rs`. It asserts that `.github/workflows/ci.yml`'s `gate` job
  runs on `ubuntu-24.04`, runs `docs/lcd/ci/install-menu-tools.sh`, and sets
  `STIDCART_REQUIRE_TOOLS=1`. It also asserts that the harness panics on a missing tool when that
  variable is set.
- [ ] **T3**: Create `crates/stidcart-cli/tests/support/mod.rs` and
  `crates/stidcart-cli/tests/support/vice.rs`. Move `boot.rs`'s runner and monitor recipe into
  the harness, and add:
  - `cfg`-split `script` invocation (BSD `script -q /dev/null cmd…`, util-linux `script -q -c "…" /dev/null`);
  - `require_tool(name)`, which skips, or panics when `STIDCART_REQUIRE_TOOLS=1` (passes T2's harness half);
  - a monitor-script builder that does `ll`, `break exec` a label or address, key writes to
    `$0277`/`$C6` in ≤10-key chunks, and `bsave`;
  - a PETSCII → screen-code matcher that keeps bit 7 as "selected".
- [ ] **T4**: Make `crates/stidcart-cli/tests/boot.rs` use `support::vice`, with the AC-4
  assertions and negative control unchanged. AC-4 must still pass locally.
- [ ] **T5**: Create `docs/lcd/ci/install-menu-tools.sh`. It builds 64tass 1.60.3243 from the
  SourceForge source zip (sha256 committed in the script) and VICE 3.10 with
  `--enable-headlessui`, into a cache directory given as an argument, and then asserts both
  `--version`s. Take the build dependencies from the spike run, not from memory.
- [ ] **T6**: Edit `.github/workflows/ci.yml`. The `gate` job gets `runs-on: ubuntu-24.04`, an
  `actions/cache` step keyed on tool, version and image, the install step, the cache directory on
  `PATH`, and `STIDCART_REQUIRE_TOOLS=1` (passes T2). Push, then watch the `gate` check: AC-4 must
  pass on Linux, not skip. If headless VICE can't run the monitor recipe, stop and escalate (plan,
  Risks). **Checkpoint: no menu work starts before this is green in CI.**

### B. Format facts first (D-018)

- [ ] **T7**: Edit `docs/format/easyflash-cart-format.md` with a short section under "Bank 0
  reserved map" covering:
  - the menu loader's entry contract: the launcher's `jmp ($8000)` (normal) or `jmp ($A000)` (Ocean),
    16K mode at bank 0, KERNAL not initialised, the RUN/STOP, Q and C= kill check;
  - the `$DE02` mode values `$04` off, `$05` Ultimax, `$06` 8K, `$07` 16K;
  - the `$DF00/$DF01` xbank base-bank convention.

  Cite `easyloader_launcher.asm` and `easyloader.asm:80-85,1789`.
- [ ] **T8** [P]: Edit `docs/lcd/MAP.md`: add a `menu` zone row (GPL, our menu source and its
  committed image). Change the licence invariant to "`firmware/` is zlib, `menu/` is our own GPL
  menu, everything else GPL". Add `menu/build.sh` to the entry points. Bump "Last refined".

### C. The menu toolchain and its embedding (AC-93)

- [ ] **T9**: Write the failing test `AC-93 (none): the embedded menu is what its source assembles
  to, and fits` in `crates/stidcart-core/tests/menu.rs`. It always asserts that
  `menu::STIDCART_MENU.len() == 0x2000`. Through `require_tool("64tass")` it runs
  `menu/build.sh <tmp>` and compares the output with `menu/menu.bin` and `menu/menu.lbl` byte for
  byte. It also checks that a source over-filling the slot fails to assemble: copy the source to a
  temp dir, append a fill past `$a000`, and expect a non-zero exit. The core test can't share the
  CLI's `tests/support`, so it uses a local copy of the ten-line `require_tool`.
- [ ] **T10**: Create `menu/build.sh` with the recon flags (`-a -C -Wall -Werror -b`,
  `--vice-labels -l`, `--list`) and output dir as its argument.
- [ ] **T11**: Create `menu/src/menu.s` as a skeleton: `* = $8000`, the entry vector, `menu_idle`
  spinning, `.fill $a000-*, $ff`, `.cerror * > $a000`. Then create `menu/src/launch.s` as an empty
  include. Assemble both and commit `menu/menu.bin` and `menu/menu.lbl`.
- [ ] **T12**: Create `crates/stidcart-core/src/menu.rs` (`Menu`, `name`, `from_name`,
  `STIDCART_MENU` via `include_bytes!`) and register it in `crates/stidcart-core/src/lib.rs`
  (passes T9).

### D. Choosing, refusing and recognising the menu (CLI)

- [ ] **T13**: Write the failing test `AC-81 (CLI): build --menu stidcart puts our menu in the
  loader slot and changes nothing else` in `crates/stidcart-cli/tests/menu_cli.rs`. It covers
  programs plus non-Ocean carts, and compares against the same build without the flag: only
  `0x0000..0x2000` of the image differs, and that range equals `STIDCART_MENU`.
- [ ] **T14**: In `menu_cli.rs`, write the failing test `AC-82 (CLI): no request means EasyLoader and
  unchanged bytes; edit keeps either menu`. `build` without `--menu` equals the frozen fixtures
  (header held fixed), and `edit --rename` on a stidcart-menu cart keeps our menu.
- [ ] **T15**: In `menu_cli.rs`, write the failing test `AC-83 (CLI): edit switches the menu
  either way, and a round trip restores the original bytes`.
- [ ] **T16**: In `menu_cli.rs`, write the failing test `AC-84 (CLI): our menu with an Ocean entry
  is refused naming it, nothing written`. It covers three cases: build, edit `--menu` on
  `cocean.crt`, and edit `--add in_ocean.crt` on a stidcart-menu cart.
- [ ] **T17**: In `menu_cli.rs`, write the failing test `AC-95 (CLI): our menu with a program
  touching $D000-$DFFF is refused naming it, nothing written`. It covers build, edit `--menu`,
  edit `--add`, and edit `--replace`, plus a control: a program ending at `$CFFF` is accepted.
- [ ] **T18**: In `menu_cli.rs`, write the failing test `AC-85 (CLI): list says which menu`, with
  three cases: `menu: EasyLoader`, `menu: stidcart`, and `menu: unrecognised` for a mutated loader
  slot. The last case is still listed but refused by `edit`, as today.
- [ ] **T19**: Edit `crates/stidcart-core/src/reserved.rs`: `write_loader(img, mode, menu)`,
  `detect_menu(img, mode) -> Option<Menu>`, `verify_regions(..) -> Result<Menu, _>`, with its unit
  tests updated and one added for detection.
- [ ] **T20**: Edit `crates/stidcart-core/src/lib.rs`: the `CartBuilder.menu` field,
  `with_menu`/`set_menu`/`menu`, `BuildError::{MenuNeedsNormalMode, ProgramInIoArea}` with
  messages naming the entry, and the checks in `build_image` (AC-84, AC-95).
- [ ] **T21**: Edit `crates/stidcart-core/src/read.rs`: `CartInfo.menu` from `detect_menu` in
  `inspect`, and `load` setting the builder's menu from `verify_regions`.
- [ ] **T22**: Edit `crates/stidcart-core/src/listing.rs` so `Listing.menu` gets its display value,
  and update `crates/stidcart-core/tests/listing.rs` where it builds a `Listing` literally.
- [ ] **T23**: Edit `crates/stidcart-cli/src/lib.rs`: `--menu easyloader|stidcart` on `build` and
  `edit` (an unknown value lists both), the usage text, and the `menu: ` line in `render` (passes
  T13–T18).
- [ ] **T24**: Regenerate `crates/stidcart-cli/tests/golden/*` with the `menu:` line, and update
  `crates/stidcart-cli/tests/list.rs` and `list_golden.rs` expectations. Diff each golden: the only
  change is the added line.

### E. The window (AC-85 RENDER, AC-86)

- [ ] **T25**: Write the failing test `AC-85 (RENDER): the listing DTO carries the menu` in
  `gui/src-tauri/tests/menu.rs`, and extend the DTO contract in `gui/src-tauri/tests/contract.rs`.
- [ ] **T26**: In `gui/src-tauri/tests/menu.rs`, write the failing test `AC-86 (RENDER): a
  stidcart-menu cart opens editable, and saving after edits equals edit's bytes`. It uses the
  `stidcart_cli::run` oracle and must not touch `gui/src-tauri/src/document.rs` unless the test
  shows it has to.
- [ ] **T27** [P]: In `gui/src/lib/Listing.test.ts`, write the failing test `AC-85 (RENDER): the
  window shows which menu`.
- [ ] **T28**: Edit `gui/src-tauri/src/lib.rs` to add `ListingDto.menu` (passes T25, T26).
- [ ] **T29**: Add the `menu` field to `gui/src/lib/types.ts` and `gui/src/lib/fixtures/*`.
- [ ] **T30**: Edit `gui/src/lib/Listing.svelte` to show the menu beside "built by" (passes T27).
  `gui-gate.sh` must be green.

### F. The menu on the C64 (AC-87..AC-92)

- [ ] **T31**: Create `crates/stidcart-cli/tests/support/markers.rs`. It generates two marker PRGs
  (one at `$07F0` with a BASIC `SYS` stub at `$0801`, one at `$C000`) and seven marker carts (8K,
  16K, Ultimax 8, Ultimax 16, xbank 8, 16 and Ultimax) as generic or EasyFlash `.crt` bytes. Each
  writes a distinct screen-code string to `$0400` and spins at a fixed address. Give it unit tests
  that the carts parse through `Entry::from_input` as the intended kind.
- [ ] **T32**: In `menu_boot.rs`, write the failing test `AC-87 (none): the menu lists visible
  entries in order, hidden ones absent, first selected`.
- [ ] **T33**: In `menu_boot.rs`, write the failing test `AC-88 (none): a long list scrolls and the
  last entry starts`, using about 30 entries and keys injected in chunks.
- [ ] **T34**: In `menu_boot.rs`, write the failing test `AC-89 (none): programs start, RUN for
  ≤$0801 and SYS above, cart switched out`. It breaks on each marker's spin address, checks the
  marker string, and checks that `$8000` no longer reads as the menu.
- [ ] **T35**: In `menu_boot.rs`, write the failing test `AC-90 (none): every non-Ocean cartridge
  kind starts`, with one boot per marker cart.
- [ ] **T36**: In `menu_boot.rs`, write the failing test `AC-91 (none): RUN/STOP leaves to BASIC
  with full memory`. It breaks at BASIC's main loop and expects `38911 BASIC BYTES FREE` and
  `READY.` on screen.
- [ ] **T37**: In `menu_boot.rs`, write the failing test `AC-92 (none): no visible entries says so,
  and leaving still works`. Each boot test has a negative control, as AC-4 does.
- [ ] **T38**: Edit `menu/src/launch.s` to add the partial-reset stub (copy to `$DF02+`, cart off,
  IOINIT/RAMTAS/RESTOR/CINT, cart on, return).
- [ ] **T39**: Edit `menu/src/menu.s`: entry, partial reset, directory walk skipping hidden
  entries, title from `$BB08`, a scrolling window with the reverse-video selection, the "no
  entries" line, and a GETIN idle loop at `menu_idle` with CRSR↓/↑ (passes T32, T33's scrolling
  and T37's first half).
- [ ] **T40**: Edit `menu/src/launch.s` to add the leave stub (`$DE02=$04`, `jmp ($FFFC)`), and
  RUN/STOP in `menu.s` (passes T36, T37).
- [ ] **T41**: Edit `menu/src/launch.s` to add the PRG start: the CHRIN trap, the bank-by-bank copier
  for types `0x01`/`0x02`/`0x03`, the BASIC pointers, `$BA`, and `RUN`/`SYS` in the key buffer.
  RETURN on a program then works in `menu.s` (passes T34 and T33's last-entry start).
- [ ] **T42**: Edit `menu/src/launch.s` to add the cart start: CIA/VIC reset, the stub at `$DF02+`
  with the base bank in `$DF00/$DF01`, `$DE00`, `$DE02` by kind, `jmp ($FFFC)` (passes T35).

### G. Docs and close

- [ ] **T43**: Create `menu/README.md`: GPL notice, the entry contract (pointing at the format
  spec), keys, I/O2 RAM use, how to rebuild, and why `menu.bin` is committed.
- [ ] **T44** [P]: Edit `README.md` with `--menu` on `build`/`edit` and a pointer to
  `menu/README.md`.
- [ ] **T45** [P]: Edit `.claude/rules/lcd-conventions.md` to add the new test stems `menu` (core),
  `menu_cli` and `menu_boot` (cli) to the unique-stem list.
- [ ] **T-audit**: Run `/lcd:audit custom-menu`. The resulting `audit.md` must have zero
  MISSING/BLOCKED rows.
- [ ] **T-final**: Run `cargo fmt --all --check && cargo clippy --workspace --all-targets
  --all-features -- -D warnings && cargo test --workspace && docs/lcd/ci/gui-gate.sh`, all green
  locally with `STIDCART_REQUIRE_TOOLS=1`. Push and confirm `gate`, `gui` and `claude-review` are
  green on the PR.

## Checkpoint validation

After every block of related tasks, the suite must stay green. If T3 passes but T5's implementation breaks T1, stop and fix before moving on.

Hard checkpoints:
- **After T6**, AC-4 passes on Linux CI.
- **After T24**, the parity suite is untouched and green.
- **After T30**, `gui-gate.sh` is green.
- **After each of T39–T42**, every boot AC passes locally, and none that passed earlier has regressed.

## Linkbacks

- `AC-81 (CLI)` → T13, T12, T19, T20, T23
- `AC-82 (CLI)` → T14, T19, T21, T23
- `AC-83 (CLI)` → T15, T20, T21, T23
- `AC-84 (CLI)` → T16, T20, T23
- `AC-85 (CLI)` → T18, T19, T21, T22, T23, T24
- `AC-85 (RENDER)` → T25, T27, T28, T29, T30
- `AC-86 (RENDER)` → T26, T28
- `AC-87 (none)` → T32, T31, T38, T39
- `AC-88 (none)` → T33, T39, T41
- `AC-89 (none)` → T34, T31, T41
- `AC-90 (none)` → T35, T31, T42
- `AC-91 (none)` → T36, T40
- `AC-92 (none)` → T37, T39, T40
- `AC-93 (none)` → T9, T10, T11, T12
- `AC-94 (none)` → T2, T3, T5, T6
- `AC-95 (CLI)` → T17, T20, T23
- Supporting (no AC of its own): T1 versioning; T4 the shared harness; T7, T8, T43, T44, T45 docs
  required by the constitution check

---

<!--
Quality checks before starting implementation:
  - Every AC × non-`none`-surface row in plan.md has at least one test task above
  - Every new file in plan.md "File structure" has at least one task that creates it
  - Tests appear before their implementation
  - T-audit appears before T-final (the audit gate must pass before suite-final)
  - T-final exists (run the suite, lint, type check)
-->
