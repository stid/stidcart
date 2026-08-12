//! The `.crt` container: a 64-byte header followed by CHIP packets.

use crate::image::{FlashImage, Half, BANK_SIZE, N_BANKS};
use crate::read::{PacketProblem, ReadError};

pub const HEADER_LEN: usize = 0x40;
/// A CHIP packet's fixed prelude, before its data.
pub const CHIP_HEADER_LEN: usize = 16;
pub const MAGIC: &[u8; 16] = b"C64 CARTRIDGE   ";
pub const NAME_OFFSET: usize = 0x20;
pub const NAME_LEN: usize = 32;

pub const CARTRIDGE_EASYFLASH: u16 = 32;

/// Chip type 2 is flash ROM.
const CHIP_TYPE_FLASH: u16 = 2;

/// Serialize the image. Bank halves that are entirely erased are omitted, so a nearly-empty cart
/// produces a small file rather than a 1 MB one.
pub fn write(img: &FlashImage, header_name: &str) -> Vec<u8> {
    let mut out = header(CARTRIDGE_EASYFLASH, header_name);

    for bank in 0..N_BANKS {
        for (half, addr) in [(Half::RomL, 0x8000u16), (Half::RomH, 0xa000u16)] {
            let data = img.bank(bank, half);
            if data.iter().all(|&b| b == crate::image::ERASED) {
                continue;
            }
            push_chip(&mut out, bank as u16, addr, data);
        }
    }

    out
}

/// What a container yields: the flash image it describes, and the name of the tool that wrote it.
#[derive(Clone)]
pub struct Container {
    pub image: FlashImage,
    pub header_name: String,
}

/// The inverse of [`write`]: validate the header, then replay every packet into an erased image.
///
/// Both the packet marker and the total-length field are checked, unlike the reference
/// implementation, which reads neither (see D-007). Every file-supplied length and bank is bounds-
/// checked before it reaches an index — the format spec records an unsigned underflow in the
/// equivalent path there that was a real memory-safety bug.
pub fn parse(bytes: &[u8]) -> Result<Container, ReadError> {
    if bytes.len() < HEADER_LEN {
        return Err(ReadError::Truncated {
            need: HEADER_LEN,
            have: bytes.len(),
        });
    }
    if &bytes[..MAGIC.len()] != MAGIC {
        return Err(ReadError::NotACartridge);
    }
    if u16::from_be_bytes([bytes[0x16], bytes[0x17]]) != CARTRIDGE_EASYFLASH {
        return Err(ReadError::NotACartridge);
    }

    // The container header name is stored in the host's encoding, NOT the machine's. Applying the
    // case swap here is the mistake that turns `ndefpack 1.0.1` into `NDEFPACK 1.0.1` and breaks
    // parity on every reference image.
    let field = &bytes[NAME_OFFSET..NAME_OFFSET + NAME_LEN];
    let end = field.iter().position(|&b| b == 0).unwrap_or(NAME_LEN);
    let header_name = String::from_utf8_lossy(&field[..end]).into_owned();

    let mut image = FlashImage::erased();
    let mut p = HEADER_LEN;
    while p < bytes.len() {
        let rest = bytes.len() - p;
        if rest < CHIP_HEADER_LEN {
            return Err(ReadError::Truncated {
                need: CHIP_HEADER_LEN,
                have: rest,
            });
        }
        if &bytes[p..p + 4] != b"CHIP" {
            return Err(ReadError::BadPacket {
                at: p,
                problem: PacketProblem::Marker,
            });
        }

        let total = u32::from_be_bytes(bytes[p + 4..p + 8].try_into().expect("4 bytes")) as usize;
        let bank = u16::from_be_bytes([bytes[p + 10], bytes[p + 11]]) as usize;
        let addr = u16::from_be_bytes([bytes[p + 12], bytes[p + 13]]);
        let len = u16::from_be_bytes([bytes[p + 14], bytes[p + 15]]) as usize;

        if total != CHIP_HEADER_LEN + len {
            return Err(ReadError::BadPacket {
                at: p,
                problem: PacketProblem::LengthDisagrees,
            });
        }
        if rest < total {
            return Err(ReadError::Truncated {
                need: total,
                have: rest,
            });
        }
        if bank >= N_BANKS {
            return Err(ReadError::OutOfBounds { at: p, len });
        }

        let data = &bytes[p + CHIP_HEADER_LEN..p + total];
        // A packet covers one bank half, or — at $8000 only — both halves of one bank.
        match (addr, len) {
            (0x8000, BANK_SIZE) => image.write_at(FlashImage::offset(bank, Half::RomL), data),
            (0xa000 | 0xe000, BANK_SIZE) => {
                image.write_at(FlashImage::offset(bank, Half::RomH), data)
            }
            (0x8000, l) if l == 2 * BANK_SIZE => {
                image.write_at(FlashImage::offset(bank, Half::RomL), &data[..BANK_SIZE]);
                image.write_at(FlashImage::offset(bank, Half::RomH), &data[BANK_SIZE..]);
            }
            _ => {
                return Err(ReadError::BadPacket {
                    at: p,
                    problem: PacketProblem::Geometry,
                })
            }
        }

        p += total;
    }

    Ok(Container { image, header_name })
}

