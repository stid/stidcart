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
use stidcart_core::{read, CartBuilder};

use crate::ListingDto;

/// One change the window asks for. Indices are the ones the listing shows.
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
            dirty: self.dirty(),
            can_undo: self.at > 0,
            can_redo: self.at + 1 < self.history.len(),
        }
    }

    pub fn apply(&mut self, _edit: Edit) -> Result<(), String> {
        if let Some(reason) = &self.read_only {
            return Err(reason.clone());
        }
        Err("not implemented".into())
    }

    pub fn undo(&mut self) -> bool {
        false
    }

    pub fn redo(&mut self) -> bool {
        false
    }

    pub fn save(&mut self) -> Result<(), String> {
        Err("not implemented".into())
    }

    pub fn save_as(&mut self, _path: &Path) -> Result<(), String> {
        if let Some(reason) = &self.read_only {
            return Err(reason.clone());
        }
        Err("not implemented".into())
    }

    fn current(&self) -> &Snapshot {
        &self.history[self.at]
    }

    fn dirty(&self) -> bool {
        self.saved != Some(self.at)
    }
}
