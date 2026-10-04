# Tasks: Cartridge entries beyond PRG

> **Deep lane · Phase 3 of 6 (Tasks).** A flat, dependency-ordered checklist. Each task names
> the file it touches. Tests come before the code they test (TDD). The granular STEPS in the
> work-item `JOURNAL.md` mirror this list as the resume view; keep them in sync.

**Slug:** `cart-entries`
**Spec:** [`spec.md`](./spec.md) • **Plan:** [`plan.md`](./plan.md) • **Journal:** [`JOURNAL.md`](./JOURNAL.md)
**Created:** 2026-09-19
**Status:** in_progress <!-- in_progress | completed | abandoned -->

---

## Conventions

- `[P]` suffix = parallelizable with the previous task (different file, no shared state).
- Tests precede implementation tasks they cover.
- One file per task where possible; tasks naming multiple files should be split unless the change is genuinely atomic.
- Test names carry the literal `AC-N (SURFACE)` in the doc comment, as the audit greps for it.

## Tasks

### Foundation

- [x] **T1**: Bump `version` `0.5.0` → `0.6.0` in `Cargo.toml` — `feat/` branch, first implementation
  commit (versioning rule). `parity.rs` pins its own header name, so no fixture moves.
- [x] **T2** [P]: Write `fixtures/make_inputs.py` — deterministic synthetic input images, one per kind:
  `in_8k.crt` (id 0, one `0x2000` packet at `$8000`), `in_16k.crt` (id 0, `0x4000` at `$8000`),
  `in_ult8.crt` (id 0, `0x2000` at `$E000`), `in_ult16.crt` (id 0, `$8000` then `$E000`),
  `in_ocean.crt` (id 5, 16 banks at `$8000` + 16 at `$A000`, 256 KiB), `in_ocean512.crt` (id 5, 64
  banks at `$8000`), `in_xb8.crt` (id 33, mode 8k, banks 0–3 ROML), `in_xb16.crt` (id 33, mode 16k,
  banks 0–3 both halves), `in_xbult.crt` (id 33, mode ult, banks 0–1 ROMH + 0–1 ROML),
  `in_xbwide.crt` (id 33, mode 16k, banks 0–39 both halves — 640 KiB, over the reference cap). Byte
  ramps per bank, formula recorded in the script and in `MANIFEST.md`. Generate and check the
  inputs in.
- [x] **T3**: Build the reference images with `ndeft` (built from `stid/ndefpack@d77724b`) from inside
  `fixtures/`, one request per behaviour, and record every request in `fixtures/MANIFEST.md`:
  `c8k.crt` (`hello.prg in_8k.crt` — cart first, program into the ROMH gap as PRG-H), `c16k.crt`
  (`ramp.prg in_16k.crt`), `cult8.crt` (`hello.prg in_ult8.crt`), `cult16.crt` (`hello.prg
  in_ult16.crt`), `cocean.crt` (`big.prg in_ocean.crt` — Ocean mode, loader at bank 1 ROMH),
  `cocean512.crt` (`hello.prg in_ocean512.crt`), `cxb16.crt` (`hello.prg in_xb16.crt`), `cxb16a.crt`
  (`"in_xb16.crt,aligned,a" hello.prg ramp.prg` — 64 KB alignment, programs fill the gap below),
  `cxbult.crt` (`hello.prg in_xbult.crt`), `cmixed.crt` (`in_8k.crt in_16k.crt in_ult16.crt
  in_xb8.crt hello.prg ramp.prg big.prg` — the full ordering). Also *attempt* `cxbwide.crt`
  (`in_xbwide.crt hello.prg`) and record whether `ndeft` produced it (D-022); if it did, verify
  `ndeft list` refuses it and keep the image. Never boot a fixture in VICE without a copy.

### Core: directory encoding of cartridge kinds (AC-31, AC-32)

