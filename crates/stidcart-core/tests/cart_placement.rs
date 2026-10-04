//! AC-33, AC-34, AC-35 and AC-38 of the **cart-entries** work-item — where cartridges and the
//! programs around them land, and that every kind reads back.
//!
//! All four are `surfaces: none`. The positions asserted here are the ones the reference tool
//! recorded in the frozen fixtures (`fixtures/MANIFEST.md`); `parity.rs` pins the bytes, these
//! tests pin the *reason* each entry sits where it does.

use std::path::{Path, PathBuf};

use stidcart_core::cartridge::Cartridge;
use stidcart_core::directory::EntryType;
use stidcart_core::{read, CartBuilder, Program};

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(fixtures_dir().join(name)).unwrap_or_else(|e| panic!("fixture {name}: {e}"))
}

fn prg(stem: &str) -> Program {
    Program::new(stem, fixture(&format!("{stem}.prg")))
}

fn cart(stem: &str) -> Cartridge {
    Cartridge::from_crt(&fixture(&format!("{stem}.crt")))
        .unwrap_or_else(|e| panic!("{stem}.crt: {e:?}"))
        .with_name(stem)
}

/// (name, kind, bank, offset) of every entry, in directory order, as the built image records it.
fn layout(b: &CartBuilder) -> Vec<(String, EntryType, u16, u16)> {
    let bytes = b.build().expect("the set should build");
    read::inspect(&bytes)
        .expect("the built image describes itself")
        .entries
        .iter()
        .map(|e| (e.name.clone(), e.kind, e.bank, e.offset))
        .collect()
}

/// AC-33 (none): cartridges are placed before programs, in weight order, first-fit in both halves,
/// aligned ones on 64 KB boundaries.
#[test]
fn ac_33_cartridges_are_placed_first_in_weight_order_first_fit() {
    // cmixed.crt: xbank 8k (4 banks tall) first, then the two 16k-tall carts in input order because
    // their weights tie, then the 8k cart.
    let mixed = CartBuilder::new()
        .with_ef_name("cmixed")
        .with_cartridge(cart("in_8k"))
        .with_cartridge(cart("in_16k"))
        .with_cartridge(cart("in_ult16"))
        .with_cartridge(cart("in_xb8"))
        .with_program(prg("hello"))
        .with_program(prg("ramp"))
        .with_program(prg("big"));
    let got = layout(&mixed);
    assert_eq!(got[0], ("in_8k".into(), EntryType::K8, 7, 0));
    assert_eq!(got[1], ("in_16k".into(), EntryType::K16, 5, 0));
    assert_eq!(got[2], ("in_ult16".into(), EntryType::Ultimax16, 6, 0));
    assert_eq!(got[3], ("in_xb8".into(), EntryType::Xbank8, 1, 0));

    // cxb16a.crt: alignment steps eight banks at a time, so the xbank skips the reserved bank 0
    // and lands at bank 8, leaving banks 1-7 free below it.
    let aligned = CartBuilder::new()
        .with_ef_name("cxb16a")
        .with_cartridge(cart("in_xb16").with_name("aligned").with_align_64k(true))
        .with_program(prg("hello"))
        .with_program(prg("ramp"));
    let got = layout(&aligned);
    assert_eq!(got[0], ("aligned".into(), EntryType::Xbank16, 8, 0));

    // Unaligned, the same xbank takes the first free run: bank 1.
    let plain = CartBuilder::new()
        .with_ef_name("cxb16")
        .with_program(prg("hello"))
        .with_cartridge(cart("in_xb16"));
    assert_eq!(
        layout(&plain)[1],
        ("in_xb16".into(), EntryType::Xbank16, 1, 0)
    );
}

/// AC-33 (none): a program heavier than a cartridge still goes after it. A one-bank cart weighs
/// `1 << 16`, so a program over 64 KiB outweighs it; the reference would hand that program to its
/// cartridge placer (D-025).
#[test]
fn ac_33_a_program_over_64k_still_goes_after_a_small_cartridge() {
    let huge = Program::new("huge", {
        let mut d = vec![0x01, 0x08];
        d.resize(0x10002, 0xea);
        d
    });
    let b = CartBuilder::new()
        .with_ef_name("huge")
        .with_program(huge)
        .with_cartridge(cart("in_8k"));
    let got = layout(&b);
    assert_eq!(got[1], ("in_8k".into(), EntryType::K8, 1, 0));
    assert_eq!(got[0].0, "huge");
    assert_eq!(got[0].1, EntryType::Prg);
}

