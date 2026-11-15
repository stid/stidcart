//! The app's native menu (gui-edit).
//!
//! The custom items only announce themselves: each one emits `menu` with its id, and the window
//! decides what it means (`viewer.svelte.ts`); Quit is one of them (see `build`). Undo and Redo
//! stay enabled, because they also undo inside a text field being edited, which the window routes
//! to the field. Cut, Copy, Paste and Select All are the standard items, so text fields keep their
//! native behaviour.
//!
//! On a Mac the first menu is the app's own (About, Hide, Quit). Windows and Linux have no such
//! menu and draw the bar inside each window, so there Quit is at the end of File (Exit on Windows,
//! Quit on Linux), About is under Help, and the Hide items are left out: a hidden window has no
//! way back.

use tauri::menu::{
    AboutMetadataBuilder, Menu, MenuBuilder, MenuItem, MenuItemBuilder, PredefinedMenuItem,
    SubmenuBuilder,
};
use tauri::{AppHandle, Emitter, Manager, Wry};

use crate::document::DocumentDto;

/// The ids the window listens for, as the `menu` event's payload.
const IDS: [&str; 8] = [
    "new",
    "open",
    "new-folder",
    "save",
    "save-as",
    "undo",
    "redo",
    "quit",
];

/// Whether the menu bar sits inside the window, as on Windows and Linux, with no app menu.
const IN_WINDOW: bool = !cfg!(target_os = "macos");

/// View ▸ Hex Viewer. Handled here, not forwarded: it opens a window, which is this side's job.
const HEX: &str = "hex";

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

    // Not the predefined Quit: it sends `terminate:`, which tao does not intercept, so the
    // process would end with no `ExitRequested` and unsaved changes would be lost unasked.
    // This one asks the window, which runs the unsaved-changes question first.
    let quit = item(
        "quit",
        if cfg!(target_os = "windows") {
            "Exit"
        } else if IN_WINDOW {
            "Quit"
        } else {
            "Quit stidcart"
        },
        "CmdOrCtrl+Q",
    )?;

    let mut file = SubmenuBuilder::new(app, "File")
        .item(&item("new", "New", "CmdOrCtrl+N")?)
        .item(&item("open", "Open…", "CmdOrCtrl+O")?)
        .separator()
        // Acts on the open cart's folder tree; the window ignores it with no editable cart (D-063).
        .item(&item("new-folder", "New Folder", "CmdOrCtrl+Shift+N")?)
        .separator()
        .item(&save)
        .item(&save_as)
        .separator()
        .item(&PredefinedMenuItem::close_window(app, None)?);
    if IN_WINDOW {
        file = file.separator().item(&quit);
    }
    let file = file.build()?;
    let edit = SubmenuBuilder::new(app, "Edit")
        .item(&item("undo", "Undo", "CmdOrCtrl+Z")?)
        .item(&item("redo", "Redo", "CmdOrCtrl+Shift+Z")?)
        .separator()
        .item(&PredefinedMenuItem::cut(app, None)?)
        .item(&PredefinedMenuItem::copy(app, None)?)
        .item(&PredefinedMenuItem::paste(app, None)?)
        .item(&PredefinedMenuItem::select_all(app, None)?)
        .build()?;
    let view = SubmenuBuilder::new(app, "View")
        .item(&item(HEX, "Hex Viewer", "CmdOrCtrl+Shift+H")?)
        .build()?;
    let window = SubmenuBuilder::new(app, "Window")
        .item(&PredefinedMenuItem::minimize(app, None)?)
        .build()?;

    let menu = if IN_WINDOW {
        // Without metadata the item does nothing off a Mac: there is no bundle to read it from.
        let about = AboutMetadataBuilder::new()
            .name(Some("stidcart"))
            .version(Some(app.package_info().version.to_string()))
            // GTK's dialog draws a broken-image mark when it is given none.
            .icon(app.default_window_icon().cloned())
            .build();
        let help = SubmenuBuilder::new(app, "Help")
            .item(&PredefinedMenuItem::about(
                app,
                Some("About stidcart"),
                Some(about),
            )?)
            .build()?;
        MenuBuilder::new(app)
            .items(&[&file, &edit, &view, &window, &help])
            .build()?
    } else {
        let app_menu = SubmenuBuilder::new(app, "stidcart")
            .item(&PredefinedMenuItem::about(app, None, None)?)
            .separator()
            .item(&PredefinedMenuItem::hide(app, None)?)
            .item(&PredefinedMenuItem::hide_others(app, None)?)
            .separator()
            .item(&quit)
            .build()?;
        MenuBuilder::new(app)
            .items(&[&app_menu, &file, &edit, &view, &window])
            .build()?
    };
    Ok((menu, DocumentItems { save, save_as }))
}

/// Install the menu and forward its custom items to the window.
pub fn install(app: &AppHandle) -> tauri::Result<()> {
    let (menu, items) = build(app)?;
    app.set_menu(menu)?;
    app.manage(items);
    app.on_menu_event(|app, event| activate(app, event.id().0.as_str()));
    Ok(())
}

/// Act on a custom item: by a click, by its accelerator, or by its shortcut typed in a window on
/// Windows (`menu_key`). The standard items never arrive here.
pub fn activate(app: &AppHandle, id: &str) {
    if IDS.contains(&id) {
        let _ = app.emit("menu", id);
    } else if id == HEX {
        if let Err(e) = crate::show_hex_window(app) {
            eprintln!("stidcart: opening the hex viewer: {e}");
        }
    }
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

#[cfg(test)]
mod tests {
    use super::IDS;

    /// Every id the frontend's `MenuId` names is forwarded; one missing here is a menu item that does
    /// nothing (found live for File ▸ New Folder, D-063).
    #[test]
    fn every_menu_id_the_frontend_knows_is_forwarded() {
        let types =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../src/lib/types.ts"))
                .expect("types.ts");
        let line = types
            .lines()
            .find(|l| l.starts_with("export type MenuId ="))
            .expect("the MenuId type");
        let ids: Vec<&str> = line.split('\'').skip(1).step_by(2).collect();
        assert!(!ids.is_empty());
        for id in ids {
            assert!(
                IDS.contains(&id),
                "menu id '{id}' is not forwarded to the window"
            );
        }
    }
}
