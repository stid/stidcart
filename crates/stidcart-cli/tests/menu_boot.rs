//! custom-menu: our menu on the C64, proven by booting carts in VICE (AC-87..AC-92), and the CI
//! requirement that these boots run rather than skip (AC-94).
//!
//! The reference tool knows nothing of this menu, so the oracle has decayed to the emulator (MAP
//! invariant). Every wait is a `break exec` on the menu's `menu_idle` label or on a marker's fixed
//! spin address — never an instruction count. Keys go straight into the KERNAL buffer, the only
//! place VICE can inject them, in chunks of at most ten with an idle wait before each.

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;

use support::markers::{self, CartKind, Marker};
use support::vice::{self, key, Step, Target};

const SCREEN: (u16, u16) = (0x0400, 0x07e7);

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("stidcart-menu-boot-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn write(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, bytes).unwrap();
    path
}

/// `build --menu stidcart -e <title> -o <dir>/cart.crt [extra…] inputs…`
fn build(dir: &Path, title: &str, extra: &[&str], inputs: &[PathBuf]) -> PathBuf {
    let cart = dir.join("cart.crt");
    let out = Command::new(env!("CARGO_BIN_EXE_stidcart"))
        .args(["build", "--menu", "stidcart", "-e", title, "-o"])
        .arg(&cart)
        .args(extra)
        .args(inputs)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "build failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    cart
}

fn idle() -> Step {
    Step::Break(Target::Label("menu_idle"))
}

fn screen() -> Step {
    Step::Dump(SCREEN.0, SCREEN.1)
}

/// The steps that wait for the menu, press `keys` (in chunks of ten), and wait for `spin`.
fn press_then_wait(keys: &[u8], spin: u16) -> Vec<Step> {
    let mut steps = vec![Step::Labels(vice::menu_labels())];
    for chunk in keys.chunks(10) {
        steps.push(idle());
        steps.push(Step::Keys(chunk.to_vec()));
    }
    steps.push(Step::Break(Target::Addr(spin)));
    steps
}

fn boot_or_panic(dir: &Path, cart: &Path, steps: &[Step]) -> Vec<Vec<u8>> {
    vice::boot(dir, cart, steps).unwrap_or_else(|| {
        panic!(
            "x64sc never reached the last breakpoint. Scratch dir left at {} — reproduce by hand \
             before suspecting the menu.",
            dir.display()
        )
    })
}

fn shows(screen: &[u8], text: &str) -> bool {
    vice::find(screen, &vice::screen_text(text)).is_some()
}

/// The text a marker wrote, read back from its own screen dump.
fn assert_marker(screen: &[u8], m: &Marker) {
    assert!(
        shows(screen, &m.text),
        "the marker '{}' is not on screen: the entry did not start",
        m.text
    );
}

/// Boot the menu and stop at its first idle, with the screen dumped.
fn menu_screen(dir: &Path, cart: &Path) -> Vec<u8> {
    let dumps = boot_or_panic(
        dir,
        cart,
        &[Step::Labels(vice::menu_labels()), idle(), screen()],
    );
    dumps.into_iter().next().unwrap()
}

/// The cart corrupted the way AC-4's negative control does it: everything after the container
/// header zeroed. The menu never runs, so no assertion about it can hold.
fn corrupted(dir: &Path, cart: &Path) -> PathBuf {
    let mut bytes = std::fs::read(cart).unwrap();
    let tail = bytes.len() - 1;
    bytes[0x40..tail].fill(0);
    write(dir, "broken.crt", &bytes)
}

// AC-87: the menu lists visible entries in order, hidden ones absent, first selected
#[test]
fn ac_87_the_menu_lists_visible_entries_in_order_with_the_first_selected() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-87: skipping — x64sc is not installed");
        return;
    }
    let dir = scratch("ac87");
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let inputs = ["alpha", "bravo", "charlie"].map(|n| write(&dir, &format!("{n}.prg"), &hello));
    let cart = build(
        &dir,
        "menu test",
        &["--hide", inputs[1].to_str().unwrap()],
        &inputs,
    );

    let shown = menu_screen(&dir, &cart);
    assert!(shows(&shown, "menu test"), "the cart name is not the title");
    let alpha = vice::find(&shown, &vice::screen_text("alpha")).expect("ALPHA is not listed");
    let charlie = vice::find(&shown, &vice::screen_text("charlie")).expect("CHARLIE is not listed");
    assert!(alpha < charlie, "entries are not in directory order");
    assert!(!shows(&shown, "bravo"), "a hidden entry is listed");
    assert!(
        vice::is_reversed(&shown, alpha, 5),
        "the first entry is not selected"
    );
    assert!(
        !vice::is_reversed(&shown, charlie, 7),
        "a second entry is selected too"
    );

    // Negative control: with the cart destroyed the menu never reaches its idle loop.
    let broken = corrupted(&dir, &cart);
    assert!(
        vice::boot(
            &dir,
            &broken,
            &[Step::Labels(vice::menu_labels()), idle(), screen()]
        )
        .is_none_or(|d| !shows(&d[0], "alpha")),
        "a corrupted cart still listed the entry; this test cannot fail"
    );
}

