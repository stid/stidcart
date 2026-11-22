# DECISIONS — stidcart

> **Append-only decision log.** Broader than an ADR: any decision worth not re-litigating —
> architectural, tooling, naming, process, content-organization — gets a block here. Newest at
> top. **Never rewrite history**: to change a past decision, add a new block and mark the old
> one `superseded by D-NNN`. Work-item-local decisions live in that item's `JOURNAL.md`; mirror
> here only the ones that outlive the work item. **Write the entry when the decision is made**
> (in-flight, context freshest), not retroactively at closeout — and keep the alternatives-rejected
> line honest: the entry is what a future session reasons from when it re-encounters the fork.

<!-- Next id: D-080 -->

---

## D-079 · 2026-11-22 · The app's filter is view state, and reordering is off while it is on

- **Context:** the listing had no way to find an entry on a full cart. The owner chose, through
  the question dialog on 2026-11-22: match the name and the title, turn reordering off during a
  filter, and offer an always-visible box plus Edit ▸ Find (⌘F).
- **Decision:**
  - The filter lives in `Listing.svelte` alone. The Rust side, the document and the undo history
    never hear of it, and a newly opened cart starts unfiltered.
  - It keeps the entries whose name or title contains the text, ignoring case. Folder names are
    not matched; a folder shows only while it holds a match, open even when collapsed.
  - While a filter is on, entries and folders do not drag and Move up/down are disabled. Rename,
    replace, hide, remove, Move to and the inspector still act on the entry chosen.
  - `filterTree` in `tree.ts` makes the tree that is drawn; every move still starts from the
    whole tree, so an edit made during a filter writes the same cart as without one.
- **Rejected:**
  - Matching every shown value (author, year, note, kind, folder): wider than asked, and a short
    text would match too much.
  - Keeping drags and steps on during a filter: "between these two rows" is ambiguous when hidden
    entries sit between them, and a move would land somewhere the user cannot see.
  - Read-only while filtering: finding an entry is usually the step before changing it.
- **Status:** active

---

## D-078 · 2026-11-17 · What the public repositories enforce, with one maintainer

- **Context:** the repository went public on 2026-11-17 and the owner asked that nobody else can
  approve a pull request or a release. The owner is the only collaborator on `stid/stidcart` and
  `stid/homebrew-tap`, so merging, tagging and publishing were already theirs alone, and `main`
  already required a pull request with `gate` and `gui` green, admins included (D-065).
- **Decision:**
  - A workflow run from any outside contributor's pull request waits for the owner's approval.
  - A `v*` tag cannot be moved or deleted (ruleset "release tags"), and a published release is
    immutable. The tap pins each file's checksum, so a file is never replaced.
  - CodeQL's default setup scans the code and the workflows.
  - Actions are pinned to commits, with Dependabot keeping the pins current.
  - The tap's `main` takes direct pushes from `tap.yml` and refuses force pushes and deletion.
- **Rejected:**
  - Requiring an approving review or a code owner's review on `main`: an author cannot approve
    their own pull request, so the one maintainer could merge nothing. Right once there is a
    second collaborator.
  - A protected environment that pauses each release for approval: only the owner can push a
    branch or a tag, and a fork's pull request receives no secrets, so it would add a click
    and stop dry runs from signing for no one kept out.
- **Status:** active

---

## D-077 · 2026-11-17 · Homebrew installs come from the project's own tap, as a formula and a cask

- **Context:** step 7 of the release roadmap (D-072), with the repository public and v0.31.1
  published. Recon against Homebrew 7.0 on 2026-11-17: a cask and a formula cannot share a name
  (the formula wins and the docs suffix the cask `-app`), a cask downloads one file, and since
  6.0 a third-party tap is trusted per item when installed by its full name. The app bundle
  does not hold the command, which ships as its own notarized tarball. A release is created as
  a draft, and a draft's files cannot be downloaded until the owner publishes it.
- **Decision:**
  - The tap is the public repository `stid/homebrew-tap`. Formula `stidcart` installs the
    command from the macOS tarball, and cask `stidcart-app` installs the app from the `.dmg`.
  - The README gives the full names, `stid/tap/stidcart` and `stid/tap/stidcart-app`.
  - A job of our own updates both recipes from `SHA256SUMS` when a release is published, with
    a token that can write to the tap and nothing else.
- **Rejected:**
  - One cask that links the command from inside the app: the app build would have to embed
    and sign the command, and the command could not be installed alone.
  - A cask for the app only: the command would stay a manual download.
  - The official `homebrew-cask` and `homebrew-core`: they turn away projects below their
    notability bar, and core builds from source.
  - A bump action (`mislav/bump-homebrew-formula-action`, `dawidd6/action-homebrew-bump-formula`,
    `macauley/action-homebrew-bump-cask`): each wants a classic token or a pull request, or
    supports casks poorly, for what is two lines rewritten in two files.
  - Updating the tap from the tag's run: the files are not public until the release is.
- **Status:** active

---

## D-076 · 2026-11-16 · The pane leaves out a stored name that its title repeats

- **Context:** under a title the pane shows the entry's stored name in grey (D-069). An entry
  titled with its own name showed the same word twice, and with lower case drawn as small
  capitals (D-070) "Alpha" over "alpha" reads as a repeat too. Found before the first public
  release.
- **Decision:** the menu skips the name row when the title equals the stored name with letters
  folded the way the search folds them. A title that only starts with the name keeps the row.
  The replaced build (66d71bd) joins the earlier builds, so carts carrying it still open.
- **Rejected:**
  - Comparing bytes exactly: a title typed with a capital would still repeat on screen.
  - Having `edit` refuse or drop such a title: the cart's data would then depend on what one
    menu draws, and carts written by earlier builds would still show the repeat.
- **Status:** active

---

## D-075 · 2026-11-16 · The release workflow signs and notarizes the macOS app, its disk image and the command

- **Context:** step 3 of the release roadmap (D-072). The owner's Apple Developer account became
  active on 2026-11-16. Tauri (CLI 2.12.2) signs the app and the `.dmg` and notarizes only the
  app. It logs a line and carries on when it has no key to notarize with, and reads an empty
  variable as a set one. The roadmap had left open whether the standalone command is signed.
- **Decision:**
  - Five repository secrets hold the Developer ID certificate as a `.p12`, its password, and an
    App Store Connect team API key. `release.yml` exports Tauri's variables only when they exist.
  - The job notarizes and staples the `.dmg` itself, then asks `stapler` and `spctl` whether the
    app and the `.dmg` are notarized, so a skipped notarization fails the job.
  - The macOS command is signed with the hardened runtime and notarized inside a `.zip`. A bare
    binary cannot carry its ticket, so Gatekeeper looks it up online on first run.
  - A run without the secrets builds ad hoc. A tag without them fails in the `version` job.
  - The key files and the command job's keychain are removed when the step that needs them ends,
    and `npm ci` runs in a step that holds no secret.
  - No entitlements file: the app passed notarization and ran without one.
- **Rejected:**
  - Leaving the command unsigned: a browser download unpacked in Finder is quarantined, and the
    certificate is already there.
  - Notarizing with an Apple ID and an app-specific password: the API key is scoped to a role
    and revoked on its own, and the notary accepted a team key from an individual account.
  - Trusting Tauri's exit status: it is 0 when notarization was skipped or stapling failed.
  - Passing the secrets straight into the build step's `APPLE_*` variables: a run without them
    would hand Tauri an empty certificate and fail the import.
- **Status:** active

---

## D-074 · 2026-11-15 · `STIDCART_X64SC` names the emulator the boot tests run

- **Context:** on a developer Mac the `x64sc` on PATH is Homebrew's GTK build, which opens a
  window for each of the ~90 boots and takes focus while the owner works on the same machine. CI
  already runs a headless build (D-038). A headless VICE 3.10 built on macOS with CI's configure
  line passed the whole suite on 2026-11-15 (366 tests, every boot, no window).
- **Decision:** `tests/support/vice.rs` runs the program named by `STIDCART_X64SC`, and the
  `x64sc` on PATH when it is unset. Set but empty panics, and a path that does not run fails the
  boot instead of skipping it. CI sets nothing and is unchanged.
- **Rejected:**
  - Putting the headless build first on PATH: `x64sc` typed in a terminal would then open no
    window either, and looking at a cart is what the GTK build is kept for.
  - Probing a fixed location such as `~/.local/vice-headless` from the test code: a path that is
    one machine's convention does not belong in the repo.
  - Passing `-minimized` to the GTK build: not tried. It would still start the GTK app for each
    boot, and local runs would keep differing from CI's build.
  - Running the boots in a Linux VM: a VM kept up all day for a loop that runs on the Mac.

---

## D-073 · 2026-11-14 · On Windows the app ships as an NSIS installer and the page forwards the menu shortcuts

- **Context:** step 5 of the release roadmap (D-072). The window was laid out for macOS: room for
  traffic lights, `⌘` on the buttons, a macOS app menu. On Windows the webview keeps the keyboard,
  so no menu accelerator fired. The owner ran the dry-run build on a real PC on 2026-11-14 and
  reported it working.
- **Decision:**
  - The app is an NSIS installer: per user, no administrator prompt. WebView2 stays on Tauri's
    default, a bootstrapper download when the runtime is missing.
  - The command is a `.zip` with the C runtime linked statically; the job fails if the `.exe`
    names `vcruntime`.
  - Platform differences live in `gui/src/lib/platform.ts` and behind the `WINDOWS` constant in
    `gui/src-tauri/src/menu.rs`. Only a Windows webview is treated as Windows.
  - On Windows the page forwards menu shortcuts to the `menu_key` command, which takes the same
    path as a click. On a Mac the page forwards nothing.
  - The installed program keeps the name `stidcart-gui.exe`, shown as "stidcart".
- **Rejected:**
  - MSI: refuses a pre-release version string and needs WiX, which depends on a Windows feature
    Microsoft is retiring.
  - Embedding WebView2: adds 127 MB for a runtime Windows 10 from 1803 and Windows 11 already have.
  - Renaming the program with `mainBinaryName`: it also renames the binary in the macOS bundle.
  - Giving Linux the same window treatment in this step: nobody has looked at the Linux window.
- **Status:** active

## D-072 · 2026-11-14 · Releases are built by a tag, signed on macOS, and reach Mac users through a Homebrew tap

- **Context:** the repository is about to go public and the only way to get stidcart is to build
  it. The owner wants stable builds for people who will not compile. Nothing is tagged or released
  yet, and only the macOS build has ever been run by a person.
- **Decision:**
  - A `v*` tag runs a hand-written `release.yml` that builds the command and the app and attaches
    them to a draft GitHub release. The owner publishes it by hand.
  - macOS builds are signed with a Developer ID and notarized. The owner pays for the Apple
    Developer Program and expects to reuse it for other projects.
  - macOS universal comes first, then Linux x86_64, then Windows x64 once the owner has run a
    build on a real PC or a VM. A platform nobody has run is held back or labelled untested.
  - Mac users install and update through the project's own Homebrew tap.
  - The work is split into steps in `docs/release-roadmap.md`, one work-item each. (All seven
    closed and the file was deleted on 2026-11-22; `RELEASING.md` says how a release is cut.)
- **Rejected:**
  - Unsigned macOS builds with a README workaround. Free, but every download meets a Gatekeeper
    block that current macOS only lifts from System Settings.
  - `cargo-dist`. It generates the workflow and installers for the command, but does not bundle
    the Tauri app, so the app would need a second pipeline next to a generated one.
  - Shipping every platform CI can build on day one. A green job says the build compiles, and the
    Windows window has never been drawn.
  - Publishing on every tag with no draft. A release is public the moment it exists, and a wrong
    asset cannot be recalled from whoever fetched it.

