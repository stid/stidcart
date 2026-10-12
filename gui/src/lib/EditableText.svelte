<script lang="ts">
  // Text that turns into a field to edit it. Enter or leaving the field commits, Esc puts the old
  // text back. Committing an unchanged value is not an edit, so it sends nothing.
  let {
    value,
    label,
    oncommit,
    editing = $bindable(false),
  }: {
    value: string;
    /** Names the field for assistive technology, e.g. "Cart name". */
    label: string;
    oncommit: (value: string) => void;
    editing?: boolean;
  } = $props();

  let draft = $state('');

  function start() {
    draft = value;
    editing = true;
  }

  function finish(commit: boolean) {
    if (!editing) return;
    editing = false;
    if (commit && draft !== value) oncommit(draft);
  }

  function focusAndSelect(node: HTMLInputElement) {
    node.focus();
    node.select();
  }

  // Started from outside (the row menu's Rename): load the current value into the field.
  $effect.pre(() => {
    if (editing && draft === '') draft = value;
  });
</script>

{#if editing}
  <input
    class="field"
    aria-label={label}
    bind:value={draft}
    use:focusAndSelect
    onkeydown={(e) => {
      if (e.key === 'Enter') finish(true);
      else if (e.key === 'Escape') finish(false);
    }}
    onblur={() => finish(true)}
  />
{:else}
  <button class="text" aria-label={`Edit ${label}`} title="Click to edit" onclick={start}>{value}</button>
{/if}

<style>
  .text {
    all: unset;
    cursor: text;
    border-radius: var(--radius-sm);
    padding: 0 var(--space-1);
    margin: 0 calc(-1 * var(--space-1));
    transition: background var(--duration) var(--ease);
  }

  .text:hover {
    background: var(--bg-hover);
  }

  .text:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  .field {
    font: inherit;
    color: inherit;
    width: 100%;
    max-width: 24em;
    box-sizing: border-box;
    padding: 0 var(--space-1);
    margin: 0 calc(-1 * var(--space-1));
    border: 1px solid var(--accent);
    border-radius: var(--radius-sm);
    background: var(--bg-elevated);
    outline: none;
  }
</style>
