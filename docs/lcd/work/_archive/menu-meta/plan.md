# Plan: Entry metadata and groups

> **Deep lane · Phase 2 of 6 (Plan).** This document owns **how**. Specify ≠ Plan: if you're
> about to write a user story or acceptance criterion, that belongs in `spec.md`.

**Slug:** `menu-meta`
**Spec:** [`spec.md`](./spec.md) · **Journal:** [`JOURNAL.md`](./JOURNAL.md)
**Created:** 2026-11-08
**Status:** approved <!-- approved by delegation, 2026-11-08 -->

---

## Goal

Every entry carries an optional title, year, author, note and group, stored in one hidden
`STIDCART.META` program entry (D-054). It round-trips through `load` and every edit, is set by
`edit` flags, a TOML import and the window, and is listed by both renderers. A cart without
metadata stays byte-identical.

## Architecture

**The model carries metadata per entry, and META is never an entry.** `Program` and `Cartridge`
each gain a `meta: Meta` field beside `hidden`, so remove, rename, replace and re-order carry it
for free, as AC-29 carries the flag. `CartBuilder::entries()` never holds the META entry. Every
index the CLI checks, the window's `check_index` and the alignable rows stay right without being
touched. The META entry is made at build time by one private helper, `placed_entries()`. It
returns the user entries plus, when any entry has metadata (or a kept unknown-version META
exists), one hidden `Program` named `stidcart.meta` appended last. `build_image` and `free_space`
both place through it, so the D-027 exact free-space boundary holds on a cart with metadata, and
the slot count includes META. With no metadata the helper returns the entries unchanged, so the
build is the byte-for-byte build of today (AC-153). The name string is lower case because
`ascii_to_petscii` swaps case: the bytes on flash read `STIDCART.META`.

**The on-flash layout, version 1** (recorded in `docs/format/` first, D-018). The entry is an
ordinary PRG with load address `$4000`. Its payload (≤ 16384 bytes, so a copy fills at most
`$4000-$7FFF`) is laid out as follows:

- A 13-byte header: magic `STIDMETA` (raw bytes), the layout version, the group count G and the
  record count R, then a 2-byte offset of the record table from the payload start.
- A group table of G fixed 16-byte PETSCII names, NUL-padded as directory names are, so group n
  (1-based) sits at `table + (n-1)*16`.
- R records in ascending entry index, written only for entries that have metadata. Each record is
  a length byte (the whole record, so a reader can skip it), the entry index (its position among
  the entries, META excluded, which equals the directory slot because META is written last), the
  16 name bytes as the slot stores them, and a group number (0 = none). Then come title, year,
  author and note, each a length byte followed by that many PETSCII bytes.

A maxed record is 1+1+16+1+(1+48)+(1+4)+(1+24)+(1+48) = 147 bytes, so 254 maxed entries need
about 37 KB, which no 16 KB window holds. A set whose payload would pass 16384 bytes is refused
with a typed `BuildError::MetadataTooLarge` naming both sizes (D-055). The spec doesn't list this
refusal. It is added, not weakened, and a typical 200-entry cart (titles, years, a group) needs
about 8-10 KB. Group order is first appearance in entry order, so the user steers the folder
order. Groups match exactly, case included.

**Reading it back sits in the one directory walk** (D-011), so `inspect`, `describe` and `load`
agree. The walk reads program data for one entry only: the _last_ slot that is a hidden program
kind (interleaved or half-confined, because placement may confine META to a gap), whose 16 name
bytes are `STIDCART.META` NUL-padded and whose payload starts with the magic. Without the magic
it is an ordinary entry. The last candidate wins because stidcart always writes its own last. The
walk lifts META out of the entries, keeps its `DirEntry` for occupancy (`CartInfo::free_space`
replays it), and decodes the payload:

- **version 1, well-formed.** Each record attaches to the entry at its index when the record's
  16 name bytes equal the raw bytes of that slot's name. Any other record (index out of range, a
  name mismatch, a second record for one index) is dropped and counted (AC-154).
- **any other version, or a malformed version 1.** The bytes are kept verbatim with a reason
  (`KeptMeta`) and no entry gets metadata. `inspect` never refuses because of META.

When META isn't last (another tool re-ordered), the entries after it are numbered without it, so
`list` indexes equal `load` indexes. A save writes it last again.

**The unknown-version guard is in the core,** so `edit` and the window refuse in the same words.
`load` puts `KeptMeta { data, reason, names }` on the builder, where `names` are the entry names
as loaded. `build_image` refuses with `BuildError::MetadataKept` when the loaded names are no
longer a prefix of the current names (a remove, rename or re-order would make the records name
the wrong entries) or when any entry has metadata set. `CartBuilder::set_meta` refuses up front
with the same reason. Hide/show, align, replace, add, the cart and header names and a menu switch
all still save, with META rewritten byte for byte.

