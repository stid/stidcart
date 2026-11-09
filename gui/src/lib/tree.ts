// The cart as our menu shows it: folders in the order their first entry appears, each holding its
// entries in index order, then the entries in no folder (D-057). Every move here is a pure function
// from one tree to the next; the window sends the result as one `arrange` edit, which the Rust side
// saves in tree order (D-060, D-061).
import type { Edit, Row } from './types';

export interface Folder {
  name: string;
  entries: number[];
}

export interface Tree {
  folders: Folder[];
  loose: number[];
}

/** Where an entry is dropped: before or after another entry, into a folder, or at the end of the
 * top level. */
export type EntryTarget = { before: number } | { after: number } | { into: string } | { end: true };

/** A move picked from an entry's menu, the non-drag path to every drag (AC-182). */
export type EntryMove =
  | { kind: 'folder'; name: string }
  | { kind: 'noFolder' }
  | { kind: 'newFolder' }
  | { kind: 'up' }
  | { kind: 'down' };

/** Where a folder is dropped: before or after another folder, or after the last one. */
export type FolderTarget = { before: string } | { after: string } | { last: true };

/** The tree of `rows`, starting from the cart's `folders` in order (empty ones included, D-063);
 * a folder a row names that the list lacks joins after the others. */
export function buildTree(rows: Row[], folders: string[] = []): Tree {
  const tree: Tree = { folders: folders.map((name) => ({ name, entries: [] })), loose: [] };
  for (const r of rows) {
    if (r.folder === '') {
      tree.loose.push(r.index);
      continue;
    }
    let folder = tree.folders.find((f) => f.name === r.folder);
    if (folder === undefined) {
      folder = { name: r.folder, entries: [] };
      tree.folders.push(folder);
    }
    folder.entries.push(r.index);
  }
  return tree;
}

function copy(tree: Tree): Tree {
  return { folders: tree.folders.map((f) => ({ name: f.name, entries: [...f.entries] })), loose: [...tree.loose] };
}

/** The list (a folder's entries, or the loose ones) holding `index`. */
function listOf(tree: Tree, index: number): number[] {
  return tree.folders.find((f) => f.entries.includes(index))?.entries ?? tree.loose;
}

/** The tree without `index`. A folder it leaves empty stays: the cart keeps its folders (D-063). */
function without(tree: Tree, index: number): Tree {
  const t = copy(tree);
  t.folders.forEach((f) => (f.entries = f.entries.filter((i) => i !== index)));
  t.loose = t.loose.filter((i) => i !== index);
  return t;
}

export function moveEntry(tree: Tree, index: number, target: EntryTarget): Tree {
  if (('before' in target && target.before === index) || ('after' in target && target.after === index)) return tree;
  const t = without(tree, index);
  if ('into' in target) {
    const folder = t.folders.find((f) => f.name === target.into);
    if (folder === undefined) return tree;
    // Dropped back into its own folder where it already is last, nothing changes.
    folder.entries.push(index);
  } else if ('end' in target) {
    t.loose.push(index);
  } else {
    const anchor = 'before' in target ? target.before : target.after;
    const list = listOf(t, anchor);
    list.splice(list.indexOf(anchor) + ('after' in target ? 1 : 0), 0, index);
  }
  return t;
}

export function moveFolder(tree: Tree, name: string, target: FolderTarget): Tree {
  if (('before' in target && target.before === name) || ('after' in target && target.after === name)) return tree;
  const t = copy(tree);
  const at = t.folders.findIndex((f) => f.name === name);
  const [folder] = t.folders.splice(at, 1);
  if ('last' in target) {
    t.folders.push(folder);
  } else {
    const other = t.folders.findIndex((f) => f.name === ('before' in target ? target.before : target.after));
    t.folders.splice(other + ('after' in target ? 1 : 0), 0, folder);
  }
  return t;
}

/** An empty folder named `name`, after the others. */
export function addFolder(tree: Tree, name: string): Tree {
  const t = copy(tree);
  t.folders.push({ name, entries: [] });
  return t;
}

/** The folder gone; its entries move to the end of the loose ones, staying in the cart. */
export function deleteFolder(tree: Tree, name: string): Tree {
  const t = copy(tree);
  const at = t.folders.findIndex((f) => f.name === name);
  if (at < 0) return tree;
  const [folder] = t.folders.splice(at, 1);
  t.loose.push(...folder.entries);
  return t;
}

/** "new folder", or the first "new folder N" no folder has, as a file browser names one. */
export function untitledName(tree: Tree): string {
  const taken = new Set(tree.folders.map((f) => f.name));
  if (!taken.has('new folder')) return 'new folder';
  let n = 2;
  while (taken.has(`new folder ${n}`)) n++;
  return `new folder ${n}`;
}

/** The entry moved into a new folder of that name, after the others. */
export function newFolder(tree: Tree, index: number, name: string): Tree {
  const t = without(tree, index);
  t.folders.push({ name, entries: [index] });
  return t;
}

export function renameFolder(tree: Tree, from: string, to: string): Tree {
  const t = copy(tree);
  t.folders.forEach((f) => {
    if (f.name === from) f.name = to;
  });
  return t;
}

function swap(list: unknown[], at: number, dir: -1 | 1): boolean {
  const other = at + dir;
  if (at < 0 || other < 0 || other >= list.length) return false;
  [list[at], list[other]] = [list[other], list[at]];
  return true;
}

/** Up or down one place within its own folder or among the loose entries; at the edge it stays. */
export function stepEntry(tree: Tree, index: number, dir: -1 | 1): Tree {
  const t = copy(tree);
  const list = listOf(t, index);
  return swap(list, list.indexOf(index), dir) ? t : tree;
}

/** Whether `stepEntry` would move it. */
export function canStep(tree: Tree, index: number, dir: -1 | 1): boolean {
  const list = listOf(tree, index);
  const other = list.indexOf(index) + dir;
  return other >= 0 && other < list.length;
}

export function stepFolder(tree: Tree, name: string, dir: -1 | 1): Tree {
  const t = copy(tree);
  return swap(
    t.folders,
    t.folders.findIndex((f) => f.name === name),
    dir,
  )
    ? t
    : tree;
}

export function sameTree(a: Tree, b: Tree): boolean {
  return JSON.stringify(a) === JSON.stringify(b);
}

/** The entries in the order saving writes them: each folder's in turn, then the loose ones. */
function flatten(tree: Tree): number[] {
  return [...tree.folders.flatMap((f) => f.entries), ...tree.loose];
}

export function toArrange(tree: Tree): Edit {
  const t = copy(tree);
  return { kind: 'arrange', folders: t.folders, loose: t.loose };
}

/** Where entry `index` sits once the tree is saved. */
export function indexAfter(tree: Tree, index: number): number {
  return flatten(tree).indexOf(index);
}
