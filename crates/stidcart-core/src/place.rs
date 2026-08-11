//! Where programs land in the image.
//!
//! This reproduces the reference implementation's ordering for the programs-only case, and only
//! that case. Placement is order-dependent, so a different-but-reasonable ordering yields a
//! different image that still boots and still fails byte parity — improving on it is a later,
//! deliberate divergence, not something to do by accident here.
//!
//! With no cartridge entries present there are never any gaps between them, so every program goes
//! into the contiguous free space at the top of flash, interleaved.

use crate::image::{BANK_SIZE, N_BANKS};

/// Bytes used per half, per bank. Index 0 is ROML, index 1 is ROMH.
#[derive(Clone)]
pub struct Placer {
    used: [[usize; N_BANKS]; 2],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Placement {
    pub bank: u16,
    /// Byte offset within the bank, with bit 13 set when the entry starts in ROMH.
    pub offset: u16,
}

impl Default for Placer {
    fn default() -> Self {
        Self::new()
    }
}

impl Placer {
    /// Bank 0 is fully spoken for: the menu loader in ROML, the reserved region in ROMH.
    pub fn new() -> Self {
        let mut used = [[0usize; N_BANKS]; 2];
        used[0][0] = BANK_SIZE;
        used[1][0] = BANK_SIZE;
        Placer { used }
    }

    /// The contiguous free space at the top, and where it starts.
    ///
    /// Returns `(bank, half_index, bytes)`. The half index is 1 when the run begins part-way
    /// through a bank's ROMH.
    fn top(&self) -> (usize, usize, usize) {
        let mut bank = None;
        for b in (0..N_BANKS).rev() {
            if self.used[0][b] != 0 || self.used[1][b] != 0 {
                bank = Some(b + 1);
                break;
            }
        }
        // An entirely empty image reports no top space, matching the reference implementation.
        // Unreachable here because bank 0 is always occupied.
        let Some(mut bank) = bank else {
            return (0, 0, 0);
        };

        let mut half = 0;
        let mut space = (N_BANKS - bank) * (BANK_SIZE * 2);

        let tail = self.used[1][bank - 1];
        if tail < BANK_SIZE {
            bank -= 1;
            space += BANK_SIZE - tail;
            half = 1;
            if tail == 0 && self.used[0][bank] < BANK_SIZE {
                space += BANK_SIZE - self.used[0][bank];
                half = 0;
            }
        }

        (bank, half, space)
    }

    pub fn free_space(&self) -> usize {
        self.top().2
    }

    /// Place `size` bytes interleaved in the top free space, or fail if it does not fit.
    pub fn place_interleaved(&mut self, size: usize) -> Option<Placement> {
        let (bank, half, space) = self.top();
        if size > space {
            return None;
        }

        let offset = self.used[half][bank] + half * BANK_SIZE;
        self.mark(bank, half, size);

        Some(Placement {
            bank: bank as u16,
            offset: offset as u16,
        })
    }

    fn mark(&mut self, bank: usize, half: usize, size: usize) {
        let (mut b, mut h, mut rest) = (bank, half, size);
        let mut room = BANK_SIZE - self.used[h][b];

        loop {
            let len = room.min(rest);
            self.used[h][b] += len;
            rest -= len;

            h ^= 1;
            if h == 0 {
                b += 1;
            }

            if rest == 0 || b >= N_BANKS {
                break;
            }
            room = BANK_SIZE - self.used[h][b];
        }
    }
}

/// Indices of `sizes`, largest first. Ties keep their original order, which is what the reference
/// implementation's sort happens to do for equal weights.
pub fn placement_order(sizes: &[usize]) -> Vec<usize> {
    let mut order: Vec<usize> = (0..sizes.len()).collect();
    order.sort_by(|&a, &b| sizes[b].cmp(&sizes[a]));
    order
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_first_program_starts_at_bank_1_roml() {
        let mut p = Placer::new();
        assert_eq!(
            p.place_interleaved(0x2002),
            Some(Placement { bank: 1, offset: 0 })
        );
    }

    #[test]
    fn the_next_program_resumes_where_the_last_one_stopped() {
        // Measured: 0x2002 bytes fill bank 1 ROML and two bytes of bank 1 ROMH, so the next
        // entry starts at 0x2002 — the bit-13 encoding of "two bytes into ROMH".
        let mut p = Placer::new();
        p.place_interleaved(0x2002);
        assert_eq!(
            p.place_interleaved(0x2002),
            Some(Placement {
                bank: 1,
                offset: 0x2002
            })
        );
    }

    #[test]
    fn free_space_starts_at_63_banks_worth() {
        assert_eq!(Placer::new().free_space(), 63 * BANK_SIZE * 2);
    }

    #[test]
    fn space_shrinks_by_exactly_what_was_placed() {
        let mut p = Placer::new();
        let before = p.free_space();
        p.place_interleaved(0x1234);
        assert_eq!(p.free_space(), before - 0x1234);
    }

    #[test]
    fn a_program_larger_than_the_image_does_not_fit() {
        let mut p = Placer::new();
        assert_eq!(p.place_interleaved(64 * BANK_SIZE * 2), None);
    }

    #[test]
    fn filling_flash_exactly_is_allowed_and_leaves_nothing() {
        let mut p = Placer::new();
        let all = p.free_space();
        assert!(p.place_interleaved(all).is_some());
        assert_eq!(p.free_space(), 0);
    }

    #[test]
    fn placement_order_is_largest_first_and_stable_on_ties() {
        assert_eq!(placement_order(&[100, 500, 300]), vec![1, 2, 0]);
        assert_eq!(placement_order(&[7, 7, 7]), vec![0, 1, 2]);
    }
}
