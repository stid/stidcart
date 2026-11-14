//! The flash image as a hex viewer shows it: rows of bytes with the address the C64 sees, and
//! which part of the cart owns each byte range.
//!
//! Rows run **bank by bank**, ROML then ROMH of each bank, which is the order the machine banks
//! them in and the order an interleaved program is stored in. That is not the image's own order
//! (all ROML halves, then all ROMH halves, see [`FlashImage`]); a *position* here is an offset in
//! bank order, and [`position`] is the one place the two are converted.
//!
//! Every renderer of the view shares these values, as with [`crate::listing`] (D-032), so the
//! addresses, the text column and the ownership cannot differ between them.

use crate::crt;
use crate::directory::{DirEntry, EntryType};
use crate::image::{FlashImage, Half, BANK_SIZE, IMAGE_SIZE, N_BANKS};
use crate::petscii;
use crate::read::{self, ReadError};
use crate::reserved;

/// Bytes in one row.
pub const LINE_BYTES: usize = 16;
/// Rows in the whole image.
pub const LINES: usize = IMAGE_SIZE / LINE_BYTES;

/// Where ROML and ROMH appear to the C64 in 16K mode. Ultimax carts and the boot code see ROMH at
/// `$E000` instead; the view keeps one address per half so a row reads the same whoever owns it.
const ROML_ADDR: u16 = 0x8000;
const ROMH_ADDR: u16 = 0xa000;

/// One row of the view.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub bank: u8,
    pub half: Half,
    /// The C64 address of the row's first byte.
    pub addr: u16,
    pub bytes: [u8; LINE_BYTES],
    /// One character per byte, through the mapping the name fields use, `.` for the unprintable.
    pub text: String,
}

/// What a byte range belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    /// The menu loader's 8 KB slot, wherever the cart's mode puts it.
    Loader,
    Directory,
    Eapi,
    /// The EF-Name magic and the cartridge name after it.
    Name,
    Boot,
    /// The directory entry at this index, as the listing numbers it.
    Entry(usize),
    /// The hidden metadata entry (D-054), which the listing does not number.
    Meta,
}

/// A run of bytes with one owner. `start` is a position in bank order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub start: usize,
    pub len: usize,
    pub owner: Owner,
}

/// The position, in bank order, of a byte `offset` into one half of `bank`.
pub const fn position(bank: usize, half: Half, offset: usize) -> usize {
    bank * 2 * BANK_SIZE
        + match half {
            Half::RomL => 0,
            Half::RomH => BANK_SIZE,
        }
        + offset
}

/// `count` rows starting at row `first`, cut where the image ends.
///
/// Refuses only what the container parse refuses: a row is bytes, whatever the directory says.
pub fn lines(bytes: &[u8], first: usize, count: usize) -> Result<Vec<Line>, ReadError> {
    let image = crt::parse(bytes)?.image;
    let end = first.saturating_add(count).min(LINES);
    Ok((first..end).map(|row| line(&image, row)).collect())
}

fn line(image: &FlashImage, row: usize) -> Line {
    let at = row * LINE_BYTES;
    let bank = at / (2 * BANK_SIZE);
    let within = at % BANK_SIZE;
    let (half, base) = if at % (2 * BANK_SIZE) < BANK_SIZE {
        (Half::RomL, ROML_ADDR)
    } else {
        (Half::RomH, ROMH_ADDR)
    };
    let mut bytes = [0; LINE_BYTES];
    bytes.copy_from_slice(&image.bank(bank, half)[within..within + LINE_BYTES]);
    Line {
        bank: bank as u8,
        half,
        addr: base + within as u16,
        bytes,
        text: bytes.iter().map(|&b| text_char(b)).collect(),
    }
}

fn text_char(b: u8) -> char {
    let c = petscii::to_char(b);
    if c.is_control() {
        '.'
    } else {
        c
    }
}

