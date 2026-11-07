//! Booting a cart in VICE's `x64sc` and reading memory back, for the tests that no byte oracle can
//! cover (MAP: the oracle decays).
//!
//! The monitor recipe came from ndefpack's `tools/vice-boot-test.sh`, whose header records traps
//! already paid for once:
//!
//!   * `x64sc` needs a tty for the monitor, so it runs under `script`. BSD `script` calls
//!     `tcgetattr` on *its own stdin* first and macOS refuses that for a socket — which is what an
//!     agent harness often supplies — so stdin is explicitly null here. util-linux `script` takes
//!     the command through `-c` instead of as trailing arguments.
//!   * `x64sc` exits non-zero even on a clean run under `-limitcycles`. The dumps are the result;
//!     the exit status is not.
//!   * `+easyflashcrtwrite` is not optional: `x64sc` defaults to saving the emulated flash back over
//!     the image on exit, stamping its own name into the header.
//!
//! Waits are breakpoints, never instruction counts: each `Step::Break` sets one, continues, and
//! deletes it once hit, so the next step's commands run at exactly that point.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// A ceiling on a run that never reaches its last breakpoint. With `-warp` it costs seconds.
const LIMIT_CYCLES: &str = "200000000";

/// PETSCII key codes as GETIN returns them.
pub mod key {
    pub const DOWN: u8 = 0x11;
    pub const UP: u8 = 0x91;
    pub const RETURN: u8 = 0x0d;
    pub const STOP: u8 = 0x03;
}

/// Where a breakpoint goes.
pub enum Target {
    /// A label from a file loaded with [`Step::Labels`].
    Label(&'static str),
    Addr(u16),
}

pub enum Step {
    /// Load a VICE label file (`64tass --vice-labels`).
    Labels(PathBuf),
    /// Run until execution reaches the target.
    Break(Target),
    /// Put key codes in the KERNAL keyboard buffer (`$0277`, count in `$C6`). At most ten fit.
    Keys(Vec<u8>),
    /// Save memory from `start` to `end` inclusive; it becomes the next element of the result.
    Dump(u16, u16),
    /// Monitor lines passed through unchanged, for a recipe the steps above don't express.
    Raw(String),
}

/// Whether `name` is on PATH. Absent, the caller skips — unless `STIDCART_REQUIRE_TOOLS=1`, as CI
/// sets it, where an absent tool fails the test instead (AC-94).
pub fn require_tool(name: &str) -> bool {
    require_tool_in(
        name,
        std::env::var("STIDCART_REQUIRE_TOOLS").as_deref() == Ok("1"),
    )
}

/// [`require_tool`] with the requirement passed in rather than read from the environment.
pub fn require_tool_in(name: &str, required: bool) -> bool {
    let present = Command::new("sh")
        .args(["-c", &format!("command -v {name} >/dev/null 2>&1")])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !present && required {
        panic!("{name} is required (STIDCART_REQUIRE_TOOLS=1) but is not on PATH");
    }
    present
}

/// The committed label file of our menu.
pub fn menu_labels() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../menu/menu.lbl")
}

/// The monitor script for `steps`, and the dump file names in order.
fn script(steps: &[Step]) -> (String, Vec<String>) {
    let mut mon = String::new();
    let mut dumps = Vec::new();
    for step in steps {
        match step {
            Step::Labels(path) => mon.push_str(&format!("ll \"{}\"\n", path.display())),
            Step::Break(target) => {
                let at = match target {
                    Target::Label(l) => format!(".{l}"),
                    Target::Addr(a) => format!("${a:04x}"),
                };
                mon.push_str(&format!("break exec {at}\ng\ndel\n"));
            }
            Step::Keys(keys) => {
                assert!(
                    !keys.is_empty() && keys.len() <= 10,
                    "the keyboard buffer holds 1..=10 keys"
                );
                let bytes: Vec<String> = keys.iter().map(|k| format!("{k:02x}")).collect();
                mon.push_str(&format!(
                    "> $0277 {}\n> $00c6 {:02x}\n",
                    bytes.join(" "),
                    keys.len()
                ));
            }
            Step::Dump(start, end) => {
                let name = format!("dump{}.bin", dumps.len());
                mon.push_str(&format!("bsave \"{name}\" 0 ${start:04x} ${end:04x}\n"));
                dumps.push(name);
            }
            Step::Raw(lines) => {
                mon.push_str(lines);
                if !lines.ends_with('\n') {
                    mon.push('\n');
                }
            }
        }
    }
    mon.push_str("quit\n");
    (mon, dumps)
}

