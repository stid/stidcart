import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import { afterEach, expect, test, vi } from 'vitest';
import FolderMenu from './FolderMenu.svelte';

// Vitest runs without globals here, so testing-library cannot unmount between tests by itself.
afterEach(cleanup);

function setup(canUp: boolean, canDown: boolean) {
  const props = {
    name: 'Games',
    canUp,
    canDown,
    onrename: vi.fn(),
    onmove: vi.fn(),
    ondelete: vi.fn(),
  };
  render(FolderMenu, { props });
  return props;
}

async function open() {
  const trigger = screen.getByRole('button', {
    name: /actions for folder games/i,
  });
  trigger.focus();
  await fireEvent.keyDown(trigger, { key: 'Enter' });
}

test('AC-183 (RENDER): Rename on a folder row starts renaming that folder', async () => {
  const p = setup(true, true);
  await open();
  await fireEvent.click(await screen.findByRole('menuitem', { name: /rename/i }));
  expect(p.onrename).toHaveBeenCalledExactlyOnceWith('Games');
  expect(p.onmove).not.toHaveBeenCalled();
});

test('AC-181 (RENDER): Move up and Move down move the folder, disabled at the edges', async () => {
  const p = setup(true, false);
  await open();
  await fireEvent.click(await screen.findByRole('menuitem', { name: /move up/i }));
  expect(p.onmove).toHaveBeenCalledExactlyOnceWith('Games', -1);

  cleanup();
  setup(false, true);
  await open();
  expect(await screen.findByRole('menuitem', { name: /move up/i })).toHaveAttribute('aria-disabled', 'true');
  expect(screen.getByRole('menuitem', { name: /move down/i })).not.toHaveAttribute('aria-disabled', 'true');
});

test('AC-193 (RENDER): Delete Folder, apart from the rest, asks to delete this folder', async () => {
  const p = setup(true, true);
  await open();
  const items = (await screen.findAllByRole('menuitem')).map((i) => i.textContent?.trim());
  expect(items).toEqual(['Rename', 'Move up', 'Move down', 'Delete Folder']);
  await fireEvent.click(screen.getByRole('menuitem', { name: /delete folder/i }));
  expect(p.ondelete).toHaveBeenCalledExactlyOnceWith('Games');
  expect(p.onmove).not.toHaveBeenCalled();
});
