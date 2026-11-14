//! custom-menu: our menu on the C64, proven by booting carts in VICE (AC-87..AC-92), and the CI
//! requirement that these boots run rather than skip (AC-94). menu-search-sort adds its filter and
//! sort toggle (AC-96..AC-101), and menu-look its colours and bars (AC-102..AC-104). menu-joystick
//! drives a stick in port 2 through VICE's I/O simulation device (AC-142..AC-146).
//!
//! The reference tool knows nothing of this menu, so the oracle has decayed to the emulator (MAP
//! invariant). Every wait is a `break exec` on the menu's `menu_idle` label or on a marker's fixed
//! spin address — never an instruction count. Keys go straight into the KERNAL buffer, the only
//! place VICE can inject them, in chunks of at most ten with an idle wait before each.

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;

use support::markers::{self, CartKind, Marker};
use support::vice::{self, key, stick, Step, Target};

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
    boot_or_panic_with(dir, cart, &[], steps)
}

fn boot_or_panic_with(dir: &Path, cart: &Path, extra: &[&str], steps: &[Step]) -> Vec<Vec<u8>> {
    vice::boot_with(dir, cart, extra, steps).unwrap_or_else(|| {
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
    if !vice::require_cart_tool("x64sc") {
        eprintln!("AC-87: skipping");
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
    let alpha = listed(&shown, "alpha").expect("ALPHA is not listed");
    let charlie = listed(&shown, "charlie").expect("CHARLIE is not listed");
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
    if !vice::require_cart_tool("x64sc") {
        eprintln!("AC-88: skipping");
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
    assert!(
        listed(&first, "e01").is_some(),
        "the first entry is not shown"
    );
    assert!(
        listed(&first, "e30").is_none(),
        "all 30 entries fit; this cart does not exercise scrolling"
    );

    // Move to the last entry and look before starting it.
    let mut steps = vec![Step::Labels(vice::menu_labels())];
    for chunk in [key::DOWN; 29].chunks(10) {
        steps.push(idle());
        steps.push(Step::Keys(chunk.to_vec()));
    }
    steps.push(idle());
    steps.push(screen());
    let scrolled = boot_or_panic(&dir, &cart, &steps).remove(0);
    let at = listed(&scrolled, "e30").expect("e30 never scrolled into view");
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
    if !vice::require_cart_tool("x64sc") {
        eprintln!("AC-89: skipping");
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
    if !vice::require_cart_tool("x64sc") {
        eprintln!("AC-90: skipping");
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
    if !vice::require_cart_tool("x64sc") {
        eprintln!("marker control: skipping");
        return;
    }
    let generic = [
        CartKind::K8,
        CartKind::K16,
        CartKind::Ultimax8,
        CartKind::Ultimax16,
    ];
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
        eprintln!("AC-91: skipping");
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
        eprintln!("AC-92: skipping");
        return;
    }
    let dir = scratch("ac92");
    let only = fixtures().join("hello.prg");
    let cart = build(
        &dir,
        "empty",
        &["--hide", only.to_str().unwrap()],
        std::slice::from_ref(&only),
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

// test-scopes (D-071): STIDCART_VICE picks which boots a run asks for
#[test]
fn stidcart_vice_picks_the_boots_a_run_asks_for() {
    use vice::Boots;
    // Unset is every boot, so a bare `cargo test` is still the whole suite.
    assert_eq!(vice::boots_in(None), Boots::Full);
    assert_eq!(vice::boots_in(Some("full")), Boots::Full);
    assert_eq!(vice::boots_in(Some("cart")), Boots::Cart);
    assert_eq!(vice::boots_in(Some("none")), Boots::None);
    // A typo must not read as a scope, least of all as "none".
    assert!(std::panic::catch_unwind(|| vice::boots_in(Some("non"))).is_err());
    assert!(std::panic::catch_unwind(|| vice::boots_in(Some(""))).is_err());

    // A cart boot runs in a cart run and a full run; any other boot in a full run only.
    assert!(Boots::Full.asks_for(true) && Boots::Full.asks_for(false));
    assert!(Boots::Cart.asks_for(true) && !Boots::Cart.asks_for(false));
    assert!(!Boots::None.asks_for(true) && !Boots::None.asks_for(false));
}

// --- menu-search-sort: the filter and the sort toggle (AC-96..AC-101, D-039) -----------------
//
// The menu keeps the list in columns 0-20 of rows 5-20, the position and order on row 22, and the
// typed text with its match count on row 23 (D-041). Assertions about the list read only that
// block, so the typed text, the status or the details pane, which repeats the selected name,
// can never stand in for a listed name.

const COLS: usize = 40;
const LIST_ROWS: std::ops::Range<usize> = 5..21;
const LIST_COLS: std::ops::Range<usize> = 0..21;
const STATUS_ROW: usize = 22;
const QUERY_ROW: usize = 23;

fn row(screen: &[u8], n: usize) -> &[u8] {
    &screen[n * COLS..(n + 1) * COLS]
}

fn row_shows(screen: &[u8], n: usize, text: &str) -> bool {
    vice::find(row(screen, n), &vice::screen_text(text)).is_some()
}

/// Where `text` is in the list, as an offset into the whole screen.
fn listed(screen: &[u8], text: &str) -> Option<usize> {
    let pattern = vice::screen_text(text);
    LIST_ROWS.into_iter().find_map(|n| {
        let start = n * COLS + LIST_COLS.start;
        vice::find(&screen[start..n * COLS + LIST_COLS.end], &pattern).map(|at| at + start)
    })
}

/// Assert every name is listed, in this order, and return where each one is.
fn assert_listed_in_order(screen: &[u8], names: &[&str], what: &str) -> Vec<usize> {
    let at: Vec<usize> = names
        .iter()
        .map(|n| listed(screen, n).unwrap_or_else(|| panic!("{what}: '{n}' is not listed")))
        .collect();
    assert!(
        at.windows(2).all(|w| w[0] < w[1]),
        "{what}: the list is not in the order {names:?}"
    );
    at
}

fn assert_not_listed(screen: &[u8], names: &[&str], what: &str) {
    for n in names {
        assert!(listed(screen, n).is_none(), "{what}: '{n}' is listed");
    }
}

/// The key codes GETIN returns for `text` typed without shift: letters `$41-$5A`, the rest as is.
fn typed(text: &str) -> Vec<u8> {
    text.bytes().map(|b| b.to_ascii_uppercase()).collect()
}

/// Boot, then for each group press its keys and dump the screen once the menu is idle again. An
/// empty group dumps the screen as it is.
fn screens_after(dir: &Path, cart: &Path, groups: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let mut steps = vec![Step::Labels(vice::menu_labels())];
    steps.extend(group_steps(groups));
    boot_or_panic(dir, cart, &steps)
}

/// The steps of [`screens_after`] after the labels.
fn group_steps(groups: &[Vec<u8>]) -> Vec<Step> {
    let mut steps = Vec::new();
    for group in groups {
        for chunk in group.chunks(10) {
            steps.push(idle());
            steps.push(Step::Keys(chunk.to_vec()));
        }
        steps.push(idle());
        steps.push(screen());
    }
    steps
}

/// `name.prg` in a directory of its own, so names equal but for case can share one cart on a
/// case-insensitive file system.
fn write_in(dir: &Path, sub: &str, name: &str, bytes: &[u8]) -> PathBuf {
    let d = dir.join(sub);
    std::fs::create_dir_all(&d).unwrap();
    write(&d, &format!("{name}.prg"), bytes)
}

// AC-96: typed text narrows the list to the visible entries containing it, ignoring case
#[test]
fn ac_96_typing_filters_to_visible_entries_containing_the_text() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-96: skipping");
        return;
    }
    let dir = scratch("ac96");
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    // SCALPEL is stored as $61-$7A (the tool swaps case), matches only past its first letter, and
    // is typed with a shifted L: all three letter ranges meet in one match.
    let inputs =
        ["alpha", "beta", "SCALPEL", "alpaca"].map(|n| write(&dir, &format!("{n}.prg"), &hello));
    let cart = build(
        &dir,
        "filter",
        &["--hide", inputs[3].to_str().unwrap()],
        &inputs,
    );

    let mut query = typed("a");
    query.push(0xcc); // shifted L
    query.extend(typed("p"));
    let dumps = screens_after(&dir, &cart, &[vec![], query]);

    // Negative control: unfiltered, the non-matching entry is there.
    assert_listed_in_order(&dumps[0], &["alpha", "beta", "SCALPEL"], "before typing");
    assert_not_listed(&dumps[0], &["alpaca"], "before typing");

    let shown = &dumps[1];
    let at = assert_listed_in_order(shown, &["alpha", "SCALPEL"], "after typing alp");
    assert_not_listed(shown, &["beta"], "after typing alp");
    assert_not_listed(shown, &["alpaca"], "a hidden entry matching the text");
    assert!(
        vice::is_reversed(shown, at[0], 5),
        "the first match is not selected"
    );
    assert!(
        !vice::is_reversed(shown, at[1], 7),
        "the second match is selected too"
    );
    assert!(
        row_shows(shown, QUERY_ROW, "alp"),
        "the typed text is not shown"
    );
    assert!(
        row_shows(shown, QUERY_ROW, "2 of 3 match"),
        "the match count is not shown"
    );
}

// AC-97: DEL removes the last character, CLR/HOME clears the text, and the text stops at 16
#[test]
fn ac_97_del_and_clr_home_edit_the_text() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-97: skipping");
        return;
    }
    let dir = scratch("ac97");
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let inputs = ["alpha", "alpine", "beta"].map(|n| write(&dir, &format!("{n}.prg"), &hello));
    let cart = build(&dir, "editing", &[], &inputs);

    let mut clr = typed("be");
    clr.push(key::CLR);
    let dumps = screens_after(
        &dir,
        &cart,
        &[
            typed("alpi"),
            vec![key::DEL],
            vec![key::HOME],
            clr,
            vec![key::DEL],
            typed(&"x".repeat(18)),
        ],
    );
    let blank = |s: &[u8]| row(s, QUERY_ROW).iter().all(|b| b & 0x7f == 0x20);

    assert_listed_in_order(&dumps[0], &["alpine"], "alpi");
    assert_not_listed(&dumps[0], &["alpha", "beta"], "alpi");
    assert!(row_shows(&dumps[0], QUERY_ROW, "alpi"));

    assert_listed_in_order(&dumps[1], &["alpha", "alpine"], "DEL");
    assert_not_listed(&dumps[1], &["beta"], "DEL");
    assert!(row_shows(&dumps[1], QUERY_ROW, "alp") && !row_shows(&dumps[1], QUERY_ROW, "alpi"));

    for (i, what) in [(2, "HOME"), (3, "CLR"), (4, "DEL with no text")] {
        assert_listed_in_order(&dumps[i], &["alpha", "alpine", "beta"], what);
        assert!(blank(&dumps[i]), "{what}: the text row is not empty");
        assert!(
            row_shows(&dumps[i], STATUS_ROW, "1 of 3"),
            "{what}: position"
        );
    }

    let x = vice::screen_text("x")[0];
    let xs = row(&dumps[5], QUERY_ROW)
        .iter()
        .filter(|&&b| b & 0x7f == x)
        .count();
    assert_eq!(xs, 16, "the text did not stop at 16 characters");
}

// AC-98: CRSR moves through the matches only, and RETURN starts the selected match
#[test]
fn ac_98_the_selection_moves_through_matches_and_starts_one() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-98: skipping");
        return;
    }
    let dir = scratch("ac98");
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let alpine = markers::basic_prg("ALPINE RAN");
    // Unfiltered, the second DOWN would select beta, which is not a marker and never spins.
    let inputs = vec![
        write(&dir, "alpha.prg", &hello),
        write(&dir, "alpine.prg", &alpine.bytes),
        write(&dir, "beta.prg", &hello),
    ];
    let cart = build(&dir, "pick", &[], &inputs);

    let mut steps = vec![Step::Labels(vice::menu_labels())];
    steps.extend([idle(), Step::Keys(typed("alp")), idle(), screen()]);
    steps.extend([Step::Keys(vec![key::DOWN, key::DOWN]), idle(), screen()]);
    steps.extend([
        Step::Keys(vec![key::RETURN]),
        Step::Break(Target::Addr(alpine.spin)),
        screen(),
    ]);
    let dumps = boot_or_panic(&dir, &cart, &steps);

    let at = assert_listed_in_order(&dumps[0], &["alpha", "alpine"], "alp");
    assert_not_listed(&dumps[0], &["beta"], "alp");
    assert!(
        vice::is_reversed(&dumps[0], at[0], 5),
        "the first match is not selected"
    );

    let at = assert_listed_in_order(&dumps[1], &["alpha", "alpine"], "two DOWNs");
    assert!(
        vice::is_reversed(&dumps[1], at[1], 6) && !vice::is_reversed(&dumps[1], at[0], 5),
        "the selection did not stop on the last match"
    );
    assert_marker(&dumps[2], &alpine);
}

// AC-99: nothing matching says so, RETURN does nothing, DEL recovers, RUN/STOP leaves
#[test]
fn ac_99_no_match_says_so_and_recovers() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-99: skipping");
        return;
    }
    let dir = scratch("ac99");
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let inputs = ["alpha", "beta"].map(|n| write(&dir, &format!("{n}.prg"), &hello));
    let cart = build(&dir, "nothing", &[], &inputs);

    let dumps = screens_after(
        &dir,
        &cart,
        &[
            vec![],
            typed("zz"),
            vec![key::RETURN],
            vec![key::DEL, key::DEL],
        ],
    );
    assert!(
        listed(&dumps[0], "no matches").is_none(),
        "says no matches before typing"
    );
    for (i, what) in [(1, "zz"), (2, "RETURN with no match")] {
        assert!(
            listed(&dumps[i], "no matches").is_some(),
            "{what}: no 'no matches'"
        );
        assert!(
            listed(&dumps[i], "no entries").is_none(),
            "{what}: says the cart is empty"
        );
        assert_not_listed(&dumps[i], &["alpha", "beta"], what);
        assert!(
            row_shows(&dumps[i], QUERY_ROW, "zz"),
            "{what}: the text is gone"
        );
    }
    assert_listed_in_order(&dumps[3], &["alpha", "beta"], "DEL DEL");

    let mut keys = typed("zz");
    keys.push(key::STOP);
    let mut steps = press_then_wait(&keys, BASIC_MAIN);
    steps.push(screen());
    let basic = boot_or_panic(&dir, &cart, &steps).remove(0);
    assert!(
        shows(&basic, "ready."),
        "RUN/STOP with text typed did not reach BASIC"
    );
}

// AC-100: F1 lists A-Z ignoring case, ties in directory order, and F1 again restores the order
#[test]
fn ac_100_f1_toggles_between_directory_order_and_a_to_z() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-100: skipping");
        return;
    }
    let dir = scratch("ac100");
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    // ALPHA is stored as $61-$7A and would sort last without folding. ALPHA and alpha are equal
    // folded, so their order shows whether ties keep directory order.
    let inputs = [
        ("1", "delta"),
        ("2", "ALPHA"),
        ("3", "charlie"),
        ("4", "alpha"),
        ("5", "bravo"),
    ]
    .map(|(sub, n)| write_in(&dir, sub, n, &hello));
    let cart = build(&dir, "sorting", &[], &inputs);

    let dumps = screens_after(&dir, &cart, &[vec![], vec![key::F1], vec![key::F1]]);
    let cart_order = ["delta", "ALPHA", "charlie", "alpha", "bravo"];

    let at = assert_listed_in_order(&dumps[0], &cart_order, "at boot");
    assert!(
        vice::is_reversed(&dumps[0], at[0], 5),
        "boot: the first entry is not selected"
    );
    assert!(
        row_shows(&dumps[0], STATUS_ROW, "cart order"),
        "boot: the order is not shown"
    );

    let at = assert_listed_in_order(
        &dumps[1],
        &["ALPHA", "alpha", "bravo", "charlie", "delta"],
        "after F1",
    );
    assert!(
        vice::is_reversed(&dumps[1], at[0], 5),
        "F1: the first entry is not selected"
    );
    assert!(
        !vice::is_reversed(&dumps[1], at[4], 5),
        "F1: the old selection is kept"
    );
    assert!(
        row_shows(&dumps[1], STATUS_ROW, "a-z order"),
        "F1: the order is not shown"
    );

    assert_listed_in_order(&dumps[2], &cart_order, "after F1 twice");
    assert!(
        row_shows(&dumps[2], STATUS_ROW, "cart order"),
        "F1 twice: the order is not shown"
    );
}

// AC-101: the filter and the order combine, whichever comes first, and F1 keeps the text
#[test]
fn ac_101_filter_and_sort_combine() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-101: skipping");
        return;
    }
    let dir = scratch("ac101");
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let inputs =
        ["mango", "banana", "kiwi", "anchovy"].map(|n| write(&dir, &format!("{n}.prg"), &hello));
    let cart = build(&dir, "both", &[], &inputs);
    let a_z = ["anchovy", "banana", "mango"];

    let dumps = screens_after(&dir, &cart, &[typed("an"), vec![key::F1]]);
    assert_listed_in_order(&dumps[0], &["mango", "banana", "anchovy"], "an");
    assert_not_listed(&dumps[0], &["kiwi"], "an");
    let at = assert_listed_in_order(&dumps[1], &a_z, "an, then F1");
    assert_not_listed(&dumps[1], &["kiwi"], "an, then F1");
    assert!(row_shows(&dumps[1], QUERY_ROW, "an"), "F1 dropped the text");
    assert!(
        vice::is_reversed(&dumps[1], at[0], 7),
        "F1: the first match is not selected"
    );
    assert!(row_shows(&dumps[1], QUERY_ROW, "3 of 4 match"), "F1: count");

    let mut keys = vec![key::F1];
    keys.extend(typed("an"));
    let dumps = screens_after(&dir, &cart, &[keys]);
    assert_listed_in_order(&dumps[0], &a_z, "F1, then an");
    assert_not_listed(&dumps[0], &["kiwi"], "F1, then an");
}

// --- the look (AC-104..AC-111, AC-210; D-040, D-041, D-069) ------------------------------------
//
// The menu draws from a RAM copy of the ROM's mixed-case set at $3800 that carries its own glyphs:
// type badges, the rule, the scroll track and thumb, and the wordmark's cells. Tests find a glyph
// by its bitmap, so they hold the approved pixels (the "Stidcart Menu Layouts" mockup for the
// badges, "Stidcart Menu Directions" for the wordmark) and never the code numbers. Colours are read from colour RAM and the VIC's registers,
// low nybble only: VICE fills the high nybble from the bus.

const COLOUR_RAM: (u16, u16) = (0xd800, 0xdbe7);
/// `$D018` (the character set) through `$D021` (the background): `[0]` and `[8..=9]` are used.
const VIC: (u16, u16) = (0xd018, 0xd021);
/// `$D018` with the screen at `$0400` and the ROM's upper-case set; bit 0 reads 1.
const UPPER_CASE: u8 = 0x15;
/// `$D018` with the screen at `$0400` and characters from RAM at `$3800`.
const RAM_CHARSET: u8 = 0x1f;
const CHARSET: (u16, u16) = (0x3800, 0x3fff);
/// The character ROM's mixed-case half, where the CPU sees it with the ROM banked in.
const ROM_MIXED_CASE: (u16, u16) = (0xd800, 0xdfff);
const TEXT_COLOUR: (u16, u16) = (0x0286, 0x0286);
/// The KERNAL's SHIFT+C= lock: bit 7 set stops the keys switching the character set.
const SHIFT_LOCK: (u16, u16) = (0x0291, 0x0291);
/// The label: rows 0-3 in light grey, a band a column in 1-5, the wordmark from column 8 of rows
/// 1-2, the cart name and the entry count ending at column 38 of rows 1 and 2 (D-069).
const LABEL_ROWS: std::ops::Range<usize> = 0..4;
const BAND_COLS: std::ops::RangeInclusive<usize> = 1..=5;
const WORDMARK_COL: usize = 8;
const WORDMARK_ROW: usize = 1;
const WORDMARK_CELLS: usize = 9;
const NAME_ROW: usize = 1;
const COUNT_ROW: usize = 2;
const LABEL_END: usize = 38;
/// The row between the label and the list, all spaces.
const GAP_ROW: usize = 4;
/// The dark grey rule under the list.
const RULE_ROW: usize = 21;
const TRACK_COL: usize = 21;
const PANE_COL: usize = 23;
const VALUE_COL: usize = 29;
const HELP_ROW: usize = 24;
const SELECTION_BAR: std::ops::RangeInclusive<usize> = 1..=20;
/// The menu's own codes: names never produce them (D-041).
const OWN_CODES: std::ops::Range<usize> = 0x60..0x80;

mod colour {
    pub const BLACK: u8 = 0;
    pub const WHITE: u8 = 1;
    pub const CYAN: u8 = 3;
    pub const PURPLE: u8 = 4;
    pub const BLUE: u8 = 6;
    pub const YELLOW: u8 = 7;
    pub const ORANGE: u8 = 8;
    pub const LIGHT_RED: u8 = 10;
    pub const DARK_GREY: u8 = 11;
    pub const LIGHT_GREEN: u8 = 13;
    pub const LIGHT_BLUE: u8 = 14;
    pub const LIGHT_GREY: u8 = 15;
}

