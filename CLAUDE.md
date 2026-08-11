# stidcart

C64 cartridge tooling in Rust. See `README.md` for orientation and `docs/lcd/MAP.md` for structure.

## Knowledge vault references

Pointers only — read the note, never copy its content here.

- `reference/easyflash-cart-format.md` (Obsidian vault at `~/Documents/obsidian/stid`) — the binary
  format: CRT container, EasyFS2 directory, bank-0 reserved map, placement rules, and which
  ndefpack behaviours are format-mandated versus implementation artifacts. **Read it before
  touching anything in `stidcart-core`.** It is the source for every offset in this repo; the
  format is documented nowhere in ndefpack's own tree, so re-deriving it from that C risks
  inheriting a quirk by accident.
- `projects/ndefpack.md` — the predecessor's decision log, including why this repo exists.

## The reference implementation

`ndeft` lives at `~/Documents/develop/ndefpack` (build it with `cmake -B build -G Ninja &&
cmake --build build -j`). It is **not** a dependency: it produced the frozen carts in `fixtures/`
once, and the test suite never invokes it. You need it only to add a new fixture — record the exact
request in `fixtures/MANIFEST.md` when you do.

## Lean Context Development (LCD)
Non-trivial new work (new feature, ≥3 files, or any architecture change) → invoke
`lcd:triage`, which picks a lane (Quick / Standard / Deep) and states it in one line.
Trivial work (typo, one-liner, dep bump, known-cause fix) → go direct, no triage.
Artifacts live under `docs/lcd/` (see `.claude/rules/lcd-conventions.md`).
Project map: `docs/lcd/MAP.md` · Decisions: `docs/lcd/DECISIONS.md`
Resume any work-item after a context reset: `/lcd:resume <slug>`.
