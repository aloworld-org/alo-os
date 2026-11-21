//! **Every window's controls are drawn through the one band, and shown only
//! as its reveal says.**
//!
//! `docs/design/the-window-controls.md`: *No controls are drawn at rest.* On a
//! machine booted from `3959dd65` (2026-11-14) the terminal's and the task
//! windows' bands drew Minimise and Close with the pointer in the far corner,
//! beside applications whose bands drew none. Both went through the band's own
//! painter; what differed was that each of alo's windows had a road of its own
//! to it, and each road said *shown, all the way* as a constant. So this reads
//! `src/` and holds the roads, rather than a list kept beside them:
//!
//! 1. **Exactly four files draw a band** (`EdgePicture::of`): the
//!    applications' (`every_windows_edge.rs`), alo's own windows'
//!    (`an_alo_windows_edge.rs`, `the_terminals_edge.rs`) and the task
//!    windows' (`a_task_windows_edge.rs`). A fifth is a new road and has to be
//!    added here on purpose.
//! 2. **No road says how shown its controls are as a constant**: `ALL_THE_WAY`
//!    is named only by the fade and the reveal, and no `how_shown:` is a
//!    number.
//! 3. **alo's three roads ask the band's reveal**
//!    (`crate::the_bands_over_alos_windows`), which is told by the same function
//!    every application's band is.
//! 4. **The old strip is never drawn on a booted machine.**
//!    `window_control_paint.rs` — 32 × 32 boxes with a pressed look, from
//!    before the specification — still draws for the nested development
//!    window and the offscreen harness; no `direct_*` file, which is what a
//!    booted machine runs, builds the scene that paints it.
//!
//! # What would have to be true for this to pass while the product is broken
//!
//! A road could say *shown* through a variable this does not recognise — a
//! `let all = 1000;` handed to `how_shown` — or a window could paint controls
//! into its own picture without any band at all, as the record's own title
//! strip once did (`record_frame_drawn`). Neither is visible to a reading of
//! the source by name. The first is what
//! `the_bands_over_alos_windows_tests.rs` catches, by counting the controls in
//! each window's real picture at rest; the second is caught only by a booted
//! machine — the KVM walk's *the terminal at rest* step.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

/// This crate's own source, wherever it is checked out.
fn source() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Every Rust file under `src/` that is not itself a test, with its code
/// lines: comments are what a file says about itself, not what it does.
fn every_source_file() -> Vec<(String, Vec<String>)> {
    let mut found = Vec::new();
    let mut to_walk = vec![source()];
    while let Some(directory) = to_walk.pop() {
        for entry in std::fs::read_dir(&directory).expect("this crate has a src directory") {
            let path = entry.expect("a directory entry can be read").path();
            let name = path
                .file_name()
                .map(|it| it.to_string_lossy().into_owned())
                .unwrap_or_default();
            if path.is_dir() {
                to_walk.push(path);
            } else if name.ends_with(".rs") && !name.ends_with("_tests.rs") {
                let text = std::fs::read_to_string(&path).expect("a source file can be read");
                let code = text
                    .lines()
                    .filter(|line| !line.trim_start().starts_with("//"))
                    .map(str::to_owned)
                    .collect();
                found.push((name, code));
            }
        }
    }
    found.sort();
    assert!(
        found.len() > 100,
        "the control: this read {} files, not the crate",
        found.len()
    );
    found
}

/// The files whose code contains `needle`.
fn the_files_saying(needle: &str) -> Vec<String> {
    every_source_file()
        .into_iter()
        .filter(|(_, code)| code.iter().any(|line| line.contains(needle)))
        .map(|(name, _)| name)
        .collect()
}

/// **Four roads draw a band, and no more.**
#[test]
fn exactly_four_files_draw_a_band() {
    assert_eq!(
        the_files_saying("EdgePicture::of("),
        vec![
            "a_task_windows_edge.rs",
            "an_alo_windows_edge.rs",
            "every_windows_edge.rs",
            "the_terminals_edge.rs",
        ],
        "a band is drawn by a road this test does not know"
    );
}

