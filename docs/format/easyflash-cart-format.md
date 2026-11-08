# EasyFlash cartridge format

> **This is the source for every offset in `stidcart-core`.** Read it before touching that crate;
> a format fact is changed here first, in a reviewable commit, and only then in code (D-018).
> Correcting what it _says_ needs its own evidence — cite `file:line` the way the rest of it does.

Everything below was read out of the ndefpack C sources (`src/libndef/`) and then checked
against a cart built by `ndeft`. It exists because the format is documented nowhere else in that
tree — `doc/README_tool` is a usage manual, `README` describes concepts, and the byte layout lives
only in `easyfs.c`, `cart.c`, `place.c` and `efname.c`. A successor tool that starts without this
has to re-derive it by trial and error against real hardware.

Target hardware is the EasyFlash 3. Output is the same `.crt` image described here; nothing below
is EF3-specific.

Citations are `file:line` in `~/Documents/develop/ndefpack` at commit `d77724b`.

Because it was derived by reading those GPL sources, this document is a derivative work of
ndefpack and is covered by this repository's GPL-2.0-or-later licence — the same argument D-004
makes for the Rust code, which was written from this document. `firmware/` remains zlib.

## Flash geometry

1 MB, 64 banks of 8 KB in each of two halves (`types.h:42-46`). The in-memory image is **not**
laid out bank-interleaved — all 64 ROML halves come first, then all 64 ROMH halves:

```
offset = (bank << 13) | (is_roml ? 0 : 1 << 19)      cart.c:284
```

ROML → `0x00000–0x7FFFF`, ROMH → `0x80000–0xFFFFF`. Reasoning from EasyFlash _bank_ numbering
gives the wrong answer here: `0x80000` is bank 0 ROMH, not bank 32. A test in ndefpack filled the
wrong half for its entire life on exactly this mistake.

ROML is seen by the C64 at `$8000`, ROMH at `$A000` (or `$E000` in Ultimax mode).

## CRT container

64-byte header, then a run of CHIP packets.

| Offset | Size | Content                                                                                                     |
| ------ | ---- | ----------------------------------------------------------------------------------------------------------- |
| `0x00` | 16   | `"C64 CARTRIDGE   "` (trailing spaces to 16)                                                                |
| `0x10` | 4    | header length, big-endian: the offset of the first CHIP packet; written as `0x00000040`, never less on read |
| `0x14` | 2    | version; written as `0x0100`                                                                                |
| `0x16` | 2    | CRT ID, big-endian                                                                                          |
| `0x18` | 1    | EXROM                                                                                                       |
| `0x19` | 1    | GAME                                                                                                        |
| `0x20` | 32   | cartridge name, NUL-padded                                                                                  |

CRT IDs: `0` generic, `5` Ocean, `32` EasyFlash, `33` EasyFlash xbank (`cart.h:31-35`).

EXROM/GAME are derived from a mode enum — `MODE_16K=0`, `MODE_ULT=1`, `MODE_8K=2`, `MODE_OFF=3`
(`cart.h:43-46`) — as `EXROM = mode & 1`, `GAME = (mode >> 1) & 1` (`cart.c:220-221`). A full
EasyFlash image is written as `MODE_ULT`, so the header carries `01 00`.

CHIP packet (`cart.c:107-127`):

| Offset | Size | Content                                          |
| ------ | ---- | ------------------------------------------------ |
| `0x00` | 4    | `"CHIP"`                                         |
| `0x04` | 4    | total packet length BE = `0x10 + data size`      |
| `0x08` | 2    | chip type BE; `2` = flash ROM                    |
| `0x0A` | 2    | bank BE                                          |
| `0x0C` | 2    | load address BE (`0x8000` / `0xA000` / `0xE000`) |
| `0x0E` | 2    | data size BE                                     |
| `0x10` | n    | data                                             |

Saving a full EasyFlash image walks all 64 banks and emits a ROML packet then a ROMH packet per
bank, **skipping any 8 KB block that is entirely `0xFF`** (`cart.c:129-140`, `cart.c:356-370`). So
a cart's file size tracks how much of the flash is actually used, and an unwritten bank is absent
rather than blank.

Loading accepts data sizes of `0x2000` and `0x4000` only; `0x4000` is legal only at `0x8000` and
fills ROML then ROMH of the same bank (`cart.c:283-300`).

