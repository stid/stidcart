<script lang="ts">
  import type { Listing } from './types';

  let { listing, file }: { listing: Listing; file: string } = $props();

  const count = $derived(`${listing.rows.length} ${listing.rows.length === 1 ? 'entry' : 'entries'}`);
</script>

<section class="listing">
  <header class="summary">
    <div class="names">
      <p class="file">{file}</p>
      <h1 class="cart">{listing.cart}</h1>
      <p class="built">built by <span>{listing.builtBy}</span></p>
    </div>
    <dl class="stats">
      <div>
        <dt>Entries</dt>
        <dd>{count}</dd>
      </div>
      <div>
        <dt>Free</dt>
        <dd class="mono">{listing.freeText}</dd>
      </div>
    </dl>
  </header>

  {#if listing.rows.length === 0}
    <p class="none">No entries</p>
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
          </tr>
        </thead>
        <tbody>
          {#each listing.rows as row (row.index)}
            <tr>
              <td class="num faint">{row.index}</td>
              <td class="name">{row.name}</td>
              <td>
                {#if row.flags === '-'}<span class="faint">-</span>{:else}{#each row.flags.split('') as f (f)}<span
                      class="flag"
                      title={f === 'H' ? 'Hidden' : f === 'A' ? '64 KB aligned' : 'Subdirectory'}>{f}</span
                    >{/each}{/if}
              </td>
              <td><span class="kind">{row.kind}</span></td>
              <td class="num mono">{row.bank}</td>
              <td class="mono muted">{row.layout}</td>
              <td class="num mono">{row.size}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</section>

<style>
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

  .built {
    margin: var(--space-1) 0 0;
    font-size: var(--text-sm);
    color: var(--text-muted);
  }

  .built span {
    color: var(--text);
    user-select: text;
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
