<script lang="ts">
  import EditableText from './EditableText.svelte';
  import { ContextMenu } from 'bits-ui';
  import EntryMenu from './EntryMenu.svelte';
  import FolderMenu from './FolderMenu.svelte';
  import MetaPanel from './MetaPanel.svelte';
  import { dragRow, type DragSource, type Drop } from './rowDrag';
  import {
    addFolder,
    buildTree,
    deleteFolder,
    untitledName,
    canStep,
    indexAfter,
    moveEntry,
    moveFolder,
    newFolder as newFolderWithEntry,
    renameFolder,
    sameTree,
    stepEntry,
    stepFolder,
    toArrange,
    type EntryMove,
    type EntryTarget,
    type FolderTarget,
    type Tree,
  } from './tree';
  import type { Edit, Listing, Row, RowActions } from './types';

  let {
    listing,
    file,
    readOnly = null,
    menuReplaceable = false,
    rows = [],
    onedit,
    onreplace,
    onadd,
  }: {
    listing: Listing;
    file: string;
    /** Why the cart cannot be edited; the viewer shows it and offers no edits. */
    readOnly?: string | null;
    /** On a view-only cart, the menu can still be chosen: that replaces an unknown loader (D-050). */
    menuReplaceable?: boolean;
    /** What each row may offer, from the Rust side. */
    rows?: RowActions[];
    /** Given (with `onreplace`), the listing offers edits. Without it, it only shows. */
    onedit?: (e: Edit) => void | Promise<boolean>;
    onreplace?: (index: number) => void;
    /** Given, the empty space's menu also offers Add… and Add Aligned…. */
    onadd?: (aligned: boolean) => void;
  } = $props();

  /** The menus a cart can boot into, by the names the listing shows (D-042). */
  const MENUS = ['EasyLoader', 'stidcart'];

  const count = $derived(`${listing.rows.length} ${listing.rows.length === 1 ? 'entry' : 'entries'}`);
  const editable = $derived(onedit !== undefined && readOnly === null);
  const menuPicker = $derived(editable || (onedit !== undefined && menuReplaceable));

  /** The row whose name is being edited, started from its menu's Rename. */
  let renaming = $state<number | null>(null);

  /** The row whose metadata the panel shows, chosen by clicking it (D-054). */
  let selected = $state<number | null>(null);
  const selectedRow = $derived(listing.rows.find((r) => r.index === selected));

  /** An exact byte count, for the hover on a size the listing shows rounded (D-044). */
  function exact(bytes: number): string {
    return `${bytes.toLocaleString('en-US')} bytes`;
  }

  function edit(e: Edit) {
    onedit?.(e);
  }

  /** The cart as our menu shows it: folders first, then the loose entries (D-057, D-060). */
  const tree = $derived(buildTree(listing.rows, listing.folders ?? []));
  /** With any folder, empty ones included, the listing is the folder tree, drawn as cards (D-062,
   * D-063); a cart without lists flat, as before. */
  const hasFolders = $derived(tree.folders.length > 0);
  const folderNames = $derived(tree.folders.map((f) => f.name));
  /** Each row by its entry index, with its position in `listing.rows` (for `rows[i]`). */
  const rowAt = $derived(new Map(listing.rows.map((r, i) => [r.index, { row: r, i }])));
  /** The total column count, for the rows that span the table. The tree has no `#` column (the
   * number sits with the name) and an edge column each side that insets the cards. */
  const columns = $derived((hasFolders ? 8 : 7) + (editable ? 1 : 0));
  /** The columns between the tree's two edges. */
  const inner = $derived(columns - 2);

  /** Folders whose entries are hidden; every folder opens expanded. */
  let collapsed = $state<string[]>([]);
  /** The folder whose name is being edited, started from its menu's Rename. */
  let renamingFolder = $state<string | null>(null);
  /** The entry waiting for the name of the new folder it goes into ("New folder…"). */
  let naming = $state<number | null>(null);
  /** The same entry, kept past the field's close: it ends its edit (clearing `naming`) before it
   * commits. */
  let namingFor = -1;

  function toggle(name: string) {
    collapsed = collapsed.includes(name) ? collapsed.filter((n) => n !== name) : [...collapsed, name];
  }

  /** Send the tree as one edit, unless it is the tree already; the selection follows its entry. */
  async function arrange(next: Tree): Promise<boolean> {
    if (sameTree(next, tree)) return false;
    const was = selected;
    const took = (await onedit?.(toArrange(next))) !== false;
    if (took && was !== null) selected = indexAfter(next, was);
    return took;
  }

  /** New Folder, as a file browser has it (D-063): an empty folder after the others, its name open
   * for typing once the cart has it. The window's button, File ▸ New Folder and the empty space's
   * menu all come here. */
  export async function newFolder() {
    if (!editable) return;
    const name = untitledName(tree);
    if (await arrange(addFolder(tree, name))) renamingFolder = name;
  }

  function dropped(source: DragSource, drop: Drop) {
    if (source.kind === 'entry') void arrange(moveEntry(tree, source.index, drop as EntryTarget));
    else void arrange(moveFolder(tree, source.name, drop as FolderTarget));
  }

  function moved(index: number, move: EntryMove) {
    if (move.kind === 'folder') void arrange(moveEntry(tree, index, { into: move.name }));
    else if (move.kind === 'noFolder') void arrange(moveEntry(tree, index, { end: true }));
    else if (move.kind === 'newFolder') naming = namingFor = index;
    else void arrange(stepEntry(tree, index, move.kind === 'up' ? -1 : 1));
  }
