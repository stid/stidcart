# Spec: Build a cartridge image from program files

> **Deep lane · Phase 1 of 6 (Specify).** This document owns **what** and **why**. No tech
> stack, no file paths, no function names. Move all of that to `plan.md` in Phase 2.

**Slug:** `write-path-skeleton`
**Created:** 2026-08-10
**Related issue:** none
**Status:** draft <!-- draft | approved | implemented | abandoned -->

---

## Problem

stidcart is a new product with a wide eventual scope — a modern editor for Commodore 64 flash
cartridges. The riskiest thing about it is not the interface but whether it can emit a cartridge
image that a real machine will actually boot: the directory format, the reserved layout, the
firmware placement and the program placer must all be exactly right, and every one of them fails
silently. An image can be structurally plausible and still do nothing on hardware.

There is a way to settle that cheaply which will not stay available. The predecessor tool is the
only implementation known to produce images that boot on real EasyFlash hardware, so its output can
be used as a reference: build the same inputs both ways and compare bytes. That check only applies
where this tool is trying to reproduce the reference's output, so it is worth the most now and
becomes worthless as soon as the product's scope widens past what the reference can do. Building
the narrowest possible end-to-end path first, and checking it against that reference, converts the
project's largest unknown into a measured fact before anything is built on top of it.

## User stories

- As the maintainer, I want to produce a cartridge image from a set of program files, so that I have
  a working end-to-end path before any interface work begins.
- As the maintainer, I want the produced image to be byte-identical to the reference tool's output
  for the same inputs, so that correctness is a measured fact rather than an assumption.
- As the maintainer, I want the check to keep working after the reference tool is gone, so that
  today's evidence does not expire.

## Acceptance criteria

**AC-1** (surfaces: CLI): Given one or more program files, an output path and optionally a
cartridge name, when a build is requested, then a cartridge image is written to that path and the
command reports success.

**AC-2** (surfaces: CLI): Given the same program files and the same cartridge name used to produce a
stored reference image, when a build is requested, then the produced file is byte-identical to that
reference image.

**AC-3** (surfaces: CLI): Given the reference images are stored alongside the exact request that
produced each one, when the byte-identity check runs, then it passes without the reference tool
being installed or invoked.

**AC-4** (surfaces: CLI): Given an image produced by this tool, when it is run on an emulated
Commodore 64, then the cartridge menu appears and lists the entry name of each program included.

**AC-5** (surfaces: CLI): Given program files whose combined size exceeds the cartridge's capacity,
when a build is requested, then the command fails with a message identifying the problem and no
image is written.

**AC-6** (surfaces: none): The image's reserved region holds the menu program, the flash-writing
interface, the cartridge name and the boot code at the positions the format fixes for each, and
every byte of the reserved region not covered by one of them is left in the erased state.

**AC-7** (surfaces: none): Each directory entry records a type, a starting bank, an in-bank offset
and a size such that reading back the recorded location yields exactly the original program bytes,
including the two-byte load address the program begins with.

**AC-8** (surfaces: none): A directory entry's name is stored in the machine's own character
encoding rather than the host's, is padded to the fixed field width, and carries the reserved flag
bits the format requires to be set.

**AC-9** (surfaces: none): The directory is terminated by an end marker, and a request containing
more programs than the format's maximum entry count is refused rather than silently truncated.

**AC-10** (surfaces: none): A program larger than the space remaining in its starting bank continues
into the next storage region in the order the format prescribes, and reading it back across that
boundary yields the original bytes unbroken.

**AC-11** (surfaces: CLI): Given no program files at all, when a build is requested, then the
outcome matches the reference tool's behaviour for the same request, whatever that is determined to
be, rather than being decided by accident.

## Out of scope

- Reading, listing or editing an existing cartridge image. The read path has an unresolved question
  about which of the reference tool's quirks to reproduce; building the write path first is what
  defers that honestly.
- Any graphical interface. The command-line entry point here is a test harness, not a product.
- Inputs other than program files — existing cartridges as entries, and the entry types that go with
  them, come later.
- Ocean-mode cartridges, which relocate the menu program and constrain placement differently.
- Replacement menu firmware of our own. The existing firmware is embedded unmodified.
- Entry features beyond a plain visible program: hiding, forced alignment, subdirectories.
- The reference tool's text project format.
- Writing to hardware, and any cartridge target that is not EasyFlash.

## Open questions

All resolved by measuring the reference tool on 2026-08-10; recorded here because the answers are
non-obvious and each one changes what "byte-identical" means.

- [x] How is an entry's menu name derived when the request does not give one? — From the file name
  with its directory and extension removed, preserving case and spacing; the case swap that the
  machine's character encoding applies happens afterwards, at storage time.
- [x] Is a cartridge name always stored, even when the request omits one? — Yes. When omitted, a
  fixed placeholder string is stored, so the region is never absent. A build that skips it will not
  match.
- [x] Are two entries with the same menu name rejected? — No, they are accepted without complaint.
- [x] What happens with no program files at all? — The reference tool exits reporting success and
  writes no file. AC-11 pins matching this rather than inventing a different answer; whether to keep
  that behaviour long-term is a product question for after the skeleton.

---

<!--
Quality checks before approving this spec:
  - No tech stack named (no language / framework / database appears anywhere)
  - No file paths or function names
  - Every acceptance criterion follows `**AC-N** (surfaces: <CSV>): <body>`
  - Surface tokens are exactly from {CLI, HTTP, MCP, RENDER, DB, EVAL, none}
  - Every acceptance criterion is testable (you can describe an assertion that proves it)
  - Out-of-scope is filled in, even if just one bullet
  - Open questions either all answered, or explicitly flagged for /lcd:plan to resolve
-->
