//! The stidcart desktop viewer's Rust side: the commands the window calls.
//!
//! File reading and the wire format live here, not in the core (MAP: the core holds no I/O policy
//! and no serde). The values themselves come from `stidcart_core::listing`, which `list` renders
//! too, so the window and the command line cannot disagree (D-032).

pub mod document;

use std::path::Path;

use serde::Serialize;
use stidcart_core::listing::{self, Listing, Row};

/// A listing as the frontend receives it. Mirrored by `Listing` in `gui/src/lib/types.ts` and
/// pinned by the golden JSON in `gui/src/lib/fixtures/` (`tests/contract.rs`).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListingDto {
    pub cart: String,
    pub built_by: String,
    pub rows: Vec<RowDto>,
    pub free: usize,
    pub free_text: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RowDto {
    pub index: usize,
    pub name: String,
    pub flags: String,
    pub kind: String,
    pub bank: String,
    pub layout: String,
    pub size: String,
}

impl From<&Listing> for ListingDto {
    fn from(l: &Listing) -> Self {
        ListingDto {
            cart: l.cart.clone(),
            built_by: l.built_by.clone(),
            rows: l.rows.iter().map(RowDto::from).collect(),
            free: l.free,
            free_text: l.free_text(),
        }
    }
}

impl From<&Row> for RowDto {
    fn from(r: &Row) -> Self {
        RowDto {
            index: r.index,
            name: r.name.clone(),
            flags: r.flags.clone(),
            kind: r.kind.to_string(),
            bank: r.bank.clone(),
            layout: r.layout.clone(),
            size: r.size.clone(),
        }
    }
}

/// Read a cartridge and describe it. Only reads: the file is never opened for writing.
///
/// Errors are the text `list` prints after `stidcart: `, so a refusal reads the same in both.
pub fn open(path: &Path) -> Result<ListingDto, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("reading '{}': {e}", path.display()))?;
    let listing = listing::describe(&bytes).map_err(|e| e.to_string())?;
    Ok(ListingDto::from(&listing))
}

#[tauri::command]
fn open_cartridge(path: String) -> Result<ListingDto, String> {
    open(Path::new(&path))
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![open_cartridge])
        .run(tauri::generate_context!())
        .expect("error while running the stidcart viewer");
}
