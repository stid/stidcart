# Plan: Read a cartridge image back into the description that produced it

> **Deep lane · Phase 2 of 6 (Plan).** This document owns **how**. Specify ≠ Plan: if you're
> about to write a user story or acceptance criterion, that belongs in `spec.md`.

**Slug:** `read-path`
**Spec:** [`spec.md`](./spec.md) · **Journal:** [`JOURNAL.md`](./JOURNAL.md)
**Created:** 2026-08-10
**Status:** draft <!-- draft | approved | implemented | superseded -->

---

## Goal

Add a loader to `stidcart-core` that turns `.crt` bytes into a `CartBuilder` equivalent to the one
that produced them, refusing anything it cannot represent faithfully, and prove it by rebuilding all
four frozen fixtures byte-identically.

## Architecture

**Each decoder lives beside its encoder.** `crt.rs` gains a container parser next to `crt::write`,
`directory.rs` gains `decode` next to `encode`, `reserved.rs` gains region verification and cartridge-name
extraction next to its writers. A new `read.rs` only orchestrates: container → image → reserved
checks → directory walk → `CartBuilder`. The alternative — one self-contained `read.rs` holding the
whole parse — was rejected because every offset it needs already lives as a `pub const` in the module
that writes it, and a second copy is exactly how an encoder and a decoder drift apart. Pairing them
also makes each module's round-trip testable on its own, so a parity failure localizes to a module
instead of to "somewhere in the read path", which is the property the existing parity test's failure
message already assumes.

**Loading returns a `CartBuilder`, not a new type.** `CartBuilder` is already "the description that
produces an image", and `build()` is already its inverse target, so `load(bytes) -> Result<CartBuilder,
ReadError>` makes AC-2 a one-line property: `load(f)?.build()? == f`. A separate `LoadedCart` type
was rejected — it would need a conversion into `CartBuilder` that is itself untested surface, and two
types describing one cartridge is the drift risk again. This does mean `CartBuilder` grows read
accessors (`header_name()`, `ef_name()`) so AC-1, AC-8 and AC-9 can assert on recovered fields;
`programs()` already exists.

**Errors are two-level, because AC-7 needs them distinguishable.** `ReadError` covers malformed input
— bad magic, a packet whose stated total disagrees with its data length, a size or location reaching
outside the image, a directory with no end marker anywhere. `ReadError::Unsupported(Unsupported)`
covers input that is well-formed but not representable here: a cartridge entry, a hidden entry, a
reserved region whose firmware differs from the embedded blobs. The split is the whole point of
D-007's "typed unsupported rather than silent misparse", and a caller that wants to distinguish
"this file is broken" from "this cartridge is fine, we just can't open it yet" can.

**Validation happens before indexing, not after.** Every length and offset the file supplies is
bounds-checked against the image and against the declared data before it is used to slice (AC-12).
The extracted format spec records that the reference implementation's equivalent path had an unsigned
underflow that was a real heap overflow, and the shape of that bug — subtracting one file-controlled
number from another and indexing with the result — is the specific thing being avoided rather than a
generic gesture at safety.

```
bytes ──crt::parse──► (FlashImage, header_name)
                            │
                            ├─reserved::verify──► () | Unsupported(ReservedRegionMismatch)
                            ├─reserved::read_ef_name──► String
                            └─directory walk (directory::decode per slot)
                                   │
                                   └──image::read_program──► Program
                                            │
                                            ▼
                                       CartBuilder
