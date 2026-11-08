// The viewer's state machine, driven through a fake host (gui-shell, gui-edit). The real host in
// host.ts is the only code that touches Tauri, so nothing here needs a webview.
import { describe, expect, test, vi } from 'vitest';
import two from './fixtures/two.json';
import refused from './fixtures/refused.json';
import documentGolden from './fixtures/document.json';
import type { Document, Host, Listing } from './types';
import { createViewer, unsavedChoice } from './viewer.svelte';

const listing: Listing = two;
// svelte-check holds the Rust side's DocumentDto golden to this type (contract.rs pins the other end).
const golden: Document = documentGolden;

function doc(overrides: Partial<Document> = {}): Document {
  return {
    file: 'two.crt',
    untitled: false,
    listing,
    rows: listing.rows.map(() => ({ alignable: false })),
    readOnly: null,
    menuReplaceable: false,
    dirty: false,
    canUndo: false,
    canRedo: false,
    ...overrides,
  };
}

const clean = doc();
const dirty = doc({ dirty: true, canUndo: true });

function fakeHost(overrides: Partial<Host> = {}): Host {
  return {
    pickFile: vi.fn(async () => null),
    onDrop: vi.fn(),
    pickInputs: vi.fn(async () => []),
    pickSavePath: vi.fn(async () => null),
    confirmUnsaved: vi.fn(async () => 'cancel' as const),
    onMenu: vi.fn(),
    onCloseRequested: vi.fn(),
    onExitRequested: vi.fn(),
    openDocument: vi.fn(async () => clean),
    newDocument: vi.fn(async () => doc({ file: 'Untitled', untitled: true, rows: [] })),
    edit: vi.fn(async () => dirty),
    undo: vi.fn(async () => clean),
    redo: vi.fn(async () => dirty),
    save: vi.fn(async () => doc({ canUndo: true })),
    saveAs: vi.fn(async () => doc({ file: 'copy.crt', canUndo: true })),
    exit: vi.fn(async () => {}),
    ...overrides,
  };
}

/** A viewer with `d` open, reached through the host the way the window reaches it. */
async function opened(host: Host, d: Document = clean) {
  host.openDocument = vi.fn(async () => d);
  const v = createViewer(host);
  await v.drop(['/carts/two.crt']);
  host.openDocument = vi.fn(async () => clean);
  return v;
}

test('the DocumentDto golden is a Document', () => {
  expect(golden.rows).toHaveLength(golden.listing.rows.length);
});

describe('opening (gui-shell)', () => {
  test('AC-52 (RENDER): a single dropped file is opened and shown', async () => {
    const host = fakeHost();
    const v = createViewer(host);
    await v.drop(['/carts/two.crt']);
    expect(host.openDocument).toHaveBeenCalledWith('/carts/two.crt');
    expect(v.state).toEqual({ kind: 'document', doc: clean });
  });

  test('AC-53 (RENDER): a picked file is opened; dismissing the chooser changes nothing', async () => {
    const host = fakeHost({ pickFile: vi.fn(async () => '/carts/two.crt') });
    const v = createViewer(host);
    await v.pick();
    expect(host.openDocument).toHaveBeenCalledWith('/carts/two.crt');
    expect(v.state).toEqual({ kind: 'document', doc: clean });

    host.pickFile = vi.fn(async () => null);
    await v.pick();
    expect(host.openDocument).toHaveBeenCalledTimes(1);
    expect(v.state).toEqual({ kind: 'document', doc: clean });
  });

  test('AC-54 (RENDER): a refused file replaces the shown cartridge with its reason', async () => {
    const host = fakeHost({ pickFile: vi.fn(async () => '/carts/in_8k.crt') });
    const v = await opened(host);
    host.openDocument = vi.fn(async () => {
      throw refused;
    });
    await v.pick();
    expect(v.state).toEqual({ kind: 'error', file: 'in_8k.crt', message: refused });
  });

  test('AC-55 (RENDER): opening a valid file after an error shows it and clears the error', async () => {
    const host = fakeHost({
      openDocument: vi.fn(async () => {
        throw refused;
      }),
    });
    const v = createViewer(host);
    await v.drop(['/carts/in_8k.crt']);
    expect(v.state.kind).toBe('error');
    host.openDocument = vi.fn(async () => clean);
    await v.drop(['/carts/two.crt']);
    expect(v.state).toEqual({ kind: 'document', doc: clean });
  });

  test('AC-56 (RENDER): with nothing editable open, dropping several files opens none and asks for one', async () => {
    const host = fakeHost({
      openDocument: vi.fn(async () => {
        throw refused;
      }),
    });
    const v = createViewer(host);
    await v.drop(['/carts/in_8k.crt']);
    await v.drop(['/carts/a.crt', '/carts/b.crt']);
    expect(host.openDocument).toHaveBeenCalledTimes(1);
    expect(v.state.kind).toBe('error');
    expect(v.notice).toMatch(/one file/i);
  });
});

