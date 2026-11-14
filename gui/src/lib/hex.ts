// Pure helpers for the hex view: who owns a row's bytes, and how figures are written.
import type { HexRegion } from './types';

/** For each of `count` bytes from position `start`, the index of the region owning it, or null.
 * `regions` is ordered by position and never overlaps, as the Rust side sends it. */
export function owners(regions: HexRegion[], start: number, count: number): (number | null)[] {
  // The first region ending after `start`.
  let lo = 0;
  let hi = regions.length;
  while (lo < hi) {
    const mid = (lo + hi) >> 1;
    if (regions[mid].start + regions[mid].len > start) hi = mid;
    else lo = mid + 1;
  }
  const out: (number | null)[] = [];
  for (let pos = start; pos < start + count; pos++) {
    while (lo < regions.length && regions[lo].start + regions[lo].len <= pos) lo++;
    out.push(lo < regions.length && regions[lo].start <= pos ? lo : null);
  }
  return out;
}

/** A figure in upper-case hex, zero-padded to `digits`. */
export function hex(value: number, digits: number): string {
  return value.toString(16).toUpperCase().padStart(digits, '0');
}
