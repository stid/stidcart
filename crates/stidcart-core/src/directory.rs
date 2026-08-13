//! EasyFS2 directory entries — 24 bytes each.

use crate::petscii;

pub const ENTRY_LEN: usize = 24;
pub const NAME_LEN: usize = 16;

/// 256 slots exist, but the last is always the end marker, so 255 entries are usable.
pub const MAX_ENTRIES: usize = 255;

pub const TYPE_MASK: u8 = 0x1f;
/// Bits 5-6 are reserved and must be set. They are also the EasyFS1/EasyFS2 version discriminator
/// on read — the format carries no version field, so these two bits are the whole signal (D-017).
/// See [`is_old_format`].
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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirEntry {
    pub name: String,
    pub kind: EntryType,
    pub bank: u16,
    /// Byte offset within the bank; bit 13 selects ROMH.
    pub offset: u16,
    pub size: u32,
    pub hidden: bool,
}

/// Bit 4 of the type value is the "this entry is a cartridge" discriminator.
pub const CARTRIDGE_BIT: u8 = 0x10;

/// What a directory slot turns out to hold. Only [`Slot::Program`] is representable as part of a
/// [`crate::CartBuilder`]; the rest exist so the loader can refuse precisely rather than guess.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Slot {
    /// The end-of-directory marker. Erased flash decodes to this, which is the format's intent.
    End,
    Program(DirEntry),
    /// A cartridge rather than a program — well-formed, but not something this tool can represent.
    Cartridge {
        type_value: u8,
    },
    /// A type value the format does not define. Malformed, not merely unsupported.
    Unknown {
        type_value: u8,
    },
}

/// A slot of erased flash, which reads back as the end-marker type.
pub fn end_marker() -> [u8; ENTRY_LEN] {
    [crate::image::ERASED; ENTRY_LEN]
}

/// Does this slot belong to an old-format (EasyFS1) directory?
///
/// The reserved bits not both being set is what says so — the format has no version field, so this
/// is the only signal there is (D-017). It is unambiguous for anything the reference implementation
/// wrote, which sets the bits unconditionally.
///
/// Only meaningful for a non-end slot. An end marker is `0xFF`-filled, so its reserved bits are set
/// and it never classifies a directory either way; [`read`](crate::read) consults this after
/// [`decode`] has ruled the slot out as [`Slot::End`].
pub fn is_old_format(raw: &[u8; ENTRY_LEN]) -> bool {
    raw[16] & RESERVED_FLAG_BITS != RESERVED_FLAG_BITS
}

