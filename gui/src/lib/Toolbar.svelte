<script lang="ts">
  // The title bar's contents: which cart is open and whether it has unsaved changes, and the
  // commands that act on it. On a Mac the bar is the window's drag region (App.svelte).
  import Mark from './Mark.svelte';
  import { dragRegion, shortcut } from './platform';
  import type { Document } from './types';

  let {
    doc,
    onadd,
    onopen,
    onnewfolder,
    opened = doc !== null,
  }: {
    doc: Document | null;
    onadd: (aligned: boolean) => void;
    /** Add an empty folder to the open cart, as a file browser's New Folder (D-063). */
    onnewfolder: () => void;
    onopen: () => void;
    /** Something is on screen (a cart or a refusal), so Open… belongs in the bar. */
    opened?: boolean;
  } = $props();

  const editable = $derived(doc !== null && doc.readOnly === null);
</script>

<div class="title" data-tauri-drag-region={dragRegion()}>
  <Mark size={16} />
  {#if doc}
    <span class="file" data-tauri-drag-region={dragRegion()}>{doc.file}</span>
    {#if doc.dirty}
      <span class="edited" data-tauri-drag-region={dragRegion()}><span class="dot" aria-hidden="true"></span>Edited</span>
    {/if}
  {:else}
    <span class="app-name" data-tauri-drag-region={dragRegion()}>stidcart</span>
  {/if}
</div>

<div class="actions">
  {#if editable}
    <button class="button" onclick={onnewfolder} title="New Folder ({shortcut('N', { shift: true })})">New Folder</button>
    <button class="button" onclick={() => onadd(false)}>Add…</button>
    <button class="button" onclick={() => onadd(true)} title="Add an xbank cartridge with 64 KB alignment"
      >Add Aligned…</button
    >
  {/if}
  {#if opened}
    <button class="button" onclick={onopen}>Open…</button>
  {/if}
</div>

<style>
  .title {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    min-width: 0;
  }

  .app-name,
  .file {
    font-size: var(--text-md);
    font-weight: 600;
    color: var(--text-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .file {
    color: var(--text);
  }

  .edited {
    display: inline-flex;
    align-items: center;
    gap: var(--space-1);
    font-size: var(--text-sm);
    color: var(--text-faint);
  }

  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--accent);
  }

  .actions {
    display: flex;
    gap: var(--space-2);
  }
</style>
