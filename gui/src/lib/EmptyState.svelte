<script lang="ts">
  import Mark from './Mark.svelte';

  let { onopen }: { onopen: () => void } = $props();
</script>

<section class="empty">
  <div class="zone">
    <Mark size={64} />
    <h1>Drop a cartridge here</h1>
    <p>An EasyFlash <code>.crt</code> image. Nothing is written to it.</p>
    <button class="button primary" onclick={onopen}>Open…</button>
    <p class="hint"><kbd>⌘</kbd> <kbd>O</kbd></p>
  </div>
</section>

<style>
  .empty {
    display: grid;
    place-items: center;
    height: 100%;
    padding: var(--space-6);
  }

  .zone {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: min(440px, 100%);
    padding: var(--space-8) var(--space-6);
    border: 1.5px dashed var(--border-strong);
    border-radius: var(--radius-lg);
    text-align: center;
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

  code {
    font-family: var(--font-mono);
    font-size: var(--text-sm);
  }

  .hint {
    margin: var(--space-3) 0 0;
  }
</style>
