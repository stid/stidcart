import { render, screen, within } from '@testing-library/svelte';
import { expect, test } from 'vitest';
import two from './fixtures/two.json';
import type { Listing as ListingData } from './types';
import Listing from './Listing.svelte';

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
