//! Where entries land in the image.
//!
//! This reproduces the reference implementation's placement (`place.c`), which is a greedy
//! first-fit over a weight-sorted list, not an optimiser. Placement is order-dependent, so a
//! different-but-reasonable ordering yields a different image that still boots and still fails
//! byte parity — improving on it is a later, deliberate divergence, not something to do by accident
//! here.
//!
//! Cartridges go first, by weight: Ocean (which must own bank 0), then 64 KB-aligned ones, then
//! the rest by how many banks tall they are. Each takes the first run of banks whose needed halves
//! are untouched. Programs come last, largest first: one that fits a gap left between cartridges
//! is confined to that half; otherwise it goes into the contiguous free space at the top of flash,
//! interleaved. With no cartridges there are no gaps, and this collapses to the programs-only path
//! the original fixtures pin.

use crate::directory::{DirEntry, EntryType};
use crate::image::{BANK_SIZE, N_BANKS};
use crate::reserved::Mode;
use crate::{BuildError, Entry};

/// Bytes used per half, per bank. Index 0 is ROML, index 1 is ROMH.
#[derive(Clone)]
pub struct Placer {
    used: [[usize; N_BANKS]; 2],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Placement {
    pub bank: u16,
    /// Programs: byte offset within the bank, with bit 13 set when the entry starts in ROMH.
    /// Cartridges: always 0.
    pub offset: u16,
    /// The kind the placement decided: an interleaved program placed at the top stays `Prg`, one
    /// placed in a gap becomes `PrgRomL` or `PrgRomH`; a cartridge keeps its kind.
    pub kind: EntryType,
}

impl Default for Placer {
    fn default() -> Self {
        Self::new(Mode::Normal)
    }
}

impl Placer {
    /// Bank 0 ROMH is always spoken for (directory, EAPI, name, boot). The menu loader takes bank 0
    /// ROML in normal mode and bank 1 ROMH in Ocean mode, where an Ocean cartridge owns bank 0
    /// ROML (`place.c:670-685`).
    pub fn new(mode: Mode) -> Self {
        let mut used = [[0usize; N_BANKS]; 2];
        used[1][0] = BANK_SIZE;
        match mode {
            Mode::Normal => used[0][0] = BANK_SIZE,
            Mode::Ocean => used[1][1] = BANK_SIZE,
        }
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
        // A used-up top starts past the last bank, where even an empty program has no position
        // to record (D-028).
        if size > space || bank == N_BANKS {
            return None;
        }

        let offset = self.used[half][bank] + half * BANK_SIZE;
        self.mark(bank, half, size);

        Some(Placement {
            bank: bank as u16,
            offset: offset as u16,
            kind: EntryType::Prg,
        })
    }

    /// Place `size` bytes in one half from the first free byte of `bank`, continuing in the same
    /// half of the following banks (`place.c:299-323`, toggle off).
    fn place_confined(&mut self, bank: usize, half: usize, size: usize) -> Placement {
        let offset = self.used[half][bank] + half * BANK_SIZE;
        let (mut b, mut rest) = (bank, size);
        while rest > 0 && b < N_BANKS {
            let len = (BANK_SIZE - self.used[half][b]).min(rest);
            self.used[half][b] += len;
            rest -= len;
            b += 1;
        }
        Placement {
            bank: bank as u16,
            offset: offset as u16,
            kind: if half == 1 {
                EntryType::PrgRomH
            } else {
                EntryType::PrgRomL
            },
        }
    }

    /// The first start bank from which a cartridge's two runs fit into untouched banks, stepping
    /// `step` banks at a time (`place.c:174-207`). Offsets are relative to the start bank.
    fn find_free_banks(
        &self,
        l_off: usize,
        l_num: usize,
        h_off: usize,
        h_num: usize,
        step: usize,
    ) -> Option<usize> {
        let bank_top = (l_off + l_num).max(h_off + h_num);
        let mut i = 0;
        while i + bank_top <= N_BANKS {
            let mut empty = true;
            for j in 0..bank_top {
                let l_hit = j >= l_off && j < l_off + l_num && self.used[0][i + j] != 0;
                let h_hit = j >= h_off && j < h_off + h_num && self.used[1][i + j] != 0;
                if l_hit || h_hit {
                    empty = false;
                    if step == 1 {
                        i += j;
                    }
                    break;
                }
            }
            if empty {
                return Some(i);
            }
            i += step;
        }
        None
    }

