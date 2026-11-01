//! **At 200 per cent every first-start sheet is drawn at twice its size**, and
//! a screen with too little room for that is drawn at the largest scale that
//! still holds the sheet rather than not at all.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use super::the_first_start_at_one_hundred_tests::{every_sheet, hash_of, the_sheet};
use crate::display_scale::DisplayScale;

/// How far a line of type may land from exactly twice its place, in pixels.
///
/// Type is shaped at each size on its own, and a glyph hinted at 32 is not
/// precisely a glyph hinted at 16 drawn twice as large, so a line's inked box
/// can begin or end a few pixels from double. The rectangles the type is placed
/// in are exact (`crate::setup_sheet_in_pixels`'s own tests).
const NEARLY: i32 = 6;

#[test]
fn at_two_hundred_every_line_of_every_sheet_is_twice_its_size_and_place() {
    for keys in every_sheet() {
        let one = the_sheet(&keys, (1440, 960), DisplayScale::ONE).unwrap();
        let two = the_sheet(&keys, (2880, 1920), DisplayScale::of_per_cent(200)).unwrap();
        assert_eq!(
            one.inked.len(),
            two.inked.len(),
            "the same pieces of text, on {keys:?}"
        );
        for (small, large) in one.inked.iter().zip(&two.inked) {
            let (a, b) = (small.area, large.area);
            for (doubled, drawn) in [
                (2 * a.loc.x, b.loc.x),
                (2 * a.loc.y, b.loc.y),
                (2 * a.size.w, b.size.w),
                (2 * a.size.h, b.size.h),
            ] {
                assert!(
                    (doubled - drawn).abs() <= NEARLY,
                    "on {keys:?}: {a:?} at one is {b:?} at two"
                );
            }
        }
        // The cards and the wallpaper cover the whole output, not a corner.
        let surface = two.surface.as_ref().unwrap();
        assert_eq!((surface.area.size.w, surface.area.size.h), (2880, 1920));
        assert_eq!(surface.area.loc.x, 0);
        assert_eq!(surface.area.loc.y, 0);
        // And the solids too, such as the PIN's dots: **exactly twice as
        // wide, and as near twice as far across as a line of type is.** Since
        // 2026-11-01 one solid follows type — the caret after what was typed
        // in a field — and type at 32 is not exactly type at 16 doubled.
        assert_eq!(one.solids.len(), two.solids.len(), "on {keys:?}");
        for (small, large) in one.solids.iter().zip(&two.solids) {
            assert_eq!(2 * small.area.size.w, large.area.size.w, "on {keys:?}");
            assert!(
                (2 * small.area.loc.x - large.area.loc.x).abs() <= NEARLY,
                "on {keys:?}: {small:?} at one is {large:?} at two"
            );
        }
    }
}

/// **A dense 1080p laptop is worked out at 175, where no sheet fits**, and is
/// drawn at the largest scale that does: larger than one-to-one and never
/// blank.
#[test]
fn a_screen_with_too_little_room_is_held_to_what_fits() {
    for keys in every_sheet() {
        let held = the_sheet(&keys, (1920, 1080), DisplayScale::of_per_cent(175)).unwrap();
        let one = the_sheet(&keys, (1920, 1080), DisplayScale::ONE).unwrap();
        assert_ne!(hash_of(&held), hash_of(&one), "on {keys:?}");
        let surface = held.surface.as_ref().unwrap();
        assert_eq!((surface.area.size.w, surface.area.size.h), (1920, 1080));
    }
}

/// **At a scale that does not divide the screen the cards still cover it**:
/// 1801 × 1126 at 125 is 1440.8 × 900.8 logical, which comes back a pixel
/// short each way when multiplied again.
#[test]
fn at_a_fractional_scale_the_surface_is_the_whole_output() {
    let drawn = the_sheet(&[], (1801, 1126), DisplayScale::of_per_cent(125)).unwrap();
    let surface = drawn.surface.as_ref().unwrap();
    assert_eq!((surface.area.size.w, surface.area.size.h), (1801, 1126));
}
