# Plan: Editing an existing cart

> **Deep lane · Phase 2 of 6 (Plan).** This document owns **how**. Specify ≠ Plan: if you're
> about to write a user story or acceptance criterion, that belongs in `spec.md`.

**Slug:** `cart-edit`
**Spec:** [`spec.md`](./spec.md) · **Journal:** [`JOURNAL.md`](./JOURNAL.md)
**Created:** 2026-08-14
**Status:** draft <!-- draft | approved | implemented | superseded -->

---

## Goal

Give `stidcart` an `edit` command that reads a cart through the strict rebuild path, applies
index-addressed operations to its entry set, and writes the result back atomically — and give the
entry model the hidden flag it has been missing.

## Architecture

**The core keeps its one owned value.** `CartBuilder` already holds everything an edit needs — the
menu name, the container name and the program list — and `read::load` already returns one populated
from a file. So editing is not a new subsystem: it is the mutating half of an API that so far only
had the accumulating half. `CartBuilder` gains index-addressed setters (`push`, `remove`,
`set_name`, `set_hidden`, `replace_data`, `set_ef_name`, `set_header_name`) beside the existing
consuming `with_*` methods, which stay exactly as they are. Nothing about `build()`/`build_image()`
changes, which is what keeps AC-30 true for free: an edited cart is emitted by the same code path a
fresh build uses, so "the bytes a fresh build would produce" is not a property to implement but the
only thing the code can do.

The setters index the program vector directly and panic on an out-of-range index, exactly as `Vec`
indexing does. The alternative — a typed `EditError` per setter — was rejected because the CLI has
to bounds-check indices anyway to produce AC-24's message ("no entry at index 7; the cart holds 4"),
and a second validation inside the core would exist only to be unreachable. A separate editable
document type was the other candidate and is rejected in D-019's neighbourhood for the reason the
MAP states: the library is one owned value, and two vocabularies for one cart is the shape that made
the predecessor's core unreusable.

**The hidden flag needs the model, not the format.** `directory.rs` already encodes bit 7 on write
(`encode`, line 122) and decodes it on read (line 111); `read::inspect` already reports it and the
listing already prints it. The only thing missing is a place to keep it between reading and writing:
`Program` gains a `hidden: bool`, `build_image` writes `program.hidden` where it currently hardcodes
`false`, `load` copies the flag out of each directory entry, and the `Unsupported::HiddenEntry`
refusal at `read.rs:344` is deleted — it exists solely because the model was lossy (D-020). No
format fact changes, so `docs/format/` is untouched; the flag is already specified there at line 119.

**The CLI grows one command with repeatable operation flags.** AC-27 forces this shape: several
operations in one invocation share one index space, resolved against the cart as given. Per-operation
subcommands (`stidcart edit add …`) would make each invocation see a different cart and the AC
unstateable. Argument parsing stays hand-rolled in the existing `while let Some(arg)` style — no
new dependency, and `edit`'s grammar is not more complex than `build`'s. Operations are collected in
the order given, then applied in three phases so index meaning never shifts underfoot: in-place
mutations first (rename / hide / show / replace, last one wins on a repeated index), then removals
by descending index, then additions appended in the order given.

**Writing is the CLI's job, and it is atomic.** The core returns bytes and decides nothing about
files (MAP invariant). The command computes the full byte vector before touching the filesystem, then
writes to a temporary file in the destination's directory and renames it over the destination. That
holds AC-24 for the in-place default: a cart is either the old one or the new one, never a truncated
prefix of the new one. Every refusal — a bad index, a cart that will not fit, an unreadable program
file, a cart `load` rejects — happens before that write.

**No recon.** This work introduces no dependency, no framework and no unfamiliar API; every primitive
it stands on is in this repo and was read while writing this plan. `lcd:recon` would return nothing
the code doesn't already say.

## Constitution check

