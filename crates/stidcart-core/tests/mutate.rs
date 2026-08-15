//! AC-29 and AC-30 of the **cart-edit** work-item — the mutating half of the builder API.
//!
//! Both criteria are `surfaces: none`: they are library behaviour with no command-line surface of
//! their own, so the cross-path audit skips them and these tests are the coverage rather than the
//! audit being it. The command that drives them is covered in `stidcart-cli/tests/edit.rs`.

use std::path::{Path, PathBuf};

use stidcart_core::{read, CartBuilder, Program};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(fixtures_dir().join(name)).unwrap_or_else(|e| panic!("fixture {name}: {e}"))
}

/// AC-29: a cart carrying hidden entries reads into editable state and saves back unchanged.
///
/// Loading such a cart was refused outright until D-020, because the model had nowhere to keep the
/// flag and a save would have silently dropped it. Widening the model is what retires the refusal,
/// so the flag surviving the round trip *is* the evidence that retiring it was safe.
#[test]
fn ac_29_a_hidden_carrying_cart_round_trips_through_editable_state() {
    let original = fixture("two.crt");

    let mut marked = read::load(&original).expect("two.crt loads");
    marked.set_hidden(1, true);
    let with_hidden = marked.build().expect("the marked cart builds");

    let reread = read::load(&with_hidden)
        .expect("a cart holding a hidden entry is no longer refused for rebuilding (D-020)");

    let names: Vec<&str> = reread.programs().iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["hello", "ramp"], "names and order survive the trip");

    let flags: Vec<bool> = reread.programs().iter().map(|p| p.hidden).collect();
    assert_eq!(
        flags,
        [false, true],
        "the hidden flag comes back on the entry it was set on, and on no other"
    );

    assert_eq!(
        reread.programs()[0].data,
        fixture("hello.prg"),
        "program data is unchanged by the round trip"
    );
    assert_eq!(reread.programs()[1].data, fixture("ramp.prg"));

    assert_eq!(
        reread.build().expect("the reread cart builds"),
        with_hidden,
        "reading a cart and saving it back with nothing applied changes no byte — this cart's \
         layout is what a rebuild places, which is the condition AC-29 states"
    );
}

/// AC-30: an edited cart is byte-for-byte a fresh build of the entry set the edits produced.
///
/// The expectation is built independently rather than from the edited builder's own program list,
/// so the assertion is about *placement* — the same set laid out the same way — and not merely
/// about the same values reaching the same code twice.
#[test]
fn ac_30_an_edited_cart_equals_a_fresh_build_of_the_resulting_entry_set() {
    let original = fixture("two.crt");
    let mut cart = read::load(&original).expect("two.crt loads");
    let header_name = cart.header_name().to_string();
    let ef_name = cart.ef_name().to_string();

    cart.remove(0);
    cart.set_name(0, "renamed");
    cart.push(Program::new("added", fixture("hello.prg")));
    let edited = cart.build().expect("the edited cart builds");

    let fresh = CartBuilder::new()
        .with_header_name(&header_name)
        .with_ef_name(&ef_name)
        .with_program(Program::new("renamed", fixture("ramp.prg")))
        .with_program(Program::new("added", fixture("hello.prg")))
        .build()
        .expect("the equivalent fresh cart builds");

    assert_eq!(
        edited, fresh,
        "saving an edit runs the same placement a fresh build runs; there is no second layout mode"
    );

    assert_ne!(
        edited, original,
        "and the input's own layout is not preserved — dropping the first entry re-places what is \
         left, which is the cost D-019 accepted"
    );
}
