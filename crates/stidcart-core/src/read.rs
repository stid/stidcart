//! Loading a cartridge image back into the description that would produce it.
//!
//! This is the inverse of [`crate::CartBuilder::build`], and it returns a `CartBuilder` rather than
//! a type of its own so that "reading then rebuilding reproduces the original" is a direct property
//! instead of a claim about a conversion.
//!
//! Which of the reference implementation's loader behaviours are reproduced is decided per
//! behaviour rather than by a blanket rule (D-007), using the classification in the extracted
//! format spec:
//!
//! - an empty directory is **accepted** — refusing it is a `break`-ordering slip there, and this
//!   tool produces exactly that image from an empty request, so refusing it would mean the library
//!   could not read what it writes;
//! - the packet marker and total-length field are **validated**, which that implementation does not
//!   do;
//! - a directory with no end marker in any slot stays **refused**, which the format does mandate;
//! - type `0x1A` entries and unrecognised reserved firmware are **refused as unsupported**,
//!   distinguishably from malformed input, rather than misparsed. Hidden entries, cartridge
//!   entries and half-confined programs used to be refused alongside them; they are not, since the
//!   model carries all three (D-020, D-023). The subdirectory *bit* on an xbank is carried too;
//!   on any other cartridge kind it is **refused as malformed**, as the reference rejects it (D-029);
//! - a cartridge entry's geometry is checked against the top of flash **before** its primary bank
//!   count is derived, which the reference tool got wrong and paid for with a heap overflow;
//! - an old-format (EasyFS1) directory is **refused rather than converted** (D-017). The reserved
//!   flag bits are what identify it, and unlike the refusals above this one applies to describing
//!   as well as rebuilding — read as EasyFS2 an EasyFS1 entry's bank field is decoded at the wrong
//!   width, so there is no accurate description to give.

use crate::cartridge::Cartridge;
use crate::directory::{self, DirEntry, EntryType, Slot, ENTRY_LEN, NAME_LEN};
use crate::image::{BANK_SIZE, N_BANKS};
use crate::menu::{Menu, Recognised};
use crate::meta::{self, Decoded, KeptMeta, Meta};
use crate::place::Placer;
use crate::reserved::Mode;
use crate::{crt, reserved, CartBuilder, Program};

/// Why a container's packet was rejected.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PacketProblem {
    /// The four-byte `CHIP` marker was not there.
    Marker,
    /// The total-length field and the data-length field describe different packets.
    LengthDisagrees,
    /// A data size or load address the format does not permit.
    Geometry,
    /// A bank number past the top of flash.
    Bank,
}

