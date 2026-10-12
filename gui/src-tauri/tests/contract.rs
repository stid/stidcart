//! The wire contract between the Rust side and the frontend.
//!
//! The frontend's `Listing` type and its tests read the JSON files in `gui/src/lib/fixtures/`.
//! These tests assert that the Rust side serializes exactly those files, so renaming a field on
//! either side fails a test, with no code generator involved. Regenerate with
//! `UPDATE_GOLDEN=1 cargo test --test contract` only when the contract is meant to change.

use std::path::{Path, PathBuf};

use stidcart_gui::open;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn golden(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../src/lib/fixtures/{name}.json"))
}

fn check(name: &str, actual: serde_json::Value) {
    let path = golden(name);
    let text = serde_json::to_string_pretty(&actual).unwrap() + "\n";
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::write(&path, text).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
    assert_eq!(
        text, expected,
        "{name}.json no longer matches what the Rust side sends"
    );
}

#[test]
fn a_listing_serializes_to_the_frontend_golden() {
    let dto = open(&fixtures().join("two.crt")).unwrap();
    check("two", serde_json::to_value(&dto).unwrap());
}

#[test]
fn a_refusal_serializes_to_the_frontend_golden() {
    let err = open(&fixtures().join("in_8k.crt")).unwrap_err();
    check("refused", serde_json::Value::from(err));
}
