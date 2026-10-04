# Plan: Cartridge entries beyond PRG

> **Deep lane · Phase 2 of 6 (Plan).** This document owns **how**. Specify ≠ Plan: if you're
> about to write a user story or acceptance criterion, that belongs in `spec.md`.

**Slug:** `cart-entries`
**Spec:** [`spec.md`](./spec.md) · **Journal:** [`JOURNAL.md`](./JOURNAL.md)
**Created:** 2026-09-19
**Status:** draft <!-- draft | approved | implemented | superseded -->

---

## Goal

Widen the entry model to the nine cartridge kinds and the two half-confined program kinds, give the
placer the reference tool's cart-first, gap-filling algorithm, make Ocean a derived whole-image mode,
and expose all of it through `build`, `edit` and `list` — pinned by new `ndeft` fixtures wherever
the oracle still reaches.

## Architecture

**One entry type, one index space.** `CartBuilder`'s `Vec<Program>` becomes `Vec<Entry>` where
`Entry` is `Program(Program) | Cartridge(Cartridge)`. This is what makes AC-43 true by construction:
`remove`, `set_name`, `set_hidden`, `replace_data` index the one vector, so a cartridge entry is
addressed exactly as a program is. `programs()` becomes `entries()`; `with_program`/`push` stay and
gain `with_cartridge`. The rejected shape — a second `Vec<Cartridge>` beside the programs — would
give the CLI two index spaces to reconcile and break AC-27's "every index refers to the cart as
given". `Cartridge` holds what the format needs and nothing derived: `kind: CartKind` (nine
variants carrying their type value), `align_64k`, and the two bank runs `roml: Vec<u8>` /
`romh: Vec<u8>` (whole 8 KB banks, contiguous) with `roml_at` / `romh_at` (each run's bank offset
from the entry's start bank; the primary half's is always 0). The directory's CrtUsage word is
derived from these at encode time (`other_off`/`other_num` are whichever half is not primary for the
kind — ROMH for 8k/16k/Ocean/xbank-8k/16k, ROML for the three Ultimax kinds, exactly bit 1 of the
type value) and `size` is `(roml_banks + romh_banks) * 0x2000`. The reference tool's two data
layouts (a flat L-then-H buffer for generic/Ocean, a full 1 MB image for xbank) are not reproduced:
they are storage conveniences, and both write the same bank copies to flash (`cart.c:844-892`).

**A cartridge is derived from its image, never declared.** New module `cartridge.rs` owns
`Cartridge::from_crt(bytes)`, applying the reference tool's derivation (`cart.c:414-527, 628-800`)
per AC-39: CRT id 0 → 8k / 16k / Ultimax-8k / Ultimax-16k from the packet load addresses; id 5 →
Ocean or Ocean-512k by total size (32/128/256 KiB → Ocean, 512 → Ocean-512k, else refused); id 33 →
xbank 8k/16k/Ultimax by the header's EXROM/GAME mode, refusing a nonzero base bank, an empty
primary half, or mode "off". Any other id — including 32, an EasyFlash multicart — is refused with
the id named. To read the packets of a non-EasyFlash image, `crt::parse`'s packet walk is split out
as `crt::packets(bytes) -> (Header, Vec<Packet>)`; `parse` keeps its EasyFlash-only checks on top
of it, so the existing container behaviour and its tests are untouched. One deliberate divergence:
packets must be whole banks (`0x2000`, or `0x4000` at `$8000`). The reference accepts anything
≥ `0x100` and then writes a `size` its own reader turns into zero primary banks, so a short packet
is refused here as an input the format cannot carry, not silently padded.

**The placer grows the reference algorithm, not a second placer.** `Placer` already keeps the
same `used[half][bank]` byte table as `flash_bank_used`, so cart placement is additive: `place_cart`
is `find_free_banks` (`place.c:174-207` — first run of `bank_top` banks whose needed halves are
untouched, stepping 8 when aligned), marking whole banks used; `find_gap` is `find_free_gap`
(`place.c:209-250` — ROML gaps below the top bank first, then ROMH); `place_confined` marks a
program into one half, next-bank continuation. `place_all(entries)` orchestrates `place_entries`
(`place.c:577-615`): weights (`place.c:504-543`), stable sort descending, carts in weight order,
then `place_all_prgs` (`place.c:434-476`) — for each round, find the first gap that fits the
*smallest* unplaced program, place the *largest* unplaced program that fits it, else place the
largest at the top interleaved. `build_image` calls `place_all` and stops sorting by size itself;
for a programs-only set the algorithm degenerates to today's largest-first interleaved path, which
is what keeps AC-40's "bytes it produced before" true and is pinned by the four existing fixtures.
`occupy` learns confined kinds and carts so `CartInfo::free_space` (AC-46) replays every entry.

