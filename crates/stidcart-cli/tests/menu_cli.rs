//! custom-menu: choosing, refusing and recognising our own menu through the binary (AC-81..AC-85,
//! AC-95, CLI surface). menu-highmem narrows the refusal to what our menu still can't start (AC-163).
//!
//! Every case works in its own scratch directory, because `edit` writes over the cart it was given
//! by default.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use stidcart_core::menu::STIDCART_MENU;
use stidcart_core::{crt, firmware, read};

/// The menu loader's slot: bank 0 ROML, the first 8 KiB of the image.
const SLOT: std::ops::Range<usize> = 0..0x2000;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn fx(name: &str) -> PathBuf {
    fixtures().join(name)
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("stidcart-menu-cli-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// A fixture copied into `dir`, so an edit never touches `fixtures/`.
fn copy(dir: &Path, fixture: &str) -> PathBuf {
    let to = dir.join(fixture);
    std::fs::copy(fx(fixture), &to).unwrap();
    to
}

fn stidcart<I, S>(args: I) -> Output
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    Command::new(env!("CARGO_BIN_EXE_stidcart"))
        .args(args)
        .output()
        .unwrap()
}

fn ok(out: &Output) -> String {
    assert!(
        out.status.success(),
        "stidcart failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

fn refused(out: &Output) -> String {
    assert!(
        !out.status.success(),
        "stidcart accepted a request it should refuse"
    );
    String::from_utf8_lossy(&out.stderr).into_owned()
}

fn image(path: &Path) -> Vec<u8> {
    crt::parse(&std::fs::read(path).unwrap())
        .unwrap()
        .image
        .as_bytes()
        .to_vec()
}

fn header(path: &Path) -> String {
    crt::parse(&std::fs::read(path).unwrap())
        .unwrap()
        .header_name
}

fn easyloader_slot() -> &'static [u8] {
    firmware::body(firmware::LOADER_NRM)
}

/// A `.prg` with the given load address and `len` payload bytes.
fn prg(dir: &Path, name: &str, load: u16, len: usize) -> PathBuf {
    let mut bytes = load.to_le_bytes().to_vec();
    bytes.extend((0..len).map(|i| i as u8));
    let path = dir.join(format!("{name}.prg"));
    std::fs::write(&path, bytes).unwrap();
    path
}

/// `build -e "menu test" -o out [extra…] inputs…`
fn build(out: &Path, extra: &[&str], inputs: &[PathBuf]) -> Output {
    let mut args: Vec<&OsStr> = vec![
        OsStr::new("build"),
        OsStr::new("-e"),
        OsStr::new("menu test"),
        OsStr::new("-o"),
        out.as_os_str(),
    ];
    args.extend(extra.iter().map(OsStr::new));
    args.extend(inputs.iter().map(|p| p.as_os_str()));
    stidcart(args)
}

/// `edit cart [args…]`
fn edit(cart: &Path, args: &[&OsStr]) -> Output {
    let mut all: Vec<&OsStr> = vec![OsStr::new("edit"), cart.as_os_str()];
    all.extend_from_slice(args);
    stidcart(all)
}

fn s(text: &str) -> &OsStr {
    OsStr::new(text)
}

/// A cart of two programs carrying our menu.
fn ours(dir: &Path) -> PathBuf {
    let out = dir.join("ours.crt");
    ok(&build(
        &out,
        &["--menu", "stidcart"],
        &[fx("hello.prg"), fx("ramp.prg")],
    ));
    out
}

// AC-81 (CLI): build --menu stidcart puts our menu in the loader slot and changes nothing else
#[test]
fn ac_81_cli_build_with_our_menu_changes_only_the_loader_slot() {
    let dir = scratch("ac81");
    let inputs = [
        fx("hello.prg"),
        fx("ramp.prg"),
        fx("in_8k.crt"),
        fx("in_xb16.crt"),
    ];
    let with = dir.join("with.crt");
    let without = dir.join("without.crt");
    ok(&build(&with, &["--menu", "stidcart"], &inputs));
    ok(&build(&without, &[], &inputs));

    let (a, b) = (image(&with), image(&without));
    assert_eq!(a.len(), b.len());
    assert_eq!(
        &a[SLOT], STIDCART_MENU,
        "the loader slot does not hold our menu"
    );
    assert_eq!(&b[SLOT], easyloader_slot());
    assert_eq!(
        a[SLOT.end..],
        b[SLOT.end..],
        "a byte outside the loader slot changed"
    );
    assert_eq!(header(&with), header(&without));
}

// AC-82 (CLI): no request means EasyLoader and unchanged bytes; edit keeps either menu
#[test]
fn ac_82_cli_no_request_keeps_easyloader_and_edit_keeps_the_menu() {
    let dir = scratch("ac82");

    // A build without --menu still matches the frozen reference image.
    let out = dir.join("two.crt");
    ok(&stidcart([
        s("build"),
        s("-e"),
        s("two entries"),
        s("-o"),
        out.as_os_str(),
        fx("hello.prg").as_os_str(),
        fx("ramp.prg").as_os_str(),
    ]));
    assert_eq!(image(&out), image(&fx("two.crt")));

    // An edit without --menu keeps EasyLoader…
    let theirs = copy(&dir, "two.crt");
    ok(&edit(&theirs, &[s("--rename"), s("0"), s("renamed")]));
    assert_eq!(&image(&theirs)[SLOT], easyloader_slot());

    // …and keeps our menu.
    let mine = ours(&dir);
    ok(&edit(&mine, &[s("--rename"), s("0"), s("renamed")]));
    assert_eq!(&image(&mine)[SLOT], STIDCART_MENU);
}

// AC-83 (CLI): edit switches the menu either way, and a round trip restores the original bytes
#[test]
fn ac_83_cli_edit_switches_the_menu_and_back() {
    let dir = scratch("ac83");
    let original = copy(&dir, "two.crt");
    let switched = dir.join("switched.crt");
    let back = dir.join("back.crt");

    ok(&edit(
        &original,
        &[s("--menu"), s("stidcart"), s("-o"), switched.as_os_str()],
    ));
    let (before, after) = (image(&original), image(&switched));
    assert_eq!(&after[SLOT], STIDCART_MENU);
    assert_eq!(
        before[SLOT.end..],
        after[SLOT.end..],
        "switching the menu changed something else"
    );

    ok(&edit(
        &switched,
        &[s("--menu"), s("easyloader"), s("-o"), back.as_os_str()],
    ));
    assert_eq!(
        std::fs::read(&back).unwrap(),
        std::fs::read(&original).unwrap(),
        "switching to our menu and back did not give the original bytes"
    );
}

// AC-84 (CLI): our menu with an Ocean entry is refused naming it, nothing written
#[test]
fn ac_84_cli_our_menu_with_an_ocean_entry_is_refused() {
    let dir = scratch("ac84");

    // build
    let out = dir.join("out.crt");
    let err = refused(&build(
        &out,
        &["--menu", "stidcart"],
        &[fx("hello.prg"), fx("in_ocean.crt")],
    ));
    assert!(
        err.contains("in_ocean"),
        "the refusal does not name the entry: {err}"
    );
    assert!(!out.exists(), "a refused build wrote a file");

    // edit --menu on a cart holding an Ocean entry
    let cocean = copy(&dir, "cocean.crt");
    let before = std::fs::read(&cocean).unwrap();
    let ocean_name = read::inspect(&before)
        .unwrap()
        .entries
        .iter()
        .find(|e| e.kind.is_ocean())
        .expect("cocean.crt holds an Ocean entry")
        .name
        .clone();
    let err = refused(&edit(&cocean, &[s("--menu"), s("stidcart")]));
    assert!(
        err.contains(&ocean_name),
        "the refusal does not name '{ocean_name}': {err}"
    );
    assert_eq!(
        std::fs::read(&cocean).unwrap(),
        before,
        "a refused edit wrote the cart"
    );

    // edit --add of an Ocean image onto a cart carrying our menu
    let mine = ours(&dir);
    let before = std::fs::read(&mine).unwrap();
    let err = refused(&edit(&mine, &[s("--add"), fx("in_ocean.crt").as_os_str()]));
    assert!(
        err.contains("in_ocean"),
        "the refusal does not name the entry: {err}"
    );
    assert_eq!(
        std::fs::read(&mine).unwrap(),
        before,
        "a refused edit wrote the cart"
    );
}

// AC-95 (CLI): our menu with a program loading into $D000-$DFFF is refused naming it, nothing
// written. menu-highmem (AC-163) keeps this refusal for a program that loads there and lifts it for
// one that only reaches it from below.
#[test]
fn ac_95_cli_our_menu_with_a_program_in_the_io_area_is_refused() {
    let dir = scratch("ac95");
    // $D000 + 0x200 bytes loads into I/O; $CE00 + 0x200 ends exactly at $CFFF.
    let io = prg(&dir, "iozone", 0xd000, 0x200);
    let below = prg(&dir, "below", 0xce00, 0x200);

    // Control: the boundary itself is accepted.
    ok(&build(
        &dir.join("fine.crt"),
        &["--menu", "stidcart"],
        std::slice::from_ref(&below),
    ));

    // build
    let out = dir.join("out.crt");
    let err = refused(&build(
        &out,
        &["--menu", "stidcart"],
        std::slice::from_ref(&io),
    ));
    assert!(
        err.contains("iozone"),
        "the refusal does not name the entry: {err}"
    );
    assert!(!out.exists(), "a refused build wrote a file");

    // edit --menu on an EasyLoader cart that holds such a program
    let theirs = dir.join("theirs.crt");
    ok(&build(&theirs, &[], &[fx("hello.prg"), io.clone()]));
    let before = std::fs::read(&theirs).unwrap();
    let err = refused(&edit(&theirs, &[s("--menu"), s("stidcart")]));
    assert!(
        err.contains("iozone"),
        "the refusal does not name the entry: {err}"
    );
    assert_eq!(std::fs::read(&theirs).unwrap(), before);

    // edit --add and --replace on a cart carrying our menu
    let mine = ours(&dir);
    let before = std::fs::read(&mine).unwrap();
    let err = refused(&edit(&mine, &[s("--add"), io.as_os_str()]));
    assert!(
        err.contains("iozone"),
        "the --add refusal does not name the entry: {err}"
    );
    let err = refused(&edit(&mine, &[s("--replace"), s("0"), io.as_os_str()]));
    assert!(
        err.contains("hello"),
        "the --replace refusal does not name the entry: {err}"
    );
    assert_eq!(
        std::fs::read(&mine).unwrap(),
        before,
        "a refused edit wrote the cart"
    );
}

/// One past the last byte of the routine our menu puts at `$0100` to copy `$D000-$FFFF`, from the
/// committed labels: a program reaching `$D000` must load at or above it.
fn high_copy_end() -> u16 {
    let labels =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../menu/menu.lbl"))
            .unwrap();
    labels
        .lines()
        .find_map(|l| l.strip_suffix(" .high_copy_end")?.strip_prefix("al "))
        .map(|a| u16::from_str_radix(a, 16).unwrap())
        .expect("menu.lbl has no .high_copy_end")
}

/// AC-163 (CLI): with our menu, a program loading at `$D000` or above, one ending past `$FFFF`, and
/// one reaching `$D000` while loading below the second copy routine's end are each refused naming
/// the entry, through `build`, `edit --menu`, `--add` and `--replace`, with nothing written. Any
/// other program reaching into `$D000-$FFFF` is accepted, and EasyLoader accepts all of them.
#[test]
fn ac_163_cli_our_menu_refuses_only_what_it_still_cannot_start() {
    let dir = scratch("ac163");

    // Accepted with our menu: the old AC-95 case, an end exactly at $10000, the largest BASIC
    // program the format holds, and one loading right at the copy routine's end.
    let accepted = [
        prg(&dir, "iozone", 0xcf00, 0x200),
        prg(&dir, "attop", 0xc000, 0x4000),
        prg(&dir, "basicmax", 0x0801, 0xdffe),
    ];
    for p in &accepted {
        ok(&build(
            &dir.join("fine.crt"),
            &["--menu", "stidcart"],
            std::slice::from_ref(p),
        ));
    }
    let hce = high_copy_end();
    ok(&build(
        &dir.join("fine.crt"),
        &["--menu", "stidcart"],
        &[prg(&dir, "abovecopy", hce, 0xd001 - hce as usize)],
    ));

    // Refused, each with what its message must say.
    let refused_cases = [
        (prg(&dir, "atio", 0xd000, 0x10), "$D000"),
        (prg(&dir, "atkernal", 0xe000, 0x10), "$D000"),
        (prg(&dir, "pasttop", 0xc000, 0x4001), "$FFFF"),
        (
            prg(&dir, "undercopy", hce - 1, 0xd001 - (hce as usize - 1)),
            "$0100",
        ),
    ];
    for (p, says) in &refused_cases {
        let name = p.file_stem().unwrap().to_str().unwrap();

        // build
        let out = dir.join("out.crt");
        let err = refused(&build(
            &out,
            &["--menu", "stidcart"],
            std::slice::from_ref(p),
        ));
        assert!(
            err.contains(name) && err.contains(says),
            "{name}: the refusal does not name it and say '{says}': {err}"
        );
        assert!(!out.exists(), "{name}: a refused build wrote a file");

        // EasyLoader: nothing about it changes.
        let theirs = dir.join("theirs.crt");
        ok(&build(&theirs, &[], &[fx("hello.prg"), p.clone()]));

        // edit --menu on that EasyLoader cart
        let before = std::fs::read(&theirs).unwrap();
        let err = refused(&edit(&theirs, &[s("--menu"), s("stidcart")]));
        assert!(err.contains(name), "{name}: --menu refusal: {err}");
        assert_eq!(std::fs::read(&theirs).unwrap(), before);

        // edit --add and --replace on a cart carrying our menu
        let mine = ours(&dir);
        let before = std::fs::read(&mine).unwrap();
        let err = refused(&edit(&mine, &[s("--add"), p.as_os_str()]));
        assert!(err.contains(name), "{name}: --add refusal: {err}");
        let err = refused(&edit(&mine, &[s("--replace"), s("0"), p.as_os_str()]));
        assert!(err.contains("hello"), "{name}: --replace refusal: {err}");
        assert_eq!(
            std::fs::read(&mine).unwrap(),
            before,
            "{name}: a refused edit wrote the cart"
        );
    }
}

// AC-85 (CLI): list says which menu the cart carries
#[test]
fn ac_85_cli_list_says_which_menu() {
    let dir = scratch("ac85");

    let theirs = ok(&stidcart([s("list"), fx("two.crt").as_os_str()]));
    assert!(theirs.contains("menu: EasyLoader\n"), "{theirs}");

    let mine = ours(&dir);
    let listed = ok(&stidcart([s("list"), mine.as_os_str()]));
    assert!(listed.contains("menu: stidcart\n"), "{listed}");

    // A loader slot matching neither menu: still described, still refused by edit.
    let mut container = crt::parse(&std::fs::read(fx("two.crt")).unwrap()).unwrap();
    container.image.write_at(0x100, &[0x5a; 16]);
    let odd = dir.join("odd.crt");
    std::fs::write(&odd, crt::write(&container.image, &container.header_name)).unwrap();
    let listed = ok(&stidcart([s("list"), odd.as_os_str()]));
    assert!(listed.contains("menu: unrecognised\n"), "{listed}");
    refused(&edit(&odd, &[s("--rename"), s("0"), s("x")]));
}
