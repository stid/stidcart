//! Building EasyFlash cartridge images for the Commodore 64.
//!
//! The whole API is one owned value. Nothing here touches the filesystem: [`CartBuilder::build`]
//! returns bytes and the caller decides what to do with them. That is deliberate — the
//! predecessor's core was a global singleton, which is exactly why nothing could reuse it.
//!
//! ```
//! use stidcart_core::{CartBuilder, Program};
//!
//! let cart = CartBuilder::new()
//!     .with_ef_name("my cart")
//!     .with_program(Program::new("hello", vec![0x01, 0x08, 0x00]))
//!     .build()
//!     .unwrap();
//! assert_eq!(&cart[..16], b"C64 CARTRIDGE   ");
//! ```

pub mod cartridge;
pub mod crt;
pub mod directory;
pub mod firmware;
pub mod image;
pub mod listing;
pub mod menu;
pub mod meta;
pub mod petscii;
pub mod place;
pub mod read;
pub mod reserved;

pub use cartridge::Cartridge;
use cartridge::CartridgeError;
pub use image::FlashImage;
pub use read::{inspect, load, CartInfo, ReadError};

use directory::{DirEntry, EntryType, ENTRY_LEN, MAX_ENTRIES};
use image::{BANK_SIZE, N_BANKS};
use menu::Menu;
use meta::{Field, KeptMeta, Meta, MetaError};
use place::Placer;
use reserved::{Mode, DEFAULT_EF_NAME, DIRECTORY_OFFSET};

/// A program to place on the cartridge. `data` is the raw `.prg`, load address included.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Program {
    pub name: String,
    pub data: Vec<u8>,
    /// Kept out of the menu the cartridge shows. The flag is part of the directory entry, so it
    /// survives a save and a reload — which is the whole reason [`read::load`] no longer refuses a
    /// cart that carries one (D-020).
    pub hidden: bool,
    /// Title, year, author, note and group. Carried as `hidden` is, so remove, rename, replace and
    /// re-order keep it with its entry (D-054).
    pub meta: Meta,
}

impl Program {
    pub fn new(name: &str, data: Vec<u8>) -> Self {
        Program {
            name: name.to_string(),
            data,
            hidden: false,
            meta: Meta::default(),
        }
    }

    pub fn with_hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }
}

/// One directory entry: a program, or a cartridge. One vector holds both, so every index-addressed
/// operation applies to either kind (D-023).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    Program(Program),
    Cartridge(Cartridge),
}

impl Entry {
    pub fn name(&self) -> &str {
        match self {
            Entry::Program(p) => &p.name,
            Entry::Cartridge(c) => &c.name,
        }
    }

    pub fn hidden(&self) -> bool {
        match self {
            Entry::Program(p) => p.hidden,
            Entry::Cartridge(c) => c.hidden,
        }
    }

    pub fn meta(&self) -> &Meta {
        match self {
            Entry::Program(p) => &p.meta,
            Entry::Cartridge(c) => &c.meta,
        }
    }

    pub(crate) fn meta_mut(&mut self) -> &mut Meta {
        match self {
            Entry::Program(p) => &mut p.meta,
            Entry::Cartridge(c) => &mut c.meta,
        }
    }

    pub fn as_program(&self) -> Option<&Program> {
        match self {
            Entry::Program(p) => Some(p),
            Entry::Cartridge(_) => None,
        }
    }

    pub fn as_cartridge(&self) -> Option<&Cartridge> {
        match self {
            Entry::Program(_) => None,
            Entry::Cartridge(c) => Some(c),
        }
    }

    fn set_name(&mut self, name: &str) {
        match self {
            Entry::Program(p) => p.name = name.to_string(),
            Entry::Cartridge(c) => c.name = name.to_string(),
        }
    }

    fn set_hidden(&mut self, hidden: bool) {
        match self {
            Entry::Program(p) => p.hidden = hidden,
            Entry::Cartridge(c) => c.hidden = hidden,
        }
    }
}

impl Entry {
    /// Read an input file's contents as the entry it is, named `name`: a cartridge image if it
    /// starts with the CRT signature, a program otherwise. Content decides, not the file name — a
    /// program starts with a two-byte load address and an image with a fixed sixteen-byte
    /// signature, so the two cannot collide. Every front end classifies through here (D-034).
    pub fn from_input(name: &str, data: Vec<u8>, aligned: bool) -> Result<Entry, InputError> {
        if data.starts_with(crt::MAGIC) {
            let cart = Cartridge::from_crt(&data)
                .map_err(InputError::Cartridge)?
                .with_name(name);
            if aligned && !cart.kind.is_xbank() {
                return Err(InputError::AlignNotXbank {
                    name: name.to_string(),
                });
            }
            Ok(cart.with_align_64k(aligned).into())
        } else if aligned {
            Err(InputError::AlignProgram {
                name: name.to_string(),
            })
        } else {
            Ok(Program::new(name, data).into())
        }
    }
}

