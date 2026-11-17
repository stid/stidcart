//! older-menus: every committed build of our menu is recognised as stidcart (AC-131, D-049).

use stidcart_core::menu::{fingerprint, Menu, EARLIER_STIDCART_BUILDS, STIDCART_MENU};
use stidcart_core::{crt, listing, read, CartBuilder, Program};

/// The build `menu/menu.bin` holds now. Changing the menu fails this until the replaced build's
/// fingerprint joins `EARLIER_STIDCART_BUILDS`, so no cart built with it is ever stranded.
const CURRENT: u64 = 0xe41d_ac82_e548_880f;

/// The earlier builds, oldest first, by the commit that last changed `menu/menu.bin`.
const EARLIER: [(&str, u64); 15] = [
    ("0d3dfba", 0x85f5_5429_73af_46a6),
    ("2fbc818", 0x7537_fc2c_cf55_210c),
    ("ee3ec0f", 0x18ca_90a5_e0c1_5d17),
    ("6b80bb6", 0xd28a_8688_0a1c_c6de),
    ("2a1aaa2", 0xe517_0a6f_5851_a075),
    ("280f474", 0x37bc_d298_704d_d07f),
    ("5b54e81", 0x9548_f328_dfac_82fb),
    ("e8eb154", 0xeac9_353f_0e59_ab86),
    ("63ba913", 0xa4cb_0aba_6e81_53e0),
    ("9f4ca36", 0xebee_59a8_625e_4c8c),
    ("5bbe7bc", 0x4088_1607_7fb5_9726),
    ("3aa1b39", 0xbe42_a63d_5738_a710),
    ("4e8f451", 0x7716_0576_a849_b6c4),
    ("7a9a0c1", 0x866f_3dcc_8415_53c1),
    ("8b1f570", 0xb8b7_0e40_1467_ea23),
];

/// The build at 5b54e81, the one carts made by stidcart 0.15 carry.
const AT_5B54E81: &[u8] = include_bytes!("../../../menu/past/5b54e81.bin");

/// A cart of one program with our menu, its loader slot then overwritten by `slot`.
fn cart_with_slot(slot: &[u8]) -> Vec<u8> {
    let built = CartBuilder::new()
        .with_menu(Menu::Stidcart)
        .with_program(Program::new("entry", vec![0x01, 0x08, 0x11, 0x22]))
        .build()
        .unwrap();
    let mut container = crt::parse(&built).unwrap();
    container.image.write_at(0, slot);
    crt::write(&container.image, &container.header_name)
}

// AC-131 (none): the current build's fingerprint is pinned
#[test]
fn ac_131_the_current_build_is_pinned() {
    assert_eq!(
        fingerprint(STIDCART_MENU),
        CURRENT,
        "menu/menu.bin changed: move {CURRENT:#018x} into EARLIER_STIDCART_BUILDS and pin the new build here"
    );
    assert!(!EARLIER_STIDCART_BUILDS.contains(&CURRENT));
}

// AC-131 (none): every earlier build is in the list, and the fingerprint is FNV-1a 64 of the slot
#[test]
fn ac_131_every_earlier_build_is_listed() {
    for (commit, f) in EARLIER {
        assert!(
            EARLIER_STIDCART_BUILDS.contains(&f),
            "the build at {commit} ({f:#018x}) is missing"
        );
    }
    assert_eq!(fingerprint(AT_5B54E81), 0x9548_f328_dfac_82fb);
}

// AC-131 (none): a cart carrying an earlier build describes as one and loads as stidcart
#[test]
fn ac_131_an_earlier_build_is_recognised() {
    let older = cart_with_slot(AT_5B54E81);
    assert_eq!(
        listing::describe(&older).unwrap().menu,
        "stidcart (older build)"
    );
    assert_eq!(read::load(&older).unwrap().menu(), Menu::Stidcart);

    let current = cart_with_slot(STIDCART_MENU);
    assert_eq!(listing::describe(&current).unwrap().menu, "stidcart");
}

// AC-131 (none): a slot matching no build is still unrecognised and refused by load
#[test]
fn ac_131_a_slot_matching_no_build_is_still_refused() {
    let mut slot = AT_5B54E81.to_vec();
    slot[0x100] ^= 0xff;
    let odd = cart_with_slot(&slot);
    assert_eq!(listing::describe(&odd).unwrap().menu, "unrecognised");
    assert!(read::load(&odd).is_err());
}

