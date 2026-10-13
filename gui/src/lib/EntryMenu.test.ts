import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import { afterEach, expect, test, vi } from 'vitest';
import two from './fixtures/two.json';
import type { Row, RowActions } from './types';
import EntryMenu from './EntryMenu.svelte';

// Vitest runs without globals here, so testing-library cannot unmount between tests by itself.
afterEach(cleanup);

const hello: Row = two.rows[0];

function setup(row: Row = hello, actions: RowActions = { alignable: false }) {
  const props = { row, actions, onedit: vi.fn(), onrename: vi.fn(), onreplace: vi.fn() };
  render(EntryMenu, { props });
  return props;
}

/** Open the row's "⋯" menu the way a keyboard user does, then pick an item by its text. */
async function choose(name: RegExp) {
  const trigger = screen.getByRole('button', { name: new RegExp(`actions for ${hello.name}`, 'i') });
  trigger.focus();
  await fireEvent.keyDown(trigger, { key: 'Enter' });
  await fireEvent.click(await screen.findByRole('menuitem', { name }));
}

async function items() {
  const trigger = screen.getByRole('button', { name: /actions for/i });
  trigger.focus();
  await fireEvent.keyDown(trigger, { key: 'Enter' });
  return (await screen.findAllByRole('menuitem')).map((i) => i.textContent?.trim());
}

test('AC-64 (RENDER): Remove removes the entry at its index', async () => {
  const { onedit } = setup();
  await choose(/remove/i);
  expect(onedit).toHaveBeenCalledExactlyOnceWith({ kind: 'remove', index: 0 });
});

test('AC-66 (RENDER): Replace… asks for the replacement for this entry', async () => {
  const { onreplace, onedit } = setup();
  await choose(/replace/i);
  expect(onreplace).toHaveBeenCalledExactlyOnceWith(0);
  expect(onedit).not.toHaveBeenCalled();
});

test('AC-67 (RENDER): Hide on a visible entry, Show on a hidden one', async () => {
  const { onedit } = setup();
  await choose(/^hide$/i);
  expect(onedit).toHaveBeenCalledExactlyOnceWith({ kind: 'setHidden', index: 0, hidden: true });

  cleanup();
  const shown = setup({ ...hello, flags: 'H' });
  await choose(/^show$/i);
  expect(shown.onedit).toHaveBeenCalledExactlyOnceWith({ kind: 'setHidden', index: 0, hidden: false });
});

test('AC-68 (RENDER): alignment is offered only where it applies, and toggles', async () => {
  setup();
  expect((await items()).some((t) => /align/i.test(t ?? ''))).toBe(false);

  cleanup();
  const xbank = setup({ ...hello, kind: 'XB16' }, { alignable: true });
  await choose(/align to 64 kb/i);
  expect(xbank.onedit).toHaveBeenCalledExactlyOnceWith({ kind: 'setAligned', index: 0, aligned: true });

  cleanup();
  const aligned = setup({ ...hello, kind: 'XB16', flags: 'A' }, { alignable: true });
  await choose(/remove 64 kb alignment/i);
  expect(aligned.onedit).toHaveBeenCalledExactlyOnceWith({ kind: 'setAligned', index: 0, aligned: false });
});