/// Boot `cart` and run `steps`, working in `dir`. Returns every dump in step order, or `None` if
/// the machine never got far enough to write all of them.
pub fn boot(dir: &Path, cart: &Path, steps: &[Step]) -> Option<Vec<Vec<u8>>> {
    let (mon, dumps) = script(steps);
    for d in &dumps {
        let _ = std::fs::remove_file(dir.join(d));
    }
    let mon_path = dir.join("mon.txt");
    std::fs::write(&mon_path, mon).unwrap();

    // -default ignores any saved vicerc so the run does not depend on it.
    let mut args: Vec<String> = [
        "-default",
        "-logfile",
        "vice-internal.log",
        "+easyflashcrtwrite",
        "-warp",
        "-sounddev",
        "dummy",
        "-jamaction",
        "1",
        "-limitcycles",
        LIMIT_CYCLES,
    ]
    .map(String::from)
    .to_vec();
    // These exist only in a build with a UI (VICE 3.10 registers them in arch/gtk3 and arch/sdl),
    // and the headless x64sc CI builds bails out on them. There they are also moot: it saves no
    // settings, asks nothing on exit, and its monitor is the terminal already. +saveres keeps the
    // reset settings from being written back over the user's vicerc; -nativemonitor puts the
    // monitor on the terminal instead of a window.
    for option in ["+saveres", "-nativemonitor", "+confirmonexit"] {
        if has_option(option) {
            args.push(option.to_string());
        }
    }
    args.extend([
        "-cartcrt".to_string(),
        cart.display().to_string(),
        "-moncommands".to_string(),
        mon_path.display().to_string(),
    ]);

    // VICE's own output goes to a log beside the dumps, so a run that never gets far enough says
    // why: the failing test prints its tail.
    let log_path = dir.join("vice.log");
    let log = std::fs::File::create(&log_path).unwrap();
    // Deliberately ignored: a clean run exits non-zero under -limitcycles.
    let _ = under_script(&args)
        .current_dir(dir)
        .stdin(Stdio::null())
        .stdout(log.try_clone().unwrap())
        .stderr(log)
        .status();

    let result: Option<Vec<Vec<u8>>> = dumps
        .iter()
        .map(|d| std::fs::read(dir.join(d)).ok())
        .collect();
    if result.is_none() {
        // Option errors come before VICE opens its log; everything after goes to the log.
        for name in ["vice.log", "vice-internal.log"] {
            let text = std::fs::read_to_string(dir.join(name)).unwrap_or_default();
            let tail: Vec<&str> = text.lines().rev().take(25).collect();
            eprintln!(
                "x64sc did not write every dump; last lines of {}:\n{}",
                dir.join(name).display(),
                tail.into_iter().rev().collect::<Vec<_>>().join("\n")
            );
        }
    }
    result
}

/// Whether this `x64sc` accepts `option`, from its `-help` listing (asked once per test binary).
fn has_option(option: &str) -> bool {
    static HELP: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    let help = HELP.get_or_init(|| {
        Command::new("x64sc")
            .arg("-help")
            .stdin(Stdio::null())
            .output()
            .map(|o| {
                String::from_utf8_lossy(&o.stdout).into_owned()
                    + &String::from_utf8_lossy(&o.stderr)
            })
            .unwrap_or_default()
    });
    help.lines()
        .any(|l| l.split_whitespace().next() == Some(option))
}

/// `x64sc args…` under `script`, which gives it the tty its monitor needs.
#[cfg(target_os = "macos")]
fn under_script(args: &[String]) -> Command {
    let mut cmd = Command::new("script");
    cmd.args(["-q", "/dev/null", "x64sc"]).args(args);
    cmd
}

#[cfg(not(target_os = "macos"))]
fn under_script(args: &[String]) -> Command {
    let quoted: Vec<String> = std::iter::once("x64sc".to_string())
        .chain(
            args.iter()
                .map(|a| format!("'{}'", a.replace('\'', r"'\''"))),
        )
        .collect();
    let mut cmd = Command::new("script");
    cmd.args(["-q", "-e", "-c", &quoted.join(" "), "/dev/null"]);
    cmd
}

/// The screen codes the ROM character set shows for `text` printed through the KERNAL in its
/// default upper-case mode: the text goes through the same ASCII→PETSCII mapping the tool stores
/// names with, then PETSCII→screen code.
pub fn screen_text(text: &str) -> Vec<u8> {
    stidcart_core::petscii::ascii_to_petscii(text)
        .into_iter()
        .map(|p| match p {
            0x40..=0x5f => p - 0x40,
            0x60..=0x7f => p - 0x20,
            0xa0..=0xbf => p - 0x40,
            0xc0..=0xfe => p - 0x80,
            other => other,
        })
        .collect()
}

/// Where `pattern` first appears on `screen`, ignoring bit 7 (reverse video).
pub fn find(screen: &[u8], pattern: &[u8]) -> Option<usize> {
    let masked: Vec<u8> = screen.iter().map(|b| b & 0x7f).collect();
    masked.windows(pattern.len()).position(|w| w == pattern)
}

/// Whether the `len` screen cells from `at` are all shown in reverse video.
pub fn is_reversed(screen: &[u8], at: usize, len: usize) -> bool {
    screen[at..at + len].iter().all(|b| b & 0x80 != 0)
}

#[cfg(test)]
mod unit {
    use super::*;

    #[test]
    fn screen_text_maps_letters_to_the_rom_charset() {
        assert_eq!(screen_text("ab 1."), vec![1, 2, 0x20, 0x31, 0x2e]);
    }

    #[test]
    fn find_ignores_reverse_video_and_is_reversed_sees_it() {
        let mut screen = vec![0x20u8; 40];
        for (i, b) in screen_text("hello").iter().enumerate() {
            screen[10 + i] = b | 0x80;
        }
        assert_eq!(find(&screen, &screen_text("hello")), Some(10));
        assert!(is_reversed(&screen, 10, 5));
        assert!(!is_reversed(&screen, 9, 5));
        assert_eq!(find(&screen, &screen_text("absent")), None);
    }

    #[test]
    fn a_break_step_sets_continues_and_deletes() {
        let (mon, dumps) = script(&[
            Step::Break(Target::Label("menu_idle")),
            Step::Keys(vec![key::DOWN, key::RETURN]),
            Step::Break(Target::Addr(0xc00d)),
            Step::Dump(0x0400, 0x07e7),
        ]);
        assert_eq!(
            mon,
            "break exec .menu_idle\ng\ndel\n> $0277 11 0d\n> $00c6 02\nbreak exec $c00d\ng\ndel\n\
             bsave \"dump0.bin\" 0 $0400 $07e7\nquit\n"
        );
        assert_eq!(dumps, ["dump0.bin"]);
    }
}
