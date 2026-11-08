<script lang="ts">
  import EditableText from './EditableText.svelte';
  import EntryMenu from './EntryMenu.svelte';
  import type { Edit, Listing, Row, RowActions } from './types';

  let {
    listing,
    file,
    readOnly = null,
    rows = [],
    onedit,
    onreplace,
  }: {
    listing: Listing;
    file: string;
    /** Why the cart cannot be edited; the viewer shows it and offers no edits. */
    readOnly?: string | null;
    /** What each row may offer, from the Rust side. */
    rows?: RowActions[];
    /** Given (with `onreplace`), the listing offers edits. Without it, it only shows. */
    onedit?: (e: Edit) => void;
    onreplace?: (index: number) => void;
  } = $props();

  /** The menus a cart can boot into, by the names the listing shows (D-042). */
  const MENUS = ['EasyLoader', 'stidcart'];

  const count = $derived(`${listing.rows.length} ${listing.rows.length === 1 ? 'entry' : 'entries'}`);
  const editable = $derived(onedit !== undefined && readOnly === null);

  /** The row whose name is being edited, started from its menu's Rename. */
  let renaming = $state<number | null>(null);

  /** An exact byte count, for the hover on a size the listing shows rounded (D-044). */
  function exact(bytes: number): string {
    return `${bytes.toLocaleString('en-US')} bytes`;
  }

  function edit(e: Edit) {
    onedit?.(e);
  }
</script>