| Rule | Applies? | Compliance note |
|---|---|---|
| `testing.md` (tests first, coverage) | yes (always) | `/lcd:test-gen` emits one failing test per (AC × surface) before any implementation; the red-green loop commits per red→green flip. Full `cargo test --workspace` before every commit per the project gate. |
| `no-downgrade.md` (preserve features) | yes (always) | The consuming `with_*` API stays and every existing caller compiles unchanged; `build` and `list` keep their current behaviour, `build` only gaining an optional marker. The one behaviour change — a cart with hidden entries is no longer refused for rebuilding — is a widening, and the user approved it explicitly (D-020). |
| `no-overengineering.md` (minimal scope) | yes (always) | No new dependency (arg parsing stays hand-rolled), no second placement mode, no reorder or extract operation, no error type for a bounds check the CLI already performs, no editable-document type beside the builder. |
| `refinement-protocol.md` (ambiguous edits) | yes (always) | Six forks were put to the user before anything was written (scope, placement, output target, hidden refusal, addressing, replace-in-scope, build marker); the answers are in `spec.md` and D-019/D-020. |
| `versioning.md` (bump in first commit) | **yes — action required** | Branch is `feat/`, so a minor bump is due: base `Cargo.toml` on `main` is `0.4.0` → `0.5.0`. The branch's first commit was docs-only, so the bump lands in the first implementation commit (task T-1). Verified no test hardcodes the version, and `parity.rs:54` sets `REFERENCE_HEADER_NAME` explicitly, so fixtures are unaffected. |
| `ac-convention.md` — EVAL coverage | no | No path here can be *silently* wrong: output is byte-exact images compared against frozen fixtures, never scored. `MAP.md` records EVAL as `n/a` for this project. |
| `MAP.md` invariant: byte parity gates the write path | yes | Nothing in the existing write path changes shape: `build_image` writes `program.hidden`, which is `false` for every program the current code constructs. AC-28 pins that a build marking nothing hidden produces the bytes it produced before. No fixture is regenerated by this work. |
| `MAP.md` invariant: the oracle decays | yes | `ndeft` has no equivalent edit command, so parity cannot cover `edit`. Its evidence is the round trip (AC-29) and the fresh-build equality (AC-30), both asserted against fixtures this repo already owns — not a new oracle. |
| `MAP.md` invariant: core holds no global state and no I/O policy | yes | The atomic temp-file-and-rename write lives entirely in `stidcart-cli`; the core still returns `Vec<u8>` and knows nothing about paths. |
| `MAP.md` invariant: the crate version reaches produced bytes | yes | See the versioning row — the bump changes the default container header name, which is why it is verified against the tests rather than assumed harmless. |
| `MAP.md` invariant: describing ≠ rebuilding (D-011) | yes | `edit` rides `read::load` (the rebuild path, with its preconditions); `list` keeps using `read::inspect` and keeps reporting stored positions. This work moves nothing between the two. |
| `MAP.md` invariant: read-path divergence is per behaviour (D-007) | yes | The retired refusal is not an ndefpack divergence being reproduced or fixed — it was this repo's own model gap. D-020 records that distinction, and that the firmware refusal, which is a genuine save-time transformation, stays. |
| `MAP.md` invariant: format facts come from `docs/format/` | yes | The hidden flag is already specified (`easyflash-cart-format.md:119`) and already implemented in `directory.rs`. No format fact is discovered or changed, so that file is not edited by this work-item. |
| `.claude/rules/lcd-conventions.md` (AC numbering, test placement, target stems) | yes | ACs claim AC-17..AC-30 and the "Next free" line is already bumped to AC-31. New test targets are `mutate` (core) and `edit` (cli) — single-word stems, unique across both crates, so `--test <stem>` resolves without `-p`. |

## Cross-path behavior matrix

| AC | Surface | Path |
|---|---|---|
| `AC-17` | CLI | `crates/stidcart-cli/src/main.rs:edit` |
| `AC-18` | CLI | `crates/stidcart-cli/src/main.rs:edit` |
| `AC-19` | CLI | `crates/stidcart-cli/src/main.rs:edit` |
| `AC-20` | CLI | `crates/stidcart-cli/src/main.rs:edit` |
| `AC-21` | CLI | `crates/stidcart-cli/src/main.rs:edit` |
| `AC-22` | CLI | `crates/stidcart-cli/src/main.rs:edit` |
| `AC-23` | CLI | `crates/stidcart-cli/src/main.rs:edit` |
| `AC-24` | CLI | `crates/stidcart-cli/src/main.rs:edit` |
| `AC-25` | CLI | `crates/stidcart-cli/src/main.rs:edit` |
| `AC-26` | CLI | `crates/stidcart-cli/src/main.rs:edit` |
| `AC-27` | CLI | `crates/stidcart-cli/src/main.rs:edit` |
| `AC-28` | CLI | `crates/stidcart-cli/src/main.rs:build` |

AC-29 and AC-30 are `none` and carry no row, per the convention.

## Reused primitives

| Existing primitive | Path | Used for |
|---|---|---|
| `read::load` | `crates/stidcart-core/src/read.rs:339` | Reading a cart into editable state; its preconditions *are* AC-25 and AC-26 — no refusal is written by this work |
| `read::inspect` + `render` | `crates/stidcart-core/src/read.rs`, `crates/stidcart-cli/src/main.rs:119` | The listing whose indices AC-18/AC-27 address; unchanged by this work |
| `directory::encode` / `decode` | `crates/stidcart-core/src/directory.rs:101,122` | The hidden bit on both sides — already implemented and unit-tested, only unused |
| `place::placement_order` + `Placer` | `crates/stidcart-core/src/place.rs` | The single placement path; AC-30 holds because nothing else exists |
| `CartBuilder::build_image` / `build` + `crt::write` | `crates/stidcart-core/src/lib.rs:141,198` | Emitting the edited cart — the save path is the build path |
| `BuildError::{TooManyEntries, OutOfSpace}` | `crates/stidcart-core/src/lib.rs:51` | The "does not fit" half of AC-24, already typed and formatted |
| `entry_name` | `crates/stidcart-cli/src/main.rs:179` | Naming entries added by `--add` (AC-17), same rule `build` uses |
| CLI test harness (`CARGO_BIN_EXE_stidcart`, `scratch()`, `fixtures()`) | `crates/stidcart-cli/tests/list.rs:12-27` | Driving the binary and giving each case its own directory; copied, not re-invented |
| Frozen fixtures | `fixtures/` | Real carts for the round trip (AC-29) and the refusal cases (AC-25/AC-26) |

