# DECISIONS — stidcart

> **Append-only decision log.** Broader than an ADR: any decision worth not re-litigating —
> architectural, tooling, naming, process, content-organization — gets a block here. Newest at
> top. **Never rewrite history**: to change a past decision, add a new block and mark the old
> one `superseded by D-NNN`. Work-item-local decisions live in that item's `JOURNAL.md`; mirror
> here only the ones that outlive the work item. **Write the entry when the decision is made**
> (in-flight, context freshest), not retroactively at closeout — and keep the alternatives-rejected
> line honest: the entry is what a future session reasons from when it re-encounters the fork.

<!-- Next id: D-031 -->

---

## D-030 · 2026-10-11 · The GUI is Tauri 2.x, in its own Cargo workspace with its own required CI job
- **Context:** the GUI's first criterion is a polished look on par with VS Code, Zed or Obsidian,
  with full control over style. Three independent analysts (`/stid-decide`) researched it against
  hard limits: a stable release, a required `gate` that stays small, and automated UI tests. The
  user relaxed "tests run from `cargo test`" to "tests run automatically in CI". The user also
  ruled that the small-footprint limit covers the `gate` only, that the GUI job is a required
  check, and that Linux is a portability check, with macOS the shipping target.
- **Decision:** Tauri 2.x (2.12 at the time), pinned to the 2.x line, with no 3.0 alphas.
  - The GUI lives outside the root workspace (`exclude`), as its own workspace with a path
    dependency on `stidcart-core`, so `--workspace` in the `gate` never compiles webkit and the
    gate string stays unchanged.
  - A separate required CI job on ubuntu-latest installs webkit2gtk, then runs fmt, clippy and the
    Rust tests plus the frontend tests (Vitest with Tauri's `mockIPC`) under Node.
  - Serde DTOs and all file reading live in the GUI crate, so the core stays free of serde and
    I/O policy.
- **Alternatives rejected:**
  - egui 0.36, ranked second by two analysts and first by the third. It passes every hard limit
    with no gate change, and its UI tests run headless in `cargo test`. Its polish at this bar is
    proven only by Rerun, a funded team. Rerun's `re_ui` is 18.8k lines, but most of that is
    product features; the theme layer itself is about 1.4k lines plus three RON files. There is no
    reusable kit, so every component would be hand-built, and egui's README calls layout a
    fundamental shortcoming of immediate mode. GitButler is the same class of proof for Tauri, but
    on Tauri the styling and component tooling is commodity CSS. The polish bar was weighted first.
  - Slint 1.18. Its default backend cannot receive files dropped from Finder (it has no winit
    drop handler), its test crate is internal and must be pinned to an exact version, and the
    binary would ship as GPLv3.
  - iced 0.14. No release in 10 months (about 15 months between releases) and no accessibility.
  - gpui. The crates.io release is 12 months stale and the maintained line is pre-release
    snapshots.
  - Dioxus/Blitz. Same webview cost as Tauri with weaker testing, or still in beta.
- **Accepted costs:** a second toolchain (Node, a frontend framework); a second lockfile and target
  dir; a TypeScript copy of the entry model that can drift from Rust; a 30–50 s apt step (cacheable)
  in the GUI job; WebKit rather than Chromium (WKWebView on macOS); a future 2→3 migration whose
  size is unknown. Tauri has no written 2.x support policy; the only evidence is that 1.x got three
  patch releases over 7.5 months after 2.0.
- **Scope:** new `gui/` workspace; CI workflow; MAP zones; lcd-conventions test placement
- **Status:** active

---

## D-029 · 2026-10-11 · The subdirectory bit is carried on xbank entries; type 0x1A stays refused
- **Context:** "subdirectory" names two things in the format: the `0x4000` CrtUsage bit and the
  type `0x1A` EasyFlash entry. Both were refused as one unsupported kind (D-026). Checked against
  the reference: `ndeft` carries the bit on an xbank through load, rename and save (only the name
  byte changed in a re-save), rejects it on any other cart type (`easyfs.c:412`), and refuses type
  `0x1A` outright (`easyfs.c:500-502`). Nothing places or boots by the bit; EasyLoader boots `0x1A`
  as Ultimax (`easyloader.asm:912`). Format spec, "Subdirectories".
- **Decision:** the model carries the bit (`DirEntry::subdir`, `Cartridge::subdir`) on xbank
  entries, with `list` showing it as `S`. The bit on another cartridge kind is malformed at that
  slot (`ReadError::SubdirectoryBitMisplaced`) in both read paths. Type `0x1A` stays refused as
  unsupported. No input sets the bit, and `--replace` takes it from the new data, which never has
  it, as `align_64k` works.
- **Alternatives rejected:** modelling `0x1A` as a tenth cartridge kind (Ultimax geometry, as
  EasyLoader treats it). That is plausible, but there is no oracle: the reference refuses it and
  no image in reach holds one, so it waits for a VICE or hardware check (MAP: the oracle decays).
  Accepting EasyFlash `.crt` inputs as `0x1A` entries ("create"). The reference refuses them and
  nothing says how a nested cart's banks are addressed. Refusing the misplaced bit as unsupported
  rather than malformed: the reference calls it an error, and no layout gives it a meaning there.
- **Scope:** `stidcart-core` `directory`, `cartridge`, `read`; `stidcart-cli` listing flags
- **Status:** active

## D-028 · 2026-10-04 · Nothing is placed once the top is used up, not even an empty program
- **Context:** when the top run is exactly used up, `top()` points past bank 63, and placing a
  zero-length program there indexed out of bounds and panicked. `edit` reached this on a cart
  holding a 0-byte entry when an added program filled flash exactly. `list` reached it through
  D-027's probe, which turned a describable cart into a crash (D-011). The reference reads the same
  out-of-range slot (`place.c:336`, undefined behaviour). It only escapes because `prg_load`
  refuses empty input (`prg.c:50`), which never covers an entry read from a stored directory.
- **Decision:** `Placer::place_interleaved` returns `None` (out of space) when the run starts past
  the last bank. The build reports out-of-space naming the entry, and the free figure stays
  consistent with that: beside a 0-byte program, filling the last byte does not fit.
- **Alternatives rejected:** recording an empty program at some in-bounds position past the data.
  The offset field cannot encode "after bank 63 ROMH", so that position would be fabricated.
  Refusing zero-length programs at input or on load. That is a new refusal, and `load` refusing it
  would also stop `edit` on carts it handles today whenever space is left.
- **Scope:** `stidcart-core` `Placer::place_interleaved`
- **Status:** active

## D-027 · 2026-10-04 · The listed free figure is what a re-placement accepts, not the top run
- **Context:** `list` printed the top run (`calc_top_space`) replayed from the stored positions.
  In normal mode that equals what `edit --add` accepts; in Ocean mode it does not. cocean512
  listed $2000 free, while `edit` accepts a $7a000 program, because a rebuild re-places every entry
  (D-019) and a confined program can take ROMH banks 2..62 (issue #29).
- **Decision:** `CartBuilder::free_space` answers "how large a program can be added" by running
  `Placer::place_all` on the entries plus a probe program appended last, as `--add` appends it.
  The search keeps a size known to fit and one known to fail, so the figure always fits and one
  byte more always fails. Placement is greedy, so fit is not proven monotone in size and the
  figure is a boundary, not a proven global maximum. `list` uses it when the image loads. An image
  `load` refuses cannot be edited at all, so it falls back to the replay figure, which describes
  the file as stored (D-011).
- **Alternatives rejected:** adding the largest stored gap to the replay figure. It undercounts,
  because `edit` moves existing entries out of the gap (cocean512's `hello` sits at bank 2 ROMH and
  moves to 63). Probing `build_image` instead of the placer. Its other refusals do not depend on
  probe size, but `TooManyEntries` would turn a full directory into "0 bytes free", mixing two
  separate limits. Claiming "largest" and searching a wider range for non-monotone fits. Nothing
  shows a non-monotone case, and AC-46 asserts the boundary only.
- **Scope:** `stidcart-core` `CartBuilder::free_space`, `stidcart-cli` `list`
- **Status:** active

## D-026 · 2026-10-04 · cart-entries shipped: cartridge images are entries, and the kind refusal narrows to subdirectories
- **Context:** Until cart-entries, both read paths, `build` and `edit` handled PRG entries only and
  refused any cart holding a cartridge entry or a half-confined program as "an entry kind it cannot
  represent" (D-019's refusal clause).
- **Decision:** the nine cartridge kinds and both half-confined program kinds are modelled end to
  end — input derivation (D-023), placement (D-025), Ocean mode (D-024), the wide-xbank bound
  (D-022), read, list and every edit operation by index. The one kind still refused is the
  EasyFlash subdirectory entry. Capabilities indexed in SPEC.md under AC-31..AC-46.
- **Alternatives rejected:** see D-022..D-025 — this entry is the SPEC anchor for the shipped set,
  not a new fork.
- **Scope:** project-wide
- **Status:** active · refusal narrowed further to type `0x1A` by D-029

## D-025 · 2026-10-04 · Cartridges are placed before programs regardless of weight
- **Context:** The reference sorts every entry by one weight and treats the first `num_crt`
  positions as cartridges (`place.c:577-615`). A one-bank cart weighs `1 << 16` and a program
  weighs its size, so a program over 64 KiB sorts among the carts: the reference would hand it to
  `place_crt`, and this port hit an `unreachable!` and panicked (`edit --add` of a 0xe2000-byte
  program to `cmixed.crt`).
- **Decision:** the sort key is (is cartridge, weight), descending. Carts keep the reference order
  among themselves, programs keep theirs, and carts always come first, which is what AC-33 states.
  No frozen fixture mixes a program over 64 KiB with a cart, so parity is unchanged.
- **Alternatives rejected:** reproducing the reference — its output for this input is a program
  placed as a cartridge (inferred from `place.c`; ndeft was not run on it), not a layout anyone depends on. Capping program size below 64 KiB — a new
  refusal for inputs the format allows.
- **Scope:** `stidcart-core` `Placer::place_all`
- **Status:** active

## D-024 · 2026-09-19 · Ocean mode is derived from the entry set, never declared
- **Context:** An Ocean cartridge must own bank 0 ROML, which is where the normal-mode menu loader
  lives. The reference tool answers this by switching the whole image into "Ocean mode": the loader
  moves to bank 1 ROMH, a different launcher blob goes in the boot region, and placement reserves
  bank 1 ROMH instead of bank 0 ROML (`loader.c:37-41`, `place.c:670-685`). `cart-entries` needed a
  home for that switch in a builder that had only ever known one mode.
- **Decision:** the mode is a function of the entries — Ocean if any entry is an Ocean kind, normal
  otherwise — computed at build time and at load time (from the directory, before the firmware
  regions are verified against the blobs that mode implies). Nothing on the builder or the command
  line declares it. A second Ocean entry is a typed build error naming it, not a placement failure.
- **Alternatives rejected:** a `with_mode` on the builder — a declared mode that disagrees with the
  entries is either an unbootable cart or a silent override, and no user need was named for
  either. Verifying firmware before reading the directory (today's order) — it cannot know which
  loader to expect, so `load` reorders instead.
- **Scope:** `stidcart-core` reserved regions, placement init, `load`; no CLI surface
- **Status:** active

## D-023 · 2026-09-19 · Cartridge entries are one entry type with the reference derivation, and short packets are refused
- **Context:** `cart-entries` widens the model from programs to the nine cartridge kinds. Three
  shapes had to be fixed before planning: how a cartridge sits beside programs in the builder, what a
  cartridge holds, and how an input image becomes one.
- **Decision:** one `Entry` enum (`Program | Cartridge`) in one vector, so every index-addressed
  operation applies to both. A `Cartridge` holds its kind, the alignment request, and two whole-bank
  runs (ROML, ROMH) with each run's bank offset from the entry's start bank; the directory's CrtUsage
  word and size are derived from those at encode time. A cartridge is only ever *derived* from an
  image by the reference tool's rules (`cart.c:414-527, 628-800`): id 0 → generic kinds by packet
  addresses, id 5 → Ocean kinds by size, id 33 → xbank kinds by header mode; any other id is refused.
  Packets must be whole banks — the reference accepts a generic packet as short as `0x100` bytes and
  then records a size its own reader decodes as zero primary banks, so that leniency is classified an
  implementation artifact and not inherited (D-007).
- **Alternatives rejected:** a second vector of cartridges (two index spaces, every mutator twice);
  the reference tool's two storage layouts (a flat L-then-H buffer, or a full 1 MB image for xbank —
  conveniences that write the same bank copies); a user-declared kind on the command line (the image
  already says what it is, and a declaration that disagrees is a cart that does not boot).
- **Scope:** `stidcart-core` model, directory, read path; CLI input handling
- **Status:** active

## D-022 · 2026-09-19 · A cartridge entry is bounded by flash geometry, not by a flat 512 KiB
- **Context:** The reference tool refuses any directory entry larger than `0x80000` when it reads a
  directory (`easyfs.c:432`), which makes a 16k xbank wider than 32 banks unreadable there. The format
  spec carried the bound as an open question — per-type or a flat 1 MB — because the size field is
  24 bits and the bank fields address all 64 banks, so nothing in the encoding imposes the cap. The
  `cart-entries` work-item had to settle it before placement could be specified.
- **Decision:** an entry is accepted whenever the banks it describes fit in flash outside the
  reserved bank, in both halves. That is the same geometry check every entry needs for memory
  safety (the bank-run bounds the spec records as a real heap overflow in the reference tool), so
  it adds no second rule. Reading follows the same bound: an image the reference tool refuses on
  size alone is described and loaded here.
- **Alternatives rejected:** reproduce the flat `0x80000` — it is a literal check in that reader,
  not a field width, and D-007 says a quirk the spec classifies as an implementation artifact gets
  fixed, not inherited. Per-type bounds (8k vs 16k xbank) — a second table to keep right for no
  case the geometry check does not already refuse.
- **Evidence:** the oracle cannot cover this case, since the reference tool refuses it (D-002
  decay). A round-trip test here plus a boot in VICE, to be recorded in the work-item JOURNAL when
  done.
- **Scope:** `stidcart-core` model, placement and both read paths
- **Status:** active

## D-021 · 2026-09-19 · The CRT header length field is honoured on read, not assumed
- **Context:** The container spec defines the field at `0x10` as the offset of the first CHIP packet
  and allows it to exceed `0x40`. ndefpack writes `0x40` and then ignores the field on load, starting
  the walk at `0x40` unconditionally. `crt::parse` inherited that: it read the magic and CRT ID from
  the header and never looked at the length. The format spec's "behaviours to decide on" list
  carried this as the last undecided pure-container item; the other two (an empty directory
  accepted, the packet marker and total length validated) were decided and built with the read path.
- **Decision:** the packet walk starts at the offset the field states. A value below `0x40` is refused
  as malformed — the fields the reader has already used live above that offset, so a smaller header
  cannot be one — and a value past the end of the file is the same truncation as any other. The
  writer is unchanged and keeps writing `0x40`, so produced bytes and fixture parity are unaffected.
- **Alternatives rejected:** keep assuming `0x40` — reproduces a quirk the spec classifies as an
  implementation artifact, for no gain. Validate the field equals `0x40` and refuse anything else —
  stricter than the container spec, and would refuse a well-formed cart from a writer that pads the
  header. Ignore a bad value and fall back to `0x40` — silently reading a file the writer described
  differently is exactly the class of leniency D-007 argues against.
- **Scope:** `stidcart-core` read path only (`crt::parse`); `docs/format/` corrected in the same PR
- **Status:** active

## D-020 · 2026-08-14 · The entry model carries the hidden flag, retiring the hidden-entry rebuild refusal
- **Context:** `read.rs:344` refuses any cart containing a hidden entry when reading it back for
  rebuilding, with the message "cannot be rebuilt unchanged yet". The refusal is not a format fact —
  the flag is bit 7 of the entry flags byte (`docs/format/easyflash-cart-format.md:119`) and
  ndefpack round-trips it. It exists because the builder had nowhere to put the flag, so a load would
  have silently dropped it. `cart-edit` adds toggling hidden as a user-facing operation, which means
  the model has to carry the flag regardless.
- **Decision:** the flag becomes part of the entry model, on both the write and read sides, and the
  `HiddenEntry` refusal is removed. A cart with hidden entries loads, edits and saves; an untouched
  load-and-save of such a cart is byte-identical (spec AC-29). Describing such a cart was never
  refused (D-011), so this closes the gap in the other direction — the rebuild path stops being the
  narrower of the two for a reason that no longer holds.
- **`build` gains the marker too.** The alternative — hidden settable only by editing — keeps
  `build`'s surface frozen, but produces a tool whose two write paths disagree about what a cart can
  contain, and the fixture parity it protects is unaffected as long as nothing is marked (spec
  AC-28).
- **The firmware refusal is not the same kind of thing and stays.** A cart whose reserved regions
  differ from the embedded blobs is still refused for rebuilding, and this entry is not an argument
  against that: the hidden refusal was a gap in the model, closed by widening the model, while a save
  genuinely rewrites bank 0 from the blobs it carries. Retiring the firmware refusal would silently
  replace a user's firmware, not preserve it.
- **This is a lossy-model fix, not a read-path divergence decision.** D-007 governs which ndefpack
  behaviours to reproduce; nothing here reproduces or diverges from one. The refusal was this repo's
  own, and it is retired by removing the reason it existed.
- **Scope:** `stidcart-core` read and write paths, `build` and `edit` CLI surfaces (narrows the
  refusal set recorded in `SPEC.md`'s Internal section)
- **Status:** active

## D-019 · 2026-08-14 · An edit re-places the whole cart and saves in place by default
- **Context:** `cart-edit` reads a cart into builder state, mutates the entry set and writes it back.
  Two forks had to be settled before planning: whether entries the user did not touch keep their
  stored bank and offset, and where the result lands.
- **Decision — re-place everything.** Saving an edited cart produces exactly the bytes a fresh build
  of the resulting entry set produces (spec AC-30). There is one placement path, the fixtures oracle
  keeps applying to it, and the described-versus-rebuilt split (D-011) stays where it is: `list`
  still reports positions as stored, and only the rebuild path re-places.
- **Preserving placement was the alternative, and it costs a second placement mode.** Fitting
  additions into the gaps left by removals would make an edited cart diffable against its input, but
  it needs a fill-the-holes placer beside the largest-first interleaved one, no fixture can pin its
  output, and it can refuse an addition that a rebuild would have fit. Not worth it for a diff nobody
  asked for. The one byte-identity that does matter — reading a cart and saving it back with no
  operations applied — holds anyway, and is pinned by AC-29.
- **Decision — in place by default, a named destination redirects.** The reason to require a
  destination is that a typo cannot then destroy a cart; the reason not to is that editing your own
  cart is the common case and a two-file dance for every change is friction the tool would be adding
  for itself. The safety this buys is bought instead by AC-24: a failed edit leaves every input
  byte-identical, so the failure mode the read-only default guards against is closed directly.
- **Scope:** the `edit` CLI surface and the rebuild path in `stidcart-core`
- **Status:** active; the "entry kind it cannot represent" refusal narrowed to subdirectory entries by D-026

## D-018 · 2026-08-14 · The format spec lives in the tree at `docs/format/`, not in the vault
- **Context:** `reference/easyflash-cart-format.md` in the Obsidian vault was the source for every
  offset in `stidcart-core` — an invariant in `MAP.md` and an instruction in `CLAUDE.md`. The vault
  is not a git repo. PR #20 turned that from a tidiness question into a concrete failure: D-017
  rests on a correction to that spec, the correction landed as a plain file edit, and the reviewer
  saw a citation to a change absent from the diff. `gate`, `audit` and `claude-review` all ran
  against a tree that did not contain the document their subject depends on, and the wrong claim had
  already been copied into `directory.rs:12,67` with a test pinning it — drift a reviewer could not
  see, because the copy and the original were not diffable.
- **Decision:** the spec moves to `docs/format/easyflash-cart-format.md` and the vault keeps a stub
  pointing at it. A format fact is now corrected there first, in the same PR as the code it
  justifies. **Supersedes the vault clause of the format-facts invariant** (`MAP.md`); the
  invariant itself — facts come from the spec, never re-derived from ndefpack's C — is unchanged and
  is the whole reason the document has to be reviewable. `projects/ndefpack.md` stays in the vault:
  it is the predecessor's decision log, not a spec this codebase owns.
- **`docs/format/` over the alternatives.** `spec/` at the root would leave D-014's docs zone
  untouched but adds a top-level directory for one file. Dropping the file into `docs/` directly
  would mean deleting the zone's anti-sprawl clause outright. Splitting the zone in two keeps the
  guardrail and moves it down a level: nothing non-LCD directly in `docs/lcd/`.
- **The licence question is settled, not assumed.** The spec was written by reading ndefpack's GPL
  C, so it is a derivative work by exactly D-004's argument and GPL-2.0-or-later is where it already
  belonged. Publishing the repo changes nothing — the obligation travels with the licence. The
  moved file says so in its own header.
- **The move is verbatim.** Only the Obsidian frontmatter, the two `[[wikilinks]]` and the closing
  "Related" line changed, plus an added header and licence note; `diff` against the vault original
  shows nothing else. Correcting what the spec *says* is separate work needing its own `file:line`
  evidence — the point of this change is that such a correction is now reviewable, not that any
  were made.
- **Scope:** project-wide (supersedes the vault clause of the format-facts invariant in `MAP.md`;
  D-014's `docs/lcd/` artifact root is unchanged)
- **Status:** active

## D-017 · 2026-08-12 · The reserved flag bits are the EasyFS version discriminator, not a quirk to tolerate
- **Context:** D-007 decided an EasyFS1 directory returns a typed `Unsupported` error, and the code
  never implemented it — `SPEC.md:58` records that every directory is walked as EasyFS2 and misread.
  It looked blocked: the vault format spec documents EasyFS2 only, and the format-facts invariant
  forbids deriving the missing fact straight out of ndefpack's C into code. The research was cheap
  and the blocker dissolved.
- **The discriminator is bits 5–6 (`0x60`) of the flags byte at offset 16.** Not both set means old
  EasyFS (`efs_check_entry`, `easyfs.c:371-389`). The first non-end slot decides for the whole
  directory; a later slot disagreeing is a hard error, not a reclassification. The end marker is
  `0xFF`-filled, so its bits are set and it never classifies.
- **The vault spec was wrong about these bits, and the error had been copied into this repo.** It
  read "A past ndefpack bug wrote them as 0 (`NEWS`); anything reading these carts should tolerate
  both" — wrong tool, wrong citation, and a prescription that cannot be satisfied. `README:82` is a
  2012 comparison against **Draco Cart Maker**, a different tool; `NEWS` says nothing about reserved
  bits; and ndefpack's writer ORs `EFS_RESERVED_MASK` unconditionally (`easyfs.c:581`), so clear bits
  are unambiguously not its output. A reader cannot both ignore these bits and detect the version —
  tolerating a clear pair as EasyFS2 gives up the only signal the format has. `directory.rs:12,67`
  repeated the claim and `decode_tolerates_the_reserved_bits_being_clear` pinned it as a test.
- **Decision:** the vault spec is corrected first (the invariant's point), then the read path checks
  the bits. An old-format directory is refused with a typed `Unsupported`, and slots that disagree
  are refused as malformed rather than as old. The **writer guidance is unchanged and still
  load-bearing**: `encode` sets the bits, and correcting the reader must not take that with it.
- **Refusing to *describe* an EasyFS1 cart is consistent with D-011, not an exception.** That
  decision drew the line at describability: EasyFS1's bank is a single byte at offset 17
  (`easyfs.c:255-260`), so read as EasyFS2 every entry gets a corrupted bank. There is no accurate
  description to give, which is a different situation from a hidden entry the walk can describe
  perfectly well and only `load` needs to refuse.
- **Alternatives rejected:** keep tolerating clear bits and detect EasyFS1 some other way — there is
  no other way; the format carries no version field, which is why these bits are it. Implement the
  EasyFS1 reader rather than refusing — D-007 already decided against, its cart types are guessed
  from size (the source carries two `WARNING: assumes …` comments), and there is no fixture to check
  a reader against. Treat a Draco-written cart as EasyFS2 by special case — nothing distinguishes it
  from a genuine EasyFS1 cart, so the special case would have to be a guess.
- **Scope:** project-wide (supersedes nothing; it implements what D-007 decided and corrects the
  format-spec fact D-007 relied on)
- **Status:** active

## D-016 · 2026-08-12 · A PR that edits `claude-review.yml` cannot pass its own review gate
- **Context:** PR #18 moved the artifact root and rewrote the doc paths inside the reviewer prompt
  in `.github/workflows/claude-review.yml`. The action then refused to run: *"the workflow file
  must exist and have identical content to the version on the repository's default branch."* That
  is a deliberate guard — without it a pull request could rewrite the prompt of the reviewer
  judging it — but the step produces no verdict, and D-012's gate step fails closed on an empty
  verdict. `claude-review` is a required check with `enforce_admins: true`, so the PR could not
  merge, including by the repository owner.
- **This is unavoidable, not a bug to fix.** Any change to `claude-review.yml` fails that PR's own
  `claude-review`. There is no ordering that escapes it: a follow-up PR touching the same file hits
  the identical wall. The action says so itself — the workflow starts working once the PR merges.
- **Decision:** when a PR must change `claude-review.yml`, expect one red required check and clear
  it manually — temporarily drop `claude-review` from the required contexts (or `enforce_admins`),
  merge, restore immediately. Restoring is the load-bearing half; an unprotected `main` left that
  way is a worse outcome than the stale prompt this was fixing.
- **Consequence worth weighing before editing that file:** batch reviewer-prompt changes rather
  than dribbling them across PRs, since each one costs a protection round-trip. The alternative —
  making the prompt reference documents indirectly, through `CLAUDE.md`, so paths never appear in
  the workflow — was offered and declined for now; it remains the way to stop paying this.
- **Alternatives rejected:** revert the path fix to keep the PR green — the reviewer would then be
  pointed at `docs/MAP.md` and `docs/DECISIONS.md`, which no longer exist, degrading quietly on
  every subsequent PR to buy one green check.
- **Scope:** project-wide
- **Status:** active

## D-015 · 2026-08-12 · `SPEC.md` is canonical for current behaviour; the MAP stops restating it
- **Context:** turning on the living spec adds a third place that describes what the tool does now,
  alongside `MAP.md`'s "Current state" paragraph and `README.md`'s State section — which said
  nearly the same thing in nearly the same words. Three copies drift in three directions.
- **Decision:** `living-spec: on`. `docs/lcd/SPEC.md` is the capability index and the canonical answer
  to "what does this do now". `MAP.md`'s Current-state paragraph is deleted and replaced by a
  pointer — the MAP is structural, and behaviour was never its job. `README.md` keeps its State
  section: it is the public front door, and a README that does not say what the tool does is worse
  than one sentence of overlap. The division is narrative (README) versus enumerated capabilities
  with provenance (SPEC).
- **The initial rows are a lower-confidence tier.** They were derived from `MAP.md` and the code on
  2026-08-12, not from acceptance criteria — no ACs existed for them and no cross-path audit has
  run against them. They are cited `D-015 (backfill)` and carry a banner at the top of `SPEC.md`.
  Each decays into an AC-pinned row the next time a work-item touches that capability.
- **The same rule is applied to the conventions adapter.** The plugin's template restates a
  project's invariants inside `Constitution notes` so they survive a fresh clone. Here they already
  live in `MAP.md`, which is in the clone, so the adapter points at that section instead of copying
  it — a deliberate deviation from the template, for the reason this decision exists. Keep it
  across `/lcd:onboard` refreshes.
- **Alternatives rejected:** leave `living-spec: off` — the whole point of the re-onboard was to
  evaluate what 0.15.0 added, and this is the headline addition. Delete README's State section to
  remove the overlap outright — that optimizes the agent's reading at the cost of the human's.
- **Scope:** project-wide
- **Status:** active

## D-014 · 2026-08-12 · LCD returns at plugin 0.15.0, rooted at the default `docs/lcd/`
- **Context:** D-013 removed the pipeline earlier the same day, on a real measurement: 1,855 lines
  of process docs against 1,978 lines of Rust. What changed is the tooling, not the measurement —
  the plugin is now at 0.15.0, which is not the version D-013 was written against, and the
  re-evaluation is deliberate rather than a walk-back of the reasoning.
- **Decision:** re-onboard at `artifact-root: docs/lcd`, the plugin's default. `MAP.md` and
  `DECISIONS.md` were `git mv`d back under it from `docs/`, where D-013 had promoted them, so this
  log keeps its history instead of being reborn empty beside itself. Work-items land at
  `docs/lcd/work/<slug>/`.
- **A shallower root was tried first and rejected in review.** This entry originally chose
  `artifact-root: docs` — reasoning that MAP and DECISIONS already lived there, and that the default
  would strand D-001…D-013 behind a freshly written empty pair. The stranding risk is real; the
  remedy is a `git mv`, not a shallower root. The shallow root breaks an assumption no config key
  states: `audit-crosspath.sh` derives the artifact tree it must exclude from its test-hit search as
  `${specs_dir%/*}` — the *parent* of `<root>/work`. At `docs/work` that resolves to `docs/`, which
  silently excludes every match anywhere under `docs/` rather than just the work-item tree; at
  `docs/lcd/work` it resolves to `docs/lcd`, which is what the script means. `claude-review` caught
  it on PR #18, before any work-item existed to be mis-audited. Corrected on the same branch, before
  merge — nothing was rewritten that had ever reached `main`.
- **The lesson generalizes past that one script.** A non-default `artifact-root` is not a free
  parameter: the suite carries assumptions about the *shape* of the tree that the machine block does
  not express. Prefer the default and move the files.
- **What returns:** triage and lane routing, per-work-item folders, the conventions adapter at
  `.claude/rules/lcd-conventions.md`, the three script permissions in `.claude/settings.json`, and
  `lcd-audit.yml` as a PR check. At the default root the bundled `ac-convention.md` auto-loads as
  shipped, so no project-local override is carried.
- **Why the audit gate comes back:** D-013 dropped it because it had degenerated into a no-op — it
  verifies every acceptance criterion has a test, and no acceptance criteria exist without the
  pipeline that produces them. With the pipeline back, that reason is void. D-009's caution about a
  check that stops reporting still applies, which is why the workflow is scoped to PRs touching
  `docs/lcd/work/**` and skips any work-item without both `spec.md` and `plan.md`.
- **What D-013 got right is kept.** `claude-review` is the check that reads the code, and it is the
  one that found seven defects — two severe — that the audit passed over. Nothing here demotes it,
  and the audit is not being sold as a substitute. The doc-to-code measurement stands as the test:
  if the ratio goes the same way again, this is the decision to revisit.
- **Alternatives rejected:** stay removed — reasonable, but the cost of re-evaluating at a new
  version is one branch, and the decision is reversible by deleting the same files again. Shallow
  `docs/` root — see above; it survived one round of review and then didn't.
- **Supersedes:** D-013
- **Scope:** project-wide
- **Status:** active

## D-013 · 2026-08-12 · The planning pipeline is removed; this log and the MAP are what survive
- **Context:** Measured on this repo: 1,855 lines of process docs against 1,978 lines of Rust, and
  the twelve commits after the list-command feature merged — four PRs — touched `crates/` zero
  times. 1,483 of those doc lines were per-work-item `spec`/`plan`/`tasks`/`JOURNAL`/`audit`, five
  documents for each of three finished items, whose purpose was surviving a context reset *during*
  work that is now done.
- **Decision:** Delete `docs/lcd/work/` and the triage/lane machinery with it. Keep `DECISIONS.md`
  and `MAP.md`, promoted to `docs/`. The gate command and branch discipline move into `CLAUDE.md`,
  which is where a session already looks. New work starts by writing a failing test.
- **`audit` is dropped as a required check** and `lcd-audit.yml` deleted: it verified that every
  acceptance criterion had a test, and acceptance criteria only existed because the pipeline
  produced them. Without that, it is a no-op check that still has to be maintained — and D-009
  records what a required check that stops reporting does to a PR. `gate` and `claude-review` stay.
  *(D-014 reinstates it, scoped to PRs that touch `docs/lcd/work/**`.)*
- **`claude-review` is the piece that earned its keep**, which is the honest counterweight here:
  the audit passed on list-command and the review then found seven defects, two severe, in code no
  acceptance criterion described. Process that reads the code paid; process that counted documents
  did not.
- **Alternatives rejected:** Remove everything including this log — D-006 alone (CRT header name vs
  EF-Name) saves a future session from a parity failure that reads like a deep bug, and D-007/D-011
  encode read-path policy that is genuinely hard to re-derive. Keep the pipeline but raise the
  threshold — it is a judgment call every session, and it had lost all three times.
- **Scope:** project-wide
- **Status:** superseded by D-014

## D-012 · 2026-08-11 · A Claude review gates `main`, and it blocks only on high-severity findings
- **Context:** `gate` and `audit` check that the suite is green and that every AC has a test. Neither
  reads the code. The list-command work-item made the gap concrete: the audit passed, and a review of
  the same branch then found seven defects, two of them severe — both in code no AC described, which a
  test-presence check cannot reach by construction.
- **Decision:** `.github/workflows/claude-review.yml` runs the **official `code-review` plugin** on
  every PR and fails the job **only** when the verdict is blocking: a finding that survived the
  plugin's own validation step and that would produce wrong behaviour, corrupt output, or a panic.
  Everything else is commented with `blocking: false` and does not fail the build. The threshold is the
  whole point — a gate that goes red on style is one that gets routinely overridden, and an overridden
  gate is not a gate. The prompt additionally points the review at this project's invariants and at
  this log, because a finding that contradicts a recorded decision is noise.
- **The plugin reviews; this workflow gates. Both halves were needed.** An earlier draft hand-wrote the
  review prompt; reading the plugin's own definition settled it — it fans out to four parallel
  reviewers (two on `CLAUDE.md` compliance, two hunting bugs) and then **independently re-validates
  every finding, dropping the ones that fail**. That validation pass is what makes its output worth
  gating on, and it is not something worth reimplementing by hand.
- **`/install-github-app`'s stock workflow is a no-op, and that is worth knowing before trusting it.**
  As generated it omits `--comment` — the plugin's step 7 then says "stop here, do not post any GitHub
  comments" — and sets `pull-requests: read`, so it could not post even if asked. Measured on PR #13:
  25 turns, 2m48s, ~$1 of subscription usage, **zero comments posted**, check green. Both halves are
  fixed here; the generated file was deleted rather than kept, since its job was also named
  `claude-review` and two jobs sharing a name make a required check ambiguous.
- **The plugin's "already commented, skip" rule had to be overridden, or the gate is bypassable.** Its
  step 1 stops when Claude has already commented on the PR — correct for a one-shot reviewer, wrong for
  a merge gate: the first push gets reviewed, and every later push sails through unread. That also
  rules out counting posted comments as the blocking signal, since comments from an earlier commit
  would either block forever or (scoped to the head SHA) clear on any empty commit. The verdict
  therefore comes from the run itself via `--json-schema`, and the prompt overrides the skip for
  everything except closed and draft PRs.
- **A missing verdict fails the check, and that guard earned its place three times over.**
  `claude-code-action` **exits successfully when it has done nothing at all**, which was observed in
  three distinct ways while building this: with no `CLAUDE_CODE_OAUTH_TOKEN` secret configured, and
  twice because the action validates that the workflow file matches the copy on the default branch and
  silently skips when it does not (so a review workflow can never be tested on the PR that changes it —
  it has to land on `main` first). Every one of those would have been a **green check with no review
  behind it**. Never treat this action's exit status as evidence a review happened; require both
  `conclusion == success` and a verdict.
- **Sequencing was the risk, not the workflow.** D-009 put `enforce_admins: true` on `main`, so a
  required check that cannot authenticate blocks every PR *including the one that fixes it*. The check
  is therefore added unrequired, run green on a real PR first, and promoted only after — the same
  ordering the `paths:` trap in D-009 taught, arrived at from a different direction.
- **Alternatives rejected:** Block on any finding at all — the strictest reading of the ask, and the
  fastest route to a check nobody trusts, with no owner bypass to clear a false positive. Report
  findings without ever failing — makes the check a comment bot, and "blocking" was the actual
  requirement. Keep both the generated workflow and a separate gate — two full multi-agent reviews per
  PR, roughly double the ~$1 subscription cost, for one merge decision. Hand-write the review prompt —
  loses the plugin's fan-out and validation pass, which is the part that makes its findings worth
  blocking on. Require it immediately on merge — the deadlock above. `paths:`-filter it to source
  files — the D-009 `Expected`-forever trap.
- **Known gap — closed 2026-08-11 on PR #15.** The `--json-schema` verdict and the skip-override could
  not be verified before merging, because of the workflow-validation rule above. Both hold: the run
  returned `{"reviewed":true,"blocking":false,...}`, so the schema does compose with a slash-command
  prompt, and the review ran rather than skipping a small docs change as trivial. It posted its comment
  (the `--comment` + write-permission fix) and cross-checked the claim under review against
  `crates/stidcart-cli/src/main.rs` instead of taking the diff at face value. `claude-review` was added
  to the required checks on `main` after that run, alongside `gate` and `audit`.
- **Cost, measured:** 12 turns / ~$0.48 for a one-line docs PR, 25 turns / ~$1.03 for the earlier code
  PR — it scales with the diff, and it draws on the Claude subscription through the GitHub App's OAuth
  token rather than API billing. That is the reason for one reviewing workflow rather than two.
- **Scope:** project-wide
- **Status:** active

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
- **Amended by D-020** (2026-08-14): one item left the rebuild-precondition list — a hidden entry,
  once `CartBuilder` could express the flag. The split itself is untouched, and so is the reason the
  entry gave for it: the line is describability, and hiding stopped being on the wrong side of it.
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
