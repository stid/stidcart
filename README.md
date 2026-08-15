# stidcart

C64 cartridge tooling in Rust. Builds EasyFlash multicart images; target hardware is the
EasyFlash 3.

Successor to [ndefpack](https://github.com/stid/ndefpack) (Hannu Nuotio, 2012), which is a
knowledge base here rather than an upstream. No code is shared. The binary format was extracted
from ndefpack's C sources into a written spec before this repo existed — that spec is
`docs/format/easyflash-cart-format.md`, and it is the reference for everything in `stidcart-core`.

## State

Write, read and list, all for PRG entries only. It builds a cart from PRG files, loads one back
into the same builder state byte-identically, and shows what one contains. It does not edit a cart,
and it has no GUI. The order — write first — was deliberate; see below.

This section is the narrative; `docs/lcd/SPEC.md` is the enumerated capability index, with each row
citing the decision that put it there.

Describing a cart and rebuilding it are held to different standards, because they need to be:
`list` shows anything it can describe, including hidden entries, while loading additionally refuses
whatever it could not reproduce unchanged (a hidden entry, or an image whose firmware regions
differ from the blobs embedded here). Both refuse what neither can represent: cartridge entries and
entries confined to one half of a bank. Directories in the older EasyFS1 format are refused by both
paths: read as EasyFS2 every entry gets a corrupted bank, so there is no accurate description to
give (D-017).

## Layout

```
crates/stidcart-core/   library: format, placement, firmware. No I/O policy, no globals.
crates/stidcart-cli/    binary `stidcart`: the test harness, not a product.
firmware/               zlib-licensed C64-side blobs (see firmware/README.md)
fixtures/               ndeft-built carts, frozen, with the exact command line for each
```

## Why the write path first

`ndeft` is the only implementation known to produce images that boot on real EasyFlash hardware,
so it works as a differential oracle: build the same inputs with both, compare bytes. That oracle
**decays** — parity only applies where this tool is trying to produce identical output, and stops
applying the moment broader compatibility or custom firmware lands. Its value is highest at the
first commit, so it gets spent there.

The write path also has no bug-compatibility question. Reading someone else's cart does: ndefpack
has documented quirks in its loader (an empty directory rejected, the CHIP magic never checked,
wide 16k xbank carts unreadable) that a new implementation may or may not want to reproduce.
Building the write path first defers that decision instead of pre-empting it.

## Build and test

```bash
cargo test --workspace
cargo run -p stidcart-cli -- build -e "name" -o out.crt some.prg
cargo run -p stidcart-cli -- list out.crt
```

The parity test needs only the checked-in fixtures; `ndeft` does not have to be installed.

## Licensing

GPL-2.0-or-later (`COPYING`), because the core was written from ndefpack's GPL C sources and is a
derivative work. `firmware/` is zlib and stays that way — that boundary is load-bearing.
