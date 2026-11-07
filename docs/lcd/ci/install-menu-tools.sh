#!/usr/bin/env bash
# Build the two tools the menu's checks need into PREFIX/bin, at pinned versions (D-038):
#   64tass 1.60.3243  reassembles menu/ and compares it with the committed image (AC-93)
#   VICE 3.10 x64sc   boots carts carrying the menu (AC-87..AC-92) and EasyLoader (AC-4)
#
# Usage: docs/lcd/ci/install-menu-tools.sh PREFIX
#
# Both come from source because the distribution packages follow the runner image: apt has
# 64tass 1.59 and an older VICE on ubuntu-24.04, and 3.10 fixed breakpoints that sometimes did not
# trigger, which every boot test relies on. VICE is built headless (no GTK), and its source
# tarball ships the C64 ROMs. Each archive is checked against its sha256 before it is used. A tool
# already present in PREFIX at the right version is left alone, so a restored cache costs nothing.
set -euo pipefail

prefix="${1:?usage: install-menu-tools.sh PREFIX}"
mkdir -p "$prefix/bin"
prefix="$(cd "$prefix" && pwd)"

TASS_VERSION=1.60.3243
TASS_SHA256=9d83be3d23a2c55e085b7c7a7856c2f96080447ea120a6a8c21a217ed76427f0
TASS_URL="https://sourceforge.net/projects/tass64/files/source/64tass-${TASS_VERSION}-src.zip/download"

VICE_VERSION=3.10
VICE_SHA256=8e5bac18cbcb9f192380ad3ef881f8790f5b75c41d7b3da65d831985d864d6d1
VICE_URL="https://sourceforge.net/projects/vice-emu/files/releases/vice-${VICE_VERSION}.tar.gz/download"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

fetch() { # url out sha256
  curl -fsSL --retry 3 -o "$2" "$1"
  echo "$3  $2" | sha256sum -c -
}

jobs="$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 2)"

if "$prefix/bin/64tass" --version 2>/dev/null | grep -q "V${TASS_VERSION}"; then
  echo "64tass ${TASS_VERSION}: cached"
else
  fetch "$TASS_URL" "$work/64tass.zip" "$TASS_SHA256"
  (cd "$work" && unzip -q 64tass.zip)
  make -C "$work/64tass-${TASS_VERSION}-src" -j"$jobs"
  install -m 755 "$work/64tass-${TASS_VERSION}-src/64tass" "$prefix/bin/64tass"
fi

if [[ -f "$prefix/.vice-${VICE_VERSION}" && -x "$prefix/bin/x64sc" ]]; then
  echo "VICE ${VICE_VERSION}: cached"
else
  fetch "$VICE_URL" "$work/vice.tar.gz" "$VICE_SHA256"
  tar -xzf "$work/vice.tar.gz" -C "$work"
  (
    cd "$work/vice-${VICE_VERSION}"
    ./configure --prefix="$prefix" \
      --enable-headlessui \
      --disable-html-docs --disable-pdf-docs \
      --without-pulse --without-alsa --without-png --without-libcurl \
      --disable-realdevice --disable-hardsid --disable-ipv6
    make -j"$jobs"
    make install
  )
  touch "$prefix/.vice-${VICE_VERSION}"
fi

"$prefix/bin/64tass" --version | head -1
# Run it, not just find it: a binary restored from the cache can be present and still not start
# (a shared library the build found is missing on this run). Say why instead of letting every
# boot test fail with an empty log.
if ! "$prefix/bin/x64sc" -help >/dev/null 2>&1; then
  echo "x64sc is installed but does not run:" >&2
  "$prefix/bin/x64sc" -help 2>&1 | tail -5 >&2 || true
  ldd "$prefix/bin/x64sc" >&2 || true
  exit 1
fi
echo "x64sc: $prefix/bin/x64sc (VICE ${VICE_VERSION})"
