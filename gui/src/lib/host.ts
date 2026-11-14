// The only module that talks to Tauri. Everything else takes a `Host`, so tests pass a fake and
// this file is exercised by the live check, not by unit tests.
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { message, open, save } from '@tauri-apps/plugin-dialog';
import type { Document, Edit, HexEntryLine, HexHost, HexInfo, HexLine, Host, MenuId } from './types';
import { unsavedChoice } from './viewer.svelte';

const CRT = { name: 'Cartridge image', extensions: ['crt'] };

export const host: Host = {
  async pickFile() {
    const path = await open({ multiple: false, directory: false, filters: [CRT] });
    return path ?? null;
  },
  onDrop(cb) {
    void getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === 'drop') cb(event.payload.paths);
    });
  },
  async pickInputs() {
    const paths = await open({
      multiple: true,
      directory: false,
      filters: [{ name: 'Programs and cartridge images', extensions: ['prg', 'crt'] }],
    });
    return paths ?? [];
  },
  async pickSavePath(suggested) {
    return (await save({ defaultPath: suggested, filters: [CRT] })) ?? null;
  },
  async confirmUnsaved(file) {
    const label = await message(`Do you want to save the changes you made to ${file}?`, {
      title: 'Unsaved changes',
      kind: 'warning',
      buttons: { yes: 'Save', no: "Don't Save", cancel: 'Cancel' },
    });
    return unsavedChoice(label);
  },
  onMenu(cb) {
    void listen<MenuId>('menu', (event) => cb(event.payload));
  },
  onCloseRequested(cb) {
    void getCurrentWindow().onCloseRequested(async (event) => {
      // preventDefault may come after the await, as long as it is before this handler returns.
      if (await cb()) await invoke('allow_exit');
      else event.preventDefault();
    });
  },
  onExitRequested(cb) {
    void listen('exit-requested', () => cb());
  },
  openDocument(path) {
    return invoke<Document>('open_document', { path });
  },
  newDocument() {
    return invoke<Document>('new_document');
  },
  edit(edit: Edit) {
    return invoke<Document>('apply_edit', { edit });
  },
  undo() {
    return invoke<Document>('undo');
  },
  redo() {
    return invoke<Document>('redo');
  },
  save() {
    return invoke<Document>('save_document');
  },
  saveAs(path) {
    return invoke<Document>('save_document_as', { path });
  },
  async exit() {
    await invoke('exit_app');
  },
  selectEntry(entry) {
    void invoke('hex_select', { entry });
  },
  async showHex(entry) {
    await invoke('show_hex', { entry });
  },
};

/** Whether this webview is the hex viewer's window (gui/src-tauri/src/lib.rs names it). */
export const isHexWindow = getCurrentWindow().label === 'hex';

export const hexHost: HexHost = {
  info() {
    return invoke<HexInfo>('hex_info');
  },
  lines(first, count) {
    return invoke<HexLine[]>('hex_lines', { first, count });
  },
  entryLines(entry, first, count) {
    return invoke<HexEntryLine[]>('hex_entry_lines', { entry, first, count });
  },
  onChanged(cb) {
    void listen('hex-changed', () => cb());
  },
  onGoto(cb) {
    void listen<number | null>('hex-goto', (event) => cb(event.payload));
  },
};
