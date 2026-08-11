# Plan: Build a cartridge image from program files

> **Deep lane · Phase 2 of 6 (Plan).** This document owns **how**. Specify ≠ Plan: if you're
> about to write a user story or acceptance criterion, that belongs in `spec.md`.

**Slug:** `write-path-skeleton`
**Spec:** [`spec.md`](./spec.md) · **Journal:** [`JOURNAL.md`](./JOURNAL.md)
**Created:** 2026-08-10
**Status:** draft <!-- draft | approved | implemented | superseded -->

---

## Goal

A library that turns a list of programs into the bytes of an EasyFlash `.crt` image, and a thin
command that writes those bytes to a file, verified byte-identical to `ndeft`'s output for the same
inputs.

## Architecture

`stidcart-core` exposes one owned value, `CartBuilder`, that accumulates programs and produces
`Vec<u8>` — the finished `.crt` bytes. It performs no file I/O at all: the caller reads the program
files and writes the result. That is the whole architectural bet. ndefpack's core is a global
singleton (`main_flash_data`, `main_flash_efs`, `main_flash_state` are file-scope arrays) with a
push-only `ui.h` callback seam, which is why nothing can reuse it — one document per process, no
threading, and every diagnostic has to go through a global. Keeping the builder owned and I/O-free
is what makes adding a GUI later a matter of adding a crate.

Internally the work splits along the format's own seams, each module owning one layer:

```
CartBuilder::build()
  ├─ place      decide bank/offset for each program (weight-sorted, largest first)
  ├─ image      1 MB array; offset = (bank << 13) | (romh ? 1<<19 : 0)
  │   ├─ reserved   bank-0 ROMH map: directory, EAPI, EF-Name, boot
  │   ├─ firmware   embedded .prg blobs, load-address aware
  │   └─ directory  24-byte EasyFS2 entries (petscii names, flags, geometry)
  └─ crt        container: 64-byte header + CHIP packets, all-0xFF blocks skipped
```

**The CRT header name is a caller-supplied field, not the crate version.** This is the one place
byte parity would be impossible otherwise: `ndeft` writes `PACKAGE_STRING` ("ndefpack 1.0.1") into
the 32-byte name at offset `0x20` of every EasyFlash cart, and `-e` does *not* change it — `-e` sets
the EF-Name, which lives inside the flash image at `0x81b08` behind an `"ef-nAME:"` magic. They are
two different strings in two different places and are easy to conflate. So `CartBuilder` takes both
independently: the CLI defaults the header name to `stidcart <version>`, and the parity test sets it
to `ndefpack 1.0.1`. The divergence becomes an explicit argument rather than a reason the test can
never pass.

All non-`none` ACs land on one path — the `build` command in `crates/stidcart-cli/src/main.rs` —
so there is no cross-surface matrix to maintain (section omitted below, per the template rule).

Rejected: a mutable global image mirroring ndefpack — it is the specific defect this project exists
to not inherit. Rejected: streaming the output through a `Write` sink — premature for a 1 MB buffer,
and returning bytes makes the parity assertion a one-liner. Rejected: deriving the header name from
the crate version with no override — it makes AC-2 unsatisfiable by construction. Rejected: a trait
abstraction over cartridge formats — there is exactly one format today (`no-overengineering.md`).

## Constitution check

| Rule | Applies? | Compliance note |
|---|---|---|
| `testing.md` (tests first, coverage) | yes (always) | Every AC gets its failing test before implementation; the red-green loop drives them. Cargo collects both integration and inline tests with no registration, so there is no false-green trap to guard against. |
| `no-downgrade.md` (preserve features) | yes (always) | Vacuous here and worth saying so plainly: greenfield work-item, no existing feature to reduce. The rule becomes live the moment the read path lands. |
| `no-overengineering.md` (minimal scope) | yes (always) | PRG entries only, normal mode only, no read path, no format abstraction, no GUI. The placer implements the top-space interleaved case that PRG-only builds actually reach, not the full gap-filling algorithm. |
| `refinement-protocol.md` (ambiguous edits) | yes (always) | Spec open questions were all closed by measuring `ndeft` before this plan; any later change to them restates first and amends `spec.md` explicitly. |
| `versioning.md` (bump in first commit) | yes (always) | `feat/` branch → minor bump, `0.1.0` → `0.2.0` in `Cargo.toml`. Verified against the branch base: the root commit set `0.1.0`. The bump lands in the next commit rather than the first, because this branch's first commit was the spec; noted rather than backdated. |
| `ac-convention.md` — EVAL coverage | no | No scored, ranked, recommended or model-generated output anywhere. Output is a deterministic byte string checked against a frozen reference; the failure mode is "wrong bytes", which an equality assertion catches exactly. An eval harness would measure nothing an assertion does not. |
| `MAP.md` invariant: byte parity gates the write path | yes | AC-2/AC-3 are the gate; fixtures are frozen with their command lines so the check outlives `ndeft`. |
| `MAP.md` invariant: the oracle decays | yes | This work-item is the deliberate front-loading. AC-4 (boot on an emulator) is the independent evidence that does not depend on the oracle. |
| `MAP.md` invariant: `firmware/` is zlib | yes | Blobs are embedded as data via `include_bytes!`. No 6502 source is written, modified or ported; nothing crosses into GPL. |
| `MAP.md` invariant: core has no global mutable state | yes | `CartBuilder` owns the image and the entry list; no `static mut`, no thread-local, no singleton. |
| `MAP.md` invariant: the crate version lands in every cart | yes | Made explicit — see Architecture. The default is the crate version, read from one place. |
| `MAP.md` invariant: format facts come from the vault spec | yes | Every offset and field below traces to `reference/easyflash-cart-format.md`, which records which `ndefpack` behaviours are format-mandated and which are artifacts. |
| `MAP.md` invariant: read-path divergence stays open | yes | Nothing here reads a cart, so nothing here prejudges it. |