// D-049: rebuilding a cart carrying an earlier build changes the loader slot and nothing else
#[test]
fn rebuilding_an_earlier_build_changes_only_the_loader_slot() {
    let older = cart_with_slot(AT_5B54E81);
    let rebuilt = read::load(&older).unwrap().build().unwrap();
    let (before, after) = (crt::parse(&older).unwrap(), crt::parse(&rebuilt).unwrap());
    assert_eq!(before.header_name, after.header_name);
    let (b, a) = (before.image.as_bytes(), after.image.as_bytes());
    assert_eq!(&a[..0x2000], STIDCART_MENU);
    assert_eq!(
        &b[0x2000..],
        &a[0x2000..],
        "something outside the loader slot moved"
    );
}

/// Whether `name` is on PATH. Absent, the test skips, unless `STIDCART_REQUIRE_TOOLS=1` (as CI
/// sets it), where an absent tool is a failure. A local copy of the helper in `tests/menu.rs`.
fn require_tool(name: &str) -> bool {
    let present = std::process::Command::new("sh")
        .args(["-c", &format!("command -v {name} >/dev/null 2>&1")])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !present && std::env::var("STIDCART_REQUIRE_TOOLS").as_deref() == Ok("1") {
        panic!("{name} is required (STIDCART_REQUIRE_TOOLS=1) but is not on PATH");
    }
    present
}

/// AC-151 (none): assembly refuses launch routines that reach $DFF6, where the menu keeps its state
/// (D-053). A copy of the source is padded before `ram_end` so the routines end exactly at $DFF5,
/// which assembles, then one byte further, which must not.
#[test]
fn ac_151_assembly_refuses_launch_routines_past_dff5() {
    if !require_tool("64tass") {
        eprintln!("AC-151: skipping — 64tass is not installed");
        return;
    }
    let menu = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../menu");
    let labels = std::fs::read_to_string(menu.join("menu.lbl")).unwrap();
    let ram_end = labels
        .lines()
        .find_map(|l| l.strip_suffix(" .ram_end")?.strip_prefix("al "))
        .map(|a| i64::from_str_radix(a, 16).unwrap())
        .expect("menu.lbl has no .ram_end");
    let room = 0xdff6 - ram_end;
    assert!(
        room >= 0,
        "the launch routines already run to ${:04X}, past $DFF5",
        ram_end - 1
    );

    let tree = std::env::temp_dir().join("stidcart-menu-ac151");
    let assemble = |pad: i64| {
        let _ = std::fs::remove_dir_all(&tree);
        std::fs::create_dir_all(tree.join("src")).unwrap();
        std::fs::copy(menu.join("build.sh"), tree.join("build.sh")).unwrap();
        for entry in std::fs::read_dir(menu.join("src")).unwrap() {
            let path = entry.unwrap().path();
            std::fs::copy(&path, tree.join("src").join(path.file_name().unwrap())).unwrap();
        }
        let launch = tree.join("src/launch.s");
        let text = std::fs::read_to_string(&launch).unwrap().replacen(
            "\nram_end:",
            &format!("\n        .fill {pad}, 0\nram_end:"),
            1,
        );
        std::fs::write(&launch, text).unwrap();
        std::process::Command::new("sh")
            .arg(tree.join("build.sh"))
            .arg(tree.join("out"))
            .current_dir(&tree)
            .output()
            .unwrap()
    };
    let fits = assemble(room);
    assert!(
        fits.status.success(),
        "routines ending at $DFF5 did not assemble: {}",
        String::from_utf8_lossy(&fits.stderr)
    );
    let over = assemble(room + 1);
    assert!(
        !over.status.success(),
        "routines running into $DFF6 assembled without error"
    );
}

/// The refusal's copy of where the menu's `$0100` routine ends is the committed build's label, so
/// AC-163's boundary moves with the routine (D-056).
#[test]
fn the_refusal_knows_where_the_high_copy_routine_ends() {
    let labels = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../menu/menu.lbl"),
    )
    .unwrap();
    let end = labels
        .lines()
        .find_map(|l| l.strip_suffix(" .high_copy_end")?.strip_prefix("al "))
        .map(|a| u16::from_str_radix(a, 16).unwrap())
        .expect("menu.lbl has no .high_copy_end");
    assert_eq!(stidcart_core::menu::HIGH_COPY_END, end);
}
