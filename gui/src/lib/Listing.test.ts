import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte';
import { afterEach, expect, test, vi } from 'vitest';
import two from './fixtures/two.json';
import type { Listing as ListingData } from './types';
import Listing from './Listing.svelte';

// Vitest runs without globals here, so testing-library cannot unmount between tests by itself.
afterEach(cleanup);

const listing: ListingData = two;

test('AC-51 (RENDER): every listing value is shown', () => {
  render(Listing, { props: { listing, file: 'two.crt' } });

  expect(screen.getByText('two.crt')).toBeInTheDocument();
  expect(screen.getByText(listing.cart)).toBeInTheDocument();
  expect(screen.getByText(listing.builtBy)).toBeInTheDocument();

  const rows = screen.getAllByRole('row').slice(1); // header row first
  expect(rows).toHaveLength(listing.rows.length);
  listing.rows.forEach((r, i) => {
    const cells = within(rows[i])
      .getAllByRole('cell')
      .map((c) => c.textContent?.trim());
    expect(cells).toEqual([String(r.index), r.name, r.flags, r.kind, r.bank, r.layout, r.size]);
  });

  expect(screen.getByText(/2 entries/)).toBeInTheDocument();
  expect(screen.getByText(new RegExp(listing.freeText.replace(/[$()]/g, '\\$&')))).toBeInTheDocument();
});

test('AC-85 (RENDER): the window shows which menu the cart carries', () => {
  render(Listing, { props: { listing, file: 'two.crt' } });
  expect(screen.getByText('EasyLoader')).toBeInTheDocument();
  cleanup();

  const ours = { ...listing, menu: 'stidcart' };
  render(Listing, { props: { listing: ours, file: 'ours.crt' } });
  expect(screen.getByText('stidcart')).toBeInTheDocument();
});

function editable(extra: Record<string, unknown> = {}) {
  const onedit = vi.fn();
  const onreplace = vi.fn();
  render(Listing, {
    props: {
      listing,
      file: 'two.crt',
      readOnly: null,
      rows: listing.rows.map(() => ({ alignable: false })),
      onedit,
      onreplace,
      ...extra,
    },
  });
  return { onedit, onreplace };
}

test('AC-60 (RENDER): a view-only cart says why and offers no edits', () => {
  const reason = 'reserved region EAPI differs from the embedded firmware, so it cannot be rebuilt unchanged';
  editable({ readOnly: reason });
  expect(screen.getByRole('status')).toHaveTextContent(reason);
  expect(screen.queryByRole('button', { name: /edit cart name/i })).toBeNull();
  expect(screen.queryByRole('button', { name: /actions for/i })).toBeNull();
});

test('AC-69 (RENDER): the cart name and the header name are edited separately', async () => {
  const { onedit } = editable();
  await fireEvent.click(screen.getByRole('button', { name: /edit cart name/i }));
  const cart = screen.getByRole('textbox', { name: /cart name/i });
  await fireEvent.input(cart, { target: { value: 'my menu' } });
  await fireEvent.keyDown(cart, { key: 'Enter' });
  expect(onedit).toHaveBeenLastCalledWith({ kind: 'setCartName', name: 'my menu' });

  await fireEvent.click(screen.getByRole('button', { name: /edit header name/i }));
  const header = screen.getByRole('textbox', { name: /header name/i });
  await fireEvent.input(header, { target: { value: 'hand made' } });
  await fireEvent.keyDown(header, { key: 'Enter' });
  expect(onedit).toHaveBeenLastCalledWith({ kind: 'setHeaderName', name: 'hand made' });
  expect(onedit).toHaveBeenCalledTimes(2);
});

test('AC-71 (RENDER): rows show the bank and layout they are given, after an edit re-placed them', () => {
  const replaced: ListingData = {
    ...listing,
    rows: listing.rows.map((r, i) => ({ ...r, bank: `$0${i + 2}`, layout: `$${1000 + i}` })),
  };
  editable({ listing: replaced });
  const rows = screen.getAllByRole('row').slice(1);
  replaced.rows.forEach((r, i) => {
    expect(within(rows[i]).getByText(r.bank)).toBeInTheDocument();
    expect(within(rows[i]).getByText(r.layout)).toBeInTheDocument();
  });
  expect(screen.getAllByRole('button', { name: /actions for/i })).toHaveLength(replaced.rows.length);
});

