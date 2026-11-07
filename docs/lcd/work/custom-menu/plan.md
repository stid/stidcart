# Plan: Our own C64 menu, first slice

> **Deep lane · Phase 2 of 6 (Plan).** This document owns **how**. Specify ≠ Plan: if you're
> about to write a user story or acceptance criterion, that belongs in `spec.md`.

**Slug:** `custom-menu`
**Spec:** [`spec.md`](./spec.md) · **Journal:** [`JOURNAL.md`](./JOURNAL.md)
**Created:** 2026-11-07
**Status:** approved <!-- draft | approved | implemented | superseded -->

---

## Goal

Ship a GPL menu of our own (64tass, `menu/`) that lists a cart's visible entries and starts them,
selectable with `--menu stidcart` on `build`/`edit`, recognised and preserved by `load`, `list` and
the GUI, and proven by VICE boots that run in CI (AC-81..AC-95).

## Architecture

**The menu is a property of the cart, like the mode.** `CartBuilder` gains a `menu: Menu` field
(`EasyLoader` by default, or `Stidcart`), and `reserved::write_loader` writes the chosen loader into
the 8 KiB slot at bank 0 ROML. The launcher, EAPI, boot region and name stay as they are, so AC-81's
"every other byte is the same" holds by construction. On the read side, `verify_regions` stops
comparing against one expected loader and reports *which* menu the loader region holds. EAPI, boot
and magic stay strict, and a loader region matching neither menu is today's `ReservedRegionMismatch`.
`load` sets the builder's menu from that, which is what makes `edit` and the GUI preserve it (AC-82,
AC-86) with no further code. `inspect` reports the same detection as `Option<Menu>` without refusing
anything (D-011). `listing::describe` turns it into a `menu` value, which both renderers show (AC-85).
Our menu has no Ocean build. `build_image` refuses `Stidcart` together with an Ocean entry (AC-84),
and refuses `Stidcart` together with a program whose loaded extent touches `$D000-$DFFF` (AC-95,
see Risks).

**The menu itself** is about 1–2 KiB of 6502 in `menu/src/`, assembled by `menu/build.sh` (pinned
flags from recon: `64tass -a -C -Wall -Werror -b`, `* = $8000`, entry vector first, `.fill` to `$a000`
with a `.cerror` guard) into the committed `menu/menu.bin` (exactly `0x2000` bytes) and
`menu/menu.lbl` (VICE labels). The core embeds `menu.bin` with `include_bytes!`, so `cargo` never
needs the assembler. A test reassembles it and compares byte for byte (AC-93). The runtime has
these parts:
- **Entry.** The launcher has done `jmp ($8000)` in 16K mode at bank 0 with the KERNAL not
  initialised. Copy a stub to I/O2 RAM, then from there switch the cart off, run
  IOINIT/RAMTAS/RESTOR/CINT so the top of memory is right, switch the cart back on, and return.
  This is EasyLoader's partial reset, `easyloader.asm:2093`.
- **Listing.** Walk the EasyFS2 directory at `$A000` (bank 0 ROMH), skip hidden slots, and print
  through KERNAL CHROUT: the cart name from `$BB08` as the title, then a scrolling window of rows,
  with the selected row in reverse. With no visible entries, print a fixed "no entries" line (AC-92).
- **Input.** The menu reads keys only through KERNAL GETIN, i.e. the `$0277`/`$C6` buffer the IRQ
  fills. Recon found that VICE can inject keys only there, and EasyLoader's direct `$DC01` scan
  could never be driven by a test. The idle loop has a `menu_idle` label, reached only when the
  buffer is empty. CRSR↓/CRSR↑ move, RETURN starts, RUN/STOP leaves.
- **Leaving (AC-91).** From I/O2 RAM, write `$04` to `$DE02` (cart off) and `jmp ($FFFC)`. The
  KERNAL then does a clean cold start into BASIC.
- **Starting a program (AC-89).** This is EasyLoader's order (`easyloader.asm:2091-2260`), which
  copies only after BASIC is fully up, so nothing the init touches can clobber the program. Put the
  copier and the entry's bank/offset/size/destination in I/O2 RAM. Patch CHRIN (`$0324`) to a trap
  there. Switch the cart off and `jmp $FCFE` (CLI, then BASIC cold start). BASIC's first CHRIN hits
  the trap, which restores the vector, switches the cart on in 16K, and copies bank by bank from
  `$8000-$BFFF` (ROML/ROMH interleave for type `0x01`, one half for `0x02`/`0x03`). It then sets
  `$2B`=`$0801` and `$2D/$2F/$31/$AE` to the end, switches the cart off, sets `$BA=8`, puts `RUN⏎`
  (load address ≤ `$0801`) or `SYS nnnnn⏎` in the key buffer, and jumps through the real CHRIN.
