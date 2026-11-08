// The window's state: what it shows, and how opening, editing and saving change it. Components only
// render this; the host (host.ts, or a fake in tests) is the only way out to the desktop. The cart
// itself, with its undo history, lives on the Rust side (D-035); this holds the latest answer.
import type { Document, Edit, Host, MenuId, Unsaved, ViewState } from './types';

/** The file name a path ends in, which is what the window shows. */
function fileName(path: string): string {
  return path.split(/[\\/]/).pop() ?? path;
}

/** The prompt's button label as an answer. Only the two going-ahead labels go ahead: Esc, a
 * dismissed dialog or anything unexpected cancels, so the worst case keeps the changes. */
export function unsavedChoice(label: string): Unsaved {
  if (label === 'Save') return 'save';
  if (label === "Don't Save") return 'discard';
  return 'cancel';
}

/** A text field has focus, so undo and redo belong to its text, not to the cart. */
function editingText(): boolean {
  const el = document.activeElement;
  return (
    el instanceof HTMLInputElement ||
    el instanceof HTMLTextAreaElement ||
    (el instanceof HTMLElement && el.isContentEditable)
  );
}

export function createViewer(host: Host) {
  let state = $state<ViewState>({ kind: 'empty' });
  let notice = $state<string | null>(null);
  /** Counts the documents shown: opening a cart or New starts a new one, an edit or a save does
   * not, so per-document view state (the selected entry) can be reset by keying on it. */
  let generation = $state(0);

  const doc = $derived(state.kind === 'document' ? state.doc : null);
  const editable = $derived(doc !== null && doc.readOnly === null);

  function show(d: Document) {
    state = { kind: 'document', doc: d };
  }

  /** Run a host call that answers with the cart's new state; a refusal becomes the notice and
   * leaves the window as it was. Returns whether it succeeded. */
  async function attempt(call: () => Promise<Document>): Promise<boolean> {
    notice = null;
    try {
      show(await call());
      return true;
    } catch (e) {
      notice = String(e);
      return false;
    }
  }

  async function open(path: string) {
    notice = null;
    try {
      show(await host.openDocument(path));
      generation++;
    } catch (e) {
      // Whatever was on screen describes another file, so it goes.
      state = { kind: 'error', file: fileName(path), message: String(e) };
    }
  }

  async function saveAs(): Promise<boolean> {
    if (!editable || !doc) return false;
    const suggested = doc.untitled ? `${doc.file}.crt` : doc.file;
    const path = await host.pickSavePath(suggested);
    if (path === null) return false;
    return attempt(() => host.saveAs(path));
  }

  async function save(): Promise<boolean> {
    if (doc?.readOnly != null && doc.dirty) {
      // Undoing a saved menu switch brings back the cart as opened, which can't be written (D-050).
      notice =
        "This step can't be saved: it is the cart as opened, which can't be edited. Redo the menu switch to save, or choose Don't Save.";
      return false;
    }
    if (!editable || !doc) return false;
    if (doc.untitled) return saveAs();
    if (!doc.dirty) return true;
    return attempt(() => host.save());
  }

  /** Ask about unsaved changes, if there are any; true when the caller may go ahead. */
  async function settle(): Promise<boolean> {
    if (!doc?.dirty) return true;
    const answer = await host.confirmUnsaved(doc.file);
    if (answer === 'discard') return true;
    if (answer === 'save') return save();
    return false;
  }

  async function pick() {
    if (!(await settle())) return;
    const path = await host.pickFile();
    if (path !== null) await open(path);
  }

  async function newCart() {
    if (!(await settle())) return;
    if (await attempt(() => host.newDocument())) generation++;
  }

  async function quit() {
    if (await settle()) await host.exit();
  }

  async function undoRedo(which: 'undo' | 'redo') {
    if (editingText()) {
      document.execCommand(which);
      return;
    }
    if (!editable) return;
    await attempt(() => (which === 'undo' ? host.undo() : host.redo()));
  }

  return {
    get state() {
      return state;
    },
    get notice() {
      return notice;
    },
    get generation() {
      return generation;
    },
    get editable() {
      return editable;
    },
    get dirty() {
      return doc?.dirty ?? false;
    },
    pick,
    newCart,
    save,
    saveAs,
    async edit(e: Edit) {
      // A view-only cart takes one edit: a menu switch, when only its loader is unknown (D-050).
      const menuSwitch = e.kind === 'setMenu' && doc !== null && doc.menuReplaceable;
      if (editable || menuSwitch) await attempt(() => host.edit(e));
    },
    async add(aligned: boolean) {
      if (!editable) return;
      const paths = await host.pickInputs();
      if (paths.length > 0) await attempt(() => host.edit({ kind: 'add', paths, aligned }));
    },
    async replace(index: number) {
      if (!editable) return;
      const [path] = await host.pickInputs();
      if (path !== undefined) await attempt(() => host.edit({ kind: 'replace', index, path }));
    },
    async drop(paths: string[]) {
      if (editable) {
        await attempt(() => host.edit({ kind: 'add', paths, aligned: false }));
        return;
      }
      if (paths.length !== 1) {
        notice = 'Drop one file at a time.';
        return;
      }
      await open(paths[0]);
    },
    async menu(id: MenuId) {
      switch (id) {
        case 'new':
          return newCart();
        case 'open':
          return pick();
        case 'save':
          await save();
          return;
        case 'save-as':
          await saveAs();
          return;
        case 'undo':
        case 'redo':
          return undoRedo(id);
        case 'quit':
          return quit();
      }
    },
    /** The window is closing; true when it may. */
    requestClose(): Promise<boolean> {
      return settle();
    },
    /** Quitting was held back for unsaved changes. */
    requestQuit: quit,
  };
}
