//! menu-meta AC-153..AC-155: entry metadata stored in one hidden `STIDCART.META` entry (D-054,
//! D-055, and the layout in `docs/format/`, "stidcart extension: entry metadata").
//!
//! There is no `ndeft` oracle for any of this, so the evidence is round trips through the one
//! build path and the one directory walk, plus the VICE boots in `stidcart-cli/tests/meta_boot.rs`.

use std::path::{Path, PathBuf};

use stidcart_core::directory::{self, Slot, ENTRY_LEN};
use stidcart_core::listing;
use stidcart_core::menu::Menu;
use stidcart_core::meta::{Field, Meta, MetaError};
use stidcart_core::reserved::DIRECTORY_OFFSET;
use stidcart_core::{crt, read, BuildError, CartBuilder, FlashImage, Program};

const MAGIC: &[u8] = b"STIDMETA";

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

/// A small BASIC-area program whose payload is `len` bytes of `fill`.
fn prg(fill: u8, len: usize) -> Vec<u8> {
    let mut data = vec![0x01, 0x08];
    data.extend(std::iter::repeat_n(fill, len));
    data
}

fn three() -> CartBuilder {
    CartBuilder::new()
        .with_ef_name("meta test")
        .with_program(Program::new("first", prg(1, 300)))
        .with_program(Program::new("second", prg(2, 5000)).with_hidden(true))
        .with_program(Program::new("third", prg(3, 20000)))
}

fn meta(pairs: &[(Field, &str)]) -> Meta {
    let mut m = Meta::default();
    for (f, v) in pairs {
        m.set(*f, v).unwrap();
    }
    m
}

fn set_all(b: &mut CartBuilder, index: usize, pairs: &[(Field, &str)]) {
    for (f, v) in pairs {
        b.set_meta(index, *f, v).unwrap();
    }
}

fn reload(b: &CartBuilder) -> CartBuilder {
    read::load(&b.build().unwrap()).unwrap()
}

fn metas(b: &CartBuilder) -> Vec<Meta> {
    b.entries().iter().map(|e| e.meta().clone()).collect()
}

fn names(b: &CartBuilder) -> Vec<String> {
    b.entries().iter().map(|e| e.name().to_string()).collect()
}

const FIRST: &[(Field, &str)] = &[
    (Field::Title, "Impossible Mission II"),
    (Field::Year, "1988"),
    (Field::Author, "Epyx"),
    (Field::Note, "Stop the madman"),
    (Field::Group, "Games"),
];
const THIRD: &[(Field, &str)] = &[(Field::Title, "Turbo Assembler"), (Field::Group, "Tools")];

/// The cart `three()` with metadata on entries 0 and 2.
fn annotated() -> CartBuilder {
    let mut b = three();
    set_all(&mut b, 0, FIRST);
    set_all(&mut b, 2, THIRD);
    b
}

/// The directory slots of an image as stored, META included.
fn slots(img: &FlashImage) -> Vec<Slot> {
    let mut out = Vec::new();
    for slot in 0..256 {
        let at = DIRECTORY_OFFSET + slot * ENTRY_LEN;
        let raw: [u8; ENTRY_LEN] = img.as_bytes()[at..at + ENTRY_LEN].try_into().unwrap();
        match directory::decode(&raw) {
            Slot::End => break,
            s => out.push(s),
        }
    }
    out
}

fn magic_at(bytes: &[u8]) -> usize {
    bytes
        .windows(MAGIC.len())
        .position(|w| w == MAGIC)
        .expect("the image holds the metadata magic")
}

