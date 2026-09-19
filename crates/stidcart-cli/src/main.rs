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
usage: stidcart build [-e NAME] [--hide INPUT.prg]... -o OUTPUT INPUT.prg...
       stidcart list CART.crt
       stidcart edit CART.crt [-o OUTPUT] [OPERATION]...

  build           create a cartridge image from program files
  -e NAME         cartridge name shown by the menu (default: \"(no EF-Name)\")
  -o OUTPUT       where to write the image
  --hide INPUT    mark that input hidden in the built cartridge
  INPUT.prg       programs to include; the entry name is the file stem

  list            show what a cartridge image contains
  CART.crt        the image to read

  edit            change an existing image, writing over it unless -o is given
  -o OUTPUT       write the result here and leave CART.crt untouched
  --add PRG       append a program, named after its file stem
  --remove N      drop the entry at index N
  --rename N NAME rename the entry at index N
  --replace N PRG swap the program data at index N, keeping its name and place
  --hide N        hide the entry at index N
  --show N        stop hiding the entry at index N
  -e NAME         set the cartridge name shown by the menu
  --header NAME   set the container header name

  Indices are the ones `list` prints, and every index in one invocation refers
  to the image as it was given, not to the state a previous operation left.
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
        "edit" => edit(&args[1..]),
        other => Err(format!("unknown command '{other}'\n{USAGE}")),
    }
}

fn build(args: &[String]) -> Result<Option<String>, String> {
    let mut ef_name: Option<String> = None;
    let mut output: Option<String> = None;
    let mut inputs: Vec<String> = Vec::new();
    // Marked by input path: at build time there are no entries yet, so there is nothing to index.
    let mut hide: Vec<String> = Vec::new();

    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-e" => ef_name = Some(it.next().ok_or("-e needs a name")?.clone()),
            "-o" => output = Some(it.next().ok_or("-o needs a path")?.clone()),
            "--hide" => hide.push(it.next().ok_or("--hide needs an input path")?.clone()),
            "-h" | "--help" => return Err(USAGE.to_string()),
            other if other.starts_with('-') => return Err(format!("unknown option '{other}'")),
            other => inputs.push(other.to_string()),
        }
    }

    if let Some(unmatched) = hide.iter().find(|h| !inputs.contains(h)) {
        return Err(format!(
            "--hide names '{unmatched}', which is not one of the programs being built"
        ));
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
        builder = builder
            .with_program(Program::new(&entry_name(path), data).with_hidden(hide.contains(path)));
    }

    let bytes = builder.build().map_err(|e| e.to_string())?;
    std::fs::write(&output, &bytes).map_err(|e| format!("writing '{output}': {e}"))?;

    Ok(Some(output))
}

/// One requested change to an existing cartridge. Every index is the one `list` prints for the
/// image as it was given — see [`edit`] for why they are all resolved before any of them apply.
enum Op {
    Add(String),
    Remove(usize),
    Rename(usize, String),
    Replace(usize, String),
    Hidden(usize, bool),
}

