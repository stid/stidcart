# Contributing

stidcart is a hobby project with one maintainer. Bug reports, reports of carts that do or don't
work, and corrections to the credits are welcome as issues. I look at pull requests when I have
time, and I can't promise to merge them or to answer quickly.

## How it is written

Most of the code was written by Claude Code, an AI coding assistant. I set the direction, make the
decisions and test on real hardware; the tests described here check the rest. The files the assistant works
from are in the repo and stay there: `CLAUDE.md`, `.claude/` and `docs/lcd/`, which holds the
project map, the current-state spec and the log of every design decision with the alternatives
rejected.

## Layout

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
cargo fmt --all --check && cargo clippy --workspace --all-targets --all-features -- -D warnings && cargo test --workspace
```

That is the gate CI runs. The parity test needs only the checked-in fixtures; `ndeft` does not have to be installed. The
boot tests need VICE's `x64sc` and the menu's reassembly check needs 64tass. Without them those
tests skip, except under `STIDCART_REQUIRE_TOOLS=1`, which CI sets, where a missing tool fails.

The desktop app is its own Cargo workspace under `gui/src-tauri`, so that gate never builds it. It
needs Node 24, pinned in `gui/.nvmrc` (`nvm use` in `gui/` picks it up), and, on Linux, the
WebKitGTK packages the CI `gui` job installs.

```bash
cd gui && npm install
npm run tauri dev          # run the app
../docs/lcd/ci/gui-gate.sh # its gate: type-check, vitest, build, then clippy and tests
```

`menu/build.sh menu/` reassembles the C64 menu after a change to `menu/src` (it needs 64tass).
Commit `menu.bin` and `menu.lbl` with the source.

Work goes on a branch and through a pull request, with conventional commit messages (`feat:`,
`fix:`, `docs:` and so on).

## The format spec comes first

`docs/format/easyflash-cart-format.md` is the source for every offset in `stidcart-core`. A format
fact changes there first, and then in the code, in the same pull request.

## Why the write path came first

`ndeft` is the only implementation known to produce images that boot on real EasyFlash hardware,
so it works as a differential oracle: build the same inputs with both, compare bytes. That oracle
**decays** — parity only applies where this tool is trying to produce identical output, and stops
applying the moment broader compatibility or custom firmware lands. Its value is highest at the
first commit, so it gets spent there.

The write path also has no bug-compatibility question. Reading someone else's cart does: ndefpack
has documented quirks in its loader (an empty directory rejected, the CHIP magic never checked,
wide 16k xbank carts unreadable) that a new implementation may or may not want to reproduce. This
one reads wide xbank carts.
Building the write path first defers that decision instead of pre-empting it.

## Licensing of contributions

Everything outside `firmware/` is GPL-2.0-or-later, and a contribution is taken under that
license. `firmware/` holds other people's zlib-licensed code, unchanged; nothing moves across
that line in either direction.
