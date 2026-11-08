//! custom-menu: our menu on the C64, proven by booting carts in VICE (AC-87..AC-92), and the CI
//! requirement that these boots run rather than skip (AC-94). menu-search-sort adds its filter and
//! sort toggle (AC-96..AC-101), and menu-look its colours and bars (AC-102..AC-104).
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
    for group in groups {
        for chunk in group.chunks(10) {
            steps.push(idle());
            steps.push(Step::Keys(chunk.to_vec()));
        }
        steps.push(idle());
        steps.push(screen());
    }
    boot_or_panic(dir, cart, &steps)
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
        eprintln!("AC-96: skipping — x64sc is not installed");
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
        eprintln!("AC-97: skipping — x64sc is not installed");
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
        eprintln!("AC-98: skipping — x64sc is not installed");
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
        eprintln!("AC-99: skipping — x64sc is not installed");
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
        eprintln!("AC-100: skipping — x64sc is not installed");
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
        eprintln!("AC-101: skipping — x64sc is not installed");
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

// --- the look: layout C (AC-104..AC-111, D-040, D-041) -----------------------------------------
//
// The menu draws from a RAM copy of the ROM's mixed-case set at $3800 that carries its own glyphs:
// type badges, the rule, the scroll track and thumb, and the logo's quarter-block cells. Tests
// find a glyph by its bitmap, so they hold the approved pixels (the "Stidcart Menu Layouts" mockup,
// layout C) and never the code numbers. Colours are read from colour RAM and the VIC's registers,
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
const LOGO_COL: usize = 7;
const NAME_ROW: usize = 3;
/// The dark grey rule under the list. Row 4 is clear: the raster strip shows through it (D-048).
const RULE_ROW: usize = 21;
const STRIP_ROW: usize = 4;
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
    pub const PURPLE: u8 = 4;
    pub const BLUE: u8 = 6;
    pub const YELLOW: u8 = 7;
    pub const ORANGE: u8 = 8;
    pub const LIGHT_RED: u8 = 10;
    pub const DARK_GREY: u8 = 11;
    pub const GREY: u8 = 12;
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