// AC-153: every entry keeps its own metadata through a load and save, a removal, a re-order, a
// rename and a menu switch and back
#[test]
fn ac_153_metadata_follows_its_entry_through_every_edit() {
    let b = annotated();
    let want = vec![meta(FIRST), Meta::default(), meta(THIRD)];

    let loaded = reload(&b);
    assert_eq!(names(&loaded), ["first", "second", "third"]);
    assert_eq!(metas(&loaded), want, "load and save");
    assert!(
        loaded.entries()[1].hidden(),
        "the hidden flag is carried too"
    );

    let mut removed = loaded.clone();
    removed.remove(1);
    let removed = reload(&removed);
    assert_eq!(names(&removed), ["first", "third"]);
    assert_eq!(metas(&removed), vec![meta(FIRST), meta(THIRD)], "a removal");

    // A re-order: the same entries pushed in another order.
    let mut reordered = CartBuilder::new();
    for e in loaded.entries().iter().rev() {
        reordered.push(e.clone());
    }
    let reordered = reload(&reordered);
    assert_eq!(names(&reordered), ["third", "second", "first"]);
    assert_eq!(
        metas(&reordered),
        vec![meta(THIRD), Meta::default(), meta(FIRST)],
        "a re-order"
    );

    let mut renamed = loaded.clone();
    renamed.set_name(0, "imp mission 2");
    let renamed = reload(&renamed);
    assert_eq!(renamed.entries()[0].name(), "imp mission 2");
    assert_eq!(metas(&renamed), want, "a rename");

    let mut switched = loaded.clone();
    switched.set_menu(Menu::Stidcart);
    let mut back = reload(&switched);
    assert_eq!(back.menu(), Menu::Stidcart);
    assert_eq!(metas(&back), want, "switched to our menu");
    back.set_menu(Menu::EasyLoader);
    let back = reload(&back);
    assert_eq!(metas(&back), want, "and back to EasyLoader");

    // Replacing an entry's data keeps its metadata, as it keeps its name and hidden flag.
    let mut replaced = loaded.clone();
    replaced.replace_data(2, prg(9, 100));
    assert_eq!(metas(&reload(&replaced)), want, "a replace");
}

// AC-153: a cart where no entry carries metadata is written exactly as before
#[test]
fn ac_153_without_metadata_the_bytes_are_unchanged() {
    for name in [
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
    ] {
        let bytes = std::fs::read(fixtures().join(format!("{name}.crt"))).unwrap();
        let loaded = read::load(&bytes).unwrap();
        assert!(metas(&loaded).iter().all(Meta::is_empty), "{name}");
        assert!(
            loaded.build().unwrap() == bytes,
            "{name}: a load and save is byte-identical"
        );
    }

    // Set, then cleared again: no trace of it.
    let plain = three().build().unwrap();
    let mut b = three();
    set_all(&mut b, 1, FIRST);
    for (f, _) in FIRST {
        b.set_meta(1, *f, "").unwrap();
    }
    assert!(
        b.build().unwrap() == plain,
        "cleared metadata writes no entry"
    );

    let img = three().build_image().unwrap();
    assert_eq!(slots(&img).len(), 3, "no extra slot without metadata");
}

