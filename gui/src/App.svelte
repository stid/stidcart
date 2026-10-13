<script lang="ts">
  import { onMount } from 'svelte';
  import EmptyState from './lib/EmptyState.svelte';
  import ErrorState from './lib/ErrorState.svelte';
  import Listing from './lib/Listing.svelte';
  import Toolbar from './lib/Toolbar.svelte';
  import type { Host } from './lib/types';
  import { createViewer } from './lib/viewer.svelte';

  let { host }: { host: Host } = $props();

  // The host is fixed for the window's lifetime, so capturing it once is the intent.
  // svelte-ignore state_referenced_locally
  const viewer = createViewer(host);

  // Shortcuts (⌘N, ⌘O, ⌘S, ⌘⇧S, ⌘Z, ⌘⇧Z) arrive only through the native menu. The webview sees
  // the same keydown too, so handling it here as well would run each command twice (T2 spike).
  onMount(() => {
    host.onDrop((paths) => void viewer.drop(paths));
    host.onMenu((id) => void viewer.menu(id));
    host.onCloseRequested(() => viewer.requestClose());
    host.onExitRequested(() => void viewer.requestQuit());
  });

  const doc = $derived(viewer.state.kind === 'document' ? viewer.state.doc : null);
</script>

<div class="app">
  <header class="titlebar" data-tauri-drag-region>
    <Toolbar
      {doc}
      opened={viewer.state.kind !== 'empty'}
      onadd={(aligned) => void viewer.add(aligned)}
      onopen={() => void viewer.pick()}
    />
  </header>

  <main>
    {#if doc}
      <Listing
        listing={doc.listing}
        file={doc.file}
        readOnly={doc.readOnly}
        rows={doc.rows}
        onedit={(e) => void viewer.edit(e)}
        onreplace={(index) => void viewer.replace(index)}
      />
    {:else if viewer.state.kind === 'error'}
      <ErrorState file={viewer.state.file} message={viewer.state.message} />
    {:else}
      <EmptyState onopen={() => void viewer.pick()} />
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