- **Starting a cartridge (AC-90).** Reset CIA/VIC as a physical reset would. Copy a stub to
  `$DF02+` with the entry's base bank in `$DF00/$DF01` (the xbank convention,
  `easyloader.asm:1789`). Write the bank to `$DE00` and the mode to `$DE02` (`$06` 8K, `$07` 16K,
  `$05` Ultimax, and the xbank kinds by their mode), then `jmp ($FFFC)`.

**Evidence is VICE (decaying oracle).** A shared harness, `tests/support/vice.rs`, is extracted from
`boot.rs`. It runs `x64sc` with a monitor script and handles util-linux `script -c` versus BSD
`script` by `cfg`. It can load `menu.lbl`, `break exec` a label or address, write key codes straight
to `$0277`/`$C6` (in chunks of ≤10, re-hitting `menu_idle` between them), and `bsave` screen RAM or
any range. `tests/support/markers.rs` generates the inputs in memory: marker PRGs (at `$07F0` with a
BASIC `SYS` stub at `$0801`, and at `$C000`) and marker carts of all seven non-Ocean kinds. Each
writes a known screen-code string to `$0400` and spins at a fixed address the test breaks on, so
no test counts instructions. The generic CRT packets are written in the helper, because the core
only writes EasyFlash containers. The matcher maps PETSCII to the ROM charset's screen codes, keeps
bit 7 for "selected", and has a negative control as AC-4 does.

**CI.** The `gate` job pins `runs-on: ubuntu-24.04`. `ubuntu-latest` moves to 26.04 this month
(recon), and a cached binary must not cross images. It builds VICE 3.10 with `--enable-headlessui`
and 64tass 1.60.3243 from its SourceForge source zip (sha256 committed), both through
`docs/lcd/ci/install-menu-tools.sh`, and caches each under a key of tool, version and image. It
sets `STIDCART_REQUIRE_TOOLS=1`, which turns the harness's "skip when absent" into a failure (AC-94).
Task 1 is a spike: get the *existing* AC-4 boot test green on Linux with this setup before any
menu work. Otherwise switching the requirement on would turn every PR red. The exact build
dependencies come from the spike, not from this plan.

Rejected, one line each (more in Risks): a separate `menu` CI job (it would need a branch-protection
change for no gain); an Ocean build of the menu (out of scope, AC-84); the menu blob as a
`firmware/` `.prg` (wrong licence side, D-037); keys read from the CIA matrix (can't be tested,
recon); copying the program before the BASIC cold start (RAMTAS clears pages 0/2/3, and a
cart-on init would set the top of memory to `$8000`).

## Constitution check

| Rule | Applies? | Compliance note |
|---|---|---|
| `testing.md` (tests first, coverage) | yes | test-gen emits AC-81..AC-95 as failing tests before any code. The 6502 is driven by the VICE boot tests, so a menu routine has no "done" until its boot AC flips |
| `no-downgrade.md` | yes | EasyLoader stays the default, and every existing build/edit/list option is kept. `list` gains a `menu:` line and loses nothing. `verify_regions` keeps EAPI/boot/magic strictness and only widens the loader check to a second known blob |
| `no-overengineering.md` | yes | Two menus as an enum, no plugin mechanism. No GUI chooser (spec), no Ocean build, no joystick. The menu is a text listing via KERNAL CHROUT, with no custom font or screen code. The VICE helper is extracted because `boot.rs` and `menu_boot.rs` both need it, which is the second real use, and the marker generator is single-purpose test code |
| `refinement-protocol.md` | yes | the spec is amended once, adding AC-95 and an out-of-scope line, and the change is called out to the user. Further drift goes through `lcd:refine` |
| `versioning.md` | yes | `feat/` → minor. The branch's first *code* commit bumps 0.8.0 → 0.9.0 in the root `Cargo.toml` **and** `gui/src-tauri/Cargo.toml` (not on the workspace version), plus both lockfiles. The two docs commits already on the branch carry no version |
| `commits.md` | yes | one commit per red→green flip. Push only when asked (the repo's CLAUDE.md asks for branch + PR, and the user's global rule is to push after commit) |
| `ac-convention.md` EVAL | no | the output is bytes and screen contents, asserted exactly. Nothing is scored (MAP: EVAL n/a) |
| MAP invariant: byte parity gates the write path | yes | builds without `--menu` are unchanged and the parity suite still passes untouched (AC-82). The version bump changes the default header name as every bump does, and the parity tests already fix the header |
| MAP invariant: the oracle decays | yes | carts with our menu have no `ndeft` oracle, so AC-87..AC-92 are VICE boots and AC-93 is a reassembly check. Nothing claims parity for them |
| MAP invariant: `firmware/` zlib, rest GPL | yes | the menu source and blob live in GPL `menu/`, and `firmware/` is untouched. MAP gets a `menu` zone, and the invariant line gains "`menu/` is our own GPL menu" |
| MAP invariant: no global state / I/O policy in core | yes | `Menu` is a value on the builder. The core embeds bytes and never runs the assembler or VICE |
| MAP invariant: crate version reaches produced bytes | yes | the bump lands in the first code commit, so fixtures or goldens that embed the header name are regenerated there, and only there |
| MAP invariant: format facts from `docs/format/` (D-018) | yes | the launcher's entry contract (`jmp ($8000)`/`($A000)`, 16K, bank 0, KERNAL uninitialised), the `$DE02` mode values, and the `$DF00/$DF01` xbank base-bank convention go into the format spec in the same PR, before the code that relies on them |
| MAP invariant: describing vs rebuilding (D-011) | yes | `inspect` reports the menu as `Option<Menu>` and refuses nothing new. Only `load` refuses an unrecognised loader, as today |
| MAP invariant: per-behaviour divergence (D-007) | no | no read-path quirk is reproduced or fixed here |
| `.claude/rules/lcd-conventions.md` AC numbering | yes | AC-81..AC-95 claimed, next free bumped to AC-96 |
| CLAUDE.md: never commit to `main`, gate before commit, GUI gate | yes | all work on `feat/custom-menu`. Both gates run before every commit that touches their workspace |

