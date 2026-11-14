// What the window does differently on Windows, in one place. The window was drawn for macOS: an
// overlay title bar with the traffic lights in it, and shortcuts written with ⌘. On Windows the
// title bar is the system's own, above the page, and the menu's shortcuts use Ctrl. The native
// menu's side of this is `gui/src-tauri/src/menu.rs`.

export type Platform = 'macos' | 'windows';

/**
 * The platform a webview's user agent names. Only WebView2 says "Windows"; everything else,
 * Linux and the test runner included, is drawn as on a Mac, as it was before Windows existed.
 */
export function platformOf(userAgent: string): Platform {
  return userAgent.includes('Windows') ? 'windows' : 'macos';
}

/** This window's platform. `App.svelte` puts it on the page as `data-platform`. */
export function platform(): Platform {
  return platformOf(navigator.userAgent);
}

/** A menu shortcut as this platform writes it: ⇧⌘N on a Mac, Ctrl+Shift+N on Windows. */
export function shortcut(key: string, { shift = false }: { shift?: boolean } = {}, on: Platform = platform()): string {
  if (on === 'windows') return `Ctrl+${shift ? 'Shift+' : ''}${key}`;
  return `${shift ? '⇧' : ''}⌘${key}`;
}

/**
 * The value of `data-tauri-drag-region` on the title bar's parts. On a Mac the page's bar is the
 * window's title bar, so it drags the window. On Windows the system's title bar does that, and a
 * drag region under it would also maximize the window on a double-click between the buttons.
 */
export function dragRegion(): '' | undefined {
  return platform() === 'macos' ? '' : undefined;
}