type Glyph = [&'static str; 8];

/// A type badge: a 16×8 pill with the kind's name knocked out, over two characters (D-041).
type Badge = [Glyph; 2];

const PRG: Badge = [
    [
        "........", ".XXXXXXX", "XX...X..", "XX.X.X.X", "XX...X..", "XX.XXX.X", "XX.XXX.X",
        ".XXXXXXX",
    ],
    [
        "........", "XXXXXXX.", ".X...XXX", ".X.XXXXX", "XX.X.XXX", ".X.X.XXX", ".X...XXX",
        "XXXXXXX.",
    ],
];
const K8: Badge = [
    [
        "........", ".XXXXXXX", "XXXX...X", "XXXX.X.X", "XXXX...X", "XXXX.X.X", "XXXX...X",
        ".XXXXXXX",
    ],
    [
        "........", "XXXXXXX.", ".X.XXXXX", ".X.XXXXX", "..XXXXXX", ".X.XXXXX", ".X.XXXXX",
        "XXXXXXX.",
    ],
];
const K16: Badge = [
    [
        "........", ".XXXXXXX", "XXX.XX..", "XX..XX.X", "XXX.XX..", "XXX.XX.X", "XX...X..",
        ".XXXXXXX",
    ],
    [
        "........", "XXXXXXX.", ".X.X.XXX", "XX.X.XXX", ".X..XXXX", ".X.X.XXX", ".X.X.XXX",
        "XXXXXXX.",
    ],
];
const ULT: Badge = [
    [
        "........", ".XXXXXXX", "XX.X.X.X", "XX.X.X.X", "XX.X.X.X", "XX.X.X.X", "XX...X..",
        ".XXXXXXX",
    ],
    [
        "........", "XXXXXXX.", "XX...XXX", "XXX.XXXX", "XXX.XXXX", "XXX.XXXX", ".XX.XXXX",
        "XXXXXXX.",
    ],
];
const XB: Badge = [
    [
        "........", ".XXXXXXX", "XXXX.X.X", "XXXX.X.X", "XXXXX.XX", "XXXX.X.X", "XXXX.X.X",
        ".XXXXXXX",
    ],
    [
        "........", "XXXXXXX.", "..XXXXXX", ".X.XXXXX", "..XXXXXX", ".X.XXXXX", "..XXXXXX",
        "XXXXXXX.",
    ],
];
const RULE: Glyph = [
    "........", "........", "........", "XXXXXXXX", "........", "........", "........", "........",
];
const TRACK: Glyph = ["...X...."; 8];
const THUMB: Glyph = ["..XXXX.."; 8];
const BADGES: [Badge; 5] = [PRG, K8, K16, ULT, XB];

/// Every badge half, left and right.
fn badge_halves() -> Vec<[u8; 8]> {
    BADGES.iter().flatten().map(|g| bits(g)).collect()
}

/// The wordmark, 72 by 12 pixels: 2-pixel strokes with the outer corners cut (the "Stidcart Menu
/// Directions" mockup, direction B). It starts on the third scanline of row 1.
const WORDMARK: [&str; 12] = [
    ".XXXXXX...XXXXXXXX..XX..XXXXXXX....XXXXXX....XXXXXX...XXXXXXX...XXXXXXXX",
    "XXXXXXXX..XXXXXXXX..XX..XXXXXXXX..XXXXXXXX..XXXXXXXX..XXXXXXXX..XXXXXXXX",
    "XX....XX.....XX.....XX..XX....XX..XX....XX..XX....XX..XX....XX.....XX...",
    "XX...........XX.....XX..XX....XX..XX........XX....XX..XX....XX.....XX...",
    "XX...........XX.....XX..XX....XX..XX........XX....XX..XX....XX.....XX...",
    "XXXXXXXX.....XX.....XX..XX....XX..XX........XXXXXXXX..XXXXXXXX.....XX...",
    "XXXXXXXX.....XX.....XX..XX....XX..XX........XXXXXXXX..XXXXXXX......XX...",
    "......XX.....XX.....XX..XX....XX..XX........XX....XX..XX....XX.....XX...",
    "......XX.....XX.....XX..XX....XX..XX........XX....XX..XX....XX.....XX...",
    "XX....XX.....XX.....XX..XX....XX..XX....XX..XX....XX..XX....XX.....XX...",
    "XXXXXXXX.....XX.....XX..XXXXXXXX..XXXXXXXX..XX....XX..XX....XX.....XX...",
    ".XXXXXX......XX.....XX..XXXXXXX....XXXXXX...XX....XX..XX....XX.....XX...",
];
const WORDMARK_TOP: usize = 2;

/// Whether pixel (x, y) of the wordmark's two rows of cells is ink.
fn wordmark_ink(x: usize, y: usize) -> bool {
    (WORDMARK_TOP..WORDMARK_TOP + WORDMARK.len()).contains(&y)
        && WORDMARK[y - WORDMARK_TOP].as_bytes()[x] == b'X'
}

/// Cell (cx, cy) of the wordmark as the character set holds it: the ink is the clear bits, so a
/// light grey cell shows it in the background's black.
fn wordmark_cell(cx: usize, cy: usize) -> [u8; 8] {
    let mut out = [0xffu8; 8];
    for (y, row) in out.iter_mut().enumerate() {
        for x in 0..8 {
            if wordmark_ink(cx * 8 + x, cy * 8 + y) {
                *row &= !(0x80 >> x);
            }
        }
    }
    out
}

/// What the label's second line says for `n` visible entries.
fn entries_text(n: usize) -> String {
    if n == 1 {
        "1 entry".into()
    } else {
        format!("{n} entries")
    }
}

fn bits(rows: &[&str]) -> [u8; 8] {
    let mut out = [0u8; 8];
    for (y, r) in rows.iter().enumerate() {
        for (x, p) in r.bytes().enumerate() {
            if p == b'X' {
                out[y] |= 0x80 >> x;
            }
        }
    }
    out
}

fn inverse(g: [u8; 8]) -> [u8; 8] {
    g.map(|b| !b)
}

/// The lower-case letters' codes, which the menu draws as small capitals (D-070).
const SMALL_CODES: std::ops::RangeInclusive<usize> = 1..=26;

/// The small capitals, a to z: five lines each, on lines 2-6 of the cell, strokes two pixels wide
/// (the "Stidcart Menu Legibility" mockup).
const SMALL_CAPS: [[u8; 5]; 26] = [
    [0x3c, 0x66, 0x7e, 0x66, 0x66],
    [0x7c, 0x66, 0x7c, 0x66, 0x7c],
    [0x3c, 0x66, 0x60, 0x66, 0x3c],
    [0x78, 0x6c, 0x66, 0x6c, 0x78],
    [0x7e, 0x60, 0x78, 0x60, 0x7e],
    [0x7e, 0x60, 0x78, 0x60, 0x60],
    [0x3c, 0x60, 0x6e, 0x66, 0x3c],
    [0x66, 0x66, 0x7e, 0x66, 0x66],
    [0x3c, 0x18, 0x18, 0x18, 0x3c],
    [0x1e, 0x0c, 0x0c, 0x6c, 0x38],
    [0x66, 0x6c, 0x78, 0x6c, 0x66],
    [0x60, 0x60, 0x60, 0x60, 0x7e],
    [0x63, 0x77, 0x7f, 0x6b, 0x63],
    [0x66, 0x76, 0x7e, 0x6e, 0x66],
    [0x3c, 0x66, 0x66, 0x66, 0x3c],
    [0x7c, 0x66, 0x7c, 0x60, 0x60],
    [0x3c, 0x66, 0x66, 0x6c, 0x36],
    [0x7c, 0x66, 0x7c, 0x6c, 0x66],
    [0x3e, 0x60, 0x3c, 0x06, 0x7c],
    [0x7e, 0x18, 0x18, 0x18, 0x18],
    [0x66, 0x66, 0x66, 0x66, 0x3c],
    [0x66, 0x66, 0x66, 0x3c, 0x18],
    [0x63, 0x6b, 0x7f, 0x77, 0x63],
    [0x66, 0x3c, 0x18, 0x3c, 0x66],
    [0x66, 0x66, 0x3c, 0x18, 0x18],
    [0x7e, 0x0c, 0x18, 0x30, 0x7e],
];

/// Every glyph the menu adds in `$60-$7F`, without repeats: badges, rule, track and thumb.
fn own_glyphs() -> Vec<[u8; 8]> {
    let mut all = badge_halves();
    all.extend([bits(&RULE), bits(&TRACK), bits(&THUMB)]);
    let mut seen = Vec::new();
    for g in all {
        if !seen.contains(&g) {
            seen.push(g);
        }
    }
    seen
}

/// A menu screen with its colours and character set, each colour masked to its low nybble.
struct Look {
    screen: Vec<u8>,
    colour: Vec<u8>,
    chars: Vec<u8>,
    border: u8,
    background: u8,
    charset: u8,
}

impl Look {
    fn from_dumps(screen: Vec<u8>, colour: Vec<u8>, vic: Vec<u8>, chars: Vec<u8>) -> Look {
        Look {
            screen,
            colour: colour.iter().map(|c| c & 0x0f).collect(),
            chars,
            charset: vic[0] | 1,
            border: vic[8] & 0x0f,
            background: vic[9] & 0x0f,
        }
    }

    /// The bitmap the VIC shows at a cell: its code's eight bytes in the RAM set.
    fn bitmap(&self, cell: usize) -> [u8; 8] {
        let code = self.screen[cell] as usize;
        self.chars[code * 8..code * 8 + 8].try_into().unwrap()
    }

    /// The colour of every cell of `text` where it first shows in row `n` from column `from`, or
    /// `None` if it doesn't.
    fn colours_of(&self, n: usize, from: usize, text: &str) -> Option<&[u8]> {
        let at = vice::find(&row(&self.screen, n)[from..], &vice::screen_text(text))? + from;
        Some(&self.colour[n * COLS + at..n * COLS + at + text.len()])
    }

    /// The ten cells of a pane value, as screen codes without reverse.
    fn value(&self, n: usize) -> Vec<u8> {
        self.screen[n * COLS + VALUE_COL..n * COLS + VALUE_COL + 10]
            .iter()
            .map(|b| b & 0x7f)
            .collect()
    }
}

/// `text` padded with spaces to the pane's ten value cells.
fn value_cells(text: &str) -> Vec<u8> {
    let mut v = vice::screen_text(text);
    v.resize(10, 0x20);
    v
}

/// Boot, then for each group press its keys and read the look once the menu is idle again. An
/// empty group reads the look as it is.
fn looks_after(dir: &Path, cart: &Path, groups: &[Vec<u8>]) -> Vec<Look> {
    let mut steps = vec![Step::Labels(vice::menu_labels())];
    for group in groups {
        for chunk in group.chunks(10) {
            steps.push(idle());
            steps.push(Step::Keys(chunk.to_vec()));
        }
        steps.push(idle());
        steps.push(screen());
        steps.push(Step::Dump(COLOUR_RAM.0, COLOUR_RAM.1));
        steps.push(Step::Dump(VIC.0, VIC.1));
        steps.push(Step::Dump(CHARSET.0, CHARSET.1));
    }
    let mut dumps = boot_or_panic(dir, cart, &steps).into_iter();
    groups
        .iter()
        .map(|_| {
            let mut next = || dumps.next().unwrap();
            let (s, c, v, ch) = (next(), next(), next(), next());
            Look::from_dumps(s, c, v, ch)
        })
        .collect()
}

/// What every menu screen has: black, the RAM set, the label with the wordmark, the cart name and
/// the count of visible entries, the rule and the help bar.
fn check_frame(l: &Look, title: &str, entries: usize) -> Result<(), String> {
    if (l.border, l.background) != (colour::BLACK, colour::BLACK) {
        return Err(format!(
            "border {} and background {} are not black",
            l.border, l.background
        ));
    }
    if l.charset != RAM_CHARSET {
        return Err(format!("$D018 is {:02x}, not the RAM set", l.charset));
    }
    let count = entries_text(entries);
    for n in LABEL_ROWS {
        let text = match n {
            NAME_ROW => title,
            COUNT_ROW => count.as_str(),
            _ => "",
        };
        let text_from = LABEL_END + 1 - text.len();
        for c in 0..COLS {
            let cell = n * COLS + c;
            let want_colour = if BAND_COLS.contains(&c) {
                BANDS[c - BAND_COLS.start()]
            } else {
                colour::LIGHT_GREY
            };
            if l.colour[cell] != want_colour {
                return Err(format!(
                    "label row {n} column {c} is colour {}, not {want_colour}",
                    l.colour[cell]
                ));
            }
            let in_wordmark = (WORDMARK_ROW..WORDMARK_ROW + 2).contains(&n)
                && (WORDMARK_COL..WORDMARK_COL + WORDMARK_CELLS).contains(&c);
            if in_wordmark {
                if l.bitmap(cell) != wordmark_cell(c - WORDMARK_COL, n - WORDMARK_ROW) {
                    return Err(format!(
                        "label row {n} column {c} does not show its wordmark cell"
                    ));
                }
            } else if (text_from..=LABEL_END).contains(&c) {
                let want = vice::screen_text(text)[c - text_from] | 0x80;
                if l.screen[cell] != want {
                    return Err(format!(
                        "label row {n} column {c} is not '{text}' reversed, ending at column {LABEL_END}"
                    ));
                }
            } else if l.screen[cell] != 0xa0 {
                return Err(format!("label row {n} column {c} is not a reverse space"));
            }
        }
    }
    for c in 0..COLS {
        let cell = RULE_ROW * COLS + c;
        if l.bitmap(cell) != bits(&RULE) || l.colour[cell] != colour::DARK_GREY {
            return Err(format!(
                "row {RULE_ROW} column {c} is not the dark grey rule"
            ));
        }
    }
    if let Some(c) = (0..COLS).find(|&c| l.screen[GAP_ROW * COLS + c] != 0x20) {
        return Err(format!("row {GAP_ROW} column {c} is not a space"));
    }
    if !vice::is_reversed(&l.screen, HELP_ROW * COLS, COLS) {
        return Err("the help bar is not reversed across all 40 columns".into());
    }
    if let Some(c) = (0..COLS).find(|&c| l.colour[HELP_ROW * COLS + c] != colour::LIGHT_GREY) {
        return Err(format!("help bar column {c} is not light grey"));
    }
    for text in ["return", "f1", "run/stop"] {
        if !row_shows(&l.screen, HELP_ROW, text) {
            return Err(format!("the help bar does not name '{text}'"));
        }
    }
    Ok(())
}

/// Column 21 of the list rows: the thumb over `at..at + len`, the track elsewhere.
fn check_track(l: &Look, len: usize, at: usize) -> Result<(), String> {
    for (k, n) in LIST_ROWS.enumerate() {
        let cell = n * COLS + TRACK_COL;
        let thumb = (at..at + len).contains(&k);
        let (want, c) = if thumb {
            (bits(&THUMB), colour::LIGHT_GREY)
        } else {
            (bits(&TRACK), colour::DARK_GREY)
        };
        if l.bitmap(cell) != want || l.colour[cell] != c {
            return Err(format!(
                "track row {k}: want the {}",
                if thumb { "thumb" } else { "track" }
            ));
        }
    }
    Ok(())
}

/// The thumb for `n` listed with `top` first on screen: its length and its row on the track.
fn thumb(n: usize, top: usize) -> (usize, usize) {
    let rows = LIST_ROWS.len();
    if n <= rows {
        return (0, 0);
    }
    let len = ((2 * rows * rows + n) / (2 * n)).max(1);
    let span = n - rows;
    (len, (2 * (rows - len) * top + span) / (2 * span))
}

/// The negative control for the frame check: a screen in the ROM set and the C64's own colours,
/// with the right text in the right rows, fails it.
#[test]
fn the_frame_check_rejects_a_plain_screen() {
    let mut screen = vec![0x20u8; 1000];
    for (i, b) in vice::screen_text("stidcart").iter().enumerate() {
        screen[WORDMARK_ROW * COLS + WORDMARK_COL + i] = *b;
    }
    for (i, b) in vice::screen_text("plain").iter().enumerate() {
        screen[NAME_ROW * COLS + LABEL_END - 4 + i] = *b;
    }
    for (i, b) in vice::screen_text(" return start  f1 sort  run/stop basic")
        .iter()
        .enumerate()
    {
        screen[HELP_ROW * COLS + i] = *b;
    }
    let plain = Look {
        screen,
        colour: vec![colour::LIGHT_BLUE; 1000],
        chars: vec![0; 2048],
        border: colour::LIGHT_BLUE,
        background: colour::BLUE,
        charset: 0x17,
    };
    assert!(check_frame(&plain, "plain", 0).is_err());
    let black = Look {
        border: colour::BLACK,
        background: colour::BLACK,
        charset: RAM_CHARSET,
        ..plain
    };
    assert!(check_frame(&black, "plain", 0).is_err());
    assert!(check_track(&black, 0, 0).is_err());
}

#[test]
fn the_thumb_follows_the_rounding_the_spec_gives() {
    assert_eq!(thumb(16, 0), (0, 0));
    assert_eq!(thumb(40, 0), (6, 0)); // 256/40 = 6.4
    assert_eq!(thumb(40, 24), (6, 10));
    assert_eq!(thumb(40, 5), (6, 2)); // 10*5/24 = 2.08
    assert_eq!(thumb(255, 0), (1, 0)); // 256/255 rounds to 1
}

// AC-105: the menu draws from a RAM copy of the ROM's mixed-case set carrying its own glyphs
#[test]
fn ac_105_the_menu_draws_from_a_ram_charset_with_its_own_glyphs() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-105: skipping");
        return;
    }
    let dir = scratch("ac105");
    let cart = build(&dir, "charset", &[], &[fixtures().join("hello.prg")]);
    let dumps = boot_or_panic(
        &dir,
        &cart,
        &[
            Step::Labels(vice::menu_labels()),
            idle(),
            Step::Dump(VIC.0, VIC.1),
            Step::Dump(CHARSET.0, CHARSET.1),
            Step::Raw("bank rom".into()),
            Step::Dump(ROM_MIXED_CASE.0, ROM_MIXED_CASE.1),
            Step::Raw("bank cpu".into()),
        ],
    );
    let (vic, ram, rom) = (&dumps[0], &dumps[1], &dumps[2]);
    assert_eq!(vic[0] | 1, RAM_CHARSET, "$D018 does not point at $3800");
    let glyph = |code: usize| -> [u8; 8] { ram[code * 8..code * 8 + 8].try_into().unwrap() };

    for code in 0..256 {
        if !OWN_CODES.contains(&(code & 0x7f)) && !SMALL_CODES.contains(&(code & 0x7f)) {
            assert_eq!(
                glyph(code),
                rom[code * 8..code * 8 + 8],
                "code ${code:02x} differs from the ROM's mixed-case set"
            );
        }
    }
    for g in own_glyphs() {
        let code = OWN_CODES
            .clone()
            .find(|&c| glyph(c) == g)
            .unwrap_or_else(|| panic!("no code in $60-$7F holds {g:02x?}"));
        if badge_halves().contains(&g) {
            assert_eq!(
                glyph(code + 0x80),
                inverse(g),
                "icon ${code:02x}: code+$80 is not its inverse"
            );
        }
    }
    // The wordmark's cells sit in the reversed half of the menu's codes (D-069).
    for cy in 0..2 {
        for cx in 0..WORDMARK_CELLS {
            let g = wordmark_cell(cx, cy);
            assert!(
                (0xe0..0x100).any(|c| glyph(c) == g),
                "no code in $E0-$FF holds wordmark cell {cx},{cy}"
            );
        }
    }
}

/// AC-213 (none): the menu draws the lower-case letters as small capitals: codes 1-26 of its
/// character set hold five lines each on lines 2-6 of the cell, codes `$81-$9A` their inverses for
/// the selection bar and the label, and the capitals and digits are still the ROM's. A program the
/// menu starts gets the ROM's set back (AC-104).
#[test]
fn ac_213_lower_case_shows_as_small_capitals() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-213: skipping");
        return;
    }
    let dir = scratch("ac213");
    let cart = build(&dir, "letters", &[], &[fixtures().join("hello.prg")]);
    let dumps = boot_or_panic(
        &dir,
        &cart,
        &[
            Step::Labels(vice::menu_labels()),
            idle(),
            Step::Dump(CHARSET.0, CHARSET.1),
            Step::Raw("bank rom".into()),
            Step::Dump(ROM_MIXED_CASE.0, ROM_MIXED_CASE.1),
            Step::Raw("bank cpu".into()),
        ],
    );
    let (ram, rom) = (&dumps[0], &dumps[1]);
    let glyph =
        |set: &[u8], code: usize| -> [u8; 8] { set[code * 8..code * 8 + 8].try_into().unwrap() };
    for (i, lines) in SMALL_CAPS.iter().enumerate() {
        let code = SMALL_CODES.start() + i;
        let mut want = [0u8; 8];
        want[2..7].copy_from_slice(lines);
        let letter = (b'a' + i as u8) as char;
        assert_eq!(
            glyph(ram, code),
            want,
            "'{letter}' is not its small capital"
        );
        assert_eq!(
            glyph(ram, code + 0x80),
            inverse(want),
            "reversed '{letter}' is not the inverse of its small capital"
        );
    }
    for code in (0x30..0x3a).chain(0x41..0x5b) {
        assert_eq!(
            glyph(ram, code),
            glyph(rom, code),
            "code ${code:02x}, a digit or a capital, is not the ROM's"
        );
    }
}

// AC-106: the label, the cart name, the rule and the help bar, on a black screen, empty cart too
#[test]
fn ac_106_the_frame_has_the_label_rule_and_help_bar() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-106: skipping");
        return;
    }
    let dir = scratch("ac106");
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let inputs = ["alpha", "bravo"].map(|n| write(&dir, &format!("{n}.prg"), &hello));
    let cart = build(&dir, "frame test", &[], &inputs);
    let l = looks_after(&dir, &cart, &[vec![]]).remove(0);
    check_frame(&l, "frame test", 2).unwrap_or_else(|e| panic!("{e}"));

    let empty_dir = scratch("ac106-empty");
    let only = fixtures().join("hello.prg");
    let empty = build(
        &empty_dir,
        "empty",
        &["--hide", only.to_str().unwrap()],
        std::slice::from_ref(&only),
    );
    let l = looks_after(&empty_dir, &empty, &[vec![]]).remove(0);
    assert!(
        listed(&l.screen, "no entries").is_some(),
        "empty cart: no message"
    );
    check_frame(&l, "empty", 0).unwrap_or_else(|e| panic!("empty cart: {e}"));
}