## D-071 · 2026-11-14 · The gate runs the test groups a change can affect

- **Context:** test-scopes. Every change, a README edit included, ran the whole suite: across five CI
  runs that day the 70 boots in `menu_boot` took 118s to 243s, against about 5s for every other root
  test together, and the `gui` job about 2m30s. The user asked for scopes: nothing for prose, VICE without the app for the menu, a
  subset of boots when the builder changes and the menu does not.
- **Decision:**
  - `docs/lcd/ci/scope.sh` maps changed paths to three values, `rust`, `vice` (none, cart, full)
    and `gui`. The rules follow what is compiled into what and are pinned by
    `crates/stidcart-cli/tests/scope.rs`. A path no rule names runs everything.
  - `menu/` runs the root tests with every boot and leaves the app out: the menu reaches the app
    only as embedded bytes. Root sources, `firmware/`, the manifests and the toolchain pin run the
    root tests with the cart set and the app, which is built on both crates. A root test or
    `fixtures/` runs the root tests alone, a boot test or its harness every boot. `gui/` runs the
    app alone. `ci.yml` and `docs/lcd/ci/` run everything.
  - `STIDCART_VICE` carries the boot scope into the tests. Unset is full, so a bare `cargo test`
    is unchanged, and any other value than the three names panics. The cart set is the 17 boots
    gated by `vice::require_cart_tool`: the EasyLoader boot (AC-4), the list and the long list
    (AC-87, AC-88), programs and cartridge kinds starting (AC-89, AC-90, the marker control), high
    memory (AC-160..AC-162), metadata and folders read off the cart (AC-166, AC-167, AC-169,
    AC-191) and the hidden metadata entry (AC-155). The user chose this set over five boots and
    over all of them.
  - Scoped everywhere: `docs/lcd/ci/gate.sh` locally, pull requests against their base, pushes to
    `main` against the commit before. The whole suite is `gate.sh --all` or "Run workflow". The
    user chose this over a full run on every push to `main` and over a weekly one.
  - `gate` and `gui` stay jobs that always start, with every step after Scope conditional. They are
    the required checks (D-065), so they must report on every PR, and a failing Scope step fails
    the job.
- **Alternatives rejected:** `paths:` filters on the workflow, which leave a required check pending
  on a PR that does not match. A separate scope job that the two `needs`: when it fails they are
  skipped, and a skipped required check passes. Naming the cart set in a script as test-name
  filters: a rename would drop a boot from it without a failure. Splitting the app's gate into
  frontend and Rust halves: its Rust tests need the built frontend anyway (D-030), and vitest is
  about 18s.
- **Scope:** `docs/lcd/ci/scope.sh`, `gate.sh`, `ci-scope.sh`, `.github/workflows/ci.yml`,
  `crates/stidcart-cli/tests/support/vice.rs`, the three boot targets, `CLAUDE.md`
- **Status:** active · narrows AC-94 (D-038): the boots still fail rather than skip for a missing
  tool, and now run only when the scope asks for them

---

## D-070 · 2026-11-14 · The menu's lower case is small capitals, and its text a step brighter

- **Context:** menu-legibility. With the label in (D-069) the user found EasyLoader's text easier
  to read and ours "not at its best", without being able to say why. Two candidate causes were put
  on switches over the current screen (artifact "Stidcart Menu Legibility"); the user chose both.