```

## Constitution check

| Rule | Applies? | Compliance note |
|---|---|---|
| `testing.md` (tests first, coverage) | yes (always) | Phase 4 emits one failing test per AC before any implementation; the red-green loop drives them. Test placement follows the conventions machine block: `crates/stidcart-core/tests/<area>.rs`, collected by cargo without registration. |
| `no-downgrade.md` (preserve features) | yes (always) | Nothing in the write path changes behaviour. Decoders are added *alongside* encoders; no encoder signature or output changes. `CartBuilder` gains read accessors only — no field is removed and no default changes. The one produced-byte change is the version bump (row below), which is the versioning rule, not a downgrade. |
| `no-overengineering.md` (minimal scope) | yes (always) | No new crate, no new dependency, no command-line surface (spec Out of scope), no cartridge-entry support, no editing. Bounds checks are added only where a file-supplied number reaches an index — not as blanket defensive code. |
| `refinement-protocol.md` (ambiguous edits) | yes (always) | Both material branch points (hidden entries, firmware mismatch) went to the maintainer before planning. Two spec amendments needed by this plan were written into `spec.md` explicitly rather than planned around silently — see Risks. |
| `versioning.md` (bump in first commit) | yes (always) | `feat/` branch → minor bump. Verified against the branch base rather than assumed: `git merge-base HEAD main:Cargo.toml` reads `0.2.0`, and the branch's two commits so far are `docs:` with no bump. **0.2.0 → 0.3.0 lands in the first code commit.** This is the standing rule and is distinct from D-008, which rejected rolling the version *back*; it changes the default container header name and therefore the bytes of any cart built without an explicit one. No fixture is affected — parity pins the header to the reference tool's string — and AC-8 means a loaded cart keeps the name it arrived with rather than being re-stamped. |
| `ac-convention.md` — EVAL coverage | no | No path here can be silently wrong in the sense the rule means: there is no scorer, ranker, or generated output. Every result is a byte comparison against a frozen image, pass or fail. Matches the conventions block's `eval: n/a`. |
| `MAP.md` invariant: byte parity gates the write path | yes | This work-item produces no new bytes. The four fixtures are unchanged and the existing parity test must still pass untouched; AC-2 adds the inverse direction over the same files. |
| `MAP.md` invariant: the oracle decays | yes | The read path takes no new dependency on the reference tool. Its evidence is the round trip, which runs from checked-in fixtures alone — the same property AC-3 already pins for the write path. |
| `MAP.md` invariant: `firmware/` is zlib, everything else GPL | yes | No file under `firmware/` is touched. Verification compares the image's reserved regions against the already-embedded blobs; no blob is copied into a Rust source file. |
| `MAP.md` invariant: core holds no global state and no I/O policy | yes | `load` takes a byte slice and returns a value. No filesystem access, no statics. Tests read fixture files themselves, as the existing parity test does. |
| `MAP.md` invariant: the crate version lands in every cart | yes | See the versioning row. The read path never re-stamps the header name (AC-8), which is what keeps a version bump from mutating carts that merely pass through the tool. |
| `MAP.md` invariant: format facts come from the vault spec | yes | Every offset needed already exists as a `pub const` derived from that spec; this plan adds no new offset. The one place the spec is silent — how the older directory format is discriminated — is why AC-7 was amended rather than guessed at by reading the reference C. |
| `MAP.md` invariant: read-path divergence is per behaviour (D-007) | yes | Directly implemented: AC-3 accepts the empty directory, AC-5 validates the packet marker and total-length together, AC-6 keeps the format-mandated rejection, AC-7 and AC-10 return unsupported rather than misparsing. |
| `.claude/rules/lcd-conventions.md` — branch and commit discipline | yes | Work is on `feat/read-path`; never `main`. Conventional commits, one per meaningful step. |
| `.claude/rules/lcd-conventions.md` — gate command | yes | `cargo fmt --all --check && cargo clippy --workspace --all-targets --all-features -D warnings && cargo test --workspace` must be green before the audit. |

## Cross-path behavior matrix

> Degenerate by construction: **every AC in this spec is `surfaces: none`**, so there are no
> non-`none` (AC × surface) pairs and no behavior exposed through two paths.
>
> **The audit gate is therefore vacuous for this work-item, and that should not be mistaken for
> coverage.** `audit-crosspath.sh` skips every `none` surface outright, so it will emit an empty
> table and exit 0 without checking anything. (The write-path audit BLOCKed on its *CLI* rows, not
> its `none` ones — its own report records that the `none` criteria sit "outside the cross-path
> contract".) The real gates here are T-final — the full fmt + clippy + test suite — and the
> round-trip ACs, which are the only thing that can actually prove the read path inverts the write
> path. The table below is kept as documentation of where each AC's test lives, not because the
> script reads it.

Each test carries the literal `AC-N (none)`, matching this project's established
`/// AC-N (SURFACE): …` doc-comment style rather than the generator's bare-`AC-N` default for
`none` ACs. Both satisfy the phase-4 quality gate; the project style wins for consistency.

