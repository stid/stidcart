# stidcart

C64 cartridge tooling in Rust. See `README.md` for orientation and `docs/lcd/MAP.md` for structure.

## The format spec

`docs/format/easyflash-cart-format.md` — the binary format: CRT container, EasyFS2 directory,
bank-0 reserved map, placement rules, and which ndefpack behaviours are format-mandated versus
implementation artifacts. **Read it before touching anything in `stidcart-core`.** It is the source
for every offset in this repo; the format is documented nowhere in ndefpack's own tree, so
re-deriving it from that C risks inheriting a quirk by accident. A format fact changes there first,
in a reviewable commit, and only then in code (D-018).

## The reference implementation

`ndeft` is the command-line tool of [ndefpack](https://ndefpack.sourceforge.net/). It is **not** a
dependency: it produced the frozen carts in `fixtures/` once, and the test suite never invokes it.
You need it only to add a new fixture — record the exact request in `fixtures/MANIFEST.md` when you
do.

## How to work here

Read `docs/lcd/MAP.md` first on a cold start — it says where things live and lists the invariants that
hold across all work here. `docs/lcd/SPEC.md` says what the tool currently does; read the surface
section you're about to touch. Non-trivial work routes through triage first (see below).

**The gate**, green before every commit:

```bash
docs/lcd/ci/gate.sh              # what the working tree changes against HEAD
docs/lcd/ci/gate.sh origin/main  # that, plus the branch's commits: before a PR
docs/lcd/ci/gate.sh --all        # everything
```

It runs only the groups the change can affect (D-071), and CI's two jobs take the same scope from
the same script, `docs/lcd/ci/scope.sh`:

| Change | Root fmt, clippy, tests | VICE boots (`STIDCART_VICE`) | GUI gate |
|---|---|---|---|
| Prose, `docs/`, `.claude/`, `scripts/`, `release.yml` | no | none | no |
| `menu/` | yes | full | no |
| `crates/` sources, `firmware/`, `Cargo.*`, the toolchain pin | yes | cart, the 17 boots that depend on what the builder writes | yes |
| A root test or `fixtures/` | yes | none; full for a boot test or `tests/support/` | no |
| `gui/` | no | none | yes |
| `ci.yml`, `docs/lcd/ci/`, a path no rule names | yes | full | yes |

A new path that is not prose needs a rule in `scope.sh` and a case in
`crates/stidcart-cli/tests/scope.rs`; until then it runs everything. A boot test whose outcome
depends on the builder's output gates on `vice::require_cart_tool`, any other on
`vice::require_tool`. Run `--all` when a scoped green is not enough evidence, and say so in the PR.
A bare `cargo test --workspace` still runs every boot.

The compiler that runs it is pinned by `rust-toolchain.toml` (for both workspaces), and the pin binds only through
rustup's shims — a Homebrew `cargo` ignores the file silently (D-010).

The GUI (`gui/`) is its own Cargo workspace, so the root cargo commands never build it. Its gate is
`docs/lcd/ci/gui-gate.sh` (CI job `gui`), which `gate.sh` calls when the scope says so. It builds the
frontend first because Tauri embeds `gui/dist` at compile time, so a bare `cargo test` in
`gui/src-tauri` fails until `npm run build` has run (D-030).

- **Tests first.** Cargo collects `crates/*/tests/*.rs` and inline `#[cfg(test)]` modules
  automatically; nothing needs registering. (Not true of ndefpack, where a test missing from
  `NDEFPACK_UNIT_TESTS` compiles into nothing and reports a false green — don't carry that habit over.)
- **Never commit to `main`.** Branch, PR, conventional commits (`feat:`/`fix:`/`refactor:`/`chore:`/
  `test:`/`docs:`). Required checks: `gate` and `gui` (D-065).
- **Releases come from a `v*` tag** through `.github/workflows/release.yml` (D-072). A PR that
  changes that file dry-runs it; `CONTRIBUTING.md` has the steps to cut one.
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
