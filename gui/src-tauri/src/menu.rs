//! The app's native menu (gui-edit).
//!
//! The custom items only announce themselves: each one emits `menu` with its id, and the window
//! decides what it means (`viewer.svelte.ts`); Quit is one of them (see `build`). Undo and Redo
//! stay enabled, because they also undo inside a text field being edited, which the window routes
//! to the field. Cut, Copy, Paste and Select All are the standard items, so text fields keep their
//! native behaviour.

use tauri::menu::{
    Menu, MenuBuilder, MenuItem, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder,
};
use tauri::{AppHandle, Emitter, Manager, Wry};

use crate::document::DocumentDto;

/// The ids the window listens for, as the `menu` event's payload.
const IDS: [&str; 7] = ["new", "open", "save", "save-as", "undo", "redo", "quit"];

/// The items whose enabled state follows the open cart.
pub struct DocumentItems {
    save: MenuItem<Wry>,
    save_as: MenuItem<Wry>,
}

pub fn build(app: &AppHandle) -> tauri::Result<(Menu<Wry>, DocumentItems)> {
    let item = |id: &str, text: &str, keys: &str| {
        MenuItemBuilder::with_id(id, text)
            .accelerator(keys)
            .build(app)
    };
    let save = MenuItemBuilder::with_id("save", "Save")
        .accelerator("CmdOrCtrl+S")
        .enabled(false)
        .build(app)?;
    let save_as = MenuItemBuilder::with_id("save-as", "Save As…")
        .accelerator("CmdOrCtrl+Shift+S")
        .enabled(false)
        .build(app)?;

    let app_menu = SubmenuBuilder::new(app, "stidcart")
        .item(&PredefinedMenuItem::about(app, None, None)?)
        .separator()
        .item(&PredefinedMenuItem::hide(app, None)?)
        .item(&PredefinedMenuItem::hide_others(app, None)?)
        .separator()
        // Not the predefined Quit: it sends `terminate:`, which tao does not intercept, so the
        // process would end with no `ExitRequested` and unsaved changes would be lost unasked.
        // This one asks the window, which runs the unsaved-changes question first.
        .item(&item("quit", "Quit stidcart", "CmdOrCtrl+Q")?)
        .build()?;
    let file = SubmenuBuilder::new(app, "File")
        .item(&item("new", "New", "CmdOrCtrl+N")?)
        .item(&item("open", "Open…", "CmdOrCtrl+O")?)
        .separator()
        .item(&save)
        .item(&save_as)
        .separator()
        .item(&PredefinedMenuItem::close_window(app, None)?)
        .build()?;
    let edit = SubmenuBuilder::new(app, "Edit")
        .item(&item("undo", "Undo", "CmdOrCtrl+Z")?)
        .item(&item("redo", "Redo", "CmdOrCtrl+Shift+Z")?)
        .separator()
        .item(&PredefinedMenuItem::cut(app, None)?)
        .item(&PredefinedMenuItem::copy(app, None)?)
        .item(&PredefinedMenuItem::paste(app, None)?)
        .item(&PredefinedMenuItem::select_all(app, None)?)
        .build()?;
    let window = SubmenuBuilder::new(app, "Window")
        .item(&PredefinedMenuItem::minimize(app, None)?)
        .build()?;

    let menu = MenuBuilder::new(app)
        .items(&[&app_menu, &file, &edit, &window])
        .build()?;
    Ok((menu, DocumentItems { save, save_as }))
}

/// Install the menu and forward its custom items to the window.
pub fn install(app: &AppHandle) -> tauri::Result<()> {
    let (menu, items) = build(app)?;
    app.set_menu(menu)?;
    app.manage(items);
    app.on_menu_event(|app, event| {
        let id = event.id().0.as_str();
        if IDS.contains(&id) {
            let _ = app.emit("menu", id);
        }
    });
    Ok(())
}

/// Enable Save only when there is something to save, and Save As whenever a cart can be saved.
pub fn sync(app: &AppHandle, doc: Option<&DocumentDto>) {
    let Some(items) = app.try_state::<DocumentItems>() else {
        return;
    };
    let editable = doc.is_some_and(|d| d.read_only.is_none());
    let _ = items
        .save
        .set_enabled(editable && doc.is_some_and(|d| d.dirty));
    let _ = items.save_as.set_enabled(editable);
}