/// The inverse of [`encode`].
///
/// Deliberately blind to the reserved flag bits, which say which *format* the directory is in
/// rather than anything about this entry — one slot cannot answer that, since the first non-end
/// slot settles it for all of them. The walk in [`read`](crate::read) owns that state and calls
/// [`is_old_format`]; keeping it out of here leaves `decode` a true inverse of [`encode`].
pub fn decode(raw: &[u8; ENTRY_LEN]) -> Slot {
    let flags = raw[16];
    let type_value = flags & TYPE_MASK;

    if type_value == EntryType::End as u8 {
        return Slot::End;
    }

    let kind = match type_value {
        v if v == EntryType::Prg as u8 => EntryType::Prg,
        v if v == EntryType::PrgRomL as u8 => EntryType::PrgRomL,
        v if v == EntryType::PrgRomH as u8 => EntryType::PrgRomH,
        v if v & CARTRIDGE_BIT != 0 => return Slot::Cartridge { type_value: v },
        v => return Slot::Unknown { type_value: v },
    };

    // The hidden flag is carried on the entry rather than turned into a slot of its own. Refusing
    // it is the *rebuild* path's business — `CartBuilder` cannot express hiding, so rebuilding one
    // would drop the flag — and a reader that only describes an image has no reason to inherit
    // that. Decoding it here keeps `decode` a true inverse of `encode`.
    Slot::Program(DirEntry {
        name: petscii::petscii_to_ascii(&raw[..NAME_LEN]),
        kind,
        bank: u16::from_le_bytes([raw[17], raw[18]]),
        offset: u16::from_le_bytes([raw[19], raw[20]]),
        size: u32::from_le_bytes([raw[21], raw[22], raw[23], 0]),
        hidden: flags & HIDDEN_FLAG != 0,
    })
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

    #[test]
    fn decode_inverts_encode() {
        let e = an_entry();
        let Slot::Program(back) = decode(&encode(&e)) else {
            panic!("a plain program entry should decode as one");
        };
        assert_eq!(back.name, e.name);
        assert_eq!(back.kind, e.kind);
        assert_eq!(back.bank, e.bank);
        assert_eq!(back.offset, e.offset);
        assert_eq!(back.size, e.size);
        assert!(!back.hidden);
    }

    #[test]
    fn decode_recovers_the_full_24_bit_size_and_the_half_selector() {
        let mut e = an_entry();
        e.size = 0x123456;
        e.offset = 0x2002;
        let Slot::Program(back) = decode(&encode(&e)) else {
            panic!("should decode as a program");
        };
        assert_eq!(back.size, 0x123456);
        assert_eq!(back.offset, 0x2002, "bit 13 selects ROMH and must survive");
    }

    #[test]
    fn the_end_marker_decodes_as_end() {
        assert_eq!(decode(&end_marker()), Slot::End);
    }

    #[test]
    fn a_hidden_entry_decodes_as_a_program_carrying_the_flag() {
        // It used to decode as a slot of its own, which discarded the position and size and forced
        // every reader to refuse it. Refusing belongs to the rebuild path; decoding describes.
        let mut e = an_entry();
        e.hidden = true;
        let Slot::Program(back) = decode(&encode(&e)) else {
            panic!("a hidden entry is still a program entry");
        };
        assert!(back.hidden);
        assert_eq!(back.name, e.name);
        assert_eq!(
            (back.bank, back.offset, back.size),
            (e.bank, e.offset, e.size)
        );
    }

    #[test]
    fn a_cartridge_type_decodes_as_a_cartridge() {
        let mut raw = encode(&an_entry());
        // 0x11 is a 16k cartridge; every cartridge type carries bit 4.
        raw[16] = 0x11 | RESERVED_FLAG_BITS;
        assert_eq!(decode(&raw), Slot::Cartridge { type_value: 0x11 });
    }

    #[test]
    fn an_undefined_type_decodes_as_unknown_not_as_a_cartridge() {
        let mut raw = encode(&an_entry());
        raw[16] = 0x07 | RESERVED_FLAG_BITS;
        assert_eq!(decode(&raw), Slot::Unknown { type_value: 0x07 });
    }

    #[test]
    fn the_reserved_bits_classify_the_format_and_do_not_reach_decode() {
        // These two bits are the EasyFS1/EasyFS2 discriminator (D-017). This module answers only
        // "which format is this slot spelled in"; deciding what to do about it needs the whole
        // directory, so it belongs to the walk in `read` and `decode` stays blind to it.
        let mut raw = encode(&an_entry());
        assert!(!is_old_format(&raw), "encode always sets the reserved bits");

        raw[16] &= !RESERVED_FLAG_BITS;
        assert!(
            is_old_format(&raw),
            "clear reserved bits mean the old format"
        );
        assert!(
            matches!(decode(&raw), Slot::Program(_)),
            "decode reports the entry the bytes spell and leaves the format question alone — one \
             slot cannot answer it, since the first non-end slot settles it for all of them"
        );
    }

    #[test]
    fn an_end_marker_never_reads_as_the_old_format() {
        // Erased flash is 0xFF, so the reserved bits are set in every end marker. If this were not
        // so, every empty trailing slot would classify the directory as EasyFS1.
        assert!(!is_old_format(&end_marker()));
    }
}