/// Block letters on a 6×6 grid of quarter-cells: three characters wide and three tall.
fn letter(c: char) -> [&'static str; 6] {
    match c {
        'S' => [".XXXXX", "XX....", ".XXXX.", "....XX", "....XX", "XXXXX."],
        'T' => ["XXXXXX", "..XX..", "..XX..", "..XX..", "..XX..", "..XX.."],
        'I' => [".XXXX.", "..XX..", "..XX..", "..XX..", "..XX..", ".XXXX."],
        'D' => ["XXXXX.", "XX..XX", "XX..XX", "XX..XX", "XX..XX", "XXXXX."],
        'C' => [".XXXXX", "XX....", "XX....", "XX....", "XX....", ".XXXXX"],
        'A' => [".XXXX.", "XX..XX", "XX..XX", "XXXXXX", "XX..XX", "XX..XX"],
        'R' => ["XXXXX.", "XX..XX", "XX..XX", "XXXXX.", "XX.XX.", "XX..XX"],
        other => panic!("no block letter for {other}"),
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

/// The 8×8 bitmap of cell (cx, cy) of a block letter: four quarter-cells of 4×4 pixels.
fn letter_cell(c: char, cx: usize, cy: usize) -> [u8; 8] {
    let g = letter(c);
    let mut out = [0u8; 8];
    for (y, row) in out.iter_mut().enumerate() {
        for x in 0..8 {
            if g[cy * 2 + y / 4].as_bytes()[cx * 2 + x / 4] == b'X' {
                *row |= 0x80 >> x;
            }
        }
    }
    out
}

const LOGO: &str = "STIDCART";
/// White beside the colourful mark, as the app's wordmark (D-048).
const LOGO_COLOURS: [u8; 3] = [colour::WHITE; 3];

/// Every glyph the menu adds, without repeats: icons, rule, track, thumb and the logo's cells.
fn own_glyphs() -> Vec<[u8; 8]> {
    let mut all = badge_halves();
    all.extend([bits(&RULE), bits(&TRACK), bits(&THUMB)]);
    for c in LOGO.chars() {
        for cy in 0..3 {
            for cx in 0..3 {
                all.push(letter_cell(c, cx, cy));
            }
        }
    }
    let mut seen = Vec::new();
    for g in all {
        if g != [0; 8] && !seen.contains(&g) {
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

/// What every menu screen has: black, the RAM set, the logo, the cart name, both rules and the help
/// bar.
fn check_frame(l: &Look, title: &str) -> Result<(), String> {
    if (l.border, l.background) != (colour::BLACK, colour::BLACK) {
        return Err(format!(
            "border {} and background {} are not black",
            l.border, l.background
        ));
    }
    if l.charset != RAM_CHARSET {
        return Err(format!("$D018 is {:02x}, not the RAM set", l.charset));
    }
    for (i, c) in LOGO.chars().enumerate() {
        for (cy, &row_colour) in LOGO_COLOURS.iter().enumerate() {
            for cx in 0..3 {
                let cell = cy * COLS + LOGO_COL + i * 4 + cx;
                let want = letter_cell(c, cx, cy);
                if l.bitmap(cell) != want {
                    return Err(format!(
                        "logo letter {c}, cell {cx},{cy} shows the wrong pixels"
                    ));
                }
                if want != [0; 8] && l.colour[cell] != row_colour {
                    return Err(format!("logo row {cy} is colour {}", l.colour[cell]));
                }
            }
        }
    }
    if !row_shows(&l.screen, NAME_ROW, title) {
        return Err(format!("row 3 does not show the cart name '{title}'"));
    }
    for c in 0..COLS {
        let cell = RULE_ROW * COLS + c;
        if l.bitmap(cell) != bits(&RULE) || l.colour[cell] != colour::DARK_GREY {
            return Err(format!(
                "row {RULE_ROW} column {c} is not the dark grey rule"
            ));
        }
    }
    if let Some(c) = (0..COLS).find(|&c| l.screen[STRIP_ROW * COLS + c] != 0x20) {
        return Err(format!(
            "row {STRIP_ROW} column {c} is not a space under the strip"
        ));
    }
    if !vice::is_reversed(&l.screen, HELP_ROW * COLS, COLS) {
        return Err("the help bar is not reversed across all 40 columns".into());
    }
    if let Some(c) = (0..COLS).find(|&c| l.colour[HELP_ROW * COLS + c] != colour::GREY) {
        return Err(format!("help bar column {c} is not grey"));
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
        screen[LOGO_COL + i] = *b;
    }
    for (i, b) in vice::screen_text("plain").iter().enumerate() {
        screen[NAME_ROW * COLS + 17 + i] = *b;
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
    assert!(check_frame(&plain, "plain").is_err());
    let black = Look {
        border: colour::BLACK,
        background: colour::BLACK,
        charset: RAM_CHARSET,
        ..plain
    };
    assert!(check_frame(&black, "plain").is_err());
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
        eprintln!("AC-105: skipping — x64sc is not installed");
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
        if !OWN_CODES.contains(&(code & 0x7f)) {
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
}

// AC-106: the logo, the cart name, the rules and the help bar, on a black screen, empty cart too
#[test]
fn ac_106_the_frame_has_the_logo_rules_and_help_bar() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-106: skipping — x64sc is not installed");
        return;
    }
    let dir = scratch("ac106");
    let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
    let inputs = ["alpha", "bravo"].map(|n| write(&dir, &format!("{n}.prg"), &hello));
    let cart = build(&dir, "frame test", &[], &inputs);
    let l = looks_after(&dir, &cart, &[vec![]]).remove(0);
    check_frame(&l, "frame test").unwrap_or_else(|e| panic!("{e}"));

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
    check_frame(&l, "empty").unwrap_or_else(|e| panic!("empty cart: {e}"));
}

// AC-107: each entry has its kind's badge in its kind's colour, and the selection reverses it
#[test]
fn ac_107_entries_carry_their_kind_icon() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-107: skipping — x64sc is not installed");
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
                        l.screen[cell] & 0x80 != 0 && l.colour[cell] == colour::ORANGE,
                        "{what}: '{name}': column {c} is not in the orange bar"
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
                    name_colours.iter().all(|&x| x == colour::LIGHT_GREY),
                    "{what}: '{name}' is not light grey"
                );
            }
        }
    }
}