// AC-107: each entry has its kind's badge in its kind's colour, and the selection reverses it
#[test]
fn ac_107_entries_carry_their_kind_icon() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-107: skipping");
        return;
    }
    let dir = scratch("ac107");
    let mut inputs = vec![write(
        &dir,
        "prog.prg",
        &std::fs::read(fixtures().join("hello.prg")).unwrap(),
    )];
    let kinds = [
        ("eight", CartKind::K8, K8, colour::YELLOW),
        ("sixteen", CartKind::K16, K16, colour::LIGHT_RED),
        ("ultimax", CartKind::Ultimax8, ULT, colour::PURPLE),
        ("ultisixteen", CartKind::Ultimax16, ULT, colour::PURPLE),
        ("xbank", CartKind::Xbank8, XB, colour::LIGHT_BLUE),
        ("xbsixteen", CartKind::Xbank16, XB, colour::LIGHT_BLUE),
        ("xbulti", CartKind::XbankUltimax, XB, colour::LIGHT_BLUE),
    ];
    for (name, kind, _, _) in &kinds {
        let m = markers::cart(*kind, "ICON");
        inputs.push(write(&dir, &format!("{name}.crt"), &m.bytes));
    }
    let cart = build(&dir, "icons", &[], &inputs);
    let mut all = vec![("prog", PRG, colour::LIGHT_GREEN)];
    all.extend(kinds.iter().map(|(n, _, g, c)| (*n, *g, *c)));

    let looks = looks_after(&dir, &cart, &[vec![], vec![key::DOWN]]);
    for (l, selected, what) in [(&looks[0], 0, "boot"), (&looks[1], 1, "CRSR down")] {
        for (i, (name, glyph, c)) in all.iter().enumerate() {
            let at =
                listed(&l.screen, name).unwrap_or_else(|| panic!("{what}: '{name}' is not listed"));
            let (n, col) = (at / COLS, at % COLS);
            assert_eq!(col, 4, "{what}: '{name}' does not start in column 4");
            let cells = [n * COLS + 1, n * COLS + 2];
            if i == selected {
                for (cell, half) in cells.iter().zip(glyph) {
                    assert_eq!(
                        l.bitmap(*cell),
                        inverse(bits(half)),
                        "{what}: '{name}' is selected but its badge is not reversed"
                    );
                }
                for c in SELECTION_BAR {
                    let cell = n * COLS + c;
                    assert!(
                        l.screen[cell] & 0x80 != 0 && l.colour[cell] == colour::CYAN,
                        "{what}: '{name}': column {c} is not in the cyan bar"
                    );
                }
            } else {
                for (cell, half) in cells.iter().zip(glyph) {
                    assert_eq!(
                        l.bitmap(*cell),
                        bits(half),
                        "{what}: '{name}' has the wrong badge"
                    );
                    assert_eq!(l.colour[*cell], *c, "{what}: '{name}' badge colour");
                }
                let name_colours = &l.colour[at..at + name.len()];
                assert!(
                    name_colours.iter().all(|&x| x == colour::WHITE),
                    "{what}: '{name}' is not white"
                );
            }
        }
    }
}

/// The pane's rows (D-069): the chip, the title or name, and the three facts at the foot.
const CHIP_ROW: usize = 5;
const HEAD_ROW: usize = 7;
const FACT_ROWS: std::ops::Range<usize> = 18..21;

/// Assert the pane's chip: `label` between two spaces, reversed, in colour `c`, from the pane's
/// first column of row 5, plain spaces after it, and row 6 blank.
fn assert_chip(l: &Look, label: &str, c: u8, what: &str) {
    let text = format!(" {label} ");
    let from = CHIP_ROW * COLS + PANE_COL;
    let want: Vec<u8> = vice::screen_text(&text).iter().map(|b| b | 0x80).collect();
    assert_eq!(
        l.screen[from..from + text.len()],
        want,
        "{what}: the chip is not '{text}' reversed"
    );
    assert!(
        l.colour[from..from + text.len()].iter().all(|&k| k == c),
        "{what}: the chip is not colour {c}"
    );
    assert!(
        l.screen[from + text.len()..from + 16]
            .iter()
            .all(|&b| b == 0x20),
        "{what}: the chip's row does not end in spaces"
    );
    assert_eq!(
        pane_row(&l.screen, CHIP_ROW + 1),
        [0x20u8; 16],
        "{what}: row 6"
    );
}

/// Assert the facts in rows 18-20: each label in light grey from the pane's first column, its value
/// exact from column 29 in white (D-070).
fn assert_facts(l: &Look, facts: [(&str, &str); 3], what: &str) {
    for (n, (key, value)) in FACT_ROWS.zip(facts) {
        let label = l
            .colours_of(n, PANE_COL, key)
            .unwrap_or_else(|| panic!("{what}: row {n} has no '{key}'"));
        assert!(
            label.iter().all(|&c| c == colour::LIGHT_GREY),
            "{what}: '{key}' is not light grey"
        );
        assert!(
            pane_row(&l.screen, n).starts_with(&vice::screen_text(key)),
            "{what}: '{key}' does not start row {n} of the pane"
        );
        assert_eq!(
            l.value(n),
            value_cells(value),
            "{what}: {key} is not exactly '{value}'"
        );
        let cells = n * COLS + VALUE_COL..n * COLS + VALUE_COL + value.len();
        assert!(
            l.colour[cells].iter().all(|&c| c == colour::WHITE),
            "{what}: the value of '{key}' is not white"
        );
    }
}

/// Assert pane rows `rows` are blank.
fn assert_pane_blank(l: &Look, rows: std::ops::Range<usize>, what: &str) {
    for n in rows {
        assert_eq!(
            pane_row(&l.screen, n),
            [0x20u8; 16],
            "{what}: pane row {n} is not blank"
        );
    }
}

/// AC-210 (none): for an entry the pane shows a chip with its kind in the kind's colour in row 5,
/// its name in white in row 7, and in rows 18-20 its size, its load address or first bank, and how
/// it starts, labels in light grey and values in white (D-070). Nothing says "type" and the rows between are
/// blank. (AC-108 until D-069, when the facts opened the pane and took the kind's colour.)
#[test]
fn ac_210_the_pane_leads_with_the_kind_and_ends_with_the_facts() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-210: skipping");
        return;
    }
    let dir = scratch("ac210");
    let low = markers::basic_prg("LOW");
    let high = markers::high_prg("HIGH");
    let eight = markers::cart(CartKind::K8, "EIGHT");
    let inputs = [
        write(&dir, "low.prg", &low.bytes),
        write(&dir, "high.prg", &high.bytes),
        write(&dir, "eight.crt", &eight.bytes),
    ];
    let cart = build(&dir, "pane", &[], &inputs);
    let listing = Command::new(env!("CARGO_BIN_EXE_stidcart"))
        .arg("list")
        .arg(&cart)
        .output()
        .unwrap();
    let listing = String::from_utf8(listing.stdout).unwrap();
    let bank = listing
        .lines()
        .find(|l| l.split_whitespace().nth(1) == Some("eight"))
        .and_then(|l| l.split_whitespace().find(|w| w.starts_with('$')))
        .map(|w| u32::from_str_radix(&w[1..], 16).unwrap())
        .expect("list shows no bank for eight");

    let looks = looks_after(&dir, &cart, &[vec![], vec![key::DOWN], vec![key::DOWN]]);
    let bank = bank.to_string();
    for (l, name, kind, kind_colour, facts) in [
        (
            &looks[0],
            "low",
            "program",
            colour::LIGHT_GREEN,
            [("size", "1K"), ("load", "$07F0"), ("start", "RUN")],
        ),
        (
            &looks[1],
            "high",
            "program",
            colour::LIGHT_GREEN,
            [("size", "1K"), ("load", "$C000"), ("start", "SYS 49152")],
        ),
        (
            &looks[2],
            "eight",
            "8K cart",
            colour::YELLOW,
            [("size", "8K"), ("bank", bank.as_str()), ("start", "reset")],
        ),
    ] {
        assert_chip(l, kind, kind_colour, name);
        assert_eq!(
            pane_row(&l.screen, HEAD_ROW),
            wrapped(name)[0],
            "'{name}': row 7 is not the name alone"
        );
        assert!(
            l.colours_of(HEAD_ROW, PANE_COL, name)
                .is_some_and(|c| c.iter().all(|&c| c == colour::WHITE)),
            "'{name}': the name is not white"
        );
        assert_pane_blank(l, HEAD_ROW + 1..FACT_ROWS.start, name);
        assert_facts(l, facts, name);
        let pane: Vec<u8> = LIST_ROWS.flat_map(|n| pane_row(&l.screen, n)).collect();
        assert!(
            vice::find(&pane, &vice::screen_text("type")).is_none(),
            "'{name}': the pane still has a 'type' row"
        );
    }
}

// AC-109: a thumb on the track in column 21 shows the share of the list in view and where it is
#[test]
fn ac_109_the_scroll_bar_shows_where_the_view_is() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-109: skipping");
        return;
    }
    let dir = scratch("ac109");
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let inputs: Vec<PathBuf> = (1..=40)
        .map(|i| write(&dir, &format!("e{i:02}.prg"), &hello))
        .collect();
    let cart = build(&dir, "forty", &[], &inputs);

    let looks = looks_after(
        &dir,
        &cart,
        &[
            vec![],
            vec![key::DOWN; 20],
            vec![key::DOWN; 19],
            typed("e1"),
        ],
    );
    for (l, (len, at), what) in [
        (&looks[0], thumb(40, 0), "boot"),
        (&looks[1], thumb(40, 5), "20 down"),
        (&looks[2], thumb(40, 24), "at the end"),
        (&looks[3], thumb(11, 0), "filtered to 11"),
    ] {
        check_track(l, len, at).unwrap_or_else(|e| panic!("{what}: {e}"));
    }

    let few_dir = scratch("ac109-few");
    let few: Vec<PathBuf> = ["a", "b", "c"]
        .iter()
        .map(|n| write(&few_dir, &format!("{n}.prg"), &hello))
        .collect();
    let few_cart = build(&few_dir, "few", &[], &few);
    let l = looks_after(&few_dir, &few_cart, &[vec![]]).remove(0);
    check_track(&l, 0, 0).unwrap_or_else(|e| panic!("three entries: {e}"));
}

// AC-110: row 22 shows the position and order; row 23 the typed text and its match count
#[test]
fn ac_110_the_status_shows_the_position_and_the_filter_its_matches() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-110: skipping");
        return;
    }
    let dir = scratch("ac110");
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let inputs = ["alpha", "alpine", "beta"].map(|n| write(&dir, &format!("{n}.prg"), &hello));
    let cart = build(&dir, "status", &[], &inputs);

    let looks = looks_after(&dir, &cart, &[vec![], vec![key::DOWN], typed("al")]);
    let grey = |l: &Look, n: usize, text: &str| {
        l.colours_of(n, 0, text)
            .unwrap_or_else(|| panic!("row {n} does not show '{text}'"))
            .iter()
            .all(|&c| c == colour::LIGHT_GREY)
    };
    assert!(
        grey(&looks[0], STATUS_ROW, "1 of 3   cart order"),
        "boot: status"
    );
    assert!(
        row(&looks[0].screen, QUERY_ROW)
            .iter()
            .all(|b| b & 0x7f == 0x20),
        "boot: row 23 shows something with no text typed"
    );
    assert!(grey(&looks[1], STATUS_ROW, "2 of 3"), "CRSR down: position");

    let l = &looks[2];
    assert!(
        grey(l, STATUS_ROW, "1 of 2"),
        "al: position among the matches"
    );
    assert!(grey(l, QUERY_ROW, "find:"), "al: the label is not grey");
    let text = l
        .colours_of(QUERY_ROW, 6, "al")
        .expect("al: the text is not shown");
    assert!(
        text.iter().all(|&c| c == colour::YELLOW),
        "al: the text is not yellow"
    );
    assert!(grey(l, QUERY_ROW, "2 of 3 match"), "al: the match count");
}

// AC-111: a cursor move that doesn't scroll leaves the other list rows alone
#[test]
fn ac_111_a_cursor_move_rewrites_only_the_rows_it_changes() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-111: skipping");
        return;
    }
    let dir = scratch("ac111");
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let inputs =
        ["alpha", "bravo", "charlie", "delta"].map(|n| write(&dir, &format!("{n}.prg"), &hello));
    let cart = build(&dir, "sentinel", &[], &inputs);

    // A star in column 15 of charlie's and delta's rows (7 and 8), past their names.
    let cells = [7 * COLS + 15, 8 * COLS + 15];
    let pokes: String = cells
        .iter()
        .map(|c| format!("> ${:04x} 2a\n", 0x0400 + c))
        .collect();
    let steps = vec![
        Step::Labels(vice::menu_labels()),
        idle(),
        Step::Raw(pokes),
        Step::Keys(vec![key::DOWN]),
        idle(),
        screen(),
    ];
    let shown = boot_or_panic(&dir, &cart, &steps).remove(0);
    let bravo = listed(&shown, "bravo").expect("bravo is not listed");
    assert!(
        vice::is_reversed(&shown, bravo, 5),
        "the DOWN did not select bravo"
    );
    for c in cells {
        assert_eq!(
            shown[c],
            0x2a,
            "row {} was rewritten by a move that touched rows 5 and 6",
            c / COLS
        );
    }
}

// AC-104: a program started from the menu, and BASIC after RUN/STOP, find the C64's own colours
// and character set, and nothing the menu starts finds SHIFT+C= locked
#[test]
fn ac_104_what_the_menu_hands_over_has_the_c64_colours() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-104: skipping");
        return;
    }
    let dir = scratch("ac104");
    let m = markers::basic_prg("COLOURS PROGRAM RAN");
    let cart = build(&dir, "colours", &[], &[write(&dir, "prog.prg", &m.bytes)]);

    for (key, spin, what) in [
        (key::RETURN, m.spin, "the started program"),
        (key::STOP, BASIC_MAIN, "BASIC after RUN/STOP"),
    ] {
        let mut steps = press_then_wait(&[key], spin);
        steps.push(screen());
        steps.push(Step::Dump(VIC.0, VIC.1));
        steps.push(Step::Dump(TEXT_COLOUR.0, TEXT_COLOUR.1));
        steps.push(Step::Dump(SHIFT_LOCK.0, SHIFT_LOCK.1));
        let dumps = boot_or_panic(&dir, &cart, &steps);
        if key == key::RETURN {
            assert_marker(&dumps[0], &m);
        }
        assert_eq!(dumps[3][0] & 0x80, 0, "{what}: SHIFT+C= is locked");
        let got = (
            dumps[1][8] & 0x0f,
            dumps[1][9] & 0x0f,
            dumps[2][0] & 0x0f,
            dumps[1][0] | 1,
        );
        assert_eq!(
            got,
            (
                colour::LIGHT_BLUE,
                colour::BLUE,
                colour::LIGHT_BLUE,
                UPPER_CASE
            ),
            "{what}: (border, background, text colour, $D018)"
        );
    }

    // A cartridge's own reset code runs before the KERNAL's CINT, so whatever the menu left in RAM
    // reaches it.
    let dir = scratch("ac104-cart");
    let m = markers::cart(CartKind::K8, "SHIFT LOCK CART RAN");
    let cart = build(&dir, "lock", &[], &[write(&dir, "k8.crt", &m.bytes)]);
    let mut steps = press_then_wait(&[key::RETURN], m.spin);
    steps.push(screen());
    steps.push(Step::Dump(SHIFT_LOCK.0, SHIFT_LOCK.1));
    let dumps = boot_or_panic(&dir, &cart, &steps);
    assert_marker(&dumps[0], &m);
    assert_eq!(
        dumps[1][0] & 0x80,
        0,
        "the started cart: SHIFT+C= is locked"
    );
}

// --- the label and the interrupt (AC-123, AC-127, AC-128, AC-207..AC-209; D-048, D-069) -----------
//
// The label is reverse characters, so screen and colour RAM hold all of it. Two tests also read
// VICE's own picture of the frame (`Step::Shot`): it is what shows the wordmark's pixels as a person
// sees them, and that no raster line is recoloured any more. Each waits two interrupts first,
// because VICE's picture of the frame in progress still holds the previous frame below the beam. In
// the 384x272 PAL capture, raster line L is image row L - 16 and the 320 visible pixels start at
// column 32.

const SHOT_SIZE: (usize, usize) = (384, 272);
const SHOT_X: usize = 32;
const SHOT_FIRST_LINE: usize = 16;
/// The raster line where character row 0 starts, with the default vertical scroll.
const ROW0_LINE: usize = 51;
const BANDS: [u8; 5] = [
    colour::LIGHT_RED,
    colour::ORANGE,
    colour::YELLOW,
    colour::LIGHT_GREEN,
    colour::LIGHT_BLUE,
];
/// The image row of raster line `line`.
fn shot_y(line: usize) -> usize {
    line - SHOT_FIRST_LINE
}

/// A cart with `n` programs, so the list scrolls when `n` passes 16.
fn programs_cart(dir: &Path, title: &str, n: usize) -> PathBuf {
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let inputs: Vec<PathBuf> = (0..n)
        .map(|i| write(dir, &format!("entry {i:02}.prg"), &hello))
        .collect();
    build(dir, title, &[], &inputs)
}

fn shot_after_boot(dir: &Path, cart: &Path) -> vice::Shot {
    let dumps = boot_or_panic(
        dir,
        cart,
        &[
            Step::Labels(vice::menu_labels()),
            idle(),
            Step::Break(Target::Label("irq")),
            Step::Break(Target::Label("irq")),
            Step::Shot,
        ],
    );
    let shot = vice::Shot::parse(&dumps[0]);
    assert_eq!(
        (shot.width, shot.height),
        SHOT_SIZE,
        "not VICE's PAL capture"
    );
    shot
}

// AC-123: the selection bar is cyan (D-059; orange until then)
#[test]
fn ac_123_the_selection_bar_is_cyan() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-123: skipping");
        return;
    }
    let dir = scratch("ac123");
    let cart = programs_cart(&dir, "brand", 3);
    let l = looks_after(&dir, &cart, &[vec![]]).remove(0);
    check_frame(&l, "brand", 3).unwrap_or_else(|e| panic!("{e}"));
    let first = LIST_ROWS.start;
    for c in SELECTION_BAR {
        assert_eq!(
            l.colour[first * COLS + c],
            colour::CYAN,
            "column {c} of the selection is not cyan"
        );
    }
}

/// The colour VICE's picture shows at pixel (x, y) of the visible screen.
fn shot_pixel(shot: &vice::Shot, x: usize, y: usize) -> u8 {
    shot.at(SHOT_X + x, shot_y(ROW0_LINE + y))
}

/// AC-207 (none): rows 0-3 are the label, reverse spaces in light grey with a band a column in
/// columns 1-5, and row 4 is spaces. In VICE's picture the label's lines are light grey and the five
/// bands, row 4 is black on every scanline (no raster strip), and no cell is multicolour.
#[test]
fn ac_207_the_label_fills_rows_0_to_3() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-207: skipping");
        return;
    }
    let dir = scratch("ac207");
    let cart = programs_cart(&dir, "label", 3);
    let l = looks_after(&dir, &cart, &[vec![]]).remove(0);
    check_frame(&l, "label", 3).unwrap_or_else(|e| panic!("{e}"));

    let dumps = boot_or_panic(
        &dir,
        &cart,
        &[
            Step::Labels(vice::menu_labels()),
            idle(),
            Step::Break(Target::Label("irq")),
            Step::Break(Target::Label("irq")),
            Step::Shot,
            Step::Dump(0xd016, 0xd016),
        ],
    );
    let shot = vice::Shot::parse(&dumps[0]);
    assert_eq!(
        (shot.width, shot.height),
        SHOT_SIZE,
        "not VICE's PAL capture"
    );
    assert_eq!(dumps[1][0] & 0x10, 0, "multicolour is on");
    for y in 0..LABEL_ROWS.end * 8 {
        for x in 0..320 {
            let (c, n) = (x / 8, y / 8);
            // The wordmark and the two lines of text are AC-208's and AC-209's.
            if (WORDMARK_ROW..WORDMARK_ROW + 2).contains(&n) && c >= WORDMARK_COL {
                continue;
            }
            let want = if BAND_COLS.contains(&c) {
                BANDS[c - BAND_COLS.start()]
            } else {
                colour::LIGHT_GREY
            };
            let got = shot_pixel(&shot, x, y);
            assert_eq!(got, want, "label pixel {x},{y} is colour {got}, not {want}");
        }
    }
    for y in GAP_ROW * 8..(GAP_ROW + 1) * 8 {
        for x in 0..320 {
            let got = shot_pixel(&shot, x, y);
            assert_eq!(got, colour::BLACK, "row 4 pixel {x},{y} is colour {got}");
        }
    }
}

/// AC-208 (none): columns 8-16 of rows 1-2 show the wordmark's 72 by 12 pixels from the third
/// scanline of row 1, black on light grey, in VICE's picture.
#[test]
fn ac_208_the_wordmark_is_drawn_on_the_label() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-208: skipping");
        return;
    }
    let dir = scratch("ac208");
    let cart = programs_cart(&dir, "wordmark", 3);
    let shot = shot_after_boot(&dir, &cart);
    let mut ink = 0;
    for y in 0..16 {
        for x in 0..WORDMARK_CELLS * 8 {
            let want = if wordmark_ink(x, y) {
                ink += 1;
                colour::BLACK
            } else {
                colour::LIGHT_GREY
            };
            let got = shot_pixel(&shot, WORDMARK_COL * 8 + x, WORDMARK_ROW * 8 + y);
            assert_eq!(
                got, want,
                "wordmark pixel {x},{y} is colour {got}, not {want}"
            );
        }
    }
    assert!(
        ink > 200,
        "the expected wordmark has only {ink} pixels of ink"
    );
}

/// AC-209 (none): the cart name ends at column 38 of row 1 and the count of visible entries at
/// column 38 of row 2, reversed in light grey: "1 entry" for one, a 16-character name whole, and
/// hidden entries not counted.
#[test]
fn ac_209_the_label_names_the_cart_and_counts_its_entries() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-209: skipping");
        return;
    }
    let dir = scratch("ac209");
    let one = programs_cart(&dir, "sixteen chars ok", 1);
    let l = looks_after(&dir, &one, &[vec![]]).remove(0);
    check_frame(&l, "sixteen chars ok", 1).unwrap_or_else(|e| panic!("one entry: {e}"));

    let dir = scratch("ac209-hidden");
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let inputs: Vec<PathBuf> = (0..12)
        .map(|i| write(&dir, &format!("entry {i:02}.prg"), &hello))
        .collect();
    let hidden = inputs[3].to_str().unwrap();
    let cart = build(&dir, "x", &["--hide", hidden], &inputs);
    let l = looks_after(&dir, &cart, &[vec![]]).remove(0);
    check_frame(&l, "x", 11).unwrap_or_else(|e| panic!("eleven of twelve visible: {e}"));
}

