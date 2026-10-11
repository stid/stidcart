// The viewer's state machine, driven through a fake host (gui-shell). The real host in host.ts is
// the only code that touches Tauri, so nothing here needs a webview.
import { describe, expect, test, vi } from 'vitest';
import two from './fixtures/two.json';
import refused from './fixtures/refused.json';
import type { Host, Listing } from './types';
import { createViewer } from './viewer.svelte';

const listing: Listing = two;

function fakeHost(overrides: Partial<Host> = {}): Host {
  return {
    pickFile: vi.fn(async () => null),
    onDrop: vi.fn(),
    openCartridge: vi.fn(async () => listing),
    ...overrides,
  };
}

describe('viewer', () => {
  test('AC-52 (RENDER): a single dropped file is opened and shown', async () => {
    const host = fakeHost();
    const v = createViewer(host);
    await v.drop(['/carts/two.crt']);
    expect(host.openCartridge).toHaveBeenCalledWith('/carts/two.crt');
    expect(v.state).toEqual({ kind: 'listing', file: 'two.crt', listing });
  });

  test('AC-53 (RENDER): a picked file is opened; dismissing the chooser changes nothing', async () => {
    const host = fakeHost({ pickFile: vi.fn(async () => '/carts/two.crt') });
    const v = createViewer(host);
    await v.pick();
    expect(host.openCartridge).toHaveBeenCalledWith('/carts/two.crt');
    expect(v.state).toEqual({ kind: 'listing', file: 'two.crt', listing });

    host.pickFile = vi.fn(async () => null);
    await v.pick();
    expect(host.openCartridge).toHaveBeenCalledTimes(1);
    expect(v.state).toEqual({ kind: 'listing', file: 'two.crt', listing });
  });

  test('AC-54 (RENDER): a refused file replaces the shown cartridge with its reason', async () => {
    const host = fakeHost();
    const v = createViewer(host);
    await v.drop(['/carts/two.crt']);
    host.openCartridge = vi.fn(async () => {
      throw refused;
    });
    await v.drop(['/carts/in_8k.crt']);
    expect(v.state).toEqual({ kind: 'error', file: 'in_8k.crt', message: refused });
  });

  test('AC-55 (RENDER): opening a valid file after an error shows it and clears the error', async () => {
    const host = fakeHost({
      openCartridge: vi.fn(async () => {
        throw refused;
      }),
    });
    const v = createViewer(host);
    await v.drop(['/carts/in_8k.crt']);
    expect(v.state.kind).toBe('error');
    host.openCartridge = vi.fn(async () => listing);
    await v.drop(['/carts/two.crt']);
    expect(v.state).toEqual({ kind: 'listing', file: 'two.crt', listing });
  });

  test('AC-56 (RENDER): dropping several files opens none and asks for one', async () => {
    const host = fakeHost();
    const v = createViewer(host);
    await v.drop(['/carts/two.crt']);
    await v.drop(['/carts/a.crt', '/carts/b.crt']);
    expect(host.openCartridge).toHaveBeenCalledTimes(1);
    expect(v.state).toEqual({ kind: 'listing', file: 'two.crt', listing });
    expect(v.notice).toMatch(/one file/i);

    await v.drop(['/carts/two.crt']);
    expect(v.notice).toBeNull();
  });
});
