#!/usr/bin/env bash
# gate.sh — the quality gate, scoped to what changed (D-071). Green before every commit.
#
#   gate.sh          what the working tree changes against HEAD: the check before a commit
#   gate.sh BASE     that, plus every commit since BASE: the check before a PR (BASE = origin/main)
#   gate.sh --all    everything, whatever changed
#
# scope.sh decides the groups; this runs them. CI's `gate` and `gui` jobs take the same scope and
# run the same commands, so keep ci.yml's steps in line with the two blocks below.
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
cd "$root"

scope="$(docs/lcd/ci/scope.sh "$@")"
eval "$scope"
echo "scope: rust=$rust vice=$vice gui=$gui"

if [[ $rust == false && $gui == false ]]; then
  echo "nothing to run: no change here reaches a test. gate.sh BASE covers commits, --all everything."
  exit 0
fi

if [[ $rust == true ]]; then
  cargo fmt --all --check
  cargo clippy --workspace --all-targets --all-features -- -D warnings
  STIDCART_VICE="$vice" cargo test --workspace
fi

if [[ $gui == true ]]; then
  docs/lcd/ci/gui-gate.sh
fi
