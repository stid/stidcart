//! Programs and cartridges whose only job is to show that they ran.
//!
//! Each one writes a known string to the top row of screen RAM and then spins on a `JMP` to
//! itself at a fixed address. A boot test breaks on that address and reads the string back, so it
//! needs no instruction count and cannot be fooled by a screen that happens to hold the text.
//! They are generated here, as a handful of hand-encoded bytes each, so the boot tests don't need
//! the assembler.

use super::vice::screen_text;

/// A generated input file and how to recognise that it ran.
pub struct Marker {
    /// The file's bytes: a `.prg` (load address first) or a `.crt`.
    pub bytes: Vec<u8>,
    /// The address of the `JMP` to itself the code ends in.
    pub spin: u16,
    /// What it writes to the top row of the screen.
    pub text: String,
}

/// The marker routine assembled at `base`: copy `text` to `$0400`, then spin. Returns the code and
/// the spin address.
///
/// ```text
/// base+0   A2 00        ldx #0
/// base+2   BD lo hi     lda msg,x
/// base+5   F0 06        beq spin
/// base+7   9D 00 04     sta $0400,x
/// base+10  E8           inx
/// base+11  D0 F5        bne base+2
/// base+13  4C lo hi     spin: jmp spin
/// base+16  msg…, 0
/// ```
fn routine(base: u16, text: &str) -> (Vec<u8>, u16) {
    let codes = screen_text(text);
    assert!(
        !codes.contains(&0),
        "'@' is screen code 0, which ends the message"
    );
    let msg = base + 16;
    let spin = base + 13;
    let mut code = vec![0xa2, 0x00, 0xbd, msg as u8, (msg >> 8) as u8, 0xf0, 0x06];
    code.extend([0x9d, 0x00, 0x04, 0xe8, 0xd0, 0xf5]);
    code.extend([0x4c, spin as u8, (spin >> 8) as u8]);
    code.extend(codes);
    code.push(0);
    (code, spin)
}

/// A program loading at `$07F0`, below the start of BASIC, holding a `10 SYS 2061` line at `$0801`
/// and the routine at `$080D`. It only runs if the menu starts it with `RUN`.
pub fn basic_prg(text: &str) -> Marker {
    let mut bytes = 0x07f0u16.to_le_bytes().to_vec();
    bytes.extend([0x00; 0x0801 - 0x07f0]); // $07F0..$0800, $0800 must be 0 for BASIC
    bytes.extend([0x0b, 0x08, 0x0a, 0x00, 0x9e]); // next line $080B, line 10, SYS
    bytes.extend(b"2061");
    bytes.extend([0x00, 0x00, 0x00]); // end of line, end of program
    assert_eq!(0x07f0 + bytes.len() - 2, 0x080d);
    let (code, spin) = routine(0x080d, text);
    bytes.extend(code);
    Marker {
        bytes,
        spin,
        text: text.to_string(),
    }
}

/// A program loading at `$C000`, above BASIC, which the menu must start with `SYS 49152`.
pub fn high_prg(text: &str) -> Marker {
    let mut bytes = 0xc000u16.to_le_bytes().to_vec();
    let (code, spin) = routine(0xc000, text);
    bytes.extend(code);
    Marker {
        bytes,
        spin,
        text: text.to_string(),
    }
}

/// The seven cartridge kinds the menu starts in this slice — every kind but Ocean.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CartKind {
    K8,
    K16,
    Ultimax8,
    Ultimax16,
    Xbank8,
    Xbank16,
    XbankUltimax,
}

impl CartKind {
    pub const ALL: [CartKind; 7] = [
        CartKind::K8,
        CartKind::K16,
        CartKind::Ultimax8,
        CartKind::Ultimax16,
        CartKind::Xbank8,
        CartKind::Xbank16,
        CartKind::XbankUltimax,
    ];

    pub fn name(self) -> &'static str {
        match self {
            CartKind::K8 => "k8",
            CartKind::K16 => "k16",
            CartKind::Ultimax8 => "ult8",
            CartKind::Ultimax16 => "ult16",
            CartKind::Xbank8 => "xb8",
            CartKind::Xbank16 => "xb16",
            CartKind::XbankUltimax => "xbult",
        }
    }

    fn ultimax(self) -> bool {
        matches!(
            self,
            CartKind::Ultimax8 | CartKind::Ultimax16 | CartKind::XbankUltimax
        )
    }
}

const BANK: usize = 0x2000;

/// An 8 KiB ROM half started through the `CBM80` signature: cold and warm vectors at `$8000`, the
/// signature at `$8004`, the routine at `$8009`.
fn cbm80_rom(text: &str) -> (Vec<u8>, u16) {
    let mut rom = vec![0xffu8; BANK];
    rom[0..4].copy_from_slice(&[0x09, 0x80, 0x09, 0x80]);
    rom[4..9].copy_from_slice(&[0xc3, 0xc2, 0xcd, 0x38, 0x30]);
    let (code, spin) = routine(0x8009, text);
    rom[9..9 + code.len()].copy_from_slice(&code);
    (rom, spin)
}