**Validation is in the core** (`meta.rs`). `Field` covers Title 48, Year 4, Author 24, Note 48
and Group 16. `Meta::set(field, value)` refuses an over-long value or one that doesn't map to
PETSCII and back unchanged (`petscii_to_ascii(ascii_to_petscii(v)) == v`, i.e. printable ASCII
`0x20-0x7E`), and the `Display` names the field. An empty value clears the field. The CLI and
the window pass the core's text through unchanged (AC-156, AC-159).

**Front ends.** `edit` gains `--title N TEXT`, `--year`, `--author`, `--note`, `--group` (in-place
ops on the index as given, like `--rename`) and `--import FILE.toml`. The import is resolved
against the cart as given: each `[[entry]]` table has `name` (matched against stored names,
ignoring case) and any of the five fields as strings (an integer `year` is accepted). Before
anything is applied, the import is refused naming the problem for an unknown key, a non-string
value, a name matching no entry or several, the same entry named twice in the file, or an invalid
value. TOML is parsed in the CLI (the core stays dependency-free) with `toml` 1.2.1 (current
stable, fetched 2026-11-08), into `toml::Table`, so no serde-derive is needed and the errors are
worded here. `list`'s `render` prints an indented line under any row with metadata
(`title: …  year: …  author: …  note: …  group: …`, present fields only), then
`metadata: N records dropped…` and `metadata: kept as stored…` lines only when they apply. The
window adds `title`/`year`/`author`/`note`/`group` to `RowDto`, plus `metaDropped`/`metaKept`
to `ListingDto`. `Listing.svelte` shows a row's title under its name and a Group column (only when
some row has a group, so AC-51's exact cells hold). Clicking a row selects it. A details panel
then shows the five fields as `EditableText`, each commit being one `Edit::SetMeta { index,
field, value }`, a single undoable step in the `Document`.

Rejected alternatives, in one sentence each:

- META as a real `Entry` in the builder: every index would be off by one, and every front end
  would need to filter it.
- Fixed-offset records: 147 B × 254 doesn't fit 16 KB, and padding wastes most of it.
- Dropping the name copy from records: D-054 settled that mismatch is detected by name.
- Parsing TOML with serde-derive: a second dependency for five string fields.
- Refusing malformed META on read: D-011 says describing refuses only what it cannot walk.

## Constitution check

| Rule                                                    | Applies? | Compliance note                                                                                                                                               |
| ------------------------------------------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `testing.md` (tests first, coverage)                    | yes      | failing tests for AC-153..AC-159 land in one commit before any code; red-green flips them one by one                                                          |
| `no-downgrade.md` (preserve features)                   | yes      | every existing edit, the listing columns and AC-51's exact cells are kept. The Group column appears only when used. Carts without metadata are byte-identical |
| `no-overengineering.md` (minimal scope)                 | yes      | no reorder op, no import at `build`, no GUI import, no C64-side reading (menu-folders). The import parser handles only `[[entry]]` tables                     |
| `refinement-protocol.md` (ambiguous edits)              | yes      | spec approved; the open forks (16 KB cap, kept-META rule) are decided here and in D-055                                                                       |
| `versioning.md` (bump in first commit)                  | deferred | the user does the version bump before the PR (brief, 2026-11-08); no commit here bumps it                                                                     |
| `commits.md` (atomic, conventional; per-flip)           | yes      | plan, format doc, tasks and tests each get their own commit, then one commit per red→green flip, each pushed with `git push origin HEAD:feat/menu-meta`       |
| `ac-convention.md` EVAL                                 | no       | output is bytes and text asserted exactly (MAP: EVAL n/a)                                                                                                     |
| MAP invariant: byte parity gates the write path         | yes      | META is written only when some entry has metadata; parity, the list goldens and a load-save of every fixture stay unchanged (AC-153)                          |
| MAP invariant: the oracle decays                        | yes      | `ndeft` knows nothing of META, so the evidence is round-trip tests and VICE boots of both menus showing META unlisted (AC-155)                                |
| MAP invariant: `firmware/` zlib, rest GPL               | no       | neither `firmware/` nor `menu/` is touched                                                                                                                    |
| MAP invariant: no global state / I/O policy in core     | yes      | `Meta` is a value on each entry. Encoding and decoding are pure; TOML and file reading stay in the CLI                                                        |
| MAP invariant: crate version reaches produced bytes     | no       | no bump in this item (deferred to the user)                                                                                                                   |
| MAP invariant: format facts from `docs/format/` (D-018) | yes      | the layout section, marked a stidcart extension, lands in its own commit before any code                                                                      |
| MAP invariant: describing vs rebuilding (D-011)         | yes      | `inspect` describes any META, kept or dropped records included, and refuses nothing new; only `build` refuses an unknown-version rewrite                      |
| MAP invariant: per-behaviour divergence (D-007)         | no       | no reference-tool quirk is involved                                                                                                                           |
| `.claude/rules/lcd-conventions.md` AC numbering         | yes      | AC-153..AC-159 were claimed at triage; the conventions line is bumped only if it isn't already (checked at tasks)                                             |
| CLAUDE.md: never commit to `main`; both gates           | yes      | work on `feat/menu-meta`; the root gate and `gui-gate.sh` run before every commit, `STIDCART_REQUIRE_TOOLS=1` at the end                                      |

