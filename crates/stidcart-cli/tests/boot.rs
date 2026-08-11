//! AC-4 (CLI) — a produced image really boots.
//!
//! This is the only check here that does not lean on the differential oracle. Byte parity says
//! the output matches a tool known to work; this says these bytes work. When the oracle stops
//! applying — custom firmware, formats the reference tool cannot produce — this is what is left.
//!
//! Skips rather than fails when `x64sc` is absent, so VICE is not a build dependency.
//!
//! The monitor recipe below (breakpoint, ignore count, `bsave` of screen RAM) is taken from
//! ndefpack's `tools/vice-boot-test.sh`, whose header records several traps already paid for
//! once. Two of them matter here and are not obvious:
//!
//!   * `x64sc` needs a tty for the monitor, so it runs under `script`. BSD `script` calls
//!     `tcgetattr` on *its own stdin* first and macOS refuses that for a socket — which is what
//!     an agent harness often supplies — so stdin is explicitly null here.
//!   * `x64sc` exits non-zero even on a clean run under `-limitcycles`. The screen dump is the
//!     result; the exit status is not.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const LIMIT_CYCLES: &str = "25000000";
const IGNORE_INSNS: &str = "$100000";
const ENTRY: &str = "vicetest";

fn have_x64sc() -> bool {
    Command::new("sh")
        .args(["-c", "command -v x64sc >/dev/null 2>&1"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

/// The EasyLoader draws entry names with its own font, whose codes are the PETSCII of the name.
/// Screen RAM is compared with bit 7 masked off, because bit 7 marks the selected row and whether
/// an entry happens to be highlighted is not what is under test.
fn name_pattern(name: &str) -> Vec<u8> {
    name.bytes().map(|b| b.wrapping_sub(32) % 128).collect()
}

fn screen_matches(screen: &[u8], pattern: &[u8]) -> bool {
    let masked: Vec<u8> = screen.iter().map(|b| b % 128).collect();
    masked.windows(pattern.len()).any(|w| w == pattern)
}

/// Boot `cart` and return the 1001-byte screen RAM dump, or `None` if the machine never got
/// far enough to reach the breakpoint.
fn boot(dir: &Path, cart: &Path) -> Option<Vec<u8>> {
    let screen = dir.join("screen.bin");
    let _ = std::fs::remove_file(&screen);

    let mon = dir.join("mon.txt");
    std::fs::write(
        &mon,
        format!(
            "break exec $0000 $ffff\nignore 1 {IGNORE_INSNS}\ng\nbsave \"screen.bin\" 0 $0400 $07e8\nquit\n"
        ),
    )
    .unwrap();

    // -default ignores any saved vicerc so the run does not depend on it; +saveres then keeps
    // the reset settings from being written back over it.
    let status = Command::new("script")
        .arg("-q")
        .arg("/dev/null")
        .arg("x64sc")
        // +easyflashcrtwrite is not optional: x64sc defaults to saving the emulated flash back
        // over the image on exit, stamping its own name into the header. Harmless in a scratch
        // dir, but the day this is pointed at a fixture it would silently rewrite it — and a
        // rewritten fixture breaks the parity test with no obvious cause.
        .args(["-default", "+saveres", "+easyflashcrtwrite", "-cartcrt"])
        .arg(cart)
        .args([
            "-warp",
            "-sounddev",
            "dummy",
            "-jamaction",
            "1",
            "-limitcycles",
            LIMIT_CYCLES,
            "-nativemonitor",
            "+confirmonexit",
            "-moncommands",
        ])
        .arg(&mon)
        .current_dir(dir)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    // Deliberately ignored: a clean run exits non-zero under -limitcycles.
    let _ = status;

    std::fs::read(&screen).ok()
}

#[test]
fn ac_4_cli_produced_image_boots_and_menu_lists_entries() {
    if !have_x64sc() {
        eprintln!("AC-4 (CLI): skipping — x64sc is not installed");
        return;
    }

    let dir = std::env::temp_dir().join("stidcart-boot-ac4");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let src = dir.join(format!("{ENTRY}.prg"));
    std::fs::copy(fixtures().join("hello.prg"), &src).unwrap();
    let cart = dir.join("boot.crt");

    assert!(Command::new(env!("CARGO_BIN_EXE_stidcart"))
        .args(["build", "-e", "boot test", "-o"])
        .arg(&cart)
        .arg(&src)
        .status()
        .unwrap()
        .success());

    let screen = boot(&dir, &cart).unwrap_or_else(|| {
        panic!(
            "x64sc never reached the screen dump: the cart did not run, or x64sc did not start. \
             Scratch dir left at {} — reproduce by hand before suspecting the cart.",
            dir.display()
        )
    });
    assert_eq!(screen.len(), 1001, "unexpected screen dump size");

    assert!(
        screen_matches(&screen, &name_pattern(ENTRY)),
        "the entry name is not on the EasyLoader menu; the cart booted but the directory did not render"
    );

    // Negative control. Without it this test passes on any screen that happens to contain the
    // name, and a regression that stops rendering the menu entirely could still look green.
    let mut corrupt = std::fs::read(&cart).unwrap();
    let tail = corrupt.len() - 1;
    corrupt[0x40..tail].fill(0x00);
    let broken = dir.join("broken.crt");
    std::fs::write(&broken, &corrupt).unwrap();

    if let Some(screen) = boot(&dir, &broken) {
        assert!(
            !screen_matches(&screen, &name_pattern(ENTRY)),
            "a deliberately corrupted cart still showed the entry name; this test cannot fail"
        );
    }
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn name_pattern_is_the_petscii_of_the_name() {
        assert_eq!(name_pattern("hello"), b"HELLO".to_vec());
    }

    #[test]
    fn screen_matching_ignores_the_selection_bit() {
        let mut screen = vec![0u8; 40];
        for (i, b) in b"HELLO".iter().enumerate() {
            screen[10 + i] = b | 0x80;
        }
        assert!(screen_matches(&screen, &name_pattern("hello")));
        assert!(!screen_matches(&screen, &name_pattern("absent")));
    }
}
