# Spec: Cartridge entries beyond PRG

> **Deep lane · Phase 1 of 6 (Specify).** This document owns **what** and **why**. No tech
> stack, no file paths, no function names. Move all of that to `plan.md` in Phase 2.

**Slug:** `cart-entries`
**Created:** 2026-09-19
**Related issue:** none
**Status:** approved <!-- draft | approved | implemented | abandoned -->

---

## Problem

A multicart is meant to hold cartridges as well as programs, and the directory format defines
nine cartridge entry kinds for that: plain 8k and 16k carts, the two Ultimax shapes, the two Ocean
shapes, and the three EasyFlash xbank shapes. This tool models none of them. A cart holding one is
refused on every path — it cannot be listed, loaded or edited — and nothing can be built that
contains one. The reason the project exists (vault: `projects/ndefpack.md`, 2026-08-10) is broader
compatibility than the reference tool, and today it is narrower: the reference tool builds, lists
and edits all nine kinds. The refusals were placed deliberately while the model had nowhere to put
a cart (D-011, D-020 for the same pattern with the hidden flag); this work widens the model so the
reason for them goes away.

Two things ride along because the reference tool's placement makes them inseparable. When
cartridges are present, programs are packed into the gaps between them as entries confined to one
half of a bank, so reproducing the reference tool's images means producing and reading those
half-confined program entries too — a second kind this tool currently refuses on read. And an
Ocean cartridge must own bank 0, which moves the menu firmware: a cart holding one is a different
mode of the whole image, with its own boot code and loader position, not just another entry.

The format spec leaves one bound open: the reference tool cannot read back a 16k xbank wider than
32 banks because it caps every entry at 512 KiB, and whether that cap is the format's or the
implementation's has never been decided. This work has to decide it.

## User stories

- As someone assembling a multicart, I want to put cartridge images on it alongside programs, so
  that one cart carries the games I own in cartridge form and the ones I own as programs.
- As someone who already has a multicart built by another tool, I want to list and edit it here
  even when it holds cartridge entries, so that switching tools does not mean rebuilding from
  sources I may not have.
- As someone putting an Ocean-format game on a cart, I want the tool to arrange the image so that
  game boots, so that the one cartridge format that needs bank 0 works without me knowing why.
- As someone with a large xbank cart, I want it accepted whenever it physically fits in flash, so
  that a limit the format does not impose does not stop me.

## Acceptance criteria

**AC-31** (surfaces: none): Given a directory entry of any of the nine cartridge kinds — 8k, 16k,
Ultimax 8k, Ultimax 16k, Ocean, Ocean 512k, xbank 8k, xbank 16k, xbank Ultimax — when it is
encoded and decoded, then every field survives: name, kind, hidden flag, start bank, the 64 KB
alignment request, the opposite half's bank offset and count, and the size; and the primary half's
bank count derived from them matches what was encoded.

**AC-32** (surfaces: none): Given a cartridge entry whose stored fields describe a run of banks
reaching past the top of flash in either half, or whose opposite-half count exceeds what its size
allows, when the directory is read, then it is refused as malformed at that slot — never read as a
smaller entry, and never by way of an arithmetic underflow, which was a real memory-safety bug in
the reference tool.

**AC-33** (surfaces: none): Given a mix of cartridge entries and programs, when the image is laid
out, then every cartridge is placed before any program, cartridges in the order Ocean first, then
64 KB-aligned ones, then the rest by height, each into the first free run of banks that fits in
both halves (aligned ones stepping eight banks at a time); the frozen reference fixtures pin the
resulting bytes.

**AC-34** (surfaces: none): Given cartridges that leave gaps between them, when programs are placed,
then a program that fits a gap lands there confined to one half, and one that does not lands in
the interleaved space at the top of flash, largest first — as the frozen reference fixtures pin.

**AC-35** (surfaces: none): Given a cart holding programs confined to one half of a bank, when it is
described or loaded, then those programs are read following the same-half, next-bank continuation
the format defines, the former refusal of them is retired, and a load-and-save of a reference
fixture holding them is byte-identical to the fixture.

**AC-36** (surfaces: none): Given an entry set containing an Ocean cartridge, when the image is
built, then the Ocean cartridge owns bank 0's lower half, the menu loader sits in bank 1's upper
half, the Ocean-mode boot code and loader are the firmware written, and the container still
identifies the whole image as an EasyFlash cart; and loading such an image verifies its firmware
against the Ocean-mode blobs rather than refusing it as mismatched.

**AC-37** (surfaces: none): Given an entry set with more than one Ocean cartridge, when the image is
built, then the build is refused with the reason named — only one entry can own bank 0.

**AC-38** (surfaces: none): Given an xbank cartridge wider than 32 banks, when it is placed, then it
is accepted as long as its banks fit in flash outside the reserved bank, is written and read back
intact, and is never refused on the reference tool's flat 512 KiB limit; and an image the reference
tool refuses to read back for that reason alone is still described and loaded here.

