//! The structured listing both renderers share (gui-shell, D-032).
//!
//! `list` and the GUI viewer must show the same values. These tests hold `listing::describe` to
//! what `list` printed before the values moved into the core: the CLI goldens in
//! `crates/stidcart-cli/tests/golden/` were frozen from the old implementation.

use std::path::{Path, PathBuf};

use stidcart_core::listing::{self, Row};
use stidcart_core::{CartBuilder, Program};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn goldens() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../stidcart-cli/tests/golden")
}

/// Every fixture with its frozen golden, split into the exit code, stdout and stderr parts.
fn fixtures_with_goldens() -> Vec<(String, Vec<u8>, i32, String, String)> {
    let mut out = Vec::new();
    let mut crts: Vec<PathBuf> = std::fs::read_dir(fixtures())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "crt"))
        .collect();
    crts.sort();
    for crt in crts {
        let name = crt.file_stem().unwrap().to_string_lossy().into_owned();
        let golden = std::fs::read_to_string(goldens().join(format!("{name}.txt"))).unwrap();
        let (head, rest) = golden.split_once("\n-- stdout --\n").unwrap();
        let (stdout, stderr) = rest.split_once("-- stderr --\n").unwrap();
        let code: i32 = head.strip_prefix("exit: ").unwrap().parse().unwrap();
        out.push((
            name,
            std::fs::read(&crt).unwrap(),
            code,
            stdout.to_string(),
            stderr.to_string(),
        ));
    }
    out
}

/// One row as `list` lays it out — the column layout stays in the CLI, the values come from core.
fn row_line(r: &Row) -> String {
    format!(
        "  {:>3}  {:<16}  {:<5}  {:<6}  {:>4}  {:<12}  {:>10}",
        r.index, r.name, r.flags, r.kind, r.bank, r.layout, r.size
    )
}

// AC-51 (RENDER): describe yields every value list printed, for every listable fixture
#[test]
fn ac_51_render_describe_yields_lists_values_for_every_fixture() {
    let mut listed = 0;
    for (name, bytes, code, stdout, _) in fixtures_with_goldens() {
        if code != 0 {
            continue;
        }
        listed += 1;
        let l = listing::describe(&bytes).unwrap_or_else(|e| panic!("{name}: {e}"));
        let lines: Vec<&str> = stdout.lines().collect();
        assert_eq!(lines[0], format!("cart: {}", l.cart), "{name}: cart name");
        assert_eq!(
            lines[1],
            format!("built by: {}", l.built_by),
            "{name}: built by"
        );
        assert_eq!(lines[2], format!("menu: {}", l.menu), "{name}: menu");

        let golden_rows: Vec<&str> = lines
            .iter()
            .skip(5) // cart, built by, menu, blank, column header
            .take_while(|s| !s.is_empty())
            .copied()
            .collect();
        let rows: Vec<String> = l.rows.iter().map(row_line).collect();
        if l.rows.is_empty() {
            assert_eq!(lines[4], "  (no entries)", "{name}");
        } else {
            assert_eq!(rows, golden_rows, "{name}: rows");
        }
        for (i, r) in l.rows.iter().enumerate() {
            assert_eq!(r.index, i, "{name}: index is the slot order");
        }

        let last = lines.last().unwrap();
        assert!(
            last.contains(&format!(", {} free", l.free_text())),
            "{name}: free figure {:?} not in {last:?}",
            l.free_text()
        );
        assert_eq!(l.free_text(), listing::readable_size(l.free));
        for r in &l.rows {
            assert_eq!(
                r.size,
                listing::readable_size(r.size_bytes),
                "{name}: size text"
            );
        }
    }
    assert!(listed > 10, "only {listed} listable fixtures found");
}

// AC-51 (RENDER): a refused image fails with the reason list prints
#[test]
fn ac_51_render_a_refused_image_fails_with_lists_reason() {
    let mut refused = 0;
    for (name, bytes, code, _, stderr) in fixtures_with_goldens() {
        if code == 0 {
            continue;
        }
        refused += 1;
        let e = listing::describe(&bytes).expect_err(&name);
        assert_eq!(format!("stidcart: {e}\n"), stderr, "{name}: refusal reason");
    }
    assert!(refused > 0, "no refused fixtures found");
}

// AC-51 (RENDER): control characters in the header name are replaced as list replaces them
#[test]
fn ac_51_render_header_name_control_characters_become_dots() {
    let bytes = CartBuilder::new()
        .with_header_name("evil\x1b[2Jname\x07")
        .with_program(Program::new("hello", vec![0x01, 0x08, 0x60]))
        .build()
        .unwrap();
    let l = listing::describe(&bytes).unwrap();
    assert_eq!(l.built_by, "evil.[2Jname.");
}

/// The size text the listing gives, for `list` and the window alike (readable-sizes, D-044): bytes
/// under 1 KB, then 1024-based KB with one decimal rounded half up, whole for an exact multiple,
/// then MB the same way.
#[test]
fn sizes_read_as_bytes_kb_or_mb() {
    for (bytes, want) in [
        (0, "0 bytes"),
        (1, "1 byte"),
        (652, "652 bytes"),
        (1023, "1023 bytes"),
        (1024, "1 KB"),
        (1075, "1.0 KB"),
        (1076, "1.1 KB"),
        (8192, "8 KB"),
        (10195, "10.0 KB"),
        (16384, "16 KB"),
        (593458, "579.5 KB"),
        (925696, "904 KB"),
        (1048575, "1.0 MB"),
        (1048576, "1 MB"),
        (1572864, "1.5 MB"),
    ] {
        assert_eq!(listing::readable_size(bytes), want, "{bytes} bytes");
    }
}

/// Each row carries the exact byte count its size text rounds.
#[test]
fn every_row_carries_its_exact_size() {
    let bytes = std::fs::read(fixtures().join("cmixed.crt")).unwrap();
    let l = listing::describe(&bytes).unwrap();
    let info = stidcart_core::read::inspect(&bytes).unwrap();
    let exact: Vec<usize> = info.entries.iter().map(|e| e.size as usize).collect();
    let rows: Vec<usize> = l.rows.iter().map(|r| r.size_bytes).collect();
    assert_eq!(rows, exact);
}

/// Each row's family follows its entry type, grouping kinds the way our menu's badges do (D-045).
#[test]
fn every_row_carries_its_kind_family() {
    let family_of = |label: &str| match label {
        "PRG" | "PRG-L" | "PRG-H" => "prg",
        "8K" => "8k",
        "16K" => "16k",
        "ULT8" | "ULT16" => "ult",
        "XB8" | "XB16" | "XBULT" => "xb",
        _ => "other",
    };
    let mut kinds = std::collections::BTreeSet::new();
    for (name, bytes, code, _, _) in fixtures_with_goldens() {
        if code != 0 {
            continue;
        }
        for r in listing::describe(&bytes).unwrap().rows {
            assert_eq!(
                r.family,
                family_of(r.kind),
                "{name}: {} '{}'",
                r.kind,
                r.name
            );
            kinds.insert(r.kind);
        }
    }
    for label in [
        "PRG", "8K", "16K", "ULT8", "ULT16", "XB8", "XB16", "XBULT", "OCEAN",
    ] {
        assert!(
            kinds.contains(label),
            "no fixture lists a {label} entry, so it is untested"
        );
    }
}
