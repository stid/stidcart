//! `stidcart` — the command-line harness over `stidcart-core`.
//!
//! This is a test harness, not a product. It exists so the core can be driven end to end and
//! diffed against a reference implementation; the interface mirrors that tool's `build` closely
//! enough to compare outputs.

use std::path::Path;
use std::process::ExitCode;

use stidcart_core::directory::EntryType;
use stidcart_core::read;
use stidcart_core::{CartBuilder, Program};

const USAGE: &str = "\
usage: stidcart build [-e NAME] -o OUTPUT INPUT.prg...
       stidcart list CART.crt

  build           create a cartridge image from program files
  -e NAME         cartridge name shown by the menu (default: \"(no EF-Name)\")
  -o OUTPUT       where to write the image
  INPUT.prg       programs to include; the entry name is the file stem

  list            show what a cartridge image contains
  CART.crt        the image to read
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(Some(path)) => {
            eprintln!("wrote {path}");
            ExitCode::SUCCESS
        }
        Ok(None) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("stidcart: {msg}");
            ExitCode::FAILURE
        }
    }
}

/// Returns the path written, or `None` when there was nothing to write.
fn run(args: &[String]) -> Result<Option<String>, String> {
    let Some(command) = args.first() else {
        return Err(format!("no command given\n{USAGE}"));
    };
    match command.as_str() {
        "build" => build(&args[1..]),
        "list" => list(&args[1..]).map(|()| None),
        other => Err(format!("unknown command '{other}'\n{USAGE}")),
    }
}

fn build(args: &[String]) -> Result<Option<String>, String> {
    let mut ef_name: Option<String> = None;
    let mut output: Option<String> = None;
    let mut inputs: Vec<String> = Vec::new();

    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-e" => ef_name = Some(it.next().ok_or("-e needs a name")?.clone()),
            "-o" => output = Some(it.next().ok_or("-o needs a path")?.clone()),
            "-h" | "--help" => return Err(USAGE.to_string()),
            other if other.starts_with('-') => return Err(format!("unknown option '{other}'")),
            other => inputs.push(other.to_string()),
        }
    }

    // Matching the reference tool: a request with no programs succeeds and writes nothing at all.
    // Whether that is the right long-term behaviour is a product question, deliberately not
    // decided here.
    if inputs.is_empty() {
        return Ok(None);
    }

    let output = output.ok_or("no output path; pass -o OUTPUT")?;

    let mut builder = CartBuilder::new();
    if let Some(name) = &ef_name {
        builder = builder.with_ef_name(name);
    }
    for path in &inputs {
        let data = std::fs::read(path).map_err(|e| format!("reading '{path}': {e}"))?;
        builder = builder.with_program(Program::new(&entry_name(path), data));
    }

    let bytes = builder.build().map_err(|e| e.to_string())?;
    std::fs::write(&output, &bytes).map_err(|e| format!("writing '{output}': {e}"))?;

    Ok(Some(output))
}

/// Show what a cartridge image contains, as the image itself records it.
///
/// Positions come from [`read::inspect`] rather than from loading and rebuilding: a rebuild
/// re-places every entry, so a listing built on it would print coordinates that describe a
/// hypothetical image rather than the file named on the command line.
fn list(args: &[String]) -> Result<(), String> {
    let mut path: Option<&str> = None;
    for arg in args {
        match arg.as_str() {
            "-h" | "--help" => return Err(USAGE.to_string()),
            other if other.starts_with('-') => return Err(format!("unknown option '{other}'")),
            other if path.is_none() => path = Some(other),
            other => return Err(format!("list takes one image; also given '{other}'")),
        }
    }
    let path = path.ok_or_else(|| format!("no cartridge image given\n{USAGE}"))?;

    let bytes = std::fs::read(path).map_err(|e| format!("reading '{path}': {e}"))?;
    let info = read::inspect(&bytes).map_err(|e| e.to_string())?;

    print!("{}", render(&info));
    Ok(())
}

/// The listing itself, kept separate from the printing so it can be asserted on directly.
fn render(info: &read::CartInfo) -> String {
    let mut out = String::new();
    // The cartridge name and every entry name arrive through the machine's own character mapping,
    // which already turns control bytes into `.`. The container header name does not — it is
    // host-encoded text carried through verbatim, and this is the first path that puts it in front
    // of a terminal, so it is filtered here.
    out.push_str(&format!("cart: {}\n", info.ef_name));
    out.push_str(&format!("built by: {}\n\n", printable(&info.header_name)));

    if info.entries.is_empty() {
        out.push_str("  (no entries)\n");
    } else {
        out.push_str(&format!(
            "  {:>3}  {:<16}  {:<5}  {:<5}  {:>4}  {:>6}  {:>8}\n",
            "#", "name", "flags", "type", "bank", "offset", "size"
        ));
        for (i, e) in info.entries.iter().enumerate() {
            let kind = match e.kind {
                EntryType::Prg => "PRG",
                // The walk refuses every other kind, so a listing cannot reach them. Labelling
                // them would advertise support that no code path here has.
                _ => "?",
            };
            out.push_str(&format!(
                "  {i:>3}  {:<16}  {:<5}  {kind:<5}  {:>4}  {:>6}  {:>8}\n",
                e.name,
                if e.hidden { "H" } else { "-" },
                format!("${:02x}", e.bank),
                format!("${:04x}", e.offset),
                format!("${:06x}", e.size),
            ));
        }
    }

    let free = info.free_space();
    out.push_str(&format!(
        "\n{} {}, ${free:06x} ({free}) bytes free\n",
        info.entries.len(),
        if info.entries.len() == 1 {
            "entry"
        } else {
            "entries"
        }
    ));
    out
}

/// Replace anything that would drive the terminal rather than print on it.
///
/// A `.crt` is untrusted input, and its 32-byte header name is free-form host-encoded text. Left
/// alone, a name holding `\x1b[2J` clears the screen the listing was just written to, so what the
/// reader believes they inspected is not what the file holds.
fn printable(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_control() { '.' } else { c })
        .collect()
}

/// The menu name a program gets when the request does not give one: the file name with its
/// directory and extension removed, case and spacing preserved.
fn entry_name(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_name_strips_directory_and_extension() {
        assert_eq!(entry_name("fixtures/My Game v2.prg"), "My Game v2");
        assert_eq!(entry_name("hello.prg"), "hello");
        assert_eq!(entry_name("/tmp/a/b/thing.PRG"), "thing");
    }

    #[test]
    fn an_unknown_command_is_an_error() {
        assert!(run(&["frobnicate".to_string()]).is_err());
    }
}