{#snippet cells(row: Row)}
  <td class="num faint">{row.index}</td>
  <td class="name">
    {#if editable}
      <EditableText
        value={row.name}
        label={`Name of entry ${row.index}`}
        bind:editing={() => renaming === row.index, (on) => (renaming = on ? row.index : null)}
        oncommit={(name) => edit({ kind: 'rename', index: row.index, name })}
      />
    {:else}
      {row.name}
    {/if}
  </td>
  <td>
    {#if row.flags === '-'}<span class="faint">-</span>{:else}{#each row.flags.split('') as f (f)}<span
          class="flag"
          title={f === 'H' ? 'Hidden' : f === 'A' ? '64 KB aligned' : 'Subdirectory'}>{f}</span
        >{/each}{/if}
  </td>
  <td><span class="kind">{row.kind}</span></td>
  <td class="num mono">{row.bank}</td>
  <td class="mono muted">{row.layout}</td>
  <td class="num mono" title={exact(row.sizeBytes)}>{row.size}</td>
{/snippet}

<section class="listing">
  {#if readOnly !== null}
    <p class="banner" role="status">
      <strong>View only.</strong> This cartridge can't be edited: {readOnly}
    </p>
  {/if}

  <header class="summary">
    <div class="names">
      <p class="file">{file}</p>
      <h1 class="cart">
        {#if editable}
          <EditableText
            value={listing.cart}
            label="Cart name"
            oncommit={(name) => edit({ kind: 'setCartName', name })}
          />
        {:else}
          {listing.cart}
        {/if}
      </h1>
      <p class="built">
        built by <span
          >{#if editable}<EditableText
              value={listing.builtBy}
              label="Header name"
              oncommit={(name) => edit({ kind: 'setHeaderName', name })}
            />{:else}{listing.builtBy}{/if}</span
        >
      </p>
      {#if editable}
        <label class="menu">
          menu
          <select
            value={listing.menu}
            onchange={(e) => {
              const menu = e.currentTarget.value;
              // The listing changes only once the cart has the new menu; a refused switch leaves it,
              // so the choice goes back to the menu the cart has until then.
              e.currentTarget.value = listing.menu;
              edit({ kind: 'setMenu', menu });
            }}
          >
            {#each MENUS as m (m)}<option value={m}>{m}</option>{/each}
          </select>
        </label>
      {:else}
        <p class="menu">menu <span>{listing.menu}</span></p>
      {/if}
    </div>
    <dl class="stats">
      <div>
        <dt>Entries</dt>
        <dd>{count}</dd>
      </div>
      <div>
        <dt>Free</dt>
        <dd class="mono" title={exact(listing.free)}>{listing.freeText}</dd>
      </div>
    </dl>
  </header>

  {#if listing.rows.length === 0}
    <p class="none">No entries{editable ? ' — drop programs or cartridge images here, or use Add…' : ''}</p>
  {:else}
    <div class="table-wrap">
      <table>
        <thead>
          <tr>
            <th class="num">#</th>
            <th>Name</th>
            <th>Flags</th>
            <th>Type</th>
            <th class="num">Bank</th>
            <th>Layout</th>
            <th class="num">Size</th>
            {#if editable}<th class="more-col"><span class="visually-hidden">Actions</span></th>{/if}
          </tr>
        </thead>
        <tbody>
          {#each listing.rows as row, i (row.index)}
            {#if editable && onreplace}
              {@const actions = rows[i] ?? { alignable: false }}
              <EntryMenu {row} {actions} onedit={edit} onrename={(n) => (renaming = n)} {onreplace}>
                {#snippet children(props)}
                  <tr {...props}>
                    {@render cells(row)}
                    <td class="more-col">
                      <EntryMenu {row} {actions} onedit={edit} onrename={(n) => (renaming = n)} {onreplace} />
                    </td>
                  </tr>
                {/snippet}
              </EntryMenu>
            {:else}
              <tr>{@render cells(row)}</tr>
            {/if}
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</section>

<style>
  .banner {
    margin: 0;
    padding: var(--space-2) var(--space-6);
    border-bottom: 1px solid var(--border);
    background: var(--danger-soft);
    font-size: var(--text-sm);
    color: var(--text);
  }

  .more-col {
    width: 1%;
    text-align: right;
  }

  .visually-hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }

  .listing {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
  }

  .summary {
    display: flex;
    align-items: flex-end;
    justify-content: space-between;
    gap: var(--space-5);
    padding: var(--space-5) var(--space-6) var(--space-4);
  }

  .file {
    margin: 0 0 var(--space-1);
    font-size: var(--text-sm);
    color: var(--text-faint);
  }

  .cart {
    margin: 0;
    font-size: var(--text-xl);
    font-weight: 600;
    letter-spacing: -0.01em;
    user-select: text;
  }

  .built,
  .menu {
    margin: var(--space-1) 0 0;
    font-size: var(--text-sm);
    color: var(--text-muted);
  }

  .built span,
  .menu span {
    color: var(--text);
    user-select: text;
  }

  label.menu {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .menu select {
    font: inherit;
    color: var(--text);
    background: var(--bg-subtle);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    padding: 2px var(--space-1);
    cursor: pointer;
  }

  .menu select:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  .stats {
    display: flex;
    gap: var(--space-5);
    margin: 0;
  }

  .stats dt {
    font-size: var(--text-xs);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-faint);
  }

  .stats dd {
    margin: 2px 0 0;
    font-size: var(--text-md);
    font-weight: 500;
  }

  .table-wrap {
    flex: 1;
    min-height: 0;
    overflow: auto;
    border-top: 1px solid var(--border);
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-variant-numeric: tabular-nums;
  }

  th {
    position: sticky;
    top: 0;
    z-index: 1;
    padding: var(--space-2) var(--space-3);
    background: var(--bg-subtle);
    border-bottom: 1px solid var(--border);
    font-size: var(--text-xs);
    font-weight: 500;
    text-align: left;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-faint);
  }

  td {
    padding: 7px var(--space-3);
    border-bottom: 1px solid var(--border);
    white-space: nowrap;
  }

  th:first-child,
  td:first-child {
    padding-left: var(--space-6);
  }

  th:last-child,
  td:last-child {
    padding-right: var(--space-6);
  }

  tbody tr {
    transition: background var(--duration) var(--ease);
  }

  tbody tr:hover {
    background: var(--bg-hover);
  }

  .num {
    text-align: right;
  }

  .mono {
    font-family: var(--font-mono);
    font-size: var(--text-sm);
  }

  .muted {
    color: var(--text-muted);
  }

  .faint {
    color: var(--text-faint);
  }

  .name {
    font-weight: 500;
    width: 40%;
    user-select: text;
  }

  .kind {
    display: inline-block;
    padding: 1px 7px;
    border-radius: var(--radius-sm);
    background: var(--badge-bg);
    color: var(--badge-text);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-weight: 500;
  }

  .flag {
    display: inline-grid;
    place-items: center;
    width: 18px;
    height: 18px;
    margin-right: 2px;
    border-radius: var(--radius-sm);
    background: var(--accent-soft);
    color: var(--accent);
    font-size: var(--text-xs);
    font-weight: 600;
  }

  .none {
    padding: var(--space-6);
    color: var(--text-muted);
  }
</style>
