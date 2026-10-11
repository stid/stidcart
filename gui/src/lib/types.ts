// The frontend's view of what the Rust side sends. Mirrors `ListingDto` in
// gui/src-tauri/src/lib.rs; the golden JSON in ./fixtures pins the two together (contract.rs on
// the Rust side, svelte-check on this one, through the tests that annotate the JSON as these types).

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

export type ViewState =
  | { kind: 'empty' }
  | { kind: 'listing'; file: string; listing: Listing }
  | { kind: 'error'; file: string; message: string };

/** Everything the viewer needs from the desktop shell. The real one is host.ts. */
export interface Host {
  /** The chosen path, or null when the chooser is dismissed. */
  pickFile(): Promise<string | null>;
  onDrop(cb: (paths: string[]) => void): void;
  /** Rejects with the refusal text `list` would print. */
  openCartridge(path: string): Promise<Listing>;
}