## Bank 0 reserved map

Bank 0 ROMH is entirely reserved. Verified by rebuilding the image from a real `.crt` and dumping
it, not by arithmetic:

| Image offset | In-bank  | Size     | Content                                                   |
| ------------ | -------- | -------- | --------------------------------------------------------- |
| `0x80000`    | `0x0000` | `0x1800` | EasyFS2 directory — 256 slots × 24 B, of which 255 usable |
| `0x81800`    | `0x1800` | `0x0300` | EAPI (`eapi.c:38-39`)                                     |
| `0x81B00`    | `0x1B00` | `0x0008` | EF-Name magic `"ef-nAME:"` (`efname.c:36-39`)             |
| `0x81B08`    | `0x1B08` | `0x0010` | EF-Name, 16 B PETSCII (`efname.c:37`)                     |
| `0x81B18`    | `0x1B18` | `0x04E8` | boot code, loaded at `$FB18` (`boot.c:38-40`)             |

The regions are exactly contiguous and fill the bank. EAPI carries the ASCII signature `"eapi"` at
its first four bytes and a 16-byte PETSCII version string at `+4` (`eapi.c:41`) — e.g.
`AM/CM29F040 V1.4`. The odd capitalisation of `ef-nAME:` is literal.

The menu loader is 8 KB and lands outside this bank:

- normal mode → `0x00000`, i.e. bank 0 ROML, load address `$8000`
- Ocean mode → `0x82000`, i.e. bank 1 ROMH, load address `$A000`

(`loader.c:37-41`.) Ocean mode moves it because an Ocean cart must own bank 0 ROML.

### How the boot code hands over to the menu loader

Any menu loader, EasyLoader or a replacement, starts under this contract set by the boot code
(`easyloader_launcher.asm`):

- The boot code runs from the reset vector at `$FFFC` (Ultimax at power-on). It sets the stack,
  clears `$0100-$01FF`, and copies a stub to `$0100`, which switches the cartridge to 16K mode
  (`$DE02 = $07`) and selects bank 0 in normal mode or bank 1 in Ocean mode (`$DE00`).
- The stub scans the keyboard. If RUN/STOP, Q or C= is held, it switches the cartridge off
  (`$DE02 = $04`) and resets, so the machine starts without the menu.
- Otherwise it jumps through the first word of the loader: `jmp ($8000)` in normal mode,
  `jmp ($A000)` in Ocean mode. The loader's first two bytes are therefore its entry address.
- The KERNAL has not been initialised at that point: no IOINIT, RAMTAS, RESTOR or CINT has run.
  The loader does that itself.

The EasyFlash control register `$DE02` takes these mode values (`easyloader.asm:80-83`):
`$04` cartridge off, `$05` Ultimax, `$06` 8K, `$07` 16K. `$DE00` selects the bank. `$DF00-$DFFF`
is 256 bytes of RAM on the cartridge, visible in every mode, including after `$DE02 = $04`.

When EasyLoader starts an xbank entry, it leaves the entry's first bank in `$DF00` (low byte) and
`$DF01` (high byte) before it selects that bank and resets (`easyloader.asm:1789-1797`). An xbank
cart finds its own banks through that convention. It is EasyLoader's, not a published spec, but
any loader that starts xbank entries has to keep it.

## EasyFS2 directory entry

24 bytes (`easyfs.h:33`), encoded by `efs_entry_code` (`easyfs.c:562-612`) and parsed by
`efs2_entry_parse` (`easyfs.c:184-254`).

| Offset  | Size | Field                                              |
| ------- | ---- | -------------------------------------------------- |
| `0–15`  | 16   | name, PETSCII, NUL-padded                          |
| `16`    | 1    | flags                                              |
| `17–18` | 2    | starting bank, little-endian                       |
| `19–20` | 2    | offset (PRG) **or** CrtUsage (cart), little-endian |
| `21–23` | 3    | size in bytes, little-endian                       |

Flags byte:

- bits 0–4 (`0x1F`) — entry type
- bits 5–6 (`0x60`) — reserved, **always written set**. These are the EasyFS1/EasyFS2 version
  discriminator on read — see below. Anything writing must set them.
- bit 7 (`0x80`) — hidden (not shown by the menu)

### The reserved bits are the version discriminator

