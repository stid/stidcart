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
//! - cartridge entries and unrecognised reserved firmware are **refused as unsupported**,
//!   distinguishably from malformed input, rather than misparsed. A hidden entry used to be refused
//!   alongside them; it is not, since the description carries the flag (D-020);
//! - an old-format (EasyFS1) directory is **refused rather than converted** (D-017). The reserved
//!   flag bits are what identify it, and unlike the refusals above this one applies to describing
//!   as well as rebuilding — read as EasyFS2 an EasyFS1 entry's bank field is decoded at the wrong
//!   width, so there is no accurate description to give.

use crate::directory::{self, DirEntry, EntryType, Slot, ENTRY_LEN};
use crate::image::{BANK_SIZE, N_BANKS};
use crate::place::Placer;
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
    CartridgeEntry {
        type_value: u8,
    },
    /// An entry confined to one half of a bank (type `0x02` or `0x03`). Refused rather than read:
    /// every reader here walks the interleaved progression, so reading one with this code would
    /// return part of the entry followed by whatever occupies the other half.
    ConfinedEntry {
        name: String,
        kind: EntryType,
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
                Unsupported::CartridgeEntry { type_value } => write!(
                    f,
                    "the cartridge holds a cartridge entry (type {type_value:#04x}), which is not supported yet"
                ),
                Unsupported::ConfinedEntry { name, kind } => write!(
                    f,
                    "the entry '{name}' is confined to one half of a bank ({kind:?}), which is not \
                     supported yet"
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

/// Walk an entry's interleave the way the image does, confirming every step stays inside flash.
///
/// This has to be checked before reading rather than during: stepping past the last bank wraps the
/// address back into the reserved region, so an over-long entry would quietly return the directory
/// as though it were program data. That is worse than a crash, and it is the shape of the bug the
/// format spec records in the reference implementation.
fn check_extent(bank: u16, offset: u16, size: u32) -> Result<(), ReadError> {
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
        // ROML -> ROMH of the same bank -> ROML of the next.
        if romh {
            bank += 1;
            romh = false;
        } else {
            romh = true;
        }
    }
    Ok(())
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
    /// The directory in slot order, which is the order the menu lists.
    pub entries: Vec<DirEntry>,
}

impl CartInfo {
    /// The contiguous run a further entry could actually occupy, given where these entries sit.
    ///
    /// Computed by replaying the entries into a placer at their recorded positions, because the
    /// figure a caller sizes a program against has to account for *where* the image is occupied.
    /// Subtracting the sizes from the capacity instead would report almost a full cartridge free
    /// for an image holding one small entry parked high — every byte below it is unused and none
    /// of it is placeable.
    pub fn free_space(&self) -> usize {
        let mut placer = Placer::new();
        for e in &self.entries {
            placer.occupy(e.bank, e.offset, e.size);
        }
        placer.free_space()
    }
}

/// The one directory walk. [`load`] and [`inspect`] are two views over it, so they cannot disagree
/// about what an image contains.
///
/// They *do* differ on what they refuse, and deliberately: a refusal whose reason is "the builder
/// cannot express this" is a precondition of rebuilding, and applying it to a read-only listing
/// makes the tool fail on exactly the images someone would want to inspect. So this walk refuses
/// only what it cannot **describe** — malformed slots, entries reaching outside the image, and the
/// two well-formed shapes with no representation here (a cartridge entry, whose type modelling
/// [`CartInfo`] does not have, and a half-confined entry, whose layout no reader here implements).
/// Everything else is described, and [`load`] applies the narrower rebuild rules on top.
fn read_directory(container: &crt::Container) -> Result<CartInfo, ReadError> {
    let mut entries = Vec::new();
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
            Slot::End => {
                return Ok(CartInfo {
                    header_name: container.header_name.clone(),
                    ef_name: reserved::read_ef_name(&container.image),
                    entries,
                })
            }
            Slot::Cartridge { type_value } => {
                return Err(Unsupported::CartridgeEntry { type_value }.into())
            }
            Slot::Unknown { type_value } => return Err(ReadError::BadEntry { slot, type_value }),
            Slot::Program(entry) => {
                if entry.kind != EntryType::Prg {
                    return Err(Unsupported::ConfinedEntry {
                        name: entry.name,
                        kind: entry.kind,
                    }
                    .into());
                }
                check_extent(entry.bank, entry.offset, entry.size)?;
                entries.push(entry);
            }
        }
    }

    Err(ReadError::DirectoryUnterminated)
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
/// the firmware this crate embeds, and no entry may carry a property `CartBuilder` cannot express.
pub fn load(bytes: &[u8]) -> Result<CartBuilder, ReadError> {
    let container = crt::parse(bytes)?;
    reserved::verify_regions(&container.image)?;
    let info = read_directory(&container)?;

    let mut builder = CartBuilder::new()
        .with_header_name(&info.header_name)
        .with_ef_name(&info.ef_name);

    for entry in &info.entries {
        let data = container.image.read_program(
            entry.bank as usize,
            entry.offset as usize,
            entry.size as usize,
        );
        builder = builder.with_program(Program::new(&entry.name, data).with_hidden(entry.hidden));
    }

    Ok(builder)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_entry_ending_exactly_at_the_top_of_flash_is_in_bounds() {
        assert!(check_extent(63, 0x2000, BANK_SIZE as u32).is_ok());
    }

    #[test]
    fn an_entry_continuing_past_the_last_bank_is_out_of_bounds() {
        assert!(check_extent(63, 0x2000, BANK_SIZE as u32 + 1).is_err());
    }

    #[test]
    fn a_zero_length_entry_needs_no_space() {
        // The loop body never runs, so this documents the base case rather than the bank check.
        assert!(check_extent(63, 0x3fff, 0).is_ok());
    }

    #[test]
    fn a_bank_past_the_top_of_flash_is_out_of_bounds_immediately() {
        // The guard has to fire before the first byte is consumed, not after.
        assert!(check_extent(N_BANKS as u16, 0, 1).is_err());
        assert!(check_extent(200, 0, 1).is_err());
    }

    #[test]
    fn an_entry_filling_the_last_bank_exactly_is_in_bounds() {
        assert!(check_extent(63, 0, 2 * BANK_SIZE as u32).is_ok());
        assert!(check_extent(63, 0, 2 * BANK_SIZE as u32 + 1).is_err());
    }
}