/// Why an input file cannot become an entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InputError {
    /// A CRT image that does not derive to a cartridge. Front ends prefix it with the file's path.
    Cartridge(CartridgeError),
    /// 64 KB alignment requested for a program.
    AlignProgram { name: String },
    /// 64 KB alignment requested for a cartridge that is not an xbank.
    AlignNotXbank { name: String },
}

impl std::fmt::Display for InputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputError::Cartridge(e) => write!(f, "{e}"),
            InputError::AlignProgram { name } => write!(
                f,
                "'{name}' is a program, not an xbank cartridge, so 64 KB alignment does not apply \
                 to it"
            ),
            InputError::AlignNotXbank { name } => write!(
                f,
                "'{name}' is not an xbank cartridge, so 64 KB alignment does not apply to it"
            ),
        }
    }
}

impl std::error::Error for InputError {}

impl From<Program> for Entry {
    fn from(p: Program) -> Self {
        Entry::Program(p)
    }
}

impl From<Cartridge> for Entry {
    fn from(c: Cartridge) -> Self {
        Entry::Cartridge(c)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BuildError {
    /// More slots needed than the directory has. `metadata` says one of them is the metadata
    /// entry's (D-054).
    TooManyEntries {
        max: usize,
        got: usize,
        metadata: bool,
    },
    OutOfSpace {
        name: String,
        size: usize,
        free: usize,
    },
    /// Only one entry can own bank 0, which every Ocean cartridge must (D-024).
    SecondOcean { name: String },
    /// 64 KB alignment is an xbank property; the format has nowhere to record it on another kind.
    AlignNotXbank { name: String },
    /// Our menu has no Ocean-mode build (D-037), and this entry is an Ocean cartridge.
    MenuNeedsNormalMode { name: String },
    /// Our menu copies programs from `$DF00`, with I/O mapped in, so it cannot start one that
    /// loads anywhere in `$D000-$DFFF` (D-038).
    ProgramInIoArea { name: String },
    /// The metadata payload would not fit the 16 KB a C64 copies it into at `$4000` (D-055).
    MetadataTooLarge { size: usize, max: usize },
    /// The cart's metadata is kept as stored in a layout this build can't read, and the entries
    /// it describes were renamed, removed or re-ordered, or metadata was set (D-055).
    MetadataKept { reason: String },
}

impl std::fmt::Display for BuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildError::TooManyEntries {
                max,
                got,
                metadata: false,
            } => {
                write!(f, "{got} entries requested but the directory holds {max}")
            }
            BuildError::TooManyEntries {
                max,
                got,
                metadata: true,
            } => write!(
                f,
                "{} entries and the metadata need {got} directory slots, but the directory holds \
                 {max}",
                got - 1
            ),
            BuildError::MetadataTooLarge { size, max } => write!(
                f,
                "the metadata needs {size} bytes, more than the {max} a C64 can copy it into"
            ),
            BuildError::MetadataKept { reason } => write!(
                f,
                "the cart's metadata is kept as stored ({reason}), and this edit would need it \
                 rewritten: it renames, removes or re-orders the entries it describes, or sets \
                 metadata"
            ),
            BuildError::OutOfSpace { name, size, free } => write!(
                f,
                "'{name}' needs {size} bytes but only {free} are free on the cartridge"
            ),
            BuildError::SecondOcean { name } => write!(
                f,
                "'{name}' is a second Ocean cartridge, and only one can own bank 0"
            ),
            BuildError::AlignNotXbank { name } => write!(
                f,
                "'{name}' is not an xbank cartridge, so 64 KB alignment cannot be requested for it"
            ),
            BuildError::MenuNeedsNormalMode { name } => write!(
                f,
                "'{name}' is an Ocean cartridge, and the stidcart menu cannot run on an Ocean cart"
            ),
            BuildError::ProgramInIoArea { name } => write!(
                f,
                "'{name}' loads into $D000-$DFFF, which the stidcart menu cannot start a program over"
            ),
        }
    }
}

impl std::error::Error for BuildError {}

