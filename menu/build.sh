#!/bin/sh
# Assemble the menu into OUTDIR/menu.bin (the 8 KiB loader slot, raw) and OUTDIR/menu.lbl (VICE
# labels). `menu/build.sh menu/` regenerates the committed pair; commit both with any source change.
# The flags are pinned (D-038): -a ASCII source, -C case-sensitive labels, -Wall -Werror, -b a raw
# image from the lowest to the highest address written.
set -eu
here=$(cd "$(dirname "$0")" && pwd)
out=${1:?usage: build.sh OUTDIR}
mkdir -p "$out"
64tass --quiet -a -C -Wall -Werror -b --vice-labels -l "$out/menu.lbl" -o "$out/menu.bin" "$here/src/menu.s"
