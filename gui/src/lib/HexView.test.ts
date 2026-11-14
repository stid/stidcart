// The hex window, driven through a fake host: rows come from a generated image whose three golden
// rows (the cart-name area of two.crt) are the ones the Rust side really sends.
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import { afterEach, expect, test, vi } from 'vitest';
import golden from './fixtures/hex.json';
import HexView from './HexView.svelte';
import type { HexHost, HexInfo, HexLine } from './types';

afterEach(cleanup);

const info: HexInfo = golden.info;
/** The row the golden lines start at: bank 0 ROMH `$BAF0`. */
const GOLDEN_AT = 0x3af;
const NAME_ROW = GOLDEN_AT + 1;

function line(row: number): HexLine {
  const known: HexLine | undefined = golden.lines[row - GOLDEN_AT];
  if (known) return known;
  const perHalf = 0x2000 / 16;
  const section = Math.floor(row / perHalf);
  return {
    bank: section >> 1,
    half: section % 2 === 0 ? 'L' : 'H',
    addr: (section % 2 === 0 ? 0x8000 : 0xa000) + (row % perHalf) * 16,
    bytes: Array(16).fill(0xff),
    text: '.'.repeat(16),
  };
}

function fakeHost(over: Partial<HexInfo> = {}) {
  let changed = () => {};
  let goto: (entry: number | null) => void = () => {};
  const host = {
    info: vi.fn(async (): Promise<HexInfo> => ({ ...info, selected: null, ...over })),
    lines: vi.fn(async (first: number, count: number) =>
      Array.from({ length: Math.min(count, info.lines - first) }, (_, i) => line(first + i)),
    ),
    onChanged: vi.fn((cb: () => void) => (changed = cb)),
    onGoto: vi.fn((cb: (entry: number | null) => void) => (goto = cb)),
  } satisfies HexHost;
  return { host, changed: () => changed(), goto: (entry: number | null) => goto(entry) };
}

const rows = () => [...document.querySelectorAll('[data-row]')];

test('AC-200 (RENDER): rows show bank, half, C64 address and sixteen bytes, from the top of the image', async () => {
  const { host } = fakeHost();
  render(HexView, { props: { host } });

  const first = await screen.findByText('$8000');
  const row = first.closest('[data-row]')!;
  expect(row).toHaveTextContent('00 L');
  expect(row.querySelectorAll('[data-byte]')).toHaveLength(16);
  expect(row.querySelector('[data-byte]')).toHaveTextContent('FF');
  expect(screen.getByText('$8010')).toBeInTheDocument();
  expect(screen.getByText('two.crt')).toBeInTheDocument();
  expect(screen.getByText('bank 00 ROML')).toBeInTheDocument();
});

test('AC-200 (RENDER): nothing in the window can be typed into', async () => {
  const { host } = fakeHost();
  render(HexView, { props: { host } });
  await screen.findByText('$8000');
  expect(document.querySelector('input, textarea, [contenteditable]')).toBeNull();
});

test('AC-204 (RENDER): only the rows in view are fetched and drawn', async () => {
  const { host } = fakeHost();
  render(HexView, { props: { host } });
  await screen.findByText('$8000');

  expect(host.lines).toHaveBeenCalled();
  for (const [, count] of host.lines.mock.calls) expect(count).toBeLessThan(200);
  expect(rows().length).toBeGreaterThan(10);
  expect(rows().length).toBeLessThan(200);
});

test('AC-201 (RENDER): each row shows its text column', async () => {
  const { host } = fakeHost();
  render(HexView, { props: { host } });
  await screen.findByText('$8000');

  await fireEvent.change(screen.getByRole('combobox', { name: /jump to/i }), {
    target: { value: String(info.regions.findIndex((r) => r.kind === 'name')) },
  });
  expect(await screen.findByText('EF-Name:two entr')).toBeInTheDocument();
  expect(screen.getByText('ies.............')).toBeInTheDocument();
});

