import { expect, test } from 'vitest';
import { hex, owners } from './hex';
import type { HexRegion } from './types';

const region = (start: number, len: number): HexRegion => ({
  start,
  len,
  kind: 'entry',
  label: '',
  family: 'prg',
  entry: 0,
});

test('AC-202 (RENDER): each byte of a row finds its region, and bytes between regions find none', () => {
  const regions = [region(0, 4), region(6, 2), region(8, 100), region(200, 16)];
  expect(owners(regions, 0, 16)).toEqual([0, 0, 0, 0, null, null, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2]);
  expect(owners(regions, 96, 16)).toEqual([2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, null, null, null, null]);
  expect(owners(regions, 192, 16)).toEqual([null, null, null, null, null, null, null, null, 3, 3, 3, 3, 3, 3, 3, 3]);
  expect(owners(regions, 216, 4)).toEqual([null, null, null, null]);
  expect(owners([], 0, 2)).toEqual([null, null]);
});

test('figures are upper-case hex, zero-padded', () => {
  expect(hex(0xa000, 4)).toBe('A000');
  expect(hex(3, 2)).toBe('03');
  expect(hex(255, 2)).toBe('FF');
});