- [x] **T4**: Write failing test `AC-31 (none): every cartridge kind round-trips through encode and
  decode` in `crates/stidcart-core/tests/cartridge.rs` (new file) — for each of the nine kinds,
  a `DirEntry` with name, hidden, bank, alignment, other-half offset/count and size; `decode(encode(e))`
  yields `Slot::Cartridge` equal to it, and the derived primary bank count matches.
- [x] **T5**: Write failing test `AC-32 (none): a cartridge entry reaching past flash or with an
  oversized opposite half is refused as malformed` in `crates/stidcart-core/tests/cartridge.rs` —
  three carts: primary run past bank 63, opposite run past bank 63, `other_num > size / 0x2000`;
  each written into an otherwise valid image and `inspect`ed; all three return
  `ReadError::OutOfBounds`/`BadEntry`-class errors naming the slot, none panic.
- [x] **T6**: Implement in `crates/stidcart-core/src/directory.rs`: `EntryKind { Prg, PrgRomL,
  PrgRomH, Cart(CartKind) }`, `CartKind` type values and `primary()` (bit 1 of the value),
  `DirEntry { align_64k, other_off, other_num }`, CrtUsage encode (`0x8000 | align << 13 |
  other_off << 8 | other_num`) and decode, `Slot::Cartridge(DirEntry)`, `Slot::Subdirectory` for type
  `0x1A` or the `0x4000` bit. `CartKind` lives in `cartridge.rs` (T9) — this task and T9 are
  committed together only if the enum has to exist for T6 to compile; otherwise define `CartKind`
  here first and re-export.
- [x] **T7**: Implement the geometry check in `crates/stidcart-core/src/read.rs`'s walk — for
  `Slot::Cartridge`: `other_num <= size / 0x2000` before subtracting, `bank + primary_num <= 64`,
  `bank + other_off + other_num <= 64`; refuse with `ReadError::OutOfBounds { at, len }` (image
  coordinates) or `BadEntry` as the case dictates; `Slot::Subdirectory` →
  `Unsupported::Subdirectory { slot }`. Cart entries are pushed into `CartInfo.entries`; confined
  programs stay refused until T17. Passes T4, T5.

### Core: a cartridge derived from its image (AC-39)

- [x] **T8**: Write failing test `AC-39 (none): a cartridge's kind and bank runs are derived from its
  image` in `crates/stidcart-core/tests/cartridge.rs` — for each `fixtures/in_*.crt`, `from_crt`
  yields the expected `CartKind`, bank counts, run offsets and bytes; an EasyFlash image
  (`fixtures/two.crt`) is refused with `UnsupportedId(32)`; an id-0 image with a `0x1000` packet is
  refused; an id-5 image of 64 KiB is refused (not a listed Ocean size); an id-33 image with mode
  off, or whose primary run starts above bank 0, is refused.
- [x] **T9**: Split the packet walk out of `parse` as `pub fn packets(bytes) -> Result<(Header,
  Vec<Packet>), ReadError>` in `crates/stidcart-core/src/crt.rs` — `Header { id, exrom, game, name
  }`, `Packet { bank, addr, data }`; `parse` keeps the EasyFlash id check and its geometry match on
  top. Existing `container.rs` tests unchanged and green.
- [x] **T10**: Create `crates/stidcart-core/src/cartridge.rs` — `CartKind`, `Cartridge { name, hidden,
  kind, align_64k, roml, roml_at, romh, romh_at }`, `banks()`, `CartridgeError`, and `from_crt`
  per the reference derivation (`cart.c:414-527, 628-800`; D-023): id 0 → generic kinds by packet
  addresses (whole banks only), id 5 → Ocean by size (32/128/256 KiB) or Ocean-512k, id 33 → xbank by
  header mode with base-bank-0 / nonempty-primary checks, else `UnsupportedId`. `pub mod cartridge`
  in `lib.rs` is part of this task. Passes T8.

### Core: placement with cartridges (AC-33, AC-34, AC-38)

