// The window's frame, driven through a fake host as viewer.test.ts does, so no webview is needed.
import { cleanup, render, screen } from '@testing-library/svelte';
import { afterEach, expect, test, vi } from 'vitest';
import App from './App.svelte';
import two from './lib/fixtures/two.json';
import type { Document, Host } from './lib/types';

afterEach(cleanup);

const clean: Document = {
  file: 'two.crt',
  untitled: false,
  listing: two,
  rows: two.rows.map(() => ({ alignable: false })),
  readOnly: null,
  menuReplaceable: false,
  dirty: false,
  canUndo: false,
  canRedo: false,
};

/** A host whose drop callback the test can fire, opening whatever `openDocument` gives. */
function fakeHost(openDocument: Host['openDocument']) {
  let dropped: (paths: string[]) => void = () => {};
  const host: Host = {
    pickFile: vi.fn(async () => null),
    onDrop: vi.fn((cb) => (dropped = cb)),
    pickInputs: vi.fn(async () => []),
    pickSavePath: vi.fn(async () => null),
    confirmUnsaved: vi.fn(async () => 'cancel' as const),
    onMenu: vi.fn(),
    onCloseRequested: vi.fn(),
    onExitRequested: vi.fn(),
    openDocument: vi.fn(openDocument),
    newDocument: vi.fn(async () => clean),
    edit: vi.fn(async () => clean),
    undo: vi.fn(async () => clean),
    redo: vi.fn(async () => clean),
    save: vi.fn(async () => clean),
    saveAs: vi.fn(async () => clean),
    exit: vi.fn(async () => {}),
    selectEntry: vi.fn(),
    showHex: vi.fn(async () => {}),
  };
  return { host, drop: (paths: string[]) => dropped(paths) };
}

/** The bands of the strip right under the title bar, in order, or null when there is none. */
function strip(): string[] | null {
  const next = document.querySelector('header.titlebar')?.nextElementSibling;
  if (!next?.matches('[data-raster]')) return null;
  return [...next.querySelectorAll('[data-band]')].map((b) => b.getAttribute('data-band') ?? '');
}

const BANDS = ['red', 'peach', 'yellow', 'green', 'blue'];

test('AC-121 (RENDER): five raster bands run under the title bar, empty or with a cart open', async () => {
  const { host, drop } = fakeHost(async () => clean);
  render(App, { props: { host } });
  expect(strip()).toEqual(BANDS);

  drop(['/carts/two.crt']);
  await screen.findByRole('table');
  expect(strip()).toEqual(BANDS);
});

test('AC-121 (RENDER): the raster bands stay when a file is refused', async () => {
  const { host, drop } = fakeHost(async () => {
    throw 'not an EasyFlash cartridge';
  });
  render(App, { props: { host } });

  drop(['/carts/bad.crt']);
  await screen.findByText(/not an EasyFlash cartridge/);
  expect(strip()).toEqual(BANDS);
});

test('AC-203 (RENDER): selecting an entry tells the host, which moves an open hex window there', async () => {
  const { host, drop } = fakeHost(async () => clean);
  render(App, { props: { host } });
  drop(['/carts/two.crt']);
  await screen.findByRole('table');
  expect(host.selectEntry).toHaveBeenLastCalledWith(null);

  screen.getByText(two.rows[1].name).click();
  await vi.waitFor(() => expect(host.selectEntry).toHaveBeenLastCalledWith(1));
});

test('AC-222 (RENDER): Edit ▸ Find puts the cursor in the filter box of the open cart', async () => {
  const { host, drop } = fakeHost(async () => clean);
  render(App, { props: { host } });
  drop(['/carts/two.crt']);
  await screen.findByRole('table');
  const box = screen.getByRole('searchbox', { name: /filter entries/i });
  expect(box).not.toHaveFocus();

  const menu = vi.mocked(host.onMenu).mock.calls[0][0];
  menu('find');
  expect(box).toHaveFocus();
});