## Reused primitives

| Existing primitive | Path | Used for |
|---|---|---|
| Rust standard library + `cargo test` | — | No third-party dependencies. Nothing this work needs is worth a crate. |
| EasyFlash firmware blobs | `firmware/*.prg` | Embedded verbatim as the menu, boot code and flash API. Copied, never regenerated. |
| Extracted format specification | vault `reference/easyflash-cart-format.md` | The source of every offset, bit layout and placement rule used below. |
| `ndeft` as a fixture generator | ndefpack `build/ndeft` | Produces the reference images once, at fixture-capture time. Never linked, never invoked by the test suite. |

## Data model / Contracts

```
Program { name: String, data: Vec<u8> }      // data includes the 2-byte load address
CartBuilder {
    ef_name: String,          // -> flash 0x81b08, PETSCII, 16 B; defaults to "(no EF-Name)"
    header_name: String,      // -> CRT header 0x20, 32 B; defaults to "stidcart <version>"
    programs: Vec<Program>,
}
CartBuilder::build(&self) -> Result<Vec<u8>, BuildError>
BuildError { TooManyEntries, OutOfSpace { unplaced: usize }, ProgramTooLarge { name, size } }
```

Format constants this plan commits to, all from the spec note:

- image `1 MiB`, 64 banks × 8 KiB × 2 halves; `offset = (bank << 13) | (romh ? 1<<19 : 0)`
- directory `0x80000`, 256 slots × 24 B, 255 usable, last slot always the end marker
- EAPI `0x81800` (0x300) · EF-Name magic `0x81b00` + name `0x81b08` (16 B) · boot `0x81b18` (0x4E8)
- normal-mode menu loader at `0x00000` (bank 0 ROML, `$8000`)
- entry flags byte: type in bits 0–4, reserved `0x60` always set, hidden `0x80`
- PRG entry offset word: byte offset in bank, bit 13 selects ROMH

## File structure

> This section is the source of truth for the JOURNAL **EDIT BOUNDARY**. The red-green loop may
> only modify paths listed here; copy them into the work-item JOURNAL's EDIT BOUNDARY block.

### New files

```
crates/stidcart-core/src/lib.rs          public API: CartBuilder, Program, BuildError
crates/stidcart-core/src/petscii.rs      ASCII <-> PETSCII case swap, owned results
crates/stidcart-core/src/image.rs        1 MB flash image and the bank/half offset mapping
crates/stidcart-core/src/reserved.rs     bank-0 ROMH map; EF-Name magic and field
crates/stidcart-core/src/firmware.rs     embedded blobs, load-address-aware injection
crates/stidcart-core/src/directory.rs    entry types, flags, 24-byte entry encoding
crates/stidcart-core/src/place.rs        weight-ordered placement of interleaved PRG entries
crates/stidcart-core/src/crt.rs          CRT header and CHIP packet writer
crates/stidcart-core/tests/reserved.rs   AC-6, AC-8 — reserved layout and name encoding
crates/stidcart-core/tests/directory.rs  AC-7, AC-9 — entry encoding, end marker, entry cap
crates/stidcart-core/tests/placement.rs  AC-10 — bank-crossing continuation
crates/stidcart-core/tests/container.rs  AC-2 (container half) — header fields, CHIP packets, erased-block skipping
crates/stidcart-core/tests/parity.rs     AC-2, AC-3 — byte identity against frozen fixtures
crates/stidcart-cli/src/main.rs          `build` command: argument parsing and file I/O
crates/stidcart-cli/tests/cli.rs         AC-1, AC-5, AC-11 — command behaviour
fixtures/MANIFEST.md                     one row per fixture: the exact ndeft request that made it
fixtures/*.prg, fixtures/*.crt           frozen inputs and reference images
```

### Modified files

- `Cargo.toml` — version `0.1.0` → `0.2.0` (`versioning.md`, `feat/` branch)
- `docs/lcd/work/write-path-skeleton/JOURNAL.md` — STEPS/NOW kept current as work proceeds
- `docs/lcd/DECISIONS.md` — D-006 for the header-name-vs-EF-Name split

## Risks & rejected alternatives

- **Conflating the CRT header name with the EF-Name**: the single most likely cause of a parity
  failure that looks like a deep bug and is a one-line mistake. They are different strings at
  different offsets, and `-e` moves only one of them. Called out in Architecture; D-006 records it.
- **Assuming the boot region is the launcher blob**: it is 0x4E8 bytes of `0xFF` with an 86-byte
  launcher overlaid at `$FFAA`. Writing only the blob leaves ~1100 bytes wrong. Measured, not
  assumed.
- **A placer that is "close enough"**: placement is order-dependent, so a different but reasonable
  ordering silently produces a different — still bootable — image that fails parity. The skeleton
  reproduces `ndeft`'s ordering for the PRG-only case rather than inventing one; a better placer is
  a later, deliberate divergence.
- **Fixtures without their command lines**: unreproducible the first time anyone asks how one was
  built. `MANIFEST.md` carries the exact request per fixture.

---
