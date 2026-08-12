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
//! - cartridge entries, hidden entries and unrecognised reserved firmware are **refused as
//!   unsupported**, distinguishably from malformed input, rather than misparsed.

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
    HiddenEntry {
        name: String,
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
            ReadError::Unsupported(u) => match u {
                Unsupported::CartridgeEntry { type_value } => write!(
                    f,
                    "the cartridge holds a cartridge entry (type {type_value:#04x}), which is not supported yet"
                ),
                Unsupported::HiddenEntry { name } => write!(
                    f,
                    "the entry '{name}' is hidden, which cannot be rebuilt unchanged yet"
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

    for slot in 0..reserved::DIRECTORY_SLOTS {
        let at = reserved::DIRECTORY_OFFSET + slot * ENTRY_LEN;
        let raw: [u8; ENTRY_LEN] = container.image.as_bytes()[at..at + ENTRY_LEN]
            .try_into()
            .expect("a directory slot is exactly one entry wide");

        match directory::decode(&raw) {
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

    if let Some(hidden) = info.entries.iter().find(|e| e.hidden) {
        return Err(Unsupported::HiddenEntry {
            name: hidden.name.clone(),
        }
        .into());
    }

    let mut builder = CartBuilder::new()
        .with_header_name(&info.header_name)
        .with_ef_name(&info.ef_name);

    for entry in &info.entries {
        let data = container.image.read_program(
            entry.bank as usize,
            entry.offset as usize,
            entry.size as usize,
        );
        builder = builder.with_program(Program::new(&entry.name, data));
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
