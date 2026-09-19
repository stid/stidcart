//! AC-45 and AC-46 of the **cart-entries** work-item — how the listing shows cartridge entries.
//!
//! Both are `surfaces: RENDER`: the formatter, seen through the binary's stdout.

use std::path::{Path, PathBuf};
use std::process::Command;

fn stidcart() -> Command {
    Command::new(env!("CARGO_BIN_EXE_stidcart"))
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn list(name: &str) -> String {
    let out = stidcart()
        .arg("list")
        .arg(fixtures().join(name))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "listing {name} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// The whitespace-split columns of the line that names `entry`.
fn row(stdout: &str, entry: &str) -> Vec<String> {
    stdout
        .lines()
        .find(|l| l.split_whitespace().any(|c| c == entry))
        .unwrap_or_else(|| panic!("no row for '{entry}' in:\n{stdout}"))
        .split_whitespace()
        .map(str::to_owned)
        .collect()
}

/// AC-45 (RENDER): every kind has a label, cartridges show their bank runs per half, aligned
/// entries are marked, and nothing is shown as `?`.
#[test]
fn ac_45_render_every_kind_is_labelled_and_cartridges_show_their_bank_runs() {
    let mixed = list("cmixed.crt");
    // (name, flags, type, bank, layout)
    for (name, flags, kind, bank, layout) in [
        ("in_8k", "-", "8K", "$07", "L1@0"),
        ("in_16k", "-", "16K", "$05", "L1@0,H1@0"),
        ("in_ult16", "-", "ULT16", "$06", "L1@0,H1@0"),
        ("in_xb8", "-", "XB8", "$01", "L4@0"),
        ("hello", "-", "PRG-H", "$03", "$39dc"),
        ("big", "-", "PRG-H", "$01", "$2000"),
    ] {
        let cols = row(&mixed, name);
        assert_eq!(
            &cols[2..6],
            [flags, kind, bank, layout],
            "row for {name}: {cols:?}"
        );
    }
    assert!(!mixed.contains('?'), "no entry is shown as ?:\n{mixed}");

    let ocean = list("cocean.crt");
    let cols = row(&ocean, "in_ocean");
    assert_eq!(&cols[3..6], ["OCEAN", "$00", "L16@0,H16@16"]);
    assert_eq!(row(&ocean, "big")[3], "PRG-L");
    assert_eq!(row(&list("cocean512.crt"), "in_ocean512")[3], "OCN512");
    assert_eq!(
        row(&list("cult8.crt"), "in_ult8")[3..6],
        ["ULT8", "$01", "H1@0"]
    );
    assert_eq!(
        row(&list("cxbult.crt"), "in_xbult")[3..6],
        ["XBULT", "$01", "L2@0,H2@0"]
    );
    assert_eq!(row(&list("cxb16.crt"), "in_xb16")[3], "XB16");

    let aligned = list("cxb16a.crt");
    let cols = row(&aligned, "aligned");
    assert_eq!(
        &cols[2..5],
        ["A", "XB16", "$08"],
        "the alignment mark: {cols:?}"
    );
    assert_eq!(row(&aligned, "hello")[2], "-", "unaligned, unhidden");
}

/// AC-46 (RENDER): the free-bytes figure accounts for the banks cartridges occupy in both halves.
#[test]
fn ac_46_render_free_bytes_account_for_cartridge_occupied_banks() {
    // c8k: bank 1 ROML is the cart, hello takes 0x66 of bank 1 ROMH; the reference tool reports
    // top space $f9f9a for this image.
    let free = 62 * 0x4000 + (0x2000 - 0x66);
    assert_eq!(free, 0xf9f9a);
    let out = list("c8k.crt");
    assert!(
        out.contains(&format!("${free:06x} ({free}) bytes free")),
        "expected {free:#x} free in:\n{out}"
    );

    // cmixed: the reference tool reports top space $e45be.
    let out = list("cmixed.crt");
    assert!(out.contains("$0e45be (935358) bytes free"), "in:\n{out}");

    // cocean512: every ROML bank is the cart; the top is bank 63 ROMH minus nothing — but hello
    // sits in the ROMH gap at bank 2, so the top run is $7bf9a as the reference tool reports.
    let out = list("cocean512.crt");
    assert!(out.contains("$07bf9a"), "in:\n{out}");
}
