import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import { afterEach, expect, test, vi } from 'vitest';
import EmptyState from './EmptyState.svelte';

afterEach(cleanup);

test('AC-57 (RENDER): before any file is opened, the window invites opening or dropping a cartridge', async () => {
  const onopen = vi.fn();
  render(EmptyState, { props: { onopen } });
  expect(screen.getByText(/drop/i)).toBeInTheDocument();
  await fireEvent.click(screen.getByRole('button', { name: /open/i }));
  expect(onopen).toHaveBeenCalledOnce();
  expect(screen.queryByRole('table')).toBeNull();
});

test('AC-122 (RENDER): the empty state is headed by the stidcart mark', () => {
  render(EmptyState, { props: { onopen: vi.fn() } });
  const mark = screen.getByTestId('mark');
  const zone = mark.closest('.zone');
  expect(zone?.firstElementChild).toBe(mark);
  expect(mark.querySelectorAll('[data-band]')).toHaveLength(5);
});
