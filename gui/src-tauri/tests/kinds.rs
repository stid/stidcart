//! readable-sizes in the window: each row carries its kind's family, so the type badge can take
//! the colour our menu gives that kind (AC-118, D-045).

use std::path::{Path, PathBuf};

use stidcart_gui::open;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

// AC-118 (RENDER): the window receives each row's family, from the entry type
#[test]
fn ac_118_render_the_window_receives_each_kind_family() {
    let families = |fixture: &str| -> Vec<String> {
        let dto = serde_json::to_value(open(&fixtures().join(fixture)).unwrap()).unwrap();
        dto["rows"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["family"].as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(
        families("cmixed.crt"),
        ["8k", "16k", "ult", "xb", "prg", "prg", "prg"]
    );
    let ocean = serde_json::to_value(open(&fixtures().join("cocean.crt")).unwrap()).unwrap();
    let row = ocean["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["kind"] == "OCEAN")
        .expect("cocean.crt lists an OCEAN entry");
    assert_eq!(row["family"], "other", "an Ocean entry has no menu colour");
}

// AC-118 (RENDER): every family our menu colours has a badge style, and colours in both themes
#[test]
fn ac_118_render_every_family_is_coloured_in_both_themes() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../src");
    let tokens = std::fs::read_to_string(src.join("styles/tokens.css")).unwrap();
    let listing = std::fs::read_to_string(src.join("lib/Listing.svelte")).unwrap();
    let (light, dark) = tokens
        .split_once("@media (prefers-color-scheme: dark)")
        .expect("tokens.css has a dark block");
    for family in ["prg", "8k", "16k", "ult", "xb"] {
        for part in ["bg", "text"] {
            let token = format!("--kind-{family}-{part}:");
            assert!(light.contains(&token), "no light {token}");
            assert!(dark.contains(&token), "no dark {token}");
            assert!(
                listing.contains(&format!("var(--kind-{family}-{part})")),
                "the badge style never uses --kind-{family}-{part}"
            );
        }
        assert!(
            listing.contains(&format!(".kind[data-family='{family}']")),
            "no badge style for {family}"
        );
    }
}
