# Tasks: Read a cartridge image back into the description that produced it

> **Deep lane · Phase 3 of 6 (Tasks).** A flat, dependency-ordered checklist. Each task names
> the file it touches. Tests come before the code they test (TDD). The granular STEPS in the
> work-item `JOURNAL.md` mirror this list as the resume view; keep them in sync.

**Slug:** `read-path`
**Spec:** [`spec.md`](./spec.md) • **Plan:** [`plan.md`](./plan.md) • **Journal:** [`JOURNAL.md`](./JOURNAL.md)
**Created:** 2026-08-11
**Status:** completed <!-- in_progress | completed | abandoned -->

---

## Conventions

- `[P]` suffix = parallelizable with the previous task (different file, no shared state).
- Tests precede implementation tasks they cover.
- One file per task where possible; tasks naming multiple files should be split unless the change is genuinely atomic.

## Tasks

> Each test task that covers a cross-path AC should have a separate task per surface, so the literal `AC-N (SURFACE)` string maps one-to-one to a task. See the ac-convention rule.
>
> Every AC here is `surfaces: none`, so each maps to exactly one test task carrying the literal
> `AC-N (none)`. T8 deliberately lands the contract (types and a `load` signature) *before* the AC
> tests: without it they would not compile, and a test file that cannot compile is a build error
> rather than a legible red. The stub returns an error, so every AC test is genuinely failing when
> written.

- [x] **T1**: Bump `Cargo.toml` 0.2.0 → 0.3.0 — `feat/` branch minor bump, first code commit of the branch (versioning rule; branch base verified as 0.2.0)
- [x] **T2**: Write failing inline tests for slot decoding in `crates/stidcart-core/src/directory.rs` — `encode`/`decode` round-trip, end marker recognised, hidden flag detected, a cartridge type value classified as unsupported, reserved flag bits tolerated whether set or clear
- [x] **T3**: Implement `decode` in `crates/stidcart-core/src/directory.rs` to pass T2 — inverse of `encode`, sharing its constants
- [x] **T4**: Write failing inline tests for container parsing in `crates/stidcart-core/src/crt.rs` — header magic and id, packet walk into a `FlashImage`, `write`→`parse` round-trip, both permitted data sizes [P]
- [x] **T5**: Implement `parse` in `crates/stidcart-core/src/crt.rs` to pass T4 — validate marker and total-length together, bounds-check before slicing, extract the header name without the character swap
- [x] **T6**: Write failing inline tests for reserved-region verification and cartridge-name extraction in `crates/stidcart-core/src/reserved.rs` — a written image verifies, a mutated region does not, name recovers through the swap [P]
- [x] **T7**: Implement `verify_regions` and `read_ef_name` in `crates/stidcart-core/src/reserved.rs` to pass T6 — compare against an image built by the existing writers rather than re-deriving expected bytes
- [x] **T8**: Create `crates/stidcart-core/src/read.rs` with the contract only — `ReadError`, `PacketProblem`, `Unsupported`, `Display`/`Error` impls, and a `load` signature returning an error so the AC tests compile and fail
- [x] **T9**: Wire `pub mod read` and re-export `ReadError` in `crates/stidcart-core/src/lib.rs`; add `header_name()` and `ef_name()` accessors alongside the existing `programs()`
- [x] **T10**: Write failing test named `AC-1 (none): a loaded image describes the programs that produced it` in `crates/stidcart-core/tests/read.rs`
- [x] **T11**: Write failing test named `AC-3 (none): an empty directory loads as a cartridge with no programs` in `crates/stidcart-core/tests/read.rs` [P]
- [x] **T12**: Write failing test named `AC-5 (none): a corrupt packet marker or disagreeing length is refused` in `crates/stidcart-core/tests/read.rs` [P]
- [x] **T13**: Write failing test named `AC-6 (none): a directory with no end marker anywhere is refused` in `crates/stidcart-core/tests/read.rs` [P]
- [x] **T14**: Write failing test named `AC-7 (none): cartridge and hidden entries are refused as unsupported` in `crates/stidcart-core/tests/read.rs` [P]
- [x] **T15**: Write failing test named `AC-8 (none): the container header name is recovered and never re-stamped` in `crates/stidcart-core/tests/read.rs` [P]
- [x] **T16**: Write failing test named `AC-9 (none): the header name is not character-swapped while entry and cartridge names are` in `crates/stidcart-core/tests/read.rs` [P]
- [x] **T17**: Write failing test named `AC-10 (none): reserved firmware differing from the embedded blobs is refused as unsupported` in `crates/stidcart-core/tests/read.rs` [P]
- [x] **T18**: Write failing test named `AC-12 (none): a size or location reaching outside the image is refused` in `crates/stidcart-core/tests/read.rs` [P]
- [x] **T19**: Write failing test named `AC-2 (none): every reference image loads and rebuilds byte-identically` in `crates/stidcart-core/tests/roundtrip.rs` [P]
- [x] **T20**: Write failing test named `AC-4 (none): a cart this tool builds from no programs loads and rebuilds byte-identically` in `crates/stidcart-core/tests/roundtrip.rs` [P]
- [x] **T21**: Write failing test named `AC-11 (none): an entry continuing past its starting region recovers unbroken` in `crates/stidcart-core/tests/roundtrip.rs` [P]
- [x] **T22**: Implement the container stage of `load` in `crates/stidcart-core/src/read.rs` — header, packet walk, header name — passing T12, T15, T16 (header half), T18
- [x] **T23**: Implement the reserved stage of `load` in `crates/stidcart-core/src/read.rs` — region verification and cartridge name — passing T16 (name half) and T17
- [x] **T24**: Implement the directory stage of `load` in `crates/stidcart-core/src/read.rs` — slot walk, entry classification, program recovery — passing T10, T11, T13, T14, T21
- [x] **T25**: Confirm the round-trip closes over all four reference images and the empty base case, passing T19 and T20; if a case fails, localize by module test before touching `read.rs`
- [x] **T-audit**: Run `/lcd:audit read-path`; resulting `audit.md` has zero MISSING/BLOCKED rows
- [x] **T-final**: Run `cargo fmt --all --check && cargo clippy --workspace --all-targets --all-features -- -D warnings && cargo test --workspace`, all green

