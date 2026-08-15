# Tasks: Editing an existing cart

> **Deep lane · Phase 3 of 6 (Tasks).** A flat, dependency-ordered checklist. Each task names
> the file it touches. Tests come before the code they test (TDD). The granular STEPS in the
> work-item `JOURNAL.md` mirror this list as the resume view; keep them in sync.

**Slug:** `cart-edit`
**Spec:** [`spec.md`](./spec.md) • **Plan:** [`plan.md`](./plan.md) • **Journal:** [`JOURNAL.md`](./JOURNAL.md)
**Created:** 2026-08-14
**Status:** in_progress <!-- in_progress | completed | abandoned -->

---

## Conventions

- `[P]` suffix = parallelizable with the previous task (different file, no shared state).
- Tests precede implementation tasks they cover.
- One file per task where possible; tasks naming multiple files should be split unless the change is genuinely atomic.

## Tasks

> Each test task that covers a cross-path AC should have a separate task per surface, so the literal `AC-N (SURFACE)` string maps one-to-one to a task. See the ac-convention rule.

### Foundation

- [ ] **T1**: Bump `version` `0.4.0` → `0.5.0` in `Cargo.toml` — `feat/` branch, first implementation
  commit (versioning rule). No test asserts the version; `parity.rs:54` sets its own header name.

### Core: the model carries the hidden flag (AC-29)

- [ ] **T2**: Write failing test named `AC-29 (none): a hidden-carrying cart round-trips through
  editable state` in `crates/stidcart-core/tests/mutate.rs` (new file) — read a fixture, mark an
  entry hidden, save, read again, assert names/order/hidden/data survive and the bytes match a
  re-save of the unmodified read.
- [ ] **T3**: Add `hidden: bool` to `Program` with `Program::with_hidden`, and write
  `program.hidden` where `build_image` currently hardcodes `false`, in
  `crates/stidcart-core/src/lib.rs`.
- [ ] **T4**: Carry the flag out of the directory into each `Program` in `read::load`, and delete the
  `Unsupported::HiddenEntry` variant, its refusal and its message, in
  `crates/stidcart-core/src/read.rs` (D-020). Atomic with `crates/stidcart-core/tests/read.rs:164,439`,
  which match on that variant and stop compiling the moment it goes: both assertions become "a
  hidden-carrying cart loads", the behaviour D-020 chose.

### Core: the entry set becomes mutable (AC-30)

- [ ] **T5**: Write failing test named `AC-30 (none): an edited cart equals a fresh build of the
  resulting entry set` in `crates/stidcart-core/tests/mutate.rs` — mutate a loaded cart through the
  setters, and assert its bytes equal those of a `CartBuilder` built from the same resulting set.
- [ ] **T6**: Add the mutators to `CartBuilder` in `crates/stidcart-core/src/lib.rs` — `push`,
  `remove`, `set_name`, `set_hidden`, `replace_data`, `set_ef_name`, `set_header_name`; document the
  index ones as panicking on an out-of-range index, `Vec`-style.

### CLI: the edit command (AC-17..AC-27)

- [ ] **T7**: Write failing test named `AC-17 (CLI): --add appends entries named after their file
  stems` in `crates/stidcart-cli/tests/edit.rs` (new file).
- [ ] **T8**: Write failing test named `AC-18 (CLI): --remove drops exactly the indexed entry` in
  `crates/stidcart-cli/tests/edit.rs`.
- [ ] **T9**: Write failing test named `AC-19 (CLI): --rename changes one entry's name and no other`
  in `crates/stidcart-cli/tests/edit.rs`.
- [ ] **T10**: Write failing test named `AC-20 (CLI): --hide and --show toggle one entry's flag` in
  `crates/stidcart-cli/tests/edit.rs`.
- [ ] **T11**: Write failing test named `AC-21 (CLI): --replace swaps program data and keeps the name
  and position` in `crates/stidcart-cli/tests/edit.rs`.
- [ ] **T12**: Write failing test named `AC-22 (CLI): -e and --header set the menu and container names
  independently` in `crates/stidcart-cli/tests/edit.rs`.
- [ ] **T13**: Write failing test named `AC-23 (CLI): the edit lands in place by default and -o leaves
  the input byte-identical` in `crates/stidcart-cli/tests/edit.rs`.
- [ ] **T14**: Write failing test named `AC-24 (CLI): a failed edit reports the reason, exits
  non-zero and leaves every input byte-identical` in `crates/stidcart-cli/tests/edit.rs` — cover a
  bad index and a set that does not fit.