**Ocean is a property of the entry set.** `reserved` gains a `Mode { Normal, Ocean }`:
`write_loader` / `write_boot` / `verify_regions` take it, Ocean putting the OCM loader at bank 1 ROMH
(`0x82000`) and the OCM launcher in the boot region; `Placer::new(mode)` reserves bank 0 ROMH plus
bank 0 ROML (normal) or bank 1 ROMH (Ocean), as `clear_placements` does (`place.c:670-685`).
`CartBuilder` derives the mode from "any entry is Ocean" — nothing is declared, and the container
header stays EasyFlash id 32 / Ultimax. A second Ocean entry fails placement at bank 0 and surfaces
as a typed `BuildError::SecondOcean` naming the entry (AC-37) rather than as an opaque "did not
fit". `load` reorders: walk the directory first, derive the mode from it, *then* verify the firmware
regions against that mode's blobs — today's verify-first order cannot know which loader to expect.

**The read path describes everything but subdirectories.** `directory::decode` yields
`Slot::Program` (now for all three PRG kinds), `Slot::Cartridge(DirEntry)` carrying the decoded
CrtUsage fields, `Slot::Subdirectory` (type `0x1A`, or the `0x4000` bit on any cart type),
`Slot::End`, `Slot::Unknown`. `DirEntry` gains `kind: EntryKind` (an enum over all eleven live
kinds), `align_64k`, `other_off`, `other_num`; for programs the last three are zero. Geometry is
checked in the walk per AC-32 — `other_num <= size / 0x2000` before the subtraction, both runs
against 64 — the same rule for every kind, which is D-022's bound. `Unsupported::CartridgeEntry`
and `Unsupported::ConfinedEntry` are deleted; `Unsupported::Subdirectory { slot }` replaces them.
`load` reads a confined program along its half and a cartridge by copying its two bank runs.

