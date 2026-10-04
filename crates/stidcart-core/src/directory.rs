//! EasyFS2 directory entries — 24 bytes each.

use crate::image::Half;
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

/// Bit 4 of the type value is the "this entry is a cartridge" discriminator.
pub const CARTRIDGE_BIT: u8 = 0x10;

/// The EasyFlash subdirectory kind: a cartridge whose payload is a whole nested directory. It is
/// the one live type value this tool does not model, so it is not an [`EntryType`] variant.
pub const SUBDIRECTORY_TYPE: u8 = 0x1a;

/// CrtUsage — what bytes 19-20 mean for a cartridge entry.
pub const USAGE_MSB: u16 = 0x8000;
pub const USAGE_SUBDIR: u16 = 0x4000;
pub const USAGE_ALIGN_64K: u16 = 0x2000;
pub const USAGE_OTHER_OFF_MASK: u16 = 0x1f00;
pub const USAGE_OTHER_NUM_MASK: u16 = 0x00ff;

/// Every live entry kind, carrying the type value the format assigns it (`easyfs.c:64-80`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum EntryType {
    /// A program stored interleaved: ROML, then ROMH of the same bank, then the next bank.
    Prg = 0x01,
    /// A program confined to ROML.
    PrgRomL = 0x02,
    /// A program confined to ROMH.
    PrgRomH = 0x03,
    K8 = 0x10,
    K16 = 0x11,
    Ultimax16 = 0x12,
    Ultimax8 = 0x13,
    Ocean512 = 0x14,
    Ocean = 0x15,
    Xbank8 = 0x1c,
    Xbank16 = 0x1d,
    XbankUltimax = 0x1e,
    End = 0x1f,
}

impl EntryType {
    /// The nine cartridge kinds, in type-value order.
    pub const CARTRIDGES: [EntryType; 9] = [
        EntryType::K8,
        EntryType::K16,
        EntryType::Ultimax16,
        EntryType::Ultimax8,
        EntryType::Ocean512,
        EntryType::Ocean,
        EntryType::Xbank8,
        EntryType::Xbank16,
        EntryType::XbankUltimax,
    ];

    pub fn from_value(v: u8) -> Option<EntryType> {
        use EntryType::*;
        Some(match v {
            0x01 => Prg,
            0x02 => PrgRomL,
            0x03 => PrgRomH,
            0x10 => K8,
            0x11 => K16,
            0x12 => Ultimax16,
            0x13 => Ultimax8,
            0x14 => Ocean512,
            0x15 => Ocean,
            0x1c => Xbank8,
            0x1d => Xbank16,
            0x1e => XbankUltimax,
            0x1f => End,
            _ => return None,
        })
    }

    pub fn is_program(self) -> bool {
        matches!(
            self,
            EntryType::Prg | EntryType::PrgRomL | EntryType::PrgRomH
        )
    }

    pub fn is_cartridge(self) -> bool {
        self as u8 & CARTRIDGE_BIT != 0 && self != EntryType::End
    }

    pub fn is_ocean(self) -> bool {
        matches!(self, EntryType::Ocean | EntryType::Ocean512)
    }

    pub fn is_xbank(self) -> bool {
        matches!(
            self,
            EntryType::Xbank8 | EntryType::Xbank16 | EntryType::XbankUltimax
        )
    }

