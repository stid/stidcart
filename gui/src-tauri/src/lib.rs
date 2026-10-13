//! The stidcart desktop app's Rust side: the commands the window calls.
//!
//! File reading and the wire format live here, not in the core (MAP: the core holds no I/O policy
//! and no serde). The values themselves come from `stidcart_core::listing`, which `list` renders
//! too, so the window and the command line cannot disagree (D-032). The open cart is a
//! [`document::Document`], held here for the one window (D-035).

pub mod document;
mod menu;

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use serde::Serialize;
use stidcart_core::listing::{self, Listing, Row};
use tauri::{AppHandle, Emitter, Manager, RunEvent, State};

use document::{Document, DocumentDto, Edit};

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

/// What the window's commands share: the open cart, and whether quitting has been cleared.
#[derive(Default)]
struct AppState {
    doc: Mutex<Option<Document>>,
    /// Set once the user has answered the unsaved-changes prompt (or there was nothing to ask), so
    /// the exit that follows a close or `exit_app` is not stopped and asked about a second time.
    exit_allowed: AtomicBool,
}

impl AppState {
    /// The open cart. A poisoned lock is taken over rather than propagated: nothing here holds
    /// the lock across a step that can leave the document half-changed.
    fn doc(&self) -> MutexGuard<'_, Option<Document>> {
        self.doc.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Run `f` on the open cart and answer with its new state, keeping the menu in step with it.
fn with_doc(
    app: &AppHandle,
    state: &State<AppState>,
    f: impl FnOnce(&mut Document) -> Result<(), String>,
) -> Result<DocumentDto, String> {
    let mut guard = state.doc();
    let doc = guard.as_mut().ok_or("no cartridge is open")?;
    f(doc)?;
    let dto = doc.dto();
    menu::sync(app, Some(&dto));
    Ok(dto)
}

#[tauri::command]
fn open_document(
    path: String,
    app: AppHandle,
    state: State<AppState>,
) -> Result<DocumentDto, String> {
    let mut guard = state.doc();
    match Document::open(Path::new(&path)) {
        Ok(doc) => {
            let dto = guard.insert(doc).dto();
            menu::sync(&app, Some(&dto));
            Ok(dto)
        }
        Err(e) => {
            // The window now shows this refusal, not the cart before it.
            *guard = None;
            menu::sync(&app, None);
            Err(e)
        }
    }
}

#[tauri::command]
fn new_document(app: AppHandle, state: State<AppState>) -> DocumentDto {
    let dto = state.doc().insert(Document::new()).dto();
    menu::sync(&app, Some(&dto));
    dto
}

#[tauri::command]
fn apply_edit(edit: Edit, app: AppHandle, state: State<AppState>) -> Result<DocumentDto, String> {
    with_doc(&app, &state, |d| d.apply(edit))
}

#[tauri::command]
fn undo(app: AppHandle, state: State<AppState>) -> Result<DocumentDto, String> {
    with_doc(&app, &state, |d| {
        d.undo();
        Ok(())
    })
}

#[tauri::command]
fn redo(app: AppHandle, state: State<AppState>) -> Result<DocumentDto, String> {
    with_doc(&app, &state, |d| {
        d.redo();
        Ok(())
    })
}

#[tauri::command]
fn save_document(app: AppHandle, state: State<AppState>) -> Result<DocumentDto, String> {
    with_doc(&app, &state, Document::save)
}

#[tauri::command]
fn save_document_as(
    path: String,
    app: AppHandle,
    state: State<AppState>,
) -> Result<DocumentDto, String> {
    with_doc(&app, &state, |d| d.save_as(Path::new(&path)))
}

/// The unsaved-changes question has been answered; let the window close.
#[tauri::command]
fn allow_exit(state: State<AppState>) {
    state.exit_allowed.store(true, Ordering::SeqCst);
}

/// Quit, once the window has asked about unsaved changes.
#[tauri::command]
fn exit_app(app: AppHandle, state: State<AppState>) {
    state.exit_allowed.store(true, Ordering::SeqCst);
    app.exit(0);
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .setup(|app| Ok(menu::install(app.handle())?))
        .invoke_handler(tauri::generate_handler![
            open_document,
            new_document,
            apply_edit,
            undo,
            redo,
            save_document,
            save_document_as,
            allow_exit,
            exit_app
        ])
        .build(tauri::generate_context!())
        .expect("error while building the stidcart app")
        .run(|app, event| {
            // An exit request with no code (closing the last window by a path the window did not
            // clear) while there are unsaved changes is stopped and handed to the window, which
            // asks and then calls `exit_app`, or doesn't. An exit with a code is `exit_app`'s own
            // and always goes through. ⌘Q never arrives here: it is the menu's own Quit item
            // (menu.rs), because the predefined one sends `terminate:`, which skips this event.
            if let RunEvent::ExitRequested {
                code: None, api, ..
            } = event
            {
                let state = app.state::<AppState>();
                let dirty = state.doc().as_ref().is_some_and(|d| d.is_dirty());
                if dirty && !state.exit_allowed.load(Ordering::SeqCst) {
                    api.prevent_exit();
                    let _ = app.emit("exit-requested", ());
                }
            }
        });
}
