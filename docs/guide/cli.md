# The command line

`stidcart` builds, lists and edits EasyFlash cartridge images. Some examples come first, then
[every option](#every-option). Installing it is in the [README](../../README.md#getting-it).

## Examples

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

## Every option

`stidcart` with no arguments prints this:

```text
usage: stidcart build [-e NAME] [--menu MENU] [--hide INPUT]... [--align INPUT]... -o OUTPUT INPUT...
       stidcart list [--bytes] CART.crt
       stidcart edit CART.crt [-o OUTPUT] [OPERATION]...
       stidcart --version

  build           create a cartridge image from programs and cartridge images
  -e NAME         cartridge name shown by the menu (default: "(no EF-Name)")
  -o OUTPUT       where to write the image
  --menu MENU     the menu the cart boots into: stidcart (default) or easyloader
  --hide INPUT    mark that input hidden in the built cartridge
  --align INPUT   request 64 KB alignment for that xbank cartridge input
  INPUT           a .prg, or a cartridge image (recognised by content: generic 8k/16k/Ultimax,
                  Ocean, EasyFlash xbank); the entry name is the file stem

  list            show what a cartridge image contains, sizes in bytes, KB or MB
  --bytes         give every size and the free space as exact byte counts
  CART.crt        the image to read

  edit            change an existing image, writing over it unless -o is given
  -o OUTPUT       write the result here and leave CART.crt untouched
  --add FILE      append a program or a cartridge image, named after its file stem
  --add-aligned FILE  append an xbank cartridge image with 64 KB alignment requested
  --remove N      drop the entry at index N
  --rename N NAME rename the entry at index N
  --move N TO     move the entry at index N so it sits at index TO; once per call,
                  and not with --add, --add-aligned or --remove
  --replace N FILE swap the data at index N, keeping its name and place; the kind follows the file
  --hide N        hide the entry at index N
  --show N        stop hiding the entry at index N
  --align N       request 64 KB alignment for the xbank cartridge at index N
  --unalign N     clear that request
  -e NAME         set the cartridge name shown by the menu
  --header NAME   set the container header name
  --menu MENU     switch the menu to easyloader or stidcart, replacing whatever the loader slot
                  holds; without it the cart keeps its menu
  --title N TEXT  set the title of the entry at index N (at most 48 characters)
  --year N TEXT   set its year (at most 4)
  --author N TEXT set its author (at most 24)
  --note N TEXT   set its note (at most 48)
  --folder N TEXT put it in a folder (at most 16; --group N TEXT also works); an
                  empty TEXT clears any of these
  --new-folder NAME   add an empty folder after the cart's folders
  --remove-folder NAME  remove a folder; its entries stay, in no folder
  --move-folder NAME TO  put a folder at position TO of the cart's folder order
  --import FILE   set metadata from a TOML file of [[entry]] tables, each naming an
                  entry by its stored name (case ignored) with any of title, year,
                  author, note and folder; any problem refuses the whole file

  Indices are the ones `list` prints, and every index in one invocation refers
  to the image as it was given, not to the state a previous operation left.
  Metadata text is printable ASCII, so it maps to PETSCII as names do.
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
its file name: generic 8K, 16K and Ultimax images, Ocean (up to 512 KB), and EasyFlash multi-bank
("xbank") cartridges of the 8K, 16K and Ultimax kinds. An xbank cartridge can be as large as the flash allows.
A cart can hold only one Ocean entry, and a cart with one has to use EasyLoader: build it with
`--menu easyloader`.

Cartridges are placed first and programs fill the gaps between them, the way ndefpack places them.
`--align` places an xbank cartridge on a 64 KB boundary, which is every 8th bank. ndefpack has the
same option. Use it if a cartridge doesn't start without it.

A hidden entry (`--hide`) stays on the cart, but neither menu lists it. `list` and the app still
show it, flagged `H`, and `--show` brings it back.

Sizes are in bytes, KB and MB as `list` prints them, where 1 KB is 1024 bytes. An EasyFlash holds
1 MB.

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
the current build when it is saved. If a cart's menu is one stidcart doesn't know, `list` still shows the cart, but `edit` refuses it
until you choose a menu with `edit --menu`, which replaces the one it has.

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
couldn't write back unchanged: one whose boot code or EAPI (the flash routines every EasyFlash
cart carries) differs from the copy stidcart embeds, or whose menu it doesn't know. For an unknown
menu, `edit --menu` gets past that by replacing it.

Both refuse two things:

- a cart with an entry of the kind the format calls an EasyFlash subdirectory (type `0x1A`).
  ndefpack refuses it too, and nothing documents how such an entry is laid out.
- a cart whose list of entries is in EasyFS1, the older of the two directory formats EasyFlash
  carts have used. stidcart reads and writes the current one, EasyFS2. Read that way, every entry
  of an old cart would show the wrong bank, so there is no accurate listing to give.

Hidden entries load, edit and save back as hidden. An xbank entry can also carry a "subdirectory"
flag that some other tool set. stidcart doesn't use it, keeps it through every operation, and
`list` marks it `S`.
