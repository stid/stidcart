import { render, screen } from '@testing-library/svelte';
import { expect, test } from 'vitest';
import refused from './fixtures/refused.json';
import ErrorState from './ErrorState.svelte';

test('AC-54 (RENDER): a refused file shows its reason and no entry rows', () => {
  render(ErrorState, { props: { file: 'in_8k.crt', message: refused } });
  expect(screen.getByText('in_8k.crt')).toBeInTheDocument();
  expect(screen.getByText(refused)).toBeInTheDocument();
  expect(screen.queryByRole('table')).toBeNull();
});