## Cross-path behavior matrix

| AC | Surface | Path |
|---|---|---|
| `AC-81` | CLI | `crates/stidcart-cli/src/lib.rs:--menu` |
| `AC-82` | CLI | `crates/stidcart-cli/src/lib.rs:--menu` |
| `AC-83` | CLI | `crates/stidcart-cli/src/lib.rs:--menu` |
| `AC-84` | CLI | `crates/stidcart-cli/src/lib.rs:--menu` |
| `AC-85` | CLI | `crates/stidcart-cli/src/lib.rs:menu: ` |
| `AC-85` | RENDER | `gui/src/lib/Listing.svelte` |
| `AC-86` | RENDER | `gui/src-tauri/src/document.rs` |
| `AC-95` | CLI | `crates/stidcart-cli/src/lib.rs:--menu` |

The behaviour that must agree across surfaces: AC-85's menu value comes from `listing::Listing.menu`
for both renderers (D-032). The CLI never computes it, and neither does the GUI. AC-86's saved bytes
must equal `edit`'s, using the CLI-as-library oracle the GUI parity tests already use (D-035).

**Test scope:** `crates/stidcart-core/tests/menu.rs`, `crates/stidcart-cli/tests/menu_cli.rs`, `crates/stidcart-cli/tests/menu_boot.rs`, `gui/src-tauri/tests/menu.rs`, `gui/src/lib/Listing.test.ts`

## Reused primitives

| Existing primitive | Path | Used for |
|---|---|---|
| `reserved::{write_loader, verify_regions, Mode, LOADER_*}` | `crates/stidcart-core/src/reserved.rs` | where the loader goes, and the region check widened to report a `Menu` |
| `CartBuilder::build_image` + `BuildError` | `crates/stidcart-core/src/lib.rs` | the two new refusals beside `SecondOcean`, with the same shape and naming |
| `read::{inspect, load}`, `CartInfo` | `crates/stidcart-core/src/read.rs` | menu detection on both read paths, with the D-011 split kept |
| `listing::describe` / `Listing` | `crates/stidcart-core/src/listing.rs` | the one place the `menu` value is computed for both renderers |
| `render()` + `list_golden` goldens | `crates/stidcart-cli/src/lib.rs`, `crates/stidcart-cli/tests/golden/` | the `menu:` line, asserted on the string |
| CLI option parsing in `build`/`edit` | `crates/stidcart-cli/src/lib.rs` | `--menu` added next to `-e`/`--header` |
| `boot.rs` monitor recipe (`script`, `-default +saveres +easyflashcrtwrite`, `bsave`) | `crates/stidcart-cli/tests/boot.rs` | moved into `tests/support/vice.rs` and reused by AC-4 and the menu boots |
| `ListingDto`, `Document`, GUI parity tests against `stidcart_cli::run` | `gui/src-tauri/src/lib.rs`, `document.rs`, `tests/parity.rs` | AC-85 RENDER field and the AC-86 save-equals-edit check |
| fixtures `hello.prg`, `two.crt`, `c8k.crt` … | `fixtures/` | AC-82 parity inputs and AC-83 round-trip subjects |
| `petscii::ascii_to_petscii` | `crates/stidcart-core/src/petscii.rs` | the screen matcher derives expected screen codes from the stored PETSCII names |
| EasyLoader's boot technique (read, not copied) | `~/Documents/develop/ndefpack/src/fw/easyloader.asm:1789,2091-2260` | the sequence for partial reset, CHRIN trap and cart launch. Our code is written fresh (GPL), with the technique cited |