/// An 8 KiB ROM half at `$E000` started through the reset vector, as Ultimax carts are. NMI and
/// IRQ point at an `RTI` so a stray interrupt cannot run off the end.
fn ultimax_rom(text: &str) -> (Vec<u8>, u16) {
    let mut rom = vec![0xffu8; BANK];
    let (code, spin) = routine(0xe000, text);
    rom[..code.len()].copy_from_slice(&code);
    let rti = 0xfff0u16;
    rom[0x1ff0] = 0x40;
    for (at, vector) in [(0x1ffa, rti), (0x1ffc, 0xe000), (0x1ffe, rti)] {
        rom[at..at + 2].copy_from_slice(&vector.to_le_bytes());
    }
    (rom, spin)
}

fn crt_header(hw: u16, exrom: u8, game: u8, name: &str) -> Vec<u8> {
    let mut h = vec![0u8; 0x40];
    h[0..16].copy_from_slice(b"C64 CARTRIDGE   ");
    h[0x10..0x14].copy_from_slice(&0x40u32.to_be_bytes());
    h[0x14..0x16].copy_from_slice(&0x0100u16.to_be_bytes());
    h[0x16..0x18].copy_from_slice(&hw.to_be_bytes());
    h[0x18] = exrom;
    h[0x19] = game;
    h[0x20..0x20 + name.len()].copy_from_slice(name.as_bytes());
    h
}

fn chip(chip_type: u16, bank: u16, addr: u16, data: &[u8]) -> Vec<u8> {
    let mut p = b"CHIP".to_vec();
    p.extend((0x10 + data.len() as u32).to_be_bytes());
    p.extend(chip_type.to_be_bytes());
    p.extend(bank.to_be_bytes());
    p.extend(addr.to_be_bytes());
    p.extend((data.len() as u16).to_be_bytes());
    p.extend(data);
    p
}

/// A marker cartridge of `kind`, as the `.crt` a user would add. Generic kinds use CRT hardware
/// type 0 and ROM chips; xbank kinds use type 33 (EasyFlash xbank) and flash chips, as
/// `fixtures/make_inputs.py` writes them.
pub fn cart(kind: CartKind, text: &str) -> Marker {
    let (hw, chip_type) = match kind {
        CartKind::K8 | CartKind::K16 | CartKind::Ultimax8 | CartKind::Ultimax16 => (0, 0),
        _ => (33, 2),
    };
    // EXROM/GAME as the header records them: 8K 0/1, 16K 0/0, Ultimax 1/0.
    let (exrom, game) = match kind {
        CartKind::K8 | CartKind::Xbank8 => (0, 1),
        CartKind::K16 | CartKind::Xbank16 => (0, 0),
        _ => (1, 0),
    };
    let mut bytes = crt_header(hw, exrom, game, kind.name());
    let filler = vec![0xffu8; BANK];
    let spin = if kind.ultimax() {
        let (rom, spin) = ultimax_rom(text);
        if kind != CartKind::Ultimax8 {
            bytes.extend(chip(chip_type, 0, 0x8000, &filler));
        }
        bytes.extend(chip(chip_type, 0, 0xe000, &rom));
        spin
    } else {
        let (mut rom, spin) = cbm80_rom(text);
        match kind {
            // A generic 16K image is one 16 KiB chip at $8000, as `in_16k.crt` is.
            CartKind::K16 => {
                rom.extend(&filler);
                bytes.extend(chip(chip_type, 0, 0x8000, &rom));
            }
            CartKind::Xbank16 => {
                bytes.extend(chip(chip_type, 0, 0x8000, &rom));
                bytes.extend(chip(chip_type, 0, 0xa000, &filler));
            }
            _ => bytes.extend(chip(chip_type, 0, 0x8000, &rom)),
        }
        spin
    };
    Marker {
        bytes,
        spin,
        text: text.to_string(),
    }
}

#[cfg(test)]
mod unit {
    use super::*;
    use stidcart_core::directory::EntryType;
    use stidcart_core::Entry;

    #[test]
    fn every_marker_cart_reads_back_as_its_kind() {
        let expected = [
            EntryType::K8,
            EntryType::K16,
            EntryType::Ultimax8,
            EntryType::Ultimax16,
            EntryType::Xbank8,
            EntryType::Xbank16,
            EntryType::XbankUltimax,
        ];
        for (kind, want) in CartKind::ALL.into_iter().zip(expected) {
            let m = cart(kind, "MARKER");
            let entry = Entry::from_input(kind.name(), m.bytes, false)
                .unwrap_or_else(|e| panic!("{kind:?} was not accepted: {e}"));
            let c = entry.as_cartridge().expect("read back as a program");
            assert_eq!(c.kind, want, "{kind:?}");
        }
    }

    #[test]
    fn the_basic_marker_puts_its_code_where_the_sys_line_says() {
        let m = basic_prg("LOW");
        assert_eq!(&m.bytes[..2], &[0xf0, 0x07]);
        // $080D is file offset 2 + ($080D - $07F0).
        assert_eq!(m.bytes[2 + 0x1d], 0xa2);
        assert_eq!(m.spin, 0x080d + 13);
    }
}