/// AC-212 (none): the key bar in row 24 is reversed in light grey across all 40 columns, on the
/// menu and on the help screen, and the help keeps rows 0-4 as they were.
#[test]
fn ac_212_the_key_bar_is_light_grey_on_the_menu_and_the_help() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-212: skipping");
        return;
    }
    let dir = scratch("ac212");
    let cart = programs_cart(&dir, "keys", 3);
    let looks = looks_after(&dir, &cart, &[vec![], vec![key::F7]]);
    for (l, what) in [(&looks[0], "menu"), (&looks[1], "help")] {
        assert!(
            vice::is_reversed(&l.screen, HELP_ROW * COLS, COLS),
            "{what}: the key bar is not reversed across all 40 columns"
        );
        assert!(
            l.colour[HELP_ROW * COLS..]
                .iter()
                .all(|&c| c == colour::LIGHT_GREY),
            "{what}: the key bar is not light grey"
        );
    }
    assert_ne!(
        looks[0].screen[HELP_ROW * COLS..],
        looks[1].screen[HELP_ROW * COLS..],
        "F7 did not change the key bar: the help is not up"
    );
    let top = ..(GAP_ROW + 1) * COLS;
    assert!(
        looks[0].screen[top] == looks[1].screen[top]
            && looks[0].colour[top] == looks[1].colour[top],
        "the help changed rows 0-4"
    );
}

/// Every stop on a write to the bank or mode register, with the I flag and whether the raster
/// interrupt was enabled, read from the monitor's `r` and `m $d01a` lines in VICE's log.
fn bank_writes(log: &str) -> Vec<(bool, bool)> {
    let mut out = Vec::new();
    let mut masked = None;
    for line in log.lines() {
        if let Some(rest) = line.trim_start().strip_prefix(".;") {
            // ".;ADDR A X Y SP 00 01 NV-BDIZC ...": the flags are the seventh field after ADDR.
            let flags = rest.split_whitespace().nth(7).unwrap_or("");
            masked = flags.as_bytes().get(5).map(|&b| b == b'1');
        } else if let Some(rest) = line.trim_start().strip_prefix(">C:d01a") {
            let byte = u8::from_str_radix(rest.split_whitespace().next().unwrap(), 16).unwrap();
            if let Some(m) = masked.take() {
                out.push((m, byte & 1 != 0));
            }
        }
    }
    out
}

// AC-127: nothing maps a bank in while the raster interrupt can run from the cart's ROM
#[test]
fn ac_127_no_bank_is_switched_while_the_raster_interrupt_is_live() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-127: skipping");
        return;
    }
    const STOPS: usize = 60;
    let k8 = markers::cart(CartKind::K8, "BANK WATCH CART");
    for (name, keys, grouped) in [
        (
            // F1 puts the cart first (A-Z), so DOWN lands on a program: RETURN starts it.
            "browse-then-program",
            vec![
                key::DOWN,
                key::DOWN,
                b'E',
                key::DEL,
                key::F1,
                key::DOWN,
                key::RETURN,
            ],
            false,
        ),
        ("cart", vec![key::RETURN], false),
        ("stop", vec![key::DOWN, key::STOP], false),
        // menu-keys (D-051): help and back, a page each way, programs only, a random pick, then
        // load it without running.
        (
            "keys-then-load",
            vec![
                key::F7,
                b'X',
                key::RIGHT,
                key::LEFT,
                key::F3,
                key::F5,
                key::SHIFT_RETURN,
            ],
            false,
        ),
        // menu-folders (D-057): a cart carrying metadata, "prog 1" and "prog 2" in a folder, which
        // is the first row. Open it, move, back out and in again, start "prog 2".
        (
            "folder-then-program",
            vec![
                key::RETURN,
                key::DOWN,
                key::DEL,
                key::RETURN,
                key::DOWN,
                key::RETURN,
            ],
            true,
        ),
    ] {
        let dir = scratch(&format!("ac127-{name}"));
        let mut inputs = vec![write(&dir, "k8.crt", &k8.bytes)];
        let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
        inputs.extend((0..3).map(|i| write(&dir, &format!("prog {i}.prg"), &hello)));
        let cart = build(&dir, "watch", &[], &inputs);
        if grouped {
            edit(
                &cart,
                &[
                    "--group", "2", "pair", "--group", "3", "pair", "--title", "3", "Two",
                ],
            );
        }
        let mut steps = vec![Step::Labels(vice::menu_labels()), idle(), Step::Keys(keys)];
        // The monitor's own log: a headless VICE splits what it prints to the terminal into one
        // "Monitor:" line per fragment, while this file reads the same in every build.
        steps.push(Step::Raw(
            "logname \"monlog.txt\"\nlog on\nwatch store $de00\nwatch store $de02".into(),
        ));
        for _ in 0..STOPS {
            steps.push(Step::Raw("g\nr\nm $d01a $d01a".into()));
        }
        // The run ends at the cycle limit once the writes stop; only the log is read.
        let _ = vice::boot(&dir, &cart, &steps);
        // The monitor's memory lines can end in raw bytes, so the log is not UTF-8.
        let log =
            String::from_utf8_lossy(&std::fs::read(dir.join("monlog.txt")).unwrap_or_default())
                .into_owned();
        let writes = bank_writes(&log);
        if writes.is_empty() {
            let vice =
                String::from_utf8_lossy(&std::fs::read(dir.join("vice.log")).unwrap_or_default())
                    .into_owned();
            let tail = |t: &str| t.lines().rev().take(40).collect::<Vec<_>>().join("\n");
            panic!(
                "{name}: no bank or mode write was seen\n-- monlog.txt, last lines reversed --\n{}\n-- vice.log, last lines reversed --\n{}",
                tail(&log),
                tail(&vice)
            );
        }
        for (i, (masked, live)) in writes.iter().enumerate() {
            assert!(
                *masked || !*live,
                "{name}: write {i} to $DE00/$DE02 with the raster interrupt live and unmasked"
            );
        }
    }
}

#[test]
fn the_bank_write_parser_reads_the_flags_and_the_interrupt_mask() {
    let log = "  ADDR A  X  Y  SP 00 01 NV-BDIZC LIN CYC  STOPWATCH\n\
               .;8d94 00 00 0a f3 2f 37 00100100 000 004    1234\n\
               >C:d01a  f1  .\n\
               .;8d98 00 00 0a f3 2f 37 00100000 000 004    1240\n\
               >C:d01a  f1  .\n";
    assert_eq!(bank_writes(log), vec![(true, true), (false, true)]);
}

// AC-128: every way out leaves the IRQ vector at $EA31 and the raster interrupt off
#[test]
fn ac_128_every_exit_puts_the_interrupts_back() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-128: skipping");
        return;
    }
    let dir = scratch("ac128");
    let prg = markers::basic_prg("IRQ PROGRAM RAN");
    let k8 = markers::cart(CartKind::K8, "IRQ CART RAN");
    let cart = build(
        &dir,
        "exits",
        &[],
        &[
            write(&dir, "a prog.prg", &prg.bytes),
            write(&dir, "b cart.crt", &k8.bytes),
        ],
    );
    for (keys, spin, what) in [
        (vec![key::RETURN], prg.spin, "the started program"),
        (vec![key::DOWN, key::RETURN], k8.spin, "the started cart"),
        (vec![key::STOP], BASIC_MAIN, "BASIC after RUN/STOP"),
        (
            vec![key::SHIFT_RETURN],
            KEY_WAIT,
            "a program loaded without running",
        ),
    ] {
        let mut steps = press_then_wait(&keys, spin);
        steps.push(Step::Dump(0x0314, 0x0315));
        steps.push(Step::Dump(0xd01a, 0xd01a));
        let dumps = boot_or_panic(&dir, &cart, &steps);
        assert_eq!(
            dumps[0],
            [0x31, 0xea],
            "{what}: the IRQ vector is not $EA31"
        );
        assert_eq!(
            dumps[1][0] & 0x0f,
            0,
            "{what}: a VIC interrupt is still enabled"
        );
    }
}

// --- menu-keys: load without running, paging, the ends, the kind filter, a random pick and help
// (AC-136..AC-141, D-051) --------------------------------------------------------------------------

/// The KERNAL's keyboard wait inside CHRIN (`$E5CD: LDA $C6 … BEQ $E5CD`), where BASIC sits at
/// `READY.` with nothing typed.
const KEY_WAIT: u16 = 0xe5cd;

/// The list rows (5-20) whose name column is shown reversed: the selection.
fn selected_rows(screen: &[u8]) -> Vec<usize> {
    LIST_ROWS
        .filter(|&n| screen[n * COLS + 4] & 0x80 != 0)
        .collect()
}

/// Assert `name` is the one selected entry, and return its screen row.
fn assert_selected(screen: &[u8], name: &str, what: &str) -> usize {
    let at = listed(screen, name).unwrap_or_else(|| panic!("{what}: '{name}' is not listed"));
    assert_eq!(
        selected_rows(screen),
        vec![at / COLS],
        "{what}: '{name}' is not the one selected entry"
    );
    at / COLS
}

/// The name in list row `n`, columns 4-19, trailing spaces dropped, as screen codes.
fn row_name(screen: &[u8], n: usize) -> Vec<u8> {
    let mut name: Vec<u8> = screen[n * COLS + 4..n * COLS + 20]
        .iter()
        .map(|b| b & 0x7f)
        .collect();
    while name.last() == Some(&0x20) {
        name.pop();
    }
    name
}

/// AC-136 (none): SHIFT+RETURN on a program loads it and stops at READY. with BASIC's pointers
/// set, so LIST and RUN work, and leaves the machine as RETURN does; on a cartridge it does nothing.
#[test]
fn ac_136_shift_return_loads_a_program_without_running_it() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-136: skipping");
        return;
    }
    let dir = scratch("ac136");
    let m = markers::basic_prg("LOADED PROGRAM RAN");
    let cart = build(&dir, "load only", &[], &[write(&dir, "prog.prg", &m.bytes)]);
    let load = 0x07f0u16;
    let end = load + m.bytes.len() as u16 - 2;
    let mut list = typed("list");
    list.push(key::RETURN);
    let mut run = typed("run");
    run.push(key::RETURN);
    let steps = vec![
        Step::Labels(vice::menu_labels()),
        idle(),
        Step::Keys(vec![key::SHIFT_RETURN]),
        Step::Break(Target::Label("basic_resume")),
        Step::Break(Target::Addr(KEY_WAIT)),
        screen(),
        Step::Dump(load, end - 1),
        Step::Dump(0x002b, 0x002e),
        Step::Dump(0x8000, 0x8001),
        Step::Dump(VIC.0, VIC.1),
        Step::Dump(TEXT_COLOUR.0, TEXT_COLOUR.1),
        Step::Dump(0x0314, 0x0315),
        Step::Dump(0xd01a, 0xd01a),
        Step::Keys(list),
        Step::Break(Target::Addr(BASIC_MAIN)),
        screen(),
        Step::Keys(run),
        Step::Break(Target::Addr(m.spin)),
        screen(),
    ];
    let d = boot_or_panic(&dir, &cart, &steps);

    let ready = vice::find(&d[0], &vice::screen_text("ready.")).expect("no READY. prompt");
    let next = ready / COLS + 1;
    assert!(
        row(&d[0], next).iter().all(|b| b & 0x7f == 0x20),
        "something was typed after READY."
    );
    assert!(!shows(&d[0], &m.text), "the program ran");
    assert_eq!(d[1], m.bytes[2..], "the program is not in memory at $07F0");
    assert_eq!(
        d[2],
        [0x01, 0x08, end as u8, (end >> 8) as u8],
        "BASIC's start and end pointers"
    );
    assert_ne!(
        d[3].as_slice(),
        &stidcart_core::menu::STIDCART_MENU[..2],
        "the cart is still mapped at $8000"
    );
    assert_eq!(
        (d[4][8] & 0x0f, d[4][9] & 0x0f, d[5][0] & 0x0f, d[4][0] | 1),
        (
            colour::LIGHT_BLUE,
            colour::BLUE,
            colour::LIGHT_BLUE,
            UPPER_CASE
        ),
        "(border, background, text colour, $D018)"
    );
    assert_eq!(d[6], [0x31, 0xea], "the IRQ vector is not $EA31");
    assert_eq!(d[7][0] & 0x0f, 0, "a VIC interrupt is still enabled");
    assert!(shows(&d[8], "10 sys2061"), "LIST does not show the program");
    assert_marker(&d[9], &m);

    // A cartridge: nothing changes, and RETURN still starts it afterwards.
    let dir = scratch("ac136-cart");
    let k8 = markers::cart(CartKind::K8, "CART AFTER SHIFT RETURN");
    let cart = build(
        &dir,
        "load cart",
        &[],
        &[
            write(&dir, "k8.crt", &k8.bytes),
            write(&dir, "prog.prg", &m.bytes),
        ],
    );
    let d = boot_or_panic(
        &dir,
        &cart,
        &[
            Step::Labels(vice::menu_labels()),
            idle(),
            screen(),
            Step::Keys(vec![key::SHIFT_RETURN]),
            idle(),
            screen(),
            Step::Keys(vec![key::RETURN]),
            Step::Break(Target::Addr(k8.spin)),
            screen(),
        ],
    );
    assert_eq!(d[0], d[1], "SHIFT+RETURN on a cartridge changed the screen");
    assert_marker(&d[2], &k8);
}

/// AC-137 (none): CRSR → and CRSR ← move the selection 16 entries, stopping at the ends, and scroll
/// the view only as far as it takes to show it; on a list that fits they reach the last and first.
#[test]
fn ac_137_crsr_right_and_left_move_a_page() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-137: skipping");
        return;
    }
    let dir = scratch("ac137");
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let inputs: Vec<PathBuf> = (1..=40)
        .map(|i| write(&dir, &format!("e{i:02}.prg"), &hello))
        .collect();
    let cart = build(&dir, "pages", &[], &inputs);
    let (first, last) = (LIST_ROWS.start, LIST_ROWS.end - 1);

    let d = screens_after(
        &dir,
        &cart,
        &[
            vec![key::RIGHT],
            vec![key::RIGHT],
            vec![key::RIGHT],
            vec![key::LEFT],
            vec![key::LEFT],
            vec![key::LEFT],
        ],
    );
    for (s, name, at, pos, what) in [
        (&d[0], "e17", last, "17 of 40", "one page down"),
        (&d[1], "e33", last, "33 of 40", "two pages down"),
        (&d[2], "e40", last, "40 of 40", "past the end"),
        (&d[3], "e24", first, "24 of 40", "one page up from the end"),
        (&d[4], "e08", first, "8 of 40", "two pages up"),
        (&d[5], "e01", first, "1 of 40", "past the start"),
    ] {
        assert_eq!(assert_selected(s, name, what), at, "{what}: '{name}' row");
        assert!(row_shows(s, STATUS_ROW, pos), "{what}: position");
    }

    let few_dir = scratch("ac137-few");
    let few: Vec<PathBuf> = ["a", "b", "c"]
        .iter()
        .map(|n| write(&few_dir, &format!("{n}.prg"), &hello))
        .collect();
    let few_cart = build(&few_dir, "few", &[], &few);
    let d = screens_after(&few_dir, &few_cart, &[vec![key::RIGHT], vec![key::LEFT]]);
    assert_selected(&d[0], "c", "a short list, CRSR right");
    assert_selected(&d[1], "a", "a short list, CRSR left");
}

/// AC-138 (none): with filter text HOME and CLR clear it; without, HOME selects the first entry and
/// CLR the last, scrolling to show it.
#[test]
fn ac_138_home_and_clr_go_to_the_ends_without_text() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-138: skipping");
        return;
    }
    let dir = scratch("ac138");
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let inputs: Vec<PathBuf> = (1..=40)
        .map(|i| write(&dir, &format!("e{i:02}.prg"), &hello))
        .collect();
    let cart = build(&dir, "ends", &[], &inputs);
    let blank = |s: &[u8]| row(s, QUERY_ROW).iter().all(|b| b & 0x7f == 0x20);
    let (first, last) = (LIST_ROWS.start, LIST_ROWS.end - 1);

    let mut clr = typed("e1");
    clr.push(key::CLR);
    let mut home = typed("e2");
    home.push(key::HOME);
    let d = screens_after(
        &dir,
        &cart,
        &[
            vec![key::CLR],
            vec![key::HOME],
            vec![key::DOWN, key::DOWN, key::DOWN, key::HOME],
            clr,
            vec![key::CLR],
            home,
        ],
    );
    for (s, name, at, pos, what) in [
        (&d[0], "e40", last, "40 of 40", "CLR"),
        (&d[1], "e01", first, "1 of 40", "HOME"),
        (&d[2], "e01", first, "1 of 40", "three DOWNs, HOME"),
        (&d[3], "e01", first, "1 of 40", "text, CLR"),
        (
            &d[4],
            "e40",
            last,
            "40 of 40",
            "CLR after the text was cleared",
        ),
        (&d[5], "e01", first, "1 of 40", "text, HOME"),
    ] {
        assert_eq!(assert_selected(s, name, what), at, "{what}: '{name}' row");
        assert!(row_shows(s, STATUS_ROW, pos), "{what}: position");
        assert!(blank(s), "{what}: the text row is not empty");
    }
}

/// AC-139 (none): F3 steps the kind filter all → programs → cartridges → all, combined with the
/// text filter; row 22 names the kind, the count counts what is listed, and the first match is
/// selected. A kind with no match says so.
#[test]
fn ac_139_f3_filters_by_kind() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-139: skipping");
        return;
    }
    let dir = scratch("ac139");
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let k8 = markers::cart(CartKind::K8, "GAMMA");
    let k16 = markers::cart(CartKind::K16, "DELTA");
    let inputs = [
        write(&dir, "alpha.prg", &hello),
        write(&dir, "gamma.crt", &k8.bytes),
        write(&dir, "beta.prg", &hello),
        write(&dir, "delta.crt", &k16.bytes),
    ];
    let cart = build(&dir, "kinds", &[], &inputs);
    let all = ["alpha", "gamma", "beta", "delta"];

    let mut l_then_f3 = typed("l");
    l_then_f3.push(key::F3);
    let mut gam = vec![key::DEL];
    gam.extend(typed("gam"));
    gam.extend([key::F3, key::F3]);
    let d = screens_after(
        &dir,
        &cart,
        &[
            vec![],
            vec![key::F3],
            vec![key::DOWN, key::F3],
            vec![key::F3],
            l_then_f3,
            vec![key::F3],
            gam,
        ],
    );

    for (s, listed_names, hidden, kind, pos, what) in [
        (&d[0], &all[..], &[][..], "all kinds", "1 of 4", "boot"),
        (
            &d[1],
            &["alpha", "beta"][..],
            &["gamma", "delta"][..],
            "programs",
            "1 of 2",
            "F3",
        ),
        (
            &d[2],
            &["gamma", "delta"][..],
            &["alpha", "beta"][..],
            "cartridges",
            "1 of 2",
            "DOWN, F3",
        ),
        (
            &d[3],
            &all[..],
            &[][..],
            "all kinds",
            "1 of 4",
            "F3 three times",
        ),
        (
            &d[4],
            &["alpha"][..],
            &["gamma", "beta", "delta"][..],
            "programs",
            "1 of 1",
            "l, F3",
        ),
        (
            &d[5],
            &["delta"][..],
            &["alpha", "gamma", "beta"][..],
            "cartridges",
            "1 of 1",
            "l, F3 twice",
        ),
    ] {
        assert_listed_in_order(s, listed_names, what);
        assert_not_listed(s, hidden, what);
        assert_selected(s, listed_names[0], what);
        assert!(
            row_shows(s, STATUS_ROW, &format!("{pos}   cart order   {kind}")),
            "{what}: row 22 does not show '{pos}   cart order   {kind}'"
        );
    }
    for (s, what) in [(&d[4], "l, F3"), (&d[5], "l, F3 twice")] {
        assert!(
            row_shows(s, QUERY_ROW, "1 of 4 match"),
            "{what}: the match count does not count what is listed"
        );
    }

    let s = &d[6];
    assert!(
        listed(s, "no matches").is_some(),
        "gam among the programs: no 'no matches'"
    );
    assert_not_listed(s, &all, "gam among the programs");
    assert!(row_shows(s, STATUS_ROW, "programs"), "gam: the kind");
    assert!(row_shows(s, QUERY_ROW, "0 of 4 match"), "gam: the count");
}

/// AC-140 (none): F5 selects a random entry among those listed, both filters applied, shows it and
/// starts nothing; repeated presses choose different entries. With one entry or none listed it
/// changes nothing.
#[test]
fn ac_140_f5_picks_a_random_listed_entry() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-140: skipping");
        return;
    }
    let dir = scratch("ac140");
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let k8 = markers::cart(CartKind::K8, "ITEM CART");
    let mut inputs: Vec<PathBuf> = (0..24)
        .map(|i| write(&dir, &format!("item {i:02}.prg"), &hello))
        .collect();
    inputs.push(write(&dir, "item cart.crt", &k8.bytes));
    inputs.push(write(&dir, "other a.prg", &hello));
    inputs.push(write(&dir, "other b.prg", &hello));
    let cart = build(&dir, "random", &[], &inputs);
    const PRESSES: usize = 8;

    let mut set_up = typed("item");
    set_up.push(key::F3);
    let mut groups = vec![set_up];
    groups.extend((0..PRESSES).map(|_| vec![key::F5]));
    let d = screens_after(&dir, &cart, &groups);
    let names: Vec<Vec<u8>> = (0..24)
        .map(|i| vice::screen_text(&format!("item {i:02}")))
        .collect();
    assert_selected(&d[0], "item 00", "item, F3");
    let mut picked = Vec::new();
    for (i, s) in d[1..].iter().enumerate() {
        let what = format!("F5 press {}", i + 1);
        let rows = selected_rows(s);
        assert_eq!(rows.len(), 1, "{what}: not exactly one entry selected");
        let name = row_name(s, rows[0]);
        let k = names
            .iter()
            .position(|n| *n == name)
            .unwrap_or_else(|| panic!("{what}: the selection is not a listed program"));
        assert!(
            row_shows(s, STATUS_ROW, &format!("{} of 24", k + 1)),
            "{what}: the position does not match the selection"
        );
        assert!(
            listed(s, "item cart").is_none(),
            "{what}: a cartridge is listed"
        );
        assert!(
            listed(s, "other").is_none(),
            "{what}: a non-match is listed"
        );
        picked.push(k);
    }
    picked.sort_unstable();
    picked.dedup();
    assert!(picked.len() >= 3, "{PRESSES} presses chose only {picked:?}");

    let mut none = vec![key::CLR];
    none.extend(typed("zz"));
    let d = screens_after(
        &dir,
        &cart,
        &[typed("other a"), vec![key::F5], none, vec![key::F5]],
    );
    assert!(listed(&d[0], "other a").is_some(), "other a: not listed");
    assert_eq!(d[0], d[1], "F5 with one entry listed changed the screen");
    assert!(listed(&d[2], "no matches").is_some(), "zz: no 'no matches'");
    assert_eq!(d[2], d[3], "F5 with nothing listed changed the screen");
}