## Data model / Contracts

```rust
// crates/stidcart-core/src/menu.rs
pub enum Menu { EasyLoader, Stidcart }            // Default = EasyLoader
impl Menu {
    pub fn name(self) -> &'static str;           // "EasyLoader" | "stidcart"  (also the --menu values, lowercased)
    pub fn from_name(s: &str) -> Option<Menu>;   // "easyloader" | "stidcart"
}
pub const STIDCART_MENU: &[u8] = include_bytes!("../../../menu/menu.bin"); // 0x2000 bytes, raw

// CartBuilder: with_menu(Menu) / set_menu(Menu) / menu() -> Menu
// BuildError::MenuNeedsNormalMode { name }   // our menu + an Ocean entry (AC-84)
// BuildError::ProgramInIoArea { name }       // our menu + a PRG touching $D000-$DFFF (AC-95)
// reserved::write_loader(img, mode, menu)
// reserved::verify_regions(img, mode) -> Result<Menu, ReadError>
// reserved::detect_menu(img, mode) -> Option<Menu>    // used by inspect, never refuses
// CartInfo.menu: Option<Menu>;  Listing.menu: String  // "EasyLoader" | "stidcart" | "unrecognised"
```

- CLI: `build --menu easyloader|stidcart`, `edit --menu easyloader|stidcart`. An unknown value is
  an error listing the two. `list` prints `menu: <value>` after `built by:`.
- GUI: `ListingDto.menu` / `types.ts Listing.menu`. `Listing.svelte` shows it beside "built by".
- Env: `STIDCART_REQUIRE_TOOLS=1`. A missing `x64sc` or `64tass` fails the test instead of skipping.
- Menu ABI (recorded in `menu/README.md` and the format spec): entry vector at `$8000`, label
  `menu_idle`, I/O2 RAM `$DF00-$DFFF` owned by the menu once it is entered.

## File structure

### New files

```
menu/README.md                              GPL notice, entry contract, keys, how to rebuild, I/O2 RAM use
menu/build.sh                               the pinned 64tass invocation; writes menu.bin + menu.lbl to a given dir
menu/src/menu.s                             the menu: entry, init, directory walk, listing, input loop
menu/src/launch.s                           the I/O2-RAM stubs: partial reset, leave, PRG trap+copier, cart launch
menu/menu.bin                               committed assembled image, exactly 0x2000 bytes
menu/menu.lbl                               committed VICE label file (tests break on menu_idle)
crates/stidcart-core/src/menu.rs            Menu enum, the embedded blob, names
crates/stidcart-core/tests/menu.rs          AC-81 image-level diff, AC-82 core parity, AC-93 size + reassembly
crates/stidcart-cli/tests/menu_cli.rs       AC-81..AC-85 (CLI), AC-95 through the binary
crates/stidcart-cli/tests/menu_boot.rs      AC-87..AC-92 VICE boots, AC-94 workflow check
crates/stidcart-cli/tests/support/mod.rs    shared test support (declares vice + markers)
crates/stidcart-cli/tests/support/vice.rs   x64sc runner, monitor script builder, key injection, screen matcher, require-tools switch
crates/stidcart-cli/tests/support/markers.rs  marker PRGs and the seven non-Ocean marker carts, generated in memory
gui/src-tauri/tests/menu.rs                 AC-85 (RENDER) DTO carries the menu; AC-86 save after edits == CLI edit
docs/lcd/ci/install-menu-tools.sh           builds 64tass 1.60.3243 (sha256-checked) and VICE 3.10 headless into a cache dir
```

### Modified files