/// Content that is well-formed but that this tool cannot represent, and so will not load rather
/// than load lossily. Kept separate from the malformed cases so a caller can tell "this file is
/// broken" from "this cartridge is fine, we cannot open it yet".
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Unsupported {
    /// A type `0x1A` entry, the format's EasyFlash "subdirectory". The one live entry kind this
    /// tool has no model for: the reference tool refuses it too, so nothing says what its layout
    /// is (D-029).
    Subdirectory {
        slot: usize,
    },
    ReservedRegionMismatch {
        region: &'static str,
    },
    /// The directory is in the older EasyFS1 format, which this tool reads nothing of. Refused
    /// rather than converted: its cart types are guessed from size in the only implementation that
    /// does read it, and there is no fixture here to check a reader against (D-007, D-017).
    ///
    /// Unlike the other variants this one refuses to *describe* as well as to rebuild. That is not
    /// a D-011 exception — an EasyFS1 entry's bank is one byte where EasyFS2's is two, so read as
    /// EasyFS2 there is no correct description to give.
    OldFormatDirectory {
        slot: usize,
    },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ReadError {
    NotACartridge,
    Truncated {
        need: usize,
        have: usize,
    },
    /// The header length field at `0x10` named an offset below the fixed header fields, which the
    /// reader has already used by the time it looks at the field (D-021).
    BadHeaderLength {
        stated: usize,
    },
    /// A malformed container packet. `at` is an offset **into the file**.
    BadPacket {
        at: usize,
        problem: PacketProblem,
    },
    /// A directory entry described a region outside the image. `at` is an offset **into the flash
    /// image**, not into the file — the two coordinate spaces do not coincide, and this variant
    /// only ever carries the latter.
    OutOfBounds {
        at: usize,
        len: usize,
    },
    /// A cartridge entry whose bank runs reach past the top of flash, or whose opposite half
    /// holds more banks than its size allows. Checked before any arithmetic on the fields, which is
    /// where the reference tool underflowed (D-022).
    BadGeometry {
        slot: usize,
    },
    /// A cartridge entry other than an xbank sets the subdirectory bit, which the reference tool
    /// rejects (`easyfs.c:412`) and the format gives no meaning on that kind (D-029).
    SubdirectoryBitMisplaced {
        slot: usize,
    },
    /// A directory slot carried a type value the format does not define.
    BadEntry {
        slot: usize,
        type_value: u8,
    },
    /// Every slot held an entry and none was the end marker. Accepting this would read as an empty
    /// cartridge and destroy the directory on the next save.
    DirectoryUnterminated,
    /// A slot disagreed with the format an earlier slot established. Malformed rather than
    /// unsupported: the format is a property of the whole directory, so one entry claiming a
    /// different one is corruption, not a cartridge this tool cannot open yet.
    DirectoryFormatDisagrees {
        slot: usize,
    },
    Unsupported(Unsupported),
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadError::NotACartridge => write!(f, "not an EasyFlash cartridge image"),
            ReadError::Truncated { need, have } => {
                write!(f, "image ends early: needed {need} bytes, {have} remain")
            }
            ReadError::BadHeaderLength { stated } => {
                write!(
                    f,
                    "the header says it is {stated:#x} bytes long, less than its fixed fields"
                )
            }
            ReadError::BadPacket { at, problem } => {
                let what = match problem {
                    PacketProblem::Marker => "no CHIP marker",
                    PacketProblem::LengthDisagrees => "stated length disagrees with its data",
                    PacketProblem::Geometry => "data size or load address the format does not allow",
                    PacketProblem::Bank => "a bank number past the top of flash",
                };
                write!(f, "bad packet at {at:#x} in the file: {what}")
            }
            ReadError::OutOfBounds { at, len } => write!(
                f,
                "an entry reaches outside the image: {len} bytes at {at:#x} in the flash image"
            ),
            ReadError::BadGeometry { slot } => write!(
                f,
                "directory slot {slot} describes a cartridge whose banks lie outside the image"
            ),
            ReadError::SubdirectoryBitMisplaced { slot } => write!(
                f,
                "directory slot {slot} sets the subdirectory bit on a cartridge that is not an \
                 xbank, which the format does not allow"
            ),
            ReadError::BadEntry { slot, type_value } => write!(
                f,
                "directory slot {slot} carries undefined entry type {type_value:#04x}"
            ),
            ReadError::DirectoryUnterminated => {
                write!(f, "the directory has no end marker in any slot")
            }
            ReadError::DirectoryFormatDisagrees { slot } => write!(
                f,
                "directory slot {slot} has the reserved flag bits clear while earlier slots have \
                 them set, so the directory contradicts itself about its own format"
            ),
            ReadError::Unsupported(u) => match u {
                Unsupported::Subdirectory { slot } => write!(
                    f,
                    "directory slot {slot} is an EasyFlash subdirectory entry (type 0x1a), which \
                     is not supported"
                ),
                Unsupported::ReservedRegionMismatch { region } => write!(
                    f,
                    "the {region} region does not match the firmware this tool embeds, so the image could not be rebuilt unchanged"
                ),
                Unsupported::OldFormatDirectory { slot } => write!(
                    f,
                    "the directory is in the old EasyFS format (slot {slot} has the reserved flag \
                     bits clear), which is not supported yet"
                ),
            },
        }
    }
}

impl std::error::Error for ReadError {}

impl From<Unsupported> for ReadError {
    fn from(u: Unsupported) -> Self {
        ReadError::Unsupported(u)
    }
}

