import { cleanup, fireEvent, render, screen } from '@testing-library/svelte';
import { afterEach, expect, test, vi } from 'vitest';
import EditableText from './EditableText.svelte';

// Vitest runs without globals here, so testing-library cannot unmount between tests by itself.
afterEach(cleanup);

function setup() {
  const oncommit = vi.fn();
  render(EditableText, { props: { value: 'hello', label: 'Name of entry 0', oncommit } });
  return oncommit;
}

async function startEditing() {
  await fireEvent.click(screen.getByRole('button', { name: /edit name of entry 0/i }));
  return screen.getByRole('textbox', { name: 'Name of entry 0' }) as HTMLInputElement;
}

test('AC-65 (RENDER): Enter commits the new name', async () => {
  const oncommit = setup();
  const input = await startEditing();
  expect(input.value).toBe('hello');
  await fireEvent.input(input, { target: { value: 'greetings' } });
  await fireEvent.keyDown(input, { key: 'Enter' });
  expect(oncommit).toHaveBeenCalledExactlyOnceWith('greetings');
  expect(screen.queryByRole('textbox')).toBeNull();
});

test('AC-65 (RENDER): Esc restores the old name and commits nothing', async () => {
  const oncommit = setup();
  const input = await startEditing();
  await fireEvent.input(input, { target: { value: 'oops' } });
  await fireEvent.keyDown(input, { key: 'Escape' });
  expect(oncommit).not.toHaveBeenCalled();
  expect(screen.queryByRole('textbox')).toBeNull();
  expect(screen.getByText('hello')).toBeInTheDocument();
});

test('AC-65 (RENDER): leaving the field commits; an unchanged name commits nothing', async () => {
  const oncommit = setup();
  let input = await startEditing();
  await fireEvent.blur(input);
  expect(oncommit).not.toHaveBeenCalled();

  input = await startEditing();
  await fireEvent.input(input, { target: { value: 'renamed' } });
  await fireEvent.blur(input);
  expect(oncommit).toHaveBeenCalledExactlyOnceWith('renamed');
});