// AC-88: a list longer than the screen scrolls, and the last entry starts
#[test]
fn ac_88_a_long_list_scrolls_and_the_last_entry_starts() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-88: skipping — x64sc is not installed");
        return;
    }
    let dir = scratch("ac88");
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let last = markers::basic_prg("LAST ENTRY RAN");
    let mut inputs: Vec<PathBuf> = (1..30)
        .map(|i| write(&dir, &format!("e{i:02}.prg"), &hello))
        .collect();
    inputs.push(write(&dir, "e30.prg", &last.bytes));
    let cart = build(&dir, "long list", &[], &inputs);

    let first = menu_screen(&dir, &cart);
    assert!(shows(&first, "e01"), "the first entry is not shown");
    assert!(
        !shows(&first, "e30"),
        "all 30 entries fit; this cart does not exercise scrolling"
    );

    // Move to the last entry and look before starting it.
    let mut steps = vec![Step::Labels(vice::menu_labels())];
    for chunk in vec![key::DOWN; 29].chunks(10) {
        steps.push(idle());
        steps.push(Step::Keys(chunk.to_vec()));
    }
    steps.push(idle());
    steps.push(screen());
    let scrolled = boot_or_panic(&dir, &cart, &steps).remove(0);
    let at =
        vice::find(&scrolled, &vice::screen_text("e30")).expect("e30 never scrolled into view");
    assert!(
        vice::is_reversed(&scrolled, at, 3),
        "e30 is shown but not selected"
    );

    let mut keys = vec![key::DOWN; 29];
    keys.push(key::RETURN);
    let mut steps = press_then_wait(&keys, last.spin);
    steps.push(screen());
    let ran = boot_or_panic(&dir, &cart, &steps).remove(0);
    assert_marker(&ran, &last);
}

// AC-89: programs start — RUN for a load address at or below $0801, SYS above — with the cart out
#[test]
fn ac_89_programs_start_with_the_cart_switched_out() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-89: skipping — x64sc is not installed");
        return;
    }
    let dir = scratch("ac89");
    let low = markers::basic_prg("LOW PROGRAM RAN");
    let high = markers::high_prg("HIGH PROGRAM RAN");
    let inputs = [
        write(&dir, "low.prg", &low.bytes),
        write(&dir, "high.prg", &high.bytes),
    ];
    let cart = build(&dir, "programs", &[], &inputs);
    let menu_start = &stidcart_core::menu::STIDCART_MENU[..2];

    for (keys, m) in [
        (vec![key::RETURN], &low),
        (vec![key::DOWN, key::RETURN], &high),
    ] {
        let mut steps = press_then_wait(&keys, m.spin);
        steps.push(screen());
        steps.push(Step::Dump(0x8000, 0x8001));
        let dumps = boot_or_panic(&dir, &cart, &steps);
        assert_marker(&dumps[0], m);
        assert_ne!(
            dumps[1].as_slice(),
            menu_start,
            "'{}' ran with the cart still mapped at $8000",
            m.text
        );
    }
}

// AC-90: every cartridge kind other than Ocean starts from the menu
#[test]
fn ac_90_every_non_ocean_cartridge_kind_starts() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-90: skipping — x64sc is not installed");
        return;
    }
    for kind in CartKind::ALL {
        let dir = scratch(&format!("ac90-{}", kind.name()));
        let m = markers::cart(kind, &format!("{} CART RAN", kind.name().to_uppercase()));
        let input = write(&dir, &format!("{}.crt", kind.name()), &m.bytes);
        let cart = build(&dir, "carts", &[], &[input]);

        let mut steps = press_then_wait(&[key::RETURN], m.spin);
        steps.push(screen());
        let dumps = vice::boot(&dir, &cart, &steps).unwrap_or_else(|| {
            panic!(
                "the {} cartridge never reached its spin address; scratch dir {}",
                kind.name(),
                dir.display()
            )
        });
        assert_marker(&dumps[0], &m);
    }
}

