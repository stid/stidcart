# Contributing

stidcart is a hobby project with one maintainer. Bug reports, reports of carts that do or don't
work, and corrections to the credits are welcome as issues. I look at pull requests when I have
time, and I can't promise to merge them or to answer quickly.

## How it is written

Most of the code was written by Claude Code, an AI coding assistant. I set the direction, make the
decisions and test on real hardware; the automated tests check the rest. The files the assistant works
from are in the repo and stay there: `CLAUDE.md`, `.claude/` and `docs/lcd/`, which holds the
project map, the current-state spec and the log of every design decision with the alternatives
rejected.

## Layout

`ndeft` is ndefpack's command-line tool. It built the reference carts in `fixtures/` once and is
not needed to build or test stidcart.

```
crates/stidcart-core/   library: format, placement, firmware. No I/O policy, no globals.
crates/stidcart-cli/    binary `stidcart`: the command line. Arguments, files and output only.
gui/                    desktop app: Tauri 2 + Svelte 5, its own Cargo workspace (gui/src-tauri)
firmware/               EasyLoader and EAPI, zlib, with their source (see firmware/README.md)
menu/                   our own C64 menu, GPL 6502, committed assembled (see menu/README.md)
fixtures/               ndeft-built carts, frozen, with the exact command line for each
```

## Building and testing

The Rust toolchain is pinned in `rust-toolchain.toml`, and the pin only binds through rustup. A
Homebrew `cargo` ignores the file without saying so.

```bash
docs/lcd/ci/gate.sh              # what your working tree changes
docs/lcd/ci/gate.sh origin/main  # that, plus your branch's commits
docs/lcd/ci/gate.sh --all        # everything
```