/// Walk a program entry the way the image does, confirming every step stays inside flash.
///
/// This has to be checked before reading rather than during: stepping past the last bank wraps the
/// address back into the reserved region, so an over-long entry would quietly return the directory
/// as though it were program data. That is worse than a crash, and it is the shape of the bug the
/// format spec records in the reference implementation. An interleaved program alternates halves;
/// a confined one stays in its half and steps to the next bank.
fn check_extent(kind: EntryType, bank: u16, offset: u16, size: u32) -> Result<(), ReadError> {
    let at = crate::image::FlashImage::offset(
        bank as usize,
        if offset & 0x2000 != 0 {
            crate::image::Half::RomH
        } else {
            crate::image::Half::RomL
        },
    ) + (offset & 0x1fff) as usize;
    let mut bank = bank as usize;
    let mut romh = offset & 0x2000 != 0;
    let mut pos = (offset & 0x1fff) as usize;
    let mut left = size as usize;

    while left > 0 {
        if bank >= N_BANKS {
            return Err(ReadError::OutOfBounds {
                at,
                len: size as usize,
            });
        }
        left -= (BANK_SIZE - pos).min(left);
        pos = 0;
        if kind != EntryType::Prg {
            bank += 1;
        } else if romh {
            // ROML -> ROMH of the same bank -> ROML of the next.
            bank += 1;
            romh = false;
        } else {
            romh = true;
        }
    }
    Ok(())
}

/// A cartridge entry's two runs must both end at or below the top of flash, and its opposite
/// half cannot hold more banks than the whole entry — checked on the fields as read, before the
/// primary count is derived by subtraction (`easyfs.c:159-174, 210-230`).
fn check_cart_geometry(e: &DirEntry) -> bool {
    let banks = e.size / BANK_SIZE as u32;
    let other = u32::from(e.other_num);
    if other > banks {
        return false;
    }
    let bank = u32::from(e.bank);
    let primary = banks - other;
    bank + primary <= N_BANKS as u32 && bank + u32::from(e.other_off) + other <= N_BANKS as u32
}

/// A cartridge as its own directory records it — **positions included**.
///
/// This is the view [`load`] cannot give. A [`CartBuilder`] holds no positions at all, because a
/// rebuild re-places every entry from scratch; anything reporting on a file rather than rebuilding
/// it has to read what the file actually says.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CartInfo {
    /// The container header name, identifying the tool that produced the image.
    pub header_name: String,
    /// The cartridge name the menu shows.
    pub ef_name: String,
    /// The directory in slot order, which is the order the menu lists. The metadata entry is not
    /// one of them (D-054): it is [`CartInfo::meta_entry`].
    pub entries: Vec<DirEntry>,
    /// Which menu the loader slot holds, or `None` when it is neither this crate embeds nor an
    /// earlier build of ours. Reported, never refused on: only `load` requires a known menu
    /// (D-011, D-038, D-049).
    pub menu: Option<Recognised>,
    /// Each entry's metadata, one per entry: empty where no record attached.
    pub metas: Vec<Meta>,
    /// The metadata entry as stored, if the cart has one. Its space is occupied like any entry's.
    pub meta_entry: Option<DirEntry>,
    /// Records whose entry no longer matches them, dropped (AC-154).
    pub meta_dropped: usize,
    /// A metadata entry in a layout this build can't read, carried as stored (D-055).
    pub meta_kept: Option<KeptMeta>,
}

impl CartInfo {
    /// The contiguous run at the top of flash, given where these entries sit as stored.
    ///
    /// Computed by replaying the entries into a placer at their recorded positions, so it accounts
    /// for *where* the image is occupied. Subtracting the sizes from the capacity instead would
    /// report almost a full cartridge free for an image holding one small entry parked high —
    /// every byte below it is unused and none of it is placeable.
    ///
    /// This describes the file, not what an edit would accept: a rebuild re-places every entry
    /// (D-019), and on an Ocean cart that frees a ROMH run this figure never counts. For the size
    /// a further program can actually take, load the image and ask
    /// [`CartBuilder::free_space`](crate::CartBuilder::free_space) (D-027).
    pub fn free_space(&self) -> usize {
        let mut placer = Placer::new(self.mode());
        for e in &self.entries {
            placer.occupy(e);
        }
        if let Some(e) = &self.meta_entry {
            placer.occupy(e);
        }
        placer.free_space()
    }