fn header(id: u16, name: &str) -> Vec<u8> {
    let mut h = vec![0u8; HEADER_LEN];
    h[..16].copy_from_slice(MAGIC);
    h[0x10..0x14].copy_from_slice(&(HEADER_LEN as u32).to_be_bytes());
    // Version 1.0.
    h[0x14] = 0x01;
    h[0x16..0x18].copy_from_slice(&id.to_be_bytes());
    // EasyFlash images are written in Ultimax mode: EXROM high, GAME low.
    h[0x18] = 1;
    h[0x19] = 0;

    let name = name.as_bytes();
    let n = name.len().min(NAME_LEN);
    h[NAME_OFFSET..NAME_OFFSET + n].copy_from_slice(&name[..n]);

    h
}

fn push_chip(out: &mut Vec<u8>, bank: u16, addr: u16, data: &[u8]) {
    let total = (data.len() + 16) as u32;
    out.extend_from_slice(b"CHIP");
    out.extend_from_slice(&total.to_be_bytes());
    out.extend_from_slice(&CHIP_TYPE_FLASH.to_be_bytes());
    out.extend_from_slice(&bank.to_be_bytes());
    out.extend_from_slice(&addr.to_be_bytes());
    out.extend_from_slice(&(data.len() as u16).to_be_bytes());
    out.extend_from_slice(data);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::BANK_SIZE;

    #[test]
    fn an_entirely_erased_image_produces_only_a_header() {
        let out = write(&FlashImage::erased(), "x");
        assert_eq!(out.len(), HEADER_LEN);
    }

    #[test]
    fn a_single_written_bank_produces_a_single_packet() {
        let mut img = FlashImage::erased();
        img.write_at(FlashImage::offset(5, Half::RomH), &[0x00; BANK_SIZE]);
        let out = write(&img, "x");
        assert_eq!(out.len(), HEADER_LEN + 16 + BANK_SIZE);
        assert_eq!(&out[HEADER_LEN..HEADER_LEN + 4], b"CHIP");
        assert_eq!(
            u32::from_be_bytes(out[HEADER_LEN + 4..HEADER_LEN + 8].try_into().unwrap()) as usize,
            16 + BANK_SIZE,
            "the length field is the whole packet, header included"
        );
        assert_eq!(&out[HEADER_LEN + 10..HEADER_LEN + 12], &[0x00, 0x05]);
        assert_eq!(&out[HEADER_LEN + 12..HEADER_LEN + 14], &[0xa0, 0x00]);
    }

    #[test]
    fn the_name_is_truncated_to_the_field_and_nul_padded() {
        let out = write(
            &FlashImage::erased(),
            "0123456789abcdefghijklmnopqrstuvwxyz",
        );
        assert_eq!(
            &out[NAME_OFFSET..NAME_OFFSET + NAME_LEN],
            b"0123456789abcdefghijklmnopqrstuv"
        );

        let out = write(&FlashImage::erased(), "short");
        assert_eq!(&out[NAME_OFFSET..NAME_OFFSET + 5], b"short");
        assert!(out[NAME_OFFSET + 5..NAME_OFFSET + NAME_LEN]
            .iter()
            .all(|&b| b == 0));
    }

    #[test]
    fn parse_inverts_write() {
        let mut img = FlashImage::erased();
        img.write_at(FlashImage::offset(3, Half::RomL), &[0xaa; BANK_SIZE]);
        img.write_at(FlashImage::offset(3, Half::RomH), &[0xbb; BANK_SIZE]);
        img.write_at(FlashImage::offset(60, Half::RomH), &[0xcc; BANK_SIZE]);

        let back = parse(&write(&img, "some tool 1.2")).expect("its own output should parse");
        assert_eq!(back.header_name, "some tool 1.2");
        assert_eq!(back.image.as_bytes(), img.as_bytes());
    }

    #[test]
    fn the_header_name_is_read_in_the_host_encoding_not_the_machines() {
        let back = parse(&write(&FlashImage::erased(), "ndefpack 1.0.1")).expect("parses");
        assert_eq!(
            back.header_name, "ndefpack 1.0.1",
            "case-swapping this field yields NDEFPACK 1.0.1 and breaks parity everywhere"
        );
    }

    #[test]
    fn a_foreign_container_is_refused() {
        let mut bytes = write(&FlashImage::erased(), "x");
        bytes[0] = b'X';
        assert!(matches!(parse(&bytes), Err(ReadError::NotACartridge)));
    }

    #[test]
    fn a_packet_spanning_both_halves_of_one_bank_is_accepted() {
        // The format permits a 16 KB packet at $8000 covering ROML then ROMH. Nothing this tool
        // writes uses that form, but images from elsewhere do.
        let mut bytes = header(CARTRIDGE_EASYFLASH, "x");
        let mut data = vec![0x11u8; BANK_SIZE];
        data.extend_from_slice(&[0x22u8; BANK_SIZE]);
        push_chip(&mut bytes, 7, 0x8000, &data);

        let back = parse(&bytes).expect("the wide form is legal at $8000");
        assert_eq!(back.image.bank(7, Half::RomL), [0x11; BANK_SIZE]);
        assert_eq!(back.image.bank(7, Half::RomH), [0x22; BANK_SIZE]);
    }

    #[test]
    fn a_wide_packet_anywhere_but_8000_is_refused() {
        let mut bytes = header(CARTRIDGE_EASYFLASH, "x");
        push_chip(&mut bytes, 7, 0xa000, &vec![0u8; 2 * BANK_SIZE]);
        assert!(matches!(
            parse(&bytes),
            Err(ReadError::BadPacket {
                problem: PacketProblem::Geometry,
                ..
            })
        ));
    }
}
