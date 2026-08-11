//! The `.crt` container: a 64-byte header followed by CHIP packets.

use crate::image::{FlashImage, Half, N_BANKS};

pub const HEADER_LEN: usize = 0x40;
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
}