- [x] **T11**: Write failing test `AC-33 (none): cartridges are placed before programs, in weight
  order, first-fit in both halves` in `crates/stidcart-core/tests/cart_placement.rs` (new file) —
  build the `cmixed.crt` entry set through `CartBuilder`, read the directory back with `inspect`, and
  assert each cart's start bank equals the reference image's; plus a unit-level case: an aligned
  xbank lands on a multiple of 8 banks with a gap below it (`cxb16a.crt`).
- [x] **T12**: Write failing test `AC-34 (none): programs fill gaps between cartridges as confined
  entries and the top interleaved otherwise` in `crates/stidcart-core/tests/cart_placement.rs` — the
  `cult8.crt` set places `hello` as `PrgRomL` at bank 1 (the Ultimax cart holds only ROMH); the
  `cxb16a.crt` set places `ramp` at bank 1 ROML offset 0 then `hello` at offset `0xbba`, both
  `PrgRomL`, below the aligned cart at bank 8; the `cmixed.crt` set puts `big`, `ramp`, `hello` into
  the ROMH gap at banks 1–3 as `PrgRomH`; and the `c8k.crt` set places `hello` at the *top* (bank 1
  ROMH, interleaved `Prg`) — gaps are searched only below the top bank, so bank 1 ROMH is top space,
  not a gap. Each position equals the reference image's directory.
- [x] **T13**: Write failing test `AC-38 (none): an xbank wider than 32 banks is accepted, written and
  read back intact` in `crates/stidcart-core/tests/cart_placement.rs` — `in_xbwide.crt` + `hello`
  builds; `load` of the result returns the same cartridge bytes and runs; and an image whose only
  fault is a 640 KiB entry is `inspect`ed and `load`ed (if `cxbwide.crt` exists, use it; else the
  built image).
- [x] **T14**: Implement in `crates/stidcart-core/src/place.rs`: `Placer::new(mode)` reserving bank 0
  ROMH + (normal: bank 0 ROML | Ocean: bank 1 ROMH); `place_cart(l_off, l_num, h_off, h_num, step)`
  = `find_free_banks`; `find_gap` = `find_free_gap`; `place_confined`; weights
  (`place.c:504-543`) and stable descending order; `place_all(&[Entry]) -> Result<Vec<Placement>,
  BuildError>` reproducing `place_entries` + `place_all_prgs`; `Placement.kind`; `occupy` for
  confined programs and carts. Existing placement unit tests stay green.
- [x] **T15**: Implement in `crates/stidcart-core/src/lib.rs`: `Entry` enum with accessors, `Vec<Entry>`
  in `CartBuilder`, `with_cartridge`, `push(Entry)`, `entries()` replacing `programs()`,
  `set_align_64k`, `replace_data(index, Entry)`, `BuildError::{SecondOcean, AlignNotXbank}`, mode
  derivation, `build_image` via `place_all` with cart bank copies and confined program writes. Atomic
  with `crates/stidcart-cli/src/main.rs`'s two `programs()` call sites (rename only) so the
  workspace compiles. Passes T11, T12 (write side).
- [x] **T16**: Add confined program write/read (same-half, next-bank stepper) and bank-run copy
  helpers to `crates/stidcart-core/src/image.rs`. Passes T13 with T17.

### Core: the read path describes and loads every kind (AC-35)

- [x] **T17**: Write failing test `AC-35 (none): confined programs are described and loaded, and a
  fixture holding them saves back byte-identical` in `crates/stidcart-core/tests/cart_placement.rs` —
  `inspect(c8k.crt)` lists the PRG-H entry with its position; `load(c8k.crt).build()` equals
  `c8k.crt`; same for `cxb16a.crt`.
- [x] **T18**: Implement in `crates/stidcart-core/src/read.rs`: the walk pushes confined programs
  (extent checked along their half) and cartridges; `load` copies confined programs and cart runs
  into `Entry`s; delete `Unsupported::{CartridgeEntry, ConfinedEntry}`. Atomic with
  `crates/stidcart-core/tests/read.rs`'s cases that match on those variants — they flip to
  asserting the entry is described. Passes T17, T13.

### Core: Ocean mode (AC-36, AC-37)

