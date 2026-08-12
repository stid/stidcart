# DECISIONS — stidcart

> **Append-only decision log.** Broader than an ADR: any decision worth not re-litigating —
> architectural, tooling, naming, process, content-organization — gets a block here. Newest at
> top. **Never rewrite history**: to change a past decision, add a new block and mark the old
> one `superseded by D-NNN`. Work-item-local decisions live in that item's `JOURNAL.md`; mirror
> here only the ones that outlive the work item. **Write the entry when the decision is made**
> (in-flight, context freshest), not retroactively at closeout — and keep the alternatives-rejected
> line honest: the entry is what a future session reasons from when it re-encounters the fork.

<!-- Next id: D-012 -->

---

## D-011 · 2026-08-11 · Describing an image and rebuilding it are held to different standards
- **Context:** `list` was built on a single directory walk shared by `load` and the new `inspect`, and
  the design note claimed the two "cannot disagree about what an image contains **or about which
  images they refuse**". A code review found the second half wrong twice over. The walk refused a
  hidden entry with the reason "the builder has no way to express hiding, so loading it would silently
  drop the flag" — a statement about *rebuilding* — and `inspect` ran `verify_regions`, whose entire
  purpose is to stop a rebuild silently swapping firmware. Both fired on a command that rebuilds
  nothing, and the first discarded entries already collected, so one hidden entry in slot 0 emptied
  the listing. Refusing to describe a cart because it could not be reproduced is the opposite of what
  an inspection command is for.
- **Decision:** Keep one walk; split the strictness. The walk refuses only what it cannot **describe**
  — malformed slots, entries reaching outside the image, and the well-formed shapes with no
  representation here. `load` layers the rebuild preconditions on top: the reserved regions must hold
  the embedded firmware, and no entry may carry a property `CartBuilder` cannot express. What an image
  *contains* is still answered in exactly one place, which was the property worth having; what may be
  *done* with it is answered by the caller that is going to do it.
- **Two consequences worth stating, since they change the model rather than just the control flow:**
  a hidden entry now decodes as `Slot::Program` carrying `hidden: true` rather than a `Slot::Hidden`
  that dropped its position and size — `decode` is a true inverse of `encode` again, and `load` finds
  the flag itself. A **cartridge entry stays refused in the walk**, which is not inconsistent: hiding
  is a `bool` the struct already had, while a cartridge entry needs xbank type modelling `CartInfo`
  does not have. The line is describability, not severity.
- **Half-confined entries (types `0x02`/`0x03`) are refused rather than implemented.** The same review
  found that nothing consulted `DirEntry.kind`: those entries were read with the interleaved ROML →
  ROMH → next-bank progression, so a round trip returned part of the entry followed by whatever
  occupied the other half, then rebuilt it as a plain `Prg` — corrupt in both directions, silently.
  Refusal rather than a correct reader, because this tool cannot produce one and the layout rule for
  them is not in the extracted format spec; deriving it from the C is what the format-facts invariant
  exists to prevent.
- **Alternatives rejected:** Relax the refusals for both paths — `load` would then accept a cart it
  rebuilds to different bytes, which is the defect `verify_regions` was added for. Give `inspect` its
  own walk — two implementations of directory decoding, free to drift on what an image contains, which
  is the one thing sharing the walk was for. Implement the half-confined layout now — it would commit
  to a bounds progression nobody has established, with no fixture to check it against, in a work-item
  about printing a listing.
- **Scope:** project-wide
- **Status:** active

## D-010 · 2026-08-11 · The Rust toolchain is pinned in-repo, not tracked as `stable`
- **Context:** [[D-009]] made `gate` a required status check on `main` with `enforce_admins: true`, and
  the gate runs `clippy -- -D warnings`. CI installed plain `stable`, so the toolchain was an input that
  changed on the Rust release calendar rather than on a commit here. A new stable adding one lint would
  fail CI on code nobody touched — and because the check is required with no owner bypass, it would
  block every PR *including the one that fixes it*.
- **Decision:** `rust-toolchain.toml` pins `channel = "1.97.1"` (current stable) with `rustfmt` and
  `clippy` on the minimal profile. CI runs `rustup toolchain install` with **no toolchain argument**,
  which installs the active toolchain — the one this file names. Naming `stable` in the workflow would
  silently un-pin CI while the file sat there looking authoritative. Upgrades are a deliberate PR:
  change `channel`, run the gate, fix any new lints in the same commit.