## Checkpoint validation

After every block of related tasks, the suite must stay green. If T3 passes but T5's implementation breaks T1, stop and fix before moving on.

Three checkpoints matter more than the rest here:

- After **T7**, every decoder exists but nothing orchestrates them. Run the whole suite: the existing
  write-path tests must still be green, because nothing so far should have changed a single produced
  byte except the version bump in T1.
- After **T9**, the AC tests compile and the whole read half of the suite is red. That is the
  intended state — check that it is red for the right reason (an error return) rather than a panic.
- After **T24**, the round-trip in T25 either closes or does not. A failure localizes by layer:
  check `crt` parsing first, then `reserved`, then `directory`, then program recovery. The existing
  parity test's failure message already assumes this ordering.

## Linkbacks

- `AC-1 (none)` → T10, T24
- `AC-2 (none)` → T19, T25
- `AC-3 (none)` → T11, T24
- `AC-4 (none)` → T20, T25
- `AC-5 (none)` → T12, T22
- `AC-6 (none)` → T13, T24
- `AC-7 (none)` → T14, T3, T24
- `AC-8 (none)` → T15, T22
- `AC-9 (none)` → T16, T22, T23
- `AC-10 (none)` → T17, T7, T23
- `AC-11 (none)` → T21, T24
- `AC-12 (none)` → T18, T5, T22

---

<!--
Quality checks before starting implementation:
  - Every AC × non-`none`-surface row in plan.md has at least one test task above
  - Every new file in plan.md "File structure" has at least one task that creates it
  - Tests appear before their implementation
  - T-audit appears before T-final (the audit gate must pass before suite-final)
  - T-final exists (run the suite, lint, type check)
-->
