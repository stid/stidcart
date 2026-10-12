// The only module that talks to Tauri. Everything else takes a `Host`, so tests pass a fake and
// this file is exercised by the live check, not by unit tests.
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { open } from '@tauri-apps/plugin-dialog';
import type { Host, Listing } from './types';

export const host: Host = {
  async pickFile() {
    const path = await open({
      multiple: false,
      directory: false,
      filters: [{ name: 'Cartridge image', extensions: ['crt'] }],
    });
    return path ?? null;
  },
  onDrop(cb) {
    void getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === 'drop') cb(event.payload.paths);
    });
  },
  openCartridge(path) {
    return invoke<Listing>('open_cartridge', { path });
  },
};
