//! A cart saved from the window is the cart the command line writes (gui-edit AC-80, D-035).
//!
//! Each scenario is a list of steps, each pairing one window edit with the `edit` options that
//! request the same change. The CLI is chained one call per step (`edit prev <op> -o next`), so its
//! within-call reordering (AC-27) never comes into play, and like the document it re-reads the image
//! before every change. After every step, and after the final save, the bytes must be identical.

use std::path::{Path, PathBuf};

use stidcart_gui::document::{Document, Edit};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

fn fx(name: &str) -> String {
    fixtures().join(name).to_string_lossy().into_owned()
}

fn scratch(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("stidcart-gui-parity-{tag}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn run(args: &[String]) {
    stidcart_cli::run(args).unwrap_or_else(|e| panic!("stidcart {args:?}: {e}"));
}

fn s(v: &str) -> String {
    v.to_string()
}

fn edit_scenario(tag: &str, cart: &str, steps: Vec<(Edit, Vec<String>)>) {
    let dir = scratch(tag);
    let source = dir.join(cart);
    std::fs::copy(fixtures().join(cart), &source).unwrap();
    let mut doc = Document::open(&source).unwrap();
    let mut prev = source.clone();

    for (i, (edit, args)) in steps.into_iter().enumerate() {
        let next = dir.join(format!("step{i}.crt"));
        let mut cli = vec![s("edit"), prev.to_string_lossy().into_owned()];
        cli.extend(args.iter().cloned());
        cli.extend([s("-o"), next.to_string_lossy().into_owned()]);
        run(&cli);
        doc.apply(edit)
            .unwrap_or_else(|e| panic!("{tag} step {i}: {e}"));
        assert_eq!(
            doc.bytes(),
            std::fs::read(&next).unwrap().as_slice(),
            "{tag} step {i} ({args:?}): the window and edit disagree"
        );
        prev = next;
    }

    let saved = dir.join("saved.crt");
    doc.save_as(&saved).unwrap();
    assert_eq!(
        std::fs::read(&saved).unwrap(),
        std::fs::read(&prev).unwrap(),
        "{tag}: the saved file is edit's output"
    );
}

// AC-80: a cart saved after edits made in the window is byte-identical to edit's output, and a new
// cart saved after adding files is byte-identical to build's.
#[test]
fn ac_80_saved_edits_are_byte_identical_to_edit() {
    edit_scenario(
        "names-flags",
        "two.crt",
        vec![
            (
                Edit::Rename {
                    index: 0,
                    name: s("greetings"),
                },
                vec![s("--rename"), s("0"), s("greetings")],
            ),
            (
                Edit::SetHidden {
                    index: 1,
                    hidden: true,
                },
                vec![s("--hide"), s("1")],
            ),
            (
                Edit::Add {
                    paths: vec![fx("big.prg")],
                    aligned: false,
                },
                vec![s("--add"), fx("big.prg")],
            ),
            (Edit::Remove { index: 0 }, vec![s("--remove"), s("0")]),
            (
                Edit::SetHidden {
                    index: 0,
                    hidden: false,
                },
                vec![s("--show"), s("0")],
            ),
        ],
    );

    edit_scenario(
        "xbank",
        "cxb16.crt",
        vec![
            (
                Edit::SetAligned {
                    index: 1,
                    aligned: true,
                },
                vec![s("--align"), s("1")],
            ),
            (
                Edit::SetCartName {
                    name: s("menu name"),
                },
                vec![s("-e"), s("menu name")],
            ),
            (
                Edit::SetHeaderName {
                    name: s("header name"),
                },
                vec![s("--header"), s("header name")],
            ),
            (
                Edit::Replace {
                    index: 0,
                    path: fx("ramp.prg"),
                },
                vec![s("--replace"), s("0"), fx("ramp.prg")],
            ),
            (
                Edit::Add {
                    paths: vec![fx("in_xb8.crt")],
                    aligned: true,
                },
                vec![s("--add-aligned"), fx("in_xb8.crt")],
            ),
            (
                Edit::SetAligned {
                    index: 1,
                    aligned: false,
                },
                vec![s("--unalign"), s("1")],
            ),
        ],
    );

    edit_scenario(
        "mixed",
        "cmixed.crt",
        vec![
            (
                Edit::Add {
                    paths: vec![fx("hello.prg"), fx("in_8k.crt"), fx("ramp.prg")],
                    aligned: false,
                },
                vec![
                    s("--add"),
                    fx("hello.prg"),
                    s("--add"),
                    fx("in_8k.crt"),
                    s("--add"),
                    fx("ramp.prg"),
                ],
            ),
            (
                Edit::Replace {
                    index: 3,
                    path: fx("in_16k.crt"),
                },
                vec![s("--replace"), s("3"), fx("in_16k.crt")],
            ),
            (Edit::Remove { index: 6 }, vec![s("--remove"), s("6")]),
        ],
    );
}

// AC-80: New followed by adds saves exactly what `build` writes for the same files in order.
#[test]
fn ac_80_a_new_cart_with_added_files_is_byte_identical_to_build() {
    let dir = scratch("new");
    let inputs = [fx("hello.prg"), fx("ramp.prg"), fx("in_xb16.crt")];

    let built = dir.join("built.crt");
    let mut cli = vec![s("build"), s("-o"), built.to_string_lossy().into_owned()];
    cli.extend(inputs.iter().cloned());
    run(&cli);

    let mut doc = Document::new();
    doc.apply(Edit::Add {
        paths: inputs[..2].to_vec(),
        aligned: false,
    })
    .unwrap();
    doc.apply(Edit::Add {
        paths: inputs[2..].to_vec(),
        aligned: false,
    })
    .unwrap();
    let saved = dir.join("saved.crt");
    doc.save_as(&saved).unwrap();
    assert_eq!(
        std::fs::read(&saved).unwrap(),
        std::fs::read(&built).unwrap()
    );
}
