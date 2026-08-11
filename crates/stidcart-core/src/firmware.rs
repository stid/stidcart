//! The embedded C64-side firmware.
//!
//! These blobs are zlib licensed and are data, not code — see `firmware/README.md`. Each is a
//! `.prg`: two bytes of little-endian load address followed by the payload.

pub const EAPI: &[u8] = include_bytes!("../../../firmware/eapi_am29f040.prg");
pub const LOADER_NRM: &[u8] = include_bytes!("../../../firmware/easyloader_nrm.prg");
pub const LOADER_OCM: &[u8] = include_bytes!("../../../firmware/easyloader_ocm.prg");
pub const LAUNCHER_NRM: &[u8] = include_bytes!("../../../firmware/easyloader_launcher_nrm.prg");
pub const LAUNCHER_OCM: &[u8] = include_bytes!("../../../firmware/easyloader_launcher_ocm.prg");

/// The load address a `.prg` declares in its first two bytes.
pub fn load_addr(prg: &[u8]) -> u16 {
    u16::from_le_bytes([prg[0], prg[1]])
}

/// The payload, with the load address stripped.
pub fn body(prg: &[u8]) -> &[u8] {
    &prg[2..]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blob_sizes_are_what_the_layout_expects() {
        assert_eq!(body(EAPI).len(), 0x300, "EAPI payload");
        assert_eq!(body(LOADER_NRM).len(), 0x2000, "menu loader payload");
        assert_eq!(body(LOADER_OCM).len(), 0x2000, "ocean menu loader payload");
    }

    #[test]
    fn the_launcher_is_much_smaller_than_the_boot_region() {
        // 86 bytes into a 0x4E8 region: the rest stays erased. Assuming otherwise leaves ~1100
        // bytes of the image wrong.
        assert_eq!(body(LAUNCHER_NRM).len(), 86);
        assert!(body(LAUNCHER_NRM).len() < crate::reserved::BOOT_SIZE);
    }

    #[test]
    fn the_launcher_loads_at_the_top_of_memory() {
        let addr = load_addr(LAUNCHER_NRM);
        assert_eq!(addr, 0xffaa);
        // It ends exactly at the top, because the 6502 vectors are its last six bytes.
        assert_eq!(addr as usize + body(LAUNCHER_NRM).len(), 0x10000);
    }

    #[test]
    fn eapi_carries_its_signature() {
        assert_eq!(&body(EAPI)[..4], b"eapi");
    }
}
