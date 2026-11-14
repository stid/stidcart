import { cleanup, fireEvent, render, screen, within } from '@testing-library/svelte';
import { afterEach, expect, test, vi } from 'vitest';
import two from './fixtures/two.json';
import type { Listing as ListingData } from './types';
import Listing from './Listing.svelte';

// Vitest runs without globals here, so testing-library cannot unmount between tests by itself.
afterEach(cleanup);

const listing: ListingData = two;

test('AC-51 (RENDER): every listing value is shown', () => {
  render(Listing, { props: { listing, file: 'two.crt' } });

  expect(screen.getByText('two.crt')).toBeInTheDocument();
  expect(screen.getByText(listing.cart)).toBeInTheDocument();
  expect(screen.getByText(listing.builtBy)).toBeInTheDocument();

  const rows = screen.getAllByRole('row').slice(1); // header row first
  expect(rows).toHaveLength(listing.rows.length);
  listing.rows.forEach((r, i) => {
    const cells = within(rows[i])
      .getAllByRole('cell')
      .map((c) => c.textContent?.trim());
    expect(cells).toEqual([String(r.index), r.name, r.flags, r.kind, r.bank, r.layout, r.size]);
  });

  expect(screen.getByText(/2 entries/)).toBeInTheDocument();
  expect(screen.getByText(new RegExp(listing.freeText.replace(/[$()]/g, '\\$&')))).toBeInTheDocument();
});

test('AC-85 (RENDER): the window shows which menu the cart carries', () => {
  render(Listing, { props: { listing, file: 'two.crt' } });
  expect(screen.getByText('EasyLoader')).toBeInTheDocument();
  cleanup();

  const ours = { ...listing, menu: 'stidcart' };
  render(Listing, { props: { listing: ours, file: 'ours.crt' } });
  expect(screen.getByText('stidcart')).toBeInTheDocument();
});

function editable(extra: Record<string, unknown> = {}) {
  const onedit = vi.fn();
  const onreplace = vi.fn();
  render(Listing, {
    props: {
      listing,
      file: 'two.crt',
      readOnly: null,
      rows: listing.rows.map(() => ({ alignable: false })),
      onedit,
      onreplace,
      ...extra,
    },
  });
  return { onedit, onreplace };
}

test('AC-60 (RENDER): a view-only cart says why and offers no edits', () => {
  const reason = 'reserved region EAPI differs from the embedded firmware, so it cannot be rebuilt unchanged';
  editable({ readOnly: reason });
  expect(screen.getByRole('status')).toHaveTextContent(reason);
  expect(screen.queryByRole('button', { name: /edit cart name/i })).toBeNull();
  expect(screen.queryByRole('button', { name: /actions for/i })).toBeNull();
});

test('AC-69 (RENDER): the cart name and the header name are edited separately', async () => {
  const { onedit } = editable();
  await fireEvent.click(screen.getByRole('button', { name: /edit cart name/i }));
  const cart = screen.getByRole('textbox', { name: /cart name/i });
  await fireEvent.input(cart, { target: { value: 'my menu' } });
  await fireEvent.keyDown(cart, { key: 'Enter' });
  expect(onedit).toHaveBeenLastCalledWith({ kind: 'setCartName', name: 'my menu' });

  await fireEvent.click(screen.getByRole('button', { name: /edit header name/i }));
  const header = screen.getByRole('textbox', { name: /header name/i });
  await fireEvent.input(header, { target: { value: 'hand made' } });
  await fireEvent.keyDown(header, { key: 'Enter' });
  expect(onedit).toHaveBeenLastCalledWith({ kind: 'setHeaderName', name: 'hand made' });
  expect(onedit).toHaveBeenCalledTimes(2);
});

test('AC-71 (RENDER): rows show the bank and layout they are given, after an edit re-placed them', () => {
  const replaced: ListingData = {
    ...listing,
    rows: listing.rows.map((r, i) => ({ ...r, bank: `$0${i + 2}`, layout: `$${1000 + i}` })),
  };
  editable({ listing: replaced });
  const rows = screen.getAllByRole('row').slice(1);
  replaced.rows.forEach((r, i) => {
    expect(within(rows[i]).getByText(r.bank)).toBeInTheDocument();
    expect(within(rows[i]).getByText(r.layout)).toBeInTheDocument();
  });
  expect(screen.getAllByRole('button', { name: /actions for/i })).toHaveLength(replaced.rows.length);
});

