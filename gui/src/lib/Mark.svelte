<script lang="ts">
  // The stidcart mark (D-046): a cartridge seen head-on, shell, edge connector and a label of five
  // raster bands. It draws the app icon's shape (src-tauri/icons/icon-source.svg), in Mocha's colours
  // in either theme, because an icon does not change with the appearance.
  let { size = 16 }: { size?: number } = $props();

  const clip = $props.id();
  const BANDS = [
    { name: 'red', fill: '#f38ba8', y: 32, h: 9 },
    { name: 'peach', fill: '#fab387', y: 41, h: 9 },
    { name: 'yellow', fill: '#f9e2af', y: 50, h: 9 },
    { name: 'green', fill: '#a6e3a1', y: 59, h: 9 },
    { name: 'blue', fill: '#89b4fa', y: 68, h: 8 },
  ];
  const PINS = [41, 48, 55, 62, 69, 76, 83];
</script>

<svg class="mark" data-testid="mark" width={size} height={size} viewBox="0 0 128 128" aria-hidden="true">
  <defs>
    <clipPath id={clip}><rect x="30" y="32" width="68" height="44" rx="4" /></clipPath>
  </defs>
  <rect width="128" height="128" rx="28" fill="#1e1e2e" />
  <rect x="0.75" y="0.75" width="126.5" height="126.5" rx="27.25" fill="none" stroke="#313244" stroke-width="1.5" />
  <rect x="36" y="84" width="56" height="20" rx="3" fill="#313244" />
  {#each PINS as x (x)}<rect {x} y="92" width="4" height="9" rx="1" fill="#f9e2af" opacity="0.75" />{/each}
  <rect x="22" y="24" width="84" height="64" rx="8" fill="#45475a" />
  <g clip-path={`url(#${clip})`}>
    {#each BANDS as band (band.name)}<rect
        data-band={band.name}
        x="30"
        y={band.y}
        width="68"
        height={band.h}
        fill={band.fill}
      />{/each}
  </g>
</svg>

<style>
  .mark {
    display: block;
    flex: none;
  }
</style>
