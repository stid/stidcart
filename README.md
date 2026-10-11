# stidcart

C64 cartridge tooling in Rust. Builds EasyFlash multicart images; target hardware is the
EasyFlash 3.

Successor to [ndefpack](https://github.com/stid/ndefpack) (Hannu Nuotio, 2012), which is a
knowledge base here rather than an upstream. No code is shared. The binary format was extracted
from ndefpack's C sources into a written spec before this repo existed — that spec is
`docs/format/easyflash-cart-format.md`, and it is the reference for everything in `stidcart-core`.

## State

Write, read, list and edit. It builds a cart from PRG files and cartridge images, loads one back
into the same builder state byte-identically, shows what one contains, and changes one in place —
adding, removing, renaming and replacing entries, hiding them, requesting 64 KB alignment, and
setting either name. It has no GUI. The order — write first — was deliberate; see below.

Entries can be programs or cartridges. A cartridge input is recognised by its content: generic 8k,
16k and Ultimax images, Ocean (up to 512 KiB), and EasyFlash xbank in all three modes. An xbank is
bounded by the flash, not by a flat 512 KiB, so one wider than the reference tool reads is accepted
(D-022). Cartridges are placed first, and programs fill the gaps between them as entries confined
to one half of a bank, the way the reference placement does. An Ocean entry puts the whole image in
Ocean mode, derived from the entries rather than declared (D-024); only one is allowed.

Editing saves through the same placement the build path uses, so an edited cart is the cart a fresh
build of the resulting entries would have produced, not a patch of the file that arrived: entries
the request never mentioned can move.

This section is the narrative; `docs/lcd/SPEC.md` is the enumerated capability index, with each row
citing the decision that put it there.

Describing a cart and rebuilding it are held to different standards, because they need to be:
`list` shows anything it can describe, while loading additionally refuses whatever it could not
reproduce unchanged (an image whose firmware regions differ from the blobs embedded here). Hidden
entries pass both: the model carries the flag, so they load, edit and save back intact (D-020).
Both refuse what neither can represent, which is now only the type `0x1A` EasyFlash entry the
format calls a subdirectory: the reference tool refuses it too, so nothing says what its layout is.
The subdirectory *bit* on an xbank entry is carried through every operation, as ndefpack does, and
`list` marks it `S` (D-029). Directories in the older EasyFS1 format are refused by both paths: read as EasyFS2 every
entry gets a corrupted bank, so there is no accurate description to give (D-017).

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
wide 16k xbank carts unreadable) that a new implementation may or may not want to reproduce. This
one reads wide xbank carts (D-022).
Building the write path first defers that decision instead of pre-empting it.

## Build and test

```bash
cargo test --workspace
cargo run -p stidcart-cli -- build -e "name" -o out.crt some.prg
cargo run -p stidcart-cli -- list out.crt
cargo run -p stidcart-cli -- edit out.crt --rename 0 "new name" --add another.prg
```

The parity test needs only the checked-in fixtures; `ndeft` does not have to be installed.

## Licensing

GPL-2.0-or-later (`COPYING`), because the core was written from ndefpack's GPL C sources and is a
derivative work. `firmware/` is zlib and stays that way — that boundary is load-bearing.