/// AC-141 (none): F7 shows a screen naming every key; the next key returns to the menu as it was
/// and does nothing else. Row 24's help bar names F7.
#[test]
fn ac_141_f7_shows_help_and_any_key_returns() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-141: skipping");
        return;
    }
    let dir = scratch("ac141");
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let k8 = markers::cart(CartKind::K8, "HELP CART");
    let inputs = [
        write(&dir, "bravo.prg", &hello),
        write(&dir, "alpha.prg", &hello),
        write(&dir, "banana.crt", &k8.bytes),
        write(&dir, "abba.prg", &hello),
        write(&dir, "kiwi.prg", &hello),
    ];
    let cart = build(&dir, "help", &[], &inputs);

    let mut set_up = typed("a");
    set_up.extend([key::F1, key::F3, key::DOWN]);
    let looks = looks_after(
        &dir,
        &cart,
        &[
            set_up,
            vec![key::F7],
            typed("x"),
            vec![key::F7],
            vec![key::RETURN],
            vec![key::F7],
            vec![key::DOWN],
        ],
    );
    let before = &looks[0];
    assert_listed_in_order(&before.screen, &["abba", "alpha", "bravo"], "the set-up");
    assert_not_listed(&before.screen, &["banana", "kiwi"], "the set-up");
    assert_selected(&before.screen, "alpha", "the set-up");
    assert!(
        row_shows(&before.screen, HELP_ROW, "f7"),
        "the help bar does not name F7"
    );

    let help = &looks[1].screen;
    assert!(
        listed(help, "alpha").is_none(),
        "F7: the list is still shown"
    );
    let body = &help[LIST_ROWS.start * COLS..];
    for k in [
        "crsr",
        "return",
        "shift+return",
        "letters",
        "del",
        "home",
        "clr",
        "f1",
        "f3",
        "f5",
        "f7",
        "run/stop",
    ] {
        assert!(
            vice::find(body, &vice::screen_text(k)).is_some(),
            "F7: the help does not name '{k}'"
        );
    }
    assert_eq!(&looks[3].screen, help, "F7 a second time: a different help");

    for (i, what) in [(2, "x"), (4, "RETURN"), (6, "DOWN")] {
        let after = &looks[i];
        assert!(
            after.screen == before.screen,
            "{what} after F7: the menu did not come back as it was"
        );
        for c in 0..1000 {
            if before.screen[c] != 0x20 {
                assert_eq!(
                    after.colour[c], before.colour[c],
                    "{what} after F7: the colour of cell {c} changed"
                );
            }
        }
    }
}

// --- menu-joystick: a stick in port 2 (AC-142..AC-146, D-052) ---------------------------------------
//
// The stick is VICE's "Joyport I/O simulation" device, set from the monitor with `jpdb`, which CIA1
// reads as it reads a real stick. Its lines stay as set while warp runs any number of frames, so the
// clock here is the frame: the raster interrupt enters `irq` once a frame (D-048), and `frames(n)`
// stops at its n-th entry. The menu reaches `menu_idle` only with the keyboard buffer empty and the
// stick seen at rest, so a release followed by `idle()` is a release the menu has read.

/// Run `n` frames.
fn frames(n: usize) -> Step {
    Step::Raw(format!("break exec .irq\n{}del", "g\n".repeat(n)))
}

/// Press `lines` for `n` frames, release, and wait until the menu has seen the release.
fn hold(lines: u8, n: usize) -> Vec<Step> {
    vec![
        Step::Stick(lines),
        frames(n),
        Step::Stick(stick::REST),
        idle(),
    ]
}

/// Boot with the stick device in port 2, its lines set to `first` before anything runs, then run
/// `steps` with the menu's labels loaded.
fn boot_stick(dir: &Path, cart: &Path, first: u8, steps: Vec<Step>) -> Vec<Vec<u8>> {
    let mut all = vec![Step::Stick(first), Step::Labels(vice::menu_labels())];
    all.extend(steps);
    boot_or_panic_with(dir, cart, &vice::STICK_DEVICE, &all)
}

/// A cart of programs named e01, e02, … in that order.
fn numbered_cart(dir: &Path, n: usize) -> PathBuf {
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let inputs: Vec<PathBuf> = (1..=n)
        .map(|i| write(dir, &format!("e{i:02}.prg"), &hello))
        .collect();
    build(dir, "stick", &[], &inputs)
}

/// The position row 22 starts with: the selection's number among those listed.
fn position(screen: &[u8]) -> usize {
    let digits: String = row(screen, STATUS_ROW)[1..]
        .iter()
        .take_while(|b| (0x30..=0x39).contains(*b))
        .map(|&b| b as char)
        .collect();
    digits
        .parse()
        .unwrap_or_else(|_| panic!("row 22 does not start with a position"))
}

/// AC-142 (none): with the stick at rest, down moves the selection one entry down, up one back, and
/// right and left a page, as CRSR → and CRSR ← do (D-051).
#[test]
fn ac_142_the_stick_moves_the_selection_as_the_cursor_keys_do() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-142: skipping");
        return;
    }
    let dir = scratch("ac142");
    let cart = numbered_cart(&dir, 40);
    let mut steps = vec![idle()];
    for lines in [
        stick::DOWN,
        stick::DOWN,
        stick::UP,
        stick::RIGHT,
        stick::LEFT,
    ] {
        // Three frames: well inside the first delay, so one press is one move.
        steps.extend(hold(lines, 3));
        steps.push(screen());
    }
    let d = boot_stick(&dir, &cart, stick::REST, steps);
    let (first, last) = (LIST_ROWS.start, LIST_ROWS.end - 1);
    for (s, name, pos, what) in [
        (&d[0], "e02", "2 of 40", "down"),
        (&d[1], "e03", "3 of 40", "down again"),
        (&d[2], "e02", "2 of 40", "up"),
        (&d[3], "e18", "18 of 40", "right: a page down"),
        (&d[4], "e02", "2 of 40", "left: a page up"),
    ] {
        assert_selected(s, name, what);
        assert!(
            row_shows(s, STATUS_ROW, pos),
            "{what}: the position is not '{pos}'"
        );
    }
    assert_eq!(assert_selected(&d[3], "e18", "right"), last, "right: row");
    assert_eq!(assert_selected(&d[4], "e02", "left"), first, "left: row");
}

/// AC-143 (none): held down, the stick moves the selection more than one entry, about ten a second
/// after a first delay of about 0.4 s (D-052), and the selection stops once the stick is released.
#[test]
fn ac_143_a_held_stick_repeats_and_stops_when_released() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-143: skipping");
        return;
    }
    let dir = scratch("ac143");
    let cart = numbered_cart(&dir, 40);
    let mut steps = vec![idle()];
    steps.extend(hold(stick::DOWN, 60));
    steps.push(screen());
    steps.push(frames(60));
    steps.push(idle());
    steps.push(screen());
    let d = boot_stick(&dir, &cart, stick::REST, steps);

    let at = position(&d[0]);
    assert_selected(&d[0], &format!("e{at:02}"), "held for 60 frames");
    // A press, then repeats from 20 frames on, one every 5: nine moves in 60 frames. A repeat every
    // loop or every frame would reach the end; no repeat would leave one move.
    let moves = at - 1;
    assert!(
        (5..=12).contains(&moves),
        "held for 60 frames (1.2 s), the selection moved {moves} entries, not about nine"
    );
    assert_eq!(
        d[0], d[1],
        "the selection moved after the stick was released"
    );
}

/// AC-144 (none): fire starts the selected program or cartridge as RETURN does, leaving AC-128's exit
/// state (IRQ vector at $EA31, raster interrupt off). Fire held while the menu starts does nothing
/// until it is released and pressed again.
#[test]
fn ac_144_fire_starts_the_selection_and_a_held_fire_waits_for_a_release() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-144: skipping");
        return;
    }
    let dir = scratch("ac144");
    let prg = markers::basic_prg("FIRE PROGRAM RAN");
    let k8 = markers::cart(CartKind::K8, "FIRE CART RAN");
    let cart = build(
        &dir,
        "fire",
        &[],
        &[
            write(&dir, "a prog.prg", &prg.bytes),
            write(&dir, "b cart.crt", &k8.bytes),
        ],
    );

    // Fire held from power-on: 40 frames of the menu, then release, then press.
    let d = boot_stick(
        &dir,
        &cart,
        stick::FIRE,
        vec![
            Step::Break(Target::Label("loop")),
            frames(40),
            screen(),
            Step::Stick(stick::REST),
            idle(),
            Step::Stick(stick::FIRE),
            Step::Break(Target::Addr(prg.spin)),
            screen(),
            Step::Dump(0x0314, 0x0315),
            Step::Dump(0xd01a, 0xd01a),
        ],
    );
    assert!(
        !shows(&d[0], &prg.text),
        "fire held at start started the program"
    );
    assert_selected(&d[0], "a prog", "fire held at start");
    assert_marker(&d[1], &prg);
    assert_eq!(d[2], [0x31, 0xea], "fire on a program: the IRQ vector");
    assert_eq!(d[3][0] & 0x0f, 0, "fire on a program: a VIC interrupt");

    // A cartridge, reached with the stick too.
    let mut steps = vec![idle()];
    steps.extend(hold(stick::DOWN, 3));
    steps.extend([
        Step::Stick(stick::FIRE),
        Step::Break(Target::Addr(k8.spin)),
        screen(),
        Step::Dump(0x0314, 0x0315),
        Step::Dump(0xd01a, 0xd01a),
    ]);
    let d = boot_stick(&dir, &cart, stick::REST, steps);
    assert_marker(&d[0], &k8);
    assert_eq!(d[1], [0x31, 0xea], "fire on a cartridge: the IRQ vector");
    assert_eq!(d[2][0] & 0x0f, 0, "fire on a cartridge: a VIC interrupt");
}

/// AC-145 (none): on the help screen (F7), a stick move or fire returns to the menu as it was, and
/// that input does nothing else, however long it is held.
#[test]
fn ac_145_the_stick_leaves_the_help_and_does_nothing_else() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-145: skipping");
        return;
    }
    let dir = scratch("ac145");
    let cart = numbered_cart(&dir, 40);
    let look = || [screen(), Step::Dump(COLOUR_RAM.0, COLOUR_RAM.1)];
    let mut steps = vec![idle()];
    steps.extend(hold(stick::DOWN, 3));
    steps.extend(look());
    let inputs = [stick::DOWN, stick::FIRE, stick::RIGHT, stick::UP];
    for lines in inputs {
        steps.push(Step::Keys(vec![key::F7]));
        steps.push(idle());
        steps.push(screen());
        // 40 frames: past the first delay and several repeats.
        steps.extend(hold(lines, 40));
        steps.extend(look());
    }
    let d = boot_stick(&dir, &cart, stick::REST, steps);
    let (before, colours) = (&d[0], &d[1]);
    assert_selected(before, "e02", "the set-up");
    for (i, lines) in inputs.iter().enumerate() {
        let what = format!("stick {lines:02x} on the help");
        let help = &d[2 + i * 3];
        assert!(listed(help, "e02").is_none(), "{what}: F7 showed no help");
        let (after, after_colours) = (&d[3 + i * 3], &d[4 + i * 3]);
        assert!(
            after == before,
            "{what}: the menu did not come back as it was"
        );
        for c in 0..1000 {
            if before[c] != 0x20 {
                // Colour RAM holds four bits; the rest of what a read returns is open bus.
                assert_eq!(
                    after_colours[c] & 0x0f,
                    colours[c] & 0x0f,
                    "{what}: the colour of cell {c} changed"
                );
            }
        }
    }
}

/// AC-146 (none): with the stick at rest in port 2 the keyboard works as before, and AC-127 holds
/// while the stick is used: no write to $DE00/$DE02 with the raster interrupt live and unmasked.
#[test]
fn ac_146_the_keyboard_works_beside_the_stick_and_no_bank_moves_under_the_interrupt() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-146: skipping");
        return;
    }
    let dir = scratch("ac146");
    let cart = numbered_cart(&dir, 40);
    let groups = [
        vec![key::DOWN, key::DOWN],
        vec![key::RIGHT],
        typed("e1"),
        vec![key::F1],
        vec![key::F7],
        vec![key::RETURN],
        vec![key::CLR],
    ];
    let without = screens_after(&dir, &cart, &groups);
    let with = boot_stick(&dir, &cart, stick::REST, group_steps(&groups));
    assert_selected(&with[0], "e03", "two DOWNs beside the stick");
    assert_selected(&with[1], "e19", "CRSR right beside the stick");
    assert_eq!(
        with, without,
        "the keyboard does not do with the stick at rest what it does without one"
    );

    // AC-127's watch, with the stick browsing onto programs (each one's pane maps its bank in) and
    // fire starting one. A breakpoint step deletes every checkpoint, so after the watch only g, r, m
    // and jpdb run: the first four bank writes are the press and the first repeat, then fire.
    let dir = scratch("ac146-watch");
    let k8 = markers::cart(CartKind::K8, "STICK WATCH CART");
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let mut inputs = vec![write(&dir, "k8.crt", &k8.bytes)];
    inputs.extend((0..5).map(|i| write(&dir, &format!("prog {i}.prg"), &hello)));
    let cart = build(&dir, "watch", &[], &inputs);
    let stop = || Step::Raw("g\nr\nm $d01a $d01a".into());
    let mut steps = vec![
        Step::Stick(stick::REST),
        Step::Labels(vice::menu_labels()),
        idle(),
        Step::Stick(stick::DOWN),
        Step::Raw("logname \"monlog.txt\"\nlog on\nwatch store $de00\nwatch store $de02".into()),
    ];
    steps.extend((0..4).map(|_| stop()));
    steps.push(Step::Stick(stick::FIRE));
    steps.extend((0..56).map(|_| stop()));
    // The run ends at the cycle limit once the writes stop; only the log is read.
    let _ = vice::boot_with(&dir, &cart, &vice::STICK_DEVICE, &steps);
    let log = String::from_utf8_lossy(&std::fs::read(dir.join("monlog.txt")).unwrap_or_default())
        .into_owned();
    let writes = bank_writes(&log);
    // The moves' four, then the start's: its load address read, the mode set for BASIC, the trap's
    // copy and the hand-over.
    assert!(
        writes.len() >= 10,
        "only {} bank or mode writes: the stick did not browse and start a program",
        writes.len()
    );
    for (i, (masked, live)) in writes.iter().enumerate() {
        assert!(
            *masked || !*live,
            "write {i} to $DE00/$DE02 with the raster interrupt live and unmasked"
        );
    }
}

// --- menu-resume: after a reset the menu comes back where it was left (AC-147..AC-152, D-053) -------
//
// A soft reset is the monitor's `reset 0`, which keeps the EasyFlash RAM page at $DF00 as the RESET
// button does; `reset 1` is a power cycle, which refills it with VICE's power-on pattern. `reset`
// resumes execution itself, so its step sets the breakpoint first and sends no `g`.

/// Where the menu keeps its state (D-053).
const STATE: (u16, u16) = (0xdff6, 0xdfff);

fn state() -> Step {
    Step::Dump(STATE.0, STATE.1)
}

/// Soft-reset and stop when the menu is idle again.
fn soft_reset() -> Step {
    Step::Raw("break exec .menu_idle\nreset 0\ndel".into())
}

/// Power-cycle and stop when the menu is idle again.
fn hard_reset() -> Step {
    Step::Raw("break exec .menu_idle\nreset 1\ndel".into())
}

/// The cart the resume tests share, 33 entries in this directory order: two 8K cartridges, an xbank
/// one, a marker program, then p01..p29 (hello.prg). A fresh boot selects "k cart a". `p99` names
/// the marker program and `extra` appends a 34th entry: both make a cart whose directory differs.
struct ResumeCart {
    cart: PathBuf,
    b: Marker,
    x: Marker,
    prog: Marker,
}

fn resume_cart(dir: &Path, p99: &str, extra: bool) -> ResumeCart {
    let a = markers::cart(CartKind::K8, "CART A RAN");
    let b = markers::cart(CartKind::K8, "CART B RAN");
    let x = markers::cart(CartKind::Xbank8, "XBANK CART RAN");
    let prog = markers::basic_prg("P99 PROG RAN");
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let mut inputs = vec![
        write(dir, "k cart a.crt", &a.bytes),
        write(dir, "k cart b.crt", &b.bytes),
        write(dir, "x cart.crt", &x.bytes),
        write(dir, &format!("{p99}.prg"), &prog.bytes),
    ];
    inputs.extend((1..=29).map(|i| write(dir, &format!("p{i:02}.prg"), &hello)));
    if extra {
        inputs.push(write(dir, "p30.prg", &hello));
    }
    let cart = build(dir, "resume", &[], &inputs);
    ResumeCart { cart, b, x, prog }
}

/// AC-147's first start: A-Z, programs only, text typed, two pages down to "p99 prog" (the 30th of
/// 30 listed, so a fresh boot's first page can't show it), RETURN.
fn start_p99() -> Vec<u8> {
    let mut keys = vec![key::F1, key::F3];
    keys.extend(typed("p"));
    keys.extend([key::RIGHT, key::RIGHT, key::RETURN]);
    keys
}

/// A-Z, cartridges only, text typed, down to "k cart b", RETURN.
fn start_cart_b() -> Vec<u8> {
    let mut keys = vec![key::F1, key::F3, key::F3];
    keys.extend(typed("k"));
    keys.extend([key::DOWN, key::RETURN]);
    keys
}

/// Cartridges only, in cart order, down to "x cart", RETURN.
fn start_xbank() -> Vec<u8> {
    vec![key::F3, key::F3, key::DOWN, key::DOWN, key::RETURN]
}

/// The screen a boot without state shows on the resume cart.
fn assert_fresh(s: &[u8], what: &str) {
    assert_selected(s, "k cart a", what);
    assert!(
        row_shows(s, STATUS_ROW, "1 of 33   cart order   all kinds"),
        "{what}: the menu did not start as it does without state"
    );
}

/// `name` selected and in view, row 22 reading `status`, and no filter text.
fn assert_resumed(s: &[u8], name: &str, status: &str, what: &str) {
    assert_selected(s, name, what);
    assert!(
        row_shows(s, STATUS_ROW, status),
        "{what}: row 22 does not read '{status}'"
    );
    assert!(
        row(s, QUERY_ROW).iter().all(|b| b & 0x7f == 0x20),
        "{what}: the filter text was kept"
    );
}

/// Boot `cart`, stop at `entry` (the menu's first instruction, before it reads any state), write
/// `bytes` into $DFF6-$DFFF from the monitor, and return the screen once the menu is idle.
fn boot_with_state(dir: &Path, cart: &Path, bytes: &[u8]) -> Vec<u8> {
    let hex: Vec<String> = bytes.iter().map(|b| format!("{b:02x}")).collect();
    boot_or_panic(
        dir,
        cart,
        &[
            Step::Labels(vice::menu_labels()),
            Step::Break(Target::Label("entry")),
            Step::Raw(format!("> ${:04x} {}", STATE.0, hex.join(" "))),
            idle(),
            screen(),
        ],
    )
    .remove(0)
}

/// AC-147 (none): an entry started from the menu (a program and a cartridge by RETURN, an xbank
/// cartridge, a program by SHIFT+RETURN) is selected and in view after a soft reset, with the order
/// and kind filter it was started under and no filter text. Each start types text or moves to an
/// entry a fresh boot doesn't select, and the order or the kind filter differs from a fresh boot's.
#[test]
fn ac_147_a_soft_reset_comes_back_to_the_entry_started() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-147: skipping");
        return;
    }
    let dir = scratch("ac147");
    let r = resume_cart(&dir, "p99 prog", false);
    let mut shift = vec![key::F3];
    shift.extend(typed("p2"));
    shift.extend([key::DOWN, key::DOWN, key::DOWN, key::SHIFT_RETURN]);
    for (keys, end, ran, name, status, what) in [
        (
            start_p99(),
            r.prog.spin,
            Some(&r.prog),
            "p99 prog",
            "30 of 30   a-z order   programs",
            "a program by RETURN",
        ),
        (
            start_cart_b(),
            r.b.spin,
            Some(&r.b),
            "k cart b",
            "2 of 3   a-z order   cartridges",
            "a cartridge by RETURN",
        ),
        (
            start_xbank(),
            r.x.spin,
            Some(&r.x),
            "x cart",
            "3 of 3   cart order   cartridges",
            "an xbank cartridge by RETURN",
        ),
        (
            shift,
            KEY_WAIT,
            None,
            "p23",
            "24 of 30   cart order   programs",
            "a program by SHIFT+RETURN",
        ),
    ] {
        let d = boot_or_panic(
            &dir,
            &r.cart,
            &[
                Step::Labels(vice::menu_labels()),
                idle(),
                Step::Keys(keys),
                Step::Break(Target::Addr(end)),
                screen(),
                soft_reset(),
                screen(),
            ],
        );
        match ran {
            Some(m) => assert_marker(&d[0], m),
            None => assert!(shows(&d[0], "ready."), "{what}: no READY. prompt"),
        }
        assert_resumed(&d[1], name, status, what);
    }
}

/// AC-148 (none): RUN/STOP, then a soft reset, brings back the entry that was selected, its order
/// and its kind filter.
#[test]
fn ac_148_run_stop_then_a_soft_reset_comes_back_to_the_selection() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-148: skipping");
        return;
    }
    let dir = scratch("ac148");
    let r = resume_cart(&dir, "p99 prog", false);
    let mut keys = vec![key::F1, key::F3];
    keys.extend(typed("p1"));
    keys.extend([key::DOWN, key::DOWN, key::DOWN, key::DOWN, key::STOP]);
    let d = boot_or_panic(
        &dir,
        &r.cart,
        &[
            Step::Labels(vice::menu_labels()),
            idle(),
            Step::Keys(keys),
            Step::Break(Target::Addr(BASIC_MAIN)),
            screen(),
            soft_reset(),
            screen(),
        ],
    );
    assert!(shows(&d[0], "ready."), "RUN/STOP did not leave to BASIC");
    assert_resumed(&d[1], "p14", "14 of 30   a-z order   programs", "RUN/STOP");
}

