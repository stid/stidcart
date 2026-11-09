# MAP — stidcart

> **Logical-organization guardrails.** Read this FIRST on any cold start. It exists so the AI (and
> you) don't re-derive the project's shape every session — and don't reinvent a place that already
> exists. Update it in the same commit as any change that makes it wrong. Keep under ~150 lines.

**Generated:** 2026-08-10 · **Last refined:** 2026-11-08

---

## Shape

A Cargo workspace producing EasyFlash cartridge images for the Commodore 64, targeting EasyFlash 3
hardware. Two crates: a pure library that knows the binary format and a thin binary that drives it.
The split is deliberate and load-bearing — the library owns no global state and makes no I/O policy
decisions, which is what keeps a future GUI (toolkit undecided) a matter of adding a crate rather
than restructuring. Successor to ndefpack; no code is shared, and the binary format was extracted
to a written spec before this repo existed.

What it does *now* is not restated here — `SPEC.md` is the current-state capability index, and
this MAP is structural. Read `SPEC.md`'s surface section before scoping a change to that surface.

## Zones / Components

> One row per top-level area. "Don't put here" is the guardrail that prevents sprawl.

| Zone | Path | Responsibility | Don't put here |
|---|---|---|---|
| core | `crates/stidcart-core/` | Flash image model, EasyFS2 directory encoding **and decoding**, cartridge-image derivation (`cartridge.rs`), PETSCII, placement, CRT container **both directions**, firmware embedding and verification, the shared listing values (`listing.rs`, D-032), classifying an input file as program or cartridge (`Entry::from_input`, D-034), entry metadata, the cart's ordered folder list and the hidden `STIDCART.META` entry's layout (`meta.rs`, D-054, D-055, D-063) | Global mutable state, file dialogs, CLI arg parsing, anything that decides *policy* about I/O, the TOML import (`stidcart-cli`) |
| cli | `crates/stidcart-cli/` | Argument parsing, file reading/writing, human-facing output. The test harness for the core, not a product. A library (`lib.rs`, `run`) plus a thin binary, so the GUI's tests can use `edit`/`build` as the parity oracle (D-035) | Format knowledge, placement rules — those belong in core |
| gui | `gui/` (frontend), `gui/src-tauri/` (crate `stidcart-gui`) | The desktop app: Tauri 2 + Svelte 5 (D-030, D-031), its **own Cargo workspace** excluded from the root, so the root gate never compiles webkit. Rust side reads files, holds the open cart as a `Document` (a stack of saved images with undo, D-035), writes saves, owns the native menu, and maps `listing` to a serde DTO; frontend renders and runs the unsaved-changes guard. `src/lib/host.ts` is the only Tauri importer. The folder tree's moves are pure functions in `src/lib/tree.ts`, and rows drag by pointer events in `src/lib/rowDrag.ts` (Tauri's native drop stays on for Finder files, D-061). Dev-depends on `stidcart-cli` for parity tests only | Format knowledge or listing values (core); serde in core; a second copy of a value `listing` already computes |
| firmware | `firmware/` | zlib-licensed C64-side blobs (EasyLoader, EAPI, boot code), embedded as data, and the unchanged ndefpack 1.0.1 source they were assembled from (`src/`, never built here) | Any Rust source, anything GPL. The licence boundary runs through this directory |
| menu | `menu/` | Our own C64 menu (D-037): GPL 6502 source in `src/`, assembled by `build.sh` (64tass) into the committed `menu.bin` (exactly the 8 KiB loader slot) and `menu.lbl` (VICE labels the boot tests break on). The core embeds `menu.bin`, so `cargo` never runs the assembler | Rust, zlib code, the launcher or EAPI (those stay EasyLoader's, in `firmware/`), anything the core needs to assemble at build time |
| fixtures | `fixtures/` | Frozen `ndeft`-built carts, their input PRGs, and the exact command line that produced each | Generated output, scratch carts, anything not reproducible from a recorded command |
| tests | `crates/*/tests/` + inline `#[cfg(test)]` | Cargo collects both automatically; no registration step | — |
| docs | `docs/lcd/` | The LCD artifact root (D-014): this MAP, DECISIONS, the SPEC capability index, per-work-item folders under `work/<slug>/`, and the vendored audit scripts in `ci/` | Format documentation (that's `docs/format/`); prose that duplicates `README.md` or `SPEC.md`; anything non-LCD directly in `docs/lcd/` — the tree is the artifact root, not a general docs folder |
| format | `docs/format/` | The EasyFlash binary format spec (D-018) — the source for every offset in `stidcart-core`, and the only place a format fact is stated | Anything about *this tool's* behaviour or decisions (`SPEC.md`, `DECISIONS.md`); prose that isn't the format itself. `docs/` holds these two subtrees and nothing loose |

## Entry points

- `cargo run -p stidcart-cli -- build -e "name" -o out.crt some.prg` → `crates/stidcart-cli/src/main.rs`
- `cargo run -p stidcart-cli -- list out.crt` → same file; prints the directory as the image records it
- `npm run tauri dev` in `gui/` → the desktop viewer (`gui/src-tauri/src/lib.rs`, `gui/src/App.svelte`); `docs/lcd/ci/gui-gate.sh` is its gate
- `menu/build.sh menu/` → reassembles the menu into `menu.bin` + `menu.lbl` (64tass); commit both with any change to `menu/src/`
- `cargo test --workspace` → the full suite, including the parity test against the frozen fixtures
- Library consumers start at `crates/stidcart-core/src/lib.rs`

## Surfaces in use

`CLI` — the `stidcart` binary. `RENDER` — two renderers of one `listing::Listing` (D-032): the `list`
formatter (`render()` in `crates/stidcart-cli/src/main.rs`, split from the printing so tests assert
on the string) and the GUI (`gui/src/lib/Listing.svelte` and its sibling components). No HTTP, MCP, or DB surface. `EVAL` is
`n/a`: output is byte-exact images compared against frozen fixtures, never scored.

## Invariants (always-true rules for THIS project)

> Recurring rules that hold across all work here (LCD principle: don't relitigate them each
> time). The Deep lane's Constitution check and the red-green loop honor these.

- **Byte parity against the frozen fixtures gates the write path.** Output changes only when a
  commit says it meant to, and updates the fixtures in the same commit.
- **The oracle decays.** Parity applies only where stidcart reproduces `ndeft`. Anything ndefpack
  cannot do needs its own evidence — a boot in VICE, or a hardware test.
- **`firmware/` is zlib, everything else is GPL-2.0-or-later.** That includes `menu/`, our own menu. No code crosses that line.
  The one other exception is the bundled fonts in `gui/src/fonts/`, which are OFL-1.1 font files, not code (D-047).
- **The core holds no global mutable state and no I/O policy.** This is the specific defect that
  made ndefpack's core unreusable; it is not being reproduced.
- **The crate version is written into every cart** as the default CRT header name, so a version
  bump changes produced bytes.
- **Format facts come from `docs/format/easyflash-cart-format.md`** (D-018), which records which
  ndefpack behaviours are format-mandated and which are implementation artifacts. Re-deriving from
  the C risks inheriting a quirk by accident. A correction lands in that file first, in the same PR
  as the code it justifies — the point of it being in the tree is that a reviewer can see both.
- **Describing an image and rebuilding it are held to different standards** (D-011). One directory
  walk, two views: it refuses only what it cannot *describe*, and `load` layers the rebuild
  preconditions on top. A refusal justified by "the builder cannot express this" belongs to the
  rebuild path alone — applied to a listing, it fails on exactly the carts worth inspecting.
- **Read-path divergence is decided per behaviour, never by a blanket rule** (D-007). The format
  spec's "behaviours to decide on" list classifies each one; follow that classification rather than
  reproducing or fixing the set wholesale. A quirk the spec calls an implementation artifact gets
  fixed; one it calls format-mandated stays.

## Conventions pointer

Toolchain, the gate command and branch/commit discipline live in `CLAUDE.md`. The machine-readable
adapter the LCD suite reads — artifact root, test placement and discovery, scoped/bail/single-test
commands, constitution notes — is `.claude/rules/lcd-conventions.md`. Decisions log:
`DECISIONS.md`. Current behaviour: `SPEC.md`.

This MAP is *structural* — where things live and what is always true. It is not a spec of current
behaviour (`SPEC.md`) and not the human-facing introduction (`README.md`).
