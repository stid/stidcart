# The command line

`stidcart` builds, lists and edits EasyFlash cartridge images. `stidcart` with no arguments prints
every option. Installing it is in the [README](../../README.md#getting-it).

## Commands

```bash
stidcart build -e "name" -o out.crt some.prg
stidcart list out.crt          # sizes as "652 bytes", "10.0 KB", "1 MB"
stidcart list --bytes out.crt  # every size and the free space as exact bytes
stidcart edit out.crt --rename 0 "new name" --add another.prg
stidcart build --menu easyloader -e "name" -o theirs.crt some.prg
stidcart edit out.crt --title 0 "Impossible Mission II" --folder 0 Games
stidcart edit out.crt --move 3 0   # entry 3 to the top, the rest in order
stidcart edit out.crt --new-folder Tools --move-folder Tools 0
stidcart edit out.crt --import meta.toml   # [[entry]] tables, below
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

## What a cart can hold, and how edits behave

Write, read, list and edit. It builds a cart from PRG files and cartridge images, loads one back
into the same builder state byte-identically, shows what one contains, and changes one in place —
adding, removing, renaming and replacing entries, hiding them, requesting 64 KB alignment, and
setting either name. The desktop app (see `app.md`) does the same edits in a window and saves
the same bytes `edit` and `build` write.

Entries can be programs or cartridges. A cartridge input is recognised by its content: generic 8k,
16k and Ultimax images, Ocean (up to 512 KiB), and EasyFlash xbank in all three modes. An xbank is
bounded by the flash, not by a flat 512 KiB, so one wider than the reference tool reads is accepted. Cartridges are placed first, and programs fill the gaps between them as entries confined
to one half of a bank, the way the reference placement does. An Ocean entry puts the whole image in
Ocean mode, derived from the entries rather than declared; only one is allowed.

Editing saves through the same placement the build path uses, so an edited cart is the cart a fresh
build of the resulting entries would have produced, not a patch of the file that arrived: entries
the request never mentioned can move.

A new cart boots into our own menu (`menu/`, GPL), whether `build` makes it or the app's New Cart
does; `--menu easyloader` (or the app's menu picker) gives it EasyLoader, the embedded
third-party menu, in the same slot instead. A cart that is opened keeps the menu it has. It lists the
visible entries, scrolls, and starts programs and every cartridge kind except Ocean, including
programs whose bytes run into `$D000-$FFFF`, which EasyLoader refuses. It is refused on
carts it cannot run: one with an Ocean entry, or with a program that loads at `$D000` or above,
ends past `$FFFF`, or reaches `$D000` from below `$0150`. `list` and the desktop app say which menu a cart carries, and edits keep it. A cart
carrying an earlier build of our menu shows as "stidcart (older build)" and is saved with the
current one. A loader slot holding no menu the tool knows is replaced only when a menu is
chosen, with `edit --menu` or the app's picker. The reference tool knows nothing of this menu, so its evidence is VICE: boot tests start carts and read the
screen back (`menu/README.md`).

Each entry can carry a title (up to 48 characters), year (4), author (24), note (48) and folder
(16), set with `edit --title N TEXT` and its siblings, or for many entries at once with
`edit --import FILE.toml`. Our menu shows folders as folders; `--group` and the import key `group` are older names for it and still work. The cart keeps its own ordered list of
folders, which may be empty: `edit --new-folder NAME`, `--remove-folder NAME` (its entries stay, in
no folder) and `--move-folder NAME TO`; a folder stays when its last entry leaves, and `list`
prints the folders with their counts. Our menu lists them in that order and hides an empty one. They are stored in one hidden `STIDCART.META` entry that both menus skip,
written only when some entry has metadata, so a cart without any is unchanged. `list`
prints them under the entry. Records that another tool's re-order or rename left pointing at the
wrong entry are dropped and counted, and a layout from a later stidcart is kept as stored.
The layout is in `docs/format/easyflash-cart-format.md`, marked as a stidcart extension.

Describing a cart and rebuilding it are held to different standards, because they need to be:
`list` shows anything it can describe, while loading additionally refuses whatever it could not
reproduce unchanged (an image whose firmware regions differ from the blobs embedded here). Hidden
entries pass both: the model carries the flag, so they load, edit and save back intact.
Both refuse what neither can represent, which is now only the type `0x1A` EasyFlash entry the
format calls a subdirectory: the reference tool refuses it too, so nothing says what its layout is.
The subdirectory *bit* on an xbank entry is carried through every operation, as ndefpack does, and
`list` marks it `S`. Directories in the older EasyFS1 format are refused by both paths: read as EasyFS2 every
entry gets a corrupted bank, so there is no accurate description to give.