/// AC-149 (none): the menu boots as it does without state after a power cycle (the real `reset 1`),
/// after any one of the ten bytes at $DFF6-$DFFF is changed, and on a cart the state wasn't written
/// for.
///
/// Every refusal sits beside the same set-up restoring, so none can pass because nothing is
/// restored at all:
/// - power cycle: one run starts "p99 prog", soft-resets (restores), then power-cycles. The state
///   bytes must change and the menu must start fresh.
/// - one byte changed: one run starts "p99 prog", then for each of the ten bytes flips its low bit
///   from the monitor and soft-resets (fresh), then writes it back and soft-resets (restores).
/// - another cart: the ten bytes cart A wrote are written from the monitor into a boot at `entry`,
///   before the menu reads them. In cart A's own boot they restore. In a cart with one more entry,
///   or with the selected slot's name changed ("p99 prog" to "p98 prog", the same count), the menu
///   starts fresh.
#[test]
fn ac_149_a_power_cycle_a_changed_byte_or_another_cart_starts_fresh() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-149: skipping");
        return;
    }
    let dir = scratch("ac149");
    let r = resume_cart(&dir, "p99 prog", false);
    let resumed = "30 of 30   a-z order   programs";
    let started = |more: Vec<Step>| {
        let mut steps = vec![
            Step::Labels(vice::menu_labels()),
            idle(),
            Step::Keys(start_p99()),
            Step::Break(Target::Addr(r.prog.spin)),
            state(),
        ];
        steps.extend(more);
        steps
    };

    let d = boot_or_panic(
        &dir,
        &r.cart,
        &started(vec![
            soft_reset(),
            screen(),
            hard_reset(),
            screen(),
            state(),
        ]),
    );
    let written = d[0].clone();
    assert_resumed(
        &d[1],
        "p99 prog",
        resumed,
        "the soft reset before the power cycle",
    );
    assert_ne!(
        d[3], written,
        "the power cycle left the state bytes as they were"
    );
    assert_fresh(&d[2], "after a power cycle");

    let mut more = Vec::new();
    for (i, &b) in written.iter().enumerate() {
        let at = STATE.0 + i as u16;
        more.push(Step::Raw(format!("> ${at:04x} {:02x}", b ^ 1)));
        more.push(soft_reset());
        more.push(screen());
        more.push(Step::Raw(format!("> ${at:04x} {b:02x}")));
        more.push(soft_reset());
        more.push(screen());
    }
    let d = boot_or_panic(&dir, &r.cart, &started(more));
    assert_eq!(
        d[0], written,
        "the state differs between two identical runs"
    );
    for i in 0..written.len() {
        let at = STATE.0 + i as u16;
        assert_fresh(&d[1 + i * 2], &format!("${at:04X} changed"));
        assert_resumed(
            &d[2 + i * 2],
            "p99 prog",
            resumed,
            &format!("${at:04X} written back"),
        );
    }

    assert_resumed(
        &boot_with_state(&dir, &r.cart, &written),
        "p99 prog",
        resumed,
        "cart A's state written into its own boot",
    );
    let count_dir = scratch("ac149-count");
    let one_more = resume_cart(&count_dir, "p99 prog", true);
    let s = boot_with_state(&count_dir, &one_more.cart, &written);
    assert_selected(&s, "k cart a", "a cart with one more entry");
    assert!(
        row_shows(&s, STATUS_ROW, "1 of 34   cart order   all kinds"),
        "a cart with one more entry: the menu did not start fresh"
    );
    let name_dir = scratch("ac149-name");
    let renamed = resume_cart(&name_dir, "p98 prog", false);
    assert_fresh(
        &boot_with_state(&name_dir, &renamed.cart, &written),
        "a cart with another name in the slot",
    );
}

/// AC-150 (none): state whose kind filter would hide its entry selects the entry with the kind
/// filter at all kinds, keeping the order. Cart A is [alpha.prg, beta.prg, gamma.crt], and beta is
/// started under A-Z and programs only. Cart B is A rebuilt with beta a cartridge of the same name,
/// so the entry count, the slot and its name all match. The bytes A wrote are written from the
/// monitor into a boot at `entry`, before the menu reads them, as in AC-149.
#[test]
fn ac_150_a_kind_filter_that_hides_the_entry_falls_back_to_all() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-150: skipping");
        return;
    }
    let dir = scratch("ac150");
    let beta = markers::basic_prg("BETA RAN");
    let gamma = markers::cart(CartKind::K8, "GAMMA");
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let cart_a = build(
        &dir,
        "kinds a",
        &[],
        &[
            write(&dir, "alpha.prg", &hello),
            write(&dir, "beta.prg", &beta.bytes),
            write(&dir, "gamma.crt", &gamma.bytes),
        ],
    );
    let d = boot_or_panic(
        &dir,
        &cart_a,
        &[
            Step::Labels(vice::menu_labels()),
            idle(),
            Step::Keys(vec![key::F1, key::F3, key::DOWN, key::RETURN]),
            Step::Break(Target::Addr(beta.spin)),
            screen(),
            state(),
        ],
    );
    assert_marker(&d[0], &beta);
    let written = &d[1];
    assert_resumed(
        &boot_with_state(&dir, &cart_a, written),
        "beta",
        "2 of 2   a-z order   programs",
        "cart A's own state",
    );

    let dir_b = scratch("ac150-b");
    let beta_cart = markers::cart(CartKind::K8, "BETA CART");
    let cart_b = build(
        &dir_b,
        "kinds b",
        &[],
        &[
            write(&dir_b, "alpha.prg", &hello),
            write(&dir_b, "beta.crt", &beta_cart.bytes),
            write(&dir_b, "gamma.crt", &gamma.bytes),
        ],
    );
    assert_resumed(
        &boot_with_state(&dir_b, &cart_b, written),
        "beta",
        "2 of 3   a-z order   all kinds",
        "beta now a cartridge",
    );
}

/// AC-151 (none): the launch routines end by $DFF5 (the labels say so here, and `menu_builds.rs`
/// shows assembly refusing one byte more), every way out leaves $DFF6-$DFFF as the menu wrote them,
/// and AC-127/AC-128 hold through a boot that restores state (A-Z, so the sort runs at boot, and a
/// program selected, so the pane maps its bank in).
#[test]
fn ac_151_the_launch_routines_leave_the_state_alone() {
    let labels = std::fs::read_to_string(vice::menu_labels()).unwrap();
    let ram_end = labels
        .lines()
        .find_map(|l| l.strip_suffix(" .ram_end")?.strip_prefix("al "))
        .map(|a| u16::from_str_radix(a, 16).unwrap())
        .expect("menu.lbl has no .ram_end");
    assert!(
        ram_end <= STATE.0,
        "the launch routines run to ${:04X}, into the state at ${:04X}",
        ram_end - 1,
        STATE.0
    );

    if !vice::require_tool("x64sc") {
        eprintln!("AC-151: skipping the boots");
        return;
    }
    let dir = scratch("ac151");
    let r = resume_cart(&dir, "p99 prog", false);
    for (keys, handoff, end, what) in [
        (start_p99(), "basic_go", r.prog.spin, "a program"),
        (start_cart_b(), "cart_go", r.b.spin, "a cartridge"),
        (start_xbank(), "cart_go", r.x.spin, "an xbank cartridge"),
        (
            vec![key::F3, key::DOWN, key::SHIFT_RETURN],
            "basic_go",
            KEY_WAIT,
            "a program by SHIFT+RETURN",
        ),
        (vec![key::DOWN, key::STOP], "leave", BASIC_MAIN, "RUN/STOP"),
    ] {
        let d = boot_or_panic(
            &dir,
            &r.cart,
            &[
                Step::Labels(vice::menu_labels()),
                idle(),
                state(),
                Step::Keys(keys),
                Step::Break(Target::Label(handoff)),
                state(),
                Step::Break(Target::Addr(end)),
                state(),
            ],
        );
        assert_ne!(
            d[1], d[0],
            "{what}: the menu wrote no state before {handoff}"
        );
        assert_eq!(d[2], d[1], "{what}: the state changed after {handoff}");
    }

    // AC-127 through the restoring boot: the watch is set before the soft reset and logs every bank
    // or mode write until the menu is idle again. AC-147's first case shows this boot restores.
    let dir = scratch("ac151-watch");
    let r = resume_cart(&dir, "p99 prog", false);
    let mut steps = vec![
        Step::Labels(vice::menu_labels()),
        idle(),
        Step::Keys(start_p99()),
        Step::Break(Target::Addr(r.prog.spin)),
        Step::Raw(
            "logname \"monlog.txt\"\nlog on\nwatch store $de00\nwatch store $de02\nreset 0\n\
             r\nm $d01a $d01a"
                .into(),
        ),
    ];
    steps.extend((0..40).map(|_| Step::Raw("g\nr\nm $d01a $d01a".into())));
    // The run ends at the cycle limit once the writes stop; only the log is read.
    let _ = vice::boot(&dir, &r.cart, &steps);
    let log = String::from_utf8_lossy(&std::fs::read(dir.join("monlog.txt")).unwrap_or_default())
        .into_owned();
    let writes = bank_writes(&log);
    assert!(
        writes.len() >= 3,
        "only {} bank or mode writes seen through the restoring boot",
        writes.len()
    );
    for (i, (masked, live)) in writes.iter().enumerate() {
        assert!(
            *masked || !*live,
            "restoring boot: write {i} to $DE00/$DE02 with the raster interrupt live and unmasked"
        );
    }

    // AC-128 after a restore: RETURN starts the restored entry and leaves the stock interrupts.
    let dir = scratch("ac151-exit");
    let r = resume_cart(&dir, "p99 prog", false);
    let d = boot_or_panic(
        &dir,
        &r.cart,
        &[
            Step::Labels(vice::menu_labels()),
            idle(),
            Step::Keys(start_p99()),
            Step::Break(Target::Addr(r.prog.spin)),
            soft_reset(),
            Step::Keys(vec![key::RETURN]),
            Step::Break(Target::Addr(r.prog.spin)),
            screen(),
            Step::Dump(0x0314, 0x0315),
            Step::Dump(0xd01a, 0xd01a),
        ],
    );
    assert_marker(&d[0], &r.prog);
    assert_eq!(d[1], [0x31, 0xea], "after a restore: the IRQ vector");
    assert_eq!(d[2][0] & 0x0f, 0, "after a restore: a VIC interrupt");
}

/// AC-152 (none): the F7 help names the joystick in port 2 and what it does: move, page, fire.
#[test]
fn ac_152_the_help_names_the_joystick() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-152: skipping");
        return;
    }
    let dir = scratch("ac152");
    let cart = numbered_cart(&dir, 3);
    let help = screens_after(&dir, &cart, &[vec![key::F7]]).remove(0);
    let line = (LIST_ROWS.start..HELP_ROW)
        .find(|&n| row_shows(&help, n, "joystick"))
        .expect("the help does not name the joystick");
    for word in ["port 2", "move", "page", "fire"] {
        assert!(
            row_shows(&help, line, word),
            "the help's joystick line does not say '{word}'"
        );
    }
}

// --- menu-highmem: programs reaching into $D000-$FFFF (AC-160..AC-162, AC-165, D-056) -------------
//
// The monitor's `bank ram` makes a dump read the RAM under I/O and the ROMs; `bank cpu` puts the
// view back to what the CPU sees, I/O included, before anything else reads memory.

/// The byte the high-memory programs hold at `addr`. At the VIC registers the handoff sets it
/// differs from their values, and across colour RAM it isn't one colour, so a copy into I/O instead
/// of RAM would show in either.
fn fill(addr: u32) -> u8 {
    ((addr >> 8) as u8) ^ (addr as u8) ^ 0xa5
}

/// Dump `start..=end` as RAM, under I/O and the ROMs.
fn ram(start: u16, end: u16) -> Vec<Step> {
    vec![
        Step::Raw("bank ram".into()),
        Step::Dump(start, end),
        Step::Raw("bank cpu".into()),
    ]
}

/// Start the selection with `keys` and stop at `end`, dumping: the state when the menu hands over
/// (0), then at `end` the screen (1), the VIC (2), colour RAM (3), the state (4) and `load..=last`
/// read as RAM (5).
fn high_boot(
    dir: &Path,
    cart: &Path,
    keys: Vec<u8>,
    end: u16,
    load: u16,
    last: u16,
) -> Vec<Vec<u8>> {
    let mut steps = vec![
        Step::Labels(vice::menu_labels()),
        idle(),
        Step::Keys(keys),
        Step::Break(Target::Label("basic_go")),
        state(),
        Step::Break(Target::Addr(end)),
        screen(),
        Step::Dump(VIC.0, VIC.1),
        Step::Dump(COLOUR_RAM.0, COLOUR_RAM.1),
        state(),
    ];
    steps.extend(ram(load, last));
    boot_or_panic(dir, cart, &steps)
}

/// What [`high_boot`] dumped holds `m` in RAM from its load address, and the copy wrote no I/O: the
/// VIC as the handoff sets it, colour RAM one colour, and the menu's state in cart RAM as it left it.
fn assert_copied_to_ram(d: &[Vec<u8>], m: &Marker, what: &str) {
    assert_eq!(
        d[5].len(),
        m.bytes.len() - 2,
        "{what}: the RAM dump's length"
    );
    if let Some(i) = d[5].iter().zip(&m.bytes[2..]).position(|(a, b)| a != b) {
        panic!(
            "{what}: RAM differs from the program at offset ${i:04X} (got ${:02X}, want ${:02X})",
            d[5][i],
            m.bytes[2 + i]
        );
    }
    assert_eq!(
        (d[2][8] & 0x0f, d[2][9] & 0x0f, d[2][0] | 1),
        (colour::LIGHT_BLUE, colour::BLUE, UPPER_CASE),
        "{what}: (border, background, $D018) are not the handoff's: the copy wrote the VIC"
    );
    assert!(
        d[3].iter().all(|c| c & 0x0f == colour::LIGHT_BLUE),
        "{what}: colour RAM is not all light blue: the copy wrote it"
    );
    assert_eq!(d[4], d[0], "{what}: the menu's state in cart RAM changed");
}

/// AC-160 (none): a program loading at `$C000` with `$4000` bytes, each a function of its address,
/// started by our menu, is in RAM at every address, `$D000-$FFFF` included, read with I/O and ROM
/// banked out; the copy writes no I/O, the program's own output shows on the screen, and it is
/// started by `SYS 49152`.
#[test]
fn ac_160_a_program_reaching_into_d000_ffff_lands_in_ram() {
    if !vice::require_cart_tool("x64sc") {
        eprintln!("AC-160: skipping");
        return;
    }
    let dir = scratch("ac160");
    let m = markers::filled_prg(0xc000, 0x10000, "HIGH PROGRAM RAN", fill);
    let cart = build(&dir, "high", &[], &[write(&dir, "high.prg", &m.bytes)]);
    let d = high_boot(&dir, &cart, vec![key::RETURN], m.spin, 0xc000, 0xffff);
    assert_marker(&d[1], &m);
    assert_copied_to_ram(&d, &m, "$C000-$FFFF by RETURN");
}

/// AC-161 (none): AC-160 holds when the program's bank boundary falls inside `$D000-$DFFF`, and when
/// the program is started with SHIFT+RETURN, which stops at `READY.` with the bytes in place.
#[test]
fn ac_161_a_bank_boundary_in_d000_dfff_and_shift_return() {
    if !vice::require_cart_tool("x64sc") {
        eprintln!("AC-161: skipping");
        return;
    }
    let dir = scratch("ac161");
    let m = markers::filled_prg(0xc000, 0x10000, "WRAPPED PROGRAM RAN", fill);
    // The heavier entry is placed first, so a first entry of $6800 bytes moves the program's first
    // byte $2800 into a bank's 16 KiB and the bank runs out inside $D000-$DFFF; the precondition
    // below checks where it really falls.
    let mut filler = 0x0801u16.to_le_bytes().to_vec();
    filler.resize(0x6800, 0);
    let cart = build(
        &dir,
        "wrap",
        &[],
        &[
            write(&dir, "a filler.prg", &filler),
            write(&dir, "b wrapped.prg", &m.bytes),
        ],
    );
    let info = stidcart_core::read::inspect(&std::fs::read(&cart).unwrap()).unwrap();
    let e = &info.entries[1];
    assert_eq!(e.kind, stidcart_core::directory::EntryType::Prg, "{e:?}");
    let wrap = 0xc000 + (0x4000 - (e.offset as u32 & 0x3fff)) - 2;
    assert!(
        (0xd000..0xe000).contains(&wrap),
        "precondition: the bank runs out at ${wrap:04X}, not inside $D000-$DFFF"
    );

    let d = high_boot(
        &dir,
        &cart,
        vec![key::DOWN, key::RETURN],
        m.spin,
        0xc000,
        0xffff,
    );
    assert_marker(&d[1], &m);
    assert_copied_to_ram(&d, &m, "a bank boundary at $D000-$DFFF");

    let d = high_boot(
        &dir,
        &cart,
        vec![key::DOWN, key::SHIFT_RETURN],
        KEY_WAIT,
        0xc000,
        0xffff,
    );
    assert!(shows(&d[1], "ready."), "SHIFT+RETURN: no READY. prompt");
    assert!(!shows(&d[1], &m.text), "SHIFT+RETURN: the program ran");
    assert_copied_to_ram(&d, &m, "SHIFT+RETURN");
}

/// AC-162 (none): a BASIC program loading at `$0801` whose bytes run past `$D000` (the largest the
/// format holds, to `$E7FE`) runs by `RUN`, with BASIC's pointers set from its end.
#[test]
fn ac_162_a_basic_program_past_d000_runs() {
    if !vice::require_cart_tool("x64sc") {
        eprintln!("AC-162: skipping");
        return;
    }
    let dir = scratch("ac162");
    // basic_prg's line and routine from $0801, then fill to $E7FE.
    let mut m = markers::basic_prg("LONG BASIC RAN");
    let mut bytes = 0x0801u16.to_le_bytes().to_vec();
    bytes.extend(&m.bytes[2 + (0x0801 - 0x07f0)..]);
    let end = 0x0801 + bytes.len() as u32 - 2;
    bytes.extend((end..0xe7ff).map(fill));
    m.bytes = bytes;
    let cart = build(
        &dir,
        "long basic",
        &[],
        &[write(&dir, "long.prg", &m.bytes)],
    );

    let mut steps = press_then_wait(&[key::RETURN], m.spin);
    steps.push(screen());
    steps.push(Step::Dump(0x002b, 0x002e));
    steps.extend(ram(0x0801, 0xe7fe));
    let d = boot_or_panic(&dir, &cart, &steps);
    assert_marker(&d[0], &m);
    assert_eq!(
        d[1],
        [0x01, 0x08, 0xff, 0xe7],
        "BASIC's start and end pointers"
    );
    assert!(
        d[2] == m.bytes[2..],
        "the program is not in RAM at $0801-$E7FE"
    );
}

/// AC-165 (none): starting a program through the second copy, AC-127 and AC-128 hold: no bank or
/// mode write with the raster interrupt live and unmasked (the second copy's own writes among those
/// seen), and the IRQ vector at `$EA31` with the raster interrupt off when the program starts. The
/// NMI vector and `$01` are back to the KERNAL's values.
#[test]
fn ac_165_the_second_copy_keeps_the_interrupt_rules() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-165: skipping");
        return;
    }
    let dir = scratch("ac165-watch");
    let m = markers::filled_prg(0xc000, 0x10000, "IRQ HIGH RAN", fill);
    let cart = build(&dir, "irq high", &[], &[write(&dir, "high.prg", &m.bytes)]);
    let mut steps = vec![
        Step::Labels(vice::menu_labels()),
        idle(),
        Step::Keys(vec![key::RETURN]),
        Step::Raw("logname \"monlog.txt\"\nlog on\nwatch store $de00\nwatch store $de02".into()),
    ];
    steps.extend((0..80).map(|_| Step::Raw("g\nr\nm $d01a $d01a".into())));
    // The run ends at the cycle limit once the writes stop; only the log is read.
    let _ = vice::boot(&dir, &cart, &steps);
    let log = String::from_utf8_lossy(&std::fs::read(dir.join("monlog.txt")).unwrap_or_default())
        .into_owned();
    let writes = bank_writes(&log);
    assert!(!writes.is_empty(), "no bank or mode write was seen");
    for (i, (masked, live)) in writes.iter().enumerate() {
        assert!(
            *masked || !*live,
            "write {i} to $DE00/$DE02 with the raster interrupt live and unmasked"
        );
    }
    assert!(
        log.lines().any(|l| l.trim_start().starts_with(".;01")),
        "no bank write was seen from the routine at $0100"
    );

    let dir = scratch("ac165-exit");
    let cart = build(&dir, "irq high", &[], &[write(&dir, "high.prg", &m.bytes)]);
    let mut steps = press_then_wait(&[key::RETURN], m.spin);
    steps.push(screen());
    steps.push(Step::Dump(0x0314, 0x0315));
    steps.push(Step::Dump(0xd01a, 0xd01a));
    steps.push(Step::Dump(0x0318, 0x0319));
    steps.push(Step::Dump(0x0001, 0x0001));
    let d = boot_or_panic(&dir, &cart, &steps);
    assert_marker(&d[0], &m);
    assert_eq!(d[1], [0x31, 0xea], "the IRQ vector is not $EA31");
    assert_eq!(d[2][0] & 0x0f, 0, "a VIC interrupt is still enabled");
    assert_eq!(
        d[3],
        [0x47, 0xfe],
        "the NMI vector is not the KERNAL's $FE47"
    );
    assert_eq!(
        d[4][0] & 0x07,
        0x07,
        "$01 does not map BASIC, KERNAL and I/O in"
    );
}

// --- menu-folders: the metadata entry read at boot, titles, the pane's fields, folders and a flat
// search (AC-166..AC-169, D-057) ----------------------------------------------------------------------

