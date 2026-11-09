//! `stidcart` — the command-line harness over `stidcart-core`.
//!
//! The binary in `main.rs` only maps [`run`] to an exit code. The commands live here so the GUI's
//! tests can hold the window's saves to what `edit` and `build` write, by calling the real thing
//! (D-035).
//!
//! This is a test harness, not a product. It exists so the core can be driven end to end and
//! diffed against a reference implementation; the interface mirrors that tool's `build` closely
//! enough to compare outputs.

use std::path::Path;

use stidcart_core::listing::{self, Listing};
use stidcart_core::menu::Menu;
use stidcart_core::meta::{self, Field};
use stidcart_core::read;
use stidcart_core::{CartBuilder, Entry, InputError};

const USAGE: &str = "\
usage: stidcart build [-e NAME] [--menu MENU] [--hide INPUT]... [--align INPUT]... -o OUTPUT INPUT...
       stidcart list [--bytes] CART.crt
       stidcart edit CART.crt [-o OUTPUT] [OPERATION]...

  build           create a cartridge image from programs and cartridge images
  -e NAME         cartridge name shown by the menu (default: \"(no EF-Name)\")
  -o OUTPUT       where to write the image
  --menu MENU     the menu the cart boots into: stidcart (default) or easyloader
  --hide INPUT    mark that input hidden in the built cartridge
  --align INPUT   request 64 KB alignment for that xbank cartridge input
  INPUT           a .prg, or a cartridge image (recognised by content: generic 8k/16k/Ultimax,
                  Ocean, EasyFlash xbank); the entry name is the file stem

  list            show what a cartridge image contains, sizes in bytes, KB or MB
  --bytes         give every size and the free space as exact byte counts
  CART.crt        the image to read

  edit            change an existing image, writing over it unless -o is given
  -o OUTPUT       write the result here and leave CART.crt untouched
  --add FILE      append a program or a cartridge image, named after its file stem
  --add-aligned FILE  append an xbank cartridge image with 64 KB alignment requested
  --remove N      drop the entry at index N
  --rename N NAME rename the entry at index N
  --move N TO     move the entry at index N so it sits at index TO; once per call,
                  and not with --add, --add-aligned or --remove
  --replace N FILE swap the data at index N, keeping its name and place; the kind follows the file
  --hide N        hide the entry at index N
  --show N        stop hiding the entry at index N
  --align N       request 64 KB alignment for the xbank cartridge at index N
  --unalign N     clear that request
  -e NAME         set the cartridge name shown by the menu
  --header NAME   set the container header name
  --menu MENU     switch the menu to easyloader or stidcart, replacing whatever the loader slot
                  holds; without it the cart keeps its menu
  --title N TEXT  set the title of the entry at index N (at most 48 characters)
  --year N TEXT   set its year (at most 4)
  --author N TEXT set its author (at most 24)
  --note N TEXT   set its note (at most 48)
  --folder N TEXT put it in a folder (at most 16; --group N TEXT also works); an
                  empty TEXT clears any of these
  --new-folder NAME   add an empty folder after the cart's folders
  --remove-folder NAME  remove a folder; its entries stay, in no folder
  --move-folder NAME TO  put a folder at position TO of the cart's folder order
  --import FILE   set metadata from a TOML file of [[entry]] tables, each naming an
                  entry by its stored name (case ignored) with any of title, year,
                  author, note and folder; any problem refuses the whole file

  Indices are the ones `list` prints, and every index in one invocation refers
  to the image as it was given, not to the state a previous operation left.
  Metadata text is printable ASCII, so it maps to PETSCII as names do.
";

/// Returns the path written, or `None` when there was nothing to write.
pub fn run(args: &[String]) -> Result<Option<String>, String> {
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
    let mut menu: Option<Menu> = None;
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
            "--menu" => menu = Some(menu_arg(it.next())?),
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
    // Our menu unless told otherwise (D-064); the core's own default stays EasyLoader, which its
    // ndeft parity proofs build with.
    builder = builder.with_menu(menu.unwrap_or(Menu::Stidcart));
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
    /// The entry at the first index, moved to sit at the second in the result (D-060).
    Move(usize, usize),
    Replace(usize, String),
    Hidden(usize, bool),
    Align(usize, bool),
    /// One metadata field on one entry; an empty value clears it (D-054).
    Meta(usize, Field, String),
    /// A TOML file of metadata, resolved against the cart as given into [`Op::Meta`]s.
    Import(String),
    /// The cart's folder list, by name (D-063).
    NewFolder(String),
    RemoveFolder(String),
    MoveFolder(String, usize),
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
    let mut menu: Option<Menu> = None;
    let mut ops: Vec<Op> = Vec::new();

    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-o" => output = Some(it.next().ok_or("-o needs a path")?.clone()),
            "-e" => ef_name = Some(it.next().ok_or("-e needs a name")?.clone()),
            "--header" => header_name = Some(it.next().ok_or("--header needs a name")?.clone()),
            "--menu" => menu = Some(menu_arg(it.next())?),
            "--add" => ops.push(Op::Add(
                it.next().ok_or("--add needs a file")?.clone(),
                false,
            )),
            "--add-aligned" => ops.push(Op::Add(
                it.next().ok_or("--add-aligned needs a file")?.clone(),
                true,
            )),
            "--title" | "--year" | "--author" | "--note" | "--folder" | "--group" => {
                let field = Field::from_name(&arg[2..]).expect("the five field options");
                let at = index(it.next(), arg)?;
                let value = it.next().ok_or_else(|| format!("{arg} needs a value"))?;
                ops.push(Op::Meta(at, field, value.clone()));
            }
            "--new-folder" => ops.push(Op::NewFolder(
                it.next().ok_or("--new-folder needs a name")?.clone(),
            )),
            "--remove-folder" => ops.push(Op::RemoveFolder(
                it.next().ok_or("--remove-folder needs a name")?.clone(),
            )),
            "--move-folder" => {
                let name = it.next().ok_or("--move-folder needs a name")?.clone();
                ops.push(Op::MoveFolder(name, index(it.next(), "--move-folder")?));
            }
            "--import" => ops.push(Op::Import(
                it.next().ok_or("--import needs a file")?.clone(),
            )),
            "--align" => ops.push(Op::Align(index(it.next(), "--align")?, true)),
            "--unalign" => ops.push(Op::Align(index(it.next(), "--unalign")?, false)),
            "--remove" => ops.push(Op::Remove(index(it.next(), "--remove")?)),
            "--move" => {
                let from = index(it.next(), "--move")?;
                ops.push(Op::Move(from, index(it.next(), "--move")?));
            }
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

    // A move's TO is an index in the result, so it is defined only while nothing else in the call
    // shifts the entries: one move, and no addition or removal beside it (D-060).
    if ops.iter().any(|op| matches!(op, Op::Move(..))) {
        if ops.iter().filter(|op| matches!(op, Op::Move(..))).count() > 1 {
            return Err("--move can be given once per call".into());
        }
        let shifting = ops.iter().find_map(|op| match op {
            Op::Add(_, false) => Some("--add"),
            Op::Add(_, true) => Some("--add-aligned"),
            Op::Remove(_) => Some("--remove"),
            _ => None,
        });
        if let Some(other) = shifting {
            return Err(format!("--move can't be combined with {other}"));
        }
    }

    let bytes = std::fs::read(&cart).map_err(|e| format!("reading '{cart}': {e}"))?;
    // An explicit menu overwrites the loader slot, so a slot holding no known menu is no reason to
    // refuse (D-050).
    let loaded = match menu {
        Some(menu) => read::load_with_menu(&bytes, menu),
        None => read::load(&bytes),
    };
    let mut builder = loaded.map_err(|e| e.to_string())?;

    // An import names entries by their stored name, so it is resolved against the cart as given,
    // in its place among the other operations, before anything applies.
    let ops: Vec<Op> = ops
        .into_iter()
        .map(|op| match op {
            Op::Import(path) => import(&path, &builder),
            op => Ok(vec![op]),
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect();

    // Every index is checked against the image as read, before anything is applied — so the two
    // halves of a request cannot disagree about what entry 1 is, and so a request that names an
    // entry the cart does not have fails without having half-changed it.
    let held = builder.entries().len();
    for op in &ops {
        let at = match op {
            Op::Move(from, to) => {
                if *to >= held {
                    return Err(format!(
                        "--move can't place an entry at index {to}; the cartridge holds {held} {}",
                        if held == 1 { "entry" } else { "entries" }
                    ));
                }
                *from
            }
            Op::Remove(at)
            | Op::Rename(at, _)
            | Op::Replace(at, _)
            | Op::Hidden(at, _)
            | Op::Align(at, _)
            | Op::Meta(at, ..) => *at,
            Op::Add(..)
            | Op::Import(_)
            | Op::NewFolder(_)
            | Op::RemoveFolder(_)
            | Op::MoveFolder(..) => continue,
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
            Op::Meta(at, field, value) => builder
                .set_meta(*at, *field, value)
                .map_err(|e| e.to_string())?,
            Op::NewFolder(name) => builder.add_folder(name).map_err(|e| e.to_string())?,
            Op::RemoveFolder(name) => builder.remove_folder(name).map_err(|e| e.to_string())?,
            Op::MoveFolder(name, to) => {
                builder.move_folder(name, *to).map_err(|e| e.to_string())?
            }
            Op::Add(..) | Op::Remove(_) | Op::Move(..) | Op::Import(_) => {}
        }
    }

    // A move after the in-place changes, which name entries as given; never beside a removal or
    // an addition, so its TO is an index in the result.
    for op in &ops {
        if let Op::Move(from, to) = op {
            builder.move_entry(*from, *to);
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
    if let Some(menu) = menu {
        builder.set_menu(menu);
    }

    let out_bytes = builder.build().map_err(|e| e.to_string())?;
    let destination = output.unwrap_or(cart);
    write_replacing(&destination, &out_bytes)?;

    Ok(Some(destination))
}

/// Read a metadata import file into the [`Op::Meta`]s it asks for, against the cart as given.
///
/// The file is `[[entry]]` tables, each with a `name` matched against the entries' stored names
/// ignoring case, and any of the five fields as text (a year may be a bare number). Anything wrong
/// refuses the whole file, naming the problem, before any of it applies: an unknown key, a value
/// that isn't text or doesn't fit its field, a name matching no entry or several, or one entry
/// named twice.
fn import(path: &str, builder: &CartBuilder) -> Result<Vec<Op>, String> {
    use toml::Value;

    let text = std::fs::read_to_string(path).map_err(|e| format!("reading '{path}': {e}"))?;
    let at = |what: String| format!("'{path}': {what}");
    let table: toml::Table = text.parse().map_err(|e| at(format!("{e}")))?;
    if let Some(key) = table.keys().find(|k| *k != "entry") {
        return Err(at(format!(
            "unknown key '{key}'; the file holds [[entry]] tables"
        )));
    }
    let tables = match table.get("entry") {
        None => return Ok(Vec::new()),
        Some(Value::Array(tables)) => tables,
        Some(_) => return Err(at("'entry' must be [[entry]] tables".into())),
    };

    let mut ops = Vec::new();
    let mut named: Vec<usize> = Vec::new();
    for (n, t) in tables.iter().enumerate() {
        let Value::Table(t) = t else {
            return Err(at(format!("entry {n} is not a table")));
        };
        let Some(Value::String(name)) = t.get("name") else {
            return Err(at(format!("entry {n} has no name")));
        };
        let matching: Vec<usize> = builder
            .entries()
            .iter()
            .enumerate()
            .filter(|(_, e)| e.name().eq_ignore_ascii_case(name))
            .map(|(i, _)| i)
            .collect();
        let index = match matching[..] {
            [i] => i,
            [] => return Err(at(format!("no entry is named '{name}'"))),
            _ => {
                return Err(at(format!(
                    "{} entries are named '{name}', so it can't say which",
                    matching.len()
                )))
            }
        };
        if named.contains(&index) {
            return Err(at(format!("'{name}' is named twice")));
        }
        named.push(index);

        for (key, value) in t.iter().filter(|(k, _)| *k != "name") {
            let field = Field::from_name(key).ok_or_else(|| {
                at(format!(
                    "entry '{name}' has an unknown key '{key}'; the fields are title, year, \
                     author, note and folder"
                ))
            })?;
            // `group` is the folder's old name (D-060), so a table may not set both.
            if ops
                .iter()
                .any(|op| matches!(op, Op::Meta(i, f, _) if *i == index && *f == field))
            {
                return Err(at(format!(
                    "entry '{name}' sets the {} twice",
                    field.name()
                )));
            }
            let value = match value {
                Value::String(s) => s.clone(),
                Value::Integer(i) if field == Field::Year => i.to_string(),
                _ => return Err(at(format!("entry '{name}': the {key} must be text"))),
            };
            meta::check(field, &value).map_err(|e| at(format!("entry '{name}': {e}")))?;
            ops.push(Op::Meta(index, field, value));
        }
    }
    Ok(ops)
}

/// Read an input file as the entry it is, named after its file stem. What the file *is* —
/// program or cartridge image — is decided in the core, so every front end decides it the same
/// way (D-034); this adds only the reading and the path in the error.
fn input_entry(path: &str, aligned: bool) -> Result<Entry, String> {
    let data = std::fs::read(path).map_err(|e| format!("reading '{path}': {e}"))?;
    Entry::from_input(&entry_name(path), data, aligned).map_err(|e| match e {
        InputError::Cartridge(e) => format!("'{path}': {e}"),
        other => other.to_string(),
    })
}

/// A `--menu` value.
fn menu_arg(arg: Option<&String>) -> Result<Menu, String> {
    let raw = arg.ok_or("--menu needs easyloader or stidcart")?;
    Menu::from_name(raw).ok_or_else(|| format!("--menu needs easyloader or stidcart, not '{raw}'"))
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
    let mut exact = false;
    for arg in args {
        match arg.as_str() {
            "-h" | "--help" => return Err(USAGE.to_string()),
            "--bytes" => exact = true,
            other if other.starts_with('-') => return Err(format!("unknown option '{other}'")),
            other if path.is_none() => path = Some(other),
            other => return Err(format!("list takes one image; also given '{other}'")),
        }
    }
    let path = path.ok_or_else(|| format!("no cartridge image given\n{USAGE}"))?;

    let bytes = std::fs::read(path).map_err(|e| format!("reading '{path}': {e}"))?;
    let listing = listing::describe(&bytes).map_err(|e| e.to_string())?;
    print!("{}", render(&listing, exact));
    Ok(())
}

/// The listing's layout. The values come from the core, shared with the GUI (D-032); this owns
/// only the columns and the wording around them, kept apart from the printing so tests can assert
/// on the string. `exact` prints every size and the free figure as exact byte counts (D-044).
fn render(l: &Listing, exact: bool) -> String {
    let mut out = String::new();
    out.push_str(&format!("cart: {}\n", l.cart));
    out.push_str(&format!("built by: {}\n", l.built_by));
    out.push_str(&format!("menu: {}\n", l.menu));
    // The cart's folders in order with their entry counts, empty ones included (D-063); a cart
    // without folders prints no line, so the listing reads as before.
    if !l.folders.is_empty() {
        let folders: Vec<String> = l
            .folders
            .iter()
            .map(|f| format!("{f} ({})", l.rows.iter().filter(|r| &r.folder == f).count()))
            .collect();
        out.push_str(&format!("folders: {}\n", folders.join("  ")));
    }
    out.push('\n');

    if l.rows.is_empty() {
        out.push_str("  (no entries)\n");
    } else {
        out.push_str(&format!(
            "  {:>3}  {:<16}  {:<5}  {:<6}  {:>4}  {:<12}  {:>10}\n",
            "#", "name", "flags", "type", "bank", "layout", "size"
        ));
        for r in &l.rows {
            let size = if exact {
                r.size_bytes.to_string()
            } else {
                r.size.clone()
            };
            out.push_str(&format!(
                "  {:>3}  {:<16}  {:<5}  {:<6}  {:>4}  {:<12}  {:>10}\n",
                r.index, r.name, r.flags, r.kind, r.bank, r.layout, size,
            ));
            // The entry's metadata, indented under its name, only when it has some (D-054).
            let fields: Vec<String> = [
                ("title", &r.title),
                ("year", &r.year),
                ("author", &r.author),
                ("note", &r.note),
                ("folder", &r.folder),
            ]
            .iter()
            .filter(|(_, v)| !v.is_empty())
            .map(|(k, v)| format!("{k}: {v}"))
            .collect();
            if !fields.is_empty() {
                out.push_str(&format!("       {}\n", fields.join("  ")));
            }
        }
    }

    let free = if exact {
        format!("{} bytes", l.free)
    } else {
        l.free_text()
    };
    out.push_str(&format!(
        "\n{} {}, {free} free\n",
        l.rows.len(),
        if l.rows.len() == 1 {
            "entry"
        } else {
            "entries"
        },
    ));
    if l.meta_dropped > 0 {
        let (records, them) = if l.meta_dropped == 1 {
            ("record", "it")
        } else {
            ("records", "them")
        };
        out.push_str(&format!(
            "metadata: {} {records} dropped: no entry matches {them} any more\n",
            l.meta_dropped
        ));
    }
    if let Some(kept) = &l.meta_kept {
        out.push_str(&format!("metadata: {kept}\n"));
    }
    out
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
