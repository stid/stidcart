// What the window does differently off a Mac, in one place. The window was drawn for macOS: an
// overlay title bar with the traffic lights in it, and shortcuts written with ⌘. On Windows and
// Linux the title bar is the system's own, above the page, and the menu's shortcuts use Ctrl. On
// Windows the page also has to pass them on, because the webview holds the keyboard. The native
// menu's side of this is `gui/src-tauri/src/menu.rs`.

import type { MenuId } from './types';

export type Platform = 'macos' | 'windows' | 'linux';

/**
 * The platform a webview's user agent names. WebView2 says "Windows" and WebKitGTK "Linux";
 * everything else, the test runner included (jsdom writes "linux" in lower case), is drawn as on
 * a Mac.
 */
export function platformOf(userAgent: string): Platform {
  if (userAgent.includes('Windows')) return 'windows';
  if (userAgent.includes('Linux')) return 'linux';
  return 'macos';
}

/** This window's platform. `App.svelte` puts it on the page as `data-platform`. */
export function platform(): Platform {
  return platformOf(navigator.userAgent);
}

/** A menu shortcut as this platform writes it: ⇧⌘N on a Mac, Ctrl+Shift+N elsewhere. */
export function shortcut(key: string, { shift = false }: { shift?: boolean } = {}, on: Platform = platform()): string {
  if (on !== 'macos') return `Ctrl+${shift ? 'Shift+' : ''}${key}`;
  return `${shift ? '⇧' : ''}⌘${key}`;
}

/**
 * The value of `data-tauri-drag-region` on the title bar's parts. On a Mac the page's bar is the
 * window's title bar, so it drags the window. Elsewhere the system's title bar does that, and a
 * drag region under it would also maximize the window on a double-click between the buttons.
 */
export function dragRegion(): '' | undefined {
  return platform() === 'macos' ? '' : undefined;
}

/** A menu item a shortcut stands for: one the window acts on, or View ▸ Hex Viewer. */
export type MenuKey = MenuId | 'hex';

/** The menu's shortcuts, as `menu.rs` gives them: the key, lower case, then whether Shift is held. */
const MENU_KEYS: Record<string, [plain: MenuKey | null, shifted: MenuKey | null]> = {
  n: ['new', 'new-folder'],
  o: ['open', null],
  s: ['save', 'save-as'],
  z: ['undo', 'redo'],
  q: ['quit', null],
  h: [null, 'hex'],
};

/** The menu item a keydown is the shortcut of, or null. */
export function menuKey(event: KeyboardEvent): MenuKey | null {
  if (!event.ctrlKey || event.altKey || event.metaKey) return null;
  return MENU_KEYS[event.key.toLowerCase()]?.[event.shiftKey ? 1 : 0] ?? null;
}

/**
 * On Windows, pass the menu's shortcuts on to `send`; returns how to stop. There the webview has
 * the keyboard whenever the window does, and the native menu's accelerators never see a key
 * (seen on a Windows runner: every item worked by click and none by its shortcut). On a Mac the
 * and on Linux the menu gets the key itself (seen on Ubuntu 26.04), so forwarding it as well
 * would run each command twice.
 */
export function forwardMenuKeys(send: (id: MenuKey) => void): () => void {
  if (platform() !== 'windows') return () => {};
  const onKey = (event: KeyboardEvent) => {
    const id = menuKey(event);
    if (id === null) return;
    event.preventDefault();
    send(id);
  };
  window.addEventListener('keydown', onKey);
  return () => window.removeEventListener('keydown', onKey);
}
