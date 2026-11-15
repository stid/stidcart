<script lang="ts">
  import { onMount } from 'svelte';
  import EmptyState from './lib/EmptyState.svelte';
  import ErrorState from './lib/ErrorState.svelte';
  import Listing from './lib/Listing.svelte';
  import { dragRegion, platform } from './lib/platform';
  import Toolbar from './lib/Toolbar.svelte';
  import type { Host } from './lib/types';
  import { createViewer } from './lib/viewer.svelte';

  let { host }: { host: Host } = $props();

  // The host is fixed for the window's lifetime, so capturing it once is the intent.
  // svelte-ignore state_referenced_locally
  const viewer = createViewer(host);

  /** The open cart's listing, which owns its folder tree: New Folder acts there (D-063). */
  let listingView = $state<ReturnType<typeof Listing> | null>(null);

  // Shortcuts (⌘N, ⌘O, ⌘S, ⌘⇧S, ⌘Z, ⌘⇧Z) arrive only through the native menu. The webview sees
  // the same keydown too, so handling it here as well would run each command twice (T2 spike).
  // On Windows the menu never sees the key, and `forwardMenuKeys` (platform.ts) sends it there.
  onMount(() => {
    host.onDrop((paths) => void viewer.drop(paths));
    host.onMenu((id) => (id === 'new-folder' ? void listingView?.newFolder() : void viewer.menu(id)));
    host.onCloseRequested(() => viewer.requestClose());
    host.onExitRequested(() => void viewer.requestQuit());
  });

  const doc = $derived(viewer.state.kind === 'document' ? viewer.state.doc : null);

  /** The raster strip's bands, top to bottom (D-046); each colour is a theme token. */
  const BANDS = ['red', 'peach', 'yellow', 'green', 'blue'];
</script>

<div class="app" data-platform={platform()}>
  <header class="titlebar" data-tauri-drag-region={dragRegion()}>
    <Toolbar
      {doc}
      opened={viewer.state.kind !== 'empty'}
      onadd={(aligned) => void viewer.add(aligned)}
      onopen={() => void viewer.pick()}
      onnewfolder={() => void listingView?.newFolder()}
    />
  </header>
  <div class="raster" data-raster aria-hidden="true">
    {#each BANDS as band (band)}<span data-band={band} style:background="var(--band-{band})"></span>{/each}
  </div>

  <main>
    {#if doc}
      {#key viewer.generation}
        <Listing
          bind:this={listingView}
          listing={doc.listing}
          file={doc.file}
          readOnly={doc.readOnly}
          menuReplaceable={doc.menuReplaceable}
          rows={doc.rows}
          onedit={(e) => viewer.edit(e)}
          onreplace={(index) => void viewer.replace(index)}
          onadd={(aligned) => void viewer.add(aligned)}
          onselect={(index) => host.selectEntry(index)}
          onhex={(index) => void host.showHex(index)}
        />
      {/key}
    {:else if viewer.state.kind === 'error'}
      <ErrorState file={viewer.state.file} message={viewer.state.message} />
    {:else}
      <EmptyState onopen={() => void viewer.pick()} onnew={() => void viewer.newCart()} />
    {/if}
  </main>

  {#if viewer.notice}
    <div class="notice" role="status">{viewer.notice}</div>
  {/if}
</div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .titlebar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    flex: none;
    height: var(--titlebar-height);
    padding: 0 var(--space-4) 0 var(--titlebar-inset);
    border-bottom: 1px solid var(--border);
    background: var(--bg-subtle);
  }

  /* The system draws the title bar above the page, so there are no traffic lights to clear. */
  .app:not([data-platform='macos']) .titlebar {
    padding-left: var(--space-4);
  }

  .raster {
    display: flex;
    flex-direction: column;
    flex: none;
  }

  .raster span {
    height: var(--band-height);
  }

  main {
    flex: 1;
    min-height: 0;
  }

  .notice {
    position: fixed;
    left: 50%;
    bottom: var(--space-5);
    transform: translateX(-50%);
    padding: var(--space-2) var(--space-4);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-elevated);
    box-shadow: var(--shadow);
    font-size: var(--text-md);
  }
</style>