test('AC-112 (RENDER): on an editable cart the menu is a choice, and choosing the other is an edit', async () => {
  const { onedit } = editable();
  const picker = screen.getByRole('combobox', { name: /menu/i }) as HTMLSelectElement;
  expect(picker.value).toBe('EasyLoader');
  expect(within(picker).getAllByRole('option').map((o) => o.textContent)).toEqual(['EasyLoader', 'stidcart']);

  await fireEvent.change(picker, { target: { value: 'stidcart' } });
  expect(onedit).toHaveBeenLastCalledWith({ kind: 'setMenu', menu: 'stidcart' });
  expect(onedit).toHaveBeenCalledTimes(1);
});

test('AC-113 (RENDER): until the cart answers with a new menu, the choice shows the one it has', async () => {
  // A refused switch leaves the listing as it was, so the picker must not keep showing the refused menu.
  editable();
  const picker = screen.getByRole('combobox', { name: /menu/i }) as HTMLSelectElement;
  await fireEvent.change(picker, { target: { value: 'stidcart' } });
  expect(picker.value).toBe('EasyLoader');
});

test('AC-114 (RENDER): a view-only cart, or a listing without edits, shows the menu as text', () => {
  editable({ readOnly: 'reserved region EAPI differs from the embedded firmware' });
  expect(screen.queryByRole('combobox')).toBeNull();
  expect(screen.getByText('EasyLoader')).toBeInTheDocument();
  cleanup();

  render(Listing, { props: { listing, file: 'two.crt' } });
  expect(screen.queryByRole('combobox')).toBeNull();
  expect(screen.getByText('EasyLoader')).toBeInTheDocument();
});

test('AC-130 (RENDER): an older build of our menu shows as such, and choosing stidcart is an edit', async () => {
  const { onedit } = editable({ listing: { ...listing, menu: 'stidcart (older build)' } });
  const picker = screen.getByRole('combobox', { name: /menu/i }) as HTMLSelectElement;
  expect(picker.value).toBe('stidcart (older build)');
  const options = within(picker).getAllByRole('option') as HTMLOptionElement[];
  expect(options.map((o) => o.textContent)).toEqual(['EasyLoader', 'stidcart', 'stidcart (older build)']);
  expect(options[2].disabled).toBe(true);

  await fireEvent.change(picker, { target: { value: 'stidcart' } });
  expect(onedit).toHaveBeenLastCalledWith({ kind: 'setMenu', menu: 'stidcart' });
});

test('AC-134 (RENDER): a view-only cart whose loader is unknown offers the menu picker', async () => {
  const { onedit } = editable({
    readOnly: 'reserved region menu loader differs from the embedded firmware',
    menuReplaceable: true,
    listing: { ...listing, menu: 'unrecognised' },
  });
  expect(screen.getByText(/view only/i)).toBeInTheDocument();
  expect(screen.getByText(/choose a menu/i)).toBeInTheDocument();
  const picker = screen.getByRole('combobox', { name: /menu/i }) as HTMLSelectElement;
  expect(picker.value).toBe('unrecognised');
  const options = within(picker).getAllByRole('option') as HTMLOptionElement[];
  expect(options.map((o) => o.textContent)).toEqual(['EasyLoader', 'stidcart', 'unrecognised']);
  expect(options[2].disabled).toBe(true);

  await fireEvent.change(picker, { target: { value: 'EasyLoader' } });
  expect(onedit).toHaveBeenLastCalledWith({ kind: 'setMenu', menu: 'EasyLoader' });
});

test('AC-135 (RENDER): a cart refused for another reason offers no menu picker', () => {
  editable({ readOnly: 'reserved region EAPI differs from the embedded firmware', menuReplaceable: false });
  expect(screen.queryByRole('combobox')).toBeNull();
  expect(screen.queryByText(/choose a menu/i)).toBeNull();
});