/// Accumulates entries and produces a cartridge image.
#[derive(Clone, Debug)]
pub struct CartBuilder {
    ef_name: String,
    header_name: String,
    menu: Menu,
    entries: Vec<Entry>,
    /// A metadata entry in a layout this build can't read, with the entry names as loaded: written
    /// back byte for byte while those names stay a prefix of the entries (D-055).
    kept: Option<(KeptMeta, Vec<String>)>,
}

impl Default for CartBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CartBuilder {
    pub fn new() -> Self {
        CartBuilder {
            ef_name: DEFAULT_EF_NAME.to_string(),
            header_name: format!("stidcart {}", env!("CARGO_PKG_VERSION")),
            menu: Menu::default(),
            entries: Vec::new(),
            kept: None,
        }
    }

    /// The cartridge name the menu shows, stored inside the flash image.
    ///
    /// This is **not** the container header name — see [`CartBuilder::with_header_name`]. They are
    /// different fields at different offsets and conflating them is the single easiest way to
    /// produce an image that differs from a reference for a reason that has nothing to do with
    /// correctness.
    pub fn with_ef_name(mut self, name: &str) -> Self {
        self.ef_name = name.to_string();
        self
    }

    /// The 32-byte name in the `.crt` container header, which identifies the producing tool.
    /// Defaults to this crate's name and version.
    pub fn with_header_name(mut self, name: &str) -> Self {
        self.header_name = name.to_string();
        self
    }

    /// The C64-side menu the cart boots into. EasyLoader unless asked otherwise (D-037).
    pub fn with_menu(mut self, menu: Menu) -> Self {
        self.menu = menu;
        self
    }

    pub fn with_program(self, program: Program) -> Self {
        self.with_entry(program)
    }

    pub fn with_cartridge(self, cartridge: Cartridge) -> Self {
        self.with_entry(cartridge)
    }

