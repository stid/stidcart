# DECISIONS — stidcart

> **Append-only decision log.** Broader than an ADR: any decision worth not re-litigating —
> architectural, tooling, naming, process, content-organization — gets a block here. Newest at
> top. **Never rewrite history**: to change a past decision, add a new block and mark the old
> one `superseded by D-NNN`. Work-item-local decisions live in that item's `JOURNAL.md`; mirror
> here only the ones that outlive the work item. **Write the entry when the decision is made**
> (in-flight, context freshest), not retroactively at closeout — and keep the alternatives-rejected
> line honest: the entry is what a future session reasons from when it re-encounters the fork.

<!-- Next id: D-006 -->

---

## D-005 · 2026-08-10 · Firmware embedded as-is; custom menu later
- **Context:** The C64-side menu (EasyLoader), boot launcher and EAPI are zlib-licensed 6502 blobs
  from the EasyFlash project. Writing our own is wanted eventually.
- **Decision:** Embed the five `.prg` blobs unmodified in `firmware/`, keep them zlib, and treat a
  custom menu as separate later work. `firmware/README.md` records the licence boundary and where
  each blob lands in the image.
- **Alternatives rejected:** Write a new menu now — it is a 6502 project in its own right and would
  block the skeleton on unrelated work, with no oracle to check it against. Regenerate from ndefpack's
  acme sources — pointless, the committed blobs are byte-identical to what that produces.
- **Scope:** project-wide
- **Status:** active

## D-004 · 2026-08-10 · GPL-2.0-or-later for everything outside `firmware/`
- **Context:** The core is being written by reading ndefpack's GPL C and the extracted format spec.
- **Decision:** The workspace is GPL-2.0-or-later. It is a derivative work and there is no honest
  reading under which it is not. `firmware/` stays zlib, as it arrived.
- **Alternatives rejected:** Clean-room reimplementation to permit a permissive licence — would mean
  giving up both the spec and the differential oracle, which are the two things making this tractable.
- **Scope:** project-wide
- **Status:** active

## D-003 · 2026-08-10 · Write path first; read-path bug-compatibility left open
- **Context:** ndefpack's loader has documented quirks — an empty new-format directory rejected, the
  CHIP magic never checked, wide 16k xbank carts unreadable. A new implementation must choose whether
  to reproduce each one, and that choice changes how the test harness is built.
- **Decision:** Build the write path first. It has no divergence question at all, so byte parity is a
  clean pass/fail oracle there. The read path, and the per-behaviour divergence policy it needs, comes
  after — decided with real context rather than up front.
- **Alternatives rejected:** Decide the divergence policy now — would be guessing. Build the read path
  first — parity stops being pass/fail exactly where the quirks live, so the oracle would be weakest
  where it is needed most.
- **Scope:** project-wide
- **Status:** active

## D-002 · 2026-08-10 · `ndeft` is a differential oracle, and it decays
- **Context:** `ndeft` is the only implementation known to produce images that boot on real hardware
  (bank 63 confirmed on an EF3, 2026-08-09). No shared code, but its output is trustworthy.
- **Decision:** Freeze `ndeft`-built carts as fixtures with the exact command line for each, and gate
  the write path on byte-identical output. Front-load the work that benefits: parity only applies
  where stidcart is trying to reproduce ndeft, so its value is highest now and falls as compatibility
  widens. `-e` is pinned on both sides so the tool name in the CRT header is not a false difference.
- **Alternatives rejected:** Depend on `ndeft` being installed at test time — the fixtures must
  outlive ndefpack. Compare only structurally — misses exactly the byte-level drift that matters.
- **Scope:** project-wide
- **Status:** active

## D-001 · 2026-08-10 · Rust workspace: core library plus thin CLI
- **Context:** New product rather than a port of ndefpack. The GUI toolkit is deliberately undecided,
  and the goal is that deciding it later costs nothing.
- **Decision:** Cargo workspace with `stidcart-core` (library: format, placement, firmware; no global
  state, no I/O policy) and `stidcart-cli` (thin binary, understood as the test harness rather than a
  deliverable). A GUI crate is added later against the same library API.
- **Alternatives rejected:** Single binary crate — the split costs nothing now and retrofitting it
  later is what people never get around to. Port ndefpack's C in place — its global-singleton core is
  precisely what makes it unreusable, and the scope here is wider than ndefpack's anyway.
- **Scope:** project-wide
- **Status:** active
