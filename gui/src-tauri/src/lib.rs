//! The stidcart desktop viewer's Rust side: the commands the window calls.

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .run(tauri::generate_context!())
        .expect("error while running the stidcart viewer");
}
