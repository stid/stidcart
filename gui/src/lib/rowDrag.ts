// Dragging rows of the listing by pointer events (D-061). Tauri's native drag-drop handler stays on
// so Finder drops keep their paths, and with it on WKWebView never hands HTML5 dragover/drop to the
// page; pointer events never start a native drag session, so the two don't meet.
import type { EntryTarget, FolderTarget } from './tree';

/** What is being dragged. */
export type DragSource = { kind: 'entry'; index: number } | { kind: 'folder'; name: string };

/** The row under the pointer: an entry (with its folder, when in one), a folder row, or the strip
 * after the last row that stands for the top level. */
export type Over =
  { kind: 'entry'; index: number; folder?: string } | { kind: 'folder'; name: string } | { kind: 'end' };

export type Drop = EntryTarget | FolderTarget;

/** Pixels the pointer travels before a press becomes a drag, so a click stays a click. */
const THRESHOLD = 4;

/**
 * Where a drop lands. An entry goes before or after an entry by half, into a folder row, or to the
 * top level at the end strip; loose entries always follow the folders, so there is nothing between
 * folders to drop an entry on. A folder goes before or after a folder by half; over a folder's
 * entry it counts as that folder, so it never lands inside one; anywhere else it goes last.
 */
export function dropTarget(
  drag: DragSource['kind'],
  over: Over,
  rect: { top: number; height: number },
  y: number,
): Drop {
  const upper = y < rect.top + rect.height / 2;
  if (drag === 'entry') {
    if (over.kind === 'entry') return upper ? { before: over.index } : { after: over.index };
    if (over.kind === 'folder') return { into: over.name };
    return { end: true };
  }
  const folder = over.kind === 'folder' ? over.name : over.kind === 'entry' ? over.folder : undefined;
  if (folder === undefined) return { last: true };
  return upper ? { before: folder } : { after: folder };
}

/** The row an element sits in, as `dropTarget` reads it, from the rows' data attributes. */
function overOf(el: Element | null): { row: Element; over: Over } | null {
  const row = el?.closest('[data-entry], [data-folder-row], [data-drop-end]');
  if (!row) return null;
  if (row.hasAttribute('data-folder-row'))
    return { row, over: { kind: 'folder', name: row.getAttribute('data-folder-row')! } };
  if (row.hasAttribute('data-drop-end')) return { row, over: { kind: 'end' } };
  const folder = row.getAttribute('data-folder');
  return { row, over: { kind: 'entry', index: Number(row.getAttribute('data-entry')), ...(folder ? { folder } : {}) } };
}

/** The attribute the row under the pointer carries during a drag, for the listing to style. */
function mark(row: Element | null, drop: Drop | null) {
  document.querySelectorAll('[data-drop]').forEach((el) => el.removeAttribute('data-drop'));
  if (row === null || drop === null) return;
  row.setAttribute('data-drop', 'into' in drop ? 'into' : 'before' in drop ? 'before' : 'after');
}

/** An attachment making a row draggable: `ondrop` gets the source and where it landed. */
export function dragRow(source: DragSource, ondrop: (source: DragSource, drop: Drop) => void) {
  return (node: HTMLElement) => {
    let start: { x: number; y: number } | null = null;
    let dragging = false;

    const at = (e: MouseEvent) => {
      const hit = overOf(document.elementFromPoint(e.clientX, e.clientY));
      if (hit === null) return null;
      return { row: hit.row, drop: dropTarget(source.kind, hit.over, hit.row.getBoundingClientRect(), e.clientY) };
    };

    const down = (e: PointerEvent) => {
      if (e.button !== 0) return;
      start = { x: e.clientX, y: e.clientY };
      dragging = false;
    };
    const move = (e: PointerEvent) => {
      if (start === null) return;
      if (!dragging) {
        if (Math.hypot(e.clientX - start.x, e.clientY - start.y) < THRESHOLD) return;
        dragging = true;
        node.setPointerCapture?.(e.pointerId);
        document.body.dataset.dragging = source.kind;
      }
      const hit = at(e);
      mark(hit?.row ?? null, hit?.drop ?? null);
    };
    const up = (e: PointerEvent) => {
      if (start === null) return;
      const dropped = dragging;
      start = null;
      dragging = false;
      delete document.body.dataset.dragging;
      mark(null, null);
      if (!dropped) return;
      // The click that ends a drag must not also select the row it started on.
      const swallow = (c: Event) => c.stopPropagation();
      window.addEventListener('click', swallow, { capture: true, once: true });
      setTimeout(() => window.removeEventListener('click', swallow, { capture: true }), 0);
      const hit = at(e);
      if (hit !== null) ondrop(source, hit.drop);
    };
    // WKWebView starts a native drag from selected text, links or images, which Tauri would take.
    const noNative = (e: Event) => e.preventDefault();

    node.addEventListener('pointerdown', down);
    node.addEventListener('pointermove', move);
    node.addEventListener('pointerup', up);
    node.addEventListener('dragstart', noNative);
    return () => {
      node.removeEventListener('pointerdown', down);
      node.removeEventListener('pointermove', move);
      node.removeEventListener('pointerup', up);
      node.removeEventListener('dragstart', noNative);
    };
  };
}
