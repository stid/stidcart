//! The flash image as a hex viewer shows it (hex-viewer): rows of 16 bytes in bank order, each
//! with the address the C64 sees, and the byte ranges every part of the cart owns.

use std::path::{Path, PathBuf};

use stidcart_core::hexmap::{self, Owner, Region, LINES, LINE_BYTES};
use stidcart_core::image::{FlashImage, Half, BANK_SIZE, IMAGE_SIZE};
use stidcart_core::meta::Field;
use stidcart_core::{crt, read, CartBuilder, Program};

fn fixture(name: &str) -> Vec<u8> {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures")
        .join(name);
    std::fs::read(path).unwrap()
}

fn owned(regions: &[Region], owner: Owner) -> Vec<(usize, usize)> {
    regions
        .iter()
        .filter(|r| r.owner == owner)
        .map(|r| (r.start, r.len))
        .collect()
}

/// AC-200 (RENDER): rows run bank by bank, ROML then ROMH, 16 bytes each, and cover the image once.
#[test]
fn rows_cover_the_image_bank_by_bank() {
    let cart = fixture("two.crt");
    let image = crt::parse(&cart).unwrap().image;
    let lines = hexmap::lines(&cart, 0, LINES).unwrap();
    assert_eq!(lines.len(), IMAGE_SIZE / LINE_BYTES);

    let per_half = BANK_SIZE / LINE_BYTES;
    for (i, line) in lines.iter().enumerate() {
        let (section, row) = (i / per_half, i % per_half);
        let half = if section % 2 == 0 {
            Half::RomL
        } else {
            Half::RomH
        };
        assert_eq!((line.bank as usize, line.half), (section / 2, half));
        let at = FlashImage::offset(section / 2, half) + row * LINE_BYTES;
        assert_eq!(line.bytes, image.as_bytes()[at..at + LINE_BYTES], "row {i}");
    }
}

/// AC-200 (RENDER): a row's address is the one the C64 sees: `$8000` up in ROML, `$A000` up in ROMH.
#[test]
fn a_row_is_addressed_as_the_c64_sees_it() {
    let cart = fixture("two.crt");
    let per_half = BANK_SIZE / LINE_BYTES;
    let at = |row: usize| {
        let line = hexmap::lines(&cart, row, 1).unwrap().remove(0);
        (line.bank, line.half, line.addr)
    };
    assert_eq!(at(0), (0, Half::RomL, 0x8000));
    assert_eq!(at(1), (0, Half::RomL, 0x8010));
    assert_eq!(at(per_half - 1), (0, Half::RomL, 0x9ff0));
    assert_eq!(at(per_half), (0, Half::RomH, 0xa000));
    assert_eq!(at(2 * per_half), (1, Half::RomL, 0x8000));
    assert_eq!(at(LINES - 1), (63, Half::RomH, 0xbff0));
}

#[test]
fn a_request_past_the_end_is_cut_to_the_image() {
    let cart = fixture("two.crt");
    assert_eq!(hexmap::lines(&cart, LINES - 2, 10).unwrap().len(), 2);
    assert!(hexmap::lines(&cart, LINES + 5, 10).unwrap().is_empty());
}

/// AC-201 (RENDER): the text column reads each byte through the names' PETSCII mapping.
#[test]
fn the_text_column_reads_petscii_with_dots_for_the_unprintable() {
    let cart = fixture("two.crt");
    let name_row = hexmap::position(0, Half::RomH, 0x1b00) / LINE_BYTES;
    let line = hexmap::lines(&cart, name_row, 1).unwrap().remove(0);
    assert_eq!(&line.bytes[..8], b"ef-nAME:");
    assert_eq!(line.text, "EF-Name:two entr");

    // Erased flash, far above anything this cart holds.
    let erased = hexmap::lines(&cart, LINES - 1, 1).unwrap().remove(0);
    assert_eq!(erased.bytes, [0xff; LINE_BYTES]);
    assert_eq!(erased.text, "................");
    assert!(hexmap::lines(&cart, 0, LINES)
        .unwrap()
        .iter()
        .all(|l| l.text.chars().count() == LINE_BYTES && l.text.chars().all(|c| !c.is_control())));
}

/// AC-202 (RENDER): the reserved areas of bank 0 are owned as the format's reserved map lays them out.
#[test]
fn the_reserved_areas_follow_the_bank_0_map() {
    let regions = hexmap::regions(&fixture("two.crt")).unwrap();
    let romh = hexmap::position(0, Half::RomH, 0);
    assert_eq!(owned(&regions, Owner::Loader), [(0, 0x2000)]);
    assert_eq!(owned(&regions, Owner::Directory), [(romh, 0x1800)]);
    assert_eq!(owned(&regions, Owner::Eapi), [(romh + 0x1800, 0x300)]);
    assert_eq!(owned(&regions, Owner::Name), [(romh + 0x1b00, 0x18)]);
    assert_eq!(owned(&regions, Owner::Boot), [(romh + 0x1b18, 0x4e8)]);
}

