// The viewer's state: what the window shows, and how opening a file changes it. Components only
// render this; the host (host.ts, or a fake in tests) is the only way out to the desktop.
import type { Host, ViewState } from './types';

/** The file name a path ends in, which is what the window shows. */
function fileName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

export function createViewer(host: Host) {
  let state = $state<ViewState>({ kind: 'empty' });
  let notice = $state<string | null>(null);

  async function open(path: string) {
    notice = null;
    const file = fileName(path);
    try {
      state = { kind: 'listing', file, listing: await host.openCartridge(path) };
    } catch (e) {
      // Whatever was on screen describes another file, so it goes.
      state = { kind: 'error', file, message: String(e) };
    }
  }

  return {
    get state() {
      return state;
    },
    get notice() {
      return notice;
    },
    async pick() {
      const path = await host.pickFile();
      if (path !== null) await open(path);
    },
    async drop(paths: string[]) {
      if (paths.length !== 1) {
        notice = 'Drop one file at a time.';
        return;
      }
      await open(paths[0]);
    },
  };
}