## Cross-path behavior matrix

| AC       | Surface | Path                                      |
| -------- | ------- | ----------------------------------------- |
| `AC-156` | CLI     | `crates/stidcart-cli/src/lib.rs:--title`  |
| `AC-157` | CLI     | `crates/stidcart-cli/src/lib.rs:--import` |
| `AC-158` | RENDER  | `gui/src/lib/Listing.svelte`              |
| `AC-159` | RENDER  | `gui/src/lib/MetaPanel.svelte`            |

What must agree across surfaces is this. The field rules and their refusal text come from
`stidcart_core::meta` for `edit` and for the window. The listed values come from
`listing::Listing` for `list` and for the window (D-032). The window's saves equal `edit`'s for
the same values, held by the CLI-as-library oracle (D-035) in `gui/src-tauri/tests/meta.rs`, on the pattern of `parity.rs`.
AC-158's `list` renderer is `render()` in `crates/stidcart-cli/src/lib.rs`, tested in
`crates/stidcart-cli/tests/meta_cli.rs`. AC-153..AC-155 (`none`) are tested in
`crates/stidcart-core/tests/meta.rs` and `crates/stidcart-cli/tests/meta_boot.rs`.

**Test scope:** `crates/stidcart-core/tests/meta.rs`, `crates/stidcart-cli/tests/meta_cli.rs`, `crates/stidcart-cli/tests/meta_boot.rs`, `gui/src-tauri/tests/meta.rs`, `gui/src/lib/Listing.test.ts`, `gui/src/lib/MetaPanel.test.ts`

## Reused primitives

| Existing primitive                                      | Path                                  | Used for                                                                |
| ------------------------------------------------------- | ------------------------------------- | ----------------------------------------------------------------------- |
| `petscii::ascii_to_petscii` / `petscii_to_ascii`        | `crates/stidcart-core/src/petscii.rs` | field text on flash, and the mappable check                             |
| `Placer::place_all`, `Placer::occupy`                   | `crates/stidcart-core/src/place.rs`   | META placed by the normal path; its occupancy in `CartInfo::free_space` |
| `FlashImage::read_program` / `read_confined`            | `crates/stidcart-core/src/image.rs`   | reading META's payload in the walk                                      |
| `read_directory` (the one walk)                         | `crates/stidcart-core/src/read.rs`    | finding, lifting and decoding META for `inspect` and `load` alike       |
| `CartBuilder::replace` (keeps name, hidden)             | `crates/stidcart-core/src/lib.rs`     | also keeps metadata                                                     |
| `listing::describe` / `Row`                             | `crates/stidcart-core/src/listing.rs` | the five values per row, the dropped count, the kept note               |
| `edit`'s in-place op loop, `index()`, `write_replacing` | `crates/stidcart-cli/src/lib.rs`      | metadata flags as in-place ops; nothing written on refusal              |
| `Document::apply` (one step per edit, refuse unchanged) | `gui/src-tauri/src/document.rs`       | `Edit::SetMeta`                                                         |
| `EditableText`                                          | `gui/src/lib/EditableText.svelte`     | the five fields in the details panel                                    |
| contract goldens (`UPDATE_GOLDEN=1`)                    | `gui/src-tauri/tests/contract.rs`     | the DTO gains fields, so `two.json` and `document.json` are regenerated |
| `edit_scenario` parity harness                          | `gui/src-tauri/tests/parity.rs`       | the AC-159 save-equals-`edit` check                                     |
| VICE runner `support::vice`                             | `crates/stidcart-cli/tests/support/`  | AC-155 boots of both menus                                              |

## Data model / Contracts