/// AC-202 (RENDER): a program owns its bytes from the position the directory records.
#[test]
fn a_program_owns_its_bytes_where_the_directory_puts_them() {
    let cart = fixture("two.crt");
    let info = read::inspect(&cart).unwrap();
    let regions = hexmap::regions(&cart).unwrap();
    let bank1 = hexmap::position(1, Half::RomL, 0);
    assert_eq!(
        owned(&regions, Owner::Entry(0)),
        [(bank1 + 0xbba, info.entries[0].size as usize)]
    );
    assert_eq!(
        owned(&regions, Owner::Entry(1)),
        [(bank1, info.entries[1].size as usize)]
    );
}

/// AC-202 (RENDER): an interleaved program crossing halves is one run in bank order.
#[test]
fn an_interleaved_program_is_one_run() {
    let cart = fixture("spill.crt");
    let size = read::inspect(&cart).unwrap().entries[0].size as usize;
    assert!(size > 2 * BANK_SIZE);
    let regions = hexmap::regions(&cart).unwrap();
    assert_eq!(
        owned(&regions, Owner::Entry(0)),
        [(hexmap::position(1, Half::RomL, 0), size)]
    );
}

/// AC-202 (RENDER): a cartridge owns its bank runs, joined where both halves of a bank are its own.
#[test]
fn a_cartridge_owns_its_bank_runs() {
    // `in_xb16`: L4@0,H4@0 from bank 1, so eight halves in a row.
    let regions = hexmap::regions(&fixture("cxb16.crt")).unwrap();
    assert_eq!(
        owned(&regions, Owner::Entry(1)),
        [(hexmap::position(1, Half::RomL, 0), 8 * BANK_SIZE)]
    );
}

/// AC-202 (RENDER): in Ocean mode the loader sits in bank 1 ROMH, and a half-confined program and
/// the Ocean cart's split runs are owned half by half.
#[test]
fn ocean_mode_moves_the_loader_and_splits_the_runs() {
    let regions = hexmap::regions(&fixture("cocean.crt")).unwrap();
    assert_eq!(
        owned(&regions, Owner::Loader),
        [(hexmap::position(1, Half::RomH, 0), BANK_SIZE)]
    );

    // `in_ocean`: L16@0,H16@16 from bank 0.
    let ocean = owned(&regions, Owner::Entry(1));
    let mut expected: Vec<(usize, usize)> = (0..16)
        .map(|b| (hexmap::position(b, Half::RomL, 0), BANK_SIZE))
        .chain((16..32).map(|b| (hexmap::position(b, Half::RomH, 0), BANK_SIZE)))
        .collect();
    expected.sort_unstable();
    assert_eq!(ocean, expected);

    // `big`: PRG-L from bank $10, 19.5 KB, so two whole ROML halves and part of a third.
    let big = owned(&regions, Owner::Entry(0));
    assert_eq!(big.len(), 3);
    assert_eq!(
        big.iter().map(|r| r.0).collect::<Vec<_>>(),
        (16..19)
            .map(|b| hexmap::position(b, Half::RomL, 0))
            .collect::<Vec<_>>()
    );
    assert_eq!(big[0].1, BANK_SIZE);
    assert_eq!(
        big.iter().map(|r| r.1).sum::<usize>(),
        read::inspect(&fixture("cocean.crt")).unwrap().entries[0].size as usize
    );
}

/// AC-202 (RENDER): the hidden metadata entry is owned as itself, not as a numbered entry.
#[test]
fn the_metadata_entry_is_owned_as_metadata() {
    let mut builder = CartBuilder::new().with_program(Program::new("one", vec![0x01, 0x08, 0x60]));
    builder.set_meta(0, Field::Title, "a title").unwrap();
    let cart = builder.build().unwrap();
    let info = read::inspect(&cart).unwrap();
    let meta = info.meta_entry.expect("a cart with metadata has the entry");

    let regions = hexmap::regions(&cart).unwrap();
    assert_eq!(
        owned(&regions, Owner::Meta)
            .iter()
            .map(|r| r.1)
            .sum::<usize>(),
        meta.size as usize
    );
    assert_eq!(owned(&regions, Owner::Entry(0)).len(), 1);
    assert!(owned(&regions, Owner::Entry(1)).is_empty());
}

/// AC-202 (RENDER): for every fixture, regions come in order, never overlap and stay in the image;
/// a file the listing refuses is refused here with the same reason.
#[test]
fn regions_are_ordered_and_disjoint_for_every_fixture() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures");
    let mut seen = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|x| x != "crt") {
            continue;
        }
        let cart = std::fs::read(&path).unwrap();
        match read::inspect(&cart) {
            Err(e) => {
                assert_eq!(hexmap::regions(&cart).unwrap_err(), e, "{path:?}");
            }
            Ok(_) => {
                seen += 1;
                let regions = hexmap::regions(&cart).unwrap();
                let mut end = 0;
                for r in &regions {
                    assert!(r.len > 0 && r.start >= end, "{path:?}: {r:?}");
                    end = r.start + r.len;
                }
                assert!(end <= IMAGE_SIZE, "{path:?}");
            }
        }
    }
    assert!(seen > 5);
}