/// Change an existing image and write it back.
///
/// Reading goes through [`read::load`] rather than [`read::inspect`], so the preconditions of
/// rebuilding apply: an image this tool could not reproduce unchanged — one carrying firmware it
/// does not embed, an entry kind it cannot represent, an old-format directory — is refused here
/// rather than saved as something quietly different from what arrived.
///
/// Saving re-places every entry, because there is one placement path and it is the one `build`
/// uses. An edited image is therefore not a patch of its input: entries the request never mentioned
/// can land at different offsets.
fn edit(args: &[String]) -> Result<Option<String>, String> {
    let mut cart: Option<String> = None;
    let mut output: Option<String> = None;
    let mut ef_name: Option<String> = None;
    let mut header_name: Option<String> = None;
    let mut ops: Vec<Op> = Vec::new();

    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-o" => output = Some(it.next().ok_or("-o needs a path")?.clone()),
            "-e" => ef_name = Some(it.next().ok_or("-e needs a name")?.clone()),
            "--header" => header_name = Some(it.next().ok_or("--header needs a name")?.clone()),
            "--add" => ops.push(Op::Add(it.next().ok_or("--add needs a program")?.clone())),
            "--remove" => ops.push(Op::Remove(index(it.next(), "--remove")?)),
            "--hide" => ops.push(Op::Hidden(index(it.next(), "--hide")?, true)),
            "--show" => ops.push(Op::Hidden(index(it.next(), "--show")?, false)),
            "--rename" => {
                let at = index(it.next(), "--rename")?;
                ops.push(Op::Rename(
                    at,
                    it.next().ok_or("--rename needs a new name")?.clone(),
                ));
            }
            "--replace" => {
                let at = index(it.next(), "--replace")?;
                ops.push(Op::Replace(
                    at,
                    it.next().ok_or("--replace needs a program")?.clone(),
                ));
            }
            "-h" | "--help" => return Err(USAGE.to_string()),
            other if other.starts_with('-') => return Err(format!("unknown option '{other}'")),
            other if cart.is_none() => cart = Some(other.to_string()),
            other => return Err(format!("edit takes one image; also given '{other}'")),
        }
    }
    let cart = cart.ok_or_else(|| format!("no cartridge image given\n{USAGE}"))?;

    let bytes = std::fs::read(&cart).map_err(|e| format!("reading '{cart}': {e}"))?;
    let mut builder = read::load(&bytes).map_err(|e| e.to_string())?;

    // Every index is checked against the image as read, before anything is applied — so the two
    // halves of a request cannot disagree about what entry 1 is, and so a request that names an
    // entry the cart does not have fails without having half-changed it.
    let held = builder.programs().len();
    for op in &ops {
        let at = match op {
            Op::Remove(at) | Op::Rename(at, _) | Op::Replace(at, _) | Op::Hidden(at, _) => *at,
            Op::Add(_) => continue,
        };
        if at >= held {
            return Err(format!(
                "no entry at index {at}; the cartridge holds {held} {}",
                if held == 1 { "entry" } else { "entries" }
            ));
        }
    }

    // In-place changes first, in the order given, so a later one on the same entry wins.
    for op in &ops {
        match op {
            Op::Rename(at, name) => builder.set_name(*at, name),
            Op::Hidden(at, hidden) => builder.set_hidden(*at, *hidden),
            Op::Replace(at, path) => {
                let data = std::fs::read(path).map_err(|e| format!("reading '{path}': {e}"))?;
                builder.replace_data(*at, data);
            }
            Op::Add(_) | Op::Remove(_) => {}
        }
    }

    // Then removals, highest index first, so removing one does not renumber the next.
    let mut removals: Vec<usize> = ops
        .iter()
        .filter_map(|op| match op {
            Op::Remove(at) => Some(*at),
            _ => None,
        })
        .collect();
    removals.sort_unstable();
    removals.dedup();
    for at in removals.iter().rev() {
        builder.remove(*at);
    }

    // Additions last: they append, so they cannot disturb an index anything else refers to.
    for op in &ops {
        if let Op::Add(path) = op {
            let data = std::fs::read(path).map_err(|e| format!("reading '{path}': {e}"))?;
            builder.push(Program::new(&entry_name(path), data));
        }
    }

    if let Some(name) = &ef_name {
        builder.set_ef_name(name);
    }
    if let Some(name) = &header_name {
        builder.set_header_name(name);
    }

    let out_bytes = builder.build().map_err(|e| e.to_string())?;
    let destination = output.unwrap_or(cart);
    write_replacing(&destination, &out_bytes)?;

    Ok(Some(destination))
}

/// An entry index from the command line, with the option that wanted it named in the error.
fn index(arg: Option<&String>, option: &str) -> Result<usize, String> {
    let raw = arg.ok_or_else(|| format!("{option} needs an entry index"))?;
    raw.parse()
        .map_err(|_| format!("{option} needs an entry index, not '{raw}'"))
}

/// Write `bytes` to `path` through a temporary file in the same directory, then rename over it.
///
/// `edit` writes over the image it was given unless told otherwise, so a write that fails halfway
/// would truncate the only copy of a cartridge. The rename is what makes the file either the old
/// image or the new one and never a prefix of the new one; the cost is that the destination is
/// replaced rather than rewritten, so its inode and permissions are new.
fn write_replacing(path: &str, bytes: &[u8]) -> Result<(), String> {
    let mut temporary = std::ffi::OsString::from(path);
    temporary.push(".stidcart-tmp");
    let temporary = Path::new(&temporary);

    std::fs::write(temporary, bytes)
        .map_err(|e| format!("writing '{}': {e}", temporary.display()))?;
    std::fs::rename(temporary, path).map_err(|e| {
        let _ = std::fs::remove_file(temporary);
        format!("replacing '{path}': {e}")
    })
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