/// Every owned range of the image, ordered by position and never overlapping.
///
/// Refuses exactly what [`read::inspect`] refuses, so it describes any image the listing does.
/// Where two owners claim the same bytes (an image laid out by something careless), the reserved
/// areas win, then the lower entry.
pub fn regions(bytes: &[u8]) -> Result<Vec<Region>, ReadError> {
    let info = read::inspect(bytes)?;
    let romh0 = |image_offset: usize| {
        position(
            0,
            Half::RomH,
            image_offset - FlashImage::offset(0, Half::RomH),
        )
    };
    let loader = match info.mode() {
        reserved::Mode::Normal => position(0, Half::RomL, 0),
        reserved::Mode::Ocean => position(1, Half::RomH, 0),
    };
    let mut claims = vec![
        (loader, reserved::LOADER_SIZE, Owner::Loader),
        (
            romh0(reserved::DIRECTORY_OFFSET),
            reserved::EAPI_OFFSET - reserved::DIRECTORY_OFFSET,
            Owner::Directory,
        ),
        (
            romh0(reserved::EAPI_OFFSET),
            reserved::EAPI_SIZE,
            Owner::Eapi,
        ),
        (
            romh0(reserved::EFNAME_MAGIC_OFFSET),
            reserved::BOOT_OFFSET - reserved::EFNAME_MAGIC_OFFSET,
            Owner::Name,
        ),
        (
            romh0(reserved::BOOT_OFFSET),
            reserved::BOOT_SIZE,
            Owner::Boot,
        ),
    ];
    for (index, entry) in info.entries.iter().enumerate() {
        claim(entry, Owner::Entry(index), &mut claims);
    }
    if let Some(entry) = &info.meta_entry {
        claim(entry, Owner::Meta, &mut claims);
    }

    // Stable, so of two claims starting together the earlier one (reserved, then by index) leads.
    claims.sort_by_key(|c| c.0);
    let mut regions: Vec<Region> = Vec::new();
    for (start, len, owner) in claims {
        let taken = regions.last().map_or(0, |r| r.start + r.len);
        let end = start + len;
        let start = start.max(taken);
        if start >= end {
            continue;
        }
        match regions.last_mut() {
            Some(last) if last.owner == owner && last.start + last.len == start => {
                last.len += end - start;
            }
            _ => regions.push(Region {
                start,
                len: end - start,
                owner,
            }),
        }
    }
    Ok(regions)
}

/// Add the ranges an entry occupies, half by half, as the directory records them.
fn claim(entry: &DirEntry, owner: Owner, claims: &mut Vec<(usize, usize, Owner)>) {
    let bank = entry.bank as usize;
    if entry.kind.is_cartridge() {
        let (primary, other) = (
            (0, entry.primary_num() as usize),
            (entry.other_off as usize, entry.other_num as usize),
        );
        let (roml, romh) = match entry.kind.primary() {
            Half::RomL => (primary, other),
            Half::RomH => (other, primary),
        };
        for (half, (off, num)) in [(Half::RomL, roml), (Half::RomH, romh)] {
            for b in bank + off..bank + off + num {
                claims.push((position(b, half, 0), BANK_SIZE, owner));
            }
        }
        return;
    }

    // A program: from its offset to the end of the half, then on by its kind (format spec,
    // "Bytes 19-20 for PRG entries").
    let mut b = bank;
    let mut half = if entry.offset & 0x2000 != 0 {
        Half::RomH
    } else {
        Half::RomL
    };
    let mut pos = (entry.offset & 0x1fff) as usize;
    let mut rest = entry.size as usize;
    while rest > 0 && b < N_BANKS {
        let len = (BANK_SIZE - pos).min(rest);
        claims.push((position(b, half, pos), len, owner));
        rest -= len;
        pos = 0;
        if entry.kind != EntryType::Prg {
            b += 1;
        } else if half == Half::RomL {
            half = Half::RomH;
        } else {
            half = Half::RomL;
            b += 1;
        }
    }
}
