# Tasks: Build a cartridge image from program files

> **Deep lane · Phase 3 of 6 (Tasks).** A flat, dependency-ordered checklist. Each task names
> the file it touches. Tests come before the code they test (TDD). The granular STEPS in the
> work-item `JOURNAL.md` mirror this list as the resume view; keep them in sync.

**Slug:** `write-path-skeleton`
**Spec:** [`spec.md`](./spec.md) • **Plan:** [`plan.md`](./plan.md) • **Journal:** [`JOURNAL.md`](./JOURNAL.md)
**Created:** 2026-08-10
**Status:** in_progress <!-- in_progress | completed | abandoned -->

---

## Conventions

- `[P]` suffix = parallelizable with the previous task (different file, no shared state).
- Tests precede implementation tasks they cover.
- One file per task where possible; tasks naming multiple files should be split unless the change is genuinely atomic.

## Tasks

> Each test task that covers a cross-path AC should have a separate task per surface, so the literal `AC-N (SURFACE)` string maps one-to-one to a task. See the ac-convention rule.

- [ ] **T1**: Write failing inline tests for the character-encoding swap in `crates/stidcart-core/src/petscii.rs`
- [ ] **T2**: Implement `crates/stidcart-core/src/petscii.rs` to pass T1 — owned results, no shared buffer
- [ ] **T3**: Write failing inline tests for the bank/half offset mapping in `crates/stidcart-core/src/image.rs`
- [ ] **T4**: Implement `crates/stidcart-core/src/image.rs` to pass T3 — 1 MB image, `(bank << 13) | (romh ? 1<<19 : 0)`, erased to `0xFF`
- [ ] **T5**: Implement `crates/stidcart-core/src/firmware.rs` — embed the five blobs, parse the 2-byte load address, overlay at `start_addr - base`
- [ ] **T6**: Write failing test named `AC-6 (none): reserved region layout` in `crates/stidcart-core/tests/reserved.rs`
- [ ] **T7**: Write failing test named `AC-8 (none): entry name encoding and reserved flag bits` in `crates/stidcart-core/tests/reserved.rs` [P]
- [ ] **T8**: Implement `crates/stidcart-core/src/reserved.rs` to pass T6 and T7 — bank-0 map, EF-Name magic and field
- [ ] **T9**: Write failing test named `AC-7 (none): entry geometry round-trips` in `crates/stidcart-core/tests/directory.rs`
- [ ] **T10**: Write failing test named `AC-9 (none): end marker and entry cap` in `crates/stidcart-core/tests/directory.rs` [P]
- [ ] **T11**: Implement `crates/stidcart-core/src/directory.rs` to pass T9 and T10 — entry types, flags byte, 24-byte encoding
- [ ] **T12**: Write failing test named `AC-10 (none): program continues across a bank boundary` in `crates/stidcart-core/tests/placement.rs`
- [ ] **T13**: Implement `crates/stidcart-core/src/place.rs` to pass T12 — weight-ordered, interleaved top-space placement
- [ ] **T14**: Write failing test named `AC-2 (CLI): container header and CHIP packets` in `crates/stidcart-core/tests/container.rs`
- [ ] **T15**: Implement `crates/stidcart-core/src/crt.rs` to pass T14 — 64-byte header, CHIP packets, all-`0xFF` blocks skipped
- [ ] **T16**: Implement `crates/stidcart-core/src/lib.rs` — `CartBuilder`, `Program`, `BuildError`; wire the modules; no file I/O
- [ ] **T17**: Capture reference fixtures into `fixtures/` and record the exact `ndeft` request for each in `fixtures/MANIFEST.md`
- [ ] **T18**: Write failing test named `AC-2 (CLI): byte-identical to the reference image` in `crates/stidcart-core/tests/parity.rs`
- [ ] **T19**: Write failing test named `AC-3 (CLI): parity check runs without the reference tool` in `crates/stidcart-core/tests/parity.rs` [P]
- [ ] **T20**: Write failing test named `AC-1 (CLI): build writes an image and reports success` in `crates/stidcart-cli/tests/cli.rs`
- [ ] **T21**: Write failing test named `AC-5 (CLI): oversized input fails without writing an image` in `crates/stidcart-cli/tests/cli.rs` [P]
- [ ] **T22**: Write failing test named `AC-11 (CLI): empty input list matches the reference behaviour` in `crates/stidcart-cli/tests/cli.rs` [P]
- [ ] **T23**: Implement `crates/stidcart-cli/src/main.rs` to pass T20–T22 — argument parsing, file reading, output writing
- [ ] **T24**: Verify `AC-4 (CLI): produced image boots and the menu lists entry names` by running a built image under the emulator; record the result in the JOURNAL LOG
- [ ] **T-audit**: Run `/lcd:audit write-path-skeleton`; resulting `audit.md` has zero MISSING/BLOCKED rows
- [ ] **T-final**: Run `cargo fmt --all --check && cargo clippy --workspace --all-targets --all-features -- -D warnings && cargo test --workspace`, all green

## Checkpoint validation

After every block of related tasks, the suite must stay green. If T3 passes but T5's implementation breaks T1, stop and fix before moving on.

Two checkpoints matter more than the rest here:

- After **T16**, `CartBuilder::build()` produces bytes for the first time. Run the whole suite before touching fixtures — a parity failure is far cheaper to read when the layer tests underneath it are known green.
- After **T18**, a parity failure localizes by layer: check `container` first (header name is the likely culprit, see D-006), then `reserved`, then `directory`, then `placement`.

## Linkbacks

- `AC-1 (CLI)` → T20, T23
- `AC-2 (CLI)` → T14, T15, T18, T16, T17
- `AC-3 (CLI)` → T17, T19
- `AC-4 (CLI)` → T24
- `AC-5 (CLI)` → T21, T23
- `AC-6 (none)` → T6, T8, T5
- `AC-7 (none)` → T9, T11
- `AC-8 (none)` → T7, T8, T1, T2
- `AC-9 (none)` → T10, T11
- `AC-10 (none)` → T12, T13
- `AC-11 (CLI)` → T22, T23

---

<!--
Quality checks before starting implementation:
  - Every AC × non-`none`-surface row in plan.md has at least one test task above
  - Every new file in plan.md "File structure" has at least one task that creates it
  - Tests appear before their implementation
  - T-audit appears before T-final (the audit gate must pass before suite-final)
  - T-final exists (run the suite, lint, type check)
-->