```rust
// crates/stidcart-core/src/meta.rs
pub const META_NAME: &str = "stidcart.meta";   // PETSCII bytes on flash: "STIDCART.META"
pub const MAGIC: &[u8; 8] = b"STIDMETA";
pub const VERSION: u8 = 1;
pub const LOAD_ADDRESS: u16 = 0x4000;
pub const MAX_PAYLOAD: usize = 0x4000;
pub enum Field { Title, Year, Author, Note, Group }        // name(), limit(), from_name(), ALL
pub struct Meta { /* five Strings, private */ }           // get, set -> Result<(), MetaError>, is_empty
pub enum MetaError { TooLong { field, len, max }, Unmappable { field, ch }, Kept { reason } }
pub struct KeptMeta { pub data: Vec<u8>, pub reason: String, pub names: Vec<String> }

// core additions
Program.meta: Meta, Cartridge.meta: Meta, Entry::meta()
CartBuilder::set_meta(index, Field, &str) -> Result<(), MetaError>
BuildError::{TooManyEntries (counts META; message names it), MetadataTooLarge { size, max },
            MetadataKept { reason }}
CartInfo { .., metas: Vec<Meta> (aligned with entries), meta_entry: Option<DirEntry>,
           meta_dropped: usize, meta_kept: Option<KeptMeta> }
listing::Row { .., title, year, author, note, group: String }
listing::Listing { .., meta_dropped: usize, meta_kept: Option<String> }

// GUI wire contract (types.ts mirrors it)
Edit::SetMeta { index, field: String, value: String }      // `setMeta`
RowDto { .., title, year, author, note, group }
ListingDto { .., metaDropped: number, metaKept: string | null }
```

## File structure

### New files

```
crates/stidcart-core/src/meta.rs          Field, Meta, MetaError, KeptMeta, encode/decode of layout v1
crates/stidcart-core/tests/meta.rs        AC-153, AC-154, AC-155 (round trip, drops, kept, slots, cap)
crates/stidcart-cli/tests/meta_cli.rs     AC-156, AC-157, AC-158 (edit flags, import, list render)
crates/stidcart-cli/tests/meta_boot.rs    AC-155 VICE boots: neither menu lists META
gui/src-tauri/tests/meta.rs               AC-158/AC-159 Rust side: DTO values, SetMeta step, undo, refusal, parity
gui/src/lib/MetaPanel.svelte              the selected entry's five fields
gui/src/lib/MetaPanel.test.ts             AC-159 (RENDER) frontend
```

### Modified files

- `docs/format/easyflash-cart-format.md` adds a "stidcart extension: entry metadata" section (layout v1)
- `crates/stidcart-core/src/lib.rs` adds `meta` on entries, `set_meta`, `placed_entries`, the new build errors, and keeps meta on `replace`
- `crates/stidcart-core/src/cartridge.rs` adds `meta` field defaults in `from_crt` and `read_from`
- `crates/stidcart-core/src/read.rs` makes the walk find, lift and decode META, adds the `CartInfo` fields, and has `load` attach metas and kept META
- `crates/stidcart-core/src/listing.rs` adds the row fields, the dropped count and the kept note
- `crates/stidcart-cli/src/lib.rs` adds `--title/--year/--author/--note/--group`, `--import`, the render lines and the usage text
- `crates/stidcart-cli/Cargo.toml` adds the `toml` dependency
- `Cargo.lock`
- `gui/src-tauri/src/document.rs` adds `Edit::SetMeta`
- `gui/src-tauri/src/lib.rs` adds the DTO fields
- `gui/src-tauri/Cargo.lock` picks up `toml` through the CLI dev-dependency
- `gui/src-tauri/tests/contract.rs` changes no code; the goldens are regenerated
- `gui/src/lib/types.ts` mirrors the DTO and `Edit`
- `gui/src/lib/fixtures/two.json` is regenerated
- `gui/src/lib/fixtures/document.json` is regenerated
- `gui/src/lib/Listing.svelte` adds the title under the name, the Group column, row selection and the panel
- `gui/src/lib/Listing.test.ts` adds AC-158 (RENDER)
- `gui/src/lib/viewer.test.ts` needs fixture-shape updates only, if any
- `README.md` documents the new `edit` options and the import file
- `docs/lcd/DECISIONS.md` gets D-055
- `docs/lcd/MAP.md` adds `meta.rs` to the core zone
- `.claude/rules/lcd-conventions.md` updates the AC range line if needed
- `docs/lcd/work/menu-meta/` holds this item's artifacts

## Risks & rejected alternatives

- **The 16 KB cap refuses a large, heavily annotated cart.** It is accepted, because the C64 side
  copies the entry into `$4000-$7FFF` and the refusal names both sizes. A later layout could
  intern authors too; version 1 doesn't (D-055).
- **A user program named `stidcart.meta` that is hidden and starts with the magic** would be read
  as META. This is negligible, and the last-candidate rule keeps stidcart's own copy authoritative
  after a save.
- **Kept unknown-version META and renames.** Refusing rename and remove on such a cart is
  stricter than the user may expect. It is the only way not to write a META whose records name
  the wrong entries, and the refusal says so.
- **GUI live check** of selection and the panel is left to the user (brief). Component tests cover
  behaviour.