/// `stidcart edit <cart> args…`, which must succeed.
fn edit(cart: &Path, args: &[&str]) {
    let out = Command::new(env!("CARGO_BIN_EXE_stidcart"))
        .arg("edit")
        .arg(cart)
        .args(args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "edit {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// The pane's sixteen columns, 23-38, of row `n`, without reverse.
fn pane_row(screen: &[u8], n: usize) -> Vec<u8> {
    screen[n * COLS + PANE_COL..n * COLS + PANE_COL + 16]
        .iter()
        .map(|b| b & 0x7f)
        .collect()
}

/// `text` cut into the pane's rows of sixteen cells, the last padded with spaces.
fn wrapped(text: &str) -> Vec<Vec<u8>> {
    vice::screen_text(text)
        .chunks(16)
        .map(|c| {
            let mut v = c.to_vec();
            v.resize(16, 0x20);
            v
        })
        .collect()
}

/// Assert `text` is in the pane wrapped from exactly row `from`, every character in colour `c`, and
/// return the row after its last.
fn assert_wrapped_at(l: &Look, from: usize, text: &str, c: u8, what: &str) -> usize {
    for (k, r) in wrapped(text).iter().enumerate() {
        let n = from + k;
        assert_eq!(
            &pane_row(&l.screen, n),
            r,
            "{what}: pane row {n} is not part {k} of '{text}'"
        );
        let shown = text.len().min((k + 1) * 16) - k * 16;
        let cells = n * COLS + PANE_COL..n * COLS + PANE_COL + shown;
        assert!(
            l.colour[cells].iter().all(|&got| got == c),
            "{what}: '{text}' is not colour {c} in pane row {n}"
        );
    }
    from + wrapped(text).len()
}

/// The pane row at or below `from` whose label is `label`.
fn pane_label(screen: &[u8], from: usize, label: &str, what: &str) -> usize {
    (from..LIST_ROWS.end)
        .find(|&n| pane_row(screen, n).starts_with(&vice::screen_text(label)))
        .unwrap_or_else(|| panic!("{what}: no pane row from {from} on is labelled '{label}'"))
}

/// What a folder row shows in columns 4-19, trailing spaces dropped as `row_name` drops them: the
/// group's name, cut to leave a space before the count, and the count ending at column 19.
fn folder_row(name: &str, count: usize) -> Vec<u8> {
    let count = count.to_string();
    let room = 16 - count.len() - 1;
    let cut = &name[..name.len().min(room)];
    vice::screen_text(&format!("{cut:<w$}{count}", w = 16 - count.len()))
}

/// The two badge cells of list row `n`, without reverse.
fn badge(screen: &[u8], n: usize) -> [u8; 2] {
    [screen[n * COLS + 1] & 0x7f, screen[n * COLS + 2] & 0x7f]
}

/// AC-166 (none): an entry with a title shows it in its list row, cut to the row's 16 columns, and
/// the pane shows every field (in AC-211's order since D-069). Every field is as long as it may be,
/// so all of it must fit. An entry without metadata looks exactly as on the same cart without any (its row and the
/// whole pane, characters and colours), also right after the pane showed an entry with metadata.
/// A-Z sorts by what each row shows.
#[test]
fn ac_166_titles_in_the_list_and_every_field_in_the_pane() {
    if !vice::require_cart_tool("x64sc") {
        eprintln!("AC-166: skipping");
        return;
    }
    let dir = scratch("ac166");
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let inputs = ["alpha", "bravo", "charlie"].map(|n| write(&dir, &format!("{n}.prg"), &hello));
    let cart = build(&dir, "titles", &[], &inputs);
    let plain = dir.join("plain.crt");
    std::fs::copy(&cart, &plain).unwrap();
    let title = "Zeta Quest: the Long Road Home, Deluxe Edition 2";
    let author = "Somebody With A Long Nam";
    let note = "Needs a joystick in port 2 and a lot of patience";
    assert_eq!(
        (title.len(), author.len(), note.len()),
        (48, 24, 48),
        "the fields are not at their longest"
    );
    edit(
        &cart,
        &[
            "--title", "0", title, "--year", "0", "1987", "--author", "0", author, "--note", "0",
            note, "--title", "2", "Able One",
        ],
    );

    let looks = looks_after(&dir, &cart, &[vec![], vec![key::DOWN], vec![key::F1]]);
    let s = &looks[0].screen;
    let cut = vice::screen_text(title[..16].trim_end());
    assert_eq!(
        row_name(s, 5),
        cut,
        "alpha's row does not show its title cut to 16"
    );
    assert_eq!(row_name(s, 6), vice::screen_text("bravo"), "bravo's row");
    assert_eq!(
        row_name(s, 7),
        vice::screen_text("Able One"),
        "charlie's row does not show its title"
    );
    assert_not_listed(s, &["alpha", "charlie"], "titled entries");
    // AC-211's order with every field at its longest: it ends on row 16, a row above the facts.
    assert_chip(&looks[0], "program", colour::LIGHT_GREEN, "alpha");
    let at = assert_wrapped_at(&looks[0], HEAD_ROW, title, colour::WHITE, "alpha");
    let at = assert_wrapped_at(&looks[0], at, "alpha", colour::LIGHT_GREY, "alpha");
    let credit = format!("{author}  1987");
    let at = assert_wrapped_at(&looks[0], at, &credit, colour::WHITE, "alpha");
    assert_pane_blank(&looks[0], at..at + 1, "alpha");
    let at = assert_wrapped_at(&looks[0], at + 1, note, colour::LIGHT_GREY, "alpha");
    assert_eq!(
        at,
        FACT_ROWS.start - 1,
        "the longest fields do not end on row 16"
    );
    assert_pane_blank(&looks[0], at..FACT_ROWS.start, "alpha");
    for (n, label) in FACT_ROWS.zip(["size", "load", "start"]) {
        assert_eq!(pane_label(s, n, label, "alpha"), n, "alpha: '{label}'");
    }

    let after = &looks[1];
    let plain_look = looks_after(&dir, &plain, &[vec![key::DOWN]]).remove(0);
    let same = |cells: std::ops::Range<usize>| {
        after.screen[cells.clone()] == plain_look.screen[cells.clone()]
            && after.colour[cells.clone()] == plain_look.colour[cells]
    };
    assert!(
        same(6 * COLS..7 * COLS),
        "bravo's row differs from the same cart without metadata"
    );
    for n in LIST_ROWS {
        assert!(
            same(n * COLS + PANE_COL - 1..(n + 1) * COLS),
            "pane row {n} for bravo, after alpha's, differs from the same cart without metadata"
        );
    }

    let az = &looks[2].screen;
    for (n, text) in [(5, "Able One"), (6, "bravo")] {
        assert_eq!(row_name(az, n), vice::screen_text(text), "A-Z row {n}");
    }
    assert_eq!(row_name(az, 7), cut, "A-Z: the title is not last");
}

/// The cart AC-167 uses, in this directory order: "zulu" (games), "demo a" (a program, in "demos and
/// things", a 16-character group), "p3" (a marker program, games), "k8" (a marker cartridge, demos
/// and things), "top one", "secret" (hidden, the only entry of "tools"), "top two".
struct FolderCart {
    cart: PathBuf,
    p3: Marker,
}

const DEMOS: &str = "demos and things";

fn folder_cart(dir: &Path) -> FolderCart {
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let p3 = markers::basic_prg("P3 FOLDER RAN");
    let k8 = markers::cart(CartKind::K8, "K8 FOLDER RAN");
    let secret = write(dir, "secret.prg", &hello);
    let inputs = [
        write(dir, "zulu.prg", &hello),
        write(dir, "demo a.prg", &hello),
        write(dir, "p3.prg", &p3.bytes),
        write(dir, "k8.crt", &k8.bytes),
        write(dir, "top one.prg", &hello),
        secret.clone(),
        write(dir, "top two.prg", &hello),
    ];
    let cart = build(
        dir,
        "folders",
        &["--hide", secret.to_str().unwrap()],
        &inputs,
    );
    edit(
        &cart,
        &[
            "--group", "0", "games", "--group", "1", DEMOS, "--group", "2", "games", "--group",
            "3", DEMOS, "--group", "5", "tools",
        ],
    );
    FolderCart { cart, p3 }
}

/// AC-167 (none): with grouped entries the top level lists a folder row per group that has a visible
/// entry (a badge, the name, the count), then the ungrouped entries. RETURN or fire opens a folder;
/// inside, the kind filter, A-Z, paging, F5, HOME/CLR and the filter text act on its entries; DEL
/// with no text, or RUN/STOP, goes back with that folder selected; RUN/STOP at the top level leaves.
/// After a soft reset, an entry started from a folder comes back inside it. The help names the
/// folder keys.
#[test]
fn ac_167_groups_show_as_folders() {
    if !vice::require_cart_tool("x64sc") {
        eprintln!("AC-167: skipping");
        return;
    }
    let dir = scratch("ac167");
    let f = folder_cart(&dir);
    let top = &screens_after(&dir, &f.cart, &[vec![]]).remove(0);
    assert_eq!(
        row_name(top, 5),
        folder_row("games", 2),
        "the games folder row"
    );
    assert_eq!(
        row_name(top, 6),
        folder_row(DEMOS, 2),
        "the 16-character folder row"
    );
    assert_eq!(row_name(top, 7), vice::screen_text("top one"));
    assert_eq!(row_name(top, 8), vice::screen_text("top two"));
    assert!(row_name(top, 9).is_empty(), "a fifth row is listed");
    assert_not_listed(
        top,
        &["zulu", "demo a", "p3", "k8", "secret", "tools"],
        "the top level",
    );
    assert_eq!(
        selected_rows(top),
        vec![5],
        "the first folder is not selected"
    );
    assert!(row_shows(
        top,
        STATUS_ROW,
        "1 of 4   cart order   all kinds"
    ));
    assert_eq!(badge(top, 5), badge(top, 6), "the folder badges differ");
    assert_ne!(
        badge(top, 5),
        badge(top, 7),
        "a folder wears a program's badge"
    );
    assert_ne!(badge(top, 5), [0x20, 0x20], "a folder row has no badge");

    let s = screens_after(
        &dir,
        &f.cart,
        &[
            vec![],                                  // 0: the top level
            vec![key::DOWN, key::RETURN],            // 1: into demos
            vec![key::F3],                           // 2: programs only
            vec![key::F3, key::F3, key::DEL],        // 3: all kinds, back out
            vec![key::UP, key::RETURN, key::F1],     // 4: into games, A-Z
            vec![key::RIGHT],                        // 5: a page down
            vec![key::HOME],                         // 6: the first row, ".." (D-058)
            vec![key::DOWN, key::F5],                // 7: from the first entry, a random other one
            typed("zu"),                             // 8: text
            vec![key::DEL, key::DEL],                // 9: text gone, still inside
            vec![key::DEL],                          // 10: back out
            vec![key::DOWN, key::RETURN, key::STOP], // 11: in and out by RUN/STOP
            vec![key::RETURN, b'K', key::STOP],      // 12: text typed, then RUN/STOP
            vec![key::UP],                           // 13: games
            vec![key::SHIFT_RETURN],                 // 14: nothing on a folder
            vec![key::F7],                           // 15: help
        ],
    );
    assert_eq!(&s[0], top, "the same boot drew another top level");
    let inside = |n: usize, names: &[&str], status: &str, what: &str| {
        assert_listed_in_order(&s[n], names, what);
        assert_not_listed(&s[n], &["games", "demos", "top one", "top two"], what);
        assert!(
            row_shows(&s[n], STATUS_ROW, status),
            "{what}: row 22 does not read '{status}'"
        );
    };
    inside(
        1,
        &["demo a", "k8"],
        "1 of 2   cart order   all kinds",
        "into demos",
    );
    inside(
        2,
        &["demo a"],
        "1 of 1   cart order   programs",
        "F3 inside",
    );
    assert_not_listed(&s[2], &["k8"], "F3 inside");
    assert_eq!(selected_rows(&s[3]), vec![6], "DEL: demos is not selected");
    assert!(row_shows(
        &s[3],
        STATUS_ROW,
        "2 of 4   cart order   all kinds"
    ));
    inside(
        4,
        &["p3", "zulu"],
        "1 of 2   a-z order   all kinds",
        "games, A-Z",
    );
    assert_selected(&s[5], "zulu", "CRSR right inside");
    assert_eq!(
        selected_rows(&s[6]),
        vec![5],
        "HOME inside: '..' is not selected"
    );
    assert_selected(&s[7], "zulu", "F5 inside");
    inside(
        8,
        &["zulu"],
        "1 of 1   a-z order   all kinds",
        "text inside",
    );
    assert_not_listed(&s[8], &["p3"], "text inside");
    assert!(
        row_shows(&s[8], QUERY_ROW, "1 of 2 match"),
        "text inside: row 23 does not count out of the folder's two entries"
    );
    inside(
        9,
        &["p3", "zulu"],
        "1 of 2   a-z order   all kinds",
        "DEL with text",
    );
    assert_eq!(selected_rows(&s[10]), vec![5], "DEL: games is not selected");
    assert_eq!(
        row_name(&s[10], 5),
        folder_row("games", 2),
        "DEL: back at the top"
    );
    assert_eq!(
        selected_rows(&s[11]),
        vec![6],
        "RUN/STOP: demos is not selected"
    );
    assert!(row_shows(
        &s[11],
        STATUS_ROW,
        "2 of 4   a-z order   all kinds"
    ));
    assert_eq!(selected_rows(&s[12]), vec![6], "RUN/STOP with text: demos");
    assert!(
        row(&s[12], QUERY_ROW).iter().all(|b| b & 0x7f == 0x20),
        "RUN/STOP with text: the text was kept"
    );
    assert_eq!(s[14], s[13], "SHIFT+RETURN on a folder changed the screen");
    let help = &s[15][LIST_ROWS.start * COLS..HELP_ROW * COLS];
    assert!(
        vice::find(help, &vice::screen_text("folder")).is_some(),
        "the help does not name the folder keys"
    );

    let d = boot_or_panic(
        &dir,
        &f.cart,
        &[
            Step::Labels(vice::menu_labels()),
            idle(),
            Step::Keys(vec![key::STOP]),
            Step::Break(Target::Addr(BASIC_MAIN)),
            screen(),
        ],
    );
    assert!(shows(&d[0], "ready."), "RUN/STOP at the top did not leave");

    let d = boot_or_panic(
        &dir,
        &f.cart,
        &[
            Step::Labels(vice::menu_labels()),
            idle(),
            Step::Keys(vec![key::RETURN, key::DOWN, key::RETURN]),
            Step::Break(Target::Addr(f.p3.spin)),
            screen(),
            soft_reset(),
            screen(),
        ],
    );
    assert_marker(&d[0], &f.p3);
    assert_selected(&d[1], "p3", "after a soft reset");
    assert_listed_in_order(&d[1], &["zulu", "p3"], "after a soft reset");
    assert_not_listed(&d[1], &["games", "top one"], "after a soft reset");
    assert!(row_shows(
        &d[1],
        STATUS_ROW,
        "2 of 2   cart order   all kinds"
    ));

    let mut steps = vec![idle()];
    steps.extend(hold(stick::FIRE, 2));
    steps.push(screen());
    let d = boot_stick(&dir, &f.cart, stick::REST, steps);
    assert_listed_in_order(&d[0], &["zulu", "p3"], "fire on a folder");
    assert!(row_shows(
        &d[0],
        STATUS_ROW,
        "1 of 2   cart order   all kinds"
    ));
}

/// AC-168 (none): text typed at the top level lists the matching visible entries of every folder and
/// of the top level flat, in the order in use, matching the name or the title, ignoring case. An
/// entry started from that list comes back inside its folder after a soft reset.
#[test]
fn ac_168_text_at_the_top_searches_every_folder() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-168: skipping");
        return;
    }
    let dir = scratch("ac168");
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let p3 = markers::basic_prg("P3 FOUND RAN");
    let hidden = write(&dir, "hidden ball.prg", &hello);
    let inputs = [
        write(&dir, "zulu.prg", &hello),
        write(&dir, "demo a.prg", &hello),
        write(&dir, "p3.prg", &p3.bytes),
        write(&dir, "football.prg", &hello),
        hidden.clone(),
        write(&dir, "top two.prg", &hello),
    ];
    let cart = build(
        &dir,
        "search",
        &["--hide", hidden.to_str().unwrap()],
        &inputs,
    );
    edit(
        &cart,
        &[
            "--title",
            "0",
            "Ballblazer",
            "--group",
            "0",
            "games",
            "--title",
            "1",
            "PINBALL Dreams",
            "--group",
            "1",
            "demos",
            "--group",
            "2",
            "games",
            "--group",
            "4",
            "demos",
        ],
    );
    let mut az = vec![key::CLR];
    az.extend(typed("ball"));
    az.push(key::F1);
    let s = screens_after(
        &dir,
        &cart,
        &[typed("ball"), vec![key::CLR], typed("zul"), az],
    );
    for (n, text) in [(5, "Ballblazer"), (6, "PINBALL Dreams"), (7, "football")] {
        assert_eq!(
            row_name(&s[0], n),
            vice::screen_text(text),
            "'ball': row {n}"
        );
    }
    assert!(row_name(&s[0], 8).is_empty(), "'ball': a fourth row");
    assert_not_listed(&s[0], &["games", "demos", "hidden"], "'ball'");
    assert!(row_shows(
        &s[0],
        STATUS_ROW,
        "1 of 3   cart order   all kinds"
    ));
    assert!(
        row_shows(&s[0], QUERY_ROW, "3 of 5 match"),
        "'ball': the match count"
    );
    assert_eq!(
        row_name(&s[1], 5),
        folder_row("games", 2),
        "CLR: the folders are back"
    );
    assert_eq!(
        row_name(&s[1], 6),
        folder_row("demos", 1),
        "CLR: the folders are back"
    );
    assert_eq!(
        row_name(&s[2], 5),
        vice::screen_text("Ballblazer"),
        "'zul' finds the name"
    );
    assert!(row_name(&s[2], 6).is_empty(), "'zul': a second row");
    for (n, text) in [(5, "Ballblazer"), (6, "football"), (7, "PINBALL Dreams")] {
        assert_eq!(
            row_name(&s[3], n),
            vice::screen_text(text),
            "'ball' A-Z: row {n}"
        );
    }

    let mut keys = typed("p3");
    keys.push(key::RETURN);
    let d = boot_or_panic(
        &dir,
        &cart,
        &[
            Step::Labels(vice::menu_labels()),
            idle(),
            Step::Keys(keys),
            Step::Break(Target::Addr(p3.spin)),
            screen(),
            soft_reset(),
            screen(),
        ],
    );
    assert_marker(&d[0], &p3);
    assert_selected(&d[1], "p3", "after a soft reset");
    assert_listed_in_order(&d[1], &["Ballblazer", "p3"], "after a soft reset");
    assert!(row_shows(
        &d[1],
        STATUS_ROW,
        "2 of 2   cart order   all kinds"
    ));
}

/// A metadata record as the format lays it out: its length, the index, the 16 name bytes a slot
/// storing `name` holds, the group number, the title, and an empty year, author and note.
fn meta_record(index: u8, name: &str, group: u8, title: &str) -> Vec<u8> {
    let mut r = vec![0, index];
    r.extend(stidcart_core::directory::name_bytes(name));
    r.push(group);
    let t = stidcart_core::petscii::ascii_to_petscii(title);
    r.push(t.len() as u8);
    r.extend(t);
    r.extend([0, 0, 0]);
    r[0] = r.len() as u8;
    r
}

/// A metadata entry's data: the load address $4000, then a payload of `len` bytes: the magic,
/// `version`, `groups`, padding, and the records as its last bytes, where the header's offset points.
/// So the records are the last bytes a copy reaches.
fn meta_data(version: u8, groups: &[&str], records: &[Vec<u8>], len: usize) -> Vec<u8> {
    let first = len - records.iter().map(Vec::len).sum::<usize>();
    let mut p = b"STIDMETA".to_vec();
    p.extend([version, groups.len() as u8, records.len() as u8]);
    p.extend((first as u16).to_le_bytes());
    for g in groups {
        let mut n = stidcart_core::petscii::ascii_to_petscii(g);
        n.resize(16, 0);
        p.extend(n);
    }
    assert!(p.len() <= first, "the records overlap the header");
    p.resize(first, 0);
    p.extend(records.concat());
    let mut data = vec![0x00, 0x40];
    data.extend(p);
    data
}

/// alpha and bravo, then `meta` (if any) as a hidden program named `stidcart.meta`.
fn meta_cart(dir: &Path, meta: Option<&[u8]>) -> PathBuf {
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let mut inputs = vec![
        write(dir, "alpha.prg", &hello),
        write(dir, "bravo.prg", &hello),
    ];
    let mut extra = Vec::new();
    if let Some(m) = meta {
        let p = write(dir, "stidcart.meta.prg", m);
        extra = vec!["--hide".to_string(), p.to_str().unwrap().to_string()];
        inputs.push(p);
    }
    let extra: Vec<&str> = extra.iter().map(String::as_str).collect();
    build(dir, "meta rules", &extra, &inputs)
}

/// AC-169 (none): the menu copies the metadata entry to `$4000-$7FFF` once at boot, with interrupts
/// masked (AC-127 holds through a boot that copies a full 16 KB, which spans two banks), and boots as
/// for a cart without metadata (the same screen) when the entry is missing or lacks the magic, its
/// version is unknown, it is larger than 16 KB, or a record's length runs past the payload. A record
/// whose name doesn't match its slot is ignored. Each refused payload differs from one the menu uses
/// in that one respect.
#[test]
fn ac_169_the_metadata_entry_is_checked_before_use() {
    if !vice::require_cart_tool("x64sc") {
        eprintln!("AC-169: skipping");
        return;
    }
    let alpha = meta_record(0, "alpha", 1, "Alpha Title");
    let bravo = meta_record(1, "bravo", 0, "Bravo Title");
    let groups = ["meta games"];
    let full = meta_data(1, &groups, &[alpha.clone(), bravo.clone()], 16384);

    let screen_of = |tag: &str, meta: Option<&[u8]>| {
        let dir = scratch(&format!("ac169-{tag}"));
        let cart = meta_cart(&dir, meta);
        menu_screen(&dir, &cart)
    };
    let without = screen_of("none", None);
    assert_eq!(row_name(&without, 5), vice::screen_text("alpha"));

    let used = screen_of("full", Some(&full));
    assert_eq!(
        row_name(&used, 5),
        folder_row("meta games", 1),
        "16 KB: the folder"
    );
    assert_eq!(
        row_name(&used, 6),
        vice::screen_text("Bravo Title"),
        "16 KB: the title"
    );

    let mut long = bravo.clone();
    long[0] += 1;
    let mut no_magic = full.clone();
    no_magic[2] = b'X';
    for (tag, data, what) in [
        ("magic", no_magic, "without the magic"),
        (
            "version",
            meta_data(2, &groups, &[alpha.clone(), bravo.clone()], 16384),
            "version 2",
        ),
        (
            "large",
            meta_data(1, &groups, &[alpha.clone(), bravo.clone()], 16385),
            "16385 bytes",
        ),
        (
            "length",
            meta_data(1, &groups, &[alpha.clone(), long], 16384),
            "a record past the payload",
        ),
    ] {
        assert_eq!(
            screen_of(tag, Some(&data)),
            without,
            "{what}: the menu did not boot as without metadata"
        );
    }

    let renamed = meta_record(0, "alphx", 0, "Alpha Title");
    let s = screen_of(
        "name",
        Some(&meta_data(1, &groups, &[renamed, bravo.clone()], 200)),
    );
    assert_eq!(
        row_name(&s, 5),
        vice::screen_text("alpha"),
        "a renamed record was used"
    );
    assert_eq!(
        row_name(&s, 6),
        vice::screen_text("Bravo Title"),
        "the other record"
    );

    // AC-127 through a boot that copies the 16 KB entry: the watch is set before a soft reset and
    // logs every bank or mode write until the menu is idle again. Every one is masked, and the copy
    // routine at $0100 is among them.
    let dir = scratch("ac169-watch");
    let cart = meta_cart(&dir, Some(&full));
    let mut steps = vec![
        Step::Labels(vice::menu_labels()),
        idle(),
        Step::Raw(
            "logname \"monlog.txt\"\nlog on\nwatch store $de00\nwatch store $de02\nreset 0\n\
             r\nm $d01a $d01a"
                .into(),
        ),
    ];
    steps.extend((0..40).map(|_| Step::Raw("g\nr\nm $d01a $d01a".into())));
    let _ = vice::boot(&dir, &cart, &steps);
    let log = String::from_utf8_lossy(&std::fs::read(dir.join("monlog.txt")).unwrap_or_default())
        .into_owned();
    let writes = bank_writes(&log);
    assert!(!writes.is_empty(), "no bank or mode write was seen");
    for (i, (masked, _)) in writes.iter().enumerate() {
        assert!(*masked, "write {i} to $DE00/$DE02 with interrupts unmasked");
    }
    assert!(
        log.lines().any(|l| l.trim_start().starts_with(".;01")),
        "no bank write was seen from the copy routine at $0100 during the boot"
    );
}