- [ ] **T15**: Write failing test named `AC-25 (CLI): a cart that cannot be rebuilt unchanged is
  refused with the reason named` in `crates/stidcart-cli/tests/edit.rs`.
- [ ] **T16**: Write failing test named `AC-26 (CLI): an EasyFS1 cart is refused by edit, naming the
  format` in `crates/stidcart-cli/tests/edit.rs`.
- [ ] **T17**: Write failing test named `AC-27 (CLI): every index in one invocation refers to the
  input listing` in `crates/stidcart-cli/tests/edit.rs` — a remove and a rename in one run, where
  applying them in sequence would hit the wrong entry.
- [ ] **T18**: Add the `edit` command to `crates/stidcart-cli/src/main.rs` — dispatch in `run`, the
  operation grammar (`-o`, `--add`, `--remove`, `--rename`, `--replace`, `--hide`, `--show`, `-e`,
  `--header`), index bounds-checking against the loaded cart, and the `USAGE` text.
- [ ] **T19**: Apply the collected operations in `crates/stidcart-cli/src/main.rs` in the plan's three
  phases — in-place mutations in the order given, removals by descending index, additions appended —
  so index meaning never shifts (AC-27).
- [ ] **T20**: Write the result atomically in `crates/stidcart-cli/src/main.rs` — full bytes first,
  then a temporary file in the destination's directory renamed over the destination, for both the
  in-place default and `-o` (AC-23, AC-24).

### CLI: build learns the marker (AC-28)

- [ ] **T21**: Write failing test named `AC-28 (CLI): build --hide marks that input hidden, and a
  build marking nothing is byte-unchanged` in `crates/stidcart-cli/tests/cli.rs`.
- [ ] **T22**: Add `--hide INPUT.prg` to `build` in `crates/stidcart-cli/src/main.rs` — matched
  against the given input paths, erroring when it matches none — and extend `USAGE`.

### Closing

- [ ] **T23** [P]: Update the State section of `README.md`, which currently says the tool does not
  edit a cart.
- [ ] **T-audit**: Run `/lcd:audit cart-edit`; resulting `audit.md` has zero MISSING/BLOCKED rows.
- [ ] **T-final**: Run the project gate — `cargo fmt --all --check && cargo clippy --workspace
  --all-targets --all-features -- -D warnings && cargo test --workspace` — all green.

## Checkpoint validation

After every block of related tasks, the suite must stay green. Two specific hazards in this
work-item:

- T4 changes the read path's refusal set. `crates/stidcart-core/tests/read.rs:164,439` match on
  `Unsupported::HiddenEntry` and will not compile once it is gone — flip both to assert the cart now
  loads. That is a deliberate behaviour change (D-020), not a regression to work around.
- T3 touches `build_image`, which the parity fixtures gate. `cargo test --workspace --test parity`
  must stay green at every step: nothing existing marks anything hidden, so the bytes must not move.

## Linkbacks

- `AC-17 (CLI)` → T7, T18, T19, T20
- `AC-18 (CLI)` → T8, T18, T19, T20
- `AC-19 (CLI)` → T9, T18, T19, T20
- `AC-20 (CLI)` → T10, T18, T19, T20 (needs T3, T6)
- `AC-21 (CLI)` → T11, T18, T19, T20
- `AC-22 (CLI)` → T12, T18, T20
- `AC-23 (CLI)` → T13, T18, T20
- `AC-24 (CLI)` → T14, T18, T20
- `AC-25 (CLI)` → T15, T18
- `AC-26 (CLI)` → T16, T18
- `AC-27 (CLI)` → T17, T18, T19
- `AC-28 (CLI)` → T21, T22
- `AC-29` → T2, T3, T4
- `AC-30` → T5, T6

---

<!--
Quality checks before starting implementation:
  - Every AC × non-`none`-surface row in plan.md has at least one test task above ✅ (T7..T17, T21)
  - Every new file in plan.md "File structure" has a creating task ✅ (T2 → mutate.rs, T7 → edit.rs)
  - Every modified file has a modifying task ✅ (lib.rs T3/T6, read.rs + tests/read.rs T4,
    main.rs T18-T20/T22, cli.rs T21, Cargo.toml T1, README.md T23)
  - Tests appear before their implementation ✅
  - T-audit appears before T-final ✅
-->
