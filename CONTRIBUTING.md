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

## Building it yourself

You need [Rust](https://rustup.rs) installed through rustup, which picks up the compiler version
this repo pins. The desktop app also needs [Node](https://nodejs.org) 24.

```bash
git clone https://github.com/stid/stidcart.git
cd stidcart
```

On a Mac, one script builds the app into `/Applications` and installs the `stidcart` command:

```bash
scripts/install.sh
```

It leaves the command alone if Homebrew has already installed it.

For the command alone, on any system:

```bash
cargo install --path crates/stidcart-cli --locked
```

What the app needs to build on Linux is under the next heading.

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

## Actions in the workflows

Each `uses:` line names an action by commit, with its version in a comment beside it
(`actions/checkout@3d3c42e… # v7.0.1`), so a tag moved upstream cannot change what runs here.
Dependabot offers the newer pins in one pull request a week (`.github/dependabot.yml`). Add a new
action the same way: the full commit of a released version, then the version.

## Cutting a release

A release is built by `.github/workflows/release.yml` from a `v*` tag on `main`.
[RELEASING.md](RELEASING.md) has the steps, the Homebrew tap and the signing secrets.

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
