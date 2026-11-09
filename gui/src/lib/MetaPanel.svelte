<script lang="ts">
  // The inspector beside the list (D-063): the selected entry's number, name, kind and size, then
  // its metadata (D-054), each change one `setMeta` edit the Rust side validates in `edit`'s words.
  // The folder is chosen from the cart's folders; choosing one moves no entry (AC-184). On a
  // view-only cart the fields are only shown.
  import EditableText from './EditableText.svelte';
  import type { Edit, MetaField, Row } from './types';

  let {
    row,
    editable,
    onedit,
    folders,
  }: {
    /** The selected entry, or null with nothing selected. */
    row: Row | null;
    editable: boolean;
    onedit: (e: Edit) => void;
    /** The cart's folders in order, for the Folder choice. */
    folders: string[];
  } = $props();

  const FIELDS: { field: Exclude<MetaField, 'folder'>; label: string }[] = [
    { field: 'title', label: 'Title' },
    { field: 'year', label: 'Year' },
    { field: 'author', label: 'Author' },
    { field: 'note', label: 'Note' },
  ];
</script>

<aside class="inspector" aria-label="Entry details">
  {#if row}
    <header>
      <p class="eyebrow">// entry {row.index}</p>
      <p class="name">
        <span>{row.name}</span>
        <span class="kind" data-family={row.family}>{row.kind}</span>
      </p>
      <p class="facts">{row.size} · bank {row.bank}</p>
    </header>
    <dl>
      {#each FIELDS as f (f.field)}
        <div class="field">
          <dt>{f.label}</dt>
          <dd>
            {#if editable}
              <EditableText
                value={row[f.field]}
                label={f.label}
                placeholder="—"
                oncommit={(value) => onedit({ kind: 'setMeta', index: row.index, field: f.field, value })}
              />
            {:else if row[f.field] !== ''}
              {row[f.field]}
            {:else}
              <span class="faint">—</span>
            {/if}
          </dd>
        </div>
      {/each}
      <div class="field">
        <dt><label for="inspector-folder">Folder</label></dt>
        <dd>
          {#if editable}
            <select
              id="inspector-folder"
              value={row.folder}
              onchange={(e) => {
                const value = e.currentTarget.value;
                // Shown as the cart has it until the cart answers with the change.
                e.currentTarget.value = row.folder;
                onedit({ kind: 'setMeta', index: row.index, field: 'folder', value });
              }}
            >
              <option value="">Not in a folder</option>
              {#each folders as name (name)}<option value={name}>{name}</option>{/each}
            </select>
          {:else if row.folder !== ''}
            {row.folder}
          {:else}
            <span class="faint">Not in a folder</span>
          {/if}
        </dd>
      </div>
    </dl>
  {:else}
    <p class="empty">Select an entry to see its details.</p>
  {/if}
</aside>

<style>
  .inspector {
    flex: none;
    width: 280px;
    min-height: 0;
    overflow-y: auto;
    border-left: 1px solid var(--border);
    background: var(--bg-subtle);
  }

  header {
    padding: var(--space-5) var(--space-5) var(--space-4);
    border-bottom: 1px solid var(--border);
  }

  header p {
    margin: 0;
  }

  .eyebrow {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--text-faint);
  }

  .name {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-top: var(--space-1) !important;
    font-size: var(--text-lg);
    font-weight: 600;
    user-select: text;
    overflow-wrap: anywhere;
  }

  .facts {
    margin-top: var(--space-1) !important;
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    color: var(--text-muted);
  }

  .kind {
    flex: none;
    padding: 1px 7px;
    border-radius: var(--radius-sm);
    background: var(--badge-bg);
    color: var(--badge-text);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-weight: 500;
  }

  /* The kinds' badge colours, as the listing shows them (D-045). */
  .kind[data-family='prg'] {
    background: var(--kind-prg-bg);
    color: var(--kind-prg-text);
  }

  .kind[data-family='8k'] {
    background: var(--kind-8k-bg);
    color: var(--kind-8k-text);
  }

  .kind[data-family='16k'] {
    background: var(--kind-16k-bg);
    color: var(--kind-16k-text);
  }

  .kind[data-family='ult'] {
    background: var(--kind-ult-bg);
    color: var(--kind-ult-text);
  }

  .kind[data-family='xb'] {
    background: var(--kind-xb-bg);
    color: var(--kind-xb-text);
  }

  dl {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    margin: 0;
    padding: var(--space-4) var(--space-5);
  }

  dt {
    font-size: var(--text-xs);
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-faint);
  }

  dd {
    margin: var(--space-1) 0 0;
    font-size: var(--text-md);
    user-select: text;
    overflow-wrap: anywhere;
  }

  select {
    width: 100%;
    padding: var(--space-1) var(--space-2);
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg);
    color: var(--text);
    font: inherit;
  }

  select:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  .empty {
    margin: 0;
    padding: var(--space-5);
    color: var(--text-muted);
  }

  .faint {
    color: var(--text-faint);
  }
</style>
