import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import { afterEach, expect, test, vi } from 'vitest';
import two from './fixtures/two.json';
import type { Document } from './types';
import Toolbar from './Toolbar.svelte';

// Vitest runs without globals here, so testing-library cannot unmount between tests by itself.
afterEach(cleanup);

function doc(overrides: Partial<Document> = {}): Document {
  return {
    file: 'two.crt',
    untitled: false,
    listing: two,
    rows: two.rows.map(() => ({ alignable: false })),
    readOnly: null,
    menuReplaceable: false,
    dirty: false,
    canUndo: false,
    canRedo: false,
    ...overrides,
  };
}

function setup(d: Document | null) {
  const props = { doc: d, onadd: vi.fn(), onopen: vi.fn(), onnewfolder: vi.fn() };
  render(Toolbar, { props });
  return props;
}

test('AC-62 (RENDER): Add Aligned… asks for files to add with 64 KB alignment', async () => {
  const { onadd } = setup(doc());
  await fireEvent.click(screen.getByRole('button', { name: /add aligned/i }));
  expect(onadd).toHaveBeenCalledExactlyOnceWith(true);
  await fireEvent.click(screen.getByRole('button', { name: /^add…$/i }));
  expect(onadd).toHaveBeenLastCalledWith(false);
});

test('AC-72 (RENDER): the title bar marks unsaved changes and names an untitled cart', () => {
  setup(doc());
  expect(screen.getByText('two.crt')).toBeInTheDocument();
  expect(screen.queryByText(/edited/i)).toBeNull();

  cleanup();
  setup(doc({ dirty: true }));
  expect(screen.getByText(/edited/i)).toBeInTheDocument();

  cleanup();
  setup(doc({ file: 'Untitled', untitled: true, dirty: true }));
  expect(screen.getByText('Untitled')).toBeInTheDocument();
});

test('a view-only cart or no cart offers nothing to add to', () => {
  setup(doc({ readOnly: 'reserved regions differ' }));
  expect(screen.queryByRole('button', { name: /add/i })).toBeNull();
  cleanup();
  setup(null);
  expect(screen.queryByRole('button', { name: /add/i })).toBeNull();
});

/** The fills of the mark's label bands, top to bottom. */
function markBands(): string[] {
  return [...screen.getByTestId('mark').querySelectorAll('[data-band]')].map((b) => b.getAttribute('fill') ?? '');
}

test('AC-122 (RENDER): the title bar carries the stidcart mark, with or without a cart open', () => {
  const raster = ['#f38ba8', '#fab387', '#f9e2af', '#a6e3a1', '#89b4fa'];
  setup(null);
  expect(markBands()).toEqual(raster);
  cleanup();
  setup(doc());
  expect(markBands()).toEqual(raster);
});

test('AC-192 (RENDER): New Folder sits beside Add… on an editable cart and asks for a folder', async () => {
  const { onnewfolder } = setup(doc());
  const buttons = screen.getAllByRole('button').map((b) => b.textContent?.trim());
  expect(buttons.indexOf('New Folder')).toBe(buttons.indexOf('Add…') - 1);
  await fireEvent.click(screen.getByRole('button', { name: /new folder/i }));
  expect(onnewfolder).toHaveBeenCalledOnce();

  cleanup();
  setup(doc({ readOnly: 'reserved region EAPI differs from the embedded firmware' }));
  expect(screen.queryByRole('button', { name: /new folder/i })).toBeNull();
});
