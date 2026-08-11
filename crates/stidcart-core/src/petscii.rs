//! ASCII <-> PETSCII for the 16-byte name fields.
//!
//! The mapping the format actually uses is a case swap and nothing more: the C64's uppercase
//! character set occupies the codes ASCII uses for lowercase. Anything outside `a-z`/`A-Z` passes
//! through untouched on the way in.
//!
//! These return owned buffers. The reference implementation returned a shared `static`, so two
//! results could not be live at once; that is not a property worth reproducing.

/// Case-swap ASCII into PETSCII. Non-alphabetic bytes pass through.
pub fn ascii_to_petscii(s: &str) -> Vec<u8> {
    s.bytes().map(swap_case).collect()
}

/// Case-swap PETSCII back into ASCII, folding the high-bit uppercase range and rejecting
/// control codes to `.` the way the reference implementation does.
pub fn petscii_to_ascii(bytes: &[u8]) -> String {
    bytes
        .iter()
        .take_while(|&&b| b != 0)
        .map(|&b| match b {
            0xc1..=0xda => (b & 0x7f) as char,
            b if b & 0x80 != 0 || b <= 0x1f => '.',
            b => swap_case(b) as char,
        })
        .collect()
}

fn swap_case(b: u8) -> u8 {
    match b {
        b'a'..=b'z' => b - b'a' + b'A',
        b'A'..=b'Z' => b - b'A' + b'a',
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowercase_ascii_becomes_uppercase_petscii() {
        // Measured against a cart built by the reference tool: "custom name" is stored
        // as the bytes "CUSTOM NAME".
        assert_eq!(ascii_to_petscii("custom name"), b"CUSTOM NAME".to_vec());
    }

    #[test]
    fn uppercase_ascii_becomes_lowercase_petscii() {
        assert_eq!(ascii_to_petscii("EF-Name:"), b"ef-nAME:".to_vec());
    }

    #[test]
    fn digits_and_punctuation_pass_through() {
        assert_eq!(ascii_to_petscii("v2.0 (x)"), b"V2.0 (X)".to_vec());
    }

    #[test]
    fn round_trips_through_petscii_and_back() {
        for s in ["custom name", "My Game v2", "(no EF-Name)", "1234"] {
            let there = ascii_to_petscii(s);
            assert_eq!(petscii_to_ascii(&there), s, "round trip failed for {s:?}");
        }
    }

    #[test]
    fn decoding_stops_at_the_first_nul() {
        assert_eq!(petscii_to_ascii(b"SAME\0\0\0\0"), "same");
    }

    #[test]
    fn high_bit_uppercase_range_folds_to_ascii_uppercase() {
        assert_eq!(petscii_to_ascii(&[0xc1, 0xc2, 0xda]), "ABZ");
    }

    #[test]
    fn control_codes_become_dots() {
        assert_eq!(petscii_to_ascii(&[0x01, b'A', 0x1f]), ".a.");
    }
}
