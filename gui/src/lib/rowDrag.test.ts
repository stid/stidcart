import { afterEach, expect, test, vi } from 'vitest';
import { dragRow, dropTarget, type Over } from './rowDrag';

const rect = { top: 100, height: 20 };

test('AC-179 (RENDER): an entry drops before or after an entry by half, into a folder row, or at the end strip', () => {
  const entry: Over = { kind: 'entry', index: 4 };
  expect(dropTarget('entry', entry, rect, 105)).toEqual({ before: 4 });
  expect(dropTarget('entry', entry, rect, 115)).toEqual({ after: 4 });
  expect(dropTarget('entry', { kind: 'folder', name: 'Games' }, rect, 101)).toEqual({ into: 'Games' });
  expect(dropTarget('entry', { kind: 'folder', name: 'Games' }, rect, 119)).toEqual({ into: 'Games' });
  expect(dropTarget('entry', { kind: 'end' }, rect, 110)).toEqual({
    end: true,
  });
});

test('AC-181 (RENDER): a folder drops before or after a folder, never into one, and never deeper', () => {
  const games: Over = { kind: 'folder', name: 'Games' };
  expect(dropTarget('folder', games, rect, 105)).toEqual({ before: 'Games' });
  expect(dropTarget('folder', games, rect, 115)).toEqual({ after: 'Games' });
  // Over an entry inside a folder it counts as that folder's row, so it can't land in the folder.
  expect(dropTarget('folder', { kind: 'entry', index: 2, folder: 'Games' }, rect, 105)).toEqual({
    before: 'Games',
  });
  // Over a loose entry or the end strip it goes after the last folder (the caller resolves which).
  expect(dropTarget('folder', { kind: 'entry', index: 7 }, rect, 105)).toEqual({
    last: true,
  });
  expect(dropTarget('folder', { kind: 'end' }, rect, 105)).toEqual({
    last: true,
  });
});

afterEach(() => {
  document.body.innerHTML = '';
  vi.restoreAllMocks();
});

/** Two entry rows in a table, with layout faked: jsdom has none. */
function table() {
  document.body.innerHTML = `<table><tbody>
    <tr data-entry="0"><td>a</td></tr>
    <tr data-entry="1" data-folder="Games"><td>b</td></tr>
  </tbody></table>`;
  const [a, b] = Array.from(document.querySelectorAll('tr'));
  a.getBoundingClientRect = () => ({ top: 0, height: 20 }) as DOMRect;
  b.getBoundingClientRect = () => ({ top: 20, height: 20 }) as DOMRect;
  return { a, b };
}

function pointer(type: string, el: Element, x: number, y: number) {
  // jsdom lacks PointerEvent; a MouseEvent with the pointer type's name carries the coordinates.
  el.dispatchEvent(new MouseEvent(type, { bubbles: true, clientX: x, clientY: y, button: 0 }));
}

test('AC-179 (RENDER): a short press stays a click, a drag past the threshold drops on the row under the pointer', () => {
  const { a, b } = table();
  const ondrop = vi.fn();
  const detach = dragRow({ kind: 'entry', index: 0 }, ondrop)(a);
  document.elementFromPoint = vi.fn(() => b.firstElementChild);

  pointer('pointerdown', a, 10, 10);
  pointer('pointermove', a, 12, 11);
  pointer('pointerup', a, 12, 11);
  expect(ondrop).not.toHaveBeenCalled();

  pointer('pointerdown', a, 10, 10);
  pointer('pointermove', a, 10, 35);
  pointer('pointerup', a, 10, 35);
  expect(ondrop).toHaveBeenCalledExactlyOnceWith({ kind: 'entry', index: 0 }, { after: 1 });

  detach?.();
});

test('AC-179 (RENDER): the end strip, shown only while dragging, takes the drop', () => {
  const { a } = table();
  const strip = document.createElement('tr');
  strip.setAttribute('data-drop-end', '');
  document.querySelector('tbody')!.append(strip);
  strip.getBoundingClientRect = () => ({ top: 40, height: 20 }) as DOMRect;
  // Like the real strip, it is under the pointer only while a drag is on.
  document.elementFromPoint = vi.fn(() => (document.body.dataset.dragging ? strip : null));
  const ondrop = vi.fn();
  dragRow({ kind: 'entry', index: 0 }, ondrop)(a);

  pointer('pointerdown', a, 10, 10);
  pointer('pointermove', a, 10, 50);
  pointer('pointerup', a, 10, 50);
  expect(ondrop).toHaveBeenCalledExactlyOnceWith({ kind: 'entry', index: 0 }, { end: true });
});

test('AC-179 (RENDER): while dragging, the source row is marked and a lifted copy follows the pointer', () => {
  const { a, b } = table();
  document.elementFromPoint = vi.fn(() => b.firstElementChild);
  dragRow({ kind: 'entry', index: 0 }, vi.fn(), '⠿  0  a')(a);

  pointer('pointerdown', a, 10, 10);
  pointer('pointermove', a, 10, 30);
  const ghost = document.querySelector<HTMLElement>('.drag-ghost');
  expect(ghost?.textContent).toBe('⠿  0  a');
  expect(ghost?.style.top).toBe('12px');
  expect(a.hasAttribute('data-drag-source')).toBe(true);

  pointer('pointerup', a, 10, 30);
  expect(document.querySelector('.drag-ghost')).toBeNull();
  expect(a.hasAttribute('data-drag-source')).toBe(false);
});

test('AC-179 (RENDER): a drag keeps the text unselected all the way, not only when it starts', () => {
  const { a, b } = table();
  document.elementFromPoint = vi.fn(() => b.firstElementChild);
  dragRow({ kind: 'entry', index: 0 }, vi.fn())(a);
  const selectText = () => {
    const range = document.createRange();
    range.selectNodeContents(b);
    window.getSelection()!.removeAllRanges();
    window.getSelection()!.addRange(range);
  };

  pointer('pointerdown', a, 10, 10);
  pointer('pointermove', a, 10, 30);
  // The press goes on extending the browser's selection as the pointer moves.
  selectText();
  pointer('pointermove', a, 10, 32);
  expect(window.getSelection()!.rangeCount).toBe(0);

  selectText();
  pointer('pointerup', a, 10, 32);
  expect(window.getSelection()!.rangeCount).toBe(0);
});

test('AC-193 (RENDER): an empty folder\'s drop row takes the entry into that folder', () => {
  const { a } = table();
  const spare = document.createElement('tr');
  spare.setAttribute('data-folder-drop', 'Spare');
  document.querySelector('tbody')!.append(spare);
  spare.getBoundingClientRect = () => ({ top: 40, height: 20 }) as DOMRect;
  document.elementFromPoint = vi.fn(() => spare);
  const ondrop = vi.fn();
  dragRow({ kind: 'entry', index: 0 }, ondrop)(a);
  pointer('pointerdown', a, 10, 10);
  pointer('pointermove', a, 10, 50);
  pointer('pointerup', a, 10, 50);
  expect(ondrop).toHaveBeenCalledExactlyOnceWith({ kind: 'entry', index: 0 }, { into: 'Spare' });
});
