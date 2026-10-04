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

/// AC-46 (RENDER): the free-bytes figure accounts for the banks cartridges occupy in both halves,
/// so a further entry of exactly that size can be placed and one byte more cannot.
///
/// The figure is the contiguous space at the top, the reference's `calc_top_space`. Beware the
/// reference's own listing: `flash.c` passes `(total, top)` into `util_display_space(top, total)`,
/// so the number it prints as "top" is the *total* free bytes, gaps included ($e45be for cmixed).
#[test]
fn ac_46_render_free_bytes_account_for_cartridge_occupied_banks() {
    // c8k: bank 1 ROML is the cart and hello takes 0x66 of bank 1 ROMH, so the top run is the rest
    // of that ROMH plus banks 2..63.
    let c8k = 62 * 0x4000 + (0x2000 - 0x66);
    // cmixed: the 8k cart at bank 7 is the highest occupied bank, so the top run is bank 7 ROMH
    // plus banks 8..63; the ROMH gaps below it are not part of a run one program could take.
    let cmixed = 56 * 0x4000 + 0x2000;
    for (name, free) in [("c8k.crt", c8k), ("cmixed.crt", cmixed)] {
        let out = list(name);
        assert!(
            out.contains(&format!("${free:06x} ({free}) bytes free")),
            "{name}: expected {free:#x} free in:\n{out}"
        );
        assert!(
            adds(name, free),
            "{name}: a {free:#x}-byte program should fit"
        );
        assert!(!adds(name, free + 1), "{name}: one byte more should not");
    }

    // cocean512: every ROML bank is the cart, so only bank 63 ROMH counts as top space. A lower
    // bound: the ROMH gap at banks 2..62 is not reported.
    assert!(list("cocean512.crt").contains("$002000 (8192) bytes free"));
    assert!(adds("cocean512.crt", 0x2000));
}

/// Whether `edit --add` of a `size`-byte program (load address included) into a copy succeeds.
fn adds(fixture: &str, size: usize) -> bool {
    let dir = std::env::temp_dir().join(format!("stidcart-cartlist-{fixture}-{size}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let prg = dir.join("p.prg");
    let mut data = vec![0x01, 0x08];
    data.resize(size, 0xea);
    std::fs::write(&prg, data).unwrap();
    stidcart()
        .arg("edit")
        .arg(fixtures().join(fixture))
        .arg("-o")
        .arg(dir.join("out.crt"))
        .arg("--add")
        .arg(&prg)
        .output()
        .unwrap()
        .status
        .success()
}
