//! What the sign-in surface draws, asked of the pixels it produced.
//!
//! **Every assertion reads the raster**, not the arguments that made it. A test
//! that checked the inputs would pass on a function that returned the ground
//! colour everywhere.
#![expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on a raster that is not there is the failure being reported"
)]

use super::{THE_GROUND_IS, TheEntryIs, sign_in_surface};
use crate::sign_in_layout::sign_in_laid_out;
use smithay::utils::{Scale, Size};

/// The reference screen, at one-to-one.
fn drawn(entry: TheEntryIs) -> (crate::painted::Washed, crate::sign_in_layout::SignInLayout) {
    let at = sign_in_laid_out(Size::from((1440, 960))).expect("the reference size lays out");
    let washed = sign_in_surface(&at, entry, Scale::from(1.0)).expect("a screen with area draws");
    (washed, at)
}

/// The pixel at a point, as red, green, blue.
fn at(washed: &crate::painted::Washed, x: i32, y: i32) -> [u8; 3] {
    let w = washed.area.size.w;
    let i = ((y * w + x) * 4) as usize;
    [washed.pixels[i], washed.pixels[i + 1], washed.pixels[i + 2]]
}

/// **The ground is the design's warm off-white, not white.**
///
/// The screen that shipped before this drew `#FFFFFF`, and warm off-white
/// against white is the kind of difference a person feels and cannot name.
#[test]
fn the_ground_is_the_warm_off_white_and_not_white() {
    let (washed, _) = drawn(TheEntryIs::Waiting);
    assert_eq!(at(&washed, 20, 20), THE_GROUND_IS);
    assert_eq!(THE_GROUND_IS, [0xFA, 0xF7, 0xF2]);
    assert_ne!(THE_GROUND_IS, [0xFF, 0xFF, 0xFF], "it must not be white");
}

/// **The raster is the whole screen**, four bytes a pixel.
#[test]
fn the_raster_covers_the_screen_exactly() {
    let (washed, at_) = drawn(TheEntryIs::Waiting);
    assert_eq!(washed.area.size.w, at_.screen.size.w);
    assert_eq!(washed.area.size.h, at_.screen.size.h);
    assert_eq!(
        washed.pixels.len(),
        (washed.area.size.w * washed.area.size.h * 4) as usize
    );
}

/// **The entry's middle is its own surface**, not the ground.
#[test]
fn the_entry_is_drawn_on_the_ground() {
    let (washed, layout) = drawn(TheEntryIs::Waiting);
    let middle = at(
        &washed,
        layout.entry.loc.x + layout.entry.size.w / 2,
        layout.entry.loc.y + layout.entry.size.h / 2,
    );
    assert_eq!(
        middle,
        [0xFF, 0xFF, 0xFF],
        "the entry's surface is bg/surface"
    );
    assert_ne!(middle, THE_GROUND_IS);
}

/// **The monogram is its own square**, `bg/cool` on the ground.
#[test]
fn the_monogram_is_drawn_in_its_own_colour() {
    let (washed, layout) = drawn(TheEntryIs::Waiting);
    let middle = at(
        &washed,
        layout.monogram.loc.x + layout.monogram.size.w / 2,
        layout.monogram.loc.y + layout.monogram.size.h / 2,
    );
    assert_eq!(middle, [0xEE, 0xF2, 0xF4]);
}

/// **The corners are rounded**, which is the whole reason this file exists.
///
/// The entry's own corner pixel must still be the ground: a square-cornered
/// rectangle would have drawn the entry's surface there. Measured one pixel in
/// from the corner, where an 18-unit radius leaves the shape well clear.
#[test]
fn the_entry_has_corners_rather_than_being_a_rectangle() {
    let (washed, layout) = drawn(TheEntryIs::Waiting);
    let corner = at(&washed, layout.entry.loc.x + 1, layout.entry.loc.y + 1);
    assert_eq!(
        corner, THE_GROUND_IS,
        "the entry's corner is filled, so it was drawn as a rectangle"
    );

    // The control: the same distance in along the flat top edge *is* the
    // entry, so the assertion above is about the corner and not about the
    // whole shape being absent.
    let along_the_top = at(
        &washed,
        layout.entry.loc.x + layout.entry.size.w / 2,
        layout.entry.loc.y + 1,
    );
    assert_ne!(
        along_the_top, THE_GROUND_IS,
        "the entry's top edge is not drawn at all"
    );
}

/// **The monogram's corners are rounded too.**
#[test]
fn the_monogram_has_corners_too() {
    let (washed, layout) = drawn(TheEntryIs::Waiting);
    assert_eq!(
        at(
            &washed,
            layout.monogram.loc.x + 1,
            layout.monogram.loc.y + 1
        ),
        THE_GROUND_IS
    );
    assert_ne!(
        at(
            &washed,
            layout.monogram.loc.x + layout.monogram.size.w / 2,
            layout.monogram.loc.y + 1
        ),
        THE_GROUND_IS,
        "the monogram's top edge is not drawn"
    );
}

/// **A refusal changes the border and nothing else.**
///
/// The two rasters must differ at the entry's edge and agree everywhere the
/// entry is not — a refusal that repainted the screen would be a flash.
#[test]
fn a_refusal_changes_the_border_and_leaves_the_rest_alone() {
    let (waiting, layout) = drawn(TheEntryIs::Waiting);
    let (refused, _) = drawn(TheEntryIs::Refused);

    // At the entry's left edge, the two differ.
    let y = layout.entry.loc.y + layout.entry.size.h / 2;
    assert_ne!(
        at(&waiting, layout.entry.loc.x, y),
        at(&refused, layout.entry.loc.x, y),
        "the border did not change on a refusal"
    );

    // Away from the entry, they are the same picture.
    for (x, y) in [
        (20, 20),
        (1400, 940),
        (layout.monogram.loc.x + 10, layout.monogram.loc.y + 10),
    ] {
        assert_eq!(
            at(&waiting, x, y),
            at(&refused, x, y),
            "a refusal repainted ({x}, {y}), which is not the entry"
        );
    }
}

/// **The entry's inside is unchanged by a refusal**, so what a person typed
/// does not change colour under them.
#[test]
fn a_refusal_does_not_touch_what_is_inside_the_entry() {
    let (waiting, layout) = drawn(TheEntryIs::Waiting);
    let (refused, _) = drawn(TheEntryIs::Refused);
    let middle = (
        layout.entry.loc.x + layout.entry.size.w / 2,
        layout.entry.loc.y + layout.entry.size.h / 2,
    );
    assert_eq!(
        at(&waiting, middle.0, middle.1),
        at(&refused, middle.0, middle.1)
    );
}

/// **It draws at a fractional scale**, which is one of the owner's checks.
#[test]
fn it_draws_at_fractional_scale() {
    let at_ = sign_in_laid_out(Size::from((1440, 960))).expect("lays out");
    for scale in [1.0, 1.25, 1.5, 2.0] {
        let washed = sign_in_surface(&at_, TheEntryIs::Waiting, Scale::from(scale))
            .expect("a screen with area draws at every scale");
        assert_eq!(
            washed.pixels.len(),
            (washed.area.size.w * washed.area.size.h * 4) as usize,
            "the raster is the wrong length at {scale}"
        );
        assert!(
            washed.area.size.w >= 1440,
            "scaling {scale} made the raster smaller than the logical screen"
        );
    }
}