    fn mark_banks(&mut self, bank: usize, l_off: usize, l_num: usize, h_off: usize, h_num: usize) {
        for b in bank + l_off..bank + l_off + l_num {
            self.used[0][b] = BANK_SIZE;
        }
        for b in bank + h_off..bank + h_off + h_num {
            self.used[1][b] = BANK_SIZE;
        }
    }

    /// The next gap below `top_bank` holding at least `min_size` bytes, scanning ROML then ROMH
    /// from the cursor `(bank, half)` and leaving the cursor on the gap found
    /// (`place.c:209-250`). The cursor persists across calls, so a later, smaller program never
    /// goes back to a gap an earlier one was placed in.
    fn find_free_gap(
        &self,
        cursor: &mut (usize, usize),
        top_bank: usize,
        min_size: usize,
    ) -> usize {
        let (mut bank, mut half) = *cursor;
        if bank >= top_bank {
            bank = 0;
            half += 1;
        }
        if half > 1 {
            return 0;
        }
        while half < 2 {
            for i in bank..top_bank {
                let mut size = BANK_SIZE - self.used[half][i];
                if size == 0 {
                    continue;
                }
                let mut j = 1;
                while i + j < top_bank && self.used[half][i + j] == 0 {
                    size += BANK_SIZE;
                    j += 1;
                }
                if size >= min_size {
                    *cursor = (i, half);
                    return size;
                }
            }
            bank = 0;
            half += 1;
        }
        *cursor = (top_bank, 1);
        0
    }

    /// Place every entry, returning one placement per entry in input order.
    ///
    /// Reproduces `place_entries` (`place.c:577-615`): weights, a stable descending sort, every
    /// cartridge in that order, then `place_all_prgs` (`place.c:434-476`) — each round finds the
    /// first gap that fits the *smallest* unplaced program and puts the *largest* unplaced program
    /// that fits into it, or, when no gap fits, sends the largest to the top interleaved.
    pub fn place_all(&mut self, entries: &[Entry]) -> Result<Vec<Placement>, BuildError> {
        // Cartridges first whatever the weights say: a program over 64 KiB outweighs a one-bank
        // cart, and the reference would then hand it to its cartridge placer (D-025).
        let weights: Vec<(bool, u64)> = entries
            .iter()
            .map(|e| (matches!(e, Entry::Cartridge(_)), weight(e)))
            .collect();
        let mut order: Vec<usize> = (0..entries.len()).collect();
        order.sort_by(|&a, &b| weights[b].cmp(&weights[a]));

        let mut placements: Vec<Option<Placement>> = vec![None; entries.len()];
        let n_carts = entries
            .iter()
            .filter(|e| matches!(e, Entry::Cartridge(_)))
            .count();

        for &i in &order[..n_carts] {
            let Entry::Cartridge(c) = &entries[i] else {
                unreachable!("every cartridge outweighs every program");
            };
            let (l, h) = c.banks();
            let step = if c.align_64k { 0x10000 / BANK_SIZE } else { 1 };
            let free = self.free_space();
            let bank = self
                .find_free_banks(c.roml_at as usize, l, c.romh_at as usize, h, step)
                .ok_or_else(|| BuildError::OutOfSpace {
                    name: c.name.clone(),
                    size: c.size() as usize,
                    free,
                })?;
            self.mark_banks(bank, c.roml_at as usize, l, c.romh_at as usize, h);
            placements[i] = Some(Placement {
                bank: bank as u16,
                offset: 0,
                kind: c.kind,
            });
        }

        // Programs. The top bank is computed once, before any program lands, as the reference
        // does: gaps are only ever searched below it.
        let programs = &order[n_carts..];
        let (top_bank, _, _) = self.top();
        let size_of = |i: usize| match &entries[i] {
            Entry::Program(p) => p.data.len(),
            Entry::Cartridge(_) => unreachable!(),
        };
        let mut cursor = (0usize, 0usize);
        let (mut low, mut high) = (0, programs.len());
        while low < high {
            let smallest = size_of(programs[high - 1]);
            let gap = self.find_free_gap(&mut cursor, top_bank, smallest);
            if gap == 0 {
                let i = programs[low];
                let size = size_of(i);
                let free = self.free_space();
                placements[i] =
                    Some(
                        self.place_interleaved(size)
                            .ok_or_else(|| BuildError::OutOfSpace {
                                name: entries[i].name().to_string(),
                                size,
                                free,
                            })?,
                    );
            } else {
                let j = programs[low..high]
                    .iter()
                    .position(|&i| placements[i].is_none() && size_of(i) <= gap)
                    .expect("the gap was sized to fit at least the smallest unplaced program");
                let i = programs[low + j];
                placements[i] = Some(self.place_confined(cursor.0, cursor.1, size_of(i)));
            }
            while low < high && placements[programs[low]].is_some() {
                low += 1;
            }
            while high > low && placements[programs[high - 1]].is_some() {
                high -= 1;
            }
        }

        Ok(placements
            .into_iter()
            .map(|p| p.expect("every entry was placed above"))
            .collect())
    }

