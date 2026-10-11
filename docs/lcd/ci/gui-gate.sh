#!/usr/bin/env bash
# gui-gate.sh — the GUI's quality gate (D-030), run locally and by CI's `gui` job alike so the
# two cannot drift. The root gate never sees gui/: it is its own Cargo workspace.
#
# Order matters: `tauri::generate_context!` embeds gui/dist at compile time, so the frontend is
# built before any cargo step.
set -euo pipefail

root="$(git rev-parse --show-toplevel)"
cd "$root/gui"

# Install only when the lockfile changed since the last install (or nothing is installed).
if [[ ! -d node_modules ]] || [[ package-lock.json -nt node_modules/.package-lock.json ]]; then
  npm ci --no-audit --no-fund
fi

npm run check
npm test
npm run build

cd src-tauri
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