    /// Normal, unless an entry is an Ocean cartridge (D-024).
    pub fn mode(&self) -> Mode {
        if self.entries.iter().any(|e| e.kind.is_ocean()) {
            Mode::Ocean
        } else {
            Mode::Normal
        }
    }
}

/// The one directory walk. [`load`] and [`inspect`] are two views over it, so they cannot disagree
/// about what an image contains.
///
/// They *do* differ on what they refuse, and deliberately: a refusal whose reason is "the builder
/// cannot express this" is a precondition of rebuilding, and applying it to a read-only listing
/// makes the tool fail on exactly the images someone would want to inspect. So this walk refuses
/// only what it cannot **describe** — malformed slots, entries reaching outside the image, and the
/// one well-formed shape with no representation here, the subdirectory entry. Everything else is
/// described, and [`load`] applies the narrower rebuild rules on top.
fn read_directory(container: &crt::Container) -> Result<CartInfo, ReadError> {
    let mut entries = Vec::new();
    // Each slot's 16 name bytes as stored, which a metadata record is matched against (D-055).
    let mut raw_names: Vec<[u8; NAME_LEN]> = Vec::new();
    // Whether any slot has already established the directory as EasyFS2. Only that direction needs
    // remembering: an EasyFS1 slot returns immediately, so the walk never continues under it.
    let mut seen_easyfs2 = false;

    for slot in 0..reserved::DIRECTORY_SLOTS {
        let at = reserved::DIRECTORY_OFFSET + slot * ENTRY_LEN;
        let raw: [u8; ENTRY_LEN] = container.image.as_bytes()[at..at + ENTRY_LEN]
            .try_into()
            .expect("a directory slot is exactly one entry wide");

        let decoded = directory::decode(&raw);

        // The reserved flag bits carry the format version, so they are read before the type is
        // acted on: under EasyFS1 the type values mean something else, and refusing by a type
        // decoded under the wrong format would name a fault the image does not have. End markers
        // are `0xFF`-filled, so their reserved bits are set and they never classify a directory.
        if !matches!(decoded, Slot::End) {
            if directory::is_old_format(&raw) {
                return Err(if seen_easyfs2 {
                    // An earlier slot already said EasyFS2, so this one contradicts it. That is
                    // corruption, not a second opinion about the format.
                    ReadError::DirectoryFormatDisagrees { slot }
                } else {
                    Unsupported::OldFormatDirectory { slot }.into()
                });
            }
            seen_easyfs2 = true;
        }

        match decoded {
            Slot::End => return Ok(finish(container, entries, raw_names)),
            Slot::Subdirectory => return Err(Unsupported::Subdirectory { slot }.into()),
            Slot::MisplacedSubdirBit => return Err(ReadError::SubdirectoryBitMisplaced { slot }),
            Slot::Unknown { type_value } => return Err(ReadError::BadEntry { slot, type_value }),
            Slot::Program(entry) => {
                check_extent(entry.kind, entry.bank, entry.offset, entry.size)?;
                entries.push(entry);
                raw_names.push(raw[..NAME_LEN].try_into().expect("sixteen bytes"));
            }
            Slot::Cartridge(entry) => {
                if !check_cart_geometry(&entry) {
                    return Err(ReadError::BadGeometry { slot });
                }
                entries.push(entry);
                raw_names.push(raw[..NAME_LEN].try_into().expect("sixteen bytes"));
            }
        }
    }

    Err(ReadError::DirectoryUnterminated)
}