That is the gate, and CI runs the same one. It runs only the test groups your change can affect:
nothing for prose, every emulator boot and no desktop app for `menu/`, a set of 17 boots and the
app for the Rust sources, the app alone for `gui/`. `docs/lcd/ci/scope.sh` holds the rules, and a
path it does not know runs everything. In full, the root half is

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets --all-features -- -D warnings && cargo test --workspace
```

The parity test needs only the checked-in fixtures; `ndeft` does not have to be installed. The
boot tests need VICE's `x64sc` and the menu's reassembly check needs 64tass. Without them those
tests skip, except under `STIDCART_REQUIRE_TOOLS=1`, which CI sets, where a missing tool fails.

A VICE with a window opens one for every boot, about ninety of them in a full run. To keep them
off your screen, build VICE without a UI (`docs/lcd/ci/install-menu-tools.sh` has the configure
line CI uses; on macOS it also needs `dos2unix` and `xa` from Homebrew) and point the tests at it:

```bash
export STIDCART_X64SC=~/.local/vice-headless/bin/x64sc
```

Unset, the boots run the `x64sc` on PATH. A path given there that does not run fails the boot
tests instead of skipping them.

The desktop app is its own Cargo workspace under `gui/src-tauri`, so that gate never builds it. It
needs Node 24, pinned in `gui/.nvmrc` (`nvm use` in `gui/` picks it up), and, on Linux, the
WebKitGTK packages the CI `gui` job installs.

```bash
cd gui && npm install
npm run tauri dev          # run the app
../docs/lcd/ci/gui-gate.sh # its half of the gate: type-check, vitest, build, then clippy and tests
```

`menu/build.sh menu/` reassembles the C64 menu after a change to `menu/src` (it needs 64tass).
Commit `menu.bin` and `menu.lbl` with the source.

Work goes on a branch and through a pull request, with conventional commit messages (`feat:`,
`fix:`, `docs:` and so on).

## Cutting a release

A release is built by `.github/workflows/release.yml` from a tag on `main`.

1. In a pull request, set the new version in `Cargo.toml` (`[workspace.package]`) and in
   `gui/src-tauri/Cargo.toml`, and let both `Cargo.lock` files follow. The two must agree:
   `docs/lcd/ci/release-version.sh` prints the version and fails if they do not.
2. Once it is merged, tag that commit `v` plus the version and push the tag:
   `git tag v0.31.0 && git push origin v0.31.0`.
3. The workflow builds the command for macOS, Linux and Windows, the macOS app as a `.dmg`, the
   Linux app as an AppImage and a `.deb` and the Windows app as an installer, and attaches them
   with a `SHA256SUMS` file to a draft release. Read the draft, try a download, then publish it.
4. Publishing runs `.github/workflows/tap.yml`, which points the Homebrew tap at the new release
   (see below). Its run and the tap's own `Tests` run should both end green.

The workflow refuses a tag that names another version or a commit that is not on `main`. A pull
request that changes the workflow runs the same build as a dry run: nothing is released, and the
files are in the run's `release` artifact. To repeat the run for a tag, delete the draft first.

A `v*` tag cannot be moved or deleted once pushed, and a published release cannot be changed:
its files and its tag are locked (D-078). Homebrew checks each download against the checksum in
the tap, so a file is never replaced. A release that went out wrong is followed by a new version.
A tag pushed on the wrong commit is removed by switching off the "release tags" ruleset in the
repository's settings for as long as that takes.

### The Homebrew tap

[stid/homebrew-tap](https://github.com/stid/homebrew-tap) holds two recipes: formula `stidcart`
for the command and cask `stidcart-app` for the app. Each names a release's macOS file and its
checksum. When a release is published, `tap.yml` rewrites both from that release's `SHA256SUMS`
and pushes, and the tap's `Tests` workflow then installs both on a Mac that has never had
stidcart on it. A prerelease leaves the tap alone.

The push needs one repository secret, `TAP_TOKEN`: a fine-grained personal access token limited to
the `homebrew-tap` repository, with read and write access to its contents and nothing else. Make
it under GitHub's Settings, Developer settings, Personal access tokens, then set it from a
terminal:

```bash
gh secret set TAP_TOKEN   # asks for the value
```

The token expires on the date chosen when it was made. After that a release still publishes and
only the tap update fails. Set a new token, then repeat the update from the Actions tab: "Tap",
"Run workflow", with the release's tag. A pull request that changes `tap.yml` dry-runs it against
the latest release, printing the change without pushing.

### Signing the macOS builds

The macOS app and command are signed with a Developer ID and notarized, from five repository
secrets. A run without them (a fork's pull request) still builds, with the app signed ad hoc, and a
tag refuses to build that way. Setting them up takes an Apple Developer Program membership and is
done once:

1. Make a certificate request: Keychain Access, Certificate Assistant, "Request a Certificate From
   a Certificate Authority", saved to disk.
2. At developer.apple.com, under Certificates, add a "Developer ID Application" certificate from
   that request. Download it and open it, which puts it in the login keychain.
3. In Keychain Access, under My Certificates, export it together with its private key as a `.p12`
   with a password.
4. In App Store Connect, under Users and Access, Integrations, create a team API key. Download its
   `.p8` file, which is offered once, and note the key ID and the issuer ID.
5. Set the secrets from a terminal, so that none of them passes through anything else:

   ```bash
   base64 -i certificate.p12 | gh secret set APPLE_CERTIFICATE
   gh secret set APPLE_CERTIFICATE_PASSWORD   # asks for the value
   gh secret set APPLE_API_KEY                # the key ID
   gh secret set APPLE_API_ISSUER             # the issuer ID
   gh secret set APPLE_API_KEY_P8 <AuthKey_XXXXXXXXXX.p8
   ```

The certificate expires (the one in use on 2027-12-16), and the developer site shows the date.
After that nothing new can be signed until steps 1 to 3 are repeated and the two certificate
secrets set again; builds already released keep opening. Keep the `.p12` and the `.p8` somewhere
safe and out of the repository.

The Windows installer is not signed, so SmartScreen stops it with "Windows protected your PC":
"More info", then "Run anyway". It installs for the current user and asks for no administrator
rights.

## The format spec comes first

`docs/format/easyflash-cart-format.md` is the source for every offset in `stidcart-core`. A format
fact changes there first, and then in the code, in the same pull request.

## Why the write path came first

stidcart could build carts before it could open them, and that order was deliberate.

ndefpack's `ndeft` was the only tool I knew whose images boot on real EasyFlash hardware. So the
first test was simple: build the same inputs with both tools and compare the bytes. That check is
only worth something while stidcart is trying to produce the same output as ndefpack. Once it does
things ndefpack can't, such as its own menu, there is nothing to compare against, and those parts
are tested by booting them in VICE and on the real cartridge. The comparison was most useful at
the start, so the work started there.

Reading carts raised a question that writing them did not: how far to copy ndefpack's quirks. Its
loader rejects an empty directory, never checks the CHIP magic, and can't read wide 16K xbank
carts. Building the write path first left time to decide those one at a time. stidcart reads wide
xbank carts, for one.

## Licensing of contributions

Everything outside `firmware/` is GPL-2.0-or-later, and a contribution is taken under that
license. `firmware/` holds other people's zlib-licensed code, unchanged; nothing moves across
that line in either direction.