/// **Nothing draws controls shown as a constant.**
#[test]
fn no_road_draws_its_controls_shown_as_a_constant() {
    assert_eq!(
        the_files_saying("ALL_THE_WAY"),
        vec!["the_controls_fade.rs", "the_window_edge_reveals.rs"],
        "controls are drawn all the way shown outside the fade and the reveal"
    );
    for (name, code) in every_source_file() {
        for line in code.iter().filter(|line| line.contains("how_shown:")) {
            let value = line.split("how_shown:").nth(1).unwrap_or_default().trim();
            assert!(
                !value.starts_with(|it: char| it.is_ascii_digit()),
                "{name} draws its controls at a fixed {value}"
            );
        }
    }
}

/// **alo's own windows' bands ask the reveal every band is told by.**
#[test]
fn alos_roads_ask_the_bands_reveal() {
    for road in [
        "a_task_windows_edge.rs",
        "an_alo_windows_edge.rs",
        "the_terminals_edge.rs",
    ] {
        let (_, code) = every_source_file()
            .into_iter()
            .find(|(name, _)| name == road)
            .expect("the road exists");
        assert!(
            code.iter().any(|line| line.contains("alos_band_as_drawn(")),
            "{road} draws a band without asking whether it is revealed"
        );
        assert!(
            !code
                .iter()
                .any(|line| line.contains("alos_band_as_pressed(")),
            "{road} finds a press on its band itself, beside the one press road"
        );
    }
    let (_, reveals) = every_source_file()
        .into_iter()
        .find(|(name, _)| name == "the_bands_over_alos_windows.rs")
        .expect("the bands over alo's windows have a file");
    assert!(
        reveals.iter().any(|line| line.contains("a_band_is_told(")),
        "alo's bands are told by a rule of their own"
    );
}

/// **One press road for every band.** Since 2026-11-21 a press on any band's
/// control, by the pointer, a screen reader or a key, is carried out by
/// `a_band_is_pressed.rs` and nowhere else, so a fix to what a control does
/// reaches every kind of window. Held two ways: only that file asks where a
/// press lands on alo's bands, and no file matches a press on one named
/// control to an action of its own (`Control(OnTheEdge::Close) =>`), which is
/// the shape each of the six old roads had.
#[test]
fn every_band_is_pressed_through_one_road() {
    assert_eq!(
        the_files_saying("alos_band_as_pressed("),
        vec!["a_band_is_pressed.rs", "the_bands_over_alos_windows.rs"],
        "a press on alo's band is found by a road beside the one press road"
    );
    for (name, code) in every_source_file() {
        for line in &code {
            assert!(
                !(line.contains("UnderThePointer::Control(OnTheEdge::")
                    || line.contains("ItRuns(OnTheEdge::Close)")
                    || line.contains("ItRuns(OnTheEdge::Minimise)")),
                "{name} carries out one named control itself: {line}"
            );
        }
    }
    let (_, one) = every_source_file()
        .into_iter()
        .find(|(name, _)| name == "a_band_is_pressed.rs")
        .expect("the one press road has a file");
    assert!(
        one.iter()
            .any(|line| line.contains("fn a_bands_control_is_pressed(")),
        "the one press road lost its one function"
    );
}

/// **The old strip is never drawn on a booted machine.**
#[test]
fn a_booted_machine_never_builds_the_old_strip() {
    for (name, code) in every_source_file()
        .into_iter()
        .filter(|(name, _)| name.starts_with("direct_"))
    {
        for line in code
            .iter()
            .filter(|line| line.contains("NativeScene::Controls"))
        {
            assert!(
                line.contains("NativeScene::Controls(_)"),
                "{name} builds the old window-control strip: {line}"
            );
        }
    }
}