- [x] **T19**: Write failing test `AC-36 (none): an Ocean entry puts the image in Ocean mode and loads
  back under it` in `crates/stidcart-core/tests/ocean.rs` (new file) — the `cocean.crt` set builds
  with the OCM loader bytes at `0x82000`, the OCM launcher in the boot region, the Ocean run at bank 0
  ROML, container id 32 with `01 00`; `load(cocean.crt)` succeeds and `.build()` equals the fixture;
  `load(two.crt)` (normal mode) still succeeds.
- [x] **T20**: Write failing test `AC-37 (none): a second Ocean entry is refused by name` in
  `crates/stidcart-core/tests/ocean.rs` — two Ocean cartridges → `BuildError::SecondOcean { name }`
  naming the second in input order.
- [x] **T21**: Implement in `crates/stidcart-core/src/reserved.rs`: `Mode`, `LOADER_OCM_OFFSET =
  0x82000`, mode-aware `write_loader`/`write_boot`/`verify_regions` (Ocean checks bank 1 ROMH for
  the loader and leaves bank 0 ROML to the entry). Existing reserved tests pass `Mode::Normal`.
- [x] **T22**: Wire the mode in `crates/stidcart-core/src/read.rs` (`load`: walk first, derive mode,
  then `verify_regions(mode)`) and in `crates/stidcart-core/src/lib.rs` (`build_image` derives the
  mode, `SecondOcean` raised before placement). Passes T19, T20.

### Core: parity and round trip on the new fixtures

- [x] **T23**: Extend the `Case` table in `crates/stidcart-core/tests/parity.rs` — inputs become
  `(stem, name, kind: Prg | Crt, aligned)`; one case per T3 image (`cxbwide` only if produced). The
  four existing cases are untouched. Green after T15–T22.
- [ ] **T24** [P]: Add the new fixtures to `FIXTURES` in `crates/stidcart-core/tests/roundtrip.rs`.
  Green after T18/T22.

### CLI: cartridge inputs, alignment, refusals (AC-40..AC-44)

- [x] **T25**: Write failing test `AC-40 (CLI): build accepts a cartridge image among its inputs,
  recognised by content` in `crates/stidcart-cli/tests/carts.rs` (new file) — `build -o out.crt
  hello.prg in_8k.crt` lists an `8K` entry named `in_8k`; the same file renamed `.prg` still
  becomes a cart; `build` of `hello.prg ramp.prg` is byte-identical to `two.crt` minus the header name.
- [x] **T26**: Write failing test `AC-41 (CLI): edit --add and --replace take cartridge images and the
  kind follows the data` in `crates/stidcart-cli/tests/carts.rs` — `--add in_16k.crt` appends a
  `16K` entry; `--replace 0 in_8k.crt` on `two.crt` keeps the name and index 0 and lists `8K`;
  `--replace 0 hello.prg` on `c8k.crt`'s cart entry turns it back into `PRG`.
- [x] **T27**: Write failing test `AC-42 (CLI): alignment is requested at build or add time and toggled
  by index, xbank only` in `crates/stidcart-cli/tests/carts.rs` — `build --align in_xb16.crt`
  lands it on bank 8 with the `A` mark; `edit --add-aligned in_xb16.crt` likewise; `--unalign N`
  clears it; `--align INPUT` on `in_8k.crt` and `--align N` on a program exit non-zero naming the
  entry, leaving the cart byte-identical.
- [x] **T28**: Write failing test `AC-43 (CLI): every edit operation works on a cartridge entry by
  index` in `crates/stidcart-cli/tests/carts.rs` — on a copy of `cmixed.crt`: `--rename`, `--hide`,
  `--show`, `--remove` on cart indices take effect in `list`; a subdirectory entry (image built
  through `crt::write` with a `0x1A` slot) is refused by `edit` naming it.
- [x] **T29**: Write failing test `AC-44 (CLI): an unsupported cartridge image is refused and every
  file is left untouched` in `crates/stidcart-cli/tests/carts.rs` — `build` with `two.crt` as an
  input (id 32) and `edit --add two.crt` both exit non-zero, mention the id, write no output, and
  leave the input cart byte-identical.
