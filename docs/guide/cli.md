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
stidcart edit out.crt --title 0 "RAM Test" --folder 0 Tools
stidcart edit out.crt --move 3 0   # entry 3 to the top, the rest in order
stidcart edit out.crt --new-folder Tools --move-folder Tools 0
stidcart edit out.crt --import meta.toml   # see "Titles for many entries at once"
```

## Titles for many entries at once

An import file names entries by their stored name, ignoring case. Fields left out are left alone,
an empty value clears one, and any problem (a name matching no entry or several, an unknown key, a
value too long) refuses the whole file before anything is written:

```toml
[[entry]]
name = "ram test"   # the entry's name as `list` shows it, at most 16 characters
title = "RAM Test"
year = 2026
author = "stid"
folder = "Tools"
```

## What goes on a cart

An entry is a program (`.prg`) or a cartridge image. A cartridge is recognised by its content, not
its file name: generic 8K, 16K and Ultimax images, Ocean (up to 512 KiB), and EasyFlash multi-bank
("xbank") cartridges in all three modes. An xbank cartridge can be as large as the flash allows.
A cart can hold only one Ocean entry, and having one puts the whole image in Ocean mode.

Cartridges are placed first and programs fill the gaps between them, the way ndefpack places them.
`--align` asks for an xbank cartridge to start on a 64 KB boundary.

## How edits are saved

`edit` doesn't patch the file. It loads the cart, applies your changes and builds the image again,
so the result is the cart a fresh `build` of the same entries would give. Entries you didn't
mention can move to another place in the flash. `edit` writes over the file unless you give
`-o OUTPUT`.

## The menu a cart boots into

A new cart gets the stidcart menu. `build --menu easyloader` gives it EasyLoader in the same slot
instead, and `edit --menu stidcart|easyloader` switches an existing cart. A cart you open and
edit keeps the menu it has. `list` says which one a cart carries.

The stidcart menu starts programs and every kind of cartridge except Ocean, including programs
that reach into `$D000-$FFFF`, which EasyLoader won't start. stidcart refuses to put its menu on a
cart the menu can't run, and names the entry:

- a cart with an Ocean entry
- a program that loads at `$D000` or above, or ends past `$FFFF`
- a program that reaches `$D000` and loads below `$0150`

A cart carrying an earlier build of the stidcart menu shows as "stidcart (older build)" and gets
the current build when it is saved. If a cart's loader is one stidcart doesn't know, it is left
alone until you choose a menu with `edit --menu`.

Using the menu on the C64 is in [menu.md](menu.md).

## Titles, details and folders

Each entry can carry a title (up to 48 characters), a year (4), an author (24), a note (48) and a
folder (16). Set them with `edit --title N TEXT`, `--year`, `--author`, `--note` and `--folder`,
or for many entries at once with `--import`. `--group` and the import key `group` are older names
for the folder and still work. `list` prints these under each entry.

The cart keeps its own ordered list of folders, and a folder can be empty:

- `edit --new-folder NAME` adds one.
- `edit --remove-folder NAME` removes it. Its entries stay on the cart, in no folder.
- `edit --move-folder NAME TO` moves it in the order.

A folder stays when its last entry leaves it. `list` prints the folders with their counts. The
stidcart menu shows them in the cart's order and hides an empty one. EasyLoader has no folders and
shows a flat list.

All of this is stored in one hidden entry, `STIDCART.META`, which both menus skip. It is written
only when some entry has a title, a detail or a folder, so a cart without any is unchanged. If
another tool reorders or renames entries, the records that no longer match their entry are dropped
and counted when you open the cart. A layout written by a later stidcart is kept as it is. The
layout is described in [the format spec](../format/easyflash-cart-format.md), marked as a stidcart
extension.

## Carts it won't open or edit

`list` shows anything it can describe. Editing asks for more: `edit` also refuses a cart it
couldn't write back unchanged, such as one whose firmware differs from the copy stidcart embeds.

Both refuse two things:

- a cart with an EasyFlash subdirectory entry (type `0x1A`). ndefpack refuses it too, and nothing
  documents its layout.
- a cart whose directory is in the older EasyFS1 format. Read as EasyFS2, every entry would show
  the wrong bank, so there is no accurate listing to give.

Hidden entries load, edit and save back as hidden. The subdirectory bit on an xbank entry is
carried through every operation, and `list` marks it `S`.
