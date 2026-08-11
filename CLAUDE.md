# stidcart

C64 cartridge tooling in Rust. See `README.md` for orientation and `docs/lcd/MAP.md` for structure.

## Lean Context Development (LCD)
Non-trivial new work (new feature, ≥3 files, or any architecture change) → invoke
`lcd:triage`, which picks a lane (Quick / Standard / Deep) and states it in one line.
Trivial work (typo, one-liner, dep bump, known-cause fix) → go direct, no triage.
Artifacts live under `docs/lcd/` (see `.claude/rules/lcd-conventions.md`).
Project map: `docs/lcd/MAP.md` · Decisions: `docs/lcd/DECISIONS.md`
Resume any work-item after a context reset: `/lcd:resume <slug>`.
