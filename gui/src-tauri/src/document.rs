//! The cart the editor window holds: every step since it was opened, as the image saving it would
//! write (D-035).
//!
//! Each snapshot is the bytes a save at that point produces, with the listing of exactly those
//! bytes, so the window never shows anything but what saving would write. Snapshot 0 of an opened
//! cart is the file itself. An edit reloads the current image, applies one change, rebuilds and
//! describes it; anything that fails on the way leaves the document as it was.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use stidcart_core::listing::{self, Listing};
use stidcart_core::menu::Menu;
use stidcart_core::{read, CartBuilder, Entry, InputError};

use crate::ListingDto;

/// One change the window asks for. Indices are the ones the listing shows; a menu is named as the
/// listing names it (D-042).
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Edit {
    Add { paths: Vec<String>, aligned: bool },
    Remove { index: usize },
    Rename { index: usize, name: String },
    Replace { index: usize, path: String },
    SetHidden { index: usize, hidden: bool },
    SetAligned { index: usize, aligned: bool },
    SetCartName { name: String },
    SetHeaderName { name: String },
    SetMenu { menu: String },
}

/// What the window knows about the open cart. Mirrored by `Document` in `gui/src/lib/types.ts`
/// and pinned by `gui/src/lib/fixtures/document.json` (`tests/contract.rs`).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentDto {
    /// The file name, or "Untitled" for a cart never saved.
    pub file: String,
    pub untitled: bool,
    pub listing: ListingDto,
    /// One per listing row: what the window may offer on it.
    pub rows: Vec<RowActions>,
    /// Why the cart cannot be edited, in the words `edit` uses; `None` when it can.
    pub read_only: Option<String>,
    pub dirty: bool,
    pub can_undo: bool,
    pub can_redo: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RowActions {
    /// 64 KB alignment applies: an xbank entry of an editable cart.
    pub alignable: bool,
}

struct Snapshot {
    bytes: Vec<u8>,
    listing: Listing,
    alignable: Vec<bool>,
}

impl Snapshot {
    fn new(bytes: Vec<u8>, listing: Listing, builder: Option<&CartBuilder>) -> Snapshot {
        let alignable = match builder {
            Some(b) => b
                .entries()
                .iter()
                .map(|e| e.as_cartridge().is_some_and(|c| c.kind.is_xbank()))
                .collect(),
            None => vec![false; listing.rows.len()],
        };
        Snapshot {
            bytes,
            listing,
            alignable,
        }
    }
}

