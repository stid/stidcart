<script lang="ts">
  // The hex viewer's window: the open cart's bytes, read-only. Every value shown (addresses, the
  // text column, who owns a byte) comes from the Rust side; this draws the rows in view and asks
  // for no others, since the image is 65536 rows long.
  import { onMount, tick } from 'svelte';
  import { hex, owners } from './hex';
  import type { HexHost, HexInfo, HexLine, HexRegion } from './types';

  let { host }: { host: HexHost } = $props();

  /** A row's height in px; the scroll position is counted in these. */
  const ROW = 20;
  /** Rows drawn beyond each edge of the view, so a scroll shows bytes before the next answer. */
  const OVERSCAN = 8;

  let info = $state<HexInfo | null>(null);
  let problem = $state<string | null>(null);
  /** The first row in view. */
  let top = $state(0);
  let height = $state(0);
  let shown = $state<{ first: number; rows: HexLine[] }>({ first: 0, rows: [] });
  /** Bumped when the cart changes, so the rows in view are asked for again. */
  let version = $state(0);
  let pointed = $state<string | null>(null);
  let scroller = $state<HTMLDivElement | null>(null);

  const first = $derived(Math.max(0, top - OVERSCAN));
  // Without layout (tests) the view has no height; assume a window's worth.
  const count = $derived(Math.ceil((height || 600) / ROW) + 2 * OVERSCAN);

  // Measured by hand, not `bind:clientHeight`: that needs a ResizeObserver, and the only thing that
  // resizes this view is its window.
  function measure() {
    height = scroller?.clientHeight ?? 0;
  }
  $effect(measure);

  let request = 0;
  $effect(() => {
    if (!info) return;
    void version;
    const from = first;
    const mine = ++request;
    host.lines(from, count).then(
      (rows) => {
        if (mine === request) shown = { first: from, rows };
      },
      (e) => {
        if (mine === request) problem = String(e);
      },
    );
  });

  /** Each drawn row with the region owning each of its bytes. */
  const drawn = $derived.by(() => {
    const view = info;
    if (!view) return [];
    return shown.rows.map((line, i) => {
      const row = shown.first + i;
      return { row, line, owned: owners(view.regions, row * view.lineBytes, line.bytes.length) };
    });
  });

  const topLine = $derived(drawn.find((d) => d.row === top)?.line ?? drawn[0]?.line);

  /** The first region of each owner, in the image's order: where the jump list goes. */
  const targets = $derived.by(() => {
    const seen = new Set<string>();
    return (info?.regions ?? [])
      .map((region, index) => ({ region, index }))
      .filter(({ region }) => !seen.has(region.label) && seen.add(region.label));
  });

  /** The reserved areas present, for the key. Entries wear their listing colours. */
  const key = $derived(targets.map((t) => t.region).filter((r) => r.kind !== 'entry'));

  async function refresh() {
    try {
      info = await host.info();
      problem = null;
      version++;
    } catch (e) {
      info = null;
      problem = String(e);
      shown = { first: 0, rows: [] };
    }
  }

  async function jump(region: HexRegion | undefined) {
    if (!info || !region) return;
    top = Math.floor(region.start / info.lineBytes);
    await tick();
    if (scroller) scroller.scrollTop = top * ROW;
  }

  async function goto(entry: number | null) {
    await refresh();
    if (entry !== null) await jump(info?.regions.find((r) => r.entry === entry));
  }

  onMount(() => {
    host.onChanged(() => void refresh());
    host.onGoto((entry) => void goto(entry));
    void refresh().then(() => jump(info?.regions.find((r) => r.entry === info?.selected && r.entry !== null)));
  });

  function where(line: HexLine, column: number): string {
    return `${hex(line.bank, 2)} ${line.half} $${hex(line.addr + column, 4)}`;
  }

  function point(e: PointerEvent) {
    const el = (e.target as HTMLElement).closest<HTMLElement>('[data-byte]');
    if (!el || !info) return;
    const d = drawn[Number(el.dataset.i)];
    const column = Number(el.dataset.byte);
    if (!d) return;
    const region = d.owned[column];
    pointed = `${where(d.line, column)} · ${region === null ? 'unused' : info.regions[region].label}`;
  }
</script>

<svelte:window onresize={measure} />

