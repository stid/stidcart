//! A cartridge held as an entry, and how one is derived from a `.crt` image.
//!
//! The nine cartridge kinds share one shape: a run of whole banks in ROML, a run in ROMH, and where
//! each run starts relative to the entry's start bank. The directory's CrtUsage word is derived from
//! that at encode time; nothing here stores the word itself (D-023).
//!
//! A cartridge is only ever *derived* from an image, by the rules the reference tool applies
//! (`cart.c:414-527, 628-800`): the container id picks the family, the packets and the header's
//! mode bits pick the kind. Nothing on the command line declares a kind — an image already says
//! what it is.

use crate::crt::{self, Packet};
use crate::directory::{DirEntry, EntryType};
use crate::image::{FlashImage, Half, BANK_SIZE, N_BANKS};
use crate::read::ReadError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cartridge {
    pub name: String,
    pub hidden: bool,
    /// Title, year, author, note and group, carried as `hidden` is (D-054).
    pub meta: crate::meta::Meta,
    /// Always one of [`EntryType::CARTRIDGES`]; [`Cartridge::from_crt`] is the only constructor.
    pub kind: EntryType,
    /// Ask the placer for a 64 KB boundary. Meaningful for the xbank kinds only.
    pub align_64k: bool,
    /// The directory's subdirectory bit, carried from a cart that had it and never interpreted.
    /// Only an xbank read back from an image can carry it; [`Cartridge::from_crt`] never sets it.
    pub subdir: bool,
    /// The ROML run: whole banks, contiguous, possibly empty.
    pub roml: Vec<u8>,
    /// Where the ROML run starts, in banks from the entry's start bank.
    pub roml_at: u8,
    /// The ROMH run, likewise.
    pub romh: Vec<u8>,
    pub romh_at: u8,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CartridgeError {
    /// No CRT header at all.
    NotACartridge,
    /// A container id this tool cannot hold as an entry: anything but 0 (generic), 5 (Ocean) and
    /// 33 (EasyFlash xbank). An EasyFlash multicart itself (32) lands here.
    UnsupportedId { id: u16 },
    /// A well-framed image whose packets do not describe a cartridge of its id's family.
    Geometry { reason: &'static str },
    /// The container itself is broken.
    Malformed(ReadError),
}

impl std::fmt::Display for CartridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CartridgeError::NotACartridge => write!(f, "not a cartridge image"),
            CartridgeError::UnsupportedId { id } => write!(
                f,
                "cartridge id {id} cannot be held as an entry (supported: 0 generic, 5 Ocean, 33 \
                 EasyFlash xbank)"
            ),
            CartridgeError::Geometry { reason } => write!(f, "cartridge image: {reason}"),
            CartridgeError::Malformed(e) => write!(f, "cartridge image: {e}"),
        }
    }
}

impl std::error::Error for CartridgeError {}

const ID_GENERIC: u16 = 0;
const ID_OCEAN: u16 = 5;
const ID_XBANK: u16 = 33;

/// The reference tool's mode enum, as the header's EXROM/GAME bits spell it (`cart.h:43-46`,
/// `cart.c:180`).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    K16,
    Ultimax,
    K8,
    Off,
}

fn mode(exrom: u8, game: u8) -> Mode {
    match (exrom & 1) | ((game & 1) << 1) {
        0 => Mode::K16,
        1 => Mode::Ultimax,
        2 => Mode::K8,
        _ => Mode::Off,
    }
}

fn geometry(reason: &'static str) -> CartridgeError {
    CartridgeError::Geometry { reason }
}

