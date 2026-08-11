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
use stidcart_core::{CartBuilder, Program};

const REFERENCE_HEADER_NAME: &str = "ndefpack 1.0.1";

struct Case {
    /// Fixture stem; the reference image is `<fixture>.crt`.
    fixture: &'static str,
    /// Input programs, as `(file stem in fixtures/, menu name)`.
    programs: &'static [(&'static str, &'static str)],
    /// Cartridge name, or `None` when the request omitted it.
    ef_name: Option<&'static str>,
}

const CASES: &[Case] = &[
    Case {
        fixture: "single",
        programs: &[("hello", "hello")],
        ef_name: None,
    },
    Case {
        fixture: "named",
        programs: &[("hello", "hello")],
        ef_name: Some("weird name"),
    },
    Case {
        fixture: "two",
        programs: &[("hello", "hello"), ("ramp", "ramp")],
        ef_name: Some("two entries"),
    },
    Case {
        fixture: "spill",
        programs: &[("big", "big")],
        ef_name: Some("spill"),
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
    for (stem, menu_name) in case.programs {
        let data = std::fs::read(dir.join(format!("{stem}.prg")))
            .unwrap_or_else(|e| panic!("fixture input {stem}.prg: {e}"));
        b = b.with_program(Program::new(menu_name, data));
    }
    b.build().expect("fixture case should build")
}

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
        for (stem, _) in case.programs {
            assert!(
                dir.join(format!("{stem}.prg")).is_file(),
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
