// The frontend's view of what the Rust side sends. Mirrors `ListingDto` and `DocumentDto` in
// gui/src-tauri/src/; the golden JSON in ./fixtures pins the two together (contract.rs on the Rust
// side, svelte-check on this one, through the tests that annotate the JSON as these types).

export interface Row {
  index: number;
  name: string;
  /** `H` hidden, `A` 64 KB aligned, `S` subdirectory bit, in that order; `-` when none is set. */
  flags: string;
  kind: string;
  bank: string;
  layout: string;
  size: string;
}

export interface Listing {
  cart: string;
  builtBy: string;
  rows: Row[];
  free: number;
  freeText: string;
}

/** What the window may offer on a row. Decided by the Rust side, never from the kind label. */
export interface RowActions {
  alignable: boolean;
}

/** The open cart: its listing as saving would write it, and its editing state. */
export interface Document {
  /** The file name, or "Untitled" for a cart never saved. */
  file: string;
  untitled: boolean;
  listing: Listing;
  rows: RowActions[];
  /** Why the cart cannot be edited, in the words `edit` uses; null when it can. */
  readOnly: string | null;
  dirty: boolean;
  canUndo: boolean;
  canRedo: boolean;
}

/** One change, as the Rust `Edit` enum receives it. Indices are the listing's. */
export type Edit =
  | { kind: 'add'; paths: string[]; aligned: boolean }
  | { kind: 'remove'; index: number }
  | { kind: 'rename'; index: number; name: string }
  | { kind: 'replace'; index: number; path: string }
  | { kind: 'setHidden'; index: number; hidden: boolean }
  | { kind: 'setAligned'; index: number; aligned: boolean }
  | { kind: 'setCartName'; name: string }
  | { kind: 'setHeaderName'; name: string };

/** The answer to "save changes?". */
export type Unsaved = 'save' | 'discard' | 'cancel';

/** The ids the native menu sends (gui/src-tauri/src/menu.rs). */
export type MenuId = 'new' | 'open' | 'save' | 'save-as' | 'undo' | 'redo' | 'quit';

export type ViewState =
  | { kind: 'empty' }
  | { kind: 'document'; doc: Document }
  | { kind: 'error'; file: string; message: string };

/** Everything the window needs from the desktop shell. The real one is host.ts. */
export interface Host {
  /** The chosen cartridge, or null when the chooser is dismissed. */
  pickFile(): Promise<string | null>;
  onDrop(cb: (paths: string[]) => void): void;
  /** Programs and cartridge images to add; empty when the chooser is dismissed. */
  pickInputs(): Promise<string[]>;
  /** Where to save, suggesting `suggested`; null when the chooser is dismissed. */
  pickSavePath(suggested: string): Promise<string | null>;
  confirmUnsaved(file: string): Promise<Unsaved>;
  onMenu(cb: (id: MenuId) => void): void;
  /** `cb` resolves true when the window may close. */
  onCloseRequested(cb: () => Promise<boolean>): void;
  /** Quitting was stopped because of unsaved changes; the window decides. */
  onExitRequested(cb: () => void): void;
  /** Each rejects with the reason, in the words `list` or `edit` use. */
  openDocument(path: string): Promise<Document>;
  newDocument(): Promise<Document>;
  edit(e: Edit): Promise<Document>;
  undo(): Promise<Document>;
  redo(): Promise<Document>;
  save(): Promise<Document>;
  saveAs(path: string): Promise<Document>;
  /** Quit, the unsaved-changes question already answered. */
  exit(): Promise<void>;
}
