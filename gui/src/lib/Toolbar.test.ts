import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import { expect, test, vi } from 'vitest';
import two from './fixtures/two.json';
import type { Document } from './types';
import Toolbar from './Toolbar.svelte';

function doc(overrides: Partial<Document> = {}): Document {
  return {
    file: 'two.crt',
    untitled: false,
    listing: two,
    rows: two.rows.map(() => ({ alignable: false })),
    readOnly: null,
    dirty: false,
    canUndo: false,
    canRedo: false,
    ...overrides,
  };
}

function setup(d: Document | null) {
  const props = { doc: d, onadd: vi.fn(), onopen: vi.fn() };
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
