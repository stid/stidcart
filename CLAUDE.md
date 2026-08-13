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

## How to work here

Read `docs/lcd/MAP.md` first on a cold start — it says where things live and lists the invariants that
hold across all work here. `docs/lcd/SPEC.md` says what the tool currently does; read the surface
section you're about to touch. Non-trivial work routes through triage first (see below).

**The gate**, green before every commit, and the same step CI requires on `main`:

```bash
cargo fmt --all --check && cargo clippy --workspace --all-targets --all-features -- -D warnings && cargo test --workspace
```

The compiler that runs it is pinned by `rust-toolchain.toml`, and the pin binds only through
rustup's shims — a Homebrew `cargo` ignores the file silently (D-010).

- **Tests first.** Cargo collects `crates/*/tests/*.rs` and inline `#[cfg(test)]` modules
  automatically; nothing needs registering. (Not true of ndefpack, where a test missing from
  `NDEFPACK_UNIT_TESTS` compiles into nothing and reports a false green — don't carry that habit over.)
- **Never commit to `main`.** Branch, PR, conventional commits (`feat:`/`fix:`/`refactor:`/`chore:`/
  `test:`/`docs:`). Required checks: `gate` and `claude-review`.
- **Log a decision when a real fork gets decided** — append to `docs/lcd/DECISIONS.md`, newest at top,
  including the alternatives rejected and why. Write it when the decision is made, not at the end.
  This is the one durable artifact; keep entries tight.

## Lean Context Development (LCD)

Non-trivial new work (new feature, ≥3 files, or any architecture change) → invoke `lcd:triage`,
which picks a lane (Quick / Standard / Deep) and states it in one line. Trivial work (typo,
one-liner, dep bump, known-cause fix) → go direct, no triage.

Artifacts live under `docs/lcd/` — the plugin's default root (D-014). See
`.claude/rules/lcd-conventions.md` for the machine block the suite reads.

Project map: `docs/lcd/MAP.md` · Decisions: `docs/lcd/DECISIONS.md` · Current state: `docs/lcd/SPEC.md`
Work-items: `docs/lcd/work/<slug>/`. Resume one after a context reset: `/lcd:resume <slug>`.