test('AC-117 (RENDER): every size and the free figure show their exact byte count on hover', () => {
  render(Listing, { props: { listing, file: 'two.crt' } });
  const exact = (n: number) => `${n.toLocaleString('en-US')} bytes`;

  const rows = screen.getAllByRole('row').slice(1);
  listing.rows.forEach((r, i) => {
    expect(within(rows[i]).getByText(r.size)).toHaveAttribute('title', exact(r.sizeBytes));
  });
  expect(screen.getByText(listing.freeText)).toHaveAttribute('title', exact(listing.free));
});

test('AC-118 (RENDER): each type badge carries its family', () => {
  const families = ['prg', '8k', '16k', 'ult', 'xb'];
  const rows = listing.rows.map((r, i) => ({ ...r, family: families[i % families.length] }));
  render(Listing, { props: { listing: { ...listing, rows }, file: 'two.crt' } });
  const table = screen.getAllByRole('row').slice(1);
  rows.forEach((r, i) => {
    expect(within(table[i]).getByText(r.kind)).toHaveAttribute('data-family', r.family);
  });
});

/** `two.json` with a title and folder on entry 0 only. */
function annotated(): ListingData {
  return {
    ...listing,
    rows: listing.rows.map((r, i) => ({
      ...r,
      title: i === 0 ? 'Hello There' : '',
      folder: i === 0 ? 'Games' : '',
    })),
  };
}

test('AC-158 (RENDER): each entry shows its title under its name, and sits in its folder', () => {
  render(Listing, { props: { listing: annotated(), file: 'two.crt' } });
  // The folder shows as the card the entry sits in (AC-178, D-062), not a column.
  const rows = screen.getAllByRole('row').filter((r) => r.hasAttribute('data-entry'));
  expect(within(rows[0]).getByText('Hello There')).toBeInTheDocument();
  expect(rows[0].closest('tbody')?.querySelector('[data-folder-row]')?.getAttribute('data-folder-row')).toBe('Games');
  expect(within(rows[1]).queryByText('Hello There')).toBeNull();
});

test('AC-158 (RENDER): without metadata there is no folder column and no title line', () => {
  render(Listing, { props: { listing, file: 'two.crt' } });
  expect(screen.queryByRole('columnheader', { name: /folder/i })).toBeNull();
  const rows = screen.getAllByRole('row').slice(1);
  listing.rows.forEach((r, i) => {
    expect(within(rows[i]).getAllByRole('cell')[1].textContent?.trim()).toBe(r.name);
  });
});

test('AC-158 (RENDER): the window says how many records were dropped, and a kept layout', () => {
  render(Listing, {
    props: {
      listing: {
        ...listing,
        metaDropped: 2,
        metaKept: 'layout version 9 is unknown to this stidcart; kept as stored',
      },
      file: 'two.crt',
    },
  });
  expect(screen.getByText(/2 metadata records dropped/i)).toBeInTheDocument();
  expect(screen.getByText(/layout version 9/)).toBeInTheDocument();
});

test('AC-159 (RENDER): selecting an entry offers its five fields, each change a setMeta edit', async () => {
  const { onedit } = editable({ listing: annotated() });
  const rows = screen.getAllByRole('row').filter((r) => r.hasAttribute('data-entry'));
  expect(screen.queryByRole('button', { name: /edit title/i })).toBeNull();

  await fireEvent.click(within(rows[1]).getByText(listing.rows[1].kind));
  expect(rows[1]).toHaveAttribute('aria-selected', 'true');
  expect(rows[0]).not.toHaveAttribute('aria-selected', 'true');

  await fireEvent.click(screen.getByRole('button', { name: /edit title/i }));
  const input = screen.getByRole('textbox', { name: /title/i });
  await fireEvent.input(input, { target: { value: 'Ramp' } });
  await fireEvent.keyDown(input, { key: 'Enter' });
  expect(onedit).toHaveBeenLastCalledWith({
    kind: 'setMeta',
    index: 1,
    field: 'title',
    value: 'Ramp',
  });
});

