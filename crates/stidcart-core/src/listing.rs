//! What a cartridge listing shows, as values every renderer shares (D-032).
//!
//! `stidcart list` and the GUI viewer must agree on every value, so the values are computed here
//! once: each one already in the form the listing displays it. Renderers own only layout — column
//! widths, pluralisation, styling.

use crate::directory::{DirEntry, EntryType};
use crate::image::Half;
use crate::read::{self, ReadError};

/// A described cartridge: its names, its directory and the space left for one more program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Listing {
    /// The cartridge name the menu shows.
    pub cart: String,
    /// The container header name — the producing tool — with control characters replaced.
    pub built_by: String,
    /// The C64-side menu the cart boots into: `EasyLoader`, `stidcart`, or `unrecognised` when the
    /// loader slot holds neither (D-038).
    pub menu: String,
    /// One row per directory entry, in slot order.
    pub rows: Vec<Row>,
    /// The bytes free: what a re-placement of the entries plus one more program accepts (D-027).
    pub free: usize,
}

/// One directory entry, each field as the listing displays it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub index: usize,
    pub name: String,
    /// `H` hidden, `A` 64 KB aligned, `S` subdirectory bit, in that order; `-` when none is set.
    pub flags: String,
    pub kind: &'static str,
    pub bank: String,
    pub layout: String,
    /// The size as the listing reads it ([`readable_size`]).
    pub size: String,
    /// The exact size in bytes, for a renderer that offers it (`list --bytes`, the window's hover).
    pub size_bytes: usize,
}

impl Listing {
    /// The free figure as the listing displays it ([`readable_size`]).
    pub fn free_text(&self) -> String {
        readable_size(self.free)
    }
}

/// A byte count as the listing reads it (D-044): bytes under 1 KB ("652 bytes", "1 byte"), then
/// 1024-based KB with one decimal rounded half up ("1.1 KB", "10.0 KB"), whole for an exact
/// multiple ("8 KB"), and MB the same way once it rounds to 1024 KB or more.
pub fn readable_size(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = 1024 * KB;
    if bytes < KB {
        return format!("{bytes} {}", if bytes == 1 { "byte" } else { "bytes" });
    }
    let unit = |size: usize, name: &str| {
        if bytes.is_multiple_of(size) {
            return format!("{} {name}", bytes / size);
        }
        let tenths = (bytes * 10 + size / 2) / size;
        format!("{}.{} {name}", tenths / 10, tenths % 10)
    };
    let kb_tenths = (bytes * 10 + KB / 2) / KB;
    if bytes >= MB || kb_tenths >= 10240 {
        unit(MB, "MB")
    } else {
        unit(KB, "KB")
    }
}

/// Describe a `.crt` image the way the listing shows it.
///
/// Refuses exactly what [`read::inspect`] refuses. The free figure is the one place the listing
/// asks a rebuild question — what an added program may take — so it comes from the rebuild view
/// when there is one (D-027). An image `load` refuses cannot be edited at all; it still gets a
/// figure, describing the space the image as stored leaves at the top.
pub fn describe(bytes: &[u8]) -> Result<Listing, ReadError> {
    let info = read::inspect(bytes)?;
    let free = read::load(bytes).map_or_else(|_| info.free_space(), |b| b.free_space());
    let rows = info
        .entries
        .iter()
        .enumerate()
        .map(|(index, e)| Row {
            index,
            name: e.name.clone(),
            flags: flags(e),
            kind: kind_label(e.kind),
            bank: format!("${:02x}", e.bank),
            layout: layout(e),
            size: readable_size(e.size as usize),
            size_bytes: e.size as usize,
        })
        .collect();
    // The cartridge name and every entry name arrive through the machine's own character mapping,
    // which already turns control bytes into `.`. The container header name does not — it is
    // host-encoded text carried through verbatim — so it is filtered here, before any renderer
    // puts it in front of a person.
    Ok(Listing {
        cart: info.ef_name.clone(),
        built_by: printable(&info.header_name),
        menu: info.menu.map_or("unrecognised", |m| m.name()).to_string(),
        rows,
        free,
    })
}

fn flags(e: &DirEntry) -> String {
    let mut flags = String::new();
    if e.hidden {
        flags.push('H');
    }
    if e.align_64k {
        flags.push('A');
    }
    if e.subdir {
        flags.push('S');
    }
    if flags.is_empty() {
        flags.push('-');
    }
    flags
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

/// Replace anything that would drive a terminal rather than print on it.
///
/// A `.crt` is untrusted input, and its 32-byte header name is free-form host-encoded text. Left
/// alone, a name holding `\x1b[2J` clears the screen the listing was just written to, so what the
/// reader believes they inspected is not what the file holds.
fn printable(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_control() { '.' } else { c })
        .collect()
}
