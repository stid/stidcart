#!/usr/bin/env bash
# release-version.sh — the version a release is built from (D-072). release.yml calls it first.
#
#   release-version.sh              print the version, after checking the app's manifest agrees
#   release-version.sh TAG          that, and fail unless TAG is "v" followed by the version
#   release-version.sh --root DIR   read the manifests under DIR instead of this checkout
#
# The version is compiled into the tools and reaches the bytes of every cart they write (the
# header name), so a binary published under a tag that names another version is a wrong binary.
# The root workspace and the app are separate Cargo workspaces with a version each; nothing else
# holds them together. crates/stidcart-cli/tests/release_version.rs pins this.
set -euo pipefail

root=""
if [[ ${1:-} == --root ]]; then
  root="$2"
  shift 2
fi
[[ -n $root ]] || root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
tag="${1:-}"

# The `version` key of one table of a manifest; empty when the table or the key is missing.
# A Windows checkout can hold the manifest with CRLF line ends.
version_in() { # FILE TABLE
  awk -v table="[$2]" '
    { sub(/\r$/, "") }
    $0 == table { inside = 1; next }
    /^\[/ { inside = 0 }
    inside && /^version[ \t]*=/ { gsub(/^[^"]*"|".*$/, ""); print; exit }
  ' "$1"
}

fail() {
  echo "release-version: $*" >&2
  exit 1
}

tools="$(version_in "$root/Cargo.toml" workspace.package)"
app="$(version_in "$root/gui/src-tauri/Cargo.toml" package)"

[[ -n $tools ]] || fail "no version under [workspace.package] in $root/Cargo.toml"
[[ -n $app ]] || fail "no version under [package] in $root/gui/src-tauri/Cargo.toml"
[[ $tools == "$app" ]] ||
  fail "the tools are at $tools and the app at $app: bump Cargo.toml and gui/src-tauri/Cargo.toml together"
[[ -z $tag || $tag == "v$tools" ]] ||
  fail "the tag is $tag and the version is $tools: the tag must be v$tools"

echo "$tools"