// AC-108: the pane shows the selection's name, type, size, load address or bank, and how it starts
#[test]
fn ac_108_the_pane_describes_the_selection() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-108: skipping — x64sc is not installed");
        return;
    }
    let dir = scratch("ac108");
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
    for (l, name, facts) in [
        (
            &looks[0],
            "low",
            [
                ("type", "program"),
                ("size", "1K"),
                ("load", "$07F0"),
                ("start", "RUN"),
            ],
        ),
        (
            &looks[1],
            "high",
            [
                ("type", "program"),
                ("size", "1K"),
                ("load", "$C000"),
                ("start", "SYS 49152"),
            ],
        ),
        (
            &looks[2],
            "eight",
            [
                ("type", "8K cart"),
                ("size", "8K"),
                ("bank", bank.as_str()),
                ("start", "reset"),
            ],
        ),
    ] {
        let at = l
            .colours_of(6, PANE_COL, name)
            .unwrap_or_else(|| panic!("the pane does not name '{name}'"));
        assert!(
            at.iter().all(|&c| c == colour::WHITE),
            "'{name}': the name is not white"
        );
        for (i, (key, value)) in facts.iter().enumerate() {
            let n = 8 + 2 * i;
            let label = l
                .colours_of(n, PANE_COL, key)
                .unwrap_or_else(|| panic!("'{name}': row {n} has no '{key}'"));
            assert!(
                label.iter().all(|&c| c == colour::GREY),
                "'{name}': '{key}' is not grey"
            );
            assert_eq!(
                l.value(n),
                value_cells(value),
                "'{name}': {key} is not exactly '{value}'"
            );
        }
    }
}

// AC-109: a thumb on the track in column 21 shows the share of the list in view and where it is
#[test]
fn ac_109_the_scroll_bar_shows_where_the_view_is() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-109: skipping — x64sc is not installed");
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
        eprintln!("AC-110: skipping — x64sc is not installed");
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
            .all(|&c| c == colour::GREY)
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
        eprintln!("AC-111: skipping — x64sc is not installed");
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
        eprintln!("AC-104: skipping — x64sc is not installed");
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

// --- the raster brand (AC-123..AC-128, D-048) ----------------------------------------------------
//
// The strip and the mark's label are made by a raster interrupt, so screen and colour RAM can't
// show them: these read VICE's own picture of the frame (`Step::Shot`). Each waits two interrupts
// first, because VICE's picture of the frame in progress still holds the previous frame below the
// beam. In the 384x272 PAL capture, raster line L is image row L - 16 and the 320 visible pixels
// start at column 32.

const SHOT_SIZE: (usize, usize) = (384, 272);
const SHOT_X: usize = 32;
const SHOT_FIRST_LINE: usize = 16;
/// The raster line where character row 0 starts, with the default vertical scroll.
const ROW0_LINE: usize = 51;
const STRIP_LINES: std::ops::RangeInclusive<usize> = 84..=88;
const BANDS: [u8; 5] = [
    colour::LIGHT_RED,
    colour::ORANGE,
    colour::YELLOW,
    colour::LIGHT_GREEN,
    colour::LIGHT_BLUE,
];
/// The mark's cells: columns 2-4 of rows 0-2, codes $EA-$F2 row by row (D-048).
const MARK_COL: usize = 2;
const MARK_CODES: std::ops::Range<usize> = 0xea..0xf3;

/// The mark in multicolour pixels, 12 wide (each two screen pixels) by 24 lines: `S` the shell
/// (`$D023`, dark grey), `L` the label (`$D022`, a band every three lines), `P` a pin (the cells'
/// own colour, yellow).
const MARK: [&str; 24] = [
    "............",
    ".SSSSSSSSSS.",
    "SSSSSSSSSSSS",
    "SSLLLLLLLLSS",
    "SSLLLLLLLLSS",
    "SSLLLLLLLLSS",
    "SSLLLLLLLLSS",
    "SSLLLLLLLLSS",
    "SSLLLLLLLLSS",
    "SSLLLLLLLLSS",
    "SSLLLLLLLLSS",
    "SSLLLLLLLLSS",
    "SSLLLLLLLLSS",
    "SSLLLLLLLLSS",
    "SSLLLLLLLLSS",
    "SSLLLLLLLLSS",
    "SSLLLLLLLLSS",
    "SSLLLLLLLLSS",
    "SSSSSSSSSSSS",
    ".SSSSSSSSSS.",
    "...SSSSSS...",
    "...SPSSPS...",
    "...SPSSPS...",
    "............",
];

