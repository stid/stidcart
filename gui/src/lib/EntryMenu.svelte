<script lang="ts">
  // The actions on one entry, offered two ways from one list: a "⋯" button, or — when given
  // `children` — a right-click menu on whatever `children` renders (the row). Which actions apply
  // comes from the Rust side (`actions`), never from the kind label.
  import { ContextMenu, DropdownMenu } from 'bits-ui';
  import type { Snippet } from 'svelte';
  import type { EntryMove } from './tree';
  import type { Edit, Row, RowActions } from './types';

  let {
    row,
    actions,
    onedit,
    onrename,
    onreplace,
    onmove,
    onhex,
    folders = [],
    canUp = false,
    canDown = false,
    children,
  }: {
    row: Row;
    actions: RowActions;
    onedit: (e: Edit) => void;
    onrename: (index: number) => void;
    onreplace: (index: number) => void;
    /** Given, the menu offers the moves a drag makes: to a folder, out of one, up and down. */
    onmove?: (move: EntryMove) => void;
    /** Given, the menu offers the hex viewer at this entry. */
    onhex?: (index: number) => void;
    /** The cart's folders, in the tree's order. */
    folders?: string[];
    /** Whether the entry can go up or down within its folder, or among the loose entries. */
    canUp?: boolean;
    canDown?: boolean;
    /** Render the row; spread `props` on its root element to make it the right-click target. */
    children?: Snippet<[Record<string, unknown>]>;
  } = $props();

  const hidden = $derived(row.flags.includes('H'));

  // Rename and New folder… open a text field in the listing; the menu handing focus back to its
  // trigger as it closes would blur that field and end the edit before it starts.
  let toField = false;
  function closeFocus(e: Event) {
    if (toField) e.preventDefault();
    toField = false;
  }
  const aligned = $derived(row.flags.includes('A'));
</script>

{#snippet items()}
  <DropdownMenu.Item
    class="menu-item"
    onSelect={() => {
      toField = true;
      onrename(row.index);
    }}>Rename</DropdownMenu.Item
  >
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
  {#if onmove}
    <DropdownMenu.Separator class="menu-separator" />
    <DropdownMenu.Sub>
      <DropdownMenu.SubTrigger class="menu-item">Move to</DropdownMenu.SubTrigger>
      <DropdownMenu.SubContent class="menu">
        <!-- Where the entry is reads as a checked choice, never as a greyed heading (D-063). -->
        <DropdownMenu.RadioGroup value={row.folder}>
          <DropdownMenu.GroupHeading class="menu-label">// folders</DropdownMenu.GroupHeading>
          {#each folders as name (name)}
            <DropdownMenu.RadioItem
              class="menu-item check-item"
              value={name}
              onSelect={() => name !== row.folder && onmove({ kind: 'folder', name })}
            >
              {#snippet children({ checked })}<span class="check" aria-hidden="true">{checked ? '✓' : ''}</span
                >{name}{/snippet}
            </DropdownMenu.RadioItem>
          {/each}
          <DropdownMenu.Separator class="menu-separator" />
          <DropdownMenu.RadioItem
            class="menu-item check-item"
            value=""
            onSelect={() => row.folder !== '' && onmove({ kind: 'noFolder' })}
          >
            {#snippet children({ checked })}<span class="check" aria-hidden="true">{checked ? '✓' : ''}</span
              >Not in a folder{/snippet}
          </DropdownMenu.RadioItem>
        </DropdownMenu.RadioGroup>
        <DropdownMenu.Separator class="menu-separator" />
        <DropdownMenu.Item
          class="menu-item check-item"
          onSelect={() => {
            toField = true;
            onmove({ kind: 'newFolder' });
          }}><span class="check" aria-hidden="true"></span>New Folder with Entry</DropdownMenu.Item
        >
      </DropdownMenu.SubContent>
    </DropdownMenu.Sub>
    <DropdownMenu.Separator class="menu-separator" />
    <DropdownMenu.Item class="menu-item" disabled={!canUp} onSelect={() => onmove({ kind: 'up' })}>Move up</DropdownMenu.Item>
    <DropdownMenu.Item class="menu-item" disabled={!canDown} onSelect={() => onmove({ kind: 'down' })}
      >Move down</DropdownMenu.Item
    >
  {/if}
  {#if onhex}
    <DropdownMenu.Separator class="menu-separator" />
    <DropdownMenu.Item class="menu-item" onSelect={() => onhex(row.index)}>Show in Hex Viewer</DropdownMenu.Item>
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
      <ContextMenu.Content class="menu" onCloseAutoFocus={closeFocus}>
        {@render items()}
      </ContextMenu.Content>
    </ContextMenu.Portal>
  </ContextMenu.Root>
{:else}
  <DropdownMenu.Root>
    <DropdownMenu.Trigger class="more" aria-label={`Actions for ${row.name}`}>⋯</DropdownMenu.Trigger>
    <DropdownMenu.Portal>
      <DropdownMenu.Content class="menu" align="end" sideOffset={4} onCloseAutoFocus={closeFocus}>
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

  /* Disabled is only ever an unavailable command, so it reads dimmer than any label (D-063). */
  :global(.menu-item[data-disabled]) {
    color: var(--text-faint);
    opacity: 0.6;
  }

  :global(.menu-label) {
    padding: var(--space-1) var(--space-3) 2px;
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--text-faint);
  }

  :global(.check-item) {
    display: flex;
    gap: var(--space-2);
  }

  :global(.check) {
    width: 12px;
    color: var(--accent);
  }

  :global(.menu-item[data-highlighted] .check) {
    color: inherit;
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