impl Cartridge {
    /// Derive a cartridge from a `.crt` image, or refuse it naming what it was.
    pub fn from_crt(bytes: &[u8]) -> Result<Cartridge, CartridgeError> {
        let (header, packets) = crt::packets(bytes).map_err(|e| match e {
            ReadError::NotACartridge => CartridgeError::NotACartridge,
            other => CartridgeError::Malformed(other),
        })?;
        let name = header.name.chars().take(16).collect();

        let (kind, roml, roml_at, romh, romh_at) = match header.id {
            ID_GENERIC => generic(&packets)?,
            ID_OCEAN => ocean(&packets)?,
            ID_XBANK => xbank(&packets, mode(header.exrom, header.game))?,
            id => return Err(CartridgeError::UnsupportedId { id }),
        };

        Ok(Cartridge {
            name,
            hidden: false,
            meta: Default::default(),
            kind,
            align_64k: false,
            subdir: false,
            roml,
            roml_at,
            romh,
            romh_at,
        })
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    pub fn with_hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    pub fn with_align_64k(mut self, align: bool) -> Self {
        self.align_64k = align;
        self
    }

    /// Whole banks in each run: `(roml, romh)`.
    pub fn banks(&self) -> (usize, usize) {
        (self.roml.len() / BANK_SIZE, self.romh.len() / BANK_SIZE)
    }

    pub fn size(&self) -> u32 {
        (self.roml.len() + self.romh.len()) as u32
    }

    /// The run in the half the kind calls primary, and the run in the other: `(offset, banks)`.
    pub fn primary_and_other(&self) -> ((u8, usize), (u8, usize)) {
        let (l, h) = self.banks();
        match self.kind.primary() {
            Half::RomL => ((self.roml_at, l), (self.romh_at, h)),
            Half::RomH => ((self.romh_at, h), (self.roml_at, l)),
        }
    }

    /// The directory entry for this cartridge placed at `bank`.
    pub fn dir_entry(&self, bank: u16) -> DirEntry {
        let (_, (other_off, other_num)) = self.primary_and_other();
        DirEntry {
            name: self.name.clone(),
            kind: self.kind,
            bank,
            offset: 0,
            size: self.size(),
            hidden: self.hidden,
            align_64k: self.align_64k,
            subdir: self.subdir,
            other_off,
            other_num: other_num as u8,
        }
    }

    /// Copy both runs into the image at the entry's start bank.
    pub fn write_to(&self, img: &mut FlashImage, bank: usize) {
        for (n, chunk) in self.roml.chunks(BANK_SIZE).enumerate() {
            img.write_at(
                FlashImage::offset(bank + self.roml_at as usize + n, Half::RomL),
                chunk,
            );
        }
        for (n, chunk) in self.romh.chunks(BANK_SIZE).enumerate() {
            img.write_at(
                FlashImage::offset(bank + self.romh_at as usize + n, Half::RomH),
                chunk,
            );
        }
    }

    /// The inverse of [`Cartridge::write_to`], from a directory entry whose geometry the walk has
    /// already checked against the top of flash.
    pub fn read_from(img: &FlashImage, entry: &DirEntry) -> Cartridge {
        let (primary_at, primary_num) = (0u8, entry.primary_num() as usize);
        let (other_at, other_num) = (entry.other_off, entry.other_num as usize);
        let ((roml_at, l), (romh_at, h)) = match entry.kind.primary() {
            Half::RomL => ((primary_at, primary_num), (other_at, other_num)),
            Half::RomH => ((other_at, other_num), (primary_at, primary_num)),
        };
        let run = |half, at: u8, n: usize| -> Vec<u8> {
            (0..n)
                .flat_map(|i| img.bank(entry.bank as usize + at as usize + i, half))
                .copied()
                .collect()
        };
        Cartridge {
            name: entry.name.clone(),
            hidden: entry.hidden,
            meta: Default::default(),
            kind: entry.kind,
            align_64k: entry.align_64k,
            subdir: entry.subdir,
            roml: run(Half::RomL, roml_at, l),
            roml_at,
            romh: run(Half::RomH, romh_at, h),
            romh_at,
        }
    }
}

type Derived = (EntryType, Vec<u8>, u8, Vec<u8>, u8);

/// Generic (id 0): one packet at `$8000` of one or two banks, optionally followed by one bank at
/// `$E000`; or one bank at `$E000` alone (`cart.c:414-457`). Whole banks only (D-023).
fn generic(packets: &[Packet<'_>]) -> Result<Derived, CartridgeError> {
    let whole = |p: &Packet<'_>| p.data.len() == BANK_SIZE;
    match packets {
        [first] if first.addr == 0x8000 && whole(first) => {
            Ok((EntryType::K8, first.data.to_vec(), 0, Vec::new(), 0))
        }
        [first] if first.addr == 0x8000 && first.data.len() == 2 * BANK_SIZE => Ok((
            EntryType::K16,
            first.data[..BANK_SIZE].to_vec(),
            0,
            first.data[BANK_SIZE..].to_vec(),
            0,
        )),
        [first] if first.addr == 0xe000 && whole(first) => {
            Ok((EntryType::Ultimax8, Vec::new(), 0, first.data.to_vec(), 0))
        }
        [first, second]
            if first.addr == 0x8000 && whole(first) && second.addr == 0xe000 && whole(second) =>
        {
            Ok((
                EntryType::Ultimax16,
                first.data.to_vec(),
                0,
                second.data.to_vec(),
                0,
            ))
        }
        [] => Err(geometry("no CHIP packets")),
        _ => Err(geometry(
            "a generic cartridge is one whole bank at $8000 or $E000, two at $8000, or one at \
             each",
        )),
    }
}

/// Ocean (id 5): whole banks at `$8000` or `$A000`, numbered straight through — the ROML chips
/// first, then the ROMH ones (`cart.c:490-527`). The kind follows the total size.
fn ocean(packets: &[Packet<'_>]) -> Result<Derived, CartridgeError> {
    let mut banks: Vec<Option<(Half, &[u8])>> = vec![None; N_BANKS];
    for p in packets {
        if p.data.len() != BANK_SIZE {
            return Err(geometry("an Ocean chip is exactly one bank"));
        }
        let half = match p.addr {
            0x8000 => Half::RomL,
            0xa000 => Half::RomH,
            _ => return Err(geometry("an Ocean chip loads at $8000 or $A000")),
        };
        let slot = &mut banks[p.bank as usize];
        if slot.is_some() {
            return Err(geometry("two Ocean chips claim the same bank"));
        }
        *slot = Some((half, p.data));
    }
    let present: Vec<(Half, &[u8])> = banks.iter().flatten().copied().collect();
    let n = present.len();
    if banks[..n].iter().any(Option::is_none) {
        return Err(geometry("Ocean chips must be numbered 0.. without gaps"));
    }
    let n_l = present.iter().take_while(|(h, _)| *h == Half::RomL).count();
    if present[n_l..].iter().any(|(h, _)| *h == Half::RomL) {
        return Err(geometry("Ocean ROML chips come before the ROMH ones"));
    }
    let kind = match (n * BANK_SIZE) >> 10 {
        32 | 128 | 256 => EntryType::Ocean,
        512 => EntryType::Ocean512,
        _ => return Err(geometry("an Ocean cartridge is 32, 128, 256 or 512 KiB")),
    };
    let roml: Vec<u8> = present[..n_l]
        .iter()
        .flat_map(|(_, d)| *d)
        .copied()
        .collect();
    let romh: Vec<u8> = present[n_l..]
        .iter()
        .flat_map(|(_, d)| *d)
        .copied()
        .collect();
    // An empty ROMH run sits at offset 0, as the directory's 5-bit field would store it anyway.
    let romh_at = if romh.is_empty() { 0 } else { n_l as u8 };
    Ok((kind, roml, 0, romh, romh_at))
}

/// EasyFlash xbank (id 33): any bank halves, laid into a full flash image; the kind is the
/// header's mode, and the primary run must start at bank 0 (`cart.c:259-337, 700-790`).
fn xbank(packets: &[Packet<'_>], mode: Mode) -> Result<Derived, CartridgeError> {
    let kind = match mode {
        Mode::K8 => EntryType::Xbank8,
        Mode::K16 => EntryType::Xbank16,
        Mode::Ultimax => EntryType::XbankUltimax,
        Mode::Off => return Err(geometry("an xbank cartridge's header has mode \"off\"")),
    };
    let mut img = FlashImage::erased();
    let mut present = [[false; N_BANKS]; 2];
    for p in packets {
        let bank = p.bank as usize;
        match (p.addr, p.data.len()) {
            (0x8000, BANK_SIZE) => {
                img.write_at(FlashImage::offset(bank, Half::RomL), p.data);
                present[0][bank] = true;
            }
            (0xa000 | 0xe000, BANK_SIZE) => {
                img.write_at(FlashImage::offset(bank, Half::RomH), p.data);
                present[1][bank] = true;
            }
            (0x8000, l) if l == 2 * BANK_SIZE => {
                img.write_at(FlashImage::offset(bank, Half::RomL), &p.data[..BANK_SIZE]);
                img.write_at(FlashImage::offset(bank, Half::RomH), &p.data[BANK_SIZE..]);
                present[0][bank] = true;
                present[1][bank] = true;
            }
            _ => {
                return Err(geometry(
                    "an xbank chip is one bank half, or both halves at $8000",
                ))
            }
        }
    }
    let run = |half: usize| -> (usize, usize) {
        let min = present[half].iter().position(|&b| b);
        let max = present[half].iter().rposition(|&b| b);
        match (min, max) {
            (Some(lo), Some(hi)) => (lo, hi + 1 - lo),
            _ => (0, 0),
        }
    };
    let (l_at, l_num) = run(0);
    let (h_at, h_num) = run(1);
    if l_num == 0 && h_num == 0 {
        return Err(geometry("an xbank cartridge with no banks"));
    }
    let (base, primary_num) = match kind.primary() {
        Half::RomL => (l_at, l_num),
        Half::RomH => (h_at, h_num),
    };
    if primary_num == 0 {
        return Err(geometry(
            "an xbank cartridge with no banks in its primary half is unbootable",
        ));
    }
    if base != 0 {
        return Err(geometry(
            "an xbank cartridge whose primary run does not start at bank 0 is unbootable",
        ));
    }
    let collect = |half, at: usize, n: usize| -> Vec<u8> {
        (0..n)
            .flat_map(|i| img.bank(at + i, half))
            .copied()
            .collect()
    };
    Ok((
        kind,
        collect(Half::RomL, l_at, l_num),
        l_at as u8,
        collect(Half::RomH, h_at, h_num),
        h_at as u8,
    ))
}
