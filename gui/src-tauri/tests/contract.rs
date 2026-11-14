//! The wire contract between the Rust side and the frontend.
//!
//! The frontend's `Listing` type and its tests read the JSON files in `gui/src/lib/fixtures/`.
//! These tests assert that the Rust side serializes exactly those files, so renaming a field on
//! either side fails a test, with no code generator involved. Regenerate with
//! `UPDATE_GOLDEN=1 cargo test --test contract` only when the contract is meant to change.

use std::path::{Path, PathBuf};

use stidcart_gui::document::{Document, Edit};
use stidcart_gui::{hex, open};

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

#[test]
fn an_edited_document_serializes_to_the_frontend_golden() {
    let mut doc = Document::open(&fixtures().join("cxb16.crt")).unwrap();
    doc.apply(Edit::SetHidden {
        index: 0,
        hidden: true,
    })
    .unwrap();
    check("document", serde_json::to_value(doc.dto()).unwrap());
}

#[test]
fn the_hex_view_serializes_to_the_frontend_golden() {
    let doc = Document::open(&fixtures().join("two.crt")).unwrap();
    // Bank 0 ROMH from `$BAF0`: the end of EAPI, the cart name, the start of the boot code.
    let lines = hex::lines(&doc, 0x3af, 3).unwrap();
    check(
        "hex",
        serde_json::json!({
            "info": hex::info(&doc, Some(0)).unwrap(),
            "lines": lines,
            // The whole of entry 0 (hello, 102 bytes).
            "entryLines": hex::entry_lines(&doc, 0, 0, 16).unwrap(),
        }),
    );
}
