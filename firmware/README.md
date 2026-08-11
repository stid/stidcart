# Firmware blobs — zlib licensed

Everything in this directory is **zlib licensed** (see `COPYING` here), not GPL. It is the
C64-side firmware, copied unmodified from ndefpack's `src/fw/`, which took it from the
EasyFlash project:

- EasyFlash — (c) 2009-2010 Thomas 'skoe' Giesel
- EasyLoader — (c) 2009-2012 ALeX Kazik

| File | Size | Role | Lands at |
|---|---|---|---|
| `eapi_am29f040.prg` | 770 | EAPI (flash-write API) | `0x81800`, 0x300 bytes |
| `easyloader_nrm.prg` | 8194 | menu loader, normal mode | `0x00000` (bank 0 ROML, `$8000`) |
| `easyloader_ocm.prg` | 8194 | menu loader, Ocean mode | `0x82000` (bank 1 ROMH, `$A000`) |
| `easyloader_launcher_nrm.prg` | 88 | boot code, normal mode | `0x81b18` region, at `$FFAA` |
| `easyloader_launcher_ocm.prg` | 88 | boot code, Ocean mode | `0x81b18` region |

Each file is a C64 `.prg`: two bytes of little-endian load address, then the data.

**The boot region is not the launcher.** It is 0x4E8 bytes based at `$FB18`, filled with `0xFF`
and then overlaid with the launcher at `start_addr - $FB18`. For the normal launcher that start
address is `$FFAA`, so only the last 86 bytes are non-`0xFF` and they end in the 6502 vectors —
RESET at `$FFFC` points to `$FFAA`. Reproducing the `0xFF` fill is required for byte parity.

The rest of this repository is GPL (see `../COPYING`). Do not move code across that boundary.
These blobs are embedded as data; nothing here is compiled from source in this repo.

Replacing this firmware with our own is planned, but is a separate piece of work — until then
these are shipped as-is.