`efs_check_entry` (`easyfs.c:371-389`) classifies a slot by these two bits: not both set means an
old EasyFS (EasyFS1) directory. The **first non-end slot decides for the whole directory**, and a
later slot that disagrees is a hard error rather than a reclassification — `efs_parse_all`
(`easyfs.c:831`) threads that state across the walk. The end marker is `0xFF`-filled, so its
reserved bits are set and it never participates in the classification.

The discriminator is unambiguous for anything ndefpack wrote: `efs_entry_code` ORs
`EFS_RESERVED_MASK` unconditionally (`easyfs.c:581`), so there is no path that emits an EasyFS2
entry with the bits clear. A reader therefore cannot both ignore these bits and detect the format
version — tolerating a clear pair as EasyFS2 gives up the only signal there is.

The one third-party writer known to get this wrong is **Draco Cart Maker** 1.0.1, which
"incorrectly sets reserved EasyFS flag bits to 0" (ndefpack `README:82` — a 2012 comparison against
a tool whose homepage is gone, never rechecked). Such a cart is classified as old EasyFS and
converted or refused on that basis. That is a consequence of the format having no other version
field, not a bug to work around.

For the entry shapes worth reading, EasyFS1's structural difference is that the **bank is a single
byte** at offset 17 — `data[18]` is not part of the field (`efs_entry_parse_old`,
`easyfs.c:255-260`) — and cart types are guessed from size, which the source itself flags with two
`WARNING: assumes …` comments. An EasyFS1 directory parsed as EasyFS2 therefore yields a corrupted
bank number rather than an obvious failure, which is why detecting the version matters before
anything else is read.

The end-of-directory marker is a slot of 24 × `0xFF`, which reads back as type `0x1F` (`easyfs.c:570-573`).
Empty trailing slots are therefore indistinguishable from end markers, which is the intent.

The region holds 256 slots but only 255 entries: the writer stops at 255 so the last slot always
carries the end marker (`flash.c:471`, `easyfs.c:850-853`). A directory with a valid entry in every
one of the 256 slots and no marker anywhere is rejected outright — accepting it would read as an
empty cart and destroy the directory on the next save.

Entry types, by the value in bits 0–4 (`easyfs.c:64-80`):

| Value  | Type               | Mode | CRT ID |
| ------ | ------------------ | ---- | ------ |
| `0x00` | (none)             | off  | —      |
| `0x01` | PRG, interleaved   | off  | —      |
| `0x02` | PRG, ROML only     | off  | —      |
| `0x03` | PRG, ROMH only     | off  | —      |
| `0x10` | 8k cart            | 8k   | 0      |
| `0x11` | 16k cart           | 16k  | 0      |
| `0x12` | Ultimax 16k        | ult  | 0      |
| `0x13` | Ultimax 8k         | ult  | 0      |
| `0x14` | Ocean 512k         | 8k   | 5      |
| `0x15` | Ocean              | 16k  | 5      |
| `0x1A` | EasyFlash (subdir) | ult  | 32     |
| `0x1C` | xbank 8k           | 8k   | 33     |
| `0x1D` | xbank 16k          | 16k  | 33     |
| `0x1E` | xbank Ultimax      | ult  | 33     |
| `0x1F` | end marker         | off  | —      |

Bit `0x10` of the type is the "this is a cart" discriminator, and it selects how bytes 19–20 are
read.

### Bytes 19–20 for PRG entries

A byte offset within the bank, plus bit 13 as a half-selector:

```
addr = bank * 0x2000 + (offset & 0x1FFF) + ((offset & 0x2000) ? 1 << 19 : 0)   prg.c:83-85
```

A PRG that overflows its bank continues by type:

- type `0x01` (interleaved): ROML → ROMH of the same bank → ROML of the next bank
- types `0x02`/`0x03`: next bank, same half

(`prg.c:106-116`.) PRG data includes its 2-byte load address, and the maximum size is `0xE000`
(`prg.c:39`).

### Bytes 19–20 for cart entries (CrtUsage)

| Mask     | Meaning                                                           |
| -------- | ----------------------------------------------------------------- |
| `0x8000` | MSB, always set on write                                          |
| `0x4000` | subdirectory                                                      |
| `0x2000` | force 64 KB alignment                                             |
| `0x1F00` | offset in banks from the start bank to the opposite half (`>> 8`) |
| `0x00FF` | number of banks in the opposite half                              |