- **The pin binds only through rustup's shims — closed 2026-08-11, and worth knowing why.** A cargo that
  is not a rustup shim ignores `rust-toolchain.toml` outright: it does not warn, it just builds with
  whatever compiler it is. On the maintainer's machine both existed — Homebrew's `rust` formula at
  `/opt/homebrew/bin` and its keg-only `rustup`, whose shims live at `/opt/homebrew/opt/rustup/bin` and
  were on no `PATH` at all — so `cargo` resolved to the formula and this file was inert locally. Fixed by
  ordering `PATH` so the shims win, in **both** `~/.zshenv` and `~/.zprofile`: the latter's
  `brew shellenv` runs after the former and would otherwise put `/opt/homebrew/bin` back in front. The
  formula was left installed, just shadowed; uninstalling it is a destructive fix to an ordering problem.
  Verified by resolution (`rustup show` reports `overridden by '…/rust-toolchain.toml'`), **not** by
  version string — both were 1.97.1, so `rustc --version` could not tell working from broken.
- **Alternatives rejected:** Track `stable` and fix lints as they land — that is the deadlock above, and
  it arrives as a red `main` with no obvious cause. Keep `stable` but drop `-D warnings` — trades a
  known, occasional chore for permanently weaker static analysis, which is the half ndefpack lacked.
  Add `rust-version` (MSRV) to `Cargo.toml` instead — a different thing: MSRV declares the floor
  consumers must clear, not which compiler CI runs. Worth doing on its own terms, not as a substitute.
- **Scope:** project-wide
- **Status:** active

## D-009 · 2026-08-11 · `main` is protected behind a PR and one required check, `gate`
- **Context:** The remote had no protection at all, so "never commit to `main`" was a convention held
  up by nothing but a session reading `CLAUDE.md`. Auto-merge and auto-delete were both off.
- **Decision:** Protect `main`: PR required, force-push and deletion refused, conversation resolution
  required, `enforce_admins` on (no owner escape hatch — the constitution says never commit to `main`,
  so the setting matches the stated intent). Required approving reviews is **0**: a single-maintainer
  repo cannot approve its own PR, and 1 would deadlock every merge. One required status check, `gate`,
  from `.github/workflows/ci.yml`, which mirrors the `gate:` key in `lcd-conventions.md` step for step
  — CI and the local pre-commit check must not drift. `strict: true`, plus `allow_update_branch`, so a
  branch must be current with `main` before it merges. Auto-merge and delete-branch-on-merge on.
  `lcd-audit`'s `audit` job is required too, after dropping its `paths:` filter (see below).
- **The `paths:` trap, stated once so it is not re-derived:** a `paths:`-filtered job never reports on a
  PR outside the filter, and a required check that never reports sits in `Expected` forever — blocking
  the PR with no way to clear it. `lcd-audit.yml` was filtered to `docs/lcd/work/**`, which is why it
  could not be required *as written*. That is a property of the filter, not of the workflow: its script
  already exits 0 when no work-item changed, so the filter was redundant and removing it made `audit`
  requirable. Any future required check must report on every PR — filter inside the job, not in `on:`.
- **Alternatives rejected:** Require `lcd-audit` as it stood, keeping the filter — the `Expected`
  deadlock above. Leave it advisory — its own header claims it enforces the Deep-lane audit for every
  contributor and agent, and that was simply false while nothing blocked a failing audit. Protection with no
  required check at all — auto-merge would be decorative, since GitHub only offers the button when
  something is actually pending. Require 1 review — deadlocks a solo repo. `required_linear_history` —
  would conflict with merge commits, which are still an allowed merge method here.
- **Scope:** project-wide
- **Status:** active

## D-008 · 2026-08-10 · Crate version stays 0.2.0, and the header default keeps tracking it
- **Context:** Open question carried by write-path-skeleton since 2026-08-10. `versioning.md` bumped
  the minor on a `feat/` branch after the scaffold commit had already claimed `0.1.0`, so the crate
  merged to main at `0.2.0`. That string is the default CRT header name, so it sits inside every cart
  built without an explicit one; the write-path test only asserts the `stidcart ` prefix.
