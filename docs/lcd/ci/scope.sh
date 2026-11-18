#!/usr/bin/env bash
# scope.sh — which test groups a change can affect (D-071). The gate, locally (gate.sh) and in CI
# (ci.yml), runs only those.
#
#   scope.sh           the working tree against HEAD: staged, unstaged and untracked files
#   scope.sh BASE      the same, plus every commit since HEAD left BASE
#   scope.sh -         paths on stdin, one per line
#   scope.sh --all     everything, whatever changed
#
# Prints three lines, in the form $GITHUB_OUTPUT and `eval` both take:
#   rust=true|false       the root workspace: fmt, clippy, cargo test
#   vice=none|cart|full   which emulator boots those tests run (STIDCART_VICE): none, the boots
#                         that depend on what the builder writes, or all of them
#   gui=true|false        gui-gate.sh
#
# The rules follow what is compiled into what. The core embeds menu/menu.bin and firmware/, the
# CLI and the app are built on the core, and the boot tests run the CLI's carts in VICE. The menu
# reaches the app only as bytes the app never looks into, so a menu change leaves the app out.
# A path no rule names runs everything: a new directory is tested until someone says what it is.
# crates/stidcart-cli/tests/scope.rs pins these rules; change both together.
set -euo pipefail

rust=false vice=none gui=false

raise_vice() { # none < cart < full, never lowered
  case "$1:$vice" in
    full:*) vice=full ;;
    cart:none) vice=cart ;;
  esac
}

classify() {
  case "$1" in
    .github/workflows/ci.yml | docs/lcd/ci/*)
      rust=true gui=true
      raise_vice full
      ;;
    # Before the prose rule: the parity test reads fixtures/MANIFEST.md.
    fixtures/*)
      rust=true
      ;;
    crates/stidcart-cli/tests/boot.rs | crates/stidcart-cli/tests/menu_boot.rs | \
      crates/stidcart-cli/tests/meta_boot.rs | crates/stidcart-cli/tests/support/*)
      rust=true
      raise_vice full
      ;;
    # A root test is compiled into nothing but itself.
    crates/*/tests/*)
      rust=true
      ;;
    *.md) ;;
    menu/*)
      rust=true
      raise_vice full
      ;;
    crates/* | firmware/* | Cargo.toml | Cargo.lock | rust-toolchain.toml)
      rust=true gui=true
      raise_vice cart
      ;;
    gui/*)
      gui=true
      ;;
    # release.yml and tap.yml are checked by a dry run of themselves, not by a test.
    docs/* | .claude/* | scripts/* | COPYING | .gitignore | .github/workflows/lcd-audit.yml | \
      .github/workflows/release.yml | .github/workflows/tap.yml) ;;
    *)
      rust=true gui=true
      raise_vice full
      ;;
  esac
}

changed() {
  if [[ $# -eq 1 ]]; then
    git diff --name-only --no-renames "$1"...HEAD || exit 1
  fi
  git diff --name-only --no-renames HEAD || exit 1
  git ls-files --others --exclude-standard
}

case "${1:-}" in
  --all)
    classify "" # no rule names the empty path
    ;;
  -)
    while IFS= read -r path; do
      [[ -n "$path" ]] && classify "$path"
    done
    ;;
  *)
    cd "$(git rev-parse --show-toplevel)"
    # An assignment, not a pipe or a process substitution, and `|| exit 1` in changed(): a failed
    # `git diff` (an unknown BASE) must stop the script, not read as "nothing more changed".
    paths="$(changed "$@")"
    while IFS= read -r path; do
      [[ -n "$path" ]] && classify "$path"
    done <<<"$paths"
    ;;
esac

printf 'rust=%s\nvice=%s\ngui=%s\n' "$rust" "$vice" "$gui"
