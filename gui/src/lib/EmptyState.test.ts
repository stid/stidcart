import { fireEvent, render, screen } from '@testing-library/svelte';
import { expect, test, vi } from 'vitest';
import EmptyState from './EmptyState.svelte';

test('AC-57 (RENDER): before any file is opened, the window invites opening or dropping a cartridge', async () => {
  const onopen = vi.fn();
  render(EmptyState, { props: { onopen } });
  expect(screen.getByText(/drop/i)).toBeInTheDocument();
  await fireEvent.click(screen.getByRole('button', { name: /open/i }));
  expect(onopen).toHaveBeenCalledOnce();
  expect(screen.queryByRole('table')).toBeNull();
});
