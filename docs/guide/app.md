# The desktop app

The app does in a window what `stidcart edit` does on the command line, and saves the same bytes.
Installing it is in the [README](../../README.md#getting-it). The shortcuts here are the macOS ones.

## Using it

Open a `.crt` with Open…, ⌘O or by dropping it on the window. It shows the cart name, the
producing tool, every entry and the free space: the values `list` prints, from the same code. Sizes read in bytes, KB or MB; hover one to see its exact byte count. A file it
can't describe shows the same reason `list` gives.

The window edits what `edit` edits:
- Drop programs or cartridge images on an open cart, or use Add… / Add Aligned….
- Rename an entry by clicking its name. Its "⋯" or right-click menu replaces, hides or shows,
  aligns or removes it.
- Click the cart name or the header name to change either one.
- Click an entry to show it in the inspector beside the list: its number, name, kind and size, and
  its title, year, author, note and folder. Click a field to change it, or pick its folder. Each
  change is one undoable edit, refused in `edit`'s words when invalid.
- New Folder (in the title bar, File ▸ New Folder ⇧⌘N, or a right-click on the list's empty space)
  adds an empty folder and opens its name. A folder's menu renames, moves or deletes it; deleting
  moves its entries out, never removes them.
- With any folder, the list shows the cart as our menu does: folder rows, each with its entries,
  then the entries in no folder. Drag an entry onto a folder, between rows, or to the strip at the
  bottom to take it out of its folder; drag a folder row to move the whole folder. An empty folder
  takes a drop too. The entry menu has Move to (every folder, the current one checked, Not in a
  folder, New Folder with Entry), Move up and Move down. After any of these the cart is saved in that order, so EasyLoader's flat list
  follows it too.
- Choose the menu the cart boots into, EasyLoader or stidcart, from the menu picker under the
  names. A cart our menu can't run (an Ocean entry, or a program it can't start; see [cli.md](cli.md)) refuses stidcart
  and names the entry, as `edit --menu` does.
- New (⌘N) starts an empty cart.

Changes stay in memory until you save:
- Undo and redo cover every edit since the cart was opened, and an "Edited" dot marks unsaved
  changes.
- ⌘S replaces the file and ⌘⇧S saves as. A saved cart is byte-identical to what `edit` or `build`
  writes for the same request.
- Closing, quitting, opening another cart or New asks first.

A cart `edit` refuses, such as firmware that differs from the embedded copy, opens view-only with
the reason. When the only reason is a loader the tool doesn't know, the menu picker stays: choosing
a menu replaces the loader and makes the cart editable.