pub struct Document {
    path: Option<PathBuf>,
    history: Vec<Snapshot>,
    at: usize,
    /// The step last saved, or opened; `None` once that step has been discarded by a new edit.
    saved: Option<usize>,
    read_only: Option<String>,
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Document {
    /// Open a cartridge. An error is the text `list` prints for it; a cart `list` describes but
    /// `edit` refuses opens view-only, carrying the reason `edit` gives.
    pub fn open(path: &Path) -> Result<Document, String> {
        let bytes =
            std::fs::read(path).map_err(|e| format!("reading '{}': {e}", path.display()))?;
        let listing = listing::describe(&bytes).map_err(|e| e.to_string())?;
        let (snapshot, read_only) = match read::load(&bytes) {
            Ok(builder) => (Snapshot::new(bytes, listing, Some(&builder)), None),
            Err(e) => (Snapshot::new(bytes, listing, None), Some(e.to_string())),
        };
        Ok(Document {
            path: Some(path.to_path_buf()),
            history: vec![snapshot],
            at: 0,
            saved: Some(0),
            read_only,
        })
    }

    /// An empty cart that has never been saved, with the names `build` gives one by default.
    pub fn new() -> Document {
        let builder = CartBuilder::new();
        let bytes = builder.build().expect("an empty cartridge always builds");
        let listing = listing::describe(&bytes).expect("a freshly built image describes");
        Document {
            path: None,
            history: vec![Snapshot::new(bytes, listing, Some(&builder))],
            at: 0,
            saved: Some(0),
            read_only: None,
        }
    }

    /// The image the window shows, which is exactly what saving now would write.
    pub fn bytes(&self) -> &[u8] {
        &self.current().bytes
    }

    pub fn dto(&self) -> DocumentDto {
        let snap = self.current();
        DocumentDto {
            file: self.path.as_deref().and_then(Path::file_name).map_or_else(
                || "Untitled".to_string(),
                |n| n.to_string_lossy().into_owned(),
            ),
            untitled: self.path.is_none(),
            listing: ListingDto::from(&snap.listing),
            rows: snap
                .alignable
                .iter()
                .map(|&alignable| RowActions { alignable })
                .collect(),
            read_only: self.read_only.clone(),
            dirty: self.is_dirty(),
            can_undo: self.at > 0,
            can_redo: self.at + 1 < self.history.len(),
        }
    }

    /// Apply one change. On success it becomes the next step and anything that could have been
    /// redone is gone; on failure the error is what `edit` says and nothing has changed.
    pub fn apply(&mut self, edit: Edit) -> Result<(), String> {
        if let Some(reason) = &self.read_only {
            return Err(reason.clone());
        }
        let mut builder = read::load(self.bytes()).map_err(|e| e.to_string())?;
        match edit {
            Edit::Add { paths, aligned } => {
                // Every file is read and classified before any is added: all of them or none.
                let entries = paths
                    .iter()
                    .map(|p| input_entry(p, aligned))
                    .collect::<Result<Vec<_>, _>>()?;
                for entry in entries {
                    builder.push(entry);
                }
            }
            Edit::Remove { index } => {
                check_index(&builder, index)?;
                builder.remove(index);
            }
            Edit::Rename { index, name } => {
                check_index(&builder, index)?;
                builder.set_name(index, &name);
            }
            Edit::Replace { index, path } => {
                check_index(&builder, index)?;
                builder.replace(index, input_entry(&path, false)?);
            }
            Edit::SetHidden { index, hidden } => {
                check_index(&builder, index)?;
                builder.set_hidden(index, hidden);
            }
            Edit::SetAligned { index, aligned } => {
                check_index(&builder, index)?;
                // The core ignores alignment on a program rather than refusing it, so the check
                // `edit --align` makes is repeated here, in its words, before a no-op becomes a step.
                let entry = &builder.entries()[index];
                if !entry.as_cartridge().is_some_and(|c| c.kind.is_xbank()) {
                    return Err(format!(
                        "entry {index} '{}' is not an xbank cartridge, so 64 KB alignment does not \
                         apply to it",
                        entry.name()
                    ));
                }
                builder.set_align_64k(index, aligned);
            }
            Edit::SetCartName { name } => builder.set_ef_name(&name),
            Edit::SetHeaderName { name } => builder.set_header_name(&name),
            Edit::SetMenu { menu } => {
                let menu = Menu::from_name(&menu).ok_or_else(|| {
                    format!("no menu named '{menu}'; it is EasyLoader or stidcart")
                })?;
                // The build below refuses a menu the cart can't carry, in the words `edit --menu`
                // uses, before anything becomes a step.
                builder.set_menu(menu);
            }
        }

        let bytes = builder.build().map_err(|e| e.to_string())?;
        let listing = listing::describe(&bytes).map_err(|e| e.to_string())?;
        self.history.truncate(self.at + 1);
        if self.saved.is_some_and(|s| s > self.at) {
            self.saved = None;
        }
        self.history
            .push(Snapshot::new(bytes, listing, Some(&builder)));
        self.at += 1;
        Ok(())
    }

    /// Step back one edit; false when there is none to undo.
    pub fn undo(&mut self) -> bool {
        if self.at == 0 {
            return false;
        }
        self.at -= 1;
        true
    }

    /// Step forward one undone edit; false when there is none to redo.
    pub fn redo(&mut self) -> bool {
        if self.at + 1 >= self.history.len() {
            return false;
        }
        self.at += 1;
        true
    }

    /// Replace the cart's file with the edited cart. A clean cart is left alone — rewriting it
    /// would re-place a cart another tool made without anyone asking — and an untitled one has
    /// nowhere to go, so the window asks for a path (Save As) instead.
    pub fn save(&mut self) -> Result<(), String> {
        if let Some(reason) = &self.read_only {
            return Err(reason.clone());
        }
        let Some(path) = self.path.clone() else {
            return Err("this cartridge has never been saved; choose where with Save As".into());
        };
        if !self.is_dirty() {
            return Ok(());
        }
        write_replacing(&path, self.bytes())?;
        self.saved = Some(self.at);
        Ok(())
    }

    /// Write the cart to `path` and make that the cart's file from now on. The file it was opened
    /// from is not touched.
    pub fn save_as(&mut self, path: &Path) -> Result<(), String> {
        if let Some(reason) = &self.read_only {
            return Err(reason.clone());
        }
        write_replacing(path, self.bytes())?;
        self.path = Some(path.to_path_buf());
        self.saved = Some(self.at);
        Ok(())
    }

    fn current(&self) -> &Snapshot {
        &self.history[self.at]
    }

    pub fn is_dirty(&self) -> bool {
        self.saved != Some(self.at)
    }
}

/// Bounds-check an index before it reaches the core, whose mutators panic on a bad one — and a
/// panic inside a command would poison the lock every later command needs. The text is `edit`'s.
fn check_index(builder: &CartBuilder, index: usize) -> Result<(), String> {
    let held = builder.entries().len();
    if index >= held {
        return Err(format!(
            "no entry at index {index}; the cartridge holds {held} {}",
            if held == 1 { "entry" } else { "entries" }
        ));
    }
    Ok(())
}

/// Read a file as the entry it is, named after its file stem, as `edit --add` does. What the file
/// is comes from the core (D-034); only the reading and the path in the error are here.
fn input_entry(path: &str, aligned: bool) -> Result<Entry, String> {
    let data = std::fs::read(path).map_err(|e| format!("reading '{path}': {e}"))?;
    let name = Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string());
    Entry::from_input(&name, data, aligned).map_err(|e| match e {
        InputError::Cartridge(e) => format!("'{path}': {e}"),
        other => other.to_string(),
    })
}

/// Write `bytes` to `path` through a temporary file in the same directory, then rename over it, so
/// the file is either the old cart or the new one and never part of one. The same approach as
/// `write_replacing` in the CLI (`crates/stidcart-cli/src/lib.rs`); I/O policy can't live in the
/// core, so each front end has its own copy. Not `tempfile`: it creates mode 0600, which would make
/// every saved cart owner-only (D-035).
fn write_replacing(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(".stidcart-tmp");
    let temporary = Path::new(&temporary);

    std::fs::write(temporary, bytes)
        .map_err(|e| format!("writing '{}': {e}", temporary.display()))?;
    std::fs::rename(temporary, path).map_err(|e| {
        let _ = std::fs::remove_file(temporary);
        format!("replacing '{}': {e}", path.display())
    })
}