/// The image row of raster line `line`.
fn shot_y(line: usize) -> usize {
    line - SHOT_FIRST_LINE
}

/// The colour the mark shows at multicolour pixel (x, y).
fn mark_colour(x: usize, y: usize) -> u8 {
    match MARK[y].as_bytes()[x] {
        b'S' => colour::DARK_GREY,
        b'P' => colour::YELLOW,
        b'L' => BANDS[(y - 3) / 3],
        _ => colour::BLACK,
    }
}

/// The eight bitmap bytes of mark cell `n` (0-8, row by row): two bits a multicolour pixel.
fn mark_cell(n: usize) -> [u8; 8] {
    let (cx, cy) = (n % 3, n / 3);
    let mut out = [0u8; 8];
    for (ly, byte) in out.iter_mut().enumerate() {
        for x in 0..4 {
            let bits = match MARK[cy * 8 + ly].as_bytes()[cx * 4 + x] {
                b'L' => 1,
                b'S' => 2,
                b'P' => 3,
                _ => 0,
            };
            *byte |= bits << (6 - 2 * x);
        }
    }
    out
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

// AC-123: the logo is white from column 7, the selection bar orange, and row 4 clear for the strip
#[test]
fn ac_123_the_logo_is_white_the_bar_orange_and_row_4_clear() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-123: skipping — x64sc is not installed");
        return;
    }
    let dir = scratch("ac123");
    let cart = programs_cart(&dir, "brand", 3);
    let l = looks_after(&dir, &cart, &[vec![]]).remove(0);
    check_frame(&l, "brand").unwrap_or_else(|e| panic!("{e}"));
    let first = LIST_ROWS.start;
    for c in SELECTION_BAR {
        assert_eq!(
            l.colour[first * COLS + c],
            colour::ORANGE,
            "column {c} of the selection is not orange"
        );
    }
}

// AC-124: scanlines 84-88 are the five bands across the visible width, 83 and 89 black
#[test]
fn ac_124_a_raster_strip_runs_under_the_cart_name() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-124: skipping — x64sc is not installed");
        return;
    }
    let dir = scratch("ac124");
    let cart = programs_cart(&dir, "strip", 3);
    let shot = shot_after_boot(&dir, &cart);
    let line = |l: usize| -> Vec<u8> {
        (SHOT_X..SHOT_X + 320)
            .map(|x| shot.at(x, shot_y(l)))
            .collect()
    };
    for (l, want) in STRIP_LINES.zip(BANDS) {
        assert!(
            line(l).iter().all(|&c| c == want),
            "line {l} is not colour {want} across: {:?}",
            line(l)
        );
    }
    for l in [STRIP_LINES.start() - 1, STRIP_LINES.end() + 1] {
        assert!(
            line(l).iter().all(|&c| c == colour::BLACK),
            "line {l} is not black"
        );
    }
}

// AC-125: the cartridge mark in columns 2-4 of rows 0-2, nothing else in rows 0-3 coloured
#[test]
fn ac_125_the_cartridge_mark_heads_the_menu() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-125: skipping — x64sc is not installed");
        return;
    }
    let dir = scratch("ac125");
    let cart = programs_cart(&dir, "mark", 3);
    let dumps = boot_or_panic(
        &dir,
        &cart,
        &[
            Step::Labels(vice::menu_labels()),
            idle(),
            Step::Break(Target::Label("irq")),
            Step::Break(Target::Label("irq")),
            Step::Shot,
            screen(),
            Step::Dump(CHARSET.0, CHARSET.1),
        ],
    );
    let (shot, scr, chars) = (vice::Shot::parse(&dumps[0]), &dumps[1], &dumps[2]);
    assert_eq!(
        (shot.width, shot.height),
        SHOT_SIZE,
        "not VICE's PAL capture"
    );

    for (n, code) in MARK_CODES.enumerate() {
        let (cx, cy) = (n % 3, n / 3);
        assert_eq!(
            scr[cy * COLS + MARK_COL + cx] as usize,
            code,
            "mark cell {cx},{cy} is not code ${code:02x}"
        );
        assert_eq!(
            chars[code * 8..code * 8 + 8],
            mark_cell(n),
            "code ${code:02x} does not hold mark cell {n}"
        );
    }
    let left = SHOT_X + MARK_COL * 8;
    for (y, row) in MARK.iter().enumerate() {
        for x in 0..row.len() {
            let want = mark_colour(x, y);
            for half in 0..2 {
                let got = shot.at(left + 2 * x + half, shot_y(ROW0_LINE + y));
                assert_eq!(got, want, "mark pixel {x},{y} is {got}, not {want}");
            }
        }
    }
    let mark = left..left + 24;
    for l in ROW0_LINE..ROW0_LINE + 32 {
        for x in SHOT_X..SHOT_X + 320 {
            let c = shot.at(x, shot_y(l));
            if !mark.contains(&x) {
                assert!(
                    [colour::BLACK, colour::WHITE, colour::LIGHT_GREY].contains(&c),
                    "line {l} pixel {x} is colour {c} outside the mark"
                );
            }
        }
    }
}

