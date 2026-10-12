<script lang="ts">
  import { onMount } from 'svelte';
  import EmptyState from './lib/EmptyState.svelte';
  import ErrorState from './lib/ErrorState.svelte';
  import Listing from './lib/Listing.svelte';
  import type { Host } from './lib/types';
  import { createViewer } from './lib/viewer.svelte';

  let { host }: { host: Host } = $props();

  // The host is fixed for the window's lifetime, so capturing it once is the intent.
  // svelte-ignore state_referenced_locally
  const viewer = createViewer(host);
  onMount(() => host.onDrop((paths) => void viewer.drop(paths)));

  function onkeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'o') {
      e.preventDefault();
      void viewer.pick();
    }
  }
</script>

<svelte:window {onkeydown} />

<div class="app">
  <header class="titlebar" data-tauri-drag-region>
    <span class="title" data-tauri-drag-region>stidcart</span>
    {#if viewer.state.kind !== 'empty'}
      <button class="button" onclick={() => viewer.pick()}>Open…</button>
    {/if}
  </header>

  <main>
    {#if viewer.state.kind === 'listing'}
      <Listing listing={viewer.state.listing} file={viewer.state.file} />
    {:else if viewer.state.kind === 'error'}
      <ErrorState file={viewer.state.file} message={viewer.state.message} />
    {:else}
      <EmptyState onopen={() => viewer.pick()} />
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

  .title {
    font-size: var(--text-md);
    font-weight: 600;
    color: var(--text-muted);
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