| AC | Surface | Path |
|---|---|---|
| `AC-1` | none | `crates/stidcart-core/tests/read.rs` |
| `AC-2` | none | `crates/stidcart-core/tests/roundtrip.rs` |
| `AC-3` | none | `crates/stidcart-core/tests/read.rs` |
| `AC-4` | none | `crates/stidcart-core/tests/roundtrip.rs` |
| `AC-5` | none | `crates/stidcart-core/tests/read.rs` |
| `AC-6` | none | `crates/stidcart-core/tests/read.rs` |
| `AC-7` | none | `crates/stidcart-core/tests/read.rs` |
| `AC-8` | none | `crates/stidcart-core/tests/read.rs` |
| `AC-9` | none | `crates/stidcart-core/tests/read.rs` |
| `AC-10` | none | `crates/stidcart-core/tests/read.rs` |
| `AC-11` | none | `crates/stidcart-core/tests/roundtrip.rs` |
| `AC-12` | none | `crates/stidcart-core/tests/read.rs` |

## Reused primitives

| Existing primitive | Path | Used for |
|---|---|---|
| `FlashImage::erased`, `write_at`, `as_bytes` | `crates/stidcart-core/src/image.rs` | Rebuilding the 1 MB image from packets; erased fill is what an omitted bank means |
| `FlashImage::read_program` | `crates/stidcart-core/src/image.rs` | Recovering an entry's bytes across the bank interleave — already the exact inverse AC-11 needs, already unit-tested both directions |
| `FlashImage::offset`, `Half` | `crates/stidcart-core/src/image.rs` | Mapping a packet's bank and load address back to an image offset |
| `petscii::petscii_to_ascii` | `crates/stidcart-core/src/petscii.rs` | Decoding entry names and the cartridge name. Already exists and is already tested against the encoder; **not** applied to the container header name (AC-9) |
| `crt::{MAGIC, HEADER_LEN, NAME_OFFSET, NAME_LEN, CARTRIDGE_EASYFLASH}` | `crates/stidcart-core/src/crt.rs` | Header validation and header-name extraction, from the same constants `write` uses |
| `reserved::{DIRECTORY_OFFSET, DIRECTORY_SLOTS, EAPI_*, EFNAME_*, BOOT_*, LOADER_*}` | `crates/stidcart-core/src/reserved.rs` | Locating and verifying each reserved region without restating an offset |
| `reserved::{write_eapi, write_boot, write_loader}` | `crates/stidcart-core/src/reserved.rs` | Building the expected reserved image to compare against (AC-10) — reuses the writers rather than re-deriving expected bytes |
| `firmware::{body, load_addr, EAPI, LAUNCHER_NRM, LOADER_NRM}` | `crates/stidcart-core/src/firmware.rs` | The embedded blobs a loaded image's reserved regions are checked against |
| `directory::{ENTRY_LEN, NAME_LEN, MAX_ENTRIES, TYPE_MASK, RESERVED_FLAG_BITS, HIDDEN_FLAG, EntryType, DirEntry}` | `crates/stidcart-core/src/directory.rs` | Slot decoding; `EntryType` already enumerates only the program types, so an unlisted type value falls straight through to unsupported |
| `CartBuilder` + `with_program` / `with_ef_name` / `with_header_name` / `programs` | `crates/stidcart-core/src/lib.rs` | The load target and the rebuild path; no new type introduced |
| `place::placement_order`, `Placer` | `crates/stidcart-core/src/place.rs` | Untouched, but load correctness depends on rebuild re-deriving the same positions — AC-2 is what proves it |
| Fixture set + `MANIFEST.md` | `fixtures/` | The frozen reference images, reused as-is; no new fixture is captured and `ndeft` is not invoked |
| cargo's built-in test harness | — | Integration targets under `crates/*/tests/`, collected without registration |

