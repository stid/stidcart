#!/usr/bin/env bash
# run-test.sh — one scoped test target, whichever runner owns it (the LCD `scoped-test` key).
#
#   <stem>                         root workspace target  → cargo test --workspace --test <stem>
#   gui/src-tauri/tests/<stem>.rs  GUI Rust target        → cargo test (GUI workspace) --test <stem>
#   <path>.test.ts                 frontend test file     → vitest run <file>
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <stem | gui/src-tauri/tests/<stem>.rs | path/to/file.test.ts>" >&2
  exit 2
fi

root="$(git rev-parse --show-toplevel)"
target="$1"

case "$target" in
  *.test.ts)
    cd "$root/gui"
    exec npx vitest run "${target#gui/}"
    ;;
  gui/src-tauri/tests/*.rs)
    stem="$(basename "$target" .rs)"
    exec cargo test --manifest-path "$root/gui/src-tauri/Cargo.toml" --test "$stem"
    ;;
  *)
    cd "$root"
    exec cargo test --workspace --test "$target"
    ;;
esac
