<script lang="ts">
  let { file, message }: { file: string; message: string } = $props();
</script>

<section class="error" role="alert">
  <div class="card">
    <svg class="icon" viewBox="0 0 24 24" aria-hidden="true">
      <circle cx="12" cy="12" r="9.5" fill="none" stroke="currentColor" stroke-width="1.6" />
      <path d="M12 7.5v5.5" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" />
      <circle cx="12" cy="16.4" r="1.1" fill="currentColor" />
    </svg>
    <h1>Can't show this file</h1>
    <p class="file">{file}</p>
    <p class="reason">{message}</p>
  </div>
</section>

<style>
  .error {
    display: grid;
    place-items: center;
    height: 100%;
    padding: var(--space-6);
  }

  .card {
    display: flex;
    flex-direction: column;
    align-items: center;
    width: min(460px, 100%);
    text-align: center;
  }

  .icon {
    width: 32px;
    height: 32px;
    color: var(--danger);
    margin-bottom: var(--space-3);
  }

  h1 {
    margin: 0;
    font-size: var(--text-lg);
    font-weight: 600;
  }

  .file {
    margin: var(--space-1) 0 var(--space-4);
    color: var(--text-faint);
    font-size: var(--text-sm);
  }

  .reason {
    margin: 0;
    padding: var(--space-3) var(--space-4);
    border-radius: var(--radius-md);
    background: var(--danger-soft);
    color: var(--danger);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    user-select: text;
  }
</style>
