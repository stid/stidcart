<script lang="ts">
  // The actions on one folder row: rename it, or move it among the folders without dragging
  // (AC-181, AC-183). Styled by the `.more` / `.menu` / `.menu-item` rules `EntryMenu` declares.
  import { DropdownMenu } from 'bits-ui';

  let {
    name,
    canUp,
    canDown,
    onrename,
    onmove,
  }: {
    name: string;
    canUp: boolean;
    canDown: boolean;
    onrename: (name: string) => void;
    onmove: (name: string, dir: -1 | 1) => void;
  } = $props();
</script>

<DropdownMenu.Root>
  <DropdownMenu.Trigger class="more" aria-label={`Actions for folder ${name}`}>⋯</DropdownMenu.Trigger>
  <DropdownMenu.Portal>
    <DropdownMenu.Content class="menu" align="end" sideOffset={4}>
      <DropdownMenu.Item class="menu-item" onSelect={() => onrename(name)}>Rename</DropdownMenu.Item>
      <DropdownMenu.Item class="menu-item" disabled={!canUp} onSelect={() => onmove(name, -1)}>Move up</DropdownMenu.Item>
      <DropdownMenu.Item class="menu-item" disabled={!canDown} onSelect={() => onmove(name, 1)}
        >Move down</DropdownMenu.Item
      >
    </DropdownMenu.Content>
  </DropdownMenu.Portal>
</DropdownMenu.Root>
