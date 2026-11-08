//! What the window's webview may ask of Tauri. A permission missing here fails silently at run
//! time (the call is refused, nothing is logged), so each one a frontend feature needs is pinned.

use std::path::Path;

fn permissions() -> Vec<String> {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("capabilities/default.json");
    let json: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    json["permissions"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p.as_str().unwrap().to_string())
        .collect()
}

/// The title bar is a `data-tauri-drag-region`: dragging it moves the window only when the webview
/// may start a drag. `core:default` grants the window's getters, not this.
#[test]
fn the_title_bar_can_drag_the_window() {
    assert!(
        permissions()
            .iter()
            .any(|p| p == "core:window:allow-start-dragging"),
        "capabilities/default.json must grant core:window:allow-start-dragging"
    );
}
