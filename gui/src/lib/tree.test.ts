import { expect, test } from 'vitest';
import two from './fixtures/two.json';
import type { Row } from './types';
import {
  addFolder,
  buildTree,
  deleteFolder,
  untitledName,
  indexAfter,
  moveEntry,
  moveFolder,
  newFolder,
  renameFolder,
  sameTree,
  stepEntry,
  stepFolder,
  toArrange,
  type Tree,
} from './tree';

/** Rows named a, b, c, … in directory order, each in the folder given ('' for none). */
function rows(folders: string[]): Row[] {
  return folders.map((folder, index) => ({
    ...two.rows[0],
    index,
    name: String.fromCharCode(97 + index),
    folder,
  })) as unknown as Row[];
}

// a:Games b:'' c:Demos d:Games e:''
const scattered = rows(['Games', '', 'Demos', 'Games', '']);

test('AC-178 (RENDER): folders in the order of their first row, entries in index order, then the loose ones', () => {
  expect(buildTree(scattered)).toEqual({
    folders: [
      { name: 'Games', entries: [0, 3] },
      { name: 'Demos', entries: [2] },
    ],
    loose: [1, 4],
  });
  expect(buildTree(rows(['', '']))).toEqual({ folders: [], loose: [0, 1] });
});

test('AC-180 (RENDER): an arrange carries the tree, so the directory comes out in tree order', () => {
  const tree = buildTree(scattered);
  expect(toArrange(tree)).toEqual({
    kind: 'arrange',
    folders: [
      { name: 'Games', entries: [0, 3] },
      { name: 'Demos', entries: [2] },
    ],
    loose: [1, 4],
  });
  // Flattened: a d c b e, so entry 3 lands at 1 and entry 1 at 3.
  expect([0, 1, 2, 3, 4].map((i) => indexAfter(tree, i))).toEqual([0, 3, 2, 1, 4]);
});

test('AC-179 (RENDER): an entry moves into a folder, out to the top level, or before/after a row', () => {
  const tree = buildTree(scattered);
  expect(moveEntry(tree, 1, { into: 'Demos' })).toEqual({
    folders: [
      { name: 'Games', entries: [0, 3] },
      { name: 'Demos', entries: [2, 1] },
    ],
    loose: [4],
  });
  expect(moveEntry(tree, 3, { end: true })).toEqual({
    folders: [
      { name: 'Games', entries: [0] },
      { name: 'Demos', entries: [2] },
    ],
    loose: [1, 4, 3],
  });
  // Before a row in another folder joins that folder at that place.
  expect(moveEntry(tree, 4, { before: 3 })).toEqual({
    folders: [
      { name: 'Games', entries: [0, 4, 3] },
      { name: 'Demos', entries: [2] },
    ],
    loose: [1],
  });
  expect(moveEntry(tree, 0, { after: 1 })).toEqual({
    folders: [
      { name: 'Games', entries: [3] },
      { name: 'Demos', entries: [2] },
    ],
    loose: [1, 0, 4],
  });
  // AC-193 (RENDER): the last entry out of a folder leaves the folder in place (D-063).
  expect(moveEntry(tree, 2, { end: true }).folders).toEqual([
    { name: 'Games', entries: [0, 3] },
    { name: 'Demos', entries: [] },
  ]);
  // Dropping an entry where it already is changes nothing.
  expect(sameTree(moveEntry(tree, 0, { before: 3 }), tree)).toBe(true);
  expect(sameTree(moveEntry(tree, 1, { into: 'Demos' }), tree)).toBe(false);
});

test('AC-181 (RENDER): a folder moves with all its entries', () => {
  const tree = buildTree(scattered);
  expect(moveFolder(tree, 'Demos', { before: 'Games' })).toEqual({
    folders: [
      { name: 'Demos', entries: [2] },
      { name: 'Games', entries: [0, 3] },
    ],
    loose: [1, 4],
  });
  expect(sameTree(moveFolder(tree, 'Games', { before: 'Demos' }), tree)).toBe(true);
  expect(stepFolder(tree, 'Games', 1).folders.map((f) => f.name)).toEqual(['Demos', 'Games']);
  expect(sameTree(stepFolder(tree, 'Games', -1), tree)).toBe(true);
});

test('AC-182 (RENDER): move to a new folder, and up or down within its own list', () => {
  const tree = buildTree(scattered);
  expect(newFolder(tree, 4, 'Tools')).toEqual({
    folders: [
      { name: 'Games', entries: [0, 3] },
      { name: 'Demos', entries: [2] },
      { name: 'Tools', entries: [4] },
    ],
    loose: [1],
  });
  expect(stepEntry(tree, 3, -1).folders[0].entries).toEqual([3, 0]);
  expect(stepEntry(tree, 1, 1).loose).toEqual([4, 1]);
  // At the edge of its list it stays: up and down never leave the folder.
  expect(sameTree(stepEntry(tree, 0, -1), tree)).toBe(true);
  expect(sameTree(stepEntry(tree, 2, 1), tree)).toBe(true);
});

test('AC-183 (RENDER): a renamed folder keeps its place and its entries', () => {
  const tree: Tree = buildTree(scattered);
  expect(renameFolder(tree, 'Games', 'Arcade')).toEqual({
    folders: [
      { name: 'Arcade', entries: [0, 3] },
      { name: 'Demos', entries: [2] },
    ],
    loose: [1, 4],
  });
});

test('AC-189 (RENDER): the tree starts from the cart\'s folders in its order, empty ones included', () => {
  expect(buildTree(scattered, ['Demos', 'Spare', 'Games'])).toEqual({
    folders: [
      { name: 'Demos', entries: [2] },
      { name: 'Spare', entries: [] },
      { name: 'Games', entries: [0, 3] },
    ],
    loose: [1, 4],
  });
});

test('AC-192 (RENDER): a new folder is empty, goes after the others, and takes the first free untitled name', () => {
  const tree = buildTree(scattered, ['Games', 'Demos']);
  expect(untitledName(tree)).toBe('new folder');
  const one = addFolder(tree, untitledName(tree));
  expect(one.folders.map((f) => [f.name, f.entries.length])).toEqual([
    ['Games', 2],
    ['Demos', 1],
    ['new folder', 0],
  ]);
  expect(untitledName(one)).toBe('new folder 2');
  expect(untitledName(addFolder(one, 'new folder 2'))).toBe('new folder 3');
});

test('AC-193 (RENDER): deleting a folder moves its entries to the end of the loose ones', () => {
  const tree = buildTree(scattered, ['Games', 'Demos']);
  expect(deleteFolder(tree, 'Games')).toEqual({
    folders: [{ name: 'Demos', entries: [2] }],
    loose: [1, 4, 0, 3],
  });
  // An entry dropped into an empty folder goes in.
  const spare = addFolder(tree, 'Spare');
  expect(moveEntry(spare, 4, { into: 'Spare' }).folders[2]).toEqual({ name: 'Spare', entries: [4] });
});