/// AC-34 (none): programs fill gaps below the top bank as half-confined entries, largest first, and
/// go to the top interleaved when no gap fits.
#[test]
fn ac_34_programs_fill_gaps_as_confined_entries_and_the_top_interleaved_otherwise() {
    // cult8.crt: an Ultimax 8k cart holds only bank 1 ROMH, so bank 1 ROML is a gap below the top.
    let ult8 = CartBuilder::new()
        .with_ef_name("cult8")
        .with_program(prg("hello"))
        .with_cartridge(cart("in_ult8"));
    assert_eq!(layout(&ult8)[0], ("hello".into(), EntryType::PrgRomL, 1, 0));

    // cxb16a.crt: both programs go into the ROML gap below the aligned cart, largest first, so
    // ramp takes offset 0 and hello follows it at 0xbba.
    let aligned = CartBuilder::new()
        .with_ef_name("cxb16a")
        .with_cartridge(cart("in_xb16").with_name("aligned").with_align_64k(true))
        .with_program(prg("hello"))
        .with_program(prg("ramp"));
    let got = layout(&aligned);
    assert_eq!(got[1], ("hello".into(), EntryType::PrgRomL, 1, 0x0bba));
    assert_eq!(got[2], ("ramp".into(), EntryType::PrgRomL, 1, 0));

    // cmixed.crt: the 8k-mode xbank leaves banks 1-4 ROMH free; big fills from bank 1, ramp and
    // hello follow in the same run — the gap cursor never goes back below an earlier gap.
    let mixed = CartBuilder::new()
        .with_ef_name("cmixed")
        .with_cartridge(cart("in_8k"))
        .with_cartridge(cart("in_16k"))
        .with_cartridge(cart("in_ult16"))
        .with_cartridge(cart("in_xb8"))
        .with_program(prg("hello"))
        .with_program(prg("ramp"))
        .with_program(prg("big"));
    let got = layout(&mixed);
    assert_eq!(got[4], ("hello".into(), EntryType::PrgRomH, 3, 0x39dc));
    assert_eq!(got[5], ("ramp".into(), EntryType::PrgRomH, 3, 0x2e22));
    assert_eq!(got[6], ("big".into(), EntryType::PrgRomH, 1, 0x2000));

    // c8k.crt: the 8k cart holds bank 1 ROML, and bank 1 ROMH is the *top* free space rather than
    // a gap (gaps are only searched below the top bank), so hello is interleaved there.
    let k8 = CartBuilder::new()
        .with_ef_name("c8k")
        .with_program(prg("hello"))
        .with_cartridge(cart("in_8k"));
    assert_eq!(layout(&k8)[0], ("hello".into(), EntryType::Prg, 1, 0x2000));
}

/// AC-35 (none): half-confined programs are described and loaded, and a fixture holding them
/// saves back byte-identical.
#[test]
fn ac_35_confined_programs_are_described_loaded_and_saved_back_identically() {
    for (name, index, kind, bank, offset) in [
        ("cult8.crt", 0, EntryType::PrgRomL, 1, 0),
        ("cxb16a.crt", 2, EntryType::PrgRomL, 1, 0),
        ("cmixed.crt", 6, EntryType::PrgRomH, 1, 0x2000),
        ("cocean512.crt", 0, EntryType::PrgRomH, 2, 0x2000),
    ] {
        let original = fixture(name);
        let info = read::inspect(&original).unwrap_or_else(|e| panic!("{name}: inspect: {e}"));
        let e = &info.entries[index];
        assert_eq!(
            (e.kind, e.bank, e.offset),
            (kind, bank, offset),
            "{name}: entry {index}"
        );

        let loaded = read::load(&original).unwrap_or_else(|e| panic!("{name}: load: {e}"));
        let program = loaded.entries()[index]
            .as_program()
            .unwrap_or_else(|| panic!("{name}: entry {index} is a program"));
        let want = fixture(&format!("{}.prg", program.name));
        assert_eq!(
            program.data, want,
            "{name}: the confined program's bytes read back whole"
        );

        let rebuilt = loaded.build().expect("a loaded fixture rebuilds");
        assert!(
            rebuilt == original,
            "{name}: load-and-save is not byte-identical (first difference at {:?})",
            rebuilt
                .iter()
                .zip(original.iter())
                .position(|(a, b)| a != b)
        );
    }
}

/// AC-38 (none): an xbank wider than 32 banks is accepted whenever it fits in flash, is written
/// and read back intact, and an image the reference tool refuses on size alone is read here.
#[test]
fn ac_38_a_wide_xbank_is_bounded_by_flash_not_by_a_flat_512_kib() {
    let wide = cart("in_xbwide");
    assert_eq!(
        wide.banks(),
        (40, 40),
        "640 KiB, over the reference tool's cap"
    );

    let built = CartBuilder::new()
        .with_ef_name("cxbwide")
        .with_cartridge(wide.clone())
        .with_program(prg("hello"))
        .build()
        .expect("a 640 KiB xbank fits in flash beside the reserved bank");

    let back = read::load(&built).expect("the built image loads");
    let got = back.entries()[0]
        .as_cartridge()
        .expect("entry 0 is the cartridge");
    assert_eq!((got.kind, got.banks()), (wide.kind, (40, 40)));
    assert_eq!((&got.roml, &got.romh), (&wide.roml, &wide.romh));

    // The reference tool wrote cxbwide.crt and then refuses to list it ("size $0a0000 > $80000").
    let reference = fixture("cxbwide.crt");
    let info = read::inspect(&reference).expect("described here regardless");
    assert_eq!(info.entries[0].size, 0xa0000);
    let rebuilt = read::load(&reference)
        .expect("loaded here regardless")
        .build()
        .expect("and rebuilt");
    assert!(
        rebuilt == reference,
        "the reference image round-trips byte-identically"
    );

    // The bound is the flash itself: 64 banks in both halves cannot fit beside the reserved bank.
    let mut too_wide = wide;
    too_wide.roml = vec![0xaa; 64 * 0x2000];
    too_wide.romh = vec![0xbb; 64 * 0x2000];
    assert!(
        CartBuilder::new().with_cartridge(too_wide).build().is_err(),
        "an xbank the size of the whole flash cannot be placed"
    );
}