// AC-154: records whose entry another tool renamed or re-ordered are dropped and counted
#[test]
fn ac_154_records_that_no_longer_match_are_dropped_and_counted() {
    let mut img = annotated().build_image().unwrap();
    // Swap the directory slots of entries 0 and 2, as a tool unaware of the metadata might.
    let (a, c) = (DIRECTORY_OFFSET, DIRECTORY_OFFSET + 2 * ENTRY_LEN);
    let slot0 = img.as_bytes()[a..a + ENTRY_LEN].to_vec();
    let slot2 = img.as_bytes()[c..c + ENTRY_LEN].to_vec();
    img.write_at(a, &slot2);
    img.write_at(c, &slot0);
    let bytes = crt::write(&img, "another tool");

    let info = read::inspect(&bytes).unwrap();
    assert_eq!(info.entries.len(), 3, "META is not an entry");
    assert!(
        info.metas.iter().all(Meta::is_empty),
        "neither record matches its slot"
    );
    assert_eq!(info.meta_dropped, 2);

    let l = listing::describe(&bytes).unwrap();
    assert_eq!(l.meta_dropped, 2, "the listing says how many were dropped");
    assert!(l
        .rows
        .iter()
        .all(|r| r.title.is_empty() && r.group.is_empty()));

    let loaded = read::load(&bytes).unwrap();
    assert!(metas(&loaded).iter().all(Meta::is_empty));
    // Saving drops them for good: nothing is left to store.
    let saved = loaded.build().unwrap();
    assert_eq!(read::inspect(&saved).unwrap().meta_dropped, 0);

    // A rename by another tool drops only that entry's record.
    let mut img = annotated().build_image().unwrap();
    img.write_at(DIRECTORY_OFFSET, b"RENAMED\0\0\0\0\0\0\0\0\0");
    let bytes = crt::write(&img, "another tool");
    let info = read::inspect(&bytes).unwrap();
    assert_eq!(info.meta_dropped, 1);
    assert!(info.metas[0].is_empty());
    assert_eq!(
        info.metas[2],
        meta(THIRD),
        "the matching record still attaches"
    );
}

/// `annotated()` with its metadata layout version byte set to `version`.
fn other_version(version: u8) -> Vec<u8> {
    let mut img = annotated().build_image().unwrap();
    let at = magic_at(img.as_bytes());
    img.write_at(at + MAGIC.len(), &[version]);
    crt::write(&img, "a later stidcart")
}

// AC-154: a layout version this tool doesn't know is described, kept, and never rewritten
#[test]
fn ac_154_an_unknown_layout_version_is_kept_and_its_rewrites_refused() {
    let bytes = other_version(9);

    let info = read::inspect(&bytes).expect("described, not refused");
    assert_eq!(info.entries.len(), 3);
    assert!(info.metas.iter().all(Meta::is_empty));
    let kept = info
        .meta_kept
        .as_ref()
        .expect("the unknown layout is reported");
    assert!(kept.reason.contains('9'), "{}", kept.reason);
    let l = listing::describe(&bytes).unwrap();
    assert!(l.meta_kept.as_deref().is_some_and(|k| k.contains('9')));

    // Kept: a load and save writes it back byte for byte.
    let loaded = read::load(&bytes).unwrap();
    assert!(loaded.build().unwrap() == bytes, "kept verbatim");

    // Edits that leave every entry's index and name alone still save, the entry kept as stored.
    let mut hidden = loaded.clone();
    hidden.set_hidden(0, true);
    hidden.push(Program::new("added", prg(4, 10)));
    let out = hidden.build_image().unwrap();
    let at = magic_at(out.as_bytes());
    assert_eq!(out.as_bytes()[at + MAGIC.len()], 9);

    // Edits that would need it rewritten are refused, naming why.
    let mut set = loaded.clone();
    match set.set_meta(0, Field::Title, "x") {
        Err(e @ MetaError::Kept { .. }) => assert!(e.to_string().contains('9'), "{e}"),
        other => panic!("setting metadata over a kept layout: {other:?}"),
    }
    let mut renamed = loaded.clone();
    renamed.set_name(1, "other");
    assert!(matches!(
        renamed.build(),
        Err(BuildError::MetadataKept { .. })
    ));
    let mut removed = loaded.clone();
    removed.remove(0);
    let err = removed.build().unwrap_err();
    assert!(matches!(err, BuildError::MetadataKept { .. }), "{err}");
    assert!(err.to_string().contains('9'), "{err}");
}

