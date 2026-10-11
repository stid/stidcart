<script lang="ts">
  let { onopen }: { onopen: () => void } = $props();
</script>

<section class="empty">
  <div class="zone">
    <svg class="icon" viewBox="0 0 48 48" aria-hidden="true">
      <rect x="9" y="6" width="30" height="36" rx="4" fill="none" stroke="currentColor" stroke-width="2" />
      <path d="M15 34h18M15 38h18" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
      <rect x="15" y="12" width="18" height="14" rx="2" fill="currentColor" opacity="0.18" />
    </svg>
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

  .icon {
    width: 44px;
    height: 44px;
    color: var(--text-faint);
    margin-bottom: var(--space-4);
  }

  h1 {
    margin: 0;
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