<div class="hex">
  {#if info}
    <header>
      <span class="file">{info.file}</span>
      {#if topLine}<span class="bank">bank {hex(topLine.bank, 2)} ROM{topLine.half}</span>{/if}
      <select
        aria-label="Jump to"
        value=""
        onchange={(e) => {
          void jump(info?.regions[Number(e.currentTarget.value)]);
          e.currentTarget.value = '';
        }}
      >
        <option value="" disabled>Jump to…</option>
        {#each targets as { region, index } (index)}
          <option value={String(index)}>{region.label}</option>
        {/each}
      </select>
    </header>

    <div
      class="scroller"
      bind:this={scroller}
      onscroll={(e) => (top = Math.floor(e.currentTarget.scrollTop / ROW))}
    >
      <div class="rows" style:height="{info.lines * ROW}px" onpointerover={point} role="presentation">
        <div class="window" style:transform="translateY({shown.first * ROW}px)">
          {#each drawn as { row, line, owned }, i (row)}
            <div class="row" data-row={row} style:height="{ROW}px">
              <span class="where">{hex(line.bank, 2)} {line.half}</span>
              <span class="addr">${hex(line.addr, 4)}</span>
              <span class="bytes">
                {#each line.bytes as byte, column (column)}
                  {@const region = owned[column] === null ? null : info.regions[owned[column]]}
                  <span
                    class="byte"
                    data-byte={column}
                    data-i={i}
                    data-owner={region?.kind}
                    data-family={region?.family ?? undefined}
                    data-entry={region?.entry ?? undefined}
                    data-start={region && region.start === row * info.lineBytes + column ? '' : undefined}
                    >{hex(byte, 2)}</span
                  >
                {/each}
              </span>
              <span class="text">{line.text}</span>
            </div>
          {/each}
        </div>
      </div>
    </div>

    <footer>
      <span class="pointed" role="status">{pointed ?? 'Point at a byte to see what owns it'}</span>
      <span class="key">
        {#each key as region (region.kind)}
          <span class="swatch" data-owner={region.kind}>{region.label}</span>
        {/each}
      </span>
    </footer>
  {:else if problem}
    <p class="none">{problem === 'no cartridge is open' ? 'No cartridge is open.' : problem}</p>
  {/if}
</div>

<style>
  .hex {
    display: flex;
    flex-direction: column;
    height: 100%;
    font-family: var(--font-mono);
    font-size: var(--text-sm);
  }

  header,
  footer {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    flex: none;
    padding: var(--space-2) var(--space-4);
    background: var(--bg-subtle);
  }

  header {
    border-bottom: 1px solid var(--border);
  }

  footer {
    justify-content: space-between;
    border-top: 1px solid var(--border);
    color: var(--text-muted);
    font-size: var(--text-xs);
  }

  .file {
    font-weight: 600;
  }

  .bank {
    color: var(--text-muted);
  }

  select {
    margin-left: auto;
    max-width: 16rem;
    padding: var(--space-1) var(--space-2);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-sm);
    background: var(--bg-elevated);
    color: inherit;
    font: inherit;
  }

  .scroller {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
  }

  .rows {
    position: relative;
  }

  .row {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: 0 var(--space-4);
    white-space: pre;
  }

  .where {
    color: var(--text-faint);
  }

  .addr {
    color: var(--text-muted);
  }

  .bytes {
    display: flex;
  }

  .byte {
    padding: 0 3px;
    line-height: 18px;
  }

  /* A wider gap after the eighth byte, as hex dumps are read. */
  .byte:nth-child(9) {
    margin-left: var(--space-2);
  }

  .byte:hover {
    outline: 1px solid var(--accent);
    outline-offset: -1px;
  }

  /* Where an owner's run begins. */
  .byte[data-start] {
    box-shadow: inset 2px 0 0 var(--text-faint);
  }

  /* Entries wear the colour the listing gives their kind (D-045). */
  .byte[data-owner='entry'] {
    background: var(--badge-bg);
  }

  .byte[data-family='prg'] {
    background: var(--kind-prg-bg);
  }

  .byte[data-family='8k'] {
    background: var(--kind-8k-bg);
  }

  .byte[data-family='16k'] {
    background: var(--kind-16k-bg);
  }

  .byte[data-family='ult'] {
    background: var(--kind-ult-bg);
  }

  .byte[data-family='xb'] {
    background: var(--kind-xb-bg);
  }

  /* The reserved areas, each a wash of one raster band. */
  [data-owner='loader'] {
    background: color-mix(in srgb, var(--band-blue) 16%, transparent);
  }

  [data-owner='directory'] {
    background: color-mix(in srgb, var(--band-peach) 18%, transparent);
  }

  [data-owner='eapi'] {
    background: color-mix(in srgb, var(--band-yellow) 22%, transparent);
  }

  [data-owner='name'] {
    background: color-mix(in srgb, var(--band-green) 20%, transparent);
  }

  [data-owner='boot'] {
    background: color-mix(in srgb, var(--band-red) 14%, transparent);
  }

  [data-owner='meta'] {
    background: var(--badge-bg);
  }

  .text {
    color: var(--text-muted);
  }

  .key {
    display: flex;
    gap: var(--space-2);
  }

  .swatch {
    padding: 0 var(--space-2);
    border-radius: var(--radius-sm);
    color: var(--text);
  }

  .none {
    margin: auto;
    color: var(--text-muted);
    font-family: var(--font-ui);
    font-size: var(--text-md);
  }
</style>
