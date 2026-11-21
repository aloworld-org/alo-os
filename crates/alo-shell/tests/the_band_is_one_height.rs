//! **A window's band is one height, everywhere it is asked.**
//!
//! `docs/design/the-window-controls.md` measures the band at **54** (`11:31`),
//! and `crate::window_edge::THE_REGION_IS_TALL` draws it at 54. Until
//! 2026-11-21 two other constants answered the same question with **48**, the
//! name band of `docs/design/the-canvas-in-numbers.md` (read 2026-09-27, before
//! the band was redrawn): `frame_handle::THE_NAMES_BAND`, which the resize grips
//! are placed from, and `window_name_band::BAND_IS_TALL`, which a new window is
//! offset by. Two tests held the one band to two heights, and both passed. The
//! grips are asked before the band, so the top 6 of the drawn band were a resize
//! grip and not the handle that moves the window.
//!
//! This is the rule rather than the example: every height the crate gives a
//! window's band is the one named measure, and no source file declares another
//! band height as a number of its own.

#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::Path;

/// **The band a window is moved and resized by is the band it is drawn with.**
#[test]
fn every_answer_to_how_tall_a_band_is_is_the_drawn_band() {
    let drawn = alo_shell::THE_REGION_IS_TALL;
    assert_eq!(
        alo_shell::the_names_band(),
        f64::from(drawn),
        "the resize grips and the move handle are placed from a band of another height"
    );
    assert_eq!(
        alo_shell::BAND_IS_TALL,
        drawn,
        "a new window is offset by a band of another height"
    );
}

/// **No source file declares a band height as a number of its own.**
///
/// A constant whose name says it is a band's height must be the one measure or
/// be derived from it. The one exception is named: alo's working band across a
/// window's foot (`alos_band_in_a_window`, `11:173`), which is another band.
#[test]
fn no_file_declares_a_second_band_height() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut seen = 0;
    let mut second = Vec::new();
    for entry in std::fs::read_dir(&source).expect("this crate has a src directory") {
        let path = entry.expect("a directory entry can be read").path();
        let name = path
            .file_name()
            .map(|it| it.to_string_lossy().into_owned())
            .unwrap_or_default();
        if !name.ends_with(".rs") || name.ends_with("_tests.rs") {
            continue;
        }
        seen += 1;
        let text = std::fs::read_to_string(&path).expect("a source file can be read");
        for line in text.lines().map(str::trim_start) {
            let declares = line.contains("const ")
                && (line.contains("BAND_IS_TALL")
                    || line.contains("THE_NAMES_BAND")
                    || line.contains("THE_REGION_IS_TALL"));
            let value = line.split('=').nth(1).unwrap_or_default().trim();
            let a_number = value.starts_with(|it: char| it.is_ascii_digit());
            if declares && a_number && line.contains("THE_REGION_IS_TALL") {
                continue;
            }
            if declares && a_number && name != "alos_band_in_a_window.rs" {
                second.push(format!("{name}: {line}"));
            }
        }
    }
    assert!(
        seen > 100,
        "the control: this read {seen} files, not the crate"
    );
    assert!(
        second.is_empty(),
        "a band height is declared as a number of its own: {second:#?}"
    );
}
