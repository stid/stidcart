//! default-stidcart-menu AC-199 (RENDER): the window's New Cart starts with our stidcart menu
//! (D-064). `build` with no inputs writes nothing (as the reference does), so the comparison is with
//! the core's empty build under our menu.

use stidcart_core::menu::Menu;
use stidcart_core::CartBuilder;
use stidcart_gui::document::Document;

// AC-199 (RENDER): New Cart starts with our menu, and saving it untouched writes an empty cart with our menu
#[test]
fn ac_199_new_cart_starts_with_our_menu() {
    let mut doc = Document::new();
    assert_eq!(doc.dto().listing.menu, "stidcart");

    let dir = std::env::temp_dir().join("stidcart-gui-default-menu");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let saved = dir.join("new.crt");
    doc.save_as(&saved).unwrap();
    let empty = CartBuilder::new()
        .with_menu(Menu::Stidcart)
        .build()
        .unwrap();
    assert_eq!(std::fs::read(&saved).unwrap(), empty);
}
