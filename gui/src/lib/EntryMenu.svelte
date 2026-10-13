<script lang="ts">
  // The actions on one entry, offered two ways from one list: a "⋯" button, or — when given
  // `children` — a right-click menu on whatever `children` renders (the row). Which actions apply
  // comes from the Rust side (`actions`), never from the kind label.
  import { ContextMenu, DropdownMenu } from 'bits-ui';
  import type { Snippet } from 'svelte';
  import type { Edit, Row, RowActions } from './types';

  let {
    row,
    actions,
    onedit,
    onrename,
    onreplace,
    children,
  }: {
    row: Row;
    actions: RowActions;
    onedit: (e: Edit) => void;
    onrename: (index: number) => void;
    onreplace: (index: number) => void;
    /** Render the row; spread `props` on its root element to make it the right-click target. */
    children?: Snippet<[Record<string, unknown>]>;
  } = $props();

  const hidden = $derived(row.flags.includes('H'));
  const aligned = $derived(row.flags.includes('A'));
</script>

{#snippet items()}
  <DropdownMenu.Item class="menu-item" onSelect={() => onrename(row.index)}>Rename</DropdownMenu.Item>
  <DropdownMenu.Item class="menu-item" onSelect={() => onreplace(row.index)}>Replace…</DropdownMenu.Item>
  <DropdownMenu.Item
    class="menu-item"
    onSelect={() => onedit({ kind: 'setHidden', index: row.index, hidden: !hidden })}
    >{hidden ? 'Show' : 'Hide'}</DropdownMenu.Item
  >
  {#if actions.alignable}
    <DropdownMenu.Item
      class="menu-item"
      onSelect={() => onedit({ kind: 'setAligned', index: row.index, aligned: !aligned })}
      >{aligned ? 'Remove 64 KB alignment' : 'Align to 64 KB'}</DropdownMenu.Item
    >
  {/if}
  <DropdownMenu.Separator class="menu-separator" />
  <DropdownMenu.Item class="menu-item danger" onSelect={() => onedit({ kind: 'remove', index: row.index })}
    >Remove</DropdownMenu.Item
  >
{/snippet}

{#if children}
  <ContextMenu.Root>
    <ContextMenu.Trigger>
      {#snippet child({ props })}
        {@render children(props)}
      {/snippet}
    </ContextMenu.Trigger>
    <ContextMenu.Portal>
      <ContextMenu.Content class="menu">
        {@render items()}
      </ContextMenu.Content>
    </ContextMenu.Portal>
  </ContextMenu.Root>
{:else}
  <DropdownMenu.Root>
    <DropdownMenu.Trigger class="more" aria-label={`Actions for ${row.name}`}>⋯</DropdownMenu.Trigger>
    <DropdownMenu.Portal>
      <DropdownMenu.Content class="menu" align="end" sideOffset={4}>
        {@render items()}
      </DropdownMenu.Content>
    </DropdownMenu.Portal>
  </DropdownMenu.Root>
{/if}

<style>
  /* The menu renders in a portal, outside this component's scope. */
  :global(.more) {
    all: unset;
    cursor: default;
    width: 24px;
    height: 20px;
    display: inline-grid;
    place-items: center;
    border-radius: var(--radius-sm);
    color: var(--text-faint);
    line-height: 1;
  }

  :global(.more:hover),
  :global(.more[data-state='open']) {
    background: var(--bg-hover);
    color: var(--text);
  }

  :global(.more:focus-visible) {
    outline: 2px solid var(--accent);
  }

  :global(.menu) {
    z-index: 50;
    min-width: 200px;
    padding: var(--space-1);
    border: 1px solid var(--border);
    border-radius: var(--radius-md);
    background: var(--bg-elevated);
    box-shadow: var(--shadow);
    font-size: var(--text-md);
    outline: none;
  }

  :global(.menu-item) {
    padding: 5px var(--space-3);
    border-radius: var(--radius-sm);
    cursor: default;
    outline: none;
    user-select: none;
  }

  :global(.menu-item[data-highlighted]) {
    background: var(--accent);
    color: var(--accent-text);
  }

  :global(.menu-item.danger:not([data-highlighted])) {
    color: var(--danger);
  }

  :global(.menu-separator) {
    height: 1px;
    margin: var(--space-1) 0;
    background: var(--border);
  }
</style>