// AC-126: the strip and the mark hold still while the list scrolls, filters and sorts
#[test]
fn ac_126_the_strip_and_mark_are_steady_while_the_menu_redraws() {
    if !vice::require_tool("x64sc") {
        eprintln!("AC-126: skipping — x64sc is not installed");
        return;
    }
    let dir = scratch("ac126");
    let cart = programs_cart(&dir, "steady", 24);
    const FRAMES: usize = 20;
    let groups = [
        vec![key::DOWN; 10],
        [vec![key::DOWN; 7], b"E".to_vec(), vec![key::DEL, key::F1]].concat(),
    ];
    let mut steps = vec![Step::Labels(vice::menu_labels())];
    for keys in &groups {
        steps.push(idle());
        steps.push(Step::Keys(keys.clone()));
        for _ in 0..FRAMES {
            steps.push(Step::Break(Target::Label("irq")));
            steps.push(Step::Shot);
        }
    }
    let shots: Vec<vice::Shot> = boot_or_panic(&dir, &cart, &steps)
        .iter()
        .map(|b| vice::Shot::parse(b))
        .collect();
    let top = |s: &vice::Shot| -> Vec<u8> {
        (shot_y(ROW0_LINE)..=shot_y(*STRIP_LINES.end() + 1))
            .flat_map(|y| (0..s.width).map(move |x| (x, y)))
            .map(|(x, y)| s.at(x, y))
            .collect()
    };
    let rest = |s: &vice::Shot| -> Vec<u8> {
        (shot_y(*STRIP_LINES.end() + 2)..s.height)
            .flat_map(|y| (0..s.width).map(move |x| (x, y)))
            .map(|(x, y)| s.at(x, y))
            .collect()
    };
    let first = top(&shots[0]);
    for (i, s) in shots.iter().enumerate() {
        assert!(top(s) == first, "frame {i}: the strip or the mark changed");
    }
    let mut seen: Vec<Vec<u8>> = Vec::new();
    for s in &shots {
        let r = rest(s);
        if !seen.contains(&r) {
            seen.push(r);
        }
    }
    assert!(
        seen.len() > 10,
        "only {} distinct screens below the strip: the frames did not catch the redraws",
        seen.len()
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
        eprintln!("AC-127: skipping — x64sc is not installed");
        return;
    }
    const STOPS: usize = 60;
    let k8 = markers::cart(CartKind::K8, "BANK WATCH CART");
    for (name, keys) in [
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
        ),
        ("cart", vec![key::RETURN]),
        ("stop", vec![key::DOWN, key::STOP]),
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
        ),
    ] {
        let dir = scratch(&format!("ac127-{name}"));
        let mut inputs = vec![write(&dir, "k8.crt", &k8.bytes)];
        let hello = std::fs::read(fixtures().join("hello.prg")).unwrap();
        inputs.extend((0..3).map(|i| write(&dir, &format!("prog {i}.prg"), &hello)));
        let cart = build(&dir, "watch", &[], &inputs);
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
        eprintln!("AC-128: skipping — x64sc is not installed");
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
        eprintln!("AC-136: skipping — x64sc is not installed");
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
        eprintln!("AC-137: skipping — x64sc is not installed");
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
        eprintln!("AC-138: skipping — x64sc is not installed");
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
        eprintln!("AC-139: skipping — x64sc is not installed");
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
        eprintln!("AC-140: skipping — x64sc is not installed");
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
        eprintln!("AC-141: skipping — x64sc is not installed");
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
