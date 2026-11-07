//! custom-menu: our menu on the C64, proven by booting carts in VICE (AC-87..AC-92), and the CI
//! requirement that these boots run rather than skip (AC-94). menu-search-sort adds its filter and
//! sort toggle (AC-96..AC-101).
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
    for chunk in [key::DOWN; 29].chunks(10) {
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
// The menu keeps the list on rows 2-21, shows the typed text on row 1 and the count and order on
// row 23. Assertions about the list read rows 2-21 only, so a typed query or the status line can
// never stand in for a listed name.

const COLS: usize = 40;
const LIST_ROWS: std::ops::Range<usize> = 2..22;
const QUERY_ROW: usize = 1;
const STATUS_ROW: usize = 23;

fn row(screen: &[u8], n: usize) -> &[u8] {
    &screen[n * COLS..(n + 1) * COLS]
}

fn row_shows(screen: &[u8], n: usize, text: &str) -> bool {
    vice::find(row(screen, n), &vice::screen_text(text)).is_some()
}

/// Where `text` is in the list rows, as an offset into the whole screen.
fn listed(screen: &[u8], text: &str) -> Option<usize> {
    let start = LIST_ROWS.start * COLS;
    vice::find(
        &screen[start..LIST_ROWS.end * COLS],
        &vice::screen_text(text),
    )
    .map(|at| at + start)
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
        row_shows(shown, STATUS_ROW, "2 of 3"),
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
        assert!(row_shows(&dumps[i], STATUS_ROW, "3 of 3"), "{what}: count");
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
    assert!(row_shows(&dumps[1], STATUS_ROW, "3 of 4"), "F1: count");

    let mut keys = vec![key::F1];
    keys.extend(typed("an"));
    let dumps = screens_after(&dir, &cart, &[keys]);
    assert_listed_in_order(&dumps[0], &a_z, "F1, then an");
    assert_not_listed(&dumps[0], &["kiwi"], "F1, then an");
}
