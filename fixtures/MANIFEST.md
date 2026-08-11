# Fixtures

Reference cartridge images produced by `ndeft` (ndefpack 1.0.1), frozen here so the byte-parity
check keeps working after that tool is gone. Nothing in the test suite invokes `ndeft`.

Captured **2026-08-10** with `ndeft` built from `stid/ndefpack@d77724b`, run from inside this
directory so the derived entry names are bare file stems.

## Inputs

Generated deterministically; each is a `.prg` loading at `$0801` with a byte ramp as its body.

| File | Body bytes | Formula |
|---|---|---|
| `hello.prg` | 100 | `(i * 7 + 3) % 251` |
| `ramp.prg` | 3000 | `(i * 7 + 11) % 251` |
| `big.prg` | 20000 | `(i * 7 + 29) % 251` |

`big.prg` is deliberately larger than one 8 KB bank, so it exercises the interleaved continuation.

## Reference images

| Image | Exact request | Covers |
|---|---|---|
| `single.crt` | `ndeft -q -o single.crt build hello.prg` | no cartridge name given — the placeholder path |
| `named.crt` | `ndeft -q -e "weird name" -o named.crt build hello.prg` | cartridge name stored in the flash region |
| `two.crt` | `ndeft -q -e "two entries" -o two.crt build hello.prg ramp.prg` | two entries, placed largest-first while the directory keeps input order |
| `spill.crt` | `ndeft -q -e "spill" -o spill.crt build big.prg` | an entry continuing across bank halves |

## Reproducing

```bash
cd fixtures
ndeft -q -o single.crt build hello.prg
ndeft -q -e "weird name" -o named.crt build hello.prg
ndeft -q -e "two entries" -o two.crt build hello.prg ramp.prg
ndeft -q -e "spill" -o spill.crt build big.prg
```

## Two things that will bite

- **`-e` is not the container header name.** It sets the cartridge name inside the flash image at
  `0x81b08`. The 32-byte name at offset `0x20` of the `.crt` always reads `ndefpack 1.0.1`, from
  `PACKAGE_STRING`. The parity test pins that string explicitly; see D-006.
- **Never boot a fixture in VICE.** `x64sc` defaults to `-easyflashcrtwrite` and saves the emulated
  flash back over the image on exit, stamping its own name into the header. Boot a copy, or pass
  `+easyflashcrtwrite`.