/// The walk's last step, once the end marker is found: lift the metadata entry out of the entries,
/// attach its records, and name the menu.
///
/// The metadata entry is the *last* hidden program slot named `STIDCART.META` whose data starts
/// with the magic; a slot with the name but not the magic is an ordinary entry. This reads one
/// entry's data, which every slot's extent check has already bounded (D-055).
fn finish(
    container: &crt::Container,
    mut entries: Vec<DirEntry>,
    mut raw_names: Vec<[u8; NAME_LEN]>,
) -> CartInfo {
    let meta_name = directory::name_bytes(meta::META_NAME);
    let mut meta_entry = None;
    let mut decoded = None;
    for i in (0..entries.len()).rev() {
        let e = &entries[i];
        if !(e.kind.is_program() && e.hidden && raw_names[i] == meta_name) {
            continue;
        }
        let (bank, offset, size) = (e.bank as usize, e.offset as usize, e.size as usize);
        let data = if e.kind == EntryType::Prg {
            container.image.read_program(bank, offset, size)
        } else {
            container.image.read_confined(bank, offset, size)
        };
        match meta::decode(&data) {
            Decoded::NotMeta => continue,
            d => {
                decoded = Some(d);
                meta_entry = Some(entries.remove(i));
                raw_names.remove(i);
                break;
            }
        }
    }

    let mut metas = vec![Meta::default(); entries.len()];
    let (mut meta_dropped, mut meta_kept) = (0, None);
    match decoded {
        Some(Decoded::Records(records)) => {
            let mut matched = vec![false; entries.len()];
            for (index, name, m) in records {
                if index < entries.len() && !matched[index] && raw_names[index] == name {
                    matched[index] = true;
                    metas[index] = m;
                } else {
                    meta_dropped += 1;
                }
            }
        }
        Some(Decoded::Kept(k)) => meta_kept = Some(k),
        Some(Decoded::NotMeta) | None => {}
    }

    let mut info = CartInfo {
        header_name: container.header_name.clone(),
        ef_name: reserved::read_ef_name(&container.image),
        entries,
        menu: None,
        metas,
        meta_entry,
        meta_dropped,
        meta_kept,
    };
    info.menu = reserved::detect_menu(&container.image, info.mode());
    info
}

/// Read a `.crt` image's directory without recovering the program data it points at.
///
/// Deliberately does **not** check the reserved firmware regions. That check exists so a rebuild
/// cannot silently swap the firmware an image carries; describing an image rebuilds nothing, and
/// running it here would refuse every cart produced by any other tool or version.
///
/// It does refuse an old-format directory, which is the one refusal here that is not about
/// rebuilding. That is not the conflation D-011 was written against: the strictness split turns on
/// whether an image can be *described*, and an EasyFS1 directory read as EasyFS2 decodes its bank
/// field at the wrong width, so every position this would report is wrong.
pub fn inspect(bytes: &[u8]) -> Result<CartInfo, ReadError> {
    read_directory(&crt::parse(bytes)?)
}

/// Load a `.crt` image into the description that would produce it.
///
/// Stricter than [`inspect`] by exactly the rebuild preconditions: the reserved regions must hold
/// the firmware this crate embeds — for the mode the directory implies, which is why the directory
/// is walked first (D-024) — and no entry may carry a property `CartBuilder` cannot express.
pub fn load(bytes: &[u8]) -> Result<CartBuilder, ReadError> {
    load_inner(bytes, None)
}

/// [`load`] for a rebuild that switches the cart to `menu`: whatever the loader slot holds, even a
/// menu this crate doesn't know, is not checked, because the rebuild overwrites it. Every other
/// rebuild precondition holds as in [`load`] (D-050).
pub fn load_with_menu(bytes: &[u8], menu: Menu) -> Result<CartBuilder, ReadError> {
    load_inner(bytes, Some(menu))
}

