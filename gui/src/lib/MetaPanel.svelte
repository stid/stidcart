<script lang="ts">
  // The selected entry's metadata (D-054): five fields, each change one `setMeta` edit, which the
  // Rust side validates in `edit`'s words. On a view-only cart they are only shown.
  import EditableText from './EditableText.svelte';
  import type { Edit, MetaField, Row } from './types';

  let {
    row,
    editable,
    onedit,
  }: {
    row: Row;
    editable: boolean;
    onedit: (e: Edit) => void;
  } = $props();

  const FIELDS: { field: MetaField; label: string }[] = [
    { field: 'title', label: 'Title' },
    { field: 'year', label: 'Year' },
    { field: 'author', label: 'Author' },
    { field: 'note', label: 'Note' },
    { field: 'folder', label: 'Folder' },
  ];
</script>

<section class="meta" aria-label={`Details of entry ${row.index}`}>
  <p class="heading"><span class="faint">#{row.index}</span> {row.name}</p>
  <dl>
    {#each FIELDS as f (f.field)}
      <div class="field" data-field={f.field}>
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
  </dl>
</section>

<style>
  .meta {
    flex: none;
    padding: var(--space-3) var(--space-6) var(--space-4);
    border-top: 1px solid var(--border);
    background: var(--bg-subtle);
  }

  .heading {
    margin: 0 0 var(--space-2);
    font-size: var(--text-sm);
    font-weight: 500;
  }

  dl {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(12em, 1fr));
    gap: var(--space-2) var(--space-5);
    margin: 0;
  }

  .field[data-field='title'],
  .field[data-field='note'] {
    grid-column: span 2;
  }

  dt {
    font-size: var(--text-xs);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-faint);
  }

  dd {
    margin: 2px 0 0;
    font-size: var(--text-md);
    user-select: text;
  }

  .faint {
    color: var(--text-faint);
  }
</style>
