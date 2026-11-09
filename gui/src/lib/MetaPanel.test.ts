import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import { afterEach, expect, test, vi } from 'vitest';
import two from './fixtures/two.json';
import type { Row } from './types';
import MetaPanel from './MetaPanel.svelte';

// Vitest runs without globals here, so testing-library cannot unmount between tests by itself.
afterEach(cleanup);

const row: Row = {
  ...two.rows[1],
  title: 'Old Title',
  year: '',
  author: '',
  note: '',
  folder: 'Games',
};

async function commit(label: string, value: string) {
  await fireEvent.click(screen.getByRole('button', { name: new RegExp(`edit ${label}`, 'i') }));
  const input = screen.getByRole('textbox', { name: new RegExp(label, 'i') });
  await fireEvent.input(input, { target: { value } });
  await fireEvent.keyDown(input, { key: 'Enter' });
}

test('AC-159 (RENDER): the selected entry shows its five fields, each change one setMeta edit', async () => {
  const onedit = vi.fn();
  render(MetaPanel, { props: { row, editable: true, onedit, folders: ['Games', 'Demos'] } });

  for (const label of ['Title', 'Year', 'Author', 'Note']) {
    expect(screen.getByRole('button', { name: new RegExp(`edit ${label}`, 'i') })).toBeInTheDocument();
  }
  // The folder is chosen from the cart's folders (AC-195, D-063).
  expect(screen.getByRole('combobox', { name: /folder/i })).toHaveValue('Games');
  expect(screen.getByText('Old Title')).toBeInTheDocument();

  await commit('Title', 'New Title');
  expect(onedit).toHaveBeenLastCalledWith({
    kind: 'setMeta',
    index: row.index,
    field: 'title',
    value: 'New Title',
  });

  await commit('Year', '1985');
  expect(onedit).toHaveBeenLastCalledWith({
    kind: 'setMeta',
    index: row.index,
    field: 'year',
    value: '1985',
  });

  await fireEvent.change(screen.getByRole('combobox', { name: /folder/i }), { target: { value: '' } });
  expect(onedit).toHaveBeenLastCalledWith({
    kind: 'setMeta',
    index: row.index,
    field: 'folder',
    value: '',
  });
  expect(onedit).toHaveBeenCalledTimes(3);

  // An unchanged field is not an edit.
  await commit('Author', '');
  expect(onedit).toHaveBeenCalledTimes(3);
});

test('AC-159 (RENDER): on a view-only cart the fields are shown, not offered for editing', () => {
  render(MetaPanel, { props: { row, editable: false, onedit: vi.fn(), folders: ['Games'] } });
  expect(screen.getByText('Old Title')).toBeInTheDocument();
  expect(screen.getByText('Games')).toBeInTheDocument();
  expect(screen.queryByRole('button', { name: /edit title/i })).toBeNull();
});

const inFolder = { ...row, folder: 'Games' } as unknown as Row;

test('AC-175 (RENDER): the panel labels the field Folder and edits field folder', async () => {
  const onedit = vi.fn();
  render(MetaPanel, { props: { row: inFolder, editable: true, onedit, folders: ['Games'] } });
  expect(screen.getByRole('combobox', { name: /^folder$/i })).toHaveValue('Games');
  expect(screen.queryByRole('combobox', { name: /group/i })).toBeNull();
});

test('AC-184 (RENDER): changing the Folder field is a setMeta edit, never an arrange', async () => {
  const onedit = vi.fn();
  render(MetaPanel, { props: { row: inFolder, editable: true, onedit, folders: ['Games', 'Demos'] } });
  await fireEvent.change(screen.getByRole('combobox', { name: /folder/i }), { target: { value: 'Demos' } });
  expect(onedit).toHaveBeenCalledExactlyOnceWith({
    kind: 'setMeta',
    index: inFolder.index,
    field: 'folder',
    value: 'Demos',
  });
});

test('AC-195 (RENDER): the inspector heads the fields with the entry\'s number, name, kind and size', () => {
  render(MetaPanel, { props: { row: inFolder, editable: true, onedit: vi.fn(), folders: ['Games', 'Demos'] } });
  const panel = screen.getByRole('complementary', { name: /entry details/i });
  expect(panel).toHaveTextContent(`// entry ${inFolder.index}`);
  expect(panel).toHaveTextContent(inFolder.name);
  expect(panel).toHaveTextContent(inFolder.kind);
  expect(panel).toHaveTextContent(inFolder.size);
  const options = Array.from(screen.getByRole('combobox', { name: /folder/i }).querySelectorAll('option')).map(
    (o) => o.textContent,
  );
  expect(options).toEqual(['Not in a folder', 'Games', 'Demos']);
});

test('AC-195 (RENDER): with nothing selected the inspector says so', () => {
  render(MetaPanel, { props: { row: null, editable: true, onedit: vi.fn(), folders: [] } });
  expect(screen.getByRole('complementary', { name: /entry details/i })).toHaveTextContent(
    /select an entry to see its details/i,
  );
});
