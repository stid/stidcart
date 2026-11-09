<script lang="ts">
  // Before any cart: open one (the drop zone, or Open…) or start a new one, side by side, each with
  // its shortcut (design canvas "Empty state", board B).
  import Mark from './Mark.svelte';

  let { onopen, onnew }: { onopen: () => void; onnew: () => void } = $props();
</script>

<div class="empty">
  <section class="zone" aria-labelledby="empty-open">
    <Mark size={56} />
    <h1 id="empty-open">Open a cartridge</h1>
    <p>Drop an EasyFlash <code>.crt</code> here. Nothing is written until you save.</p>
    <button class="button primary" onclick={onopen}>Open… <kbd>⌘O</kbd></button>
  </section>
  <section class="start" aria-labelledby="empty-new">
    <svg class="new-mark" width="56" height="56" viewBox="0 0 64 64" aria-hidden="true">
      <rect x="8" y="10" width="48" height="44" rx="5" fill="none" stroke-width="2" stroke-dasharray="4 3" />
      <path d="M32 22v20M22 32h20" stroke-width="2.5" stroke-linecap="round" />
    </svg>
    <h1 id="empty-new">Start a new cart</h1>
    <p>An empty cart, ready for programs, cartridges and folders.</p>
    <button class="button" onclick={onnew}>New Cart <kbd>⌘N</kbd></button>
  </section>
</div>

<style>
  .empty {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: center;
    align-content: center;
    gap: var(--space-5);
    height: 100%;
    padding: var(--space-6);
  }

  .zone,
  .start {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: min(340px, 100%);
    min-height: 300px;
    padding: var(--space-6);
    border-radius: var(--radius-lg);
    text-align: center;
  }

  /* Only the left panel takes a drop, so only it wears the dashed drop border. */
  .zone {
    border: 1.5px dashed var(--border-strong);
  }

  .start {
    border: 1px solid var(--border);
    background: var(--bg-subtle);
  }

  .new-mark rect {
    stroke: var(--border-strong);
  }

  .new-mark path {
    stroke: var(--accent);
  }

  h1 {
    margin: var(--space-4) 0 0;
    font-size: var(--text-lg);
    font-weight: 600;
  }

  p {
    margin: var(--space-2) 0 var(--space-5);
    color: var(--text-muted);
  }

  code,
  kbd {
    font-family: var(--font-mono);
    font-size: var(--text-sm);
  }

  /* The shortcut reads as part of the button, not as a separate key cap. */
  kbd {
    margin-left: var(--space-2);
    padding: 0;
    border: none;
    background: none;
    box-shadow: none;
    color: inherit;
    opacity: 0.7;
  }
</style>
