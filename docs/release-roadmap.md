# Release roadmap

How stidcart gets from "build it yourself" to downloadable, signed builds. Decided on 2026-11-14
(D-072). This file is the index for the work: each step below is its own LCD work-item, started in
its own session with `lcd:triage`, and ticked off here when its PR merges. Delete the file when the
last step closes; by then `SPEC.md` and the README say what exists.

## Starting a session on this

1. Read this file, then D-072 in `docs/lcd/DECISIONS.md`.
2. Pick the first unticked step whose "needs" are met.
3. Run `lcd:triage` on it with the slug given here. The lane noted is an expectation, not a verdict.
4. Run `lcd:recon` before choosing an action version, a Tauri bundle option or a Homebrew layout.
   Everything marked *(recon)* below is from memory and has to be checked against current docs.
5. When the PR merges, tick the step here in the same PR or the closeout one.

## What is decided

- **A `v*` tag builds a release.** A hand-written `release.yml` builds everything and attaches it
  to a draft GitHub release, which the owner publishes by hand.
- **macOS builds are signed with a Developer ID and notarized.** The owner pays for the Apple
  Developer Program.
- **Platforms, in order of confidence:** macOS universal (app and command), Linux x86_64 (command
  first, app after someone has looked at it), Windows x64 (after the owner has run it on a real PC
  or a VM). Windows and Linux installers ship unsigned.
- **Homebrew is the main install path on a Mac**, through the project's own tap.
- **No release goes out for a platform nobody has run.** A build that only CI has seen is either
  held back or labelled untested in the release notes.

## What is not decided yet

Each belongs to the step named, to be settled there and logged as a decision if it is a real fork.

- The version of the first public release: carry on from 0.30.x or something else (step 6).
- How the tap is laid out: a cask that installs the app and links the command from inside the
  bundle, or a cask for the app plus a separate formula for the command (step 7).
- Whether the command's standalone macOS archive is signed and notarized too. A browser download
  is quarantined, a Homebrew or `cargo install` one is not (step 3).
- Whether the Linux app ships as "untested" or waits for a look in a VM (step 4).

Left out on purpose until someone asks: Linux arm64, the Tauri auto-updater, Windows code signing,
the official `homebrew-cask` repository (it has notability rules *(recon)*), crates.io.

## Steps

Waves are ordered; steps inside a wave are independent of each other.

### Wave A: needs nothing

- [x] **1. `release-workflow`** (#84) (expect Standard). `.github/workflows/release.yml`, on a `v*` tag
  and on `workflow_dispatch` as a dry run that uploads workflow artifacts and creates no release.
  - Fails unless the tag equals the workspace version and `gui/src-tauri/Cargo.toml`'s. The version
    reaches the bytes of every cart built (MAP invariant), so a mislabelled binary is a wrong one.
    Put the check in a script with a test next to `scope.rs`, written first.
  - Command archives for macOS universal and Linux x86_64, each carrying `COPYING` and
    `firmware/COPYING`. Build Linux on the oldest runner image still offered, for glibc reach.
  - The macOS app as a `.dmg`, with bundling turned on by a `--config` override the way
    `scripts/install.sh` does. `tauri.conf.json` keeps `bundle.active: false`.
  - A `SHA256SUMS` file, and a draft release holding all of it.
  - Signing steps are skipped when the secrets are absent, so this lands before step 3.
  - Give `release.yml` a rule in `docs/lcd/ci/scope.sh` and a case in `scope.rs`.
  - Uses the pinned toolchain: `rustup toolchain install` with no argument, as `ci.yml` does.
- [x] **2. `windows-ci`** (#85) (expect Standard, may shrink to Quick). A `windows-latest` job in
  `ci.yml` running the root tests, not a required check. The emulator boots skip there. Fix what
  breaks: path separators, line endings on fixtures, anything else. This says whether a Windows
  command can ship at all, and it does not touch `release.yml`.

### Wave B: needs step 1

All three add a leg to `release.yml`, so run them in separate worktrees and merge one at a time.

- [ ] **3. `macos-signing`** (expect Quick or Standard). Needs the Apple Developer account.
  - The owner creates a Developer ID Application certificate and an App Store Connect API key, and
    sets the repository secrets with `gh secret set`. Secrets never go through a Claude session.
  - Tauri reads them from environment variables *(recon: the exact names)* and signs, notarizes
    and staples the `.dmg`.
  - Done when a dry-run `.dmg`, downloaded through a browser onto a Mac, opens with no Gatekeeper
    prompt, and `spctl -a -vv` reports a notarized Developer ID.
- [x] **4. `linux-app`** (#86) (expect Quick). AppImage and `.deb` from the Linux leg, with the system
  packages the `gui` job already installs. Open it once in a Linux VM, or ship it marked untested.
  - Nobody has yet opened the Linux app on a desktop; the job only sees it open a window on a
    virtual display.
- [x] **5. `windows-app`** (#88) (expect Standard). Needs step 2 green. A Windows leg: the command as a
  `.zip` and the app as an installer *(recon: NSIS or MSI)*. The window config uses macOS-only
  title bar options, so expect layout fixes. The owner runs the dry-run build on the gaming PC or
  a VM and reports; the step closes on that report, not on a green job.
  - The owner ran the test script on a real PC on 2026-11-14 and reported it working, with
    nothing to fix (D-073).

### Wave C: needs steps 1 and 3, and the owner's word

- [ ] **6. First public release.** After the repository is public (its own plan, the owner's call).
  Tag, let the workflow run, read the draft, publish. Check one download per platform shipped.
  Rewrite the README's "Getting it" section around downloads, keeping the build-from-source path.

### Wave D: needs a published release

Release assets of a private repository cannot be downloaded anonymously, so the tap waits for step 6.

- [ ] **7. `homebrew-tap`** (expect Standard). A new public repository, `homebrew-tap`, which the
  owner creates or approves by name. The release job updates it after a release is published,
  using a token scoped to that one repository. Done when `brew install` from the tap gives a
  working app and command on a clean Mac, and `brew upgrade` picks up the next release.

## Working notes

- **Cost.** GitHub's runners are free for a public repository. While this one is private, macOS
  minutes count ten times against the free quota *(recon)*, so keep macOS dry runs few.
- **Who does what.** Use subagents for the independent steps of a wave. Opus for the contained
  ones (2, 4, the README pass), Fable for the ones where a wrong call is expensive (1, 3, 5, 7).
  A step that turns out harder than expected moves up.
- **Recon is cheap here and guessing is not.** Action versions, Tauri's bundler flags and
  Homebrew's rules all change; a release workflow that is wrong fails on the day of a release.
- **Acceptance criteria.** These steps have no CLI or GUI surface, so most need no numbered ACs.
  If one does, number from the next free id in `.claude/rules/lcd-conventions.md`.