/// Every directory slot of `cart` as stored, up to the end marker.
fn stored_slots(cart: &Path) -> Vec<stidcart_core::directory::DirEntry> {
    use stidcart_core::directory::{self, Slot, ENTRY_LEN};
    let bytes = std::fs::read(cart).unwrap();
    let image = stidcart_core::crt::parse(&bytes).unwrap().image;
    let mut out = Vec::new();
    for slot in 0..256 {
        let at = stidcart_core::reserved::DIRECTORY_OFFSET + slot * ENTRY_LEN;
        let raw: [u8; ENTRY_LEN] = image.as_bytes()[at..at + ENTRY_LEN].try_into().unwrap();
        match directory::decode(&raw) {
            Slot::Program(e) | Slot::Cartridge(e) => out.push(e),
            _ => break,
        }
    }
    out
}

/// AC-169 (none): a metadata entry confined to one half (type `0x02` or `0x03`), as the placer
/// makes one that fits a gap left between cartridges, is read like an interleaved one. Three 8K
/// cartridges of a kind in banks 1-3 leave the other half of banks 1-2 free (gaps are searched
/// below the top bank only), a 16 KB run, and a 10 KB entry placed there
/// crosses from one bank's half into the next one's, with its records last. The test reads the
/// slot's type and extent back from the cart, so a change in placement can't quietly turn it into
/// an interleaved entry.
#[test]
fn ac_169_a_metadata_entry_confined_to_one_half_is_read() {
    use stidcart_core::directory::EntryType;
    if !vice::require_cart_tool("x64sc") {
        eprintln!("AC-169: skipping");
        return;
    }
    for (kind, tag, want) in [
        (CartKind::K8, "romh", EntryType::PrgRomH),
        (CartKind::Ultimax8, "roml", EntryType::PrgRomL),
    ] {
        let dir = scratch(&format!("ac169-confined-{tag}"));
        let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
        let carts = ["c1", "c2", "c3"].map(|n| {
            let m = markers::cart(kind, &format!("{} RAN", n.to_uppercase()));
            write(&dir, &format!("{n}.crt"), &m.bytes)
        });
        let meta = meta_data(
            1,
            &[],
            &[
                meta_record(3, "alpha", 0, "Alpha Title"),
                meta_record(4, "bravo", 0, "Bravo Title"),
            ],
            10000,
        );
        let meta = write(&dir, "stidcart.meta.prg", &meta);
        let inputs = [
            carts[0].clone(),
            carts[1].clone(),
            carts[2].clone(),
            write(&dir, "alpha.prg", &hello),
            write(&dir, "bravo.prg", &hello),
            meta.clone(),
        ];
        let cart = build(
            &dir,
            "confined",
            &["--hide", meta.to_str().unwrap()],
            &inputs,
        );
        let slot = stored_slots(&cart)
            .into_iter()
            .find(|e| e.name == "stidcart.meta")
            .expect("no metadata slot");
        assert_eq!(
            slot.kind, want,
            "{tag}: the entry is not confined to that half"
        );
        assert!(
            (slot.offset as usize & 0x1fff) + slot.size as usize > 0x2000,
            "{tag}: the entry doesn't cross into the next bank's half"
        );

        let s = menu_screen(&dir, &cart);
        assert_eq!(
            row_name(&s, 8),
            vice::screen_text("Alpha Title"),
            "{tag}: alpha"
        );
        assert_eq!(
            row_name(&s, 9),
            vice::screen_text("Bravo Title"),
            "{tag}: bravo"
        );
    }
}

/// AC-169 (none): with two hidden `STIDCART.META` slots carrying the magic, the last one is the
/// metadata, and with the last one lacking the magic, the one before it is.
#[test]
fn ac_169_the_last_of_two_metadata_entries_is_the_one() {
    if !vice::require_cart_tool("x64sc") {
        eprintln!("AC-169: skipping");
        return;
    }
    let titles = |prefix: &str| {
        meta_data(
            1,
            &[],
            &[
                meta_record(0, "alpha", 0, &format!("{prefix} Alpha")),
                meta_record(1, "bravo", 0, &format!("{prefix} Bravo")),
            ],
            200,
        )
    };
    let mut no_magic = titles("Last");
    no_magic[2] = b'X';
    for (tag, last, want) in [
        ("both", titles("Last"), "Last"),
        ("no-magic", no_magic, "First"),
    ] {
        let dir = scratch(&format!("ac169-two-{tag}"));
        let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
        let first = write_in(&dir, "a", "stidcart.meta", &titles("First"));
        let second = write_in(&dir, "b", "stidcart.meta", &last);
        let inputs = [
            write(&dir, "alpha.prg", &hello),
            write(&dir, "bravo.prg", &hello),
            first.clone(),
            second.clone(),
        ];
        let cart = build(
            &dir,
            "two metas",
            &[
                "--hide",
                first.to_str().unwrap(),
                "--hide",
                second.to_str().unwrap(),
            ],
            &inputs,
        );
        let slots: Vec<(String, bool)> = stored_slots(&cart)
            .into_iter()
            .map(|e| (e.name, e.hidden))
            .collect();
        let meta = ("stidcart.meta".to_string(), true);
        assert_eq!(
            slots[2..],
            [meta.clone(), meta],
            "{tag}: the cart doesn't carry two hidden metadata slots last"
        );

        let s = menu_screen(&dir, &cart);
        for (n, entry) in [(5, "Alpha"), (6, "Bravo")] {
            assert_eq!(
                row_name(&s, n),
                vice::screen_text(&format!("{want} {entry}")),
                "{tag}: row {n}"
            );
        }
    }
}

// --- menu-folder-up: the ".." row that goes back up (AC-170..AC-172, D-058) -----------------------

/// What the `..` row shows in columns 4-19.
fn up_row() -> Vec<u8> {
    vice::screen_text("..")
}

/// Assert list row 5 is `..` wearing the badge of the top level's folder rows.
fn assert_up_first(s: &[u8], folder_badge: [u8; 2], what: &str) {
    assert_eq!(
        row_name(s, 5),
        up_row(),
        "{what}: the first row is not '..'"
    );
    assert_eq!(
        badge(s, 5),
        folder_badge,
        "{what}: '..' does not wear the folder badge"
    );
}

/// Assert the top level is listed with the folder row at screen row `at` selected.
fn assert_back_at(s: &[u8], at: usize, name: &str, count: usize, what: &str) {
    assert_eq!(
        row_name(s, at),
        folder_row(name, count),
        "{what}: not back at the top level"
    );
    assert_eq!(
        selected_rows(s),
        vec![at],
        "{what}: the folder '{name}' is not selected"
    );
}

/// AC-211 (none): with metadata the pane shows from row 7 the title in white, the stored name in
/// light grey, the author, two spaces and the year in white (whichever are set), a blank row and
/// the note in light grey (D-070 made each a step brighter), the facts staying in rows 18-20. Without a title the name leads in white. A folder
/// shows a white "folder" chip and its name; ".." shows the chip, "..", and that it goes back to
/// the top level.
#[test]
fn ac_211_the_pane_orders_metadata_and_names_folders() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-211: skipping");
        return;
    }
    let dir = scratch("ac211");
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let inputs =
        ["alpha", "bravo", "charlie", "delta"].map(|n| write(&dir, &format!("{n}.prg"), &hello));
    let cart = build(&dir, "fields", &[], &inputs);
    edit(
        &cart,
        &[
            "--title",
            "0",
            "RAM Test",
            "--year",
            "0",
            "2026",
            "--author",
            "0",
            "stid",
            "--note",
            "0",
            "Checks all 64 KB",
            "--title",
            "1",
            "Only Year",
            "--year",
            "1",
            "1987",
            "--author",
            "2",
            "nobody",
            "--note",
            "3",
            "just a note",
        ],
    );
    let down = vec![key::DOWN];
    let looks = looks_after(&dir, &cart, &[vec![], down.clone(), down.clone(), down]);
    let facts = [("size", "1K"), ("load", "$0801"), ("start", "RUN")];

    let l = &looks[0];
    assert_chip(l, "program", colour::LIGHT_GREEN, "alpha");
    let at = assert_wrapped_at(l, HEAD_ROW, "RAM Test", colour::WHITE, "alpha");
    let at = assert_wrapped_at(l, at, "alpha", colour::LIGHT_GREY, "alpha");
    let at = assert_wrapped_at(l, at, "stid  2026", colour::WHITE, "alpha");
    assert_pane_blank(l, at..at + 1, "alpha");
    let at = assert_wrapped_at(l, at + 1, "Checks all 64 KB", colour::LIGHT_GREY, "alpha");
    assert_pane_blank(l, at..FACT_ROWS.start, "alpha");
    assert_facts(l, facts, "alpha");

    let l = &looks[1];
    let at = assert_wrapped_at(l, HEAD_ROW, "Only Year", colour::WHITE, "bravo");
    let at = assert_wrapped_at(l, at, "bravo", colour::LIGHT_GREY, "bravo");
    let at = assert_wrapped_at(l, at, "1987", colour::WHITE, "bravo");
    assert_pane_blank(l, at..FACT_ROWS.start, "bravo");
    assert_facts(l, facts, "bravo");

    let l = &looks[2];
    let at = assert_wrapped_at(l, HEAD_ROW, "charlie", colour::WHITE, "charlie");
    let at = assert_wrapped_at(l, at, "nobody", colour::WHITE, "charlie");
    assert_pane_blank(l, at..FACT_ROWS.start, "charlie");

    let l = &looks[3];
    let at = assert_wrapped_at(l, HEAD_ROW, "delta", colour::WHITE, "delta");
    assert_pane_blank(l, at..at + 1, "delta");
    let at = assert_wrapped_at(l, at + 1, "just a note", colour::LIGHT_GREY, "delta");
    assert_pane_blank(l, at..FACT_ROWS.start, "delta");
    assert_facts(l, facts, "delta");

    let dir = scratch("ac211-folders");
    let f = folder_cart(&dir);
    let looks = looks_after(&dir, &f.cart, &[vec![], vec![key::RETURN, key::UP]]);
    let l = &looks[0];
    assert_chip(l, "folder", colour::WHITE, "games");
    let at = assert_wrapped_at(l, HEAD_ROW, "games", colour::WHITE, "games");
    assert_pane_blank(l, at..LIST_ROWS.end, "games");
    let l = &looks[1];
    assert_chip(l, "folder", colour::WHITE, "'..'");
    let at = assert_wrapped_at(l, HEAD_ROW, "..", colour::WHITE, "'..'");
    assert_pane_blank(l, at..at + 1, "'..'");
    let at = assert_wrapped_at(
        l,
        at + 1,
        "goes back to thetop level",
        colour::LIGHT_GREY,
        "'..'",
    );
    assert_pane_blank(l, at..LIST_ROWS.end, "'..'");
}

/// AC-170 (none): a folder opens with `..` as its first row, wearing the folder badge, and the first
/// entry below it selected. RETURN on `..`, and fire on the joystick in port 2, go back to the top
/// level with the folder that was open selected. The pane on `..` says it goes back to the top level.
#[test]
fn ac_170_a_folder_starts_with_a_row_that_goes_back_up() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-170: skipping");
        return;
    }
    let dir = scratch("ac170");
    let f = folder_cart(&dir);
    let s = screens_after(
        &dir,
        &f.cart,
        &[
            vec![],                       // 0: the top level
            vec![key::DOWN, key::RETURN], // 1: into demos
            vec![key::UP],                // 2: on ".."
            vec![key::RETURN],            // 3: back up
        ],
    );
    let folder_badge = badge(&s[0], 5);
    assert_up_first(&s[1], folder_badge, "into demos");
    assert_listed_in_order(&s[1], &["demo a", "k8"], "into demos");
    assert_eq!(
        row_name(&s[1], 6),
        vice::screen_text("demo a"),
        "into demos: the first entry is not right under '..'"
    );
    assert_selected(&s[1], "demo a", "into demos");
    assert_eq!(selected_rows(&s[2]), vec![5], "UP: '..' is not selected");
    assert_eq!(
        pane_row(&s[2], HEAD_ROW),
        wrapped("..")[0],
        "the pane does not name '..'"
    );
    let pane: Vec<u8> = (LIST_ROWS.start + 1..LIST_ROWS.end)
        .flat_map(|n| pane_row(&s[2], n))
        .collect();
    assert!(
        vice::find(&pane, &vice::screen_text("top level")).is_some(),
        "the pane on '..' does not say it goes back to the top level"
    );
    assert_back_at(&s[3], 6, DEMOS, 2, "RETURN on '..'");

    // The same with the stick alone: down to demos, fire, up to "..", fire.
    let mut steps = vec![idle()];
    for lines in [stick::DOWN, stick::FIRE, stick::UP] {
        steps.extend(hold(lines, 3));
    }
    steps.push(screen());
    steps.extend(hold(stick::FIRE, 3));
    steps.push(screen());
    let d = boot_stick(&dir, &f.cart, stick::REST, steps);
    assert_up_first(&d[0], folder_badge, "the stick in demos");
    assert_eq!(
        selected_rows(&d[0]),
        vec![5],
        "stick up: '..' is not selected"
    );
    assert_back_at(&d[1], 6, DEMOS, 2, "fire on '..'");
}

/// AC-171 (none): `..` stays the first row and can be chosen with filter text that matches nothing,
/// under A-Z and under each kind filter. HOME selects it and CLR the last entry.
#[test]
fn ac_171_the_up_row_is_always_listed_first() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-171: skipping");
        return;
    }
    let dir = scratch("ac171");
    let f = folder_cart(&dir);
    // "!" ($21) sorts before "." ($2E): a ".." sorted with the entries would come second under A-Z.
    edit(&f.cart, &["--title", "3", "!k8"]);
    let s = screens_after(
        &dir,
        &f.cart,
        &[
            vec![],                                // 0: the top level
            vec![key::DOWN, key::RETURN],          // 1: into demos
            typed("qq"),                           // 2: text matching nothing
            vec![key::RETURN],                     // 3: ".." chosen
            vec![key::RETURN, key::F1],            // 4: into demos, A-Z
            vec![key::F3],                         // 5: programs
            vec![key::F3],                         // 6: cartridges
            vec![key::UP, key::RETURN],            // 7: ".." chosen
            vec![key::RETURN, key::F3, key::HOME], // 8: all kinds, HOME
            vec![key::CLR],                        // 9: CLR
        ],
    );
    let folder_badge = badge(&s[0], 5);
    assert_up_first(&s[2], folder_badge, "no match");
    assert_eq!(
        selected_rows(&s[2]),
        vec![5],
        "no match: '..' is not selected"
    );
    assert_not_listed(&s[2], &["demo a", "k8"], "no match");
    assert_back_at(&s[3], 6, DEMOS, 2, "RETURN on '..' with no match");
    for (n, names, what) in [
        (4, &["!k8", "demo a"][..], "A-Z"),
        (5, &["demo a"][..], "programs"),
        (6, &["!k8"][..], "cartridges"),
    ] {
        assert_up_first(&s[n], folder_badge, what);
        assert_listed_in_order(&s[n], names, what);
        assert_selected(&s[n], names[0], what);
    }
    assert_not_listed(&s[5], &["k8"], "programs");
    assert_not_listed(&s[6], &["demo a"], "cartridges");
    assert_back_at(&s[7], 6, DEMOS, 2, "RETURN on '..' listing cartridges");
    assert_up_first(&s[8], folder_badge, "HOME");
    assert_eq!(selected_rows(&s[8]), vec![5], "HOME: '..' is not selected");
    assert_selected(&s[9], "demo a", "CLR");
    assert_eq!(row_name(&s[9], 5), up_row(), "CLR: '..' moved");
}

/// AC-172 (none): inside a folder, rows 22-23 count its entries and never `..`, F5 never selects
/// `..`, and an entry started there comes back after a soft reset, never `..`. DEL with no text and
/// RUN/STOP still go back up.
#[test]
fn ac_172_the_up_row_is_not_an_entry() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-172: skipping");
        return;
    }
    let dir = scratch("ac172");
    let f = folder_cart(&dir);
    let mut groups = vec![
        vec![key::RETURN], // 0: into games
        vec![key::UP],     // 1: on ".."
        typed("u"),        // 2: text
        vec![key::DEL],    // 3: text gone
    ];
    // Eight presses from an entry: a pick that could land on ".." would, about half the time.
    let f5_from = groups.len();
    groups.extend(std::iter::repeat_n(vec![key::F5], 8));
    groups.push(vec![key::HOME, key::F5]); // from "..": either entry, never ".."
    groups.push(vec![key::DEL]); // back out by DEL
    groups.push(vec![key::RETURN, key::STOP]); // in and out by RUN/STOP
    let s = screens_after(&dir, &f.cart, &groups);
    let status = |n: usize, text: &str, what: &str| {
        assert!(
            row_shows(&s[n], STATUS_ROW, text),
            "{what}: row 22 does not read '{text}'"
        );
    };
    status(0, "1 of 2   cart order   all kinds", "into games");
    assert_selected(&s[0], "zulu", "into games");
    status(1, "0 of 2   cart order   all kinds", "on '..'");
    assert_eq!(selected_rows(&s[1]), vec![5], "UP: '..' is not selected");
    status(2, "1 of 1   cart order   all kinds", "text");
    assert!(
        row_shows(&s[2], QUERY_ROW, "1 of 2 match"),
        "text: row 23 does not count the folder's two entries alone"
    );
    assert_selected(&s[3], "zulu", "DEL with text");
    let mut last = "zulu";
    for (k, screen) in s[f5_from..f5_from + 8].iter().enumerate() {
        let next = if last == "zulu" { "p3" } else { "zulu" };
        assert_selected(screen, next, &format!("F5 press {}", k + 1));
        last = next;
    }
    let picked = selected_rows(&s[f5_from + 8]);
    assert!(
        picked == [6] || picked == [7],
        "F5 on '..': rows {picked:?} selected, not one of the two entries"
    );
    assert_back_at(&s[f5_from + 9], 5, "games", 2, "DEL with no text");
    assert_back_at(&s[f5_from + 10], 5, "games", 2, "RUN/STOP");

    // The first entry under "..", started and soft-reset: it comes back, not "..".
    let d = boot_or_panic(
        &dir,
        &f.cart,
        &[
            Step::Labels(vice::menu_labels()),
            idle(),
            Step::Keys(vec![key::RETURN, key::F1, key::RETURN]),
            Step::Break(Target::Addr(f.p3.spin)),
            screen(),
            soft_reset(),
            screen(),
        ],
    );
    assert_marker(&d[0], &f.p3);
    assert_eq!(
        row_name(&d[1], 5),
        up_row(),
        "after a soft reset: '..' is not first"
    );
    assert_eq!(
        assert_selected(&d[1], "p3", "after a soft reset"),
        6,
        "after a soft reset: p3 is not right under '..'"
    );
    assert!(row_shows(
        &d[1],
        STATUS_ROW,
        "1 of 2   a-z order   all kinds"
    ));
}

// folder-browser (AC-191, D-063) ------------------------------------------------------------------

/// AC-191 (none): our menu lists a cart's folders in the cart's folder order, not the order entries
/// first use them, and shows no row for an empty folder. The menu itself is unchanged: it lists
/// groups by number and skips one with no visible entry, and the folder table is now the cart's
/// ordered folder list (D-063).
#[test]
fn ac_191_folders_follow_the_carts_order_and_an_empty_one_is_hidden() {
    if !vice::require_cart_tool("x64sc") {
        eprintln!("AC-191: skipping");
        return;
    }
    let dir = scratch("ac191");
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let inputs = ["alpha", "bravo", "loose one"].map(|n| write(&dir, &format!("{n}.prg"), &hello));
    let cart = build(&dir, "order", &[], &inputs);
    // alpha (first entry) is in "games", bravo in "tools"; the order puts tools first and an
    // empty "spare" between them.
    edit(
        &cart,
        &[
            "--folder",
            "0",
            "games",
            "--folder",
            "1",
            "tools",
            "--new-folder",
            "spare",
            "--move-folder",
            "tools",
            "0",
            "--move-folder",
            "spare",
            "1",
        ],
    );
    let top = &screens_after(&dir, &cart, &[vec![]]).remove(0);
    assert_eq!(
        row_name(top, 5),
        folder_row("tools", 1),
        "the cart's first folder first"
    );
    assert_eq!(
        row_name(top, 6),
        folder_row("games", 1),
        "the empty folder is skipped"
    );
    assert_eq!(row_name(top, 7), vice::screen_text("loose one"));
    assert!(row_name(top, 8).is_empty(), "a fourth row is listed");
    assert_not_listed(top, &["spare"], "the top level");
}