    /// The half a cartridge's start bank refers to. Bit 1 of the type value set means the
    /// *opposite* half is ROML, i.e. the primary is ROMH — the three Ultimax kinds
    /// (`easyfs.c:124-127`). Programs have no primary half; they answer ROML here so the caller
    /// need not special-case them, but the bit-13 selector in `offset` is what places a program.
    pub fn primary(self) -> Half {
        if self.is_cartridge() && self as u8 & 2 != 0 {
            Half::RomH
        } else {
            Half::RomL
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DirEntry {
    pub name: String,
    pub kind: EntryType,
    pub bank: u16,
    /// Programs: byte offset within the bank, bit 13 selecting ROMH. Cartridges: always 0 — their
    /// bytes 19-20 are the CrtUsage word, spelled out in the three fields below.
    pub offset: u16,
    pub size: u32,
    pub hidden: bool,
    /// Cartridges only: the 64 KB alignment request.
    pub align_64k: bool,
    /// Cartridges only: where the opposite half's run starts, in banks from `bank`.
    pub other_off: u8,
    /// Cartridges only: how many banks the opposite half's run holds. The primary half's count is
    /// not stored — it is `size / 0x2000 - other_num`.
    pub other_num: u8,
}

impl DirEntry {
    /// Banks in the primary half. Only meaningful for a cartridge whose geometry has been checked
    /// (`other_num <= size / 0x2000`), which the directory walk does before anything else.
    pub fn primary_num(&self) -> u32 {
        (self.size / crate::image::BANK_SIZE as u32).saturating_sub(u32::from(self.other_num))
    }
}

/// What a directory slot turns out to hold.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Slot {
    /// The end-of-directory marker. Erased flash decodes to this, which is the format's intent.
    End,
    Program(DirEntry),
    Cartridge(DirEntry),
    /// An EasyFlash subdirectory — type `0x1a`, or any cartridge type with the subdirectory bit in
    /// its usage word. Well-formed, and the one kind this tool does not model.
    Subdirectory,
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
/// [`is_old_format`]; keeping it out of here leaves `decode` a true inverse of `encode`.
pub fn decode(raw: &[u8; ENTRY_LEN]) -> Slot {
    let flags = raw[16];
    let type_value = flags & TYPE_MASK;

    if type_value == EntryType::End as u8 {
        return Slot::End;
    }
    if type_value == SUBDIRECTORY_TYPE {
        return Slot::Subdirectory;
    }
    let Some(kind) = EntryType::from_value(type_value) else {
        return Slot::Unknown { type_value };
    };

    let word = u16::from_le_bytes([raw[19], raw[20]]);
    let mut entry = DirEntry {
        name: petscii::petscii_to_ascii(&raw[..NAME_LEN]),
        kind,
        bank: u16::from_le_bytes([raw[17], raw[18]]),
        offset: word,
        size: u32::from_le_bytes([raw[21], raw[22], raw[23], 0]),
        hidden: flags & HIDDEN_FLAG != 0,
        align_64k: false,
        other_off: 0,
        other_num: 0,
    };

    if !kind.is_cartridge() {
        return Slot::Program(entry);
    }
    if word & USAGE_SUBDIR != 0 {
        return Slot::Subdirectory;
    }
    entry.offset = 0;
    entry.align_64k = word & USAGE_ALIGN_64K != 0;
    entry.other_off = ((word & USAGE_OTHER_OFF_MASK) >> 8) as u8;
    entry.other_num = (word & USAGE_OTHER_NUM_MASK) as u8;
    Slot::Cartridge(entry)
}

pub fn encode(entry: &DirEntry) -> [u8; ENTRY_LEN] {
    let mut p = [0u8; ENTRY_LEN];

    let mut name = petscii::ascii_to_petscii(&entry.name);
    name.truncate(NAME_LEN);
    p[..name.len()].copy_from_slice(&name);

    p[16] = (entry.kind as u8) | RESERVED_FLAG_BITS | if entry.hidden { HIDDEN_FLAG } else { 0 };
    p[17..19].copy_from_slice(&entry.bank.to_le_bytes());

    let word = if entry.kind.is_cartridge() {
        USAGE_MSB
            | if entry.align_64k { USAGE_ALIGN_64K } else { 0 }
            | ((u16::from(entry.other_off) << 8) & USAGE_OTHER_OFF_MASK)
            | (u16::from(entry.other_num) & USAGE_OTHER_NUM_MASK)
    } else {
        entry.offset
    };
    p[19..21].copy_from_slice(&word.to_le_bytes());
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
            align_64k: false,
            other_off: 0,
            other_num: 0,
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
        assert_eq!(back, e);
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
    fn a_cartridge_type_decodes_as_a_cartridge_with_its_usage_word_unpacked() {
        let mut raw = encode(&an_entry());
        // 0x11 is a 16k cartridge; usage word: MSB, aligned, opposite run at +3 for 2 banks.
        raw[16] = 0x11 | RESERVED_FLAG_BITS;
        raw[19..21].copy_from_slice(&(0x8000u16 | 0x2000 | (3 << 8) | 2).to_le_bytes());
        let Slot::Cartridge(c) = decode(&raw) else {
            panic!("should decode as a cartridge: {:?}", decode(&raw));
        };
        assert_eq!(c.kind, EntryType::K16);
        assert_eq!((c.align_64k, c.other_off, c.other_num), (true, 3, 2));
        assert_eq!(c.offset, 0, "a cartridge has no byte offset");
    }

    #[test]
    fn a_subdirectory_decodes_as_such_by_type_or_by_usage_bit() {
        let mut raw = encode(&an_entry());
        raw[16] = SUBDIRECTORY_TYPE | RESERVED_FLAG_BITS;
        assert_eq!(decode(&raw), Slot::Subdirectory);

        let mut raw = encode(&an_entry());
        raw[16] = 0x11 | RESERVED_FLAG_BITS;
        raw[19..21].copy_from_slice(&(USAGE_MSB | USAGE_SUBDIR).to_le_bytes());
        assert_eq!(decode(&raw), Slot::Subdirectory);
    }

    #[test]
    fn an_undefined_type_decodes_as_unknown_not_as_a_cartridge() {
        let mut raw = encode(&an_entry());
        raw[16] = 0x07 | RESERVED_FLAG_BITS;
        assert_eq!(decode(&raw), Slot::Unknown { type_value: 0x07 });
        // 0x16..0x19 and 0x1b carry the cartridge bit but name nothing the format defines.
        raw[16] = 0x17 | RESERVED_FLAG_BITS;
        assert_eq!(decode(&raw), Slot::Unknown { type_value: 0x17 });
    }

    #[test]
    fn the_primary_half_is_romh_exactly_for_the_ultimax_kinds() {
        for kind in EntryType::CARTRIDGES {
            let want = if matches!(
                kind,
                EntryType::Ultimax8 | EntryType::Ultimax16 | EntryType::XbankUltimax
            ) {
                Half::RomH
            } else {
                Half::RomL
            };
            assert_eq!(kind.primary(), want, "{kind:?}");
        }
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