- `Cargo.toml`, `Cargo.lock`, `gui/src-tauri/Cargo.toml`, `gui/src-tauri/Cargo.lock`: 0.8.0 → 0.9.0 in the first code commit.
- `crates/stidcart-core/src/lib.rs`: `mod menu`, the `CartBuilder.menu` field and accessors, the two `BuildError` variants, `build_image` passing the menu.
- `crates/stidcart-core/src/reserved.rs`: `write_loader` takes the menu, `verify_regions` returns it, `detect_menu`, unit tests updated.
- `crates/stidcart-core/src/read.rs`: `CartInfo.menu`, and `load` sets the builder menu from verification.
- `crates/stidcart-core/src/listing.rs`: `Listing.menu`.
- `crates/stidcart-cli/src/lib.rs`: `--menu` on build/edit, usage text, the `menu:` line in `render`.
- `crates/stidcart-cli/tests/boot.rs`: uses `support::vice`. The AC-4 assertions are unchanged.
- `crates/stidcart-cli/tests/golden/*`: every `list` golden gains the `menu:` line.
- `crates/stidcart-cli/tests/list.rs`, `list_golden.rs`, `crates/stidcart-core/tests/listing.rs`: expectations that build a `Listing` literally gain `menu`.
- `gui/src-tauri/src/lib.rs`: `ListingDto.menu`.
- `gui/src-tauri/tests/contract.rs`: the DTO contract includes `menu`.
- `gui/src/lib/types.ts`, `gui/src/lib/Listing.svelte`, `gui/src/lib/Listing.test.ts`, `gui/src/lib/fixtures/*`: the menu field shown and tested.
- `.github/workflows/ci.yml`: `gate` on `ubuntu-24.04`, tool cache plus install step, `STIDCART_REQUIRE_TOOLS=1`.
- `docs/format/easyflash-cart-format.md`: the loader entry contract, `$DE02` modes, the `$DF00/$DF01` xbank convention.
- `docs/lcd/MAP.md`: the `menu` zone, the licence invariant line, entry points (`menu/build.sh`).
- `docs/lcd/DECISIONS.md`: D-038.
- `docs/lcd/work/custom-menu/spec.md`: AC-95 and an out-of-scope line (amendment, called out).
- `.claude/rules/lcd-conventions.md`: next free AC-96, and the new test stems `menu`, `menu_cli`, `menu_boot`.
- `README.md`: `--menu` and a pointer to `menu/README.md`.
- `docs/lcd/SPEC.md`: only at closeout (reconcile).

## Risks & rejected alternatives

- **The CI tool build is the riskiest unknown** (headless VICE dependencies, `script` under
  util-linux, ROMs bundled with a source build). That is why it is task 1, proven on the existing
  AC-4 test before menu code exists. If headless VICE won't run the monitor recipe, escalate to the
  user rather than fall back to skipping, which AC-94 forbids.
- **Programs touching `$D000-$DFFF` are refused at build time (AC-95), not handled by the menu.**
  The copier runs from `$DF00` and writes with I/O mapped in, so such a program would overwrite I/O
  and the copier itself. The tool knows every program's load address and size, so it can refuse
  with a named error. A menu that silently ignored the pick would be untestable from the CLI and
  confusing at the C64. Banking I/O out during the copy also hides `$DE00/$DF00` (recon), so a
  menu-side fix would need a second copier in low RAM. That is a later item if real programs need it.
- **Copying before the BASIC cold start:** rejected. RAMTAS clears pages 0/2/3 (it would destroy a
  program loading below `$0801`), and an init with the cart on sets the top of memory to `$8000`.
- **Reading keys from the CIA matrix like EasyLoader:** rejected. VICE injects only into the KERNAL
  buffer, so no boot test could drive the menu. GETIN also reads real keys on hardware.
- **Committing a `.prg` into `firmware/`:** rejected. That is the zlib side, and its README says
  nothing there is compiled (D-037).
- **apt's `vice`/`64tass` in CI:** rejected. The versions follow the runner image, and the
  breakpoint fix needs 3.10 (user's choice, D-038).
- **Inline 6502 marker bytes vs assembled test programs:** marker PRGs and carts are generated
  in Rust as about 20 hand-encoded bytes each, so the boot tests don't need the assembler and stay
  readable next to their assertions.
- **An ignore-count wait as in `boot.rs`:** not used by the new tests. Every wait is a `break exec`
  on a label or a marker's fixed spin address.

---

<!--
Quality checks before approving this plan:
  - Every spec acceptance criterion maps to something in this plan
  - Constitution check has at least one filled-in row beyond the always-on bundled rules
  - If behavior is multi-surface → matrix has one row per (AC × non-`none` surface) pair, each cell in the <file>:<token> or <file> convention above
  - Every AC ID referenced in the matrix matches the spec.md (no orphan rows, no missing ACs)
  - Reused primitives section is non-empty (something is being reused — even if just the test framework)
  - File structure is concrete enough to translate directly into tasks.md AND the JOURNAL EDIT BOUNDARY
-->