    /// Record an entry that already exists at a known position, rather than choosing one.
    ///
    /// [`Placer::mark`] fills from each half's current watermark, which is right when this placer
    /// chose the position itself. An entry read out of a file may start above the watermark — its
    /// image was laid out by something else, or has gaps — so each half is raised to the entry's
    /// own end instead.
    pub fn occupy(&mut self, entry: &DirEntry) {
        if entry.kind.is_cartridge() {
            let (primary_num, other_off, other_num) = (
                entry.primary_num() as usize,
                entry.other_off as usize,
                entry.other_num as usize,
            );
            let (l_off, l_num, h_off, h_num) = match entry.kind.primary() {
                crate::image::Half::RomL => (0, primary_num, other_off, other_num),
                crate::image::Half::RomH => (other_off, other_num, 0, primary_num),
            };
            self.mark_banks(entry.bank as usize, l_off, l_num, h_off, h_num);
            return;
        }

        let mut b = entry.bank as usize;
        let mut h = usize::from(entry.offset & 0x2000 != 0);
        let mut pos = (entry.offset & 0x1fff) as usize;
        let mut rest = entry.size as usize;
        let interleaved = entry.kind == EntryType::Prg;

        while rest > 0 && b < N_BANKS {
            let len = (BANK_SIZE - pos).min(rest);
            self.used[h][b] = self.used[h][b].max(pos + len);
            rest -= len;
            pos = 0;

            if interleaved {
                h ^= 1;
                if h == 0 {
                    b += 1;
                }
            } else {
                b += 1;
            }
        }
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

/// Larger is placed earlier (`place.c:504-543`). Ocean carts have the strictest constraint (bank
/// 0), 64 KB-aligned carts follow, then carts by height with the opposite half's count as a
/// tie-break, and programs last by size — they can be split across banks and are the most
/// flexible.
fn weight(entry: &Entry) -> u64 {
    match entry {
        Entry::Program(p) => p.data.len() as u64,
        Entry::Cartridge(c) if c.kind.is_ocean() => 1 << 30,
        Entry::Cartridge(c) => {
            let ((_, primary_num), (other_off, other_num)) = c.primary_and_other();
            let bank_top = primary_num.max(other_off as usize + other_num) as u64;
            let mut w = (1 << 16) * bank_top + other_num as u64;
            if c.align_64k {
                w += 1 << 28;
            }
            w
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
        let mut p = Placer::new(Mode::Normal);
        assert_eq!(
            p.place_interleaved(0x2002),
            Some(Placement {
                bank: 1,
                offset: 0,
                kind: EntryType::Prg
            })
        );
    }

    #[test]
    fn the_next_program_resumes_where_the_last_one_stopped() {
        // Measured: 0x2002 bytes fill bank 1 ROML and two bytes of bank 1 ROMH, so the next
        // entry starts at 0x2002 — the bit-13 encoding of "two bytes into ROMH".
        let mut p = Placer::new(Mode::Normal);
        p.place_interleaved(0x2002);
        assert_eq!(
            p.place_interleaved(0x2002),
            Some(Placement {
                bank: 1,
                offset: 0x2002,
                kind: EntryType::Prg
            })
        );
    }

    #[test]
    fn free_space_starts_at_63_banks_worth() {
        assert_eq!(Placer::new(Mode::Normal).free_space(), 63 * BANK_SIZE * 2);
    }

    #[test]
    fn ocean_mode_reserves_bank_1_romh_instead_of_bank_0_roml() {
        let p = Placer::new(Mode::Ocean);
        assert_eq!(p.used[0][0], 0, "bank 0 ROML is the Ocean cartridge's");
        assert_eq!(p.used[1][0], BANK_SIZE);
        assert_eq!(p.used[1][1], BANK_SIZE, "the loader moved to bank 1 ROMH");
        assert_eq!(p.free_space(), 62 * BANK_SIZE * 2);
    }

    #[test]
    fn space_shrinks_by_exactly_what_was_placed() {
        let mut p = Placer::new(Mode::Normal);
        let before = p.free_space();
        p.place_interleaved(0x1234);
        assert_eq!(p.free_space(), before - 0x1234);
    }

    #[test]
    fn a_program_larger_than_the_image_does_not_fit() {
        let mut p = Placer::new(Mode::Normal);
        assert_eq!(p.place_interleaved(64 * BANK_SIZE * 2), None);
    }

    #[test]
    fn filling_flash_exactly_is_allowed_and_leaves_nothing() {
        let mut p = Placer::new(Mode::Normal);
        let all = p.free_space();
        assert!(p.place_interleaved(all).is_some());
        assert_eq!(p.free_space(), 0);
    }

    #[test]
    fn nothing_more_is_placed_once_the_top_is_used_up_not_even_an_empty_program() {
        // The run then starts past the last bank; indexing it panicked (D-028).
        let mut p = Placer::new(Mode::Normal);
        let all = p.free_space();
        p.place_interleaved(all);
        assert_eq!(p.place_interleaved(0), None);
    }

    #[test]
    fn placement_order_is_largest_first_and_stable_on_ties() {
        assert_eq!(placement_order(&[100, 500, 300]), vec![1, 2, 0]);
        assert_eq!(placement_order(&[7, 7, 7]), vec![0, 1, 2]);
    }

    #[test]
    fn free_banks_are_found_first_fit_and_aligned_ones_step_by_eight() {
        let p = Placer::new(Mode::Normal);
        // Bank 0 is taken in both halves, so a 16k-tall cart lands at bank 1...
        assert_eq!(p.find_free_banks(0, 1, 0, 1, 1), Some(1));
        // ...and an aligned one skips to bank 8.
        assert_eq!(p.find_free_banks(0, 1, 0, 1, 8), Some(8));
        // A ROMH-only cart (Ultimax 8k) fits at bank 1 too, and a ROML-only one at bank 1.
        assert_eq!(p.find_free_banks(0, 0, 0, 1, 1), Some(1));
        // 64 banks in both halves cannot fit beside the reserved bank.
        assert_eq!(p.find_free_banks(0, 64, 0, 64, 1), None);
    }

    #[test]
    fn a_gap_is_only_searched_below_the_top_bank_and_the_cursor_persists() {
        let mut p = Placer::new(Mode::Normal);
        // An Ultimax 8k at bank 1 ROMH: bank 1 ROML becomes a gap below top bank 2.
        p.mark_banks(1, 0, 0, 0, 1);
        let (top_bank, _, _) = p.top();
        assert_eq!(top_bank, 2);
        let mut cursor = (0, 0);
        assert_eq!(p.find_free_gap(&mut cursor, top_bank, 100), BANK_SIZE);
        assert_eq!(cursor, (1, 0));
        p.place_confined(1, 0, 100);
        // The rest of that bank is still a gap, and the cursor stays there.
        assert_eq!(p.find_free_gap(&mut cursor, top_bank, 100), BANK_SIZE - 100);
        // Nothing below the top bank fits a whole bank now.
        assert_eq!(p.find_free_gap(&mut cursor, top_bank, BANK_SIZE), 0);
    }
}
