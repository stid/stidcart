#!/usr/bin/env bash
# Install stidcart on this Mac, for its owner only: the desktop app into /Applications and the
# command-line tool into ~/.cargo/bin. Nothing is published or signed for distribution; the app
# is built here, so macOS opens it without a quarantine prompt. Run it again after pulling new
# work to update both.
#
#   scripts/install.sh
#
# Where Homebrew has installed the command (`brew install stid/tap/stidcart`), this leaves the
# command alone: Homebrew's comes first on PATH, so a second copy here would never run.
#
# The app's checked-in config keeps bundling off (CI never bundles), so the build turns it on for
# this run only, for a macOS .app.
set -euo pipefail

repo=$(cd "$(dirname "$0")/.." && pwd)
app=/Applications/stidcart.app
bundle="$repo/gui/src-tauri/target/release/bundle/macos/stidcart.app"

# Replacing the app under a running copy leaves that copy on the old code, and quitting it from
# here could meet the unsaved-changes prompt, so the owner quits it.
if pgrep -f "$app/Contents/MacOS/" >/dev/null; then
  echo "stidcart is running. Quit it (saving any changes), then run this again." >&2
  exit 1
fi

echo "-- building the app (release, a few minutes)"
cd "$repo/gui"
[[ -d node_modules ]] || npm ci
npm run tauri build -- --config '{"bundle":{"active":true,"targets":["app"]}}'

echo "-- installing $app"
rm -rf "$app"
ditto "$bundle" "$app"

if command -v brew >/dev/null && brew list --formula stidcart >/dev/null 2>&1; then
  echo "-- leaving the stidcart command to Homebrew"
else
  echo "-- installing the stidcart command"
  cargo install --path "$repo/crates/stidcart-cli" --locked --quiet
fi

echo "-- installed"
echo "app:     $app ($(/usr/libexec/PlistBuddy -c 'Print CFBundleShortVersionString' "$app/Contents/Info.plist"))"
echo "command: $(command -v stidcart || echo "$HOME/.cargo/bin/stidcart, which is not on PATH")"
