//! **The panel must not reach the camera.** This file is what holds that.
//!
//! `alo-canvas` states the architecture the panel lives under: a plane that moves under a
//! viewport that does not, and *nothing in the viewport layer may read the camera to
//! correct itself. If it has to, it is in the wrong layer.* The panel is a viewport
//! surface.
//!
//! # Why a test and not an absent dependency
//!
//! Because the absent dependency was never the boundary it looked like. This crate's
//! manifest said for a while that not listing `alo-canvas` was what held the rule, and
//! proposed moving `Zoom` out of that crate's `camera` module so the dependency could be
//! taken. **Rust's privacy boundary is the crate, not the module.** `alo-canvas` declares
//! `pub mod camera`, so `alo_canvas::camera::Camera` is nameable from any crate that
//! depends on it, whatever module `Zoom` is exported from. The split would have bought a
//! tidier import and no boundary — and a boundary somebody believes in and does not have
//! is worse than none, because it is the one they stop checking.
//!
//! The dependency is taken for [`alo_canvas::Zoom`], which is what refuses 0 and 50_000.
//! The alternative was to store a zoom as raw thousandths and depend on nothing: a panel
//! holding a number nobody validated, which is a worse trade than a panel holding a
//! checked type beside a check that it holds nothing else.
//!
//! # Why it reads source and not behaviour
//!
//! A test asserting the panel is still at some coordinate after a pan **passes for a panel
//! that is in the wrong layer and compensating correctly** — that is `alo-canvas`'s own
//! warning about a dock that subtracts a pan to stay still. The question is not whether
//! the answer came out right, it is whether anything here can ask. A grep over this
//! crate's source either finds the name or does not, and it fails the moment somebody adds
//! the import.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported — \
              the same exemption `alo-dock`'s own test modules take, with its words. \
              `clippy::unwrap_used` is deliberately not in this list: clippy reported the \
              expectation as unfulfilled, which is the lint doing exactly its job, and \
              carrying a permission nothing needs is how a file ends up permitting more \
              than its author read"
)]

use std::path::{Path, PathBuf};

/// What no line of code in this crate may name.
///
/// Two entries and not more, because each one has to be a name that reaching actually
/// requires. `Camera` is the type — and it *is* the view: `alo-canvas` documents it as
/// *what a person is looking at*, holding the plane point under the viewport's corner and
/// the zoom. There is no separate view struct to forbid; the first draft of this list
/// invented one, which would have been a guard against a name nobody could write.
///
/// `camera::` is the module path, which reaches the same type while not spelling it.
const WHAT_THE_PANEL_MAY_NOT_REACH: &[&str] = &["Camera", "camera::"];

#[test]
fn no_source_file_in_this_crate_can_reach_the_camera() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut reaching: Vec<String> = Vec::new();
    for file in every_file(&src) {
        let text = std::fs::read_to_string(&file).expect("this crate's own source");
        for line in text.lines() {
            let code = only_the_code(line);
            for name in WHAT_THE_PANEL_MAY_NOT_REACH {
                if code.contains(name) {
                    reaching.push(format!("{}: {}", file.display(), line.trim()));
                }
            }
        }
    }
    assert!(
        reaching.is_empty(),
        "The panel must not reach the camera. It is a viewport surface, and alo-canvas's \
         rule is that nothing in the viewport layer may read the camera to correct itself \
         — if it has to, it is in the wrong layer.\n\n\
         This check is what holds that rule, and not the dependency graph: alo-canvas is \
         a dependency of this crate (for Zoom, which is checked where a raw number would \
         not be), and Rust's privacy boundary is the crate rather than the module, so \
         alo_canvas::camera::Camera is nameable from here. Nothing but this test stops \
         it. Deleting it does not simplify the crate, it removes the boundary.\n\n\
         What reaches:\n{reaching:#?}\n\n\
         If a zoom or a view is genuinely needed, it is handed in by whoever performed \
         the gesture — see Panel::put_aside, which takes one rather than asking for it."
    );
}

/// **And the manifest takes nothing else from that crate's layer.**
///
/// One dependency was argued for and one was taken. This is the check that the argument
/// was not then used as a door: `alo-canvas` is here for `Zoom`, and a second canvas-layer
/// crate appearing in this manifest is a decision nobody made.
#[test]
fn the_manifest_takes_the_canvas_for_one_reason_and_no_other() {
    let manifest =
        std::fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
            .expect("this crate's own manifest");
    let dependencies = manifest
        .split("[dependencies]")
        .nth(1)
        .unwrap_or_default()
        .split("\n[")
        .next()
        .unwrap_or_default();
    for reaching in [
        "alo-compositing",
        "alo-engine",
        "alo-arranging",
        "alo-desktops",
    ] {
        assert!(
            !dependencies.contains(reaching),
            "alo-put-aside depends on {reaching}, which is not a dependency this panel's \
             design argued for. alo-canvas is here for Zoom alone, and that argument is \
             not a licence to take the rest of the layer — a viewport surface that can \
             reach the compositor can correct itself, which is the fault the layer rule \
             exists to prevent."
        );
    }
}

/// A line with its documentation and its string literals taken out, so that a file which
/// **names** the camera in order to forbid it is not read as reaching for it.
///
/// This crate's `lib.rs`, its manifest comment and this very file say `Camera` on purpose,
/// in the sentences explaining why no code may. Without this, the check would fail on its
/// own explanation — and the fix somebody reached for would be to delete the explanation.
fn only_the_code(line: &str) -> String {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        return String::new();
    }
    let mut code = String::new();
    let mut inside_a_string = false;
    let mut letters = line.chars().peekable();
    while let Some(letter) = letters.next() {
        match letter {
            '\\' if inside_a_string => {
                letters.next();
            }
            '"' => inside_a_string = !inside_a_string,
            _ if !inside_a_string => code.push(letter),
            _ => {}
        }
    }
    code
}

/// Every `.rs` under a folder.
fn every_file(folder: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let Ok(entries) = std::fs::read_dir(folder) else {
        return found;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            found.extend(every_file(&path));
        } else if path.extension().is_some_and(|kind| kind == "rs") {
            found.push(path);
        }
    }
    found
}