// AC-155: the metadata takes one hidden directory slot and is never an entry
#[test]
fn ac_155_metadata_takes_one_hidden_slot_and_is_never_listed() {
    let img = annotated().build_image().unwrap();
    let stored = slots(&img);
    assert_eq!(stored.len(), 4, "three entries and the metadata");
    match &stored[3] {
        Slot::Program(e) => {
            assert!(e.hidden, "hidden from both menus");
            assert_eq!(e.name, "stidcart.meta");
        }
        other => panic!("the last slot is a program: {other:?}"),
    }
    let raw = &img.as_bytes()[DIRECTORY_OFFSET + 3 * ENTRY_LEN..][..16];
    assert_eq!(raw, b"STIDCART.META\0\0\0", "the name as stored");

    let bytes = crt::write(&img, "t");
    assert_eq!(read::inspect(&bytes).unwrap().entries.len(), 3);
    assert_eq!(read::load(&bytes).unwrap().entries().len(), 3);
    let l = listing::describe(&bytes).unwrap();
    assert_eq!(l.rows.len(), 3);
    assert!(l.rows.iter().all(|r| r.name != "stidcart.meta"));
    assert_eq!(l.rows[0].title, "Impossible Mission II");
}

// AC-155: a set needing more slots than the directory has is refused naming the shortfall
#[test]
fn ac_155_the_metadata_slot_counts_against_the_directory() {
    let mut full = CartBuilder::new();
    for i in 0..255 {
        full.push(Program::new(&format!("p{i}"), prg(i as u8, 1)));
    }
    assert!(
        full.build().is_ok(),
        "255 entries without metadata still fit"
    );
    full.set_meta(0, Field::Title, "one too many").unwrap();
    let err = full.build().unwrap_err();
    assert!(matches!(err, BuildError::TooManyEntries { .. }), "{err:?}");
    assert!(err.to_string().contains("metadata"), "{err}");
    assert!(err.to_string().contains("256"), "{err}");

    full.remove(254);
    assert!(
        full.build().is_ok(),
        "254 entries and the metadata fill it exactly"
    );
}

// AC-155: our menu's I/O-area rule never refuses the metadata itself
#[test]
fn ac_155_our_menu_accepts_the_metadata_entry() {
    let mut b = annotated();
    b.set_menu(Menu::Stidcart);
    // Enough records to make the entry over 8 KB, so its load range is wide too.
    for i in 0..60 {
        b.push(Program::new(&format!("x{i}"), prg(i as u8, 10)));
        let at = b.entries().len() - 1;
        set_all(
            &mut b,
            at,
            &[
                (Field::Title, &"T".repeat(48)),
                (Field::Note, &"N".repeat(48)),
            ],
        );
    }
    let loaded = reload(&b);
    assert_eq!(loaded.menu(), Menu::Stidcart);
    assert_eq!(
        loaded.entries()[62].meta().get(Field::Title),
        "T".repeat(48)
    );
}

// AC-155 (D-055): the metadata a C64 copies to $4000-$7FFF can't be more than 16 KB
#[test]
fn ac_155_metadata_over_16_kb_is_refused_naming_the_sizes() {
    let mut b = CartBuilder::new();
    for i in 0..120 {
        b.push(Program::new(&format!("p{i}"), prg(1, 1)));
        set_all(
            &mut b,
            i,
            &[
                (Field::Title, &"T".repeat(48)),
                (Field::Year, "1984"),
                (Field::Author, &"A".repeat(24)),
                (Field::Note, &"N".repeat(48)),
                (Field::Group, "Games"),
            ],
        );
    }
    let err = b.build().unwrap_err();
    assert!(
        matches!(err, BuildError::MetadataTooLarge { .. }),
        "{err:?}"
    );
    assert!(err.to_string().contains("16384"), "{err}");
}

// D-027 with metadata: the free figure counts the metadata entry, so a program of exactly that
// size can be added and one byte more cannot.
#[test]
fn the_free_figure_counts_the_metadata_entry() {
    let b = annotated();
    let free = b.free_space();
    assert!(free < three().free_space(), "the metadata takes room");
    for (size, fits) in [(free, true), (free + 1, false)] {
        let mut more = b.clone();
        more.push(Program::new("probe", vec![0; size]));
        assert_eq!(more.build().is_ok(), fits, "a program of {size} bytes");
    }
}