**The CLI gets no new command.** `build` and `edit --add` / `--replace` classify each input by
its first sixteen bytes: the CRT magic means `Cartridge::from_crt`, anything else is a program
(AC-40's "by content"). `build --align INPUT` mirrors `--hide INPUT`; `edit --add-aligned FILE`
adds with the request set (an index cannot name an entry that does not exist yet, AC-27), and
`edit --align N` / `--unalign N` toggle it by index; the core's `set_align_64k` panics on a
non-xbank entry as the other index mutators do on a bad index, and the CLI checks first so the
refusal names the entry (AC-42). `render` labels the eleven kinds, widens the flags column to
`H`/`A` marks, and replaces the offset column with a layout column: the program's offset word as
today, or a cart's runs as `L<n>@+<off> H<n>@+<off>`.

**Fixtures are the parity evidence, and they need synthetic inputs.** `fixtures/make_inputs.py`
generates deterministic `.crt` inputs (one per kind, byte ramps per bank) and the MANIFEST records
each `ndeft` request; the parity table in `tests/parity.rs` gains one case per new image. The wide
xbank case is the one the oracle cannot pin (D-022): `ndeft` may build it but cannot list it, so its
evidence is the round trip here plus a VICE boot logged in the JOURNAL.

**No recon.** Every fact this plan stands on is in `docs/format/easyflash-cart-format.md` or was
read from the reference sources with `file:line` cited above; no library or external API is involved.

## Constitution check

| Rule | Applies? | Compliance note |
|---|---|---|
| `testing.md` (tests first, coverage) | yes (always) | `/lcd:test-gen` emits one failing test per (AC × surface) before implementation; red-green commits per flip; full `cargo test --workspace` in the gate before every commit. |
| `no-downgrade.md` (preserve features) | yes (always) | Every existing command, option and refusal stays. Two refusals *narrow* (cartridge entries, confined programs) because the model now carries them — the widening the user approved in the spec. `CartBuilder::programs()` is renamed `entries()` with a wider element type; the CLI is its only caller in this repo and is updated in the same commit. The listing keeps every column and adds marks; no field is dropped. |
| `no-overengineering.md` (minimal scope) | yes (always) | No new dependency; one entry enum, not a trait; the reference algorithm reproduced, not improved; no reorder/extract/sort/firmware-override; no per-kind bound table (D-022). Labels and grammar are the smallest that satisfy the ACs. |
| `refinement-protocol.md` (ambiguous edits) | yes (always) | The five forks were put to the user in the spec and answered before this plan; the remaining choices here (enum model, `--add-aligned`, whole-bank packets) are recorded with their rejections below and in D-023/D-024. |
| `versioning.md` (bump in first commit) | **yes — action required** | Branch is `feat/`; base `Cargo.toml` on `main` is `0.5.0` → `0.6.0` in the first implementation commit (T-1). `parity.rs` pins `REFERENCE_HEADER_NAME` explicitly, so the fixtures are unaffected by the header default changing. |
| `commits.md` (atomic, conventional) | yes (always) | One commit per red→green flip, conventional prefixes, pushed per the project's own commits rule. |
| `ac-convention.md` — EVAL coverage | no | Output is byte-exact images diffed against frozen fixtures; nothing is scored. `MAP.md` records EVAL as n/a. |
| `MAP.md` invariant: byte parity gates the write path | yes | The four existing fixtures must still match: `place_all` degenerates to today's path with no carts (asserted by the existing parity cases, which never change). New behaviour is pinned by new fixtures added in the same commits as the code they cover. |
| `MAP.md` invariant: the oracle decays | yes | The wide xbank (AC-38) is outside the oracle; its evidence is the round trip plus a VICE boot recorded in the JOURNAL, per D-022. Everything else has an `ndeft` fixture. |
| `MAP.md` invariant: `firmware/` is zlib, no code crosses | yes | The OCM blobs are already embedded (`firmware.rs`); this work only starts writing them. No file moves across the boundary. |
| `MAP.md` invariant: core holds no global state / I/O policy | yes | `Cartridge::from_crt` takes bytes; input classification by content happens in the CLI on bytes it read; the core still returns `Vec<u8>`. |
| `MAP.md` invariant: the crate version reaches produced bytes | yes | Covered by the versioning row. |
| `MAP.md` invariant: describing ≠ rebuilding (D-011) | yes | `inspect` describes carts, confined programs and their positions; `load` layers firmware verification (now mode-aware) on top. The only remaining "cannot describe" refusal is the subdirectory, whose nested directory `CartInfo` has no model for. |
| `MAP.md` invariant: read-path divergence per behaviour (D-007) | yes | Two divergences, each classified: the flat 512 KiB cap is fixed (D-022, spec lists it as an artifact); short generic packets are refused rather than accepted-then-broken (D-023). The subdirectory refusal and the whole-bank Ocean/xbank checks reproduce the reference. |
| `MAP.md` invariant: format facts come from `docs/format/` | yes | Every offset, mask and rule used here is already in the spec (CrtUsage §, placement §, bank 0 map). The Ocean loader offset `0x82000` is stated there. If the fixture work surfaces a fact the spec lacks, it lands there first, in its own commit (D-018). |
| `.claude/rules/lcd-conventions.md` (AC ids, test placement, stems) | yes | ACs AC-31..AC-46, "Next free" already bumped to AC-47. New test stems `cartridge`, `cart_placement`, `ocean` (core) and `carts`, `cartlist` (cli) are unique across both crates. |

## Cross-path behavior matrix

| AC | Surface | Path |
|---|---|---|
| `AC-40` | CLI | `crates/stidcart-cli/src/main.rs:build` |
| `AC-41` | CLI | `crates/stidcart-cli/src/main.rs:edit` |
| `AC-42` | CLI | `crates/stidcart-cli/src/main.rs:--align` |
| `AC-43` | CLI | `crates/stidcart-cli/src/main.rs:edit` |
| `AC-44` | CLI | `crates/stidcart-cli/src/main.rs:from_crt` |
| `AC-45` | RENDER | `crates/stidcart-cli/src/main.rs` |
| `AC-46` | RENDER | `crates/stidcart-cli/src/main.rs` |

AC-31..AC-39 are `none` and carry no row. Their tests are the coverage (see File structure).

**Test scope:** `crates/stidcart-core/tests/cartridge.rs`, `crates/stidcart-core/tests/cart_placement.rs`, `crates/stidcart-core/tests/ocean.rs`, `crates/stidcart-cli/tests/carts.rs`, `crates/stidcart-cli/tests/cartlist.rs`

## Reused primitives

| Existing primitive | Path | Used for |
|---|---|---|
| `Placer` + `used[half][bank]` table, `top()`, `place_interleaved`, `mark`, `occupy` | `crates/stidcart-core/src/place.rs` | The byte table the reference algorithm needs already exists; cart placement, gaps and confined marking extend it |
| `placement_order` | `crates/stidcart-core/src/place.rs:170` | Becomes the stable descending sort over weights (programs' weight is their size, so the existing tests keep holding) |
| `crt::parse` packet walk, `header()`, `push_chip` | `crates/stidcart-core/src/crt.rs` | Split into `packets()` and reused by both the EasyFlash parser and `Cartridge::from_crt`; header writer unchanged |
| `directory::encode` / `decode`, `is_old_format`, `Slot` | `crates/stidcart-core/src/directory.rs` | Extended with the cart kinds and CrtUsage; the EasyFS1 discriminator and end-marker logic untouched |
| `FlashImage::{bank, write_at, write_program, read_program}` | `crates/stidcart-core/src/image.rs` | Bank copies for cart runs; confined program I/O adds a same-half stepper beside the interleaved one |
| `reserved::{write_eapi, write_boot, write_loader, verify_regions, write_ef_name}` | `crates/stidcart-core/src/reserved.rs` | Gain a `Mode` parameter; EAPI and EF-Name are mode-independent and unchanged |
| `firmware::{LOADER_OCM, LAUNCHER_OCM, body, load_addr}` | `crates/stidcart-core/src/firmware.rs` | Already embedded and size-tested; Ocean mode starts writing them |
| `read::read_directory` (one walk, two views), `check_extent` | `crates/stidcart-core/src/read.rs` | The walk grows cart and confined cases; `check_extent` gains the per-kind continuation |
| `CartBuilder` mutators, `BuildError` | `crates/stidcart-core/src/lib.rs` | Index-addressed edits over the wider `Entry`; `SecondOcean` joins the error enum |
| `entry_name`, `write_replacing`, three-phase op application | `crates/stidcart-cli/src/main.rs` | Naming, atomic write and index discipline for the new inputs and options |
| CLI test harness (`stidcart()`, `scratch()`, `fixtures()`, `cart_copy()`) | `crates/stidcart-cli/tests/edit.rs:15-40` | Copied into the new CLI test files |
| Parity table + `assert_same_bytes` | `crates/stidcart-core/tests/parity.rs`, `roundtrip.rs` | New fixture cases slot into the existing `Case` table; the first-differing-byte reporter is reused |
| `fixtures/MANIFEST.md` conventions | `fixtures/` | Every new image records its exact `ndeft` request and its inputs' generator |

## Data model / Contracts

```rust
// stidcart-core
pub enum Entry { Program(Program), Cartridge(Cartridge) }
impl Entry { pub fn name(&self) -> &str; pub fn hidden(&self) -> bool; /* + setters */ }

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CartKind { K8, K16, Ultimax8, Ultimax16, Ocean, Ocean512, Xbank8, Xbank16, XbankUltimax }
impl CartKind { pub fn type_value(self) -> u8; pub fn primary(self) -> Half; pub fn is_ocean(self) -> bool; pub fn is_xbank(self) -> bool }

pub struct Cartridge {
    pub name: String, pub hidden: bool, pub kind: CartKind, pub align_64k: bool,
    pub roml: Vec<u8>, pub roml_at: u8,   // ROML run: whole banks, offset from the start bank
    pub romh: Vec<u8>, pub romh_at: u8,   // ROMH run likewise; the primary half's offset is 0
}
impl Cartridge {
    pub fn from_crt(bytes: &[u8]) -> Result<Cartridge, CartridgeError>;   // AC-39
    pub fn banks(&self) -> (usize, usize);                                // (roml, romh) bank counts
}
pub enum CartridgeError { NotACartridge, UnsupportedId(u16), Geometry(&'static str), Malformed(ReadError) }

// directory
pub enum EntryKind { Prg, PrgRomL, PrgRomH, Cart(CartKind) }
pub struct DirEntry { name, kind: EntryKind, hidden, bank: u16, offset: u16, size: u32,
                      align_64k: bool, other_off: u8, other_num: u8 }
pub enum Slot { End, Program(DirEntry), Cartridge(DirEntry), Subdirectory, Unknown { type_value } }

// reserved
pub enum Mode { Normal, Ocean }
pub fn write_loader(img, mode); pub fn write_boot(img, mode); pub fn verify_regions(img, mode);

// place
impl Placer { pub fn new(mode: Mode); pub fn place_all(&mut self, entries: &[Entry]) -> Result<Vec<Placement>, BuildError>; }
pub struct Placement { pub bank: u16, pub offset: u16, pub kind: EntryKind }   // offset = 0 for carts

// CartBuilder
pub fn with_cartridge(self, c: Cartridge) -> Self;  pub fn push(&mut self, e: Entry);
pub fn entries(&self) -> &[Entry];  pub fn set_align_64k(&mut self, index: usize, on: bool);
pub fn replace_data(&mut self, index: usize, e: Entry);  // kind follows the data (AC-41)
pub enum BuildError { TooManyEntries, OutOfSpace, SecondOcean { name } , AlignNotXbank { name } }
```

CLI grammar added to `USAGE`:

```
stidcart build [-e NAME] [--hide INPUT]... [--align INPUT]... -o OUTPUT INPUT...
  INPUT            a .prg, or a cartridge image (recognised by content)
  --align INPUT    request 64 KB alignment for that xbank input
stidcart edit CART.crt [...]
  --add FILE           append a program or a cartridge image
  --add-aligned FILE   append an xbank image with 64 KB alignment requested
  --replace N FILE     swap the data at index N; the entry's kind follows the file
  --align N / --unalign N   set or clear 64 KB alignment on the xbank entry at N
```

Listing row: `#  name  flags  type  bank  layout  size` — flags `H`/`A` marks (`--`, `H-`, `-A`,
`HA`), type one of `PRG PRG-L PRG-H 8K 16K ULT8 ULT16 OCEAN OCN512 XB8 XB16 XBULT`, layout the
offset word for programs (`$0002`) or `L<n>@+<o> H<n>@+<o>` for carts.

## File structure

### New files

```
crates/stidcart-core/src/cartridge.rs        Cartridge, CartKind, from_crt (AC-39), bank-run helpers
crates/stidcart-core/tests/cartridge.rs      AC-31 round trip per kind, AC-32 geometry refusals, AC-39 derivation
crates/stidcart-core/tests/cart_placement.rs AC-33 order/first-fit/alignment, AC-34 gaps, AC-35 confined read-back, AC-38 wide xbank
crates/stidcart-core/tests/ocean.rs          AC-36 Ocean mode image + load, AC-37 second Ocean refused
crates/stidcart-cli/tests/carts.rs           AC-40..AC-44 build/edit with cartridge inputs, alignment, refusals
crates/stidcart-cli/tests/cartlist.rs        AC-45 labels/layout/marks, AC-46 free bytes
fixtures/make_inputs.py                      deterministic generator for the synthetic input .crt files
fixtures/in_8k.crt in_16k.crt in_ult8.crt in_ult16.crt in_ocean.crt in_ocean512.crt
         in_xb8.crt in_xb16.crt in_xbult.crt in_xbwide.crt     generated inputs (checked in)
fixtures/c8k.crt c16k.crt cult8.crt cult16.crt cocean.crt cocean512.crt
         cxb16.crt cxb16a.crt cxbult.crt cmixed.crt           ndeft-built reference images
```

### Modified files

- `Cargo.toml` — `0.5.0` → `0.6.0` (first implementation commit)
- `crates/stidcart-core/src/lib.rs` — `Entry`, `pub mod cartridge`, `CartBuilder` over entries, mode derivation, `place_all` in `build_image`, cart bank writes, `SecondOcean` / `AlignNotXbank`
- `crates/stidcart-core/src/directory.rs` — `EntryKind`, cart type values, CrtUsage encode/decode, `Slot::{Cartridge, Subdirectory}`
- `crates/stidcart-core/src/place.rs` — `new(mode)`, `place_cart`, `find_gap`, `place_confined`, `place_all`, weights, `occupy` for every kind
- `crates/stidcart-core/src/read.rs` — walk handles cart/confined/subdir, geometry checks (AC-32), `load` derives mode then verifies, `Unsupported::{CartridgeEntry, ConfinedEntry}` → `Subdirectory`
- `crates/stidcart-core/src/reserved.rs` — `Mode`, OCM loader/boot writes, mode-aware `verify_regions`, `LOADER_OCM_OFFSET`
- `crates/stidcart-core/src/crt.rs` — `packets()` split out of `parse`; `Packet`/`Header` types
- `crates/stidcart-core/src/image.rs` — confined program write/read (same-half stepper), bank-run copy helpers
- `crates/stidcart-core/tests/read.rs` — cases asserting the two retired refusals flip to "described/loaded"
- `crates/stidcart-core/tests/parity.rs` — `Case` gains cartridge inputs and align marks; one case per new fixture
- `crates/stidcart-core/tests/roundtrip.rs` — new fixtures join `FIXTURES`
- `crates/stidcart-cli/src/main.rs` — input classification, `--align`, `--add-aligned`, `--align N`/`--unalign N`, `render` labels/layout/marks, `USAGE`
- `crates/stidcart-cli/tests/list.rs`, `edit.rs` — rows asserting the old flags/offset columns and the AC-25 "cartridge entry refused" case are updated in the same commit as the change
- `fixtures/MANIFEST.md` — inputs' generator, every new request
- `README.md` — State section ("PRG entries only") and the ndefpack-quirks paragraph
- `docs/lcd/DECISIONS.md` — D-023, D-024 (logged with this plan)
- `docs/lcd/work/cart-entries/` — JOURNAL upkeep
- `docs/lcd/SPEC.md` — folded at closeout by `lcd:reconcile`

## Risks & rejected alternatives

- **A parallel `Vec<Cartridge>` instead of an `Entry` enum**: rejected — two index spaces break
  AC-27/AC-43 and force every mutator to exist twice.
- **Reproducing the reference tool's entry-data layouts** (flat L-then-H buffer; 1 MB image for
  xbank): rejected — they are memory conveniences of one implementation; the flash bytes they
  produce are identical bank copies, which the two-run model writes directly.
- **Accepting short generic packets as the reference does**: rejected (D-023) — the size it then
  records decodes to zero primary banks in its own reader; refusing at the input is the honest
  version of that behaviour.
- **Declaring Ocean mode on the builder**: rejected (D-024) — the reference derives it from the
  entry set, and a declared mode that disagrees with the entries is a cart that does not boot.
- **`--align` by index at add time in `edit`**: rejected — AC-27 says indices name the cart as
  given, and the added entry does not exist yet; `--add-aligned` is the smallest grammar that avoids
  a second index space.
- **Keeping `programs()` beside `entries()`**: rejected — a count that excludes carts would mislead
  exactly the bounds check the CLI relies on. Rename, one caller, same commit.
- **A wide-xbank fixture from `ndeft`**: attempted, not relied on — the reference may write one it
  cannot list. If it does, it joins the parity table; if not, AC-38's evidence is the round trip and
  the VICE boot, per D-022.
- **Ocean-512k fixture size** (~0.5 MB): accepted — it is the only kind that fills a whole half, and
  the ramp content compresses well in git.

---

<!--
Quality checks before approving this plan:
  - Every spec AC maps to something here ✅ (AC-40..46 in the matrix; AC-31..39 in Architecture and the new core tests)
  - Constitution check has rows beyond the bundled rules ✅ (9 MAP invariants + lcd-conventions)
  - Matrix has one row per (AC × non-`none` surface) ✅
  - Reused primitives non-empty ✅
  - File structure concrete enough for tasks.md and the EDIT BOUNDARY ✅
-->
