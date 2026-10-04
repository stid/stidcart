//! AC-2, AC-4 and AC-11 — loading an image and rebuilding it reproduces the original bytes.
//!
//! This is the read path's oracle, and it is deliberately not byte parity against the reference
//! tool. Parity compares produced bytes; it says nothing about a path whose answer is "refused",
//! and the reference tool is not consulted here at all. What these tests check is that the read
//! path is the write path's inverse, anchored on images the reference tool froze — so the two
//! halves verify each other rather than both being checked against one reading of the format.

use std::path::{Path, PathBuf};
use stidcart_core::{read, CartBuilder};

const FIXTURES: &[&str] = &[
    "single",
    "named",
    "two",
    "spill",
    "c8k",
    "c16k",
    "cult8",
    "cult16",
    "cocean",
    "cocean512",
    "cxb16",
    "cxb16a",
    "cxbult",
    "cmixed",
    "cxbwide",
];

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(fixtures_dir().join(name)).unwrap_or_else(|e| panic!("fixture {name}: {e}"))
}

/// Report the first differing byte rather than dumping two multi-kilobyte vectors.
fn assert_same_bytes(got: &[u8], want: &[u8], case: &str) {
    assert_eq!(
        got.len(),
        want.len(),
        "[{case}] rebuilt length {} != original {}",
        got.len(),
        want.len()
    );
    if got != want {
        let at = got
            .iter()
            .zip(want)
            .position(|(a, b)| a != b)
            .expect("lengths matched so a differing byte must exist");
        let n = got.iter().zip(want).filter(|(a, b)| a != b).count();
        panic!(
            "[{case}] {n} bytes differ; first at {at:#x}: rebuilt {:#04x}, original {:#04x}\n\
             localize by layer before digging here: crt parsing, then reserved, then directory, \
             then program recovery",
            got[at], want[at]
        );
    }
}

/// AC-2 (none): every reference image loads and rebuilds byte-identically.
#[test]
fn ac_2_every_reference_image_loads_and_rebuilds_byte_identically() {
    for case in FIXTURES {
        let original = fixture(&format!("{case}.crt"));
        let cart = read::load(&original).unwrap_or_else(|e| panic!("[{case}] should load: {e}"));
        let rebuilt = cart
            .build()
            .unwrap_or_else(|e| panic!("[{case}] should rebuild: {e}"));
        assert_same_bytes(&rebuilt, &original, case);
    }
}

/// AC-4 (none): a cart this tool builds from no programs loads and rebuilds byte-identically.
#[test]
fn ac_4_a_cart_built_from_no_programs_loads_and_rebuilds_byte_identically() {
    let original = CartBuilder::new().build().expect("an empty cart builds");

    let cart = read::load(&original).expect(
        "the tool must be able to read what the tool writes. An empty request produces a directory \
         whose first slot is the end marker, which is exactly the image the reference \
         implementation refuses — being bug-compatible there would fail this base case on day one",
    );
    assert!(cart.programs().is_empty());

    let rebuilt = cart.build().expect("a loaded empty cart rebuilds");
    assert_same_bytes(&rebuilt, &original, "empty");
}

/// AC-11 (none): an entry continuing past its starting region recovers unbroken.
#[test]
fn ac_11_an_entry_continuing_past_its_starting_region_recovers_unbroken() {
    let cart = read::load(&fixture("spill.crt")).expect("spill.crt should load");

    let big = fixture("big.prg");
    assert!(
        big.len() > 0x2000,
        "the fixture only exercises the continuation if it outgrows one bank half"
    );

    assert_eq!(cart.programs().len(), 1);
    assert_eq!(
        cart.programs()[0].data,
        big,
        "the entry spans ROML into ROMH of the same bank and on into the next; recovery follows \
         the same order the writer used, and the seam must not appear in the bytes"
    );
}
