#!/usr/bin/env python3
"""Synthetic cartridge images used as `ndeft` inputs for the reference fixtures.

Deterministic, so a fixture can be reproduced from this script plus the request recorded in
MANIFEST.md. Every bank half of every image carries its own byte ramp, so a bank copied to the
wrong place shows up as wrong data rather than as a duplicate.

Bank body byte i of file k, bank b, half h (0 = ROML, 1 = ROMH):  (i * 7 + 3 + 5*k + 2*b + h) % 251

The second table is not synthetic: each entry patches the directory of a frozen reference image,
for an entry shape the reference tool carries but never writes itself (see PATCHED).

Run from inside fixtures/:  python3 make_inputs.py
"""

import os

BANK = 0x2000
MAGIC = b"C64 CARTRIDGE   "
ROML, ROMH, ULT = 0x8000, 0xA000, 0xE000


def header(crt_id, exrom, game, name):
    h = bytearray(0x40)
    h[0:16] = MAGIC
    h[0x10:0x14] = (0x40).to_bytes(4, "big")
    h[0x14:0x16] = (0x0100).to_bytes(2, "big")
    h[0x16:0x18] = crt_id.to_bytes(2, "big")
    h[0x18] = exrom
    h[0x19] = game
    n = name.encode("ascii")[:32]
    h[0x20 : 0x20 + len(n)] = n
    return bytes(h)


def chip(bank, addr, data, chip_type):
    return (
        b"CHIP"
        + (0x10 + len(data)).to_bytes(4, "big")
        + chip_type.to_bytes(2, "big")
        + bank.to_bytes(2, "big")
        + addr.to_bytes(2, "big")
        + len(data).to_bytes(2, "big")
        + data
    )


def body(k, b, h, n=BANK):
    return bytes((i * 7 + 3 + 5 * k + 2 * b + h) % 251 for i in range(n))


# EXROM/GAME per mode, as the reference tool reads them (cart.c:180): 16k = 0/0, 8k = 0/1, Ultimax = 1/0.
M16K, M8K, MULT = (0, 0), (0, 1), (1, 0)

# (name, crt id, (exrom, game), chip type, [(bank, addr, half, length)])
IMAGES = [
    ("in_8k", 0, M8K, 0, [(0, ROML, 0, BANK)]),
    ("in_16k", 0, M16K, 0, [(0, ROML, 0, 2 * BANK)]),
    ("in_ult8", 0, MULT, 0, [(0, ULT, 1, BANK)]),
    ("in_ult16", 0, MULT, 0, [(0, ROML, 0, BANK), (0, ULT, 1, BANK)]),
    # Ocean 256 KiB: 16 banks at $8000 then 16 at $A000, numbered straight through (VICE's layout).
    ("in_ocean", 5, M16K, 0, [(b, ROML, 0, BANK) for b in range(16)] + [(b, ROMH, 1, BANK) for b in range(16, 32)]),
    ("in_ocean512", 5, M8K, 0, [(b, ROML, 0, BANK) for b in range(64)]),
    ("in_xb8", 33, M8K, 2, [(b, ROML, 0, BANK) for b in range(4)]),
    ("in_xb16", 33, M16K, 2, [p for b in range(4) for p in ((b, ROML, 0, BANK), (b, ROMH, 1, BANK))]),
    ("in_xbult", 33, MULT, 2, [p for b in range(2) for p in ((b, ROML, 0, BANK), (b, ULT, 1, BANK))]),
    # 40 banks in both halves: 640 KiB, past the reference tool's 512 KiB entry cap (D-022).
    ("in_xbwide", 33, M16K, 2, [p for b in range(40) for p in ((b, ROML, 0, BANK), (b, ROMH, 1, BANK))]),
]

# CrtUsage subdirectory bit (bytes 19-20 of a cart entry). The reference tool reads and writes it
# back on an xbank entry but no request of its own sets it, so the input is a reference image with
# the bit patched in. (output, source reference image, directory slot)
SUBDIR = 0x4000
PATCHED = [
    ("in_cxb16sub", "cxb16.crt", 1),
]


def patch_subdir(src, slot):
    """Set the subdirectory bit in one slot of the directory at bank 0 ROMH."""
    img = bytearray(open(src, "rb").read())
    p = int.from_bytes(img[0x10:0x14], "big")
    while p < len(img):
        length = int.from_bytes(img[p + 4 : p + 8], "big")
        bank = int.from_bytes(img[p + 10 : p + 12], "big")
        addr = int.from_bytes(img[p + 12 : p + 14], "big")
        if bank == 0 and addr in (ROMH, ULT):
            at = p + 0x10 + slot * 24 + 19
            word = int.from_bytes(img[at : at + 2], "little") | SUBDIR
            img[at : at + 2] = word.to_bytes(2, "little")
            return bytes(img)
        p += length
    raise SystemExit(f"{src}: no bank 0 ROMH packet")


def main():
    os.chdir(os.path.dirname(os.path.abspath(__file__)))
    for k, (name, crt_id, (exrom, game), chip_type, packets) in enumerate(IMAGES):
        out = bytearray(header(crt_id, exrom, game, name))
        for bank, addr, half, length in packets:
            data = body(k, bank, half, length) if length == BANK else body(k, bank, 0) + body(k, bank, 1)
            out += chip(bank, addr, data, chip_type)
        with open(f"{name}.crt", "wb") as f:
            f.write(out)
        print(f"{name}.crt  {len(out)} bytes")
    for name, src, slot in PATCHED:
        out = patch_subdir(src, slot)
        with open(f"{name}.crt", "wb") as f:
            f.write(out)
        print(f"{name}.crt  {len(out)} bytes (from {src})")


if __name__ == "__main__":
    main()