**AC-39** (surfaces: none): Given a cartridge image as input, when it is turned into an entry, then
its kind is derived from the image's own header and packet layout — generic images become 8k, 16k
or one of the Ultimax kinds; Ocean images become Ocean or Ocean 512k; EasyFlash xbank images become
the matching xbank kind — its banks map into the entry's two halves in order, and an image of any
other kind (an EasyFlash multicart itself, or an id the format does not list) is refused naming what
it was.

**AC-40** (surfaces: CLI): Given a build request whose inputs include a cartridge image, when the
user builds, then the built cart holds a cartridge entry of the derived kind named after the input's
file stem, alongside the program entries; an input is recognised as a cartridge by its content, not
its file name; and a build of programs only produces the bytes it produced before this capability
existed.

**AC-41** (surfaces: CLI): Given an existing cart, when the user adds a cartridge image to it or
replaces an entry's data with one, then the saved cart holds the entry with the kind the new data
implies — replacing keeps the name and the place in the order, as it does for programs.

**AC-42** (surfaces: CLI): Given an xbank cartridge being built in or added, when the user asks for
64 KB alignment on it, then the saved entry carries the request and is placed on a 64 KB boundary;
asking for alignment on any other kind is refused naming the entry; and an existing entry's
alignment can be set or cleared by index when editing.

**AC-43** (surfaces: CLI): Given a cart holding cartridge entries, when the user lists or edits it,
then every operation that works on a program entry — remove, rename, hide, show, replace — works on a
cartridge entry by the same index; the former whole-cart refusal on "an entry kind it cannot
represent" narrows to the one kind still unmodelled, the EasyFlash subdirectory entry.

**AC-44** (surfaces: CLI): Given a cartridge image input this tool cannot represent — an EasyFlash
multicart, a subdirectory, an unlisted id — when the user builds with it or adds it, then the command
reports what the image is and exits non-zero, and every file it was given is byte-identical to
before.

**AC-45** (surfaces: RENDER): Given a cart holding cartridge entries and half-confined programs,
when it is listed, then every entry shows a short label for its kind (one distinct label per kind,
the half-confined program kinds included), a cartridge entry shows the banks it occupies in each
half instead of a raw usage word, an aligned entry is marked as such, and no entry is shown as
`?`.

**AC-46** (surfaces: RENDER): Given a cart holding cartridge entries, when it is listed, then the
free-bytes figure accounts for the banks the cartridges occupy in both halves, so a further entry
of that size could actually be placed.

## Out of scope

- **EasyFlash subdirectory entries** (the one cartridge kind that nests a whole directory). Still
  refused, and the only remaining "cannot represent" refusal (AC-43).
- **Reading, converting or upgrading old-format (EasyFS1) carts.** Unchanged from D-017; the
  old format's size-guessed cart types are exactly why.
- **Extracting a cartridge entry back out to an image file.** Consistent with cart-edit: data goes
  in, nothing reads it back out as a file.
- **Custom or overridden firmware.** The Ocean-mode firmware written is the embedded blob, as the
  normal-mode one is; there is no way to supply another, and a cart carrying different firmware is
  still refused for rebuilding.
- **Omitting EAPI** (the reference tool's `-d`), sorting the menu, and reordering entries.
- **A cartridge whose own image is malformed** being repaired or partially imported: it is
  refused at the input, with the container's existing malformed-input reasons.
- **Any graphical surface.**

## Open questions

All five answered 2026-09-19, each on its recommendation; the ACs stand as written.

- [x] **Ocean in this work-item, or a follow-up?** Ocean changes the whole image's firmware and
  loader position, so it is the largest single piece. *Recommended: in.* Leaving it out keeps a
  hole in the model that every read path has to refuse around, and the reference tool produces
  fixtures for it, so the oracle still applies.
- [x] **The wide-xbank bound: flash geometry, or the reference tool's flat 512 KiB?** *Recommended:
  flash geometry* (AC-38). The 512 KiB cap is a literal check the reference tool applies when it reads a directory, not a
  field width — the size field is 24 bits and the bank fields address all of flash. The oracle
  cannot cover this case (the reference tool refuses it), so its evidence is a round-trip here plus
  a boot in VICE, recorded when done.
- [x] **Is 64 KB alignment exposed on the command line, or only preserved?** *Recommended:
  exposed* (AC-42) — settable when adding and toggled by index when editing, matching the reference
  tool's `a` marker and its align/unalign operations. Preserve-only would make the tool unable to
  produce a cart it can read.
- [x] **May `replace` swap a program for a cartridge or vice versa?** *Recommended: yes* (AC-41);
  the entry's kind follows its data, and the name and position are what replace preserves.
- [x] **Cartridge input recognised by content or by extension?** *Recommended: content.* A program
  starts with a two-byte load address and an image with a fixed sixteen-byte signature; the two
  cannot collide, and file names lie.

---

<!--
Quality checks before approving this spec:
  - No tech stack named (no language / framework / database appears anywhere)
  - No file paths or function names
  - Every acceptance criterion follows `**AC-N** (surfaces: <CSV>): <body>`
  - Surface tokens are exactly from {CLI, RENDER, none} — the surfaces MAP.md records in use
  - Every acceptance criterion is testable (you can describe an assertion that proves it)
  - Out-of-scope is filled in, even if just one bullet
  - Open questions either all answered, or explicitly flagged for /lcd:plan to resolve
-->

