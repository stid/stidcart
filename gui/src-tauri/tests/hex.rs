//! What the hex window is sent (hex-viewer): the open cart's bytes in rows, and who owns them,
//! always for the step the editor is on.

use std::path::{Path, PathBuf};

use stidcart_core::hexmap::{self, LINES, LINE_BYTES};
use stidcart_core::image::Half;
use stidcart_gui::document::{Document, Edit};
use stidcart_gui::hex;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(name)
}

/// AC-200 (RENDER): the window is told the size of the view and gets rows with bank, half, address
/// and the image's bytes.
#[test]
fn rows_carry_bank_half_address_and_bytes() {
    let doc = Document::open(&fixture("two.crt")).unwrap();
    let info = hex::info(&doc, None).unwrap();
    assert_eq!(info.file, "two.crt");
    assert_eq!((info.lines, info.line_bytes), (LINES, LINE_BYTES));

    let first_romh = hexmap::position(0, Half::RomH, 0) / LINE_BYTES;
    let rows = hex::lines(&doc, first_romh, 2).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(
        (rows[0].bank, rows[0].half.as_str(), rows[0].addr),
        (0, "H", 0xa000)
    );
    assert_eq!(rows[1].addr, 0xa010);
    let core = hexmap::lines(doc.bytes(), first_romh, 2).unwrap();
    assert_eq!(rows[0].bytes, core[0].bytes);

    let roml = hex::lines(&doc, 0, 1).unwrap();
    assert_eq!(
        (roml[0].bank, roml[0].half.as_str(), roml[0].addr),
        (0, "L", 0x8000)
    );
}

/// AC-201 (RENDER): each row carries its text column.
#[test]
fn rows_carry_the_text_column() {
    let doc = Document::open(&fixture("two.crt")).unwrap();
    let name_row = hexmap::position(0, Half::RomH, 0x1b00) / LINE_BYTES;
    let rows = hex::lines(&doc, name_row, 1).unwrap();
    assert_eq!(rows[0].text, "EF-Name:two entr");
}

/// AC-204 (RENDER): one request is capped, so a window can never ask for the whole image at once.
#[test]
fn a_request_is_capped() {
    let doc = Document::open(&fixture("two.crt")).unwrap();
    assert_eq!(hex::lines(&doc, 0, LINES).unwrap().len(), hex::MAX_LINES);
    assert!(hex::lines(&doc, LINES, 10).unwrap().is_empty());
}

/// AC-202 (RENDER): every owner is named in words, and an entry by its number, name and kind with
/// the family that picks its colour.
#[test]
fn owners_are_named() {
    let doc = Document::open(&fixture("cxb16.crt")).unwrap();
    let info = hex::info(&doc, None).unwrap();
    let label = |kind: &str| {
        info.regions
            .iter()
            .find(|r| r.kind == kind)
            .unwrap_or_else(|| panic!("no {kind} region"))
            .label
            .clone()
    };
    assert_eq!(label("loader"), "Menu loader");
    assert_eq!(label("directory"), "Directory");
    assert_eq!(label("eapi"), "EAPI");
    assert_eq!(label("name"), "Cart name");
    assert_eq!(label("boot"), "Boot code");

    let xb = info
        .regions
        .iter()
        .find(|r| r.entry == Some(1))
        .expect("the xbank's region");
    assert_eq!(xb.kind, "entry");
    assert_eq!(xb.label, "1 · in_xb16 (XB16)");
    assert_eq!(xb.family.as_deref(), Some("xb"));
    assert_eq!(
        (xb.start, xb.len),
        (hexmap::position(1, Half::RomL, 0), 0x10000)
    );
    assert!(info
        .regions
        .iter()
        .all(|r| (r.kind == "entry") == r.entry.is_some()));
}

/// AC-202 (RENDER): the metadata entry is named as metadata.
#[test]
fn the_metadata_entry_is_named() {
    let mut doc = Document::open(&fixture("two.crt")).unwrap();
    doc.apply(Edit::SetMeta {
        index: 0,
        field: "title".into(),
        value: "a title".into(),
    })
    .unwrap();
    let info = hex::info(&doc, None).unwrap();
    let meta = info.regions.iter().find(|r| r.kind == "meta").unwrap();
    assert_eq!(meta.label, "Metadata");
    assert_eq!(meta.entry, None);
}

/// AC-203 (RENDER): the entry selected in the listing reaches the window with the rest.
#[test]
fn the_selection_travels_with_the_info() {
    let doc = Document::open(&fixture("two.crt")).unwrap();
    assert_eq!(hex::info(&doc, Some(1)).unwrap().selected, Some(1));
    assert_eq!(hex::info(&doc, None).unwrap().selected, None);
}

/// AC-204 (RENDER): rows and owners are those of the step the editor is on, so an edit shows at
/// once and undo takes it back.
#[test]
fn the_view_follows_edits_and_undo() {
    let mut doc = Document::open(&fixture("two.crt")).unwrap();
    let name_row = hexmap::position(0, Half::RomH, 0x1b00) / LINE_BYTES;
    let before = hex::info(&doc, None).unwrap();

    doc.apply(Edit::SetCartName {
        name: "renamed".into(),
    })
    .unwrap();
    assert_eq!(
        hex::lines(&doc, name_row, 1).unwrap()[0].text,
        "EF-Name:renamed."
    );
    doc.apply(Edit::Remove { index: 1 }).unwrap();
    let after = hex::info(&doc, None).unwrap();
    assert_eq!(
        after.regions.iter().filter(|r| r.kind == "entry").count(),
        1
    );

    doc.undo();
    doc.undo();
    assert_eq!(
        hex::lines(&doc, name_row, 1).unwrap()[0].text,
        "EF-Name:two entr"
    );
    assert_eq!(
        serde_json::to_value(hex::info(&doc, None).unwrap()).unwrap(),
        serde_json::to_value(before).unwrap()
    );
}