/// Control for AC-90: each generic marker cart, booted on its own without any menu, writes its
/// marker and reaches its spin address. If this fails, the marker is broken, not the menu. The xbank
/// markers can't be checked this way: VICE has no stand-alone xbank cartridge (type 33 attach
/// returns "NO CART EXISTS", `c64carthooks.c`), so it only meets them inside an EasyFlash image.
#[test]
fn every_marker_cart_runs_on_its_own() {
    if !vice::require_tool("x64sc") {
        eprintln!("marker control: skipping — x64sc is not installed");
        return;
    }
    let generic = [CartKind::K8, CartKind::K16, CartKind::Ultimax8, CartKind::Ultimax16];
    for kind in generic {
        let dir = scratch(&format!("marker-{}", kind.name()));
        let m = markers::cart(kind, "MARKER RAN");
        let cart = write(&dir, "marker.crt", &m.bytes);
        let dumps = vice::boot(&dir, &cart, &[Step::Break(Target::Addr(m.spin)), screen()])
            .unwrap_or_else(|| panic!("the {} marker never reached its spin address", kind.name()));
        assert_marker(&dumps[0], &m);
    }
}

/// BASIC's main input loop, which a cold start reaches after printing `READY.`.
const BASIC_MAIN: u16 = 0xa480;

// AC-91: RUN/STOP leaves to BASIC with the cart out and the full BASIC memory free
#[test]
fn ac_91_run_stop_leaves_to_basic_with_full_memory() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-91: skipping — x64sc is not installed");
        return;
    }
    let dir = scratch("ac91");
    let cart = build(&dir, "leave", &[], &[fixtures().join("hello.prg")]);

    let mut steps = press_then_wait(&[key::STOP], BASIC_MAIN);
    steps.push(screen());
    let basic = boot_or_panic(&dir, &cart, &steps).remove(0);
    assert!(
        shows(&basic, "38911 basic bytes free"),
        "BASIC did not start with full memory"
    );
    assert!(shows(&basic, "ready."), "no READY. prompt");
}

// AC-92: with no visible entries the menu says so, and leaving still works
#[test]
fn ac_92_no_visible_entries_says_so_and_leaving_works() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-92: skipping — x64sc is not installed");
        return;
    }
    let dir = scratch("ac92");
    let only = fixtures().join("hello.prg");
    let cart = build(
        &dir,
        "empty",
        &["--hide", only.to_str().unwrap()],
        &[only.clone()],
    );

    let screen_now = menu_screen(&dir, &cart);
    assert!(
        shows(&screen_now, "no entries"),
        "the menu does not say the cart has no entries"
    );
    assert!(!shows(&screen_now, "hello"), "a hidden entry is listed");

    let mut steps = press_then_wait(&[key::STOP], BASIC_MAIN);
    steps.push(screen());
    let basic = boot_or_panic(&dir, &cart, &steps).remove(0);
    assert!(
        shows(&basic, "ready."),
        "leaving an empty menu did not reach BASIC"
    );
}

// AC-94: in the required checks, the emulator boots run rather than skip
#[test]
fn ac_94_ci_runs_the_boot_checks_instead_of_skipping() {
    let ci = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../.github/workflows/ci.yml"),
    )
    .unwrap();
    // The `gate` job: its lines up to the next job key, which sits at two spaces of indentation.
    let is_job_key = |l: &str| {
        l.starts_with("  ")
            && !l.starts_with("   ")
            && !l.trim().starts_with('#')
            && l.ends_with(':')
    };
    let gate: String = ci
        .lines()
        .skip_while(|l| *l != "  gate:")
        .enumerate()
        .take_while(|(i, l)| *i == 0 || !is_job_key(l))
        .map(|(_, l)| format!("{l}\n"))
        .collect();
    assert!(!gate.is_empty(), "no gate job in ci.yml");

    assert!(
        gate.contains("runs-on: ubuntu-24.04"),
        "gate is not pinned to ubuntu-24.04"
    );
    assert!(
        gate.contains("docs/lcd/ci/install-menu-tools.sh"),
        "gate does not install VICE and 64tass"
    );
    assert!(
        gate.contains("STIDCART_REQUIRE_TOOLS: \"1\"")
            || gate.contains("STIDCART_REQUIRE_TOOLS: 1"),
        "gate does not require the tools, so a missing emulator would skip"
    );

    // And the switch does what it says.
    assert!(!vice::require_tool_in("stidcart-no-such-tool", false));
    assert!(
        std::panic::catch_unwind(|| vice::require_tool_in("stidcart-no-such-tool", true)).is_err(),
        "a required but missing tool did not fail"
    );
}