    pub fn with_entry(mut self, entry: impl Into<Entry>) -> Self {
        self.entries.push(entry.into());
        self
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// The program entries only, in directory order. A position in this list is **not** an entry
    /// index once the cart holds cartridges — address entries through [`CartBuilder::entries`].
    pub fn programs(&self) -> Vec<&Program> {
        self.entries.iter().filter_map(Entry::as_program).collect()
    }

    /// Append an entry. The directory keeps input order, so it lands last in the menu.
    pub fn push(&mut self, entry: impl Into<Entry>) {
        self.entries.push(entry.into());
    }

    /// Remove the entry at `index` and hand it back.
    ///
    /// This and the other index-addressed mutators panic on an out-of-range index, exactly as
    /// indexing a [`Vec`] does. A caller editing a cart knows how many entries it has — the
    /// command-line front end bounds-checks against [`CartBuilder::entries`] so it can name the
    /// index in its own error — and a second, unreachable check here would only be a second place
    /// for the message to be wrong.
    pub fn remove(&mut self, index: usize) -> Entry {
        self.entries.remove(index)
    }

    pub fn set_name(&mut self, index: usize, name: &str) {
        self.entries[index].set_name(name);
    }

    pub fn set_hidden(&mut self, index: usize, hidden: bool) {
        self.entries[index].set_hidden(hidden);
    }

    /// Set one metadata field on the entry at `index`, or clear it with an empty value. Refused
    /// for a value [`Meta::set`] refuses, and on a cart whose metadata is kept as stored in a
    /// layout this build can't read (D-055).
    pub fn set_meta(&mut self, index: usize, field: Field, value: &str) -> Result<(), MetaError> {
        if let Some((kept, _)) = &self.kept {
            return Err(MetaError::Kept {
                reason: kept.reason.clone(),
            });
        }
        self.entries[index].meta_mut().set(field, value)
    }

    /// An entry's metadata as read from an image, which the walk has already validated.
    pub(crate) fn entry_meta_mut(&mut self, index: usize) -> &mut Meta {
        self.entries[index].meta_mut()
    }

    /// Carry a metadata entry this build can't read, against the entries as they are now.
    pub(crate) fn keep_meta(&mut self, kept: KeptMeta) {
        let names = self.entries.iter().map(|e| e.name().to_string()).collect();
        self.kept = Some((kept, names));
    }

    /// The metadata entry a build writes last, or `None` when no entry has metadata (D-054). The
    /// layout is `docs/format/`'s "stidcart extension: entry metadata".
    fn meta_entry(&self) -> Result<Option<Entry>, BuildError> {
        let data = if let Some((kept, names)) = &self.kept {
            let same = self.entries.len() >= names.len()
                && self.entries.iter().zip(names).all(|(e, n)| e.name() == n);
            if !same || self.entries.iter().any(|e| !e.meta().is_empty()) {
                return Err(BuildError::MetadataKept {
                    reason: kept.reason.clone(),
                });
            }
            kept.data.clone()
        } else {
            let records: Vec<meta::Record> = self
                .entries
                .iter()
                .enumerate()
                .filter(|(_, e)| !e.meta().is_empty())
                .map(|(index, e)| meta::Record {
                    index,
                    name: directory::name_bytes(e.name()),
                    meta: e.meta(),
                })
                .collect();
            if records.is_empty() {
                return Ok(None);
            }
            let data = meta::encode(&records);
            let size = data.len() - 2;
            if size > meta::MAX_PAYLOAD {
                return Err(BuildError::MetadataTooLarge {
                    size,
                    max: meta::MAX_PAYLOAD,
                });
            }
            data
        };
        Ok(Some(
            Program::new(meta::META_NAME, data).with_hidden(true).into(),
        ))
    }

    /// Request or clear 64 KB alignment on the cartridge at `index`. Recorded on any cartridge;
    /// [`CartBuilder::build`] refuses it on anything but an xbank, and it is meaningless on a
    /// program, where it is ignored.
    pub fn set_align_64k(&mut self, index: usize, align: bool) {
        if let Entry::Cartridge(c) = &mut self.entries[index] {
            c.align_64k = align;
        }
    }

    /// Swap the entry at `index` for another, keeping the name and its place in the order. The
    /// kind follows the new data: a program can become a cartridge and back.
    pub fn replace(&mut self, index: usize, entry: impl Into<Entry>) {
        let mut entry = entry.into();
        entry.set_name(self.entries[index].name());
        entry.set_hidden(self.entries[index].hidden());
        *entry.meta_mut() = self.entries[index].meta().clone();
        self.entries[index] = entry;
    }

    /// Swap the program data at `index`, keeping the entry's name and its place in the order.
    pub fn replace_data(&mut self, index: usize, data: Vec<u8>) {
        let name = self.entries[index].name().to_string();
        self.replace(index, Program::new(&name, data));
    }

    /// The menu name — see [`CartBuilder::with_ef_name`] for why it is not the header name.
    pub fn set_ef_name(&mut self, name: &str) {
        self.ef_name = name.to_string();
    }

    pub fn set_header_name(&mut self, name: &str) {
        self.header_name = name.to_string();
    }

    /// The cartridge name the menu shows.
    pub fn set_menu(&mut self, menu: Menu) {
        self.menu = menu;
    }

    pub fn menu(&self) -> Menu {
        self.menu
    }

    pub fn ef_name(&self) -> &str {
        &self.ef_name
    }

    /// The name in the container header, identifying the tool that produced the image. A cart
    /// loaded from a file keeps the name it arrived with — see [`read::load`].
    pub fn header_name(&self) -> &str {
        &self.header_name
    }

    /// Normal, unless an entry is an Ocean cartridge (D-024).
    pub fn mode(&self) -> Mode {
        mode_of(self.entries.iter())
    }

    /// The size of a further program these entries leave room for (D-027): one of exactly this
    /// many bytes can be added and one byte more cannot.
    ///
    /// Answered by placing the entries afresh with a probe program appended last, as `push` would
    /// add it, because a build re-places everything (D-019) — on an Ocean cart that frees a ROMH
    /// run far larger than the top space the image as stored shows. Placement is greedy, so fit is
    /// not proven monotone in size; the search keeps one size known to fit and one known to fail,
    /// which makes the boundary exact even where it might not be the global maximum.
    pub fn free_space(&self) -> usize {
        let mode = self.mode();
        let mut entries = self.entries.clone();
        let probe = entries.len();
        entries.push(Program::new("", Vec::new()).into());
        // The metadata entry is placed last, after the probe, as a build after `push` would place
        // it. A set whose metadata can't be written won't build at all, so it counts for nothing here.
        if let Ok(Some(m)) = self.meta_entry() {
            entries.push(m);
        }
        let mut fits = |size: usize| {
            if let Entry::Program(p) = &mut entries[probe] {
                p.data.resize(size, 0);
            }
            Placer::new(mode).place_all(&entries).is_ok()
        };

        let (mut lo, mut hi) = (0, N_BANKS * BANK_SIZE * 2 + 1);
        while hi - lo > 1 {
            let mid = lo + (hi - lo) / 2;
            if fits(mid) {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        lo
    }

    /// Lay out the flash image without wrapping it in a container.
    pub fn build_image(&self) -> Result<FlashImage, BuildError> {
        let meta = self.meta_entry()?;
        let slots = self.entries.len() + usize::from(meta.is_some());
        if slots > MAX_ENTRIES {
            return Err(BuildError::TooManyEntries {
                max: MAX_ENTRIES,
                got: slots,
                metadata: meta.is_some(),
            });
        }
        let mut oceans = self
            .entries
            .iter()
            .filter_map(Entry::as_cartridge)
            .filter(|c| c.kind.is_ocean());
        if let (Some(_), Some(second)) = (oceans.next(), oceans.next()) {
            return Err(BuildError::SecondOcean {
                name: second.name.clone(),
            });
        }
        if let Some(c) = self
            .entries
            .iter()
            .filter_map(Entry::as_cartridge)
            .find(|c| c.align_64k && !c.kind.is_xbank())
        {
            return Err(BuildError::AlignNotXbank {
                name: c.name.clone(),
            });
        }

        if self.menu == Menu::Stidcart {
            if let Some(c) = self
                .entries
                .iter()
                .filter_map(Entry::as_cartridge)
                .find(|c| c.kind.is_ocean())
            {
                return Err(BuildError::MenuNeedsNormalMode {
                    name: c.name.clone(),
                });
            }
            if let Some(p) = self
                .entries
                .iter()
                .filter_map(Entry::as_program)
                .find(|p| touches_io_area(&p.data))
            {
                return Err(BuildError::ProgramInIoArea {
                    name: p.name.clone(),
                });
            }
        }

        let mode = self.mode();
        let mut placer = Placer::new(mode);
        // The entries as written: the metadata entry, if any, last (D-054).
        let mut placed = self.entries.clone();
        placed.extend(meta);
        let placements = placer.place_all(&placed)?;

        let mut img = FlashImage::erased();
        reserved::write_eapi(&mut img);
        reserved::write_boot(&mut img, mode);
        reserved::write_loader(&mut img, mode, self.menu);
        reserved::write_ef_name(&mut img, &self.ef_name);

        for (i, entry) in placed.iter().enumerate() {
            let at = placements[i];
            let dir_entry = match entry {
                Entry::Program(program) => {
                    let (bank, offset) = (at.bank as usize, at.offset as usize);
                    if at.kind == EntryType::Prg {
                        img.write_program(bank, offset, &program.data);
                    } else {
                        img.write_confined(bank, offset, &program.data);
                    }
                    DirEntry {
                        name: program.name.clone(),
                        kind: at.kind,
                        bank: at.bank,
                        offset: at.offset,
                        size: program.data.len() as u32,
                        hidden: program.hidden,
                        align_64k: false,
                        subdir: false,
                        other_off: 0,
                        other_num: 0,
                    }
                }
                Entry::Cartridge(cartridge) => {
                    cartridge.write_to(&mut img, at.bank as usize);
                    cartridge.dir_entry(at.bank)
                }
            };
            img.write_at(
                DIRECTORY_OFFSET + i * ENTRY_LEN,
                &directory::encode(&dir_entry),
            );
        }

        img.write_at(
            DIRECTORY_OFFSET + placed.len() * ENTRY_LEN,
            &directory::end_marker(),
        );

        Ok(img)
    }

    /// The finished `.crt` bytes.
    pub fn build(&self) -> Result<Vec<u8>, BuildError> {
        Ok(crt::write(&self.build_image()?, &self.header_name))
    }
}

/// Whether a `.prg` (load address, then payload) would occupy any of `$D000-$DFFF` once loaded.
fn touches_io_area(prg: &[u8]) -> bool {
    if prg.len() < 2 {
        return false;
    }
    let start = u16::from_le_bytes([prg[0], prg[1]]) as usize;
    let end = start + (prg.len() - 2);
    // `end` is one past the last byte; a program with no payload occupies nothing.
    end > start && start <= 0xdfff && end > 0xd000
}

/// The mode an entry set implies: Ocean if any entry is an Ocean cartridge.
pub(crate) fn mode_of<'a>(entries: impl Iterator<Item = &'a Entry>) -> Mode {
    if entries
        .filter_map(Entry::as_cartridge)
        .any(|c| c.kind.is_ocean())
    {
        Mode::Ocean
    } else {
        Mode::Normal
    }
}
