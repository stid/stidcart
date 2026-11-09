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
setting either name. The desktop app (`gui/`, below) does the same edits in a window and saves
the same bytes `edit` and `build` write. The order — write first — was deliberate; see below.

Entries can be programs or cartridges. A cartridge input is recognised by its content: generic 8k,
16k and Ultimax images, Ocean (up to 512 KiB), and EasyFlash xbank in all three modes. An xbank is
bounded by the flash, not by a flat 512 KiB, so one wider than the reference tool reads is accepted
(D-022). Cartridges are placed first, and programs fill the gaps between them as entries confined
to one half of a bank, the way the reference placement does. An Ocean entry puts the whole image in
Ocean mode, derived from the entries rather than declared (D-024); only one is allowed.

Editing saves through the same placement the build path uses, so an edited cart is the cart a fresh
build of the resulting entries would have produced, not a patch of the file that arrived: entries
the request never mentioned can move.

A new cart boots into our own menu (`menu/`, GPL), whether `build` makes it or the app's New Cart
does (D-064); `--menu easyloader` (or the app's menu picker) gives it EasyLoader, the embedded
third-party menu, in the same slot instead (D-037). A cart that is opened keeps the menu it has. It lists the
visible entries, scrolls, and starts programs and every cartridge kind except Ocean, including
programs whose bytes run into `$D000-$FFFF`, which EasyLoader refuses (D-056). It is refused on
carts it cannot run: one with an Ocean entry, or with a program that loads at `$D000` or above,
ends past `$FFFF`, or reaches `$D000` from below `$0150` (D-038, D-056). `list` and the desktop app say which menu a cart carries, and edits keep it. A cart
carrying an earlier build of our menu shows as "stidcart (older build)" and is saved with the
current one (D-049). A loader slot holding no menu the tool knows is replaced only when a menu is
chosen, with `edit --menu` or the app's picker (D-050). The reference tool knows nothing of this menu, so its evidence is VICE: boot tests start carts and read the
screen back (`menu/README.md`).

Each entry can carry a title (up to 48 characters), year (4), author (24), note (48) and folder
(16), set with `edit --title N TEXT` and its siblings, or for many entries at once with
`edit --import FILE.toml`. Our menu shows folders as folders; the field was called group before
D-060, and `--group` and the import key `group` still work. The cart keeps its own ordered list of
folders, which may be empty: `edit --new-folder NAME`, `--remove-folder NAME` (its entries stay, in
no folder) and `--move-folder NAME TO`; a folder stays when its last entry leaves, and `list`
prints the folders with their counts. Our menu lists them in that order and hides an empty one
(D-063). They are stored in one hidden `STIDCART.META` entry that both menus skip,
written only when some entry has metadata, so a cart without any is unchanged (D-054). `list`
prints them under the entry. Records that another tool's re-order or rename left pointing at the
wrong entry are dropped and counted, and a layout from a later stidcart is kept as stored (D-055).
The layout is in `docs/format/`, marked as a stidcart extension.

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
gui/                    desktop app: Tauri 2 + Svelte 5, its own Cargo workspace (gui/src-tauri)
firmware/               zlib-licensed C64-side blobs (see firmware/README.md)
menu/                   our own C64 menu, GPL 6502, committed assembled (see menu/README.md)
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
cargo run -p stidcart-cli -- list out.crt          # sizes as "652 bytes", "10.0 KB", "1 MB"
cargo run -p stidcart-cli -- list --bytes out.crt  # every size and the free space as exact bytes
cargo run -p stidcart-cli -- edit out.crt --rename 0 "new name" --add another.prg
cargo run -p stidcart-cli -- build --menu easyloader -e "name" -o theirs.crt some.prg
cargo run -p stidcart-cli -- edit out.crt --title 0 "Impossible Mission II" --folder 0 Games
cargo run -p stidcart-cli -- edit out.crt --move 3 0   # entry 3 to the top, the rest in order
cargo run -p stidcart-cli -- edit out.crt --new-folder Tools --move-folder Tools 0
cargo run -p stidcart-cli -- edit out.crt --import meta.toml   # [[entry]] tables, below
menu/build.sh menu/      # reassemble the menu after changing menu/src (64tass)
```

An import file names entries by their stored name, ignoring case. Fields left out are left alone,
an empty value clears one, and any problem (a name matching no entry or several, an unknown key, a
value too long) refuses the whole file before anything is written:

```toml
[[entry]]
name = "impossible missi"   # as `list` shows it: a 16-character name
title = "Impossible Mission II"
year = 1988
author = "Epyx"
folder = "Games"
```

The parity test needs only the checked-in fixtures; `ndeft` does not have to be installed. The
boot tests need VICE's `x64sc` and the menu's reassembly check needs 64tass. Without them those
tests skip, except under `STIDCART_REQUIRE_TOOLS=1`, which CI sets, where a missing tool fails.

## Desktop app

Open a `.crt` with Open…, ⌘O or by dropping it on the window. It shows the cart name, the
producing tool, every entry and the free space: the values `list` prints, from the same code
(D-032). Sizes read in bytes, KB or MB; hover one to see its exact byte count (D-044). A file it
can't describe shows the same reason `list` gives.

The window edits what `edit` edits:
- Drop programs or cartridge images on an open cart, or use Add… / Add Aligned….
- Rename an entry by clicking its name. Its "⋯" or right-click menu replaces, hides or shows,
  aligns or removes it.
- Click the cart name or the header name to change either one.
- Click an entry to show it in the inspector beside the list: its number, name, kind and size, and
  its title, year, author, note and folder. Click a field to change it, or pick its folder. Each
  change is one undoable edit, refused in `edit`'s words when invalid.
- New Folder (in the title bar, File ▸ New Folder ⇧⌘N, or a right-click on the list's empty space)
  adds an empty folder and opens its name. A folder's menu renames, moves or deletes it; deleting
  moves its entries out, never removes them.
- With any folder, the list shows the cart as our menu does: folder rows, each with its entries,
  then the entries in no folder. Drag an entry onto a folder, between rows, or to the strip at the
  bottom to take it out of its folder; drag a folder row to move the whole folder. An empty folder
  takes a drop too. The entry menu has Move to (every folder, the current one checked, Not in a
  folder, New Folder with Entry), Move up and Move down. After any of these the cart is saved in that order, so EasyLoader's flat list
  follows it too (D-060, D-061).
- Choose the menu the cart boots into, EasyLoader or stidcart, from the menu picker under the
  names. A cart our menu can't run (an Ocean entry, or one of the programs above) refuses stidcart
  and names the entry, as `edit --menu` does (D-042).
- New (⌘N) starts an empty cart.

Changes stay in memory until you save:
- Undo and redo cover every edit since the cart was opened, and an "Edited" dot marks unsaved
  changes.
- ⌘S replaces the file and ⌘⇧S saves as. A saved cart is byte-identical to what `edit` or `build`
  writes for the same request (D-035).
- Closing, quitting, opening another cart or New asks first.

A cart `edit` refuses, such as firmware that differs from the embedded copy, opens view-only with
the reason. When the only reason is a loader the tool doesn't know, the menu picker stays: choosing
a menu replaces the loader and makes the cart editable (D-050). The app needs Node 24, pinned in `gui/.nvmrc` (`nvm use` in `gui/` picks it up, and CI reads the same file), and, on Linux, the WebKitGTK packages the CI `gui` job installs.

```bash
cd gui && npm install
npm run tauri dev          # run the app
../docs/lcd/ci/gui-gate.sh # its gate: type-check, vitest, build, then clippy and tests
```

To use it day to day on your own Mac, `scripts/install.sh` builds `stidcart.app` into
`/Applications` and installs the `stidcart` command into `~/.cargo/bin`. Run it again after pulling
new work to update both; it asks you to quit the app first. Nothing is published or signed for
distribution.

## Licensing

GPL-2.0-or-later (`COPYING`), because the core was written from ndefpack's GPL C sources and is a
derivative work. `firmware/` is zlib and stays that way — that boundary is load-bearing.
The Geist fonts bundled in `gui/src/fonts/` are under the SIL Open Font License 1.1 (`OFL.txt` there).