test('AC-112 (RENDER): on an editable cart the menu is a choice, and choosing the other is an edit', async () => {
  const { onedit } = editable();
  const picker = screen.getByRole('combobox', { name: /menu/i }) as HTMLSelectElement;
  expect(picker.value).toBe('EasyLoader');
  expect(within(picker).getAllByRole('option').map((o) => o.textContent)).toEqual(['EasyLoader', 'stidcart']);

  await fireEvent.change(picker, { target: { value: 'stidcart' } });
  expect(onedit).toHaveBeenLastCalledWith({ kind: 'setMenu', menu: 'stidcart' });
  expect(onedit).toHaveBeenCalledTimes(1);
});

test('AC-113 (RENDER): until the cart answers with a new menu, the choice shows the one it has', async () => {
  // A refused switch leaves the listing as it was, so the picker must not keep showing the refused menu.
  editable();
  const picker = screen.getByRole('combobox', { name: /menu/i }) as HTMLSelectElement;
  await fireEvent.change(picker, { target: { value: 'stidcart' } });
  expect(picker.value).toBe('EasyLoader');
});

test('AC-114 (RENDER): a view-only cart, or a listing without edits, shows the menu as text', () => {
  editable({ readOnly: 'reserved region EAPI differs from the embedded firmware' });
  expect(screen.queryByRole('combobox')).toBeNull();
  expect(screen.getByText('EasyLoader')).toBeInTheDocument();
  cleanup();

  render(Listing, { props: { listing, file: 'two.crt' } });
  expect(screen.queryByRole('combobox')).toBeNull();
  expect(screen.getByText('EasyLoader')).toBeInTheDocument();
});

test('AC-130 (RENDER): an older build of our menu shows as such, and choosing stidcart is an edit', async () => {
  const { onedit } = editable({ listing: { ...listing, menu: 'stidcart (older build)' } });
  const picker = screen.getByRole('combobox', { name: /menu/i }) as HTMLSelectElement;
  expect(picker.value).toBe('stidcart (older build)');
  const options = within(picker).getAllByRole('option') as HTMLOptionElement[];
  expect(options.map((o) => o.textContent)).toEqual(['EasyLoader', 'stidcart', 'stidcart (older build)']);
  expect(options[2].disabled).toBe(true);

  await fireEvent.change(picker, { target: { value: 'stidcart' } });
  expect(onedit).toHaveBeenLastCalledWith({ kind: 'setMenu', menu: 'stidcart' });
});

test('AC-134 (RENDER): a view-only cart whose loader is unknown offers the menu picker', async () => {
  const { onedit } = editable({
    readOnly: 'reserved region menu loader differs from the embedded firmware',
    menuReplaceable: true,
    listing: { ...listing, menu: 'unrecognised' },
  });
  expect(screen.getByText(/view only/i)).toBeInTheDocument();
  expect(screen.getByText(/choose a menu/i)).toBeInTheDocument();
  const picker = screen.getByRole('combobox', { name: /menu/i }) as HTMLSelectElement;
  expect(picker.value).toBe('unrecognised');
  const options = within(picker).getAllByRole('option') as HTMLOptionElement[];
  expect(options.map((o) => o.textContent)).toEqual(['EasyLoader', 'stidcart', 'unrecognised']);
  expect(options[2].disabled).toBe(true);

  await fireEvent.change(picker, { target: { value: 'EasyLoader' } });
  expect(onedit).toHaveBeenLastCalledWith({ kind: 'setMenu', menu: 'EasyLoader' });
});

test('AC-135 (RENDER): a cart refused for another reason offers no menu picker', () => {
  editable({ readOnly: 'reserved region EAPI differs from the embedded firmware', menuReplaceable: false });
  expect(screen.queryByRole('combobox')).toBeNull();
  expect(screen.queryByText(/choose a menu/i)).toBeNull();
});

test('AC-117 (RENDER): every size and the free figure show their exact byte count on hover', () => {
  render(Listing, { props: { listing, file: 'two.crt' } });
  const exact = (n: number) => `${n.toLocaleString('en-US')} bytes`;

  const rows = screen.getAllByRole('row').slice(1);
  listing.rows.forEach((r, i) => {
    expect(within(rows[i]).getByText(r.size)).toHaveAttribute('title', exact(r.sizeBytes));
  });
  expect(screen.getByText(listing.freeText)).toHaveAttribute('title', exact(listing.free));
});

test('AC-118 (RENDER): each type badge carries its family', () => {
  const families = ['prg', '8k', '16k', 'ult', 'xb'];
  const rows = listing.rows.map((r, i) => ({ ...r, family: families[i % families.length] }));
  render(Listing, { props: { listing: { ...listing, rows }, file: 'two.crt' } });
  const table = screen.getAllByRole('row').slice(1);
  rows.forEach((r, i) => {
    expect(within(table[i]).getByText(r.kind)).toHaveAttribute('data-family', r.family);
  });
});