- **Decision:** Keep `0.2.0`, and keep the default deriving from `CARGO_PKG_VERSION`. The number is
  arbitrary before 1.0, no fixture is affected (parity pins the header to the reference tool's own
  string), and rolling it back would change produced bytes to buy nothing. The open question is closed
  as intentional rather than left to drift a second work-item.
- **Alternatives rejected:** Roll back to `0.1.0` — a bytes-changing commit whose only benefit is that
  a first release "should" be 0.1.0. Decouple the header default from the crate version so output is
  reproducible across releases — a real argument, but it discards the field's actual purpose of
  identifying the producing tool *and version*, and the read path preserves what it finds (D-007)
  rather than re-stamping, which is what would have made cross-version churn bite.
- **Scope:** project-wide
- **Status:** active

## D-007 · 2026-08-10 · Read-path divergence is decided per behaviour, not by a blanket rule
- **Context:** D-003 deferred this deliberately. The proposal on the table was to be bug-compatible
  with ndefpack on accept/reject now and fix each quirk later, on the reasoning that reproducing them
  keeps byte parity a clean pass/fail oracle through the read path.
- **Decision:** Split the list by the classification `reference/easyflash-cart-format.md` already
  carries, because the right answer differs per item. **Accept** an empty new-format directory (the
  note calls it a `break`-ordering slip and prescribes accepting). **Validate** the CHIP magic and the
  `0x04` total-length field as one decision, settling the read-by-`total` vs read-by-data-length fork
  on malformed input. **Keep** rejecting 256 valid slots with no end marker — the note marks that one
  format-mandated, since accepting it reads as an empty cart and destroys the directory on save.
  **Do not implement** the wide-16k-xbank bound or EasyFS1 type-guessing; both return a typed
  `Unsupported` error, because the write path cannot produce a cart entry yet and so neither has
  anything to round-trip against.
- **Why the stated reasoning did not hold:** parity compares produced bytes, and "rejected" has no
  bytes to compare — accept/reject was never under the parity oracle. The read path's oracle is the
  round trip against the frozen fixtures, and all four (`single`, `named`, `two`, `spill`) are
  well-formed PRG-only carts that cannot trigger any of the quirks. Bug-compatibility buys zero oracle
  strength on the tests this work-item writes. It also breaks self-consistency: `CartBuilder::new()
  .build()` succeeds today and emits a 16480-byte cart with the end marker in slot 0 — exactly the
  empty directory ndefpack rejects — so a bug-compatible reader could not read what this library
  writes, failing the zero-program base case of the round-trip property.
- **Alternatives rejected:** Blanket bug-compatible now, fix later — would mean writing code to reject
  a cart we produce, plus a test asserting a known-wrong answer that gets deleted when the fix lands.
  Fix everything now — would commit to an xbank bound the note says nobody has needed to pick, with no
  fixture to check it against.
- **Scope:** project-wide (supersedes nothing; it answers the question D-003 left open)
- **Status:** active

## D-006 · 2026-08-10 · CRT header name and EF-Name are separate caller-supplied fields
- **Context:** Byte parity against `ndeft` looked unsatisfiable: `ndeft` writes `PACKAGE_STRING`
  ("ndefpack 1.0.1") into the 32-byte CRT header name at offset `0x20` of every EasyFlash cart, and
  `-e` does not change it. `-e` sets the *EF-Name*, a different 16-byte PETSCII string living inside
  the flash image at `0x81b08` behind an `"ef-nAME:"` magic. Two strings, two offsets, easy to
  conflate — and conflating them produces a parity failure that reads like a deep bug.
- **Decision:** `CartBuilder` takes both independently. `header_name` defaults to `stidcart <crate
  version>`; `ef_name` defaults to the same `"(no EF-Name)"` placeholder `ndeft` writes when `-e` is
  omitted. The parity test sets `header_name` to `ndefpack 1.0.1` explicitly, so the divergence is an
  argument rather than a reason the test can never pass.
- **Alternatives rejected:** Hardcode the header name to the crate version — makes AC-2 unsatisfiable
  by construction. Mask the 32-byte range when comparing — hides real drift in a field that is part
  of the format, and the mask would have to be maintained forever.
- **Scope:** write-path-skeleton (the API shape it sets is project-wide)
- **Status:** active

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