## Data model / Contracts

**`Program`** gains one field:

```rust
pub struct Program { pub name: String, pub data: Vec<u8>, pub hidden: bool }
impl Program { pub fn with_hidden(mut self, hidden: bool) -> Self }   // `new` keeps its signature
```

**`CartBuilder`** gains mutators (all `&mut self`; index-addressed ones panic on an out-of-range
index, documented as `Vec`-indexing semantics):

```rust
pub fn push(&mut self, program: Program)
pub fn remove(&mut self, index: usize) -> Program
pub fn set_name(&mut self, index: usize, name: &str)
pub fn set_hidden(&mut self, index: usize, hidden: bool)
pub fn replace_data(&mut self, index: usize, data: Vec<u8>)
pub fn set_ef_name(&mut self, name: &str)
pub fn set_header_name(&mut self, name: &str)
```

**CLI grammar** (added to `USAGE`):

```
stidcart edit CART.crt [-o OUTPUT] [OPERATION...]
  -o OUTPUT         write here instead of over CART.crt
  --add PRG         append a program, named after its file stem
  --remove N        drop the entry at index N
  --rename N NAME   rename the entry at index N
  --replace N PRG   swap the program data at index N, keeping name and position
  --hide N          mark the entry at index N hidden
  --show N          clear the hidden mark on the entry at index N
  -e NAME           set the menu name
  --header NAME     set the container header name
stidcart build ... [--hide INPUT.prg]
  --hide INPUT.prg  mark that input hidden in the built cart
```

`build` addresses by input path because no indices exist before the cart does; `edit` addresses by
the index `list` prints. Indices are resolved against the cart as read, then applied in phases:
mutations (last wins per index), removals descending, additions appended.

## File structure

### New files

```
crates/stidcart-core/tests/mutate.rs   AC-29, AC-30 — the round trip through the mutating API and
                                       the equality with a fresh build of the resulting set
crates/stidcart-cli/tests/edit.rs      AC-17..AC-28 — the edit command driven as a binary, and the
                                       marker `build` gains beside it
```

### Modified files

- `crates/stidcart-core/src/lib.rs` — `Program.hidden` + `with_hidden`; `CartBuilder` mutators;
  `build_image` writes `program.hidden` instead of `false`
- `crates/stidcart-core/src/read.rs` — `load` carries the hidden flag into each `Program`; the
  `Unsupported::HiddenEntry` variant, its refusal and its message are removed
- `crates/stidcart-core/tests/read.rs` — lines 164 and 439 match on that variant, so they change in
  the same commit: both flip to asserting that a hidden-carrying cart loads
- `crates/stidcart-cli/src/main.rs` — the `edit` command, its parsing and its atomic write;
  `build` gains `--hide`; `USAGE` updated
- `Cargo.toml` — version `0.4.0` → `0.5.0` (first implementation commit)
- `README.md` — the State section currently says "It does not edit a cart"
- `docs/lcd/work/cart-edit/` — JOURNAL upkeep as the work proceeds
- `docs/lcd/SPEC.md` — folded at closeout by `lcd:reconcile`, not during the build

## Risks & rejected alternatives

- **A typed error per setter instead of `Vec`-style panics**: rejected — the CLI must bounds-check
  to produce AC-24's message anyway, so the core's copy would be unreachable validation. The panic is
  documented and reachable only from a caller that skipped its own check.
- **Direct write instead of temp-file-and-rename**: rejected — the in-place default means a failure
  mid-write would truncate the user's only copy of the cart. The cost is that an in-place edit
  replaces the file rather than rewriting it, so permissions and inode change; acceptable for a
  command-line harness, and worth naming in review.
- **`clap` for the new grammar**: rejected — one more dependency for a parser the existing 20 lines
  already model, and it would reshape `build`'s and `list`'s parsing along with it.
- **Sub-subcommands (`stidcart edit add …`)**: rejected — AC-27 needs several operations sharing one
  index space in one invocation, which per-operation invocations cannot express.
- **Preserving the input cart's placement**: rejected in D-019; it needs a second placer no fixture
  can pin.
- **The `--hide` asymmetry** (path at build time, index at edit time) is deliberate: indices do not
  exist until the cart does. If it reads badly in review, the alternative is naming entries by name
  at build time, which breaks on two inputs with the same stem.

---

<!--
Quality checks before approving this plan:
  - Every spec acceptance criterion maps to something in this plan ✅ (AC-17..AC-28 in the matrix,
    AC-29/AC-30 in Architecture + the new core test file)
  - Constitution check has rows beyond the bundled rules ✅ (7 MAP invariants + lcd-conventions)
  - Matrix has one row per (AC × non-`none` surface) pair ✅
  - Reused primitives non-empty ✅
  - File structure concrete enough for tasks.md and the EDIT BOUNDARY ✅
-->
