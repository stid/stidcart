//! readable-sizes in the window: the listing it receives reads sizes as `list` prints them
//! (AC-115) and carries every exact byte count (AC-117), from the one core description (D-032,
//! D-044).

use std::path::{Path, PathBuf};

use stidcart_gui::open;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn cmixed() -> serde_json::Value {
    serde_json::to_value(open(&fixtures().join("cmixed.crt")).unwrap()).unwrap()
}

// AC-115 (RENDER): the window receives the readable sizes and free figure `list` prints
#[test]
fn ac_115_render_the_window_receives_readable_sizes() {
    let dto = cmixed();
    let sizes: Vec<&str> = dto["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["size"].as_str().unwrap())
        .collect();
    assert_eq!(
        sizes,
        [
            "8 KB",
            "16 KB",
            "16 KB",
            "32 KB",
            "102 bytes",
            "2.9 KB",
            "19.5 KB"
        ]
    );
    assert_eq!(dto["freeText"], "904 KB");
}

// AC-117 (RENDER): every size and the free figure carry their exact byte count
#[test]
fn ac_117_render_the_window_receives_exact_byte_counts() {
    let dto = cmixed();
    let exact: Vec<u64> = dto["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["sizeBytes"].as_u64().unwrap())
        .collect();
    assert_eq!(exact, [8192, 16384, 16384, 32768, 102, 3002, 20002]);
    assert_eq!(dto["free"], 925696);
}