fn load_inner(bytes: &[u8], replacing: Option<Menu>) -> Result<CartBuilder, ReadError> {
    let container = crt::parse(bytes)?;
    let info = read_directory(&container)?;
    let menu = match replacing {
        Some(menu) => reserved::verify_regions_replacing(&container.image, info.mode(), menu)?,
        None => reserved::verify_regions(&container.image, info.mode())?,
    };

    let mut builder = CartBuilder::new()
        .with_header_name(&info.header_name)
        .with_ef_name(&info.ef_name)
        .with_menu(menu);

    for (entry, meta) in info.entries.iter().zip(info.metas) {
        let (bank, offset, size) = (
            entry.bank as usize,
            entry.offset as usize,
            entry.size as usize,
        );
        match entry.kind {
            EntryType::Prg => {
                let data = container.image.read_program(bank, offset, size);
                builder.push(Program::new(&entry.name, data).with_hidden(entry.hidden));
            }
            EntryType::PrgRomL | EntryType::PrgRomH => {
                let data = container.image.read_confined(bank, offset, size);
                builder.push(Program::new(&entry.name, data).with_hidden(entry.hidden));
            }
            _ => builder.push(Cartridge::read_from(&container.image, entry)),
        }
        // Carried as the hidden flag is (AC-29): the records attached in the walk.
        let last = builder.entries().len() - 1;
        *builder.entry_meta_mut(last) = meta;
    }

    if let Some(kept) = info.meta_kept {
        builder.keep_meta(kept);
    }
    Ok(builder)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_entry_ending_exactly_at_the_top_of_flash_is_in_bounds() {
        assert!(check_extent(EntryType::Prg, 63, 0x2000, BANK_SIZE as u32).is_ok());
    }

    #[test]
    fn an_entry_continuing_past_the_last_bank_is_out_of_bounds() {
        assert!(check_extent(EntryType::Prg, 63, 0x2000, BANK_SIZE as u32 + 1).is_err());
    }

    #[test]
    fn a_zero_length_entry_needs_no_space() {
        // The loop body never runs, so this documents the base case rather than the bank check.
        assert!(check_extent(EntryType::Prg, 63, 0x3fff, 0).is_ok());
    }

    #[test]
    fn a_bank_past_the_top_of_flash_is_out_of_bounds_immediately() {
        // The guard has to fire before the first byte is consumed, not after.
        assert!(check_extent(EntryType::Prg, N_BANKS as u16, 0, 1).is_err());
        assert!(check_extent(EntryType::Prg, 200, 0, 1).is_err());
    }

    #[test]
    fn a_confined_entry_steps_banks_in_its_own_half() {
        // Two banks from bank 62 ROMH end exactly at the top; one byte more does not.
        assert!(check_extent(EntryType::PrgRomH, 62, 0x2000, 2 * BANK_SIZE as u32).is_ok());
        assert!(check_extent(EntryType::PrgRomH, 62, 0x2000, 2 * BANK_SIZE as u32 + 1).is_err());
        // Interleaved from the same start, the second bank half is 63 ROML, so three halves fit.
        assert!(check_extent(EntryType::Prg, 62, 0x2000, 3 * BANK_SIZE as u32).is_ok());
    }

    #[test]
    fn cart_geometry_is_checked_before_the_subtraction() {
        let mut e = DirEntry {
            name: "c".into(),
            kind: EntryType::K16,
            bank: 62,
            offset: 0,
            size: 2 * BANK_SIZE as u32,
            hidden: false,
            align_64k: false,
            subdir: false,
            other_off: 0,
            other_num: 1,
        };
        assert!(
            check_cart_geometry(&e),
            "one bank in each half from bank 62"
        );
        e.other_num = 3;
        assert!(
            !check_cart_geometry(&e),
            "more opposite banks than the entry holds"
        );
        e.other_num = 1;
        e.bank = 63;
        assert!(check_cart_geometry(&e));
        e.other_off = 1;
        assert!(!check_cart_geometry(&e), "the opposite run reaches bank 64");
    }

    #[test]
    fn an_entry_filling_the_last_bank_exactly_is_in_bounds() {
        assert!(check_extent(EntryType::Prg, 63, 0, 2 * BANK_SIZE as u32).is_ok());
        assert!(check_extent(EntryType::Prg, 63, 0, 2 * BANK_SIZE as u32 + 1).is_err());
    }
}