- [x] **T30**: Implement in `crates/stidcart-cli/src/main.rs`: `from_crt` classification by the
  16-byte magic for `build` inputs, `--add`, `--add-aligned`, `--replace`; `--align INPUT`
  (build) and `--align N` / `--unalign N` (edit) with the xbank check before the core call; the
  subdirectory/unsupported refusals reach stderr; `USAGE` updated. Passes T25–T29.

### RENDER: the listing (AC-45, AC-46)

- [x] **T31**: Write failing test `AC-45 (RENDER): every kind has a label, carts show their bank runs,
  aligned entries are marked` in `crates/stidcart-cli/tests/cartlist.rs` (new file) — `list
  cmixed.crt` shows `8K`, `16K`, `ULT16`, `XB8`, `PRG`, `PRG-L`/`PRG-H` rows with `L<n>@+<o> H<n>@+<o>`
  layouts; `list cxb16a.crt` shows the `A` mark; no row contains `?`.
- [x] **T32**: Write failing test `AC-46 (RENDER): free bytes account for cartridge-occupied banks` in
  `crates/stidcart-cli/tests/cartlist.rs` — `list c8k.crt` reports free bytes equal to
  `62 * 0x4000 + (0x2000 - hello's size)` (bank 1 ROML taken by the cart, its ROMH gap partly used).
- [x] **T33**: Implement `render` in `crates/stidcart-cli/src/main.rs` — labels for the eleven kinds,
  two-character flags, layout column, `free_space` via `occupy` (T14). Atomic with
  `crates/stidcart-cli/tests/list.rs` and `edit.rs` rows that assert the old column shapes. Passes
  T31, T32.

### Docs and gates

- [x] **T34**: Update `README.md` — State section ("PRG entries only" → the kinds now handled, Ocean
  mode, the subdirectory as the one refusal) and the read-path quirks paragraph (wide xbank now
  read).
- [x] **T-audit**: Run `/lcd:audit cart-entries`; resulting `audit.md` has zero MISSING/BLOCKED rows.
- [x] **T-final**: Run the gate — `cargo fmt --all --check && cargo clippy --workspace --all-targets
  --all-features -- -D warnings && cargo test --workspace` — all green; then `lcd:reconcile` folds
  AC-31..AC-46 into `docs/lcd/SPEC.md`.

## Checkpoint validation

After every block of related tasks, the suite must stay green. The four original parity fixtures
are the canary for T14/T15: if `single`/`named`/`two`/`spill` stop matching, the programs-only
degenerate path of `place_all` has drifted — stop and fix before moving on. A VICE boot of
`cxbwide.crt` (or the T13-built image) is recorded in the JOURNAL LOG when done (D-022).

## Linkbacks

- `AC-31 (none)` → T4, T6
- `AC-32 (none)` → T5, T7
- `AC-33 (none)` → T3, T11, T14, T15, T23
- `AC-34 (none)` → T3, T12, T14, T15, T16, T23
- `AC-35 (none)` → T17, T16, T18, T24
- `AC-36 (none)` → T19, T21, T22, T23
- `AC-37 (none)` → T20, T22
- `AC-38 (none)` → T2, T3, T13, T14, T16, T18
- `AC-39 (none)` → T8, T9, T10
- `AC-40 (CLI)` → T25, T30
- `AC-41 (CLI)` → T26, T30
- `AC-42 (CLI)` → T27, T30
- `AC-43 (CLI)` → T28, T30
- `AC-44 (CLI)` → T29, T30
- `AC-45 (RENDER)` → T31, T33
- `AC-46 (RENDER)` → T32, T33

---

<!--
Quality checks before starting implementation:
  - Every AC × non-`none`-surface row in plan.md has at least one test task above ✅
  - Every new file in plan.md "File structure" has at least one task that creates it ✅ (T2, T3, T4, T10, T11, T19, T25, T31)
  - Tests appear before their implementation ✅
  - T-audit appears before T-final ✅
-->
