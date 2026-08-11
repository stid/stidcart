//! EasyFS2 directory entries — 24 bytes each.

use crate::petscii;

pub const ENTRY_LEN: usize = 24;
pub const NAME_LEN: usize = 16;

/// 256 slots exist, but the last is always the end marker, so 255 entries are usable.
pub const MAX_ENTRIES: usize = 255;

pub const TYPE_MASK: u8 = 0x1f;
/// Bits 5-6 are reserved and must be set. A past bug in the reference implementation wrote them
/// as zero, which makes this format-relevant rather than cosmetic.
pub const RESERVED_FLAG_BITS: u8 = 0x60;
pub const HIDDEN_FLAG: u8 = 0x80;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum EntryType {
    /// A program stored interleaved: ROML, then ROMH of the same bank, then the next bank.
    Prg = 0x01,
    /// A program confined to ROML.
    PrgRomL = 0x02,
    /// A program confined to ROMH.
    PrgRomH = 0x03,
    End = 0x1f,
}

#[derive(Clone, Debug)]
pub struct DirEntry {
    pub name: String,
    pub kind: EntryType,
    pub bank: u16,
    /// Byte offset within the bank; bit 13 selects ROMH.
    pub offset: u16,
    pub size: u32,
    pub hidden: bool,
}

/// A slot of erased flash, which reads back as the end-marker type.
pub fn end_marker() -> [u8; ENTRY_LEN] {
    [crate::image::ERASED; ENTRY_LEN]
}

pub fn encode(entry: &DirEntry) -> [u8; ENTRY_LEN] {
    let mut p = [0u8; ENTRY_LEN];

    let mut name = petscii::ascii_to_petscii(&entry.name);
    name.truncate(NAME_LEN);
    p[..name.len()].copy_from_slice(&name);

    p[16] = (entry.kind as u8) | RESERVED_FLAG_BITS | if entry.hidden { HIDDEN_FLAG } else { 0 };
    p[17..19].copy_from_slice(&entry.bank.to_le_bytes());
    p[19..21].copy_from_slice(&entry.offset.to_le_bytes());
    p[21] = (entry.size & 0xff) as u8;
    p[22] = ((entry.size >> 8) & 0xff) as u8;
    p[23] = ((entry.size >> 16) & 0xff) as u8;

    p
}

#[cfg(test)]
mod tests {
    use super::*;

    fn an_entry() -> DirEntry {
        DirEntry {
            name: "custom name".into(),
            kind: EntryType::Prg,
            bank: 1,
            offset: 0,
            size: 0x2002,
            hidden: false,
        }
    }

    #[test]
    fn encodes_the_fields_the_reference_image_shows() {
        // Byte-for-byte from a dumped cart: name, then 61 01 00 00 00 02 20 00.
        let p = encode(&an_entry());
        let mut want_name = b"CUSTOM NAME".to_vec();
        want_name.resize(NAME_LEN, 0);
        assert_eq!(&p[..16], want_name.as_slice());
        assert_eq!(p[16], 0x61);
        assert_eq!(&p[17..19], &[0x01, 0x00]);
        assert_eq!(&p[19..21], &[0x00, 0x00]);
        assert_eq!(&p[21..24], &[0x02, 0x20, 0x00]);
    }

    #[test]
    fn hidden_sets_the_high_bit() {
        let mut e = an_entry();
        e.name = "easyloader ocm".into();
        e.hidden = true;
        e.offset = 0x2002;
        let p = encode(&e);
        assert_eq!(p[16], 0xe1);
        assert_eq!(&p[19..21], &[0x02, 0x20]);
    }

    #[test]
    fn reserved_bits_are_always_set() {
        for hidden in [false, true] {
            let mut e = an_entry();
            e.hidden = hidden;
            assert_eq!(encode(&e)[16] & RESERVED_FLAG_BITS, RESERVED_FLAG_BITS);
        }
    }

    #[test]
    fn size_is_a_24_bit_little_endian_field() {
        let mut e = an_entry();
        e.size = 0x123456;
        assert_eq!(&encode(&e)[21..24], &[0x56, 0x34, 0x12]);
    }

    #[test]
    fn an_overlong_name_is_truncated_to_the_field() {
        let mut e = an_entry();
        e.name = "a name far longer than sixteen".into();
        let p = encode(&e);
        assert_eq!(&p[..NAME_LEN], b"A NAME FAR LONGE");
        assert_eq!(p[16] & TYPE_MASK, EntryType::Prg as u8);
    }

    #[test]
    fn the_end_marker_reads_back_as_the_end_type() {
        assert_eq!(end_marker()[16] & TYPE_MASK, EntryType::End as u8);
    }
}
