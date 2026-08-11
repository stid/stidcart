//! The 1 MB flash image and how banks map into it.

pub const BANK_SIZE: usize = 0x2000;
pub const N_BANKS: usize = 64;
pub const ROMH_BASE: usize = 1 << 19;
pub const IMAGE_SIZE: usize = 1024 * 1024;

/// Unwritten flash reads as all ones.
pub const ERASED: u8 = 0xff;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Half {
    RomL,
    RomH,
}

impl Half {
    const fn base(self) -> usize {
        match self {
            Half::RomL => 0,
            Half::RomH => ROMH_BASE,
        }
    }
}

/// A whole EasyFlash image.
///
/// The layout is **not** bank-interleaved: all 64 ROML halves come first, then all 64 ROMH
/// halves. Reasoning from EasyFlash bank numbering instead gives `0x80000` as bank 32 ROML, which
/// holds nothing — it is really bank 0 ROMH, and a test in the reference implementation filled the
/// wrong half for its entire life on exactly this mistake.
#[derive(Clone)]
pub struct FlashImage {
    data: Vec<u8>,
}

impl FlashImage {
    pub fn erased() -> Self {
        FlashImage {
            data: vec![ERASED; IMAGE_SIZE],
        }
    }

    /// Byte offset of a bank half within the image.
    pub const fn offset(bank: usize, half: Half) -> usize {
        (bank << 13) | half.base()
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn bank(&self, bank: usize, half: Half) -> &[u8] {
        let at = Self::offset(bank, half);
        &self.data[at..at + BANK_SIZE]
    }

    pub fn write_at(&mut self, offset: usize, bytes: &[u8]) {
        self.data[offset..offset + bytes.len()].copy_from_slice(bytes);
    }

    /// Where an interleaved program entry starts, given the bank and offset word the directory
    /// records. Bit 13 of the offset word selects ROMH.
    fn program_start(bank: usize, offset: usize) -> usize {
        bank * BANK_SIZE + (offset & 0x1fff) + if offset & 0x2000 != 0 { ROMH_BASE } else { 0 }
    }

    /// Step to the next chunk of an interleaved entry: ROML -> ROMH of the same bank, then ROML
    /// of the next bank.
    fn next_chunk(addr: usize) -> usize {
        let addr = addr & !0x1fff;
        if addr & ROMH_BASE != 0 {
            addr - ROMH_BASE + BANK_SIZE
        } else {
            addr + ROMH_BASE
        }
    }

    pub fn write_program(&mut self, bank: usize, offset: usize, data: &[u8]) {
        let mut addr = Self::program_start(bank, offset);
        let mut rest = data;
        while !rest.is_empty() {
            let len = ((addr | 0x1fff) - addr + 1).min(rest.len());
            self.data[addr..addr + len].copy_from_slice(&rest[..len]);
            rest = &rest[len..];
            addr = Self::next_chunk(addr);
        }
    }

    pub fn read_program(&self, bank: usize, offset: usize, size: usize) -> Vec<u8> {
        let mut addr = Self::program_start(bank, offset);
        let mut out = Vec::with_capacity(size);
        while out.len() < size {
            let len = ((addr | 0x1fff) - addr + 1).min(size - out.len());
            out.extend_from_slice(&self.data[addr..addr + len]);
            addr = Self::next_chunk(addr);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roml_halves_precede_all_romh_halves() {
        assert_eq!(FlashImage::offset(0, Half::RomL), 0x00000);
        assert_eq!(FlashImage::offset(1, Half::RomL), 0x02000);
        assert_eq!(FlashImage::offset(63, Half::RomL), 0x7e000);
        assert_eq!(FlashImage::offset(0, Half::RomH), 0x80000);
        assert_eq!(FlashImage::offset(1, Half::RomH), 0x82000);
        assert_eq!(FlashImage::offset(63, Half::RomH), 0xfe000);
    }

    #[test]
    fn offset_0x80000_is_bank_0_romh_not_bank_32() {
        assert_eq!(FlashImage::offset(0, Half::RomH), 0x80000);
        assert_ne!(FlashImage::offset(32, Half::RomL), 0x80000);
    }

    #[test]
    fn a_fresh_image_is_erased() {
        let img = FlashImage::erased();
        assert_eq!(img.as_bytes().len(), IMAGE_SIZE);
        assert!(img.as_bytes().iter().all(|&b| b == ERASED));
    }

    #[test]
    fn program_round_trips_within_one_bank() {
        let mut img = FlashImage::erased();
        let data: Vec<u8> = (0..100).collect();
        img.write_program(1, 0, &data);
        assert_eq!(img.read_program(1, 0, data.len()), data);
    }

    #[test]
    fn program_round_trips_across_the_interleave() {
        let mut img = FlashImage::erased();
        let data: Vec<u8> = (0..(BANK_SIZE * 2 + 5)).map(|i| (i % 251) as u8).collect();
        img.write_program(1, 0, &data);
        assert_eq!(img.read_program(1, 0, data.len()), data);
        // The second chunk really did land in ROMH of the same bank.
        assert_eq!(
            img.as_bytes()[FlashImage::offset(1, Half::RomH)],
            data[BANK_SIZE]
        );
    }

    #[test]
    fn offset_bit_13_selects_romh() {
        let mut img = FlashImage::erased();
        img.write_program(3, 0x2002, &[0xab, 0xcd]);
        assert_eq!(img.as_bytes()[FlashImage::offset(3, Half::RomH) + 2], 0xab);
    }
}