/** `two` with hello in folder Games and ramp loose, or in the folders given. */
function foldered(folders: string[] = ['Games', '']): ListingData {
  return {
    ...listing,
    rows: listing.rows.map((r, i) => ({ ...r, folder: folders[i] })),
  } as unknown as ListingData;
}

const folderRows = () => screen.getAllByRole('row').filter((r) => r.hasAttribute('data-folder-row'));
const entryRows = () => screen.getAllByRole('row').filter((r) => r.hasAttribute('data-entry'));

test('AC-175 (RENDER): the listing has no Group or Folder column; the tree shows the folder', () => {
  render(Listing, { props: { listing: foldered(), file: 'two.crt' } });
  expect(screen.queryByRole('columnheader', { name: /folder/i })).toBeNull();
  expect(screen.queryByRole('columnheader', { name: /group/i })).toBeNull();
  expect(within(folderRows()[0]).getByText('Games')).toBeInTheDocument();
});

test('AC-178 (RENDER): each folder is a card holding its entries, each number inside the indent', () => {
  render(Listing, { props: { listing: foldered(), file: 'two.crt' } });
  const hello = entryRows().find((r) => r.getAttribute('data-entry') === '0')!;
  const card = hello.closest('tbody')!;
  expect(card).toHaveClass('card');
  expect(card.querySelector('[data-folder-row]')).toHaveAttribute('data-folder-row', 'Games');
  // No number column: the number shares the name's cell, indented under the folder.
  expect(screen.queryByRole('columnheader', { name: /^#$/ })).toBeNull();
  const name = within(hello).getByText('0').closest('td')!;
  expect(name).toHaveClass('nested');
  expect(within(name).getByText('hello')).toBeInTheDocument();
  // The loose entry follows under its label, outside every card.
  const ramp = entryRows().find((r) => r.getAttribute('data-entry') === '1')!;
  expect(ramp.closest('tbody')).not.toHaveClass('card');
  expect(screen.getByText('// not in a folder')).toBeInTheDocument();
});

test('AC-178 (RENDER): a cart with folders lists folder rows with their count, then the loose entries', async () => {
  render(Listing, {
    props: { listing: foldered(['Games', 'Games']), file: 'two.crt' },
  });
  const [games] = folderRows();
  expect(within(games).getByText('Games')).toBeInTheDocument();
  expect(within(games).getByText('2')).toBeInTheDocument();
  expect(entryRows().map((r) => r.getAttribute('data-entry'))).toEqual(['0', '1']);

  // Open on start; the folder's toggle hides and shows its entries.
  const toggle = within(games).getByRole('button', { name: /games/i });
  expect(toggle).toHaveAttribute('aria-expanded', 'true');
  await fireEvent.click(toggle);
  expect(toggle).toHaveAttribute('aria-expanded', 'false');
  expect(entryRows()).toHaveLength(0);
  await fireEvent.click(toggle);
  expect(entryRows()).toHaveLength(2);
  cleanup();

  // The loose entry comes after the folder; without folders there are no folder rows.
  render(Listing, {
    props: { listing: foldered(['', 'Games']), file: 'two.crt' },
  });
  expect(
    screen
      .getAllByRole('row')
      .filter((r) => r.hasAttribute('data-folder-row') || r.hasAttribute('data-entry'))
      .map((r) => r.getAttribute('data-folder-row') ?? r.getAttribute('data-entry')),
  ).toEqual(['Games', '1', '0']);
  cleanup();
  render(Listing, { props: { listing, file: 'two.crt' } });
  expect(screen.getAllByRole('row').filter((r) => r.hasAttribute('data-folder-row'))).toHaveLength(0);
});

function pointer(type: string, el: Element, y: number) {
  el.dispatchEvent(new MouseEvent(type, { bubbles: true, clientX: 10, clientY: y, button: 0 }));
}

/** Drag `from` and release over `over`, with layout faked: jsdom has none. The fakes are taken
 * back after, since the menus also measure and hit-test. */
async function drag(from: Element, over: Element) {
  const rect = vi.spyOn(Element.prototype, 'getBoundingClientRect').mockReturnValue({ top: 0, height: 20 } as DOMRect);
  const hitTest = document.elementFromPoint;
  document.elementFromPoint = vi.fn(() => over);
  pointer('pointerdown', from, 5);
  pointer('pointermove', from, 15);
  pointer('pointerup', from, 15);
  document.elementFromPoint = hitTest;
  rect.mockRestore();
  // A drop swallows the click that ends it until the next task; let that pass before going on.
  await new Promise((r) => setTimeout(r, 0));
}

test('AC-179 (RENDER): dropping an entry on a folder row sends one arrange; a drop that changes nothing sends none', async () => {
  const { onedit } = editable({ listing: foldered() });
  const ramp = entryRows().find((r) => r.getAttribute('data-entry') === '1')!;
  await drag(ramp, ramp);
  expect(onedit).not.toHaveBeenCalled();

  await drag(ramp, folderRows()[0]);
  expect(onedit).toHaveBeenCalledExactlyOnceWith({
    kind: 'arrange',
    folders: [{ name: 'Games', entries: [0, 1] }],
    loose: [],
  });
  vi.restoreAllMocks();
});

test('AC-182 (RENDER): New folder… asks for a name inline, and committing moves the entry into it', async () => {
  const { onedit } = editable({ listing: foldered() });
  const trigger = screen.getByRole('button', { name: /actions for ramp/i });
  trigger.focus();
  await fireEvent.keyDown(trigger, { key: 'Enter' });
  const sub = await screen.findByRole('menuitem', { name: /^move to$/i });
  sub.focus();
  await fireEvent.keyDown(sub, { key: 'ArrowRight' });
  // jsdom has no layout, so the menu's positioning takes the trigger inside the scrolling table as
  // clipped and hides the open menus with visibility: hidden, which also blanks their accessible
  // names. They are open all the same, so the item is found by its text.
  await fireEvent.click(await screen.findByText('New Folder with Entry'));

  const input = await screen.findByRole('textbox', { name: /new folder/i });
  await fireEvent.input(input, { target: { value: 'Tools' } });
  await fireEvent.keyDown(input, { key: 'Enter' });
  expect(onedit).toHaveBeenCalledExactlyOnceWith({
    kind: 'arrange',
    folders: [
      { name: 'Games', entries: [0] },
      { name: 'Tools', entries: [1] },
    ],
    loose: [],
  });
});

test('AC-182 (RENDER): the right-click menu offers the same moves, and its other items still work', async () => {
  const { onedit } = editable({ listing: foldered() });
  const ramp = entryRows().find((r) => r.getAttribute('data-entry') === '1')!;
  await fireEvent.contextMenu(ramp);
  // Found by text: jsdom hides menus opened inside the scrolling table (see the test above).
  expect(await screen.findByText('Move to')).toBeInTheDocument();
  await fireEvent.click(screen.getByText('Hide'));
  expect(onedit).toHaveBeenCalledExactlyOnceWith({ kind: 'setHidden', index: 1, hidden: true });

  await fireEvent.contextMenu(ramp);
  const sub = await screen.findByText('Move to');
  sub.focus();
  await fireEvent.keyDown(sub, { key: 'ArrowRight' });
  await fireEvent.click(await screen.findByText('New Folder with Entry'));
  expect(await screen.findByRole('textbox', { name: /new folder/i })).toBeInTheDocument();
});

test('AC-184 (RENDER): after a folder field change the entry is listed under its new folder', () => {
  render(Listing, {
    props: { listing: foldered(['Games', 'Games']), file: 'two.crt' },
  });
  expect(folderRows()).toHaveLength(1);
  expect(within(folderRows()[0]).getByText('2')).toBeInTheDocument();
  expect(
    screen
      .getAllByRole('row')
      .filter((r) => r.hasAttribute('data-folder-row') || r.hasAttribute('data-entry'))
      .map((r) => r.getAttribute('data-folder-row') ?? r.getAttribute('data-entry')),
  ).toEqual(['Games', '0', '1']);
});

/** `two` with hello in Games and an empty folder Spare after it, as the cart lists them. */
function withSpare(): ListingData {
  return { ...foldered(), folders: ['Games', 'Spare'] } as unknown as ListingData;
}

test('AC-189 (RENDER): an empty folder is a card of its own that reads empty and takes a drop', () => {
  render(Listing, { props: { listing: withSpare(), file: 'two.crt' } });
  expect(folderRows().map((r) => r.getAttribute('data-folder-row'))).toEqual(['Games', 'Spare']);
  const spare = folderRows()[1];
  expect(within(spare).getByText('0')).toBeInTheDocument();
  const drop = spare.closest('tbody')!.querySelector('[data-folder-drop]')!;
  expect(drop).toHaveAttribute('data-folder-drop', 'Spare');
  expect(drop).toHaveTextContent(/empty · drag entries here/);
});

test('AC-192 (RENDER): New Folder adds an empty "new folder" after the others and opens its name', async () => {
  const onedit = vi.fn(async () => true);
  const { component, rerender } = render(Listing, {
    props: {
      listing: withSpare(),
      file: 'two.crt',
      readOnly: null,
      rows: listing.rows.map(() => ({ alignable: false })),
      onedit,
      onreplace: vi.fn(),
    },
  });
  await component.newFolder();
  expect(onedit).toHaveBeenCalledExactlyOnceWith({
    kind: 'arrange',
    folders: [
      { name: 'Games', entries: [0] },
      { name: 'Spare', entries: [] },
      { name: 'new folder', entries: [] },
    ],
    loose: [1],
  });
  // The cart answers with the folder; its name opens for typing, as Finder does.
  await rerender({ listing: { ...withSpare(), folders: ['Games', 'Spare', 'new folder'] } as unknown as ListingData });
  expect(await screen.findByRole('textbox', { name: /name of folder new folder/i })).toBeInTheDocument();
});

test('AC-192 (RENDER): a right-click on the list\'s empty space offers New Folder', async () => {
  const { onedit } = editable({ listing: withSpare() });
  await fireEvent.contextMenu(document.querySelector('[data-empty-space]')!);
  await fireEvent.click(await screen.findByText('New Folder'));
  expect(onedit).toHaveBeenCalledOnce();
  expect(onedit.mock.calls[0][0]).toMatchObject({ kind: 'arrange' });
  expect((onedit.mock.calls[0][0] as { folders: { name: string }[] }).folders.map((f) => f.name)).toEqual([
    'Games',
    'Spare',
    'new folder',
  ]);
});

test('AC-193 (RENDER): Delete Folder sends one arrange that moves its entries out', async () => {
  const { onedit } = editable({ listing: withSpare() });
  const trigger = screen.getByRole('button', { name: /actions for folder games/i });
  trigger.focus();
  await fireEvent.keyDown(trigger, { key: 'Enter' });
  await fireEvent.click(await screen.findByText('Delete Folder'));
  expect(onedit).toHaveBeenCalledExactlyOnceWith({
    kind: 'arrange',
    folders: [{ name: 'Spare', entries: [] }],
    loose: [1, 0],
  });
});

test('AC-195 (RENDER): the inspector sits beside the list, empty until an entry is selected', async () => {
  editable({ listing: withSpare() });
  const inspector = screen.getByRole('complementary', { name: /entry details/i });
  expect(inspector).toHaveTextContent(/select an entry/i);
  // Beside the list, not under it: the two share one row.
  const table = screen.getByRole('table');
  expect(inspector.parentElement).toBe(table.closest('.table-wrap')!.parentElement);
  await fireEvent.click(within(entryRows()[0]).getByText('PRG'));
  expect(inspector).toHaveTextContent('// entry 0');
  expect(within(inspector).getByRole('combobox', { name: /folder/i })).toHaveValue('Games');
});

// jsdom never matches :hover, so this reads the stylesheet: in tree mode the rule that removes
// the row hover tint must not also apply to the selected row, or it wipes the highlight (#68).
test('the tree hover reset leaves the selected row alone', async () => {
  const { default: source } = await import('./Listing.svelte?raw');
  expect(source).not.toMatch(/\.tree tbody tr:hover\s*\{/);
  expect(source).toMatch(/\.tree tbody tr:not\(\[aria-selected='true'\]\):hover\s*\{\s*background: none;/);
});