test('AC-202 (RENDER): bytes are marked by owner, and pointing at one names it', async () => {
  const { host } = fakeHost();
  render(HexView, { props: { host } });
  await screen.findByText('$8000');

  // The top of the image is the menu loader.
  const loader = rows()[0].querySelector('[data-byte]')!;
  expect(loader).toHaveAttribute('data-owner', 'loader');
  await fireEvent.pointerOver(loader);
  expect(screen.getByRole('status')).toHaveTextContent('00 L $8000 · Menu loader');

  await fireEvent.change(screen.getByRole('combobox', { name: /jump to/i }), {
    target: { value: String(info.regions.findIndex((r) => r.kind === 'name')) },
  });
  const nameRow = (await screen.findByText('EF-Name:two entr')).closest('[data-row]')!;
  const bytes = [...nameRow.querySelectorAll('[data-byte]')];
  expect(bytes.every((b) => b.getAttribute('data-owner') === 'name')).toBe(true);
  // The name area ends 8 bytes into the next row, where the boot code starts.
  const next = [...nameRow.nextElementSibling!.querySelectorAll('[data-byte]')];
  expect(next.map((b) => b.getAttribute('data-owner'))).toEqual([...Array(8).fill('name'), ...Array(8).fill('boot')]);
  await fireEvent.pointerOver(next[8]);
  expect(screen.getByRole('status')).toHaveTextContent('00 H $BB18 · Boot code');
});

test('AC-202 (RENDER): an entry is marked with its kind family, and unowned bytes are plain', async () => {
  const { host } = fakeHost({ selected: 0 });
  render(HexView, { props: { host } });

  // Entry 0 (hello) starts at position 19386: row 1211, column 10.
  const row = (await screen.findByText('$8BB0')).closest('[data-row]')!;
  const bytes = [...row.querySelectorAll('[data-byte]')];
  expect(bytes[9]).toHaveAttribute('data-entry', '1');
  expect(bytes[10]).toHaveAttribute('data-entry', '0');
  expect(bytes[10]).toHaveAttribute('data-family', 'prg');
  await fireEvent.pointerOver(bytes[10]);
  expect(screen.getByRole('status')).toHaveTextContent('01 L $8BBA · 0 · hello (PRG)');

  // Seven rows on, past hello's 102 bytes, nothing owns the flash.
  const free = (await screen.findByText('$8C30')).closest('[data-row]')!;
  const plain = free.querySelector('[data-byte]')!;
  expect(plain).not.toHaveAttribute('data-owner');
  await fireEvent.pointerOver(plain);
  expect(screen.getByRole('status')).toHaveTextContent('01 L $8C30 · unused');
});

test('AC-203 (RENDER): the window opens at the selected entry and follows the selection', async () => {
  const { host, goto } = fakeHost({ selected: 0 });
  render(HexView, { props: { host } });
  expect(await screen.findByText('$8BB0')).toBeInTheDocument();
  expect(screen.queryByText('bank 00 ROML')).toBeNull();

  // Entry 1 (ramp) starts bank 1 ROML.
  host.info.mockImplementation(async () => ({ ...info, selected: 1 }));
  goto(1);
  const ramp = (await screen.findByText('$8000')).closest('[data-row]')!;
  expect(ramp).toHaveTextContent('01 L');
  expect(ramp.querySelector('[data-byte]')).toHaveAttribute('data-entry', '1');
  expect(screen.getByText('bank 01 ROML')).toBeInTheDocument();

  // Nothing selected moves nothing.
  goto(null);
  await waitFor(() => expect(host.info).toHaveBeenCalledTimes(3));
  expect(screen.getByText('bank 01 ROML')).toBeInTheDocument();
});

test('AC-204 (RENDER): the window follows the cart when it changes, keeping its place', async () => {
  const { host, changed } = fakeHost();
  render(HexView, { props: { host } });
  await screen.findByText('$8000');
  const fetched = host.lines.mock.calls.length;

  host.info.mockImplementation(async () => ({ ...info, file: 'other.crt', selected: null }));
  changed();
  expect(await screen.findByText('other.crt')).toBeInTheDocument();
  await waitFor(() => expect(host.lines.mock.calls.length).toBeGreaterThan(fetched));
  expect(screen.getByText('$8000')).toBeInTheDocument();
});

test('AC-204 (RENDER): with no cart open the window says so, and shows one when it arrives', async () => {
  const { host, changed } = fakeHost();
  host.info.mockImplementationOnce(async () => {
    throw 'no cartridge is open';
  });
  render(HexView, { props: { host } });
  expect(await screen.findByText(/no cartridge is open/i)).toBeInTheDocument();
  expect(rows()).toHaveLength(0);

  changed();
  expect(await screen.findByText('$8000')).toBeInTheDocument();
});