## Data model / Contracts

New public surface in `stidcart-core`:

```rust
pub fn read::load(bytes: &[u8]) -> Result<CartBuilder, ReadError>;

pub enum ReadError {
    NotACartridge,                                  // header magic or CRT id wrong
    Truncated { need: usize, have: usize },
    BadPacket { at: usize, problem: PacketProblem },
    OutOfBounds { at: usize, len: usize },          // AC-12
    DirectoryUnterminated,                          // AC-6
    Unsupported(Unsupported),                       // AC-7, AC-10
}

pub enum PacketProblem { Marker, LengthDisagrees, Geometry }

pub enum Unsupported {
    CartridgeEntry { type_value: u8 },
    HiddenEntry { name: String },
    ReservedRegionMismatch { region: &'static str },
}
```

`ReadError` implements `Display` + `std::error::Error`, matching `BuildError`'s existing shape.
`CartBuilder` gains `header_name()` and `ef_name()` accessors; `programs()` already exists. No
existing signature changes.

## File structure

> This section is the source of truth for the JOURNAL **EDIT BOUNDARY**. The red-green loop may
> only modify paths listed here; copy them into the work-item JOURNAL's EDIT BOUNDARY block.

### New files

```
crates/stidcart-core/src/read.rs          orchestrates container -> reserved checks -> directory -> CartBuilder; owns ReadError
crates/stidcart-core/tests/read.rs        AC-1, AC-3, AC-5..AC-10, AC-12 — loading and the refusals
crates/stidcart-core/tests/roundtrip.rs   AC-2, AC-4, AC-11 — load-then-rebuild byte identity over the fixtures
```

### Modified files

- `crates/stidcart-core/src/crt.rs` — add `parse` (header validation, packet walk into a `FlashImage`, header-name extraction) beside `write`
- `crates/stidcart-core/src/directory.rs` — add `decode` beside `encode`; classify end marker / program / cartridge / hidden
- `crates/stidcart-core/src/reserved.rs` — add reserved-region verification against the embedded firmware, and cartridge-name extraction
- `crates/stidcart-core/src/lib.rs` — declare `pub mod read`, re-export `ReadError`, add `header_name()` / `ef_name()` accessors
- `Cargo.toml` — version 0.2.0 → 0.3.0 in the first code commit (versioning rule)
- `docs/lcd/work/read-path/*` — tasks, journal, audit as the pipeline proceeds

## Risks & rejected alternatives

- **One self-contained `read.rs` holding the whole parse**: rejected — it would restate offsets that
  already exist as constants in the writing modules, which is how an encoder and decoder drift apart.
- **A separate `LoadedCart` type**: rejected — `CartBuilder` already is the description that produces
  an image, so returning it makes AC-2 a direct property instead of a claim about a conversion.
- **Carrying the image's own firmware through into the rebuild** (instead of refusing a mismatch):
  rejected by the maintainer; more faithful, but widens the description beyond programs and names and
  would let the tool emit firmware it never validated. Recorded in `spec.md` open questions.
- **Spec amendment, called out per the instruction not to contradict the spec**: AC-7 originally also
  promised a distinguishable refusal for the older directory format. The extracted format spec records
  that the reference implementation detects that format but not *how*, and deriving a discriminator by
  reading its C directly is what the format-facts invariant forbids. AC-7 now covers cartridge and
  hidden entries only, and Out of scope states that an older-format image is refused as malformed.
- **Second spec amendment**: Out of scope now also records that the container permits a data packet
  spanning two adjoining storage regions; both forms load, and a rebuild always emits the narrower
  form, so an image using the wider one recovers its contents without rebuilding to identical bytes.
  No fixture uses it, and refusing it instead would reject valid cartridges the format allows.
- **Re-placing entries and verifying they land where the file says**: not done. Spec Out of scope
  already accepts that a cartridge laid out by another placer recovers its contents without rebuilding
  identically, so a verification step would refuse files the spec says to accept.

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