- **Decision:**
  - Screen codes 1-26, the lower-case letters, are redrawn as small capitals: five lines on lines
    2-6 of the cell, strokes two pixels wide, on the capitals' baseline, with their inverses at
    `$81-$9A`. Capitals, digits and punctuation stay the ROM's. The glyphs are ours, drawn for this.
  - Text moves one step brighter: what was light grey is white (list names, the pane's values,
    author and year), and what was mid grey is light grey (labels, the note, the stored name, the
    status, the find label, the help's descriptions). Mid grey is no longer used for text.
  - The label, the key bar, the rule, the scroll bar and the badges are unchanged.
- **Alternatives rejected:** EasyLoader's own font, which is zlib and stays behind the `firmware/`
  line. A whole custom character set, about 600 bytes, when the lower case was the part that hurt.
  Storing five bytes a glyph, 78 bytes saved for a second copy loop. Plainer badges, which the user
  chose over pictograms in D-041 and did not ask to change.
- **Scope:** `menu/src/menu.s`, `menu/menu.bin`, the menu's boot tests, `docs/guide/menu.md`
- **Status:** active · refines D-041 ("the text is still the ROM's") and D-069's pane colours

---

## D-069 · 2026-11-13 · The C64 menu's header is a cartridge label; the pane leads with what a person reads

- **Context:** menu-label. The user found the menu's design naive. Six directions were drawn pixel
  for pixel beside the current screen (artifact "Stidcart Menu Directions"); the user chose B, "Label".
- **Decision:**
  - Rows 0-3 are a label: reverse spaces in light grey, with the five bands as one reverse cell each
    in columns 1-5 down all four rows. A wordmark drawn at 12 pixels tall with 2-pixel strokes sits in
    columns 8-16 of rows 1-2, its 18 cells stored with the ink as background at `$EC-$FD`. The cart
    name and the count of visible entries end at column 38 of rows 1 and 2. All of it is black on
    light grey, the only ink a reverse cell has.
  - The block logo, the cartridge mark and the raster strip go, with the quarter-block glyphs and the
    multicolour cells.
  - The raster interrupt stays, drawing nothing: it is the menu's 50 Hz tick, which the stick's
    repeat is counted in, and AC-127 and AC-128 hold as written.
  - The pane leads with a chip for the kind, then the title or name, the stored name under a title,
    author and year on one line, and the note. Size, load address or bank, and start sit in rows
    18-20 with light grey values. The "type" row goes: the chip says it.
  - The key bar is light grey, as the label.
- **Alternatives rejected:** A "Index", C "Raster", D "Spine", E "Catalogue", F "Directory" (user's
  choice). Removing the interrupt for the CIA timer's: jiffies would run at 60 Hz and change the
  stick's repeat. Storing the wordmark in `$60-$7F` and reversing it: 17 codes are free there and it
  needs 18.
- **Scope:** `menu/src/menu.s`, `menu/menu.bin`, the menu's boot tests, `docs/guide/menu.md`
- **Status:** active · supersedes D-048's logo, mark and strip, D-041's block logo and pane order

---

## D-068 · 2026-11-13 · The hex viewer also shows one entry on its own

- **Context:** on first use the user expected to see an individual entry's bytes, not the whole
  image with a jump (D-067).
- **Decision:** (user's choices) keep the whole-image view and add a Selected entry view beside
  it, its rows addressed by offset from the entry's first byte. (Claude's choices) the window
  still opens on the whole cart; the entry is the one selected in the main window, so there is
  no second picker; a cartridge entry reads in bank order, ROML then ROMH of each of its banks,
  the order of the whole-image view; the entry view carries no owner tints, since every byte has
  the same owner.
- **Alternatives rejected:** an entry view only (offered): the directory, loader and free space
  would no longer be viewable. Bank + C64 addresses in the entry view, or both columns (offered,
  not chosen). All ROML banks then all ROMH for a cartridge: it differs from the whole-image
  view for no gain.
- **Scope:** `crates/stidcart-core/src/hexmap.rs`, `gui/src-tauri/src/hex.rs`,
  `gui/src/lib/HexView.svelte`

---

## D-067 · 2026-11-13 · The hex viewer is a second window over values the core computes

- **Context:** issue #66 asked for a read-only hex view of the cart and left open its scope, its
  addressing, where it sits and which extras to include.
- **Decision:** (user's choices, through the question dialog) the whole flash image with a jump
  from the selected entry; rows addressed by bank and C64 address; a separate window; the text
  column, entry ownership and the reserved bank-0 marking all in the first version. (Claude's
  choices) `stidcart_core::hexmap` computes the rows and the owned ranges, so the frontend holds
  no format knowledge, as with `listing` (D-032). Rows run bank by bank, ROML then ROMH, where an
  interleaved program is one contiguous run. ROMH always reads `$A000`, as the issue's example
  does. The window is the same page as the editor, told apart in `main.ts` by its label `hex`,
  with its own capability file. It shows the document's current step, so what saving would write
  (D-035), and the Rust side tells it when that changes or the selection moves. Each row request
  re-parses the container; nothing is cached.
- **Alternatives rejected:** a switch or a pane in the main window (offered; the user chose the
  window). The image's own order, all ROML halves then all ROMH: it splits every interleaved
  program into alternating far-apart pieces. `$E000` for ROMH on Ultimax entries and the boot
  code: a row's address would depend on its owner, and unowned rows have none. CRT file offsets:
  offered, not chosen. Decoding addresses or PETSCII in the frontend: a second copy of format
  facts. A URL query to pick the view: the window label needs no routing and cannot be mistyped
  into the wrong page.
- **Scope:** `crates/stidcart-core/src/hexmap.rs`, `gui/src-tauri/src/hex.rs`,
  `gui/src-tauri/src/lib.rs`, `gui/src/lib/HexView.svelte`, `gui/src-tauri/capabilities/hex.json`

---

## D-066 · 2026-11-08 · EasyLoader stays embedded, with its source and its real credits

- **Context:** preparing the repo to go public. The question was whether to keep embedding
  EasyLoader or make users import it. The audit also found `firmware/README.md` calling the blobs
  "copied unmodified" while crediting only ALeX Kazik and skoe: they are byte-identical to
  ndefpack 1.0.1's, but that EasyLoader is Hannu Nuotio's altered build (ACME port, `$FF` fill,
  fire-release delay), and zlib asks for altered versions to be marked. No source was shipped, and
  the original Bitbucket repo is gone.
- **Decision:** (user's choice, Claude's recommendation) keep embedding. `firmware/src/` holds the
  ndefpack 1.0.1 `src/fw/` source unchanged, taken from the `ndefpack-1.0.1` tag on SourceForge
  minus ndefpack's build glue. It is never built
  here; the `.prg` files stay the embedded data. `firmware/README.md` credits all three authors
  and marks the version as altered; `CREDITS.md` lists everyone; the README stops linking the
  private copy of ndefpack and states the derivative-work position once.
- **Alternatives rejected:** an external import. zlib already allows bundling, and every cart
  carries EAPI and the boot code whatever its menu, so importing would only move the 8 KiB menu
  slot while breaking the `ndeft` parity fixtures, `--menu easyloader`, Ocean carts and editing
  EasyLoader carts. Linking the source instead of including it: SourceForge is the only copy left.
- **Scope:** `firmware/`, `CREDITS.md`, `README.md`, `fixtures/MANIFEST.md`, `docs/lcd/MAP.md`

## D-065 · 2026-11-08 · The Claude workflows leave before the repo goes public

- **Context:** the repo is being prepared to go public. `claude-review` (D-012) and the `@claude`
  workflow ran on the maintainer's own Claude subscription token, and a hobby project can't pay for
  usage on a public repo. Fork PRs never get the secret either, so a required `claude-review` could
  never pass for an outside contributor.
- **Decision:** (user's choice) `.github/workflows/claude.yml` and `claude-review.yml` are deleted,
  `claude-review` leaves the required checks on `main`, and the `CLAUDE_CODE_OAUTH_TOKEN` secret
  goes. Required checks are `gate` and `gui`. `lcd-audit` stays: it is a shell script and calls no
  model. Development stays AI-assisted and says so openly; `CLAUDE.md`, `.claude/` and `docs/lcd/`
  stay public. A review can still be run locally before opening a PR.
- **Supersedes:** D-012 (a Claude review gates `main`). D-016 no longer applies.
- **Alternatives rejected:** keep the review for the maintainer's PRs but unrequired: it still
  spends the token on every PR. Restrict it to forks or label triggers: same cost question, more
  workflow to maintain.
- **Honest counterweight:** the review did find real defects early (PR #18's `audit-crosspath.sh`
  path bug; seven defects in the list command, D-013), so this trades a check that has paid for
  itself for a cost the project can't carry.
- **Scope:** `.github/workflows/`, `CLAUDE.md`, `.claude/rules/lcd-conventions.md`

## D-064 · 2026-11-08 · A new cart gets our menu unless told otherwise

- **Context:** default-stidcart-menu. With folders, titles and the inspector, our menu is the one
  the tool is built around, yet `build` and the window's New Cart gave EasyLoader. The user asked
  for stidcart as the default, in both (Claude's recommendation, 2026-11-08).
- **Decision:**
  - `build` naming no menu writes our menu; `--menu easyloader` gives EasyLoader, byte for byte what
    `ndeft` wrote. The window's New Cart starts with our menu. A cart that is opened keeps the menu
    it has.
  - The core's `Menu::default()` stays EasyLoader: the core's own parity proofs against `ndeft`
    build with it, and the library has no product default to state. The default lives at the
    product surfaces.
  - Content our menu can't run is refused, as before, and the refusal now ends "EasyLoader can
    start it". No silent fallback: a cart's menu never depends on what was added to it.
  - Tests that assert EasyLoader or `ndeft` behaviour name `--menu easyloader`; AC-82's "no request
    means EasyLoader" is superseded (its edit half stands).
- **Alternatives rejected:** changing only the window (two defaults to remember); falling back to
  EasyLoader when content needs it (a menu chosen by accident); changing the core default (rewrites
  the core's parity proofs for no product gain).
- **Scope:** `crates/stidcart-cli/src/lib.rs` (`build`), `gui/src-tauri/src/document.rs`
  (`Document::new`), the four menu refusals in `crates/stidcart-core/src/lib.rs`, tests
- **Status:** active · supersedes D-037's "EasyLoader by default" for new carts

---

## D-063 · 2026-11-08 · The cart owns an ordered folder list; folders may be empty

- **Context:** folder-browser. Trying the folder tree, the user found that folders vanished with
  their last entry and that making one meant picking an entry first. They asked for a stable
  structure with empty folders, a file-browser New Folder action, clearer menus and the details
  on the right, and said the project isn't public: no compatibility with earlier stidcart builds,
  only with carts from other tools (2026-11-08).
- **Decision:**
  - The cart keeps its folders as an ordered list, independent of its entries. Moving the last
    entry out or removing it leaves the folder. Delete Folder moves a folder's entries out.
  - The stored folder table is that list, in order, empty folders included. Group numbers are
    positions in it. The metadata entry is written when an entry has metadata or the cart has a
    folder. It stays layout version 1, with no compatibility code for earlier stidcart carts.
  - `edit` gains `--new-folder`, `--remove-folder` and `--move-folder`. `--folder` and the import
    still create a folder they name. `list` prints the folders.
  - The window has New Folder (button, File ▸ New Folder ⇧⌘N, right-click on empty space), empty
    folder cards that take drops, "Move to" with a check on the current folder, Delete Folder,
    and the details as an inspector on the right whose Folder field picks from the cart's folders.
  - Our C64 menu is unchanged: it already lists groups in number order and hides a group with no
    visible entry.
- **Alternatives rejected:** a new layout version (nothing to stay compatible with); a folder list
  only in the window (lost on save); deleting a folder with its entries (destructive); keeping
  "folders exist only while used" (D-060), which the user found confusing.
- **Scope:** `docs/format/`, `crates/stidcart-core` (`meta.rs`, `lib.rs`, `read.rs`,
  `listing.rs`), the CLI, `gui/src-tauri`, `gui/src/lib`; canvas
  https://claude.ai/artifact/21eX5WRUmSu77f8zxzMmjY
- **Status:** active · supersedes D-060's "a folder is created by moving an entry into it, never
  empty"; the rest of D-060, D-061 and D-062 stand

---

## D-062 · 2026-11-08 · The folder tree is drawn as grouped cards, with no Folder column

- **Context:** folder-editor, after the user tried the tree in the app: folder rows looked like
  entry rows, and an entry's number sat in the left column, outside its folder's indent, so the
  nesting didn't read. Three directions were drawn on a design canvas in the stid design system
  (directory listing, section bands, grouped cards), plus drag states; the user chose grouped
  cards, asked for the numbers to move inside the indent, and approved dropping the Folder column
  (2026-11-08).
- **Decision:**
  - With any folder, each folder is a stid card (hairline border, `--radius-lg`, the mantle
    surface) whose header holds the chevron, the name and a count pill; its entries sit inside,
    indented under the name. Entries in no folder follow under a `// not in a folder` label.
  - In the tree an entry's number moves into the name cell, inside the indent, and there is no
    `#` or Folder column. The entry panel still shows and edits the folder. A cart with no folder
    keeps the flat list as before.
  - Dragging: a lifted copy of the row or folder header follows the pointer, and the source fades.
    Dropping into a folder lights its whole card. A folder's insertion line runs above or below
    the whole card. The end strip for no folder is dashed. The drop rules stay as D-061 has them.
- **Alternatives rejected:** a directory listing (mono `name/` folder bands with `├─` guides) and
  section bands (mono caps headings): the user preferred the cards' grouping. Keeping the Folder
  column: in the tree it only repeats what the card shows.
- **Scope:** `gui/src/lib/Listing.svelte`, `gui/src/lib/rowDrag.ts`, their tests; spec AC-175,
  AC-178 amended
- **Status:** active · refines D-060, D-061; canvas https://claude.ai/artifact/21eX5WRUmSu77f8zxzMmjY

---

## D-061 · 2026-11-08 · The window sends a folder tree as one edit, dragged by pointer events

- **Context:** folder-editor plan. Every tree operation (D-060) must save the cart in tree order
  and equal `edit`'s `--folder`/`--move` result. The recon (2026-11-08, tauri-runtime-wry 2.12.1,
  wry 0.57.0, tauri#14373) found that with Tauri's native drag-drop handler on, which the Finder
  drop needs for file paths, WKWebView never delivers HTML5 `dragover`/`drop` to the page.
- **Decision:**
  - One window edit, `Arrange { folders: [{name, entries}], loose }`. The Document checks that it
    names every entry once, with non-empty, distinct folder names, then sets each entry's folder
    and moves the entries into flattened order with the one new core primitive,
    `CartBuilder::move_entry`, which `edit --move` also uses. Tree order holds by construction.
  - The frontend builds the tree from the listing rows, applies the gesture as a pure tree
    operation and sends `toArrange(tree)`. An unchanged tree sends nothing.
  - Dragging is hand-rolled on pointer events: pointer capture, a 4 px threshold,
    `elementFromPoint` with a pure two-band hit test. Tauri's `dragDropEnabled` stays on. Rows
    disable native element drags. The user chose hand-rolled over a library (2026-11-08).
  - The code renames `Field::Group` and the listing's `group` to folder. The format doc, the
    stored bytes and the menu source keep "group", the name of the byte layout.
- **Alternatives rejected:** a flat permutation plus a folder per entry (it can express a
  scattered order, so tree order would rest on frontend care); one Rust edit per gesture (puts the
  view's tree rules in Rust, four edits for one); HTML5 drag and drop or `dragDropEnabled: false`
  (no drop events, or no Finder paths); @dnd-kit/svelte (pre-1.0, a new dependency, and dropping
  onto a row unconfirmed).
- **Scope:** `crates/stidcart-core/src/lib.rs` (`move_entry`), `meta.rs`, `listing.rs`, the CLI's
  `edit`, `gui/src-tauri/src/document.rs`, `gui/src/lib/` (tree, drag, menus, listing)
- **Status:** active · refines D-060

---

## D-060 · 2026-11-08 · The group field is a folder, and the window edits folders as a tree

- **Context:** folder-editor. The metadata's "group" (D-054) is always shown as a folder by our
  menu (D-057), so filling it in as a label makes folders by accident, and the window shows it only
  as a column. The user asked whether to keep group as metadata and add folders, or make it a
  folder edited by drag and drop (2026-11-08).
- **Decision:**
  - The field is renamed Folder everywhere a user sees it: `edit --folder` (`--group` kept as an
    alias), the import key `folder` (`group` still accepted), `list`, the window. No stored byte
    changes.
  - With any folder, the window shows a tree in our menu's order. Drags and the entry menu's
    move commands are tree operations, and after one the directory is written in tree order
    (each folder's entries together, folders in tree order, then the rest). The panel's Folder
    field, `--folder` and the import stay metadata edits and re-order nothing.
  - A folder is created by moving an entry into it, never empty. The tree shows whatever menu
    the cart carries.
  - Re-order gets a CLI form, `edit --move FROM TO`, once per invocation and refused next to
    `--add`/`--remove`, so the window still edits only what `edit` does (D-035's oracle).
- **Alternatives rejected:** keeping group as a label beside folders, via a per-cart "show as
  folders" switch or a separate genre field (both change the metadata layout, and the user found
  it too heavy); moving only the dragged entry and leaving folders scattered in the directory
  (EasyLoader's order and the tree would disagree); empty folders in the window (they would vanish
  on save); a tree only under our menu (folders survive a switch to EasyLoader, so they'd be
  hidden while still stored); window-only re-order (breaks "the window edits what `edit` does").
- **Scope:** CLI `edit` and import, `list`, the window's listing and entry panel, the core's entry
  order
- **Status:** active · refines D-054 (the field's name) and D-057 (its meaning)

---

## D-059 · 2026-11-08 · The selection bar is cyan

- **Context:** The user tried the folders on their own cart in VICE and didn't like the orange
  selection bar (D-048). The bar is drawn in reverse, so its text is black and the bar has to be a
  light colour. Light green, yellow, light red, purple and light blue are badge colours, yellow is
  also the filter text, and grey is the help bar. Four previews were patched into copies of the
  user's cart (cyan, white, light grey, light blue) and compared side by side in VICE; the user
  chose cyan (2026-11-08).
- **Decision:** the selection bar is cyan (C64 colour 3), which the menu uses nowhere else. The raster
  strip, the mark and the logo keep the brand's colours, orange band included.
- **Alternatives rejected:** white (same as the logo), light grey (close to the cart name and the
  help bar), light blue (the multi-bank badge's colour and a band of the strip).
- **Scope:** `menu/src/menu.s` (one constant), AC-123's check and the shared bar check in the boot
  tests
- **Status:** active · supersedes D-048's orange bar (the rest of D-048 stands)

---

## D-058 · 2026-11-08 · Inside a folder the list starts with a ".." row that goes back up

- **Context:** menu-folder-up. Trying the folders in VICE, the user found no way back up with a
  joystick: going up was DEL with no filter text or RUN/STOP (D-057), both keyboard-only. They
  asked for an item at the top of a folder that goes one level up when selected, so the joystick
  works too (2026-11-08).
- **Decision:**
  - Inside a folder the first row is `..` with the folder badge. RETURN or fire on it goes back to
    the top level with that folder selected, as DEL and RUN/STOP still do.
  - It is always the first row and always listed: A-Z, the text filter and the kind filter leave it
    in place, so there is always a way out, even when nothing matches.
  - It is not an entry. The position and match counts on rows 22-23 leave it out, F5 never picks
    it, and the resume state never names it.
  - Opening a folder selects the first entry, not `..`, so the joystick lands on something to
    start; `..` is one row up. HOME selects `..` (the first row); CLR the last entry.
  - The details pane on `..` says it goes back to the top level.
- **Alternatives rejected:** a joystick gesture for going up, such as left at the first row or a
  held fire (invisible, and left is already page-up, D-052); a `..` row that only appears when the
  filter is empty (no way out with the stick while a filter is set); landing on `..` when a folder
  opens (one more move before every start).
- **Scope:** `menu/src/menu.s`, the boot tests
- **As built:** `..` is a folder row for group 0 at the head of the list, so the existing
  folder-row paths (start, saving and restoring the state) skip it with no extra code; a byte at
  `$C141` keeps it out of the counts and F5's range. Row 22 reads "0 of N" while `..` is
  selected, "no matches" sits on the row under it, SHIFT+RETURN on it does nothing, and F7's help
  gains "`..` return or fire: go up". In a folder with one entry, F5 on `..` selects that entry: F5
  moves to a listed entry other than the selected one, and `..` isn't one (AC-140 still holds for
  entries). Opening a folder, typing, F1 and F3 select the first entry.
- **Status:** active · refines D-057

---

## D-057 · 2026-11-08 · Our menu reads the metadata entry once at boot and shows titles, the pane fields and folders

- **Context:** menu-folders, the C64 half of the metadata the user asked for (D-054, D-055, spec
  `docs/lcd/work/menu-meta/spec.md`, approved 2026-11-08: M-8..M-11 → AC-166..AC-169). menu-meta
  stores it in a hidden `STIDCART.META` program entry (layout v1, load address `$4000`, at most
  16 KB), which our menu skips today.
- **Decision:**
  - At boot, before the raster interrupt is on (interrupts still masked from `entry`), the menu
    finds the last hidden program slot named `STIDCART.META` whose payload starts `STIDMETA` with
    version 1, and copies its payload to `$4000-$7FFF` with a RAM-resident loop, following the
    entry's bank layout (interleaved or confined to one half). Anything else (missing, another
    version, too big, a bad length) and the menu runs as a cart without metadata.
  - A record is used only when its 16 name bytes equal its slot's; records are stepped by their
    length byte.
  - The list row shows the title, cut to the row's width, when the entry has one; A-Z sorts by
    what the row shows. The details pane adds the title in full (wrapped), the year, the author and
    the note, under the existing lines.
  - Folders: with any grouped entry, the top level lists one folder row per group with a visible
    entry (a folder badge, the name, the count), then the ungrouped entries. RETURN or fire on a
    folder opens it; inside, DEL with no filter text or RUN/STOP goes back to the top level with
    that folder selected. RUN/STOP at the top level leaves to BASIC as today. The kind filter,
    A-Z, paging, F5 and HOME/CLR act on the folder's entries.
  - Filter text typed at the top level searches every visible entry in every folder and lists the
    matches flat. The text matches the name or the title, ignoring case.
  - Resume (D-053) restores an entry inside its folder: the menu opens that folder and selects it.
- **Alternatives rejected:** reading records from flash per row (bank switches with the raster
  interrupt live, against D-048); a group filter key instead of folders (the user chose folders);
  the `←` key for going back (it is printable and the filter takes it; CRSR ← is page-up, D-051).
- **Scope:** `menu/src/menu.s`, `menu/src/launch.s` if the copy loop lives there, the boot tests
- **As built:**
  - The copy runs through D-056's routine at `$0100` before `irq_on` and takes about 1.3 s of blank
    screen for a full 16 KB (PAL, VICE). Tables sit at `$C600-$C9FF`, variables at `$C130-$C140`,
    zero page `$26-$29`.
  - The menu is stricter than the core's decoder: it also wants load address `$4000` and a payload
    of at least 13 bytes. Anything else is a cart without metadata, which the tool never writes
    anyway.
  - An entry with any field set puts its facts on pane rows 6-10, a blank row 11, then the title,
    year, author and note from row 12, hard-wrapped at 16. Group-only entries keep AC-108's
    layout.
  - Folder rows count visible entries, ignore F3, keep group order under F1, cut long names to fit
    the count, and wear a white badge.
  - Inside a folder, row 23 counts matches out of the folder's entries. RUN/STOP inside a folder
    also clears the text. RUN/STOP at the top with a folder row selected spoils the resume state.
  - Resume opens the folder of any grouped entry, including one started from a flat search.
  - Tested: META confined to either half (types `0x02`/`0x03`) and the last of two candidates.
    About 1.7 KB of ROM is left.
- **Status:** active

---

## D-056 · 2026-11-08 · Our menu starts programs that reach into `$D000-$FFFF`, copying that part into the RAM under I/O and the KERNAL

- **Context:** menu-highmem, the fifth menu improvement the user asked for (D-054 and D-055 belong
  to menu-meta, on its branch). Since D-038 the tool refuses our menu on a
  cart holding a program whose bytes touch `$D000-$DFFF` (AC-95), because the copier runs from
  `$DF00` and writes with I/O mapped in. EasyLoader refuses these too
  (`ndefpack/src/fw/easyloader.asm:957-985`: an end at or past `$D000`, or one that wraps past
  `$FFFF`), so this goes past it. Recon found the check also lets through a program loading at
  `$E000` or above, which our menu would start by `SYS` into the KERNAL ROM (`touches_io_area`
  tests only `start <= $DFFF`), and that no check limits a program's end to `$FFFF`.
- **Decision:**
  - The copy splits at `$D000`. The part below is copied by today's trap with I/O in. The rest is
    copied by a small routine put at `$0100` (bottom of the stack page) once the first part is done.
    With interrupts masked and the NMI pointed at an `RTI`, it reads each byte with `$01=$37` (so
    the bank register works) and writes it with `$01=$33`, so every byte lands in RAM, under I/O at
    `$D000-$DFFF` and under the KERNAL at `$E000-$FFFF`. It puts `$01`, the NMI vector and the bank
    back before returning to today's path (BASIC's pointers, then `RUN`, `SYS` or `READY.`).
  - The bytes for `$D000-$DFFF` go to RAM, not to the VIC, SID, CIA or colour RAM as a KERNAL
    `LOAD` would put them. One-file programs that reach that high expect them in RAM, and writing
    them to I/O would reprogram the chips under the menu.
  - With our menu the tool still refuses, naming the entry: a program that loads at `$D000` or
    above (the menu would `SYS` into I/O or ROM), a program that ends past `$FFFF`, and a program
    reaching `$D000` that loads below `$0100` plus the routine's length (the routine would overwrite
    it). EasyLoader carts are not checked differently than today.
- **Alternatives rejected:** a second copier staged in a RAM buffer (banking I/O out hides the bank
  register, so it would copy in chunks for no gain over reading through `$01=$37` per byte); writing
  `$D000-$DFFF` to I/O as a KERNAL `LOAD` does (reprograms the VIC and CIAs mid-copy); keeping the
  refusal (EasyLoader parity is no reason to stay as limited as it).
- **Scope:** `menu/src/menu.s`, `menu/src/launch.s`, the refusal in `crates/stidcart-core/src/lib.rs`,
  the CLI and GUI refusal tests, the boot tests
- **As built:** the routine is 80 bytes (`$0100-$014F`), so the low floor is `$0150`
  (`menu::HIGH_COPY_END`, held equal to the label by a test). It brackets the copy with
  `php`/`sei` … `plp` and saves and restores `$0318/9` on the stack. The core's one refusal became
  three: `ProgramLoadsHigh`, `ProgramPastTop`, `ProgramOverHighCopy`. Known limits, not refused:
  a program that loads below about `$0334` has always overwritten the stack and the KERNAL vectors
  the trap is using, and still does; after SHIFT+RETURN on a program ending exactly at `$10000`,
  BASIC's end pointer is `$0000`, so `SYS` works but `LIST`/`RUN` don't; the NMI vector is written
  in two stores, so a RESTORE press between them would jump through half a vector. The `$01=$33`
  writes are proven in VICE only.
- **Status:** active · refines D-038 (AC-95's refusal)

---

## D-055 · 2026-11-08 · The metadata entry's layout v1: a 16 KB cap, records by index and stored name, kept-version edits refused by name prefix

- **Context:** menu-meta, planning D-054's hidden `STIDCART.META` entry. The C64 side
  (menu-folders) will copy the whole entry into `$4000-$7FFF` once at boot, so it can never be
  more than 16 KB. A fully filled record is 147 bytes, so 254 of them need about 37 KB. The spec
  lists no refusal for that, and D-054 left open what "an edit that would have to rewrite" a kept
  unknown-version entry is.
- **Decision:**
  - The layout is version 1, a stidcart extension in `docs/format/`. It is a PRG loading at
    `$4000`, holding the magic `STIDMETA`, the version, the group and record counts and the
    record-table offset. Then come fixed 16-byte NUL-padded group names (group n at
    `table + (n-1)*16`, in order of first use), then one length-prefixed record per entry that
    has metadata, in index order. Each record holds its index, the 16 name bytes its slot
    stores, a group number and four length-prefixed PETSCII fields.
  - A set whose payload would pass 16384 bytes is refused, naming both sizes
    (`BuildError::MetadataTooLarge`). This adds a refusal; it weakens nothing.
  - On read, META is the _last_ hidden program slot whose name is `STIDCART.META` and whose
    payload starts with the magic. A record attaches when its 16 bytes equal its slot's raw name
    bytes. META is always written last, so a record's index is the directory slot.
  - A kept (unknown or malformed) META is written back byte for byte. A build is refused when
    the entry names as loaded are no longer a prefix of the current names (a remove, rename or
    re-order) or when any entry has metadata set. Hide, align, replace, add, names and a menu
    switch still save.
  - Text maps to PETSCII when `petscii_to_ascii(ascii_to_petscii(v)) == v`, that is, printable
    ASCII. Entry names stay unvalidated as before.
- **Alternatives rejected:** fixed-offset records (37 KB worst case, mostly padding); records
  without the name copy (D-054 detects a re-order by name); refusing every save of a cart with a
  kept META (blocks harmless edits); a hash of the name instead of the bytes (the menu would need
  to compute it).
- **Scope:** `docs/format/`, `crates/stidcart-core/src/meta.rs`, `read.rs`, `lib.rs`
- **Status:** active · refines D-054

---

## D-054 · 2026-11-08 · Entry metadata lives in one hidden directory entry, carried per entry by the model

- **Context:** menu-meta. The user asked for a full title, year, author, note and group per entry,
  shown by our menu with groups as folders, and edited three ways: `edit` flags, an import file
  and the window (2026-11-08, spec approved the same day). EasyFS2 has no field for any of it. A
  200-entry cart needs about 8-12 KB, and the loader slot has about 3 KB to spare once the
  planned menu keys are in.
- **Decision:**
  - The metadata is one hidden program entry, `STIDCART.META`, placed by the normal placement
    path, written only when at least one entry carries metadata. A cart without metadata is
    byte-identical to today's. EasyLoader skips hidden entries (`easyloader.asm:900`, `:935-941`),
    so the cart works under either menu and keeps its metadata through a menu switch.
  - `load` lifts that entry out of the entry list and attaches each record to its entry, as the
    hidden flag is carried (AC-29), so remove, re-order and rename carry it and a save writes it
    again. Each record holds its entry's index and stored name; a record whose name no longer
    matches is dropped and counted, which catches another tool's re-order.
  - The layout carries a version. One the tool doesn't know is kept verbatim and described, and an
    edit that would have to rewrite it is refused.
  - The format doc gains its section first (D-018), marked as a stidcart extension, not an
    ndefpack fact. Its evidence is round-trip tests and VICE boots, not reference citations.
  - Its load address avoids `$D000-$DFFF`, so the D-038 rule never refuses it.
  - The work splits in two: `menu-meta` (store, round-trip, CLI, import, listing, window) and
    `menu-folders` (our menu reads it at boot and shows titles, the pane fields and folders), so
    the first can run while other menu work holds the menu source.
- **Alternatives rejected:** spare space in the loader slot (too small, changes the menu
  fingerprint, lost on a switch to EasyLoader); free regions of bank 0 (the boot region is checked
  byte-for-byte, so `load` would refuse the cart); a fixed high bank (nothing in the directory
  points at it, so the placer and other tools treat it as free space); a sidecar file beside the
  `.crt` (the C64 can't read it).
- **Scope:** `docs/format/`, the core's entry model and read/write paths, `edit`, `list`, the
  window, and later `menu/`
- **Status:** active

---

## D-053 · 2026-11-08 · After a reset the menu comes back where it was left, from ten bytes of cart RAM

- **Context:** menu-resume, the third menu improvement the user asked for. Today every boot starts
  at the first entry, so after a game and a reset the user scrolls back to where they were.
  EasyFlash's 256 bytes of RAM at `$DF00` are always visible (EasyFlash ProgRef §2.3-2.5). Recon in
  VICE 3.10 found the page kept across a soft reset and replaced by a power-on pattern on a hard
  one, and found that anything running with the cart off can write it. The launch routines fill
  `$DF02-$DFF5`, which leaves `$DFF6-$DFFF`.
- **Decision:**
  - The menu keeps its state in `$DFF6-$DFFF`: a magic word, the selected entry's directory slot,
    the order, the kind filter, the number of directory entries, an 8-bit sum of the selected
    entry's name, and a checksum over all of them. The filter text doesn't fit and isn't kept.
  - It writes the state when it starts an entry (RETURN, SHIFT+RETURN, fire) and when it leaves on
    RUN/STOP.
  - At boot it restores order, kind filter and selection only when the magic and checksum hold,
    the entry count and name sum match this cart, and the slot is a visible entry. If the kind
    filter would hide the entry, the filter goes back to all kinds. Otherwise it starts as today.
  - The launch routines' assembly cap drops so they can never grow into the state. They already
    reached `$DFF6` (`p_only`, D-051), so `leave` now shares the tail of `cart_go` to free it: the
    routines end at `$DFF2`, `$DFF3-$DFF5` are free.
  - As built: `$DFF6-$DFF9` the magic `STID` (four bytes, so all ten are checked), `$DFFA` slot,
    `$DFFB` order, `$DFFC` kind filter, `$DFFD` the end marker's slot (so hidden entries count),
    `$DFFE` the name sum, `$DFFF` a checksum (`$A5` plus the other nine). RUN/STOP with nothing
    listed spoils the magic. A restored entry below the first page is shown on the last row.
    A rename that keeps both the slot count and the 8-bit name sum would still restore that slot.
  - The state is a convenience, not a promise: a game that uses EAPI (`$DF80-$DFFF`) or an xbank
    game that uses the page overwrites it, and the next boot starts at the top.
- **Alternatives rejected:** keeping the filter text (16 bytes; it doesn't fit); saving to flash
  through EAPI (wears the chip, slow, and a crash mid-write could damage the cart); storing the
  position in the list rather than the directory slot (a different order or filter would point at
  a different entry).
- **Scope:** `menu/src/menu.s`, `menu/src/launch.s`, the boot tests. The F7 help also gains a
  line for the joystick, which menu-joystick left out
- **Status:** active

---

## D-052 · 2026-11-08 · The menu takes a joystick in port 2, as presses of the keys it already handles

- **Context:** menu-joystick, the second menu improvement the user asked for. Ours reads input only
  through GETIN (D-038), because that was the only input VICE could inject for the boot tests.
  Recon found that VICE 3.10's "Joyport I/O simulation" device (`-controlport2device 37`) and the
  monitor's `jpdb` command drive CIA1 port A through the same path a real stick does, so a
  joystick can be proven in the boot tests after all.
- **Decision:**
  - Port 2 only. Port 1 shares CIA1 lines with the keyboard matrix and would read key presses as
    stick moves.
  - The stick does what keys already do: up and down move one entry, left and right a page (as
    CRSR ← / →), fire starts the selection (as RETURN). On the help screen any move or fire
    returns, as any key does.
  - A move acts on the press, then repeats while held, after a first delay of about 0.4 s and then
    about ten a second, timed by the jiffy clock. Fire acts once per press and never repeats.
  - Fire or a direction held when the menu starts is ignored until released, so a stick resting
    on fire doesn't start the first entry. A direction that closed the help screen doesn't repeat
    until released.
  - The stick is read once a jiffy, which also absorbs contact bounce, with `$FF` in `$DC00` under
    `sei` and the KERNAL's `$7F` put back after, so two keys held in one row can't pull a stick
    line low on real hardware.
  - The stick is read in the menu's own loop, not in the raster interrupt, masked to its five
    bits (the emulated device reads bits 5-7 as 0).
- **Alternatives rejected:** port 1 as well (keyboard crosstalk); reading the stick inside the
  raster interrupt (more code running where D-048 keeps banks still); proving the stick by writing
  from the monitor the value a read would see (weaker evidence than the device, which goes through
  CIA1).
- **Scope:** `menu/src/menu.s`, the boot tests and `support/vice.rs` (extra emulator arguments)
- **Status:** active

---

## D-051 · 2026-11-08 · The menu's new keys: load without running, paging, ends, kind filter, random pick, help

- **Context:** menu-keys, the first of the menu improvements the user asked to have built end to
  end (2026-11-08), with the spec decided here by delegation. The menu reads keys only from GETIN
  (D-038), and the C64 has no page or end keys, so each new action needs a key the KERNAL buffer
  can carry and that the filter text doesn't already take.
- **Decision:**
  - SHIFT+RETURN on a program loads it as RETURN does and stops at `READY.` without typing `RUN`
    or `SYS`, BASIC's pointers set, so `LIST`, `RUN` or a `SYS` work. On a cartridge it does
    nothing: a cartridge has no "loaded but not started" state.
  - CRSR → and CRSR ← move a page (16 rows) down and up, stopping at the last and first entry.
  - HOME and CLR still clear the filter text when there is some. With no text, HOME goes to the
    first entry and CLR to the last, so both keep their meaning and gain one.
  - F3 cycles a kind filter: all, programs, cartridges. It combines with the text filter, row 22
    names it beside the order, and changing it lists again from the first match, as the text does.
  - F5 moves the selection to a random entry among those listed, never the one already selected,
    so every press visibly moves. It never starts anything.
  - A move that leaves the view scrolls only as far as it takes to show the selection. The help
    bar reads `return go  f1 sort  f7 help  run/stop` to fit F7 in its 38 cells.
  - F7 shows a full-screen help naming every key; any key returns to the menu as it was.
- **Alternatives rejected:** F-keys for first/last (F2/F4, F6/F8): shifted F-keys are hard to
  remember, and HOME/CLR already mean "home". Making F5 start the random pick: one key press
  shouldn't start a game the user hasn't seen. Starting a cartridge on SHIFT+RETURN: it would do
  the same as RETURN, so the key would mean nothing on half the list.
- **Scope:** `menu/src/menu.s`, `menu/src/launch.s`, the boot tests
- **Status:** active · refines D-039 (HOME/CLR)

---

## D-050 · 2026-11-08 · An explicit menu switch replaces a loader slot that holds no known menu

- **Context:** replace-loader, the second half of the user's choice in older-menus. A cart whose
  loader slot holds neither embedded menu nor an earlier build of ours (D-049), for example a
  loader another tool wrote, is described but refused by `load`, so it can only be viewed. The user
  asked that choosing a menu be able to replace it.
- **Decision:**
  - The core gains `read::load_with_menu(bytes, menu)`. It holds EAPI, boot and the EF-Name magic
    to exact bytes as `load` does, skips only the loader-slot check, and gives a builder carrying
    the chosen menu. The slot is overwritten on build, so what was in it doesn't matter.
  - `edit --menu easyloader|stidcart` takes that path, so it works on such a cart, and the other
    operations in the same call apply too. Without `--menu`, `edit` refuses as before.
  - The window opens such a cart view-only with `edit`'s reason and a menu picker showing
    "unrecognised". Choosing a menu is one undoable edit, after which the cart is editable.
    Undoing it returns to view-only, where nothing else can be edited or saved. A cart refused for
    any other reason shows no picker.
  - So read-only belongs to each step of the window's history, not to the document, and the
    document says whether its menu can be replaced (`menuReplaceable` in the wire contract).
- **Alternatives rejected:** replacing an unknown loader whenever a cart is loaded, which would
  overwrite something the tool knows nothing about without anyone asking. Keeping the unknown slot's
  bytes through an edit, which can't be vouched for: the tool can't know that loader works with a
  re-placed directory. Leaving such carts view-only, the status quo.
- **Scope:** `crates/stidcart-core/src/read.rs`, `crates/stidcart-core/src/reserved.rs`, `edit
--menu`, the window's document and its menu picker
- **Status:** active · refines D-038 and D-042 (the picker on a view-only cart)

---

## D-049 · 2026-11-08 · A cart carrying an earlier build of our menu loads as stidcart, and saving writes the current build

- **Context:** older-menus. After 0.16.0 changed the menu, the user's `stidcollection.crt`, built
  with the 0.15 menu (commit `5b54e81`), opened view-only as "menu unrecognised": `load` accepts a
  loader slot only when it is byte-for-byte one of the two menus this build embeds (D-038). Every
  change to `menu/menu.bin` would strand every cart built before it the same way.
- **Decision:**
  - Every committed build of `menu/menu.bin` is recognised by an FNV-1a 64 fingerprint of the full
    8 KiB slot, normal mode only (our menu has no Ocean build). The list lives in
    `crates/stidcart-core/src/menu.rs`. A test pins the current build's fingerprint, so changing
    the menu fails it until the replaced fingerprint joins the list of earlier builds.
  - Such a cart describes as `stidcart (older build)` and loads as `Menu::Stidcart`, so any edit, and
    choosing stidcart in the window's picker or `edit --menu stidcart`, writes the current build.
    The rebuild differs from the input in the loader slot only.
  - This is a scoped exception to D-011's "rebuild unchanged": the loader slot of a known earlier
    build is the one region a rebuild may change on its own. EAPI, boot and the EF-Name magic are
    held to exact bytes as before, and a slot matching no known build is still refused.
  - One earlier build is committed at `menu/past/5b54e81.bin`, the one the user's cart carries, so a
    test can produce a cart that carries it. Fingerprints can't be forged, so the tests need real bytes.
- **Alternatives rejected:** a signature string in the menu, which can't tell our build from a
  modified or foreign one with the same text. Embedding every past build and keeping it on save:
  56 KB for eight builds, and an edit would keep the old menu's bugs. Leaving such carts view-only:
  the status quo, with no way back to an editable cart.
- **Scope:** `crates/stidcart-core` (menu, reserved, read, listing), the window's menu picker
- **Status:** active · supersedes D-038 in part (the loader slot may also hold an earlier build of our menu)

---

## D-048 · 2026-11-08 · The C64 menu wears the raster brand: a white logo beside the cartridge mark, a raster strip, an orange bar

- **Context:** menu-raster. After D-046 the user asked for the same brand on our C64 menu, within
  what the machine allows. Three levels were prototyped on throwaway copies of `menu/src` and
  captured in VICE (the "c64 menu · vice captures" board on the brand canvas): palette only, plus a
  raster strip, plus the cartridge mark. The user picked the third after running it live in VICE.
- **Decision:**
  - The five bands map to the VIC-II's light red, orange, yellow, light green and light blue. Peach
    maps to orange, so the selection bar turns orange. The logo turns white, the neutral wordmark
    beside a colourful mark, as in the app.
  - The rule under the cart name (row 4) gives way to a raster strip: a raster interrupt sets the
    background colour on scanlines 84-88, one band a line, then back to black. Row 4 starts on the
    badline at 83, and one line a band is the most a character row holds without a badline inside it.
  - The mark is the app's cartridge in 3x3 multicolour cells at columns 2-4 of rows 0-2, its glyphs
    in codes `$EA-$F2` (names never reach the reversed range of the menu's own codes). The logo
    moves to column 7. The interrupt turns multicolour on for rows 0-2 only and rewrites `$D022`
    every three lines (54-66, no badline at a boundary), so only the label shows the bands. The
    shell is dark grey (`$D023`), the pins yellow (the cells' own colour).
  - The raster interrupt replaces the CIA timer's: it runs the KERNAL's `$EA31` once a frame, so
    the keyboard is scanned 50 times a second and nothing can delay the bars. An entry later than
    its first line skips the bands for that frame and forces multicolour off.
  - The handler runs from the cart's ROM, so interrupts are masked whenever the menu maps another
    bank in, and every exit (program, cartridge, RUN/STOP) puts back the stock state first: raster
    interrupt off, `$0314` at `$EA31`, the CIA timer interrupt on.
  - Its evidence is VICE screenshots as well as memory dumps: screen RAM can't show raster colours.
- **Alternatives rejected:** palette only (no strip, no mark), cheapest but the menu wouldn't read as
  the same brand. Bars in the top border, two lines a band: robust, but not under the title as in
  the app. Five colours drawn in the label as character cells, impossible: a hires cell has one
  colour and a multicolour cell's own colour is limited to 0-7. The handler in RAM instead of
  masking: possible later, but nothing in RAM is the menu's to keep while it starts a program.
- **Scope:** `menu/src/menu.s`, `menu/menu.bin`, the menu's boot tests
- **Status:** active · supersedes D-040's cyan bar and D-041's logo colours and row-4 rule

---

## D-047 · 2026-11-08 · The window's fonts ship inside the app, copied from Fontsource

- **Context:** raster-brand. D-046 sets text in Geist and Geist Mono, and the app must work
  offline, so the faces can't come from Google Fonts at runtime.
- **Decision:** the Latin and Latin Extended variable woff2 files of `@fontsource-variable/geist`
  and `@fontsource-variable/geist-mono` 5.3.0 are copied into `gui/src/fonts/` with their OFL-1.1
  licence. A local `fonts.css` declares them under the family names Fontsource uses. Vite bundles
  them into `gui/dist`, which Tauri embeds. The folder is OFL-1.1, a third licence next to `firmware/`
  (zlib) and the GPL code, and MAP's licence invariant names it.
- **Alternatives rejected:** the two npm packages as dependencies. They'd be a new dependency
  for four static files and would pull in Cyrillic and Vietnamese subsets the app never draws.
  Keeping system fonts, which the user declined.
- **Scope:** `gui/src/fonts/`, `gui/src/styles/`
- **Status:** active · serves D-046

---

## D-046 · 2026-11-08 · The window wears the stid design system, with a raster band as its mark

- **Context:** raster-brand. The app had no identity: system fonts, a generic blue accent, a
  stock icon. The user asked to align it with their stid design system and give it a logo in
  1980s colour bands on a dark ground. Four directions were drawn on a canvas, and the user picked
  "raster".
- **Decision:** the theme tokens take the stid system's Catppuccin values: Mocha in dark, Latte in
  light, the peach accent, and stid's radii (3/5/8 px). Light mode stays, still following the OS.
  The mark is a cartridge seen head-on whose label is five horizontal bands: Catppuccin red, peach,
  yellow, green and blue. The same five bands run as a 10 px strip under the title bar in every
  state. The mark is also the app icon, in the title bar and on the empty state. The type badges
  keep their VIC-II colours (D-045) because they describe data, not the brand. Text on the accent is
  dark in both themes, because Latte's peach with white text fails 4.5:1.
- **Alternatives rejected:** "loading border" (VIC-II raster bars round the stid prompt). It would
  share the badge palette, but the colours are muted and the icon has no cartridge. "Sunset stripes"
  (four saturated diagonals) would bring hexes from outside the stid system. "Prompt cursor" (the
  stid mark with a striped cursor) says nothing about cartridges. Keeping the blue accent, which the
  user declined.
- **Scope:** `gui/src/styles/tokens.css`, the title bar, the empty state, `gui/src-tauri/icons/`
- **Status:** active · keeps D-045

---

## D-045 · 2026-11-08 · The window colours each type badge like our menu's badge for that kind

- **Context:** readable-sizes. With sizes readable, the user asked for the type column to carry
  the colours our C64 menu gives each kind (D-041), so the app and the cart read the same way.
- **Decision:** the core's listing gives each row a family next to its label: `prg` (PRG, PRG-L,
  PRG-H), `8k`, `16k`, `ult` (ULT8, ULT16), `xb` (XB8, XB16, XBULT) and `other` (Ocean, which our
  menu refuses). It's mapped from the entry type, never parsed from the label. The window tints the
  badge in that family's colour: light green, yellow, light red, purple and light blue as on the C64,
  and neutral for `other`. Each colour has a light-mode and a dark-mode pair, picked for contrast
  rather than copied, since raw C64 yellow on white can't be read. 8K uses a warmer amber-yellow
  than the C64's own greenish yellow: in the dark theme the first one read too close to PRG's green
  (user's live check).
- **Alternatives rejected:** the raw C64 palette, unreadable in light mode. Deriving the family
  from the label in the window, which the types file rules out for decisions about a row. Colour in
  `list`, where a terminal's palette is the user's own.
- **Scope:** `stidcart-core` listing, the GUI listing and theme tokens
- **Status:** active · refines D-044, follows D-041's colours

---

## D-044 · 2026-11-08 · Sizes read as KB in the listing; exact bytes stay one hover or flag away

- **Context:** the user uses the app day to day and found the hex sizes (`$0027d3`) hard to read.
  Bank and layout are addresses, where hex is the convention, and stay as they are.
- **Decision:** (user's choices) the core's listing gives each size, and the free figure, as text:
  under 1024 bytes in bytes ("652 bytes", "1 byte"), otherwise in 1024-based KB with one decimal
  rounded half up ("1.1 KB", "10.0 KB"), whole when the size is an exact multiple ("8 KB"), and in
  MB the same way from 1024 KB. `list` and the window both show it, so they still agree value for
  value (D-032). Each row also carries the exact byte count: the window shows it on hover, and
  `list --bytes` prints exact decimal counts for every size and the free figure.
- **Alternatives rejected:** the window only, which would break D-032's one description. Exact
  grouped bytes, or KB with bytes beside it, which the user didn't pick. Decimal kB (1000), which
  doesn't match how C64 sizes are counted (a 16K cart is 16384 bytes). Dropping the exact figure,
  which would lose precision the hex had.
- **Scope:** `stidcart-core` listing, `list`, the GUI listing
- **Status:** active · refines D-032

---

## D-043 · 2026-11-08 · The Claude review skips a pull request whose whole diff is Markdown

- **Context:** the user asked where CI spends budget. About a third of merged PRs (12 of the last 38) changed only Markdown, and the review of a one-line docs PR measured about $0.48 of
  subscription usage (D-012), plus a billed Actions minute.
- **Decision:** (user's choices) `claude-review` diffs the PR head against its base. When every
  changed path ends in `.md`, it skips the review step and the gate passes with a notice. Anything
  else, including scripts under `docs/` and an empty diff, is reviewed as before. The test is the
  whole PR, never the latest push. Markdown that steers agents or this review is always reviewed:
  `CLAUDE.md`, `.claude/`, `docs/lcd/DECISIONS.md` and `MAP.md` (the review prompt treats them as
  settled), `docs/format/`, and `fixtures/`. Otherwise an unreviewed PR could rewrite what a later
  review takes as authority (raised by the automated security review on the PR).
- **Alternatives rejected:** skipping per push, which would let a docs-only push after a blocking
  finding turn the gate green without fixing the code. A `paths:` filter in `on:`, which D-009
  rules out for a required check. Dropping the review from required checks for docs PRs by hand.
- **Landing:** the action refuses to run while the workflow differs from `main`'s, so this change
  fails its own required check. It lands by unrequiring `claude-review` for that one merge, with
  the user's OK, and requiring it again straight after. Any later change to this workflow needs the
  same.
- **Scope:** `.github/workflows/claude-review.yml`
- **Status:** active · refines D-012 (the review gate)

---

## D-042 · 2026-11-07 · The window switches the menu as an edit, through the same refusals as `edit --menu`

- **Context:** menu-picker. D-037 had the window recognise, show and keep the menu but not choose
  it; switching stayed `edit --menu`. With our menu usable on the user's real collection, they
  asked to choose it in the window, where the menu box already looked like a control.
- **Decision:**
  - On an editable cart the menu box is a picker with EasyLoader and stidcart, the cart's own
    selected. Choosing the other is one edit (`setMenu`): undoable, marked unsaved, and saved as
    the bytes `edit --menu` writes.
  - The refusals are the core's, unchanged: a cart with an Ocean entry, or a program loading into
    `$D000-$DFFF`, refuses stidcart naming the entry, and the cart stays as it was.
  - A view-only cart, or one whose slot holds neither menu, shows the menu as text with no picker.
- **Alternatives rejected:** a menu-bar item, which hides a per-cart setting away from where the
  menu is shown. A two-button toggle, which reads as an on/off switch for two names. Greying out
  stidcart for carts it would refuse, which needs the refusal checked ahead of time and hides the
  reason the user would want to read.
- **Scope:** `gui/src-tauri/src/document.rs`, `gui/src/lib/Listing.svelte`, `gui/src/lib/types.ts`, GUI tests
- **Status:** active · refines D-037 (the window now chooses the menu)

---

## D-041 · 2026-11-07 · The menu is layout C: block logo, type badges, a details pane and a scroll bar, from a RAM character set

- **Context:** menu-details. After D-040 the user asked for type icons drawn as pixel graphics, a
  scroll bar for long lists, and an end to the flicker. Three layouts were mocked up pixel for
  pixel (artifact "Stidcart Menu Layouts"). The user chose C, the logo-and-details layout, with
  the scroll bar it gained in review. The first build used the page's default pictogram icons; the
  user then asked for the badges, which replaced them before merge.
- **Decision:**
  - The ROM's mixed-case set is copied to RAM at `$3800` (`$D018=$1E`) at boot. The menu's own
    glyphs (a two-character badge per kind, "PRG", "8K", "16K", "ULT" and "XB", then the rule,
    track, thumb and 16 quarter-block cells for the logo) replace screen codes `$60-$7F`, each badge
    half with its inverse at code+`$80`. The bitmaps are the mockup's,
    byte for byte. This supersedes D-040's "ROM font only": the text is still the ROM's.
  - PETSCII `$A0-$BF` in a name now shows as `?`, since its screen codes are the menu's glyphs.
    stidcart never writes those bytes into a name.
  - The screen: logo on rows 0-2, cart name on row 3, rules on rows 4 and 21, and 16 list rows
    (5-20, columns 1-20), each with its badge in columns 1-2. The scroll track runs down column 21, and the
    details pane for the selection (type, size, load address or bank, how it starts) sits in
    columns 23-38. Row 22 holds the position and order, row 23 the filter text with its match
    count, and row 24 the help bar.
  - The status counts the position ("17 of 40") instead of matches. The match count moves next to
    the filter text so it isn't lost (D-039 had both on one line).
  - Flicker: every cell is written once, never cleared first. A cursor move that doesn't scroll
    redraws only the two rows it changes, the pane, the status and the scroll bar. Drawing starts
    below the visible screen.
- **Alternatives rejected:** layouts A and B, and the pictograms (user's choice). Sprites for
  icons, which can't cover 16 rows without multiplexing. The ROM's own PETSCII graphics for icons,
  which can't draw a disk or a cartridge. Dropping the match count, which nobody asked for.
- **Scope:** `menu/src/menu.s`, `menu/README.md`, boot tests
- **Status:** superseded in part by D-048 (logo colours, row-4 rule) · supersedes D-040 in part (font, layout) · refines D-039 (status line)

---

## D-040 · 2026-11-07 · The menu's look is colour and two reverse bars, drawn straight into screen RAM

- **Context:** menu-look, the third custom-menu slice (D-037). The menu printed everything through
  CHROUT in the C64's default light blue on blue. It uses about 1.6 KiB of its 8 KiB slot.
- **Decision:** (user's choices, following the recommendation)
  - Colour and layout only, with the ROM font.
  - Dark scheme: black border and background, light grey entries, yellow filter text, medium grey
    status. Row 0 is a full-width reverse cyan title bar with "stidcart" and the cart name, and the
    selection is a reverse cyan bar too. Both are cyan because the title colour and the selection
    colour the user picked are the same, and a reverse bar can only show black text.
  - The rows stay where AC-87..AC-101 pin them: filter on row 1, list on rows 2-21, count and order
    on row 23. Row 24 is a new full-width reverse grey bar with the key help. The preview's
    single bottom line ("2/57 A-Z RET run…") doesn't fit in 40 columns beside the existing status
    wording.
  - `draw` writes screen and colour RAM directly instead of using CHROUT. A 40-character row through
    CHROUT links into the next line, and reaching column 39 of row 24 scrolls the screen.
  - The menu shows the ROM's mixed-case character set (`$D018=$17`, SHIFT+C= locked). In the
    upper-case set, the `$61-$7A` an upper-case file stem becomes showed as graphics, under CHROUT
    too. Found on the first screenshot and added with the user told.
  - Before a program starts, the menu restores the C64's own colours (border and text light blue,
    background blue) and the upper-case set. `basic_go` enters the reset after CINT, so nothing
    else would undo them. Cartridges and RUN/STOP go through a full reset, which does, except
    the SHIFT+C= lock: a cart's reset runs before CINT, so the menu clears it before a cart starts.
- **Alternatives rejected:** a custom font, and sprites or raster effects, which need RAM copies,
  an IRQ beside the KERNAL's, and harder tests, for a look the user didn't ask for. A full PETSCII
  frame, which costs two rows of the list. Colour alone on today's layout. The classic blue and
  amber palettes. Rewriting the status line to match the preview, which moves four tests and SPEC
  for no gain.
- **Scope:** `menu/src/menu.s`, `menu/README.md`, boot tests
- **Status:** superseded in part by D-041 (font and layout) and D-048 (cyan bar) · refines D-037

---

## D-039 · 2026-11-07 · Menu search is a substring filter as you type; sort is a view toggle that starts in directory order

- **Context:** menu-search-sort, the second custom-menu slice (D-037). Letters are free in the menu,
  which reads only CRSR, RETURN and RUN/STOP. The directory order is fixed by the format's placement
  rules, so a sort can only change what the menu shows. It can't reorder the cart.
- **Decision:** (user's choices, following the recommendation)
  - Typed characters build a query of at most 16 characters, shown on the blank row above the list.
    The list narrows to visible entries whose name contains the query, ignoring case. All three
    letter ranges are folded to one (`$41-$5A` and `$C1-$DA` from GETIN, plus `$61-$7A`, which an
    uppercase file stem becomes). DEL removes a character, and CLR/HOME clears the query.
  - F1 toggles between directory order and A-Z, with the same folding. Ties keep directory order.
    Every boot starts in directory order, so the AC-87 screen is unchanged. The sorted order is
    computed on the first F1 and kept.
  - RUN/STOP leaves to BASIC whatever the filter state, as before.
- **Alternatives rejected:** jump-to-letter, which never narrows a long list. A prefix-only
  filter, which finds fewer entries for a small saving. Always A-Z, which hides the build order on
  the C64. A three-way cycle that adds by-kind order, which nobody asked for. A build-time sort
  that reorders the directory, which placement rules forbid.
- **Scope:** `menu/src/menu.s`, `menu/README.md`, boot tests
- **Status:** active · refines D-037

---

## D-038 · 2026-11-07 · The menu is a builder property detected from the loader slot; boots are proven in VICE built in CI

- **Context:** custom-menu plan. Recon found three things: VICE can inject keys only into the KERNAL
  buffer, apt's VICE and 64tass versions follow the runner image, and `ubuntu-latest` moves to 26.04
  this month. EasyLoader's PRG start copies only after a BASIC cold start, through a CHRIN trap
  (`easyloader.asm:2091-2260`).
- **Decision:**
  - `CartBuilder` carries a `Menu` (EasyLoader default, or Stidcart). `verify_regions` reports which
    menu the loader slot holds, keeping EAPI/boot/magic strict. `inspect` reports it as an
    `Option` and never refuses on it (D-011). `load` sets it, so edits preserve it.
  - Our menu refuses two things at build time: an Ocean entry, and a program whose loaded extent
    touches `$D000-$DFFF` (the copier lives at `$DF00`). Spec AC-95 was added for the second.
  - The menu reads keys only through GETIN and starts programs in EasyLoader's order (partial reset
    with the cart off, BASIC cold start, CHRIN trap, copy). The code is ours, and the technique is
    cited.
  - CI's `gate` job pins `ubuntu-24.04` and builds VICE 3.10 (`--enable-headlessui`) and 64tass
    1.60.3243 from source, cached by tool, version and image. `STIDCART_REQUIRE_TOOLS=1` turns the
    harness's skip into a failure. The first task proves the existing AC-4 boot on Linux before any
    menu code (user chose the source build over apt or a macOS job).
- **Alternatives rejected:** a CIA-matrix key scan (no test could drive it). Copying before BASIC
  init (RAMTAS clears pages 0/2/3). A menu-side workaround for `$D000` programs (it needs a second
  copier in low RAM). apt's vice, which drifts and lacks 3.10's breakpoint fix. A macOS CI job
  (10x minutes, no version pin). A separate CI job, which would need a branch-protection change.
- **Scope:** `menu/`, `crates/stidcart-core` (`menu.rs`, `reserved.rs`, `read.rs`, `listing.rs`), CLI `--menu`, GUI DTO, `ci.yml`
- **Status:** active · superseded in part by D-049 (earlier builds of our menu load too)

---

## D-037 · 2026-11-07 · Our own menu is opt-in, GPL in `menu/`, assembled with 64tass, and lands in slices

- **Context:** D-005 deferred a custom C64-side menu. The user wants all of it: a new look, search
  and sort, and subdirectory navigation. The launcher only does `jmp ($8000)` (or `($a000)` in
  Ocean mode), so a replacement loader needs to fit the 8 KiB slot and keep that entry vector. The
  launcher and EAPI can stay as they are. A VICE boot harness already exists (`stidcart-cli` `boot.rs`).
- **Decision:**
  - The menu is opt-in. EasyLoader stays the default, so a build without the option is
    byte-identical to today and the ndeft fixtures keep their meaning. Carts carrying our menu
    have no ndeft oracle, so their evidence is a VICE boot (the decaying-oracle invariant).
  - The source is GPL-2.0-or-later in a new top-level `menu/`. `firmware/` stays third-party
    zlib blobs and the licence boundary does not move.
  - It is assembled with 64tass. The assembled `.prg` is committed, and a check reassembles it
    byte-identically, so `cargo` never needs the assembler.
  - It lands in work-items. The first is the minimal slice: list the entries and boot them, a build
    option that selects the menu, and `load`/`edit`/GUI recognising the second firmware set. Look
    and feel, search/sort and subdirectory navigation follow, one item each.
  - First-slice scope (user's choices): asking for our menu on a cart with an Ocean entry is a typed
    refusal. The GUI recognises, shows and keeps the menu but doesn't choose it. VICE is installed
    in CI so the boot checks run there instead of skipping. A VICE boot is sufficient evidence
    without a hardware check.
- **Alternatives rejected:** Replacing the default would invalidate every parity fixture and leave
  the write path with no oracle at all. zlib inside `firmware/` would break that directory's "nothing
  here is compiled" rule and blur the boundary. KickAssembler (an earlier pick in the private notes) needs a
  pinned Java jar, and 64tass is a native binary that is already installed. acme is not installed.
  Building every feature in one item would spread it across too many sessions to resume cheaply.
- **Scope:** `menu/` (new), `stidcart-core` firmware embedding and verification, `build`/`edit`, GUI
- **Status:** active · refines D-005 · its EasyLoader default for new carts superseded by D-064

---

## D-036 · 2026-10-12 · ⌘Q is the app's own Quit item; macOS ending the app does not ask yet

- **Context:** the live check of gui-edit found that ⌘Q quit without asking about unsaved changes
  and lost an edit. The predefined Quit menu item (muda) sends `terminate:`, and tao 0.37
  implements no `applicationShouldTerminate`, so the process ends with no `RunEvent::ExitRequested`
  and the guard never runs. Dock Quit, logout, restart and shutdown take the same path.
- **Decision:** Quit is a custom ⌘Q menu item that sends `quit` to the window, which runs the
  unsaved-changes guard and then calls `exit_app`. The paths macOS starts itself stay unguarded for
  now. gui-edit AC-78 is narrowed to say so, and they are a follow-up (user's choice).
- **Alternatives rejected:** a native `applicationShouldTerminate` hook in this slice. Its cost is
  unknown (tao owns the app delegate), so it gets its own spike, not a delay to this PR. Keeping the
  predefined item and relying on `ExitRequested` would leave ⌘Q silently losing work.
- **Scope:** `gui/src-tauri/src/menu.rs`, `gui/src/lib/viewer.svelte.ts`
- **Status:** active

---

## D-035 · 2026-10-11 · The GUI document is a stack of saved images in Rust; parity is proven against the CLI as a library

- **Context:** the GUI editor (D-033) needs undo, an unsaved-changes state and a listing after each
  edit, and AC-80 requires its saves to be byte-identical to `edit`/`build`.
- **Decision:**
  - The GUI crate holds the document. Each undo step is the image saving would write (bytes), plus
    its `describe` listing. An edit loads the current image, applies the change, builds and
    describes. A refusal changes nothing.
  - Snapshot 0 of an opened cart is the file's own bytes, so it shows positions as stored, and a
    Save As without edits is a verbatim copy.
  - Soundness rests on `load(build(b)).build() == build(b)`, which a property test checks.
  - `stidcart-cli` becomes a library plus a thin binary. The GUI tests dev-depend on it and check
    each GUI step against one chained `edit` call (New with adds against `build`).
- **Alternatives rejected:**
  - Snapshots of `CartBuilder`: the same memory, but the listing would stop describing what a save
    writes.
  - Frontend-held state: the frontend would need the image bytes and format calls.
  - Committed goldens written by a root CLI test: scenarios written twice, and goldens to regenerate
    on each version bump.
  - Comparing against direct core calls: tests core against core and misses front-end drift.
  - `tempfile` for the atomic save: mode 0600 on every saved cart. The std temp-and-rename is
    mirrored from the CLI instead.
- **Scope:** `gui/src-tauri` (`Document`), `crates/stidcart-cli` (lib + bin)
- **Status:** active

---

## D-034 · 2026-10-11 · Classifying an input file as a program or a cartridge entry is core logic

- **Context:** `edit --add`/`--replace` and `build` decide what a file is in a CLI-private
  `input_entry`: the CRT signature, `Cartridge::from_crt`, and the alignment checks. The GUI must
  decide identically (gui-edit AC-61), and the MAP keeps format knowledge out of both front ends.
- **Decision:** `Entry::from_input(name, data, aligned) -> Result<Entry, InputError>` in core, with
  no I/O and no path. `InputError` reproduces the CLI's alignment messages verbatim, and front ends
  prefix a cartridge refusal with `'<path>': ` as the CLI does. Reading the file and deriving the
  name from the file stem stay in each front end.
- **Alternatives rejected:**
  - Duplicating the check in the GUI: two copies of format knowledge.
  - The GUI depending on the CLI crate at runtime: the CLI is the test harness, not a library for
    products.
- **Scope:** `stidcart-core` (`lib.rs`), `stidcart-cli` (`input_entry`), `stidcart-gui`
- **Status:** active

---

## D-033 · 2026-10-11 · The GUI edits in memory with an explicit save; drops on an open cart add entries

- **Context:** the second GUI slice brings the full `edit` set into the window. Changes could
  either be written immediately, as `edit` without `-o` does, or held until the user saves. A
  dropped file could also mean either "open this" or "add this".
- **Decision:** (user's choice, following the recommendation)
  - Edits are held in memory, with undo and redo and an unsaved-changes indicator. ⌘S replaces the
    file and is unavailable when there are no changes, so a cart from another tool is never
    silently re-placed.
  - ⌘⇧S saves as. Closing, quitting, opening another cart or New asks first when there are
    unsaved changes.
  - With an editable cart open, every drop adds, `.crt` included. A multi-file drop adds all of
    them or none, as one undo step. With nothing editable open, one dropped file opens, as before.
  - New (⌘N) is in this slice.
  - Each edit is applied whole or refused with the reason `edit` gives, and a saved cart is
    byte-identical to `edit`/`build` for the same request.
- **Alternatives rejected:**
  - Write on every change: no undo, and it rewrites the original at the first click.
  - Save As only: safe, but clumsy for the common case.
  - A dropped `.crt` opens, or asks each time: adding cart images is the main reason to drop one on
    an open cart, and opening is one ⌘O away.
- **Scope:** `gui/` (gui-edit, AC-59..AC-80)
- **Status:** active

---

## D-032 · 2026-10-11 · The listing is one structured description in the core, rendered by both `list` and the GUI

- **Context:** the GUI viewer must show exactly what `list` prints (gui-shell AC-51). The values
  are derived by private helpers in the CLI binary: kind labels, flags, bank-run layout, the
  header-name control-character filter, and the free figure that tries `load` and falls back to
  `inspect` (D-027).
- **Decision:** a core `listing` module turns image bytes into a `Listing`: cart name, filtered
  producing-tool name, one row per entry with every value as `list` displays it, and the free
  figure. `list` keeps only column layout and pluralisation. The GUI maps the same struct to a
  serde DTO inside its own crate. Core gains display labels, but no serde and no I/O.
- **Alternatives rejected:** a separate `stidcart-view` crate, since one more workspace member buys
  nothing a module doesn't. Letting the GUI re-derive the values, which would mean two
  implementations kept equal only by tests.
- **Scope:** `stidcart-core` new `listing` module; `stidcart-cli` `list`/`render`
- **Status:** active

---

## D-031 · 2026-10-11 · The GUI frontend is Svelte 5

- **Context:** D-030 puts the GUI in a Tauri web frontend. The framework inside it was still open.
  The candidates, as of npm on 2026-10-11, all build with Vite 8, test with Vitest 5 and have
  TanStack Table 9: Svelte 5.57, React 19.3, Solid 1.9, and plain TypeScript.
- **Decision:** Svelte 5, with bits-ui 2.x as the headless component layer and
  `@testing-library/svelte` for the frontend tests (user's choice, following the recommendation).
- **Alternatives rejected:**
  - React: the deepest component ecosystem (Radix + shadcn/ui), but more boilerplate and runtime
    for a Rust-first maintainer, and the polish bar doesn't need it.
  - Solid: its main headless component library, Kobalte, is still pre-1.0 (0.13), against the
    stable-release limit.
  - Plain TypeScript: every component, state update and accessibility detail would be
    hand-written, which gets expensive once editing arrives.
    GitButler, the polished Tauri app D-030 cites, ships on Svelte.
- **Scope:** `gui/` frontend
- **Status:** active

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
  word and size are derived from those at encode time. A cartridge is only ever _derived_ from an
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

## D-018 · 2026-08-14 · The format spec lives in the tree at `docs/format/`, not in the private notes

- **Context:** The format spec in the author's private notes was the source for every
  offset in `stidcart-core` — an invariant in `MAP.md` and an instruction in `CLAUDE.md`. The private notes are
  not in a git repo. PR #20 turned that from a tidiness question into a concrete failure: D-017
  rests on a correction to that spec, the correction landed as a plain file edit, and the reviewer
  saw a citation to a change absent from the diff. `gate`, `audit` and `claude-review` all ran
  against a tree that did not contain the document their subject depends on, and the wrong claim had
  already been copied into `directory.rs:12,67` with a test pinning it — drift a reviewer could not
  see, because the copy and the original were not diffable.
- **Decision:** the spec moves to `docs/format/easyflash-cart-format.md` and the private notes keep a stub
  pointing at it. A format fact is now corrected there first, in the same PR as the code it
  justifies. **Supersedes the private-notes clause of the format-facts invariant** (`MAP.md`); the
  invariant itself — facts come from the spec, never re-derived from ndefpack's C — is unchanged and
  is the whole reason the document has to be reviewable. The project note stays in the private notes:
  it is the predecessor's decision log, not a spec this codebase owns.
- **`docs/format/` over the alternatives.** `spec/` at the root would leave D-014's docs zone
  untouched but adds a top-level directory for one file. Dropping the file into `docs/` directly
  would mean deleting the zone's anti-sprawl clause outright. Splitting the zone in two keeps the
  guardrail and moves it down a level: nothing non-LCD directly in `docs/lcd/`.
- **The licence question is settled, not assumed.** The spec was written by reading ndefpack's GPL
  C, so it is a derivative work by exactly D-004's argument and GPL-2.0-or-later is where it already
  belonged. Publishing the repo changes nothing — the obligation travels with the licence. The
  moved file says so in its own header.
- **The move is verbatim.** Only the YAML frontmatter, the two `[[wikilinks]]` and the closing
  "Related" line changed, plus an added header and licence note; `diff` against the private-notes original
  shows nothing else. Correcting what the spec _says_ is separate work needing its own `file:line`
  evidence — the point of this change is that such a correction is now reviewable, not that any
  were made.
- **Scope:** project-wide (supersedes the private-notes clause of the format-facts invariant in `MAP.md`;
  D-014's `docs/lcd/` artifact root is unchanged)
- **Status:** active

## D-017 · 2026-08-12 · The reserved flag bits are the EasyFS version discriminator, not a quirk to tolerate

- **Context:** D-007 decided an EasyFS1 directory returns a typed `Unsupported` error, and the code
  never implemented it — `SPEC.md:58` records that every directory is walked as EasyFS2 and misread.
  It looked blocked: the private-notes format spec documents EasyFS2 only, and the format-facts invariant
  forbids deriving the missing fact straight out of ndefpack's C into code. The research was cheap
  and the blocker dissolved.
- **The discriminator is bits 5–6 (`0x60`) of the flags byte at offset 16.** Not both set means old
  EasyFS (`efs_check_entry`, `easyfs.c:371-389`). The first non-end slot decides for the whole
  directory; a later slot disagreeing is a hard error, not a reclassification. The end marker is
  `0xFF`-filled, so its bits are set and it never classifies.
- **The private-notes spec was wrong about these bits, and the error had been copied into this repo.** It
  read "A past ndefpack bug wrote them as 0 (`NEWS`); anything reading these carts should tolerate
  both" — wrong tool, wrong citation, and a prescription that cannot be satisfied. `README:82` is a
  2012 comparison against **Draco Cart Maker**, a different tool; `NEWS` says nothing about reserved
  bits; and ndefpack's writer ORs `EFS_RESERVED_MASK` unconditionally (`easyfs.c:581`), so clear bits
  are unambiguously not its output. A reader cannot both ignore these bits and detect the version —
  tolerating a clear pair as EasyFS2 gives up the only signal the format has. `directory.rs:12,67`
  repeated the claim and `decode_tolerates_the_reserved_bits_being_clear` pinned it as a test.
- **Decision:** the private-notes spec is corrected first (the invariant's point), then the read path checks
  the bits. An old-format directory is refused with a typed `Unsupported`, and slots that disagree
  are refused as malformed rather than as old. The **writer guidance is unchanged and still
  load-bearing**: `encode` sets the bits, and correcting the reader must not take that with it.
- **Refusing to _describe_ an EasyFS1 cart is consistent with D-011, not an exception.** That
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
  in `.github/workflows/claude-review.yml`. The action then refused to run: _"the workflow file
  must exist and have identical content to the version on the repository's default branch."_ That
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
  `${specs_dir%/*}` — the _parent_ of `<root>/work`. At `docs/work` that resolves to `docs/`, which
  silently excludes every match anywhere under `docs/` rather than just the work-item tree; at
  `docs/lcd/work` it resolves to `docs/lcd`, which is what the script means. `claude-review` caught
  it on PR #18, before any work-item existed to be mis-audited. Corrected on the same branch, before
  merge — nothing was rewritten that had ever reached `main`.
- **The lesson generalizes past that one script.** A non-default `artifact-root` is not a free
  parameter: the suite carries assumptions about the _shape_ of the tree that the machine block does
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
  documents for each of three finished items, whose purpose was surviving a context reset _during_
  work that is now done.
- **Decision:** Delete `docs/lcd/work/` and the triage/lane machinery with it. Keep `DECISIONS.md`
  and `MAP.md`, promoted to `docs/`. The gate command and branch discipline move into `CLAUDE.md`,
  which is where a session already looks. New work starts by writing a failing test.
- **`audit` is dropped as a required check** and `lcd-audit.yml` deleted: it verified that every
  acceptance criterion had a test, and acceptance criteria only existed because the pipeline
  produced them. Without that, it is a no-op check that still has to be maintained — and D-009
  records what a required check that stops reporting does to a PR. `gate` and `claude-review` stay.
  _(D-014 reinstates it, scoped to PRs that touch `docs/lcd/work/**`.)_
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

_Superseded by D-065: the Claude workflows were removed before the repo went public._

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
  required check that cannot authenticate blocks every PR _including the one that fixes it_. The check
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
  drop the flag" — a statement about _rebuilding_ — and `inspect` ran `verify_regions`, whose entire
  purpose is to stop a rebuild silently swapping firmware. Both fired on a command that rebuilds
  nothing, and the first discarded entries already collected, so one hidden entry in slot 0 emptied
  the listing. Refusing to describe a cart because it could not be reproduced is the opposite of what
  an inspection command is for.
- **Decision:** Keep one walk; split the strictness. The walk refuses only what it cannot **describe**
  — malformed slots, entries reaching outside the image, and the well-formed shapes with no
  representation here. `load` layers the rebuild preconditions on top: the reserved regions must hold
  the embedded firmware, and no entry may carry a property `CartBuilder` cannot express. What an image
  _contains_ is still answered in exactly one place, which was the property worth having; what may be
  _done_ with it is answered by the caller that is going to do it.
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
  block every PR _including the one that fixes it_.
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
  could not be required _as written_. That is a property of the filter, not of the workflow: its script
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
  identifying the producing tool _and version_, and the read path preserves what it finds (D-007)
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
  `-e` does not change it. `-e` sets the _EF-Name_, a different 16-byte PETSCII string living inside
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
- **Status:** active · the custom menu now exists, opt-in, beside the blobs (D-037)

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
