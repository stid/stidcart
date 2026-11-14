// What differs between macOS and Windows in the window: how a shortcut is written, and the room
// the title bar leaves for the traffic lights (App.svelte's `data-platform`).
import { cleanup, render, screen } from '@testing-library/svelte';
import { afterEach, expect, test, vi } from 'vitest';
import App from '../App.svelte';
import EmptyState from './EmptyState.svelte';
import Toolbar from './Toolbar.svelte';
import { platformOf, shortcut } from './platform';

const WEBVIEW2 =
  'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36 Edg/142.0.0.0';
const WKWEBVIEW = 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko)';

/** Render what follows as a Windows webview would. */
function onWindows() {
  vi.stubGlobal('navigator', { userAgent: WEBVIEW2 });
}

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

test('the platform is Windows only in a Windows webview; anything else is drawn as on a Mac', () => {
  expect(platformOf(WEBVIEW2)).toBe('windows');
  expect(platformOf(WKWEBVIEW)).toBe('macos');
  expect(platformOf('Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko)')).toBe('macos');
  expect(platformOf('')).toBe('macos');
});

test('AC-214 (RENDER): a shortcut is written with ⌘ on a Mac and with Ctrl on Windows', () => {
  expect(shortcut('O', {}, 'macos')).toBe('⌘O');
  expect(shortcut('N', { shift: true }, 'macos')).toBe('⇧⌘N');
  expect(shortcut('O', {}, 'windows')).toBe('Ctrl+O');
  expect(shortcut('N', { shift: true }, 'windows')).toBe('Ctrl+Shift+N');
});

test('AC-214 (RENDER): on Windows the empty state names Ctrl+O and Ctrl+N', () => {
  onWindows();
  render(EmptyState, { props: { onopen: vi.fn(), onnew: vi.fn() } });
  expect(screen.getByRole('region', { name: /open a cartridge/i })).toHaveTextContent('Ctrl+O');
  expect(screen.getByRole('region', { name: /start a new cart/i })).toHaveTextContent('Ctrl+N');
  expect(document.body).not.toHaveTextContent('⌘');
});

test('AC-214 (RENDER): on Windows the New Folder button names Ctrl+Shift+N', () => {
  onWindows();
  const doc = { file: 'a.crt', readOnly: null, dirty: false } as never;
  render(Toolbar, { props: { doc, onadd: vi.fn(), onopen: vi.fn(), onnewfolder: vi.fn() } });
  expect(screen.getByRole('button', { name: 'New Folder' })).toHaveAttribute('title', 'New Folder (Ctrl+Shift+N)');
});

test('the New Folder button names ⇧⌘N on a Mac', () => {
  const doc = { file: 'a.crt', readOnly: null, dirty: false } as never;
  render(Toolbar, { props: { doc, onadd: vi.fn(), onopen: vi.fn(), onnewfolder: vi.fn() } });
  expect(screen.getByRole('button', { name: 'New Folder' })).toHaveAttribute('title', 'New Folder (⇧⌘N)');
});

/** Enough of a host for the window with no cart open. */
const idle = { onDrop: vi.fn(), onMenu: vi.fn(), onCloseRequested: vi.fn(), onExitRequested: vi.fn() } as never;

test('AC-215 (RENDER): on a Mac the title bar keeps the traffic-light inset and drags the window', () => {
  render(App, { props: { host: idle } });
  const bar = document.querySelector('header.titlebar')!;
  expect(bar.closest('[data-platform]')).toHaveAttribute('data-platform', 'macos');
  expect(bar).toHaveAttribute('data-tauri-drag-region');
  expect(screen.getByText('stidcart')).toHaveAttribute('data-tauri-drag-region');
});

test('AC-215 (RENDER): on Windows the title bar is marked for no inset and is no drag region', () => {
  onWindows();
  render(App, { props: { host: idle } });
  const bar = document.querySelector('header.titlebar')!;
  expect(bar.closest('[data-platform]')).toHaveAttribute('data-platform', 'windows');
  expect(document.querySelectorAll('[data-tauri-drag-region]')).toHaveLength(0);
});
