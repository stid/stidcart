//! AC-2 and AC-3 — byte identity against images the reference tool produced.
//!
//! The fixtures are frozen in `fixtures/`, each with the exact request that produced it recorded
//! in `fixtures/MANIFEST.md`. Nothing here invokes or requires `ndeft`: the point is evidence that
//! outlives the reference implementation.
//!
//! Every case pins the container header name to the reference tool's own string. That field is
//! not the cartridge name — see D-006 — and leaving it at our default would make every case fail
//! for a reason that has nothing to do with correctness.

use std::path::{Path, PathBuf};
use stidcart_core::{CartBuilder, Cartridge, Program};

const REFERENCE_HEADER_NAME: &str = "ndefpack 1.0.1";

struct Case {
    /// Fixture stem; the reference image is `<fixture>.crt`.
    fixture: &'static str,
    /// Inputs in request order, as `(file name in fixtures/, menu name)`. A `.crt` input is a
    /// cartridge image, a `.prg` a program — the same split the reference tool makes.
    inputs: &'static [(&'static str, &'static str)],
    /// Cartridge name, or `None` when the request omitted it.
    ef_name: Option<&'static str>,
    /// Inputs the request marked for 64 KB alignment (the reference tool's `a` type), by menu name.
    aligned: &'static [&'static str],
}

const CASES: &[Case] = &[
    Case {
        fixture: "single",
        inputs: &[("hello.prg", "hello")],
        ef_name: None,
        aligned: &[],
    },
    Case {
        fixture: "named",
        inputs: &[("hello.prg", "hello")],
        ef_name: Some("weird name"),
        aligned: &[],
    },
    Case {
        fixture: "two",
        inputs: &[("hello.prg", "hello"), ("ramp.prg", "ramp")],
        ef_name: Some("two entries"),
        aligned: &[],
    },
    Case {
        fixture: "spill",
        inputs: &[("big.prg", "big")],
        ef_name: Some("spill"),
        aligned: &[],
    },
    // Cartridge entries (cart-entries work-item): one image per kind, then the mixed ordering.
    Case {
        fixture: "c8k",
        inputs: &[("hello.prg", "hello"), ("in_8k.crt", "in_8k")],
        ef_name: Some("c8k"),
        aligned: &[],
    },
    Case {
        fixture: "c16k",
        inputs: &[("ramp.prg", "ramp"), ("in_16k.crt", "in_16k")],
        ef_name: Some("c16k"),
        aligned: &[],
    },
    Case {
        fixture: "cult8",
        inputs: &[("hello.prg", "hello"), ("in_ult8.crt", "in_ult8")],
        ef_name: Some("cult8"),
        aligned: &[],
    },
    Case {
        fixture: "cult16",
        inputs: &[("hello.prg", "hello"), ("in_ult16.crt", "in_ult16")],
        ef_name: Some("cult16"),
        aligned: &[],
    },
    Case {
        fixture: "cocean",
        inputs: &[("big.prg", "big"), ("in_ocean.crt", "in_ocean")],
        ef_name: Some("cocean"),
        aligned: &[],
    },
    Case {
        fixture: "cocean512",
        inputs: &[("hello.prg", "hello"), ("in_ocean512.crt", "in_ocean512")],
        ef_name: Some("cocean512"),
        aligned: &[],
    },
    Case {
        fixture: "cxb16",
        inputs: &[("hello.prg", "hello"), ("in_xb16.crt", "in_xb16")],
        ef_name: Some("cxb16"),
        aligned: &[],
    },
    Case {
        fixture: "cxb16a",
        inputs: &[
            ("in_xb16.crt", "aligned"),
            ("hello.prg", "hello"),
            ("ramp.prg", "ramp"),
        ],
        ef_name: Some("cxb16a"),
        aligned: &["aligned"],
    },
    Case {
        fixture: "cxbult",
        inputs: &[("hello.prg", "hello"), ("in_xbult.crt", "in_xbult")],
        ef_name: Some("cxbult"),
        aligned: &[],
    },
    Case {
        fixture: "cmixed",
        inputs: &[
            ("in_8k.crt", "in_8k"),
            ("in_16k.crt", "in_16k"),
            ("in_ult16.crt", "in_ult16"),
            ("in_xb8.crt", "in_xb8"),
            ("hello.prg", "hello"),
            ("ramp.prg", "ramp"),
            ("big.prg", "big"),
        ],
        ef_name: Some("cmixed"),
        aligned: &[],
    },
    // The reference tool wrote this one and cannot read it back (D-022): the write side is pinned
    // here, the read side in `cart_placement.rs`.
    Case {
        fixture: "cxbwide",
        inputs: &[("in_xbwide.crt", "in_xbwide"), ("hello.prg", "hello")],
        ef_name: Some("cxbwide"),
        aligned: &[],
    },
];

fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn build(case: &Case) -> Vec<u8> {
    let dir = fixtures_dir();
    let mut b = CartBuilder::new().with_header_name(REFERENCE_HEADER_NAME);
    if let Some(n) = case.ef_name {
        b = b.with_ef_name(n);
    }
    for (file, menu_name) in case.inputs {
        let data =
            std::fs::read(dir.join(file)).unwrap_or_else(|e| panic!("fixture input {file}: {e}"));
        if file.ends_with(".crt") {
            let cart = Cartridge::from_crt(&data)
                .unwrap_or_else(|e| panic!("fixture input {file}: {e}"))
                .with_name(menu_name)
                .with_align_64k(case.aligned.contains(menu_name));
            b = b.with_cartridge(cart);
        } else {
            b = b.with_program(Program::new(menu_name, data));
        }
    }
    b.build().expect("fixture case should build")
}

/// AC-2 (CLI): the produced file is byte-identical to the stored reference image.
#[test]
fn ac_2_cli_byte_identical_to_the_reference_image() {
    for case in CASES {
        let want = std::fs::read(fixtures_dir().join(format!("{}.crt", case.fixture)))
            .unwrap_or_else(|e| panic!("reference image {}.crt: {e}", case.fixture));
        let got = build(case);

        assert_eq!(
            got.len(),
            want.len(),
            "[{}] length {} != reference {}",
            case.fixture,
            got.len(),
            want.len()
        );

        if got != want {
            let at = got
                .iter()
                .zip(&want)
                .position(|(a, b)| a != b)
                .expect("lengths matched so a differing byte must exist");
            let n = got.iter().zip(&want).filter(|(a, b)| a != b).count();
            panic!(
                "[{}] {n} bytes differ; first at {at:#x}: got {:#04x}, reference {:#04x}\n\
                 localize with the container/reserved/directory/placement tests before digging here",
                case.fixture, got[at], want[at]
            );
        }
    }
}

/// AC-3 (CLI): the byte-identity check runs from checked-in fixtures alone.
#[test]
fn ac_3_cli_parity_check_runs_without_the_reference_tool() {
    let dir = fixtures_dir();
    assert!(dir.is_dir(), "fixtures/ is missing: {}", dir.display());

    for case in CASES {
        assert!(
            dir.join(format!("{}.crt", case.fixture)).is_file(),
            "reference image {}.crt is not checked in",
            case.fixture
        );
        for (stem, _) in case.inputs {
            assert!(
                dir.join(stem).is_file(),
                "fixture input {stem}.prg is not checked in"
            );
        }
    }

    // A fixture whose originating request is not written down is unreproducible the first time
    // anyone asks how it was built.
    let manifest = std::fs::read_to_string(dir.join("MANIFEST.md")).expect("fixtures/MANIFEST.md");
    for case in CASES {
        assert!(
            manifest.contains(&format!("{}.crt", case.fixture)),
            "MANIFEST.md does not record how {}.crt was produced",
            case.fixture
        );
    }
}