(`easyfs.c:50-55`, `easyfs.c:210-230`.) The bank count of the _primary_ half is not stored — it is
derived:

```
bank_num = (size / 0x2000) - other_bank_num
```

Both terms are file-controlled, so a parser must reject `other_bank_num > size / 0x2000` before
subtracting; unsigned underflow here was a real heap overflow in ndefpack (issue #32). It must
also bound both halves against the top of flash — `bank + bank_num` and
`bank + other_off + other_num` against 64 (`easyfs.c:159-174`). Neither check is implied by
validating the fields individually.

Which half is "primary" depends on the type: for 8k/16k the primary is ROML and the opposite is
ROMH; for Ultimax it is reversed (`easyfs.h:99-104`, `easyfs.c:124-127` — the test is bit 1 of the
type value).

### Subdirectories

Two things in the format carry the word, and the reference implementation treats them differently:

- **The `0x4000` CrtUsage bit** is accepted only on type `0x1A` and the xbank types `0x1C`–`0x1E`;
  on any other cart type the parser rejects the entry ("subdir flag set for non-EF/xbank cart",
  `easyfs.c:412`). Where accepted it is read into a flag (`easyfs.c:211-212`) and written back
  unchanged (`easyfs.c:590-591`). Nothing else reads it: `place.c` never consults it, so it does
  not affect placement, and nothing in the tool creates it. It is carried, not interpreted.
- **Type `0x1A` ("EasyFlash", CRT id 32)** is refused outright when a directory is read
  ("unsupported entry type", `easyfs.c:500-502`), and an EasyFlash `.crt` given as an input is
  refused too (`cart.c:668`, no loader for id 32). So the reference tool can neither create nor
  carry one, and its layout has no oracle.

The embedded EasyLoader does not navigate either. Its directory scan masks a cart type with `$13`
(`easyloader.asm:912`), so a `0x1A` entry shows and boots as an Ultimax cart at its bank, and the
`0x4000` bit is never tested. Whatever a "subdirectory" meant to the tool that wrote one, this
menu treats it as a plain cartridge.

## Names and PETSCII

Both the 16-byte entry name and the 16-byte EF-Name are stored as PETSCII and converted on the way
in and out (`easyfs.c:232`, `easyfs.c:578`). The conversion uppercases: `"custom name"` in, and the
bytes on flash read `CUSTOM NAME`. In ndefpack both converters return a shared `static` buffer, so
two results cannot be live at once — a trap worth not reproducing.

## Placement

`place.c` decides where entries land. It is a greedy first-fit over a weight-sorted list, not an
optimiser.

Weights, larger placed first (`place.c:504-543`):

| Entry                   | Weight                                              |
| ----------------------- | --------------------------------------------------- |
| Ocean / Ocean 512k      | `1 << 30`                                           |
| cart with 64k alignment | `(1 << 16) * bank_top + other_bank_num` + `1 << 28` |
| other carts             | `(1 << 16) * bank_top + other_bank_num`             |
| PRG                     | its size in bytes                                   |

where `bank_top = max(bank_num, other_bank_off + other_bank_num)`.

The ordering is what makes it work: Ocean carts have the strictest constraint (they must start at
bank 0), 64k-aligned xbank carts next, then carts by height, and PRGs last because they can be
split across banks and are the most flexible. All carts are placed before any PRG
(`place.c:590-617`).

Carts are placed by scanning for a free run of banks in both halves at once, stepping 8 banks at a
time when 64k alignment is requested and 1 otherwise (`place.c:400-406`). PRGs are then packed
largest-first, either into the contiguous free space at the top of flash (interleaved, type `0x01`)
or into a gap left between carts (fixed half, types `0x02`/`0x03`).

Consequence worth knowing before designing a replacement: entry order in the directory is
independent of physical placement, and placement is order-dependent — adding one entry can move
others. A tool offering undo needs to treat the whole placement as the unit of change, not the
individual entry.

## Behaviours to decide on, not inherit

Each of these is what the 2012 implementation does. Reproducing them is a choice; the point of
listing them is that the choice should be deliberate.

- **The CHIP magic is never checked on read.** `read_chip_hdr` (`cart.c:142-155`) unpacks bank,
  address and length and ignores both the `"CHIP"` string and the total-length field at `0x04`,
  reading exactly the data length instead. Implementation artifact. Tightening it changes
  accept/reject behaviour on malformed input.
- **The CRT header length field is written but never read.** Loading assumes a 64-byte header
  (`cart.c:171`) and starts the packet walk at `0x40` whatever the field says. Fine for every cart
  in the wild, wrong per the container spec, which defines the field as the offset of the first
  packet and permits it to exceed `0x40`. Implementation artifact. A reader that honours it starts
  the walk where the field points and refuses a value below `0x40`, since the fields above end
  there.
- **An empty new-format directory is rejected.** With an end marker in slot 0, `efs_parse_all`
  reports "no EasyFS detected". This is a bug in ndefpack (a `break` ordering slip), not the
  format saying anything. A new implementation should accept it.
- **Old EasyFS entries are type-guessed from size.** EasyFS1 stored less, so
  `efs_entry_parse_old` (`easyfs.c:259+`) infers Ocean vs xbank from the size and whether the bank
  is 0, and carries two `WARNING: assumes …` comments where it guesses ROML-only or a "square"
  ROML/ROMH split. `README:110-113` is candid that conversion may fail. This is a genuine format
  limitation, not sloppiness — but the guesses are lossy and worth doing differently.
- **Wide 16k xbank carts cannot be read back at all.** An entry is bounded at 512 KiB while the
  loader sets its size to the whole source cart's byte count, and a 16k xbank counts both halves —
  32 banks hits `0x80000` exactly and anything wider is refused. Whether the right bound is
  per-type or a flat 1 MB is an open question nobody has needed to answer.
- **The subdirectory bit is rejected on non-xbank cart types** (`easyfs.c:412`; see
  "Subdirectories" above). Format-consistent rather than an artifact: the bit names a property of
  an EasyFlash-family cart, and a generic or Ocean cart cannot have one. A reader refuses it as
  malformed at that slot.
- **Type `0x1A` is refused on read** (`easyfs.c:500-502`). No implementation in reach writes,
  carries or tests one, so accepting it would mean guessing its layout; refusing it as unsupported
  stays until a boot in VICE or on hardware says what it is.
- **The producing tool's name and version land in the CRT header** — ndefpack uses
  `PACKAGE_STRING` as the default (`cart.c:45`). A new tool writes its own string there, so this
  is an expected byte difference rather than a parity failure. Pin it explicitly on both sides
  when diffing output.

## stidcart extension: entry metadata (`STIDCART.META`)

> **Not an ndefpack fact.** Nothing in the reference tree defines this. stidcart defines it
> (D-054, D-055), and its evidence is stidcart's own round-trip tests and VICE boots, not
> citations. Every other tool sees an ordinary hidden program entry.

A cart may carry a title, year, author, note and group per entry in one extra directory entry.
It is written only when at least one entry has metadata, so a cart without any is unchanged.

**The entry.** It is an ordinary program entry: interleaved type `0x01`, or `0x02`/`0x03` if
placement confines it to a gap, so a reader must follow all three program kinds. It has the
hidden bit set, and its 16 name bytes are `STIDCART.META` (`53 54 49 44 43 41 52 54 2E 4D 45 54
41`) NUL-padded. It is placed like any program and written in the **last** directory slot. Both
menus skip hidden entries, and EasyLoader skips it as it skips any hidden entry
(`easyloader.asm:900`, `:935-941`). Its data is a PRG with load address `$4000`, followed by a
payload of at most 16384 bytes, so a copy of the payload fills at most `$4000-$7FFF`.

**Recognising it.** A hidden program slot with that name whose payload begins with the magic.
Without the magic it is an ordinary entry. When several qualify, the last slot is the one. A
reader numbers entries without it: the _entry index_ of a slot is its position among the other
entries. A writer that puts the entry last makes that the directory slot.

**Payload, layout version 1.** All multi-byte values are little-endian. Text is PETSCII,
converted the way names are (see "Names and PETSCII").

| Offset | Size | Field                                                                |
| ------ | ---- | -------------------------------------------------------------------- |
| `0`    | 8    | magic `53 54 49 44 4D 45 54 41` (ASCII `STIDMETA`)                   |
| `8`    | 1    | layout version, `1`                                                  |
| `9`    | 1    | G, the number of groups                                              |
| `10`   | 1    | R, the number of records                                             |
| `11`   | 2    | offset of the first record from the payload start (`13 + 16G`)       |
| `13`   | 16·G | group names, 16 bytes each, NUL-padded; group n is at `13 + 16(n-1)` |
| …      | …    | R records, one after another                                         |

Groups are numbered from 1, in the order entries first use them. A record exists only for an
entry with some metadata, and records run in ascending entry index:

| Offset | Size | Field                                                    |
| ------ | ---- | -------------------------------------------------------- |
| `0`    | 1    | L, the record's whole length including this byte (≤ 147) |
| `1`    | 1    | entry index                                              |
| `2`    | 16   | the entry's name bytes as its directory slot stores them |
| `18`   | 1    | group number, `0` for none                               |
| `19`   | 1+n  | title: length n (≤ 48), then n bytes                     |
| …      | 1+n  | year: length n (≤ 4), then n bytes                       |
| …      | 1+n  | author: length n (≤ 24), then n bytes                    |
| …      | 1+n  | note: length n (≤ 48), then n bytes                      |

A field of length 0 is absent. Group names are at most 16 characters. Text is restricted to
characters that convert to PETSCII and back unchanged (printable ASCII, `0x20-0x7E`).

**Attaching records.** A record belongs to the entry at its index only when its 16 name bytes
equal that slot's stored name bytes. A record whose index is out of range, whose name differs (the
directory was re-ordered or renamed by a tool that doesn't know this entry), or which repeats an
index already matched is dropped. Writing the cart again drops it for good.

**Other versions.** A payload with the magic but another version, or a version-1 payload that
doesn't parse, is kept byte for byte by a writer that can't read it. Such a writer must not change
anything a record could refer to: the entries before it, their order and their names.

**Reading it on the C64.** Copy the payload to `$4000`, check the magic and the version, and walk
the records from the offset at `$400B`. Each record's length byte skips to the next. A record's
group name is at `$400D + 16(group-1)`.

## Checking an image by hand

Rebuild the flash image from a `.crt` and look at it directly. This is how the bank-0 map above
was confirmed:

```python
import struct
f = open('some.crt','rb').read()
img = bytearray(b'\xff' * 0x100000)
p = 0x40
while p + 16 <= len(f) and f[p:p+4] == b'CHIP':
    total = struct.unpack('>I', f[p+4:p+8])[0]
    bank  = struct.unpack('>H', f[p+10:p+12])[0]
    addr  = struct.unpack('>H', f[p+12:p+14])[0]
    ln    = struct.unpack('>H', f[p+14:p+16])[0]
    off   = (bank << 13) | (0 if addr == 0x8000 else 1 << 19)
    img[off:off+ln] = f[p+16:p+16+ln]
    p += total                      # total length, not 16 + data
open('img.bin','wb').write(img)
```

Then `xxd -s 0x80000 -l 80 img.bin` shows the directory, and `xxd -s 0x81b00` the EF-Name.

## Using ndefpack as an oracle

`ndeft` is the only implementation known to produce images that boot on real EasyFlash hardware —
bank 63 was confirmed on an EF3 on 2026-08-09. That makes it a free test fixture for a successor,
independent of any code being shared: build the same inputs with both and compare bytes. Pin the
EF-Name with `-e` on both sides so `PACKAGE_STRING` does not show up as a spurious difference.

Its `.lst` text format is also a candidate interchange format, but scope the claim to what was
actually measured on 2026-08-10: a cart with two interleaved PRG entries, one renamed and one
hidden, plus the boot/loader/EAPI blobs, survived `dump` → `make` byte-identically. Entry names,
the hidden flag and entry order all round-trip.

Two gaps. The EF-Name is not carried at all — it is not a `.lst` field but a flash region
(`0x81B08`, magic at `0x81B00`) that `ndeft` writes from its `-e` option, so it has to travel
out-of-band; supply it and the hashes match. And the `a` force-64k-align flag and xbank entries
were **not** tested, so their round-trip is unverified. Check both before building a document
model on this.

## Where the rest of the context lives

This document is the format only. Why ndefpack was set aside and this tool written instead is the
predecessor's decision log, `projects/ndefpack.md` in the Obsidian vault at
`~/Documents/obsidian/stid` — personal project history, not something this codebase owns. Decisions
this repo made _about_ the format are in `docs/lcd/DECISIONS.md`; what the tool does with it today
is `docs/lcd/SPEC.md`.
