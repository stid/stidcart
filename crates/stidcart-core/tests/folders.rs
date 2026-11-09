//! folder-browser AC-190: the cart owns an ordered folder list, stored as the metadata's folder
//! table in that order, empty folders included (D-063). Carts from other tools carry no metadata
//! entry, so they load with no folders and save as before.

use std::path::{Path, PathBuf};

use stidcart_core::listing;
use stidcart_core::meta::Field;
use stidcart_core::{read, CartBuilder, Program};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn prg(fill: u8, len: usize) -> Vec<u8> {
    let mut data = vec![0x01, 0x08];
    data.extend(std::iter::repeat_n(fill, len));
    data
}

fn cart() -> CartBuilder {
    CartBuilder::new()
        .with_ef_name("folders")
        .with_program(Program::new("alpha", prg(1, 200)))
        .with_program(Program::new("bravo", prg(2, 200)))
        .with_program(Program::new("charlie", prg(3, 200)))
}

fn reload(b: &CartBuilder) -> CartBuilder {
    read::load(&b.build().unwrap()).unwrap()
}

fn folder(b: &CartBuilder, i: usize) -> String {
    b.entries()[i].meta().get(Field::Folder).to_string()
}

// AC-190: the folder table holds the cart's folders in its order, empty ones included
#[test]
fn ac_190_the_folder_table_is_the_carts_ordered_folder_list() {
    let mut b = cart();
    b.add_folder("Utilities").unwrap();
    b.set_meta(2, Field::Folder, "Games").unwrap();
    b.set_meta(0, Field::Folder, "Demos").unwrap();
    // Not the order entries use them: Utilities is empty, Games comes before Demos.
    assert_eq!(b.folders(), ["Utilities", "Games", "Demos"]);
    b.move_folder("Demos", 0).unwrap();
    assert_eq!(b.folders(), ["Demos", "Utilities", "Games"]);

    let back = reload(&b);
    assert_eq!(
        back.folders(),
        ["Demos", "Utilities", "Games"],
        "order and the empty folder survive"
    );
    assert_eq!(folder(&back, 0), "Demos");
    assert_eq!(folder(&back, 2), "Games");
    assert_eq!(folder(&back, 1), "");

    let l = listing::describe(&b.build().unwrap()).unwrap();
    assert_eq!(l.folders, ["Demos", "Utilities", "Games"]);
}

// AC-190: a folder stays when it empties, and a cart whose only metadata is an empty folder stores it
#[test]
fn ac_190_an_empty_folder_is_stored_and_stays() {
    let mut b = cart();
    b.set_meta(1, Field::Folder, "Games").unwrap();
    b.set_meta(1, Field::Folder, "").unwrap();
    assert_eq!(
        b.folders(),
        ["Games"],
        "clearing the last entry's folder keeps the folder"
    );
    let back = reload(&b);
    assert_eq!(
        back.folders(),
        ["Games"],
        "a cart whose only metadata is an empty folder stores it"
    );

    let mut b = cart();
    b.set_meta(0, Field::Folder, "Games").unwrap();
    b.remove(0);
    assert_eq!(
        reload(&b).folders(),
        ["Games"],
        "removing the last entry keeps the folder"
    );

    let mut b = cart();
    b.add_folder("Games").unwrap();
    b.set_meta(0, Field::Folder, "Games").unwrap();
    b.remove_folder("Games").unwrap();
    assert!(b.folders().is_empty());
    assert_eq!(
        folder(&b, 0),
        "",
        "removing a folder takes it off its entries, which stay"
    );
    assert_eq!(b.entries().len(), 3);

    let mut b = cart();
    assert!(b.add_folder("").is_err());
    assert!(b.add_folder(&"g".repeat(17)).is_err());
    b.add_folder("Games").unwrap();
    assert!(b.add_folder("Games").is_err(), "a folder the cart has");
    assert!(b.remove_folder("Nope").is_err());
    assert!(
        b.move_folder("Games", 1).is_err(),
        "a position outside the order"
    );
    b.set_folders(vec!["Tools".into(), "Games".into()]).unwrap();
    assert_eq!(b.folders(), ["Tools", "Games"]);
    assert!(b.set_folders(vec!["A".into(), "A".into()]).is_err());
}

// AC-190: a cart from another tool loads with no folders and saves as before
#[test]
fn ac_190_legacy_carts_have_no_folders_and_save_as_before() {
    for name in [
        "single.crt",
        "named.crt",
        "c8k.crt",
        "cxb16.crt",
        "cmixed.crt",
    ] {
        let bytes = std::fs::read(fixtures().join(name)).unwrap();
        let b = read::load(&bytes).unwrap();
        assert!(b.folders().is_empty(), "{name}");
        assert!(
            listing::describe(&bytes).unwrap().folders.is_empty(),
            "{name}"
        );
        assert_eq!(b.build().unwrap(), bytes, "{name} saves byte for byte");
    }
}