</script>

{#snippet cells(row: Row)}
  <!-- In the tree the number sits inside the indent with the name, under its folder (D-062). -->
  {#if !hasFolders}<td class="num faint">{row.index}</td>{/if}
  <td class="name" class:nested={hasFolders && row.folder !== ''} class:in-tree={hasFolders}>
    {#if hasFolders}<span class="idx">{row.index}</span>{/if}{#if editable}
      <EditableText
        value={row.name}
        label={`Name of entry ${row.index}`}
        bind:editing={() => renaming === row.index, (on) => (renaming = on ? row.index : null)}
        oncommit={(name) => edit({ kind: 'rename', index: row.index, name })}
      />
    {:else}
      {row.name}
    {/if}
    {#if row.title !== ''}<span class="title">{row.title}</span>{/if}
  </td>
  <td>
    {#if row.flags === '-'}<span class="faint">-</span>{:else}{#each row.flags.split('') as f (f)}<span
          class="flag"
          title={f === 'H' ? 'Hidden' : f === 'A' ? '64 KB aligned' : 'Subdirectory'}>{f}</span
        >{/each}{/if}
  </td>
  <td><span class="kind" data-family={row.family}>{row.kind}</span></td>
  <td class="num mono">{row.bank}</td>
  <td class="mono muted">{row.layout}</td>
  <td class="num mono" title={exact(row.sizeBytes)}>{row.size}</td>
{/snippet}

{#snippet entry(index: number)}
  {@const at = rowAt.get(index)}
  {#if at}
    {@const row = at.row}
    {#if editable && onreplace}
      {@const actions = rows[at.i] ?? { alignable: false }}
      {@const menu = {
        folders: folderNames,
        canUp: canStep(tree, index, -1),
        canDown: canStep(tree, index, 1),
        onmove: (m: EntryMove) => moved(index, m),
      }}
      <EntryMenu {row} {actions} onedit={edit} onrename={(n) => (renaming = n)} {onreplace} {...menu}>
        {#snippet children(props)}
          <tr
            {...props}
            data-entry={row.index}
            data-folder={row.folder || undefined}
            aria-selected={selected === row.index}
            onclick={() => (selected = row.index)}
            {@attach dragRow({ kind: 'entry', index: row.index }, dropped, `⠿  ${row.index}  ${row.name}`)}
          >
            {#if hasFolders}<td class="edge" aria-hidden="true"></td>{/if}
            {@render cells(row)}
            <td class="more-col">
              <EntryMenu {row} {actions} onedit={edit} onrename={(n) => (renaming = n)} {onreplace} {...menu} />
            </td>
            {#if hasFolders}<td class="edge" aria-hidden="true"></td>{/if}
          </tr>
        {/snippet}
      </EntryMenu>
    {:else}
      <tr
        data-entry={row.index}
        data-folder={row.folder || undefined}
        aria-selected={selected === row.index}
        onclick={() => (selected = row.index)}
      >
        {#if hasFolders}<td class="edge" aria-hidden="true"></td>{/if}
        {@render cells(row)}
        {#if hasFolders}<td class="edge" aria-hidden="true"></td>{/if}
      </tr>
    {/if}
  {/if}
{/snippet}

<section class="listing">
  {#if readOnly !== null}
    <p class="banner" role="status">
      <strong>View only.</strong> This cartridge can't be edited: {readOnly}{#if menuPicker}. Choose a
        menu to replace its loader and edit it.{/if}
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
      {#if menuPicker}
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
            <!-- An earlier build of our menu (D-049): shown as what the cart holds, never chosen. -->
            {#if !MENUS.includes(listing.menu)}<option value={listing.menu} disabled>{listing.menu}</option>{/if}
          </select>
        </label>
      {:else}
        <p class="menu">menu <span>{listing.menu}</span></p>
      {/if}
      {#if listing.metaDropped > 0}
        <p class="meta-note">
          {listing.metaDropped} metadata {listing.metaDropped === 1 ? 'record' : 'records'} dropped: no entry
          matches {listing.metaDropped === 1 ? 'it' : 'them'} any more
        </p>
      {/if}
      {#if listing.metaKept !== null}
        <p class="meta-note">Metadata {listing.metaKept}</p>
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

  {#if listing.rows.length === 0 && tree.folders.length === 0}
    <p class="none">No entries{editable ? ' — drop programs or cartridge images here, or use Add…' : ''}</p>
  {:else}
    <div class="body">
    <div class="table-wrap">
      <table class:tree={hasFolders}>
        <thead>
          <tr>
            {#if hasFolders}
              <th class="edge" aria-hidden="true"></th>
              <th class="in-tree"><span class="idx">#</span>Name</th>
            {:else}
              <th class="num">#</th>
              <th>Name</th>
            {/if}
            <th>Flags</th>
            <th>Type</th>
            <th class="num">Bank</th>
            <th>Layout</th>
            <th class="num">Size</th>
            {#if editable}<th class="more-col"><span class="visually-hidden">Actions</span></th>{/if}
            {#if hasFolders}<th class="edge" aria-hidden="true"></th>{/if}
          </tr>
        </thead>
        {#if hasFolders}
          {#each tree.folders as folder, f (folder.name)}
            {@const open = !collapsed.includes(folder.name)}
            <tbody class="gap" aria-hidden="true"><tr><td colspan={columns}></td></tr></tbody>
            <tbody class="card">
              <tr
                class="folder-row"
                data-folder-row={folder.name}
                {@attach editable && dragRow({ kind: 'folder', name: folder.name }, dropped, `▸  ${folder.name}  ${folder.entries.length}`)}
              >
                <td class="edge" aria-hidden="true"></td>
                <td class="name in-tree" colspan={inner - (editable ? 1 : 0)} data-name={folder.name}>
                  <button
                    class="toggle"
                    aria-expanded={open}
                    aria-label={`Folder ${folder.name}`}
                    onclick={() => toggle(folder.name)}>{open ? '▾' : '▸'}</button
                  >
                  {#if editable}
                    <EditableText
                      value={folder.name}
                      label={`Name of folder ${folder.name}`}
                      bind:editing={
                        () => renamingFolder === folder.name, (on) => (renamingFolder = on ? folder.name : null)
                      }
                      oncommit={(name) => void arrange(renameFolder(tree, folder.name, name))}
                    />
                  {:else}
                    <span>{folder.name}</span>
                  {/if}
                  <span class="count">{folder.entries.length}</span>
                </td>
                {#if editable}
                  <td class="more-col">
                    <FolderMenu
                      name={folder.name}
                      canUp={f > 0}
                      canDown={f < tree.folders.length - 1}
                      onrename={(name) => (renamingFolder = name)}
                      onmove={(name, dir) => void arrange(stepFolder(tree, name, dir))}
                      ondelete={(name) => void arrange(deleteFolder(tree, name))}
                    />
                  </td>
                {/if}
                <td class="edge" aria-hidden="true"></td>
              </tr>
              {#if open}
                {#each folder.entries as index (index)}
                  {@render entry(index)}
                {/each}
                {#if folder.entries.length === 0}
                  <!-- An empty folder stays (D-063) and takes a dragged entry here. -->
                  <tr class="empty-drop" data-folder-drop={folder.name}>
                    <td class="edge" aria-hidden="true"></td>
                    <td colspan={inner}><div class="empty-strip">empty · drag entries here</div></td>
                    <td class="edge" aria-hidden="true"></td>
                  </tr>
                {/if}
              {/if}
            </tbody>
          {/each}
          {#if naming !== null}
            <tbody class="gap" aria-hidden="true"><tr><td colspan={columns}></td></tr></tbody>
            <tbody class="card">
              <tr class="folder-row">
                <td class="edge" aria-hidden="true"></td>
                <td class="name in-tree" colspan={inner}>
                  <EditableText
                    value=""
                    label="New folder name"
                    bind:editing={() => naming !== null, (on) => !on && (naming = null)}
                    oncommit={(name) => void arrange(newFolderWithEntry(tree, namingFor, name))}
                  />
                </td>
                <td class="edge" aria-hidden="true"></td>
              </tr>
            </tbody>
          {/if}
          <tbody class="loose">
            {#if tree.loose.length > 0}
              <tr class="loose-label">
                <td class="edge" aria-hidden="true"></td>
                <td class="in-tree" colspan={inner}>// not in a folder</td>
                <td class="edge" aria-hidden="true"></td>
              </tr>
            {/if}
            {#each tree.loose as index (index)}
              {@render entry(index)}
            {/each}
            {#if editable}
              <!-- Shown only while dragging an entry: the top level, after the last row. -->
              <tr class="drop-end" data-drop-end aria-hidden="true">
                <td class="edge"></td>
                <td colspan={inner}><div class="strip">drop here for no folder</div></td>
                <td class="edge"></td>
              </tr>
            {/if}
          </tbody>
        {:else}
          <tbody>
            {#each tree.loose as index (index)}
              {@render entry(index)}
            {/each}
          </tbody>
        {/if}
      </table>
      {#if editable}
        <!-- The list's empty space: a right-click here makes a folder, as in a file browser (D-063). -->
        <ContextMenu.Root>
          <ContextMenu.Trigger>
            {#snippet child({ props })}
              <div class="empty-space" data-empty-space {...props}></div>
            {/snippet}
          </ContextMenu.Trigger>
          <ContextMenu.Portal>
            <ContextMenu.Content class="menu">
              <ContextMenu.Item class="menu-item shortcut-item" onSelect={() => void newFolder()}
                ><span>New Folder</span><span class="keys">⇧⌘N</span></ContextMenu.Item
              >
              {#if onadd}
                <ContextMenu.Separator class="menu-separator" />
                <ContextMenu.Item class="menu-item" onSelect={() => onadd(false)}>Add…</ContextMenu.Item>
                <ContextMenu.Item class="menu-item" onSelect={() => onadd(true)}>Add Aligned…</ContextMenu.Item>
              {/if}
            </ContextMenu.Content>
          </ContextMenu.Portal>
        </ContextMenu.Root>
      {/if}
    </div>
    <MetaPanel row={selectedRow ?? null} {editable} onedit={edit} folders={folderNames} />
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

  /* The list and the inspector side by side, each scrolling on its own (D-063). */
  .body {
    flex: 1;
    min-height: 0;
    display: flex;
  }

  .table-wrap {
    flex: 1;
    min-width: 0;
    min-height: 0;
    display: flex;
    flex-direction: column;
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

  /* The colours our C64 menu gives each kind's badge, tuned per theme for contrast (D-045). */
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

  .title {
    display: block;
    font-size: var(--text-sm);
    font-weight: 400;
    color: var(--text-muted);
  }

  .meta-note {
    margin: var(--space-1) 0 0;
    font-size: var(--text-sm);
    color: var(--text-muted);
  }

  /* The folder tree as grouped cards (D-062): each folder a stid card on the mantle surface, a
     hairline border and --radius-lg; an edge column each side insets the cards from the window. */
  table.tree {
    border-collapse: separate;
    border-spacing: 0;
  }

  .tree .edge {
    width: var(--space-4);
    min-width: var(--space-4);
    padding: 0;
    border-bottom: none;
  }

  .tree thead .edge {
    border-bottom: 1px solid var(--border);
  }

  /* Not the selected row: this outranks the selection rule and would wipe its highlight (#68). */
  .tree tbody tr:not([aria-selected='true']):hover {
    background: none;
  }

  .empty-space {
    min-height: var(--space-8);
    flex: 1;
  }

  .empty-strip {
    margin: var(--space-1) 0 var(--space-3) calc(var(--space-4) + var(--space-5));
    height: 34px;
    display: grid;
    place-items: center;
    border: 1px dashed var(--border-strong);
    border-radius: var(--radius-md);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    color: var(--text-faint);
  }

  .empty-drop td {
    padding: 0 var(--space-3) 0 0;
  }

  .empty-drop[data-drop] .empty-strip {
    border-color: var(--accent);
    background: var(--accent-soft);
    color: var(--text);
  }

  :global(.shortcut-item) {
    display: flex;
    justify-content: space-between;
    gap: var(--space-5);
  }

  :global(.keys) {
    font-family: var(--font-mono);
    color: var(--text-faint);
  }

  :global(.menu-item[data-highlighted] .keys) {
    color: inherit;
  }

  .tree .gap td {
    height: var(--space-3);
    padding: 0;
    border: none;
  }

  .card td:not(.edge) {
    background: var(--bg-subtle);
  }

  .card tr:first-child td:not(.edge) {
    border-top: 1px solid var(--border);
  }

  .card td.edge + td {
    border-left: 1px solid var(--border);
  }

  .card td:nth-last-child(2) {
    border-right: 1px solid var(--border);
  }

  .card tr:first-child td.edge + td {
    border-top-left-radius: var(--radius-lg);
  }

  .card tr:first-child td:nth-last-child(2) {
    border-top-right-radius: var(--radius-lg);
  }

  .card tr:last-child td.edge + td {
    border-bottom-left-radius: var(--radius-lg);
  }

  .card tr:last-child td:nth-last-child(2) {
    border-bottom-right-radius: var(--radius-lg);
  }

  .card tr:hover td:not(.edge) {
    box-shadow: inset 0 0 0 999px var(--bg-hover);
  }

  .card tr[aria-selected='true'] td:not(.edge) {
    box-shadow: inset 0 0 0 999px var(--accent-soft);
  }

  /* The number travels with the name, inside the indent: under its folder when nested. */
  .in-tree {
    padding-left: var(--space-4);
  }

  td.in-tree.nested {
    padding-left: calc(var(--space-4) + var(--space-5));
  }

  .idx {
    display: inline-block;
    width: var(--space-6);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-weight: 400;
    color: var(--text-faint);
  }

  .in-tree .title {
    padding-left: var(--space-6);
  }

  .folder-row td.name {
    font-weight: 600;
  }

  .toggle {
    all: unset;
    cursor: default;
    width: 14px;
    margin-right: var(--space-2);
    text-align: center;
    font-family: var(--font-mono);
    color: var(--accent);
  }

  .toggle:focus-visible {
    outline: 2px solid var(--accent);
  }

  .count {
    margin-left: var(--space-2);
    padding: 1px 6px;
    border-radius: var(--radius-sm);
    background: var(--badge-bg);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    font-weight: 400;
    color: var(--text-muted);
  }

  .loose-label td {
    padding-top: var(--space-5);
    padding-bottom: var(--space-2);
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--text-faint);
  }

  .loose-label td.edge {
    border-bottom: none;
  }

  /* Dragging (D-061, D-062). The end strip stands for "no folder" and shows only while an entry is
     dragged; the source fades; the target lights up. */
  .drop-end {
    display: none;
  }

  :global(body[data-dragging='entry']) .drop-end {
    display: table-row;
  }

  .drop-end td {
    padding: var(--space-3) 0 0;
    border: none;
  }

  .strip {
    height: 40px;
    display: grid;
    place-items: center;
    border: 1px dashed var(--border-strong);
    border-radius: var(--radius-lg);
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    color: var(--text-faint);
  }

  .drop-end[data-drop] .strip {
    border-color: var(--accent);
    background: var(--accent-soft);
    color: var(--text);
  }

  :global(body[data-dragging]) {
    cursor: grabbing;
    user-select: none;
  }

  :global(body[data-dragging='entry']) tr[data-drop='before'] td:not(.edge) {
    box-shadow: inset 0 2px 0 var(--accent);
  }

  :global(body[data-dragging='entry']) tr[data-drop='after'] td:not(.edge) {
    box-shadow: inset 0 -2px 0 var(--accent);
  }

  /* An entry over a folder's header goes into it: the whole card lights up. */
  .card:has(tr[data-drop='into']) td:not(.edge) {
    box-shadow: inset 0 0 0 999px var(--accent-soft);
  }

  .card:has(tr[data-drop='into']) tr:first-child td:not(.edge) {
    border-top-color: var(--accent);
  }

  .card:has(tr[data-drop='into']) tr:last-child td:not(.edge) {
    border-bottom-color: var(--accent);
  }

  .card:has(tr[data-drop='into']) td.edge + td {
    border-left-color: var(--accent);
  }

  .card:has(tr[data-drop='into']) td:nth-last-child(2) {
    border-right-color: var(--accent);
  }

  tr[data-drop='into'] td.name::after {
    content: '→ move into ' attr(data-name);
    float: right;
    font-family: var(--font-mono);
    font-size: var(--text-sm);
    font-weight: 400;
    color: var(--text-muted);
  }

  /* A folder lands before or after a whole card: the line runs along the card's edge. */
  :global(body[data-dragging='folder']) .card:has(tr[data-drop='before']) tr:first-child td:not(.edge) {
    box-shadow: 0 -4px 0 -2px var(--accent);
  }

  :global(body[data-dragging='folder']) .card:has(tr[data-drop='after']) tr:last-child td:not(.edge) {
    box-shadow: 0 4px 0 -2px var(--accent);
  }

  tr[data-drag-source] td:not(.edge) {
    opacity: 0.35;
  }

  .card:has(tr.folder-row[data-drag-source]) td:not(.edge) {
    opacity: 0.35;
    border-style: dashed;
  }

  /* The lifted copy that follows the pointer; rowDrag places it, and it never takes the pointer. */
  :global(.drag-ghost) {
    position: fixed;
    z-index: 100;
    pointer-events: none;
    padding: var(--space-2) var(--space-3);
    background: var(--bg-elevated);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow);
    font-weight: 500;
    white-space: pre;
    transform: rotate(-1deg);
  }

  tbody tr[aria-selected='true'] {
    background: var(--accent-soft);
  }

  .none {
    padding: var(--space-6);
    color: var(--text-muted);
  }
</style>
