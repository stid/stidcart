// What differs between macOS, Windows and Linux in the window: how a shortcut is written, the room
// the title bar leaves for the traffic lights (App.svelte's `data-platform`), and who gets the keys.
import { cleanup, render, screen } from '@testing-library/svelte';
import { afterEach, expect, test, vi } from 'vitest';
import App from '../App.svelte';
import EmptyState from './EmptyState.svelte';
import Toolbar from './Toolbar.svelte';
import { forwardMenuKeys, menuKey, platformOf, shortcut } from './platform';

const WEBVIEW2 =
  'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36 Edg/142.0.0.0';
const WEBKITGTK = 'Mozilla/5.0 (X11; Linux aarch64) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15';
const WKWEBVIEW = 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko)';

/** Render what follows as a Windows webview would. */
function onWindows() {
  vi.stubGlobal('navigator', { userAgent: WEBVIEW2 });
}

/** Render what follows as a Linux webview would. */
function onLinux() {
  vi.stubGlobal('navigator', { userAgent: WEBKITGTK });
}

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

test('the platform is the one the webview names; anything else is drawn as on a Mac', () => {
  expect(platformOf(WEBVIEW2)).toBe('windows');
  expect(platformOf(WEBKITGTK)).toBe('linux');
  expect(platformOf('Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15 (KHTML, like Gecko)')).toBe('linux');
  expect(platformOf(WKWEBVIEW)).toBe('macos');
  expect(platformOf('')).toBe('macos');
  // The test runner on a Linux machine: jsdom writes the platform in lower case.
  expect(platformOf('Mozilla/5.0 (linux) AppleWebKit/537.36 (KHTML, like Gecko) jsdom/27.0.0')).toBe('macos');
});

test('on Linux a shortcut is written with Ctrl, as on Windows', () => {
  expect(shortcut('O', {}, 'linux')).toBe('Ctrl+O');
  expect(shortcut('N', { shift: true }, 'linux')).toBe('Ctrl+Shift+N');
  onLinux();
  render(EmptyState, { props: { onopen: vi.fn(), onnew: vi.fn() } });
  expect(screen.getByRole('region', { name: /open a cartridge/i })).toHaveTextContent('Ctrl+O');
  expect(document.body).not.toHaveTextContent('⌘');
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

test('on Linux the title bar is marked for no inset and is no drag region', () => {
  onLinux();
  render(App, { props: { host: idle } });
  const bar = document.querySelector('header.titlebar')!;
  expect(bar.closest('[data-platform]')).toHaveAttribute('data-platform', 'linux');
  expect(document.querySelectorAll('[data-tauri-drag-region]')).toHaveLength(0);
});

/** A keydown as the page sees it. */
function key(k: string, mods: { ctrl?: boolean; shift?: boolean; alt?: boolean; meta?: boolean } = {}) {
  return new KeyboardEvent('keydown', {
    key: k,
    ctrlKey: mods.ctrl,
    shiftKey: mods.shift,
    altKey: mods.alt,
    metaKey: mods.meta,
    bubbles: true,
    cancelable: true,
  });
}

test('AC-216 (RENDER): each menu shortcut names its menu item, and no other key does', () => {
  expect(menuKey(key('n', { ctrl: true }))).toBe('new');
  expect(menuKey(key('o', { ctrl: true }))).toBe('open');
  expect(menuKey(key('N', { ctrl: true, shift: true }))).toBe('new-folder');
  expect(menuKey(key('s', { ctrl: true }))).toBe('save');
  expect(menuKey(key('S', { ctrl: true, shift: true }))).toBe('save-as');
  expect(menuKey(key('z', { ctrl: true }))).toBe('undo');
  expect(menuKey(key('Z', { ctrl: true, shift: true }))).toBe('redo');
  expect(menuKey(key('q', { ctrl: true }))).toBe('quit');
  expect(menuKey(key('H', { ctrl: true, shift: true }))).toBe('hex');
  expect(menuKey(key('f', { ctrl: true }))).toBe('find');

  expect(menuKey(key('n'))).toBeNull();
  expect(menuKey(key('h', { ctrl: true }))).toBeNull();
  expect(menuKey(key('c', { ctrl: true }))).toBeNull();
  expect(menuKey(key('n', { ctrl: true, alt: true }))).toBeNull();
  expect(menuKey(key('n', { meta: true }))).toBeNull();
});

test('AC-216 (RENDER): on Windows a menu shortcut pressed in the page is sent on as its menu item, once', () => {
  onWindows();
  const send = vi.fn();
  const stop = forwardMenuKeys(send);
  const typed = key('N', { ctrl: true, shift: true });
  document.body.dispatchEvent(typed);
  expect(send).toHaveBeenCalledExactlyOnceWith('new-folder');
  expect(typed.defaultPrevented).toBe(true);

  const letter = key('n');
  document.body.dispatchEvent(letter);
  expect(send).toHaveBeenCalledOnce();
  expect(letter.defaultPrevented).toBe(false);
  stop();
});

test('AC-216 (RENDER): on a Mac the page forwards no key: the native menu has them', () => {
  const send = vi.fn();
  const stop = forwardMenuKeys(send);
  document.body.dispatchEvent(key('n', { ctrl: true }));
  document.body.dispatchEvent(key('n', { meta: true }));
  expect(send).not.toHaveBeenCalled();
  stop();
});

test('on Linux the page forwards no key: the native menu gets them (seen on Ubuntu 26.04)', () => {
  onLinux();
  const send = vi.fn();
  const stop = forwardMenuKeys(send);
  document.body.dispatchEvent(key('n', { ctrl: true }));
  expect(send).not.toHaveBeenCalled();
  stop();
});
