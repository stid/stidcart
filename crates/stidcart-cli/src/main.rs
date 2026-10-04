//! `stidcart` — the command-line harness over `stidcart-core`.
//!
//! This is a test harness, not a product. It exists so the core can be driven end to end and
//! diffed against a reference implementation; the interface mirrors that tool's `build` closely
//! enough to compare outputs.

use std::path::Path;
use std::process::ExitCode;

use stidcart_core::directory::{DirEntry, EntryType};
use stidcart_core::image::Half;
use stidcart_core::read;
use stidcart_core::{crt, CartBuilder, Cartridge, Entry, Program};

const USAGE: &str = "\
usage: stidcart build [-e NAME] [--hide INPUT]... [--align INPUT]... -o OUTPUT INPUT...
       stidcart list CART.crt
       stidcart edit CART.crt [-o OUTPUT] [OPERATION]...

  build           create a cartridge image from programs and cartridge images
  -e NAME         cartridge name shown by the menu (default: \"(no EF-Name)\")
  -o OUTPUT       where to write the image
  --hide INPUT    mark that input hidden in the built cartridge
  --align INPUT   request 64 KB alignment for that xbank cartridge input
  INPUT           a .prg, or a cartridge image (recognised by content: generic 8k/16k/Ultimax,
                  Ocean, EasyFlash xbank); the entry name is the file stem

  list            show what a cartridge image contains
  CART.crt        the image to read

  edit            change an existing image, writing over it unless -o is given
  -o OUTPUT       write the result here and leave CART.crt untouched
  --add FILE      append a program or a cartridge image, named after its file stem
  --add-aligned FILE  append an xbank cartridge image with 64 KB alignment requested
  --remove N      drop the entry at index N
  --rename N NAME rename the entry at index N
  --replace N FILE swap the data at index N, keeping its name and place; the kind follows the file
  --hide N        hide the entry at index N
  --show N        stop hiding the entry at index N
  --align N       request 64 KB alignment for the xbank cartridge at index N
  --unalign N     clear that request
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
    let mut align: Vec<String> = Vec::new();

    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-e" => ef_name = Some(it.next().ok_or("-e needs a name")?.clone()),
            "-o" => output = Some(it.next().ok_or("-o needs a path")?.clone()),
            "--hide" => hide.push(it.next().ok_or("--hide needs an input path")?.clone()),
            "--align" => align.push(it.next().ok_or("--align needs an input path")?.clone()),
            "-h" | "--help" => return Err(USAGE.to_string()),
            other if other.starts_with('-') => return Err(format!("unknown option '{other}'")),
            other => inputs.push(other.to_string()),
        }
    }

    if let Some(unmatched) = hide.iter().find(|h| !inputs.contains(h)) {
        return Err(format!(
            "--hide names '{unmatched}', which is not one of the inputs being built"
        ));
    }
    if let Some(unmatched) = align.iter().find(|a| !inputs.contains(a)) {
        return Err(format!(
            "--align names '{unmatched}', which is not one of the inputs being built"
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
    for (i, path) in inputs.iter().enumerate() {
        builder = builder.with_entry(input_entry(path, align.contains(path))?);
        builder.set_hidden(i, hide.contains(path));
    }

    let bytes = builder.build().map_err(|e| e.to_string())?;
    std::fs::write(&output, &bytes).map_err(|e| format!("writing '{output}': {e}"))?;

    Ok(Some(output))
}

/// One requested change to an existing cartridge. Every index is the one `list` prints for the
/// image as it was given — see [`edit`] for why they are all resolved before any of them apply.
enum Op {
    /// A file to append, and whether 64 KB alignment was requested for it.
    Add(String, bool),
    Remove(usize),
    Rename(usize, String),
    Replace(usize, String),
    Hidden(usize, bool),
    Align(usize, bool),
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
            "--add" => ops.push(Op::Add(
                it.next().ok_or("--add needs a file")?.clone(),
                false,
            )),
            "--add-aligned" => ops.push(Op::Add(
                it.next().ok_or("--add-aligned needs a file")?.clone(),
                true,
            )),
            "--align" => ops.push(Op::Align(index(it.next(), "--align")?, true)),
            "--unalign" => ops.push(Op::Align(index(it.next(), "--unalign")?, false)),
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
                    it.next().ok_or("--replace needs a file")?.clone(),
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
    let held = builder.entries().len();
    for op in &ops {
        let at = match op {
            Op::Remove(at)
            | Op::Rename(at, _)
            | Op::Replace(at, _)
            | Op::Hidden(at, _)
            | Op::Align(at, _) => *at,
            Op::Add(..) => continue,
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
            Op::Replace(at, path) => builder.replace(*at, input_entry(path, false)?),
            Op::Align(at, on) => {
                let entry = &builder.entries()[*at];
                if !entry.as_cartridge().is_some_and(|c| c.kind.is_xbank()) {
                    return Err(format!(
                        "entry {at} '{}' is not an xbank cartridge, so 64 KB alignment does not \
                         apply to it",
                        entry.name()
                    ));
                }
                builder.set_align_64k(*at, *on);
            }
            Op::Add(..) | Op::Remove(_) => {}
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
        if let Op::Add(path, aligned) = op {
            builder.push(input_entry(path, *aligned)?);
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

/// Read an input file as the entry it is: a cartridge image if it starts with the CRT signature,
/// a program otherwise. Content decides, not the file name — a program starts with a two-byte
/// load address and an image with a fixed sixteen-byte signature, so the two cannot collide.
fn input_entry(path: &str, aligned: bool) -> Result<Entry, String> {
    let data = std::fs::read(path).map_err(|e| format!("reading '{path}': {e}"))?;
    let name = entry_name(path);
    if data.starts_with(crt::MAGIC) {
        let cart = Cartridge::from_crt(&data)
            .map_err(|e| format!("'{path}': {e}"))?
            .with_name(&name);
        if aligned && !cart.kind.is_xbank() {
            return Err(format!(
                "'{name}' is not an xbank cartridge, so 64 KB alignment does not apply to it"
            ));
        }
        Ok(cart.with_align_64k(aligned).into())
    } else if aligned {
        Err(format!(
            "'{name}' is a program, not an xbank cartridge, so 64 KB alignment does not apply to it"
        ))
    } else {
        Ok(Program::new(&name, data).into())
    }
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
    // The free figure is the one place the listing asks a rebuild question — what `edit --add`
    // would accept — so it comes from the rebuild view when there is one (D-027). An image `load`
    // refuses cannot be edited at all; it still gets a figure, describing the space the image as
    // stored leaves at the top.
    let free = read::load(&bytes).map_or_else(|_| info.free_space(), |b| b.free_space());

    print!("{}", render(&info, free));
    Ok(())
}

/// The listing itself, kept separate from the printing so it can be asserted on directly.
fn render(info: &read::CartInfo, free: usize) -> String {
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
            "  {:>3}  {:<16}  {:<5}  {:<6}  {:>4}  {:<12}  {:>8}\n",
            "#", "name", "flags", "type", "bank", "layout", "size"
        ));
        for (i, e) in info.entries.iter().enumerate() {
            let mut flags = String::new();
            if e.hidden {
                flags.push('H');
            }
            if e.align_64k {
                flags.push('A');
            }
            if flags.is_empty() {
                flags.push('-');
            }
            out.push_str(&format!(
                "  {i:>3}  {:<16}  {flags:<5}  {:<6}  {:>4}  {:<12}  {:>8}\n",
                e.name,
                kind_label(e.kind),
                format!("${:02x}", e.bank),
                layout(e),
                format!("${:06x}", e.size),
            ));
        }
    }

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

/// One short label per kind the directory can hold.
fn kind_label(kind: EntryType) -> &'static str {
    match kind {
        EntryType::Prg => "PRG",
        EntryType::PrgRomL => "PRG-L",
        EntryType::PrgRomH => "PRG-H",
        EntryType::K8 => "8K",
        EntryType::K16 => "16K",
        EntryType::Ultimax8 => "ULT8",
        EntryType::Ultimax16 => "ULT16",
        EntryType::Ocean => "OCEAN",
        EntryType::Ocean512 => "OCN512",
        EntryType::Xbank8 => "XB8",
        EntryType::Xbank16 => "XB16",
        EntryType::XbankUltimax => "XBULT",
        // The walk never yields an end marker as an entry.
        EntryType::End => "END",
    }
}

/// Where an entry's bytes sit: a program's offset word, or a cartridge's bank runs per half as
/// `L<banks>@<offset>,H<banks>@<offset>` (offsets in banks from the start bank; an empty run is
/// omitted).
fn layout(e: &DirEntry) -> String {
    if !e.kind.is_cartridge() {
        return format!("${:04x}", e.offset);
    }
    let (primary, other) = (
        (0u8, e.primary_num()),
        (e.other_off, u32::from(e.other_num)),
    );
    let ((l_at, l), (h_at, h)) = match e.kind.primary() {
        Half::RomL => (primary, other),
        Half::RomH => (other, primary),
    };
    let mut runs = Vec::new();
    if l > 0 {
        runs.push(format!("L{l}@{l_at}"));
    }
    if h > 0 {
        runs.push(format!("H{h}@{h_at}"));
    }
    runs.join(",")
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
