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
  /** The size as the listing reads it: "652 bytes", "10.0 KB", "1 MB" (D-044). */
  size: string;
  /** The exact size in bytes, shown on hover. */
  sizeBytes: number;
  /** The kind's family, which picks the type badge's colour as our menu does (D-045): `prg`,
   * `8k`, `16k`, `ult`, `xb`, or `other`. */
  family: string;
  /** The entry's metadata, each empty when not set (D-054). */
  title: string;
  year: string;
  author: string;
  note: string;
  folder: string;
}

export interface Listing {
  cart: string;
  builtBy: string;
  /** The C64-side menu the cart boots into: `EasyLoader`, `stidcart`, `stidcart (older build)`
   * (D-049), or `unrecognised`. */
  menu: string;
  rows: Row[];
  /** The cart's folders in order, empty ones included (D-063). */
  folders: string[];
  free: number;
  freeText: string;
  /** Metadata records dropped because their entry no longer matches (AC-154). */
  metaDropped: number;
  /** Why the cart's metadata is kept as stored, when it is (D-055). */
  metaKept: string | null;
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
  /** View-only only because the loader slot holds no known menu: choosing a menu replaces it and
   * makes the cart editable (D-050). */
  menuReplaceable: boolean;
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
  | { kind: 'setHeaderName'; name: string }
  /** The C64-side menu, by the name the listing shows (D-042). */
  | { kind: 'setMenu'; menu: string }
  /** One metadata field, by its `edit` option name; an empty value clears it (D-054). */
  | { kind: 'setMeta'; index: number; field: MetaField; value: string }
  /** The whole folder tree after a drag or a move command; saved in tree order (D-060, D-061). */
  | { kind: 'arrange'; folders: { name: string; entries: number[] }[]; loose: number[] };

/** The five metadata fields, as `edit` names them. */
export type MetaField = 'title' | 'year' | 'author' | 'note' | 'folder';

/** The answer to "save changes?". */
export type Unsaved = 'save' | 'discard' | 'cancel';

/** The ids the native menu sends (gui/src-tauri/src/menu.rs). */
export type MenuId = 'new' | 'open' | 'new-folder' | 'save' | 'save-as' | 'undo' | 'redo' | 'quit';

export type ViewState =
  { kind: 'empty' } | { kind: 'document'; doc: Document } | { kind: 'error'; file: string; message: string };

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
