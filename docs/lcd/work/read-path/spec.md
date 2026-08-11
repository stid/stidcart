# Spec: Read a cartridge image back into the description that produced it

> **Deep lane · Phase 1 of 6 (Specify).** This document owns **what** and **why**. No tech
> stack, no file paths, no function names. Move all of that to `plan.md` in Phase 2.

**Slug:** `read-path`
**Created:** 2026-08-10
**Related issue:** none
**Status:** draft <!-- draft | approved | implemented | abandoned -->

---

## Problem

stidcart can produce a cartridge image and cannot read one. Every capability the product is
eventually for — listing what is on a cartridge, editing it, adding an entry to one that already
exists — starts by turning an image back into a description of its contents. Until that exists the
tool can only ever create, never open.

Reading is also where correctness stops being obvious. Writing has one input and one right answer;
reading takes a file that may have been produced by another tool, an older version, or nothing
trustworthy at all, and every field in it is attacker-controlled in the sense that matters: a size
or a location that the image itself supplies, used without checking, indexes wherever it says. The
predecessor had exactly that defect, and the extracted format spec records it as a real memory-safety
bug rather than a hypothetical one.

There is a check available here that costs almost nothing and is strong: reading an image and
rebuilding it must reproduce the original bytes. That turns the read path into the write path's
inverse and makes the two verify each other, using reference images already frozen for the write
path rather than any new source of truth. It is a property the whole capability either has or
visibly lacks.

## User stories

- As the maintainer, I want to load a cartridge image into the same description that would produce
  it, so that every later capability that opens rather than creates has something to build on.
- As the maintainer, I want loading and rebuilding an image to reproduce it byte for byte, so that
  the read path is checked against the write path rather than against my reading of the format.
- As the maintainer, I want an image the tool cannot faithfully represent to be refused in a way
  that says so, so that an unsupported cartridge never becomes a silently wrong one.
- As the maintainer, I want a malformed image to be refused rather than trusted, so that a file from
  an unknown source cannot drive the tool outside the bounds of the image it describes.

## Acceptance criteria

**AC-1** (surfaces: none): Given a stored reference image, when it is loaded, then the result
describes the same programs as the request that produced that image — each entry's name, its
contents including the two-byte load address it begins with, and the order the entries appear in the
directory.

**AC-2** (surfaces: none): Given any stored reference image, when it is loaded and rebuilt with no
modification, then the rebuilt image is byte-identical to the original.

**AC-3** (surfaces: none): Given an image whose directory carries an end marker in its first slot
and no entries at all, when it is loaded, then it is accepted as a cartridge containing no programs
rather than refused as having no directory.

**AC-4** (surfaces: none): Given an image this tool produces from no programs at all, when it is
loaded and rebuilt, then the rebuilt image is byte-identical to it — the tool can read what the tool
writes, including at the empty base case.

**AC-5** (surfaces: none): Given a container whose data packets carry a corrupted leading marker, or
whose stated packet length disagrees with the data length the same packet declares, when it is
loaded, then it is refused rather than parsed onward from an assumed position.

**AC-6** (surfaces: none): Given a directory in which every slot holds a valid entry and no end
marker appears anywhere, when it is loaded, then it is refused — accepting it would read as an empty
cartridge and destroy the directory on the next save.

**AC-7** (surfaces: none): Given an image holding an entry that is itself a cartridge rather than a
program, or an entry marked hidden, or a directory in the older format, when it is loaded, then it
is refused with an outcome identifying the content as not yet supported, distinguishable from the
outcome for a malformed image.

**AC-8** (surfaces: none): Given an image, when it is loaded, then the producing-tool name recorded
in the container header is recovered exactly as stored, and a rebuild writes that same name back
rather than substituting this tool's own.

**AC-9** (surfaces: none): Given an image, when it is loaded, then the producing-tool name is read
in the host's character encoding while entry names and the cartridge name are read in the machine's
encoding, so that each of the three recovers with the capitalisation it was given.

**AC-10** (surfaces: none): Given an image whose reserved region holds firmware differing from what
this tool embeds, when it is loaded, then it is refused as not supported rather than accepted into a
description that would rebuild to different bytes.

**AC-11** (surfaces: none): Given an entry whose recorded location and size describe a program
continuing past the end of its starting region, when it is loaded, then the recovered bytes follow
that continuation in the order the format prescribes and match the original unbroken.

**AC-12** (surfaces: none): Given an image whose recorded sizes or locations describe a region
extending beyond the image, when it is loaded, then it is refused, and no read occurs outside the
bounds of the image as it was supplied.

## Out of scope

- Any command-line way to list or display what a loaded cartridge contains. This work-item delivers
  the capability; exposing it to a user is separate work.
- Editing a loaded cartridge — adding, removing, renaming or reordering entries, and writing the
  result back.
- Cartridge entries, hidden entries, forced alignment and subdirectories. Loading refuses them by
  AC-7; supporting each is its own work with its own reference images.
- The older directory format, and the lossy type-guessing its conversion requires. The extracted
  format spec is candid that the conversion can fail, and doing it differently is a separate problem.
- Choosing the size bound for wide cartridge entries. The format spec records it as an open question
  nobody has needed to answer, and refusing those entries means this work-item still does not.
- Recovering data present in the image but referenced by no directory entry. A rebuild does not
  preserve it.
- Rebuilding a cartridge whose entries were laid out by some other placement algorithm so that it is
  byte-identical. Contents recover, but positions are regenerated by this tool's placer, so only
  images whose placement already agrees with it will rebuild identically.
- The reference tool's text project format, and cartridges using the relocated-menu layout.

## Open questions

All resolved before planning. The first two were material enough to put to the maintainer rather
than settle by inference; both were answered on 2026-08-10.

- [x] What happens to an entry the menu hides, given the description has no way to express hiding? —
  Refused as not supported (AC-7). It is the same treatment the divergence policy already gives
  cartridge entries and the older directory format, no reference image contains one, and the
  alternatives were to widen the write path's interface inside a read-path work-item or to load the
  entry while silently losing the flag.
- [x] What happens when an image's reserved firmware differs from what this tool embeds? — Refused as
  not supported (AC-10). Rebuilding writes this tool's own firmware, so accepting a mismatch would
  turn byte-identity into a failure whose cause sits far from where it surfaces. The rejected
  alternative — carrying the image's firmware through into the rebuild — is more faithful but widens
  the description beyond programs and names and would allow emitting firmware never validated here.
- [x] How is the cartridge name reported when the image holds the placeholder written for requests
  that omit one? — Exactly as stored, placeholder included. Reporting it as "none given" would rebuild
  to the same bytes either way, so the choice is free; reporting what is there avoids a special case
  and keeps loading a plain description of the image.
- [x] Does a name recover exactly? — Yes across the character range the reference tool writes: the
  machine's encoding is a case swap and is its own inverse there. Control codes, the high range and
  anything past the field's terminator are lossy coming back. No reference image contains them, and
  this is a stated boundary of the encoding rather than a behaviour to decide on.

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
