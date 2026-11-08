//! menu-meta AC-155: neither C64 menu shows the metadata entry (D-054). The reference tool knows
//! nothing of it, so the evidence is a boot of each menu in VICE (MAP: the oracle decays).
//!
//! Skips when `x64sc` is absent, unless `STIDCART_REQUIRE_TOOLS=1`, as the other boot tests do.

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;

use stidcart_core::directory::{self, Slot, ENTRY_LEN};
use stidcart_core::reserved::DIRECTORY_OFFSET;
use support::vice::{self, Step, Target};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("stidcart-meta-boot-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn stidcart(args: &[&str]) {
    let out = Command::new(env!("CARGO_BIN_EXE_stidcart"))
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "stidcart {args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Two programs, `alpha` and `bravo`, built for `menu`, with a title and group on `alpha` set
/// through `edit`, so the cart carries the metadata entry.
fn annotated_cart(dir: &Path, menu: &str) -> PathBuf {
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let inputs: Vec<PathBuf> = ["alpha", "bravo"]
        .iter()
        .map(|n| {
            let p = dir.join(format!("{n}.prg"));
            std::fs::write(&p, &hello).unwrap();
            p
        })
        .collect();
    let cart = dir.join("cart.crt");
    let cart_s = cart.to_str().unwrap();
    let mut build = vec!["build", "--menu", menu, "-e", "meta boot", "-o", cart_s];
    build.extend(inputs.iter().map(|p| p.to_str().unwrap()));
    stidcart(&build);
    stidcart(&[
        "edit",
        cart_s,
        "--title",
        "0",
        "Alpha Game",
        "--group",
        "0",
        "Games",
    ]);
    assert_eq!(
        stored_names(&cart),
        [
            ("alpha".to_string(), false),
            ("bravo".into(), false),
            ("stidcart.meta".into(), true)
        ],
        "the cart carries the hidden metadata entry, so the boot below can show it absent"
    );
    cart
}

/// Every directory slot as stored: its name and hidden flag.
fn stored_names(cart: &Path) -> Vec<(String, bool)> {
    let bytes = std::fs::read(cart).unwrap();
    let image = stidcart_core::crt::parse(&bytes).unwrap().image;
    let mut out = Vec::new();
    for slot in 0..256 {
        let at = DIRECTORY_OFFSET + slot * ENTRY_LEN;
        let raw: [u8; ENTRY_LEN] = image.as_bytes()[at..at + ENTRY_LEN].try_into().unwrap();
        match directory::decode(&raw) {
            Slot::Program(e) | Slot::Cartridge(e) => out.push((e.name, e.hidden)),
            _ => break,
        }
    }
    out
}

/// EasyLoader draws a name as its PETSCII minus 32, bit 7 marking the selected row (boot.rs).
fn easyloader_shows(screen: &[u8], name: &str) -> bool {
    let pattern: Vec<u8> = name.bytes().map(|b| b.wrapping_sub(32) % 128).collect();
    let masked: Vec<u8> = screen.iter().map(|b| b % 128).collect();
    masked.windows(pattern.len()).any(|w| w == pattern)
}

// AC-155: EasyLoader skips the metadata entry like any hidden entry
#[test]
fn ac_155_easyloader_does_not_list_the_metadata_entry() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-155: skipping — x64sc is not installed");
        return;
    }
    let dir = scratch("easyloader");
    let cart = annotated_cart(&dir, "easyloader");
    let steps = [
        Step::Raw("break exec $0000 $ffff\nignore 1 $100000\ng".to_string()),
        Step::Dump(0x0400, 0x07e8),
    ];
    let screen = vice::boot(&dir, &cart, &steps)
        .unwrap_or_else(|| panic!("x64sc never reached the dump; scratch at {}", dir.display()))
        .remove(0);
    assert!(
        easyloader_shows(&screen, "alpha"),
        "EasyLoader did not list the entries"
    );
    assert!(easyloader_shows(&screen, "bravo"));
    assert!(
        !easyloader_shows(&screen, "stidcart.meta"),
        "EasyLoader lists the metadata entry"
    );
}

// AC-155: our menu skips the metadata entry, and counts only the entries
#[test]
fn ac_155_our_menu_does_not_list_the_metadata_entry() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-155: skipping — x64sc is not installed");
        return;
    }
    let dir = scratch("ours");
    let cart = annotated_cart(&dir, "stidcart");
    let steps = [
        Step::Labels(vice::menu_labels()),
        Step::Break(Target::Label("menu_idle")),
        Step::Dump(0x0400, 0x07e7),
    ];
    let screen = vice::boot(&dir, &cart, &steps)
        .unwrap_or_else(|| panic!("x64sc never reached the menu; scratch at {}", dir.display()))
        .remove(0);
    let shows = |t: &str| vice::find(&screen, &vice::screen_text(t)).is_some();
    assert!(
        shows("alpha") && shows("bravo"),
        "our menu did not list the entries"
    );
    assert!(!shows("stidcart.meta"), "our menu lists the metadata entry");
    assert!(shows("1 of 2"), "our menu counts the metadata entry");
}