describe('editing (gui-edit)', () => {
  test('AC-59 (RENDER): an opened cart is a document, editable and clean', async () => {
    const v = await opened(fakeHost());
    expect(v.state).toEqual({ kind: 'document', doc: clean });
    expect(v.editable).toBe(true);
    expect(v.dirty).toBe(false);
  });

  test('AC-61 (RENDER): the add command sends the picked files as one add', async () => {
    const host = fakeHost({ pickInputs: vi.fn(async () => ['/in/hello.prg']) });
    const v = await opened(host);
    await v.add(false);
    expect(host.edit).toHaveBeenCalledWith({ kind: 'add', paths: ['/in/hello.prg'], aligned: false });
    expect(v.state).toEqual({ kind: 'document', doc: dirty });

    host.pickInputs = vi.fn(async () => []);
    await v.add(false);
    expect(host.edit).toHaveBeenCalledTimes(1);
  });

  test('AC-63 (RENDER): several files are sent as a single edit', async () => {
    const paths = ['/in/a.prg', '/in/b.crt', '/in/c.prg'];
    const host = fakeHost({ pickInputs: vi.fn(async () => paths) });
    const v = await opened(host);
    await v.add(false);
    expect(host.edit).toHaveBeenCalledOnce();
    expect(host.edit).toHaveBeenCalledWith({ kind: 'add', paths, aligned: false });
  });

  test('AC-70 (RENDER): a refused edit shows its reason and keeps the cart as it was', async () => {
    const reason = "'big' needs 999999 bytes but only 12 are free on the cartridge";
    const host = fakeHost({
      edit: vi.fn(async () => {
        throw reason;
      }),
    });
    const v = await opened(host);
    await v.edit({ kind: 'remove', index: 0 });
    expect(v.notice).toBe(reason);
    expect(v.state).toEqual({ kind: 'document', doc: clean });
  });

  test('AC-73 (RENDER): undo and redo act on the cart, or on the text field being edited', async () => {
    const host = fakeHost();
    const v = await opened(host, dirty);
    await v.menu('undo');
    expect(host.undo).toHaveBeenCalledOnce();
    expect(v.state).toEqual({ kind: 'document', doc: clean });
    await v.menu('redo');
    expect(host.redo).toHaveBeenCalledOnce();

    const input = document.createElement('input');
    document.body.append(input);
    input.focus();
    const execCommand = vi.fn(() => true);
    document.execCommand = execCommand;
    await v.menu('undo');
    await v.menu('redo');
    expect(host.undo).toHaveBeenCalledOnce();
    expect(host.redo).toHaveBeenCalledOnce();
    expect(execCommand).toHaveBeenNthCalledWith(1, 'undo');
    expect(execCommand).toHaveBeenNthCalledWith(2, 'redo');
    input.remove();
  });

  test('AC-74 (RENDER): save writes a dirty cart and does nothing for a clean one', async () => {
    const host = fakeHost();
    const v = await opened(host);
    await v.menu('save');
    expect(host.save).not.toHaveBeenCalled();

    const w = await opened(host, dirty);
    await w.menu('save');
    expect(host.save).toHaveBeenCalledOnce();
    expect(w.dirty).toBe(false);
  });

  test('AC-75 (RENDER): Save As saves to the picked path; dismissing the chooser changes nothing', async () => {
    const host = fakeHost();
    const v = await opened(host, dirty);
    await v.menu('save-as');
    expect(host.pickSavePath).toHaveBeenCalledWith('two.crt');
    expect(host.saveAs).not.toHaveBeenCalled();
    expect(v.state).toEqual({ kind: 'document', doc: dirty });

    host.pickSavePath = vi.fn(async () => '/carts/copy.crt');
    await v.menu('save-as');
    expect(host.saveAs).toHaveBeenCalledWith('/carts/copy.crt');
    expect(v.state.kind === 'document' && v.state.doc.file).toBe('copy.crt');
  });

  test('AC-76 (RENDER): a failed save shows the reason and keeps the unsaved changes', async () => {
    const host = fakeHost({
      save: vi.fn(async () => {
        throw "replacing '/carts/two.crt': Permission denied";
      }),
    });
    const v = await opened(host, dirty);
    await v.menu('save');
    expect(v.notice).toMatch(/Permission denied/);
    expect(v.dirty).toBe(true);
  });

  test('AC-77 (RENDER): New opens an untitled cart, and saving it goes through Save As', async () => {
    const host = fakeHost({ pickSavePath: vi.fn(async () => '/carts/new.crt') });
    const v = createViewer(host);
    await v.menu('new');
    expect(host.newDocument).toHaveBeenCalledOnce();
    expect(v.state.kind === 'document' && v.state.doc.untitled).toBe(true);
    expect(v.editable).toBe(true);

    host.edit = vi.fn(async () => doc({ file: 'Untitled', untitled: true, dirty: true }));
    await v.edit({ kind: 'add', paths: ['/in/hello.prg'], aligned: false });
    await v.menu('save');
    expect(host.save).not.toHaveBeenCalled();
    expect(host.pickSavePath).toHaveBeenCalledWith('Untitled.crt');
    expect(host.saveAs).toHaveBeenCalledWith('/carts/new.crt');
  });

  describe('AC-78 (RENDER): unsaved changes are never lost without asking', () => {
    test('the prompt labels map to save, discard and cancel; anything else cancels', () => {
      expect(unsavedChoice('Save')).toBe('save');
      expect(unsavedChoice("Don't Save")).toBe('discard');
      expect(unsavedChoice('Cancel')).toBe('cancel');
      expect(unsavedChoice('')).toBe('cancel');
      expect(unsavedChoice('Ok')).toBe('cancel');
    });

    test('a clean cart is replaced, closed or quit without asking', async () => {
      const host = fakeHost({ pickFile: vi.fn(async () => '/carts/other.crt') });
      const v = await opened(host);
      await v.menu('open');
      await v.menu('new');
      expect(await v.requestClose()).toBe(true);
      await v.requestQuit();
      expect(host.confirmUnsaved).not.toHaveBeenCalled();
      expect(host.exit).toHaveBeenCalledOnce();
    });

    test('Cancel stops New, Open, close and quit, leaving everything as it was', async () => {
      const host = fakeHost({ pickFile: vi.fn(async () => '/carts/other.crt') });
      const v = await opened(host, dirty);
      await v.menu('new');
      await v.menu('open');
      expect(await v.requestClose()).toBe(false);
      await v.requestQuit();
      expect(host.confirmUnsaved).toHaveBeenCalledTimes(4);
      expect(host.newDocument).not.toHaveBeenCalled();
      expect(host.pickFile).not.toHaveBeenCalled();
      expect(host.exit).not.toHaveBeenCalled();
      expect(host.save).not.toHaveBeenCalled();
      expect(v.state).toEqual({ kind: 'document', doc: dirty });
    });

    test("Don't Save goes ahead and the changes are dropped, asking once", async () => {
      const host = fakeHost({ confirmUnsaved: vi.fn(async () => 'discard' as const) });
      const v = await opened(host, dirty);
      await v.menu('new');
      expect(host.confirmUnsaved).toHaveBeenCalledOnce();
      expect(host.save).not.toHaveBeenCalled();
      expect(host.newDocument).toHaveBeenCalledOnce();

      const w = await opened(host, dirty);
      expect(await w.requestClose()).toBe(true);
      await w.requestQuit();
      expect(host.exit).toHaveBeenCalledOnce();
    });

    test('the Quit menu item goes through the same question before quitting', async () => {
      const host = fakeHost();
      const v = await opened(host, dirty);
      await v.menu('quit');
      expect(host.confirmUnsaved).toHaveBeenCalledOnce();
      expect(host.exit).not.toHaveBeenCalled();

      host.confirmUnsaved = vi.fn(async () => 'discard' as const);
      await v.menu('quit');
      expect(host.exit).toHaveBeenCalledOnce();
    });

    test('Save saves first, then goes ahead', async () => {
      const host = fakeHost({ confirmUnsaved: vi.fn(async () => 'save' as const) });
      const v = await opened(host, dirty);
      expect(await v.requestClose()).toBe(true);
      expect(host.save).toHaveBeenCalledOnce();
      expect(host.confirmUnsaved).toHaveBeenCalledOnce();
    });

    test('Save on an untitled cart asks for a path; dismissing it or a failed save stops', async () => {
      const host = fakeHost({ confirmUnsaved: vi.fn(async () => 'save' as const) });
      const untitled = doc({ file: 'Untitled', untitled: true, dirty: true, canUndo: true });
      const v = await opened(host, untitled);
      expect(await v.requestClose()).toBe(false);
      expect(host.pickSavePath).toHaveBeenCalledOnce();
      expect(host.saveAs).not.toHaveBeenCalled();

      host.pickSavePath = vi.fn(async () => '/carts/kept.crt');
      host.saveAs = vi.fn(async () => {
        throw 'disk full';
      });
      await v.requestQuit();
      expect(host.exit).not.toHaveBeenCalled();
      expect(v.notice).toMatch(/disk full/);
      expect(v.state).toEqual({ kind: 'document', doc: untitled });
    });
  });

  test('AC-79 (RENDER): a drop adds to an editable cart and opens otherwise', async () => {
    const host = fakeHost();
    const v = await opened(host);
    await v.drop(['/in/a.prg', '/in/b.crt']);
    expect(host.edit).toHaveBeenCalledWith({ kind: 'add', paths: ['/in/a.prg', '/in/b.crt'], aligned: false });
    await v.drop(['/in/c.crt']);
    expect(host.edit).toHaveBeenLastCalledWith({ kind: 'add', paths: ['/in/c.crt'], aligned: false });
    expect(host.openDocument).not.toHaveBeenCalled();

    const viewOnly = doc({ readOnly: 'reserved regions differ from the embedded firmware' });
    const w = await opened(host, viewOnly);
    expect(w.editable).toBe(false);
    await w.drop(['/carts/a.crt', '/carts/b.crt']);
    expect(w.notice).toMatch(/one file/i);
    await w.drop(['/carts/two.crt']);
    expect(host.openDocument).toHaveBeenCalledWith('/carts/two.crt');
    expect(host.edit).toHaveBeenCalledTimes(2);
  });
});

describe('a view-only cart whose loader is unknown (replace-loader)', () => {
  test('AC-134 (RENDER): only a menu switch reaches the host', async () => {
    const host = fakeHost();
    const v = await opened(
      host,
      doc({ readOnly: 'reserved region menu loader differs from the embedded firmware', menuReplaceable: true }),
    );
    expect(v.editable).toBe(false);
    await v.edit({ kind: 'rename', index: 0, name: 'x' });
    expect(host.edit).not.toHaveBeenCalled();
    await v.edit({ kind: 'setMenu', menu: 'stidcart' });
    expect(host.edit).toHaveBeenCalledWith({ kind: 'setMenu', menu: 'stidcart' });
  });

  test('AC-135 (RENDER): a cart refused for another reason takes no menu switch', async () => {
    const host = fakeHost();
    const v = await opened(host, doc({ readOnly: 'reserved region EAPI differs from the embedded firmware' }));
    await v.edit({ kind: 'setMenu', menu: 'stidcart' });
    expect(host.edit).not.toHaveBeenCalled();
  });
});
