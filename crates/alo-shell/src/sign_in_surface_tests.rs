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

use super::{TheColours, TheEntryIs, sign_in_surface};
use crate::sign_in_layout::sign_in_laid_out;

/// The designed light palette, which is what these pixels were read against.
fn designed() -> TheColours {
    TheColours::of(crate::Contrast::AsDesigned, alo_appearance::Scheme::Light)
}
use smithay::utils::{Scale, Size};

/// The reference screen, at one-to-one.
fn drawn(entry: TheEntryIs) -> (crate::painted::Washed, crate::sign_in_layout::SignInLayout) {
    let at = sign_in_laid_out(Size::from((1440, 960))).expect("the reference size lays out");
    let washed = sign_in_surface(&at, (1440, 960), entry, Scale::from(1.0), designed())
        .expect("a screen with area draws");
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
    assert_eq!(at(&washed, 20, 20), designed().ground);
    assert_eq!(designed().ground, [0xFA, 0xF7, 0xF2]);
    assert_ne!(
        designed().ground,
        [0xFF, 0xFF, 0xFF],
        "it must not be white"
    );
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
    assert_ne!(middle, designed().ground);
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
        corner,
        designed().ground,
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
        along_the_top,
        designed().ground,
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
        designed().ground
    );
    assert_ne!(
        at(
            &washed,
            layout.monogram.loc.x + layout.monogram.size.w / 2,
            layout.monogram.loc.y + 1
        ),
        designed().ground,
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
    for (scale, pixels) in [
        (1.0, (1440, 960)),
        (1.25, (1800, 1200)),
        (1.5, (2160, 1440)),
        (2.0, (2880, 1920)),
    ] {
        let washed = sign_in_surface(
            &at_,
            pixels,
            TheEntryIs::Waiting,
            Scale::from(scale),
            designed(),
        )
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

/// **The whole output is painted, at a scale that does not divide it.** 1366
/// at 300 per cent is 455 logical pixels, which is 1365 again: the raster is
/// the output's own size, so no strip at its edge is left unpainted.
#[test]
fn the_whole_output_is_painted_at_a_scale_that_does_not_divide_it() {
    let at_ = sign_in_laid_out(Size::from((455, 384))).expect("lays out");
    let washed = sign_in_surface(
        &at_,
        (1366, 1152),
        TheEntryIs::Waiting,
        Scale::from(3.0),
        designed(),
    )
    .expect("draws");
    assert_eq!(
        (washed.area.size.w, washed.area.size.h),
        (1366, 1152),
        "the raster is not the output"
    );
}

/// **The light colours are still the design's own**, to the unit where the
/// palette answers and within three where they are mixed.
///
/// This is what makes taking the colours from the palette a correction rather
/// than a redesign: in light, at the designed contrast, the screen draws the
/// same pixels it drew when these were seven written-down hexes.
#[test]
fn the_light_colours_are_still_the_designs() {
    let it = designed();
    assert_eq!(it.ground, [0xFA, 0xF7, 0xF2], "the ground is not #FAF7F2");
    assert_eq!(it.entry, [0xFF, 0xFF, 0xFF], "the entry is not #FFFFFF");
    assert_eq!(it.ink, [0x10, 0x2A, 0x43], "the ink is not #102A43");

    // The four with no palette role: the design's own recorded values.
    for (what, drawn, designed) in [
        ("the monogram, bg/cool", it.monogram, [0xEE_u8, 0xF2, 0xF4]),
        (
            "the border, border/default",
            it.border,
            [0xE7_u8, 0xEB, 0xEF],
        ),
        (
            "the placeholder, text/muted",
            it.muted,
            [0x59_u8, 0x6B, 0x78],
        ),
    ] {
        for (channel, (a, b)) in drawn.iter().zip(designed.iter()).enumerate() {
            assert!(
                u32::from(*a).abs_diff(u32::from(*b)) <= 3,
                "{what}: channel {channel} is {a} and the design says {b}"
            );
        }
    }
}

/// **Every scheme and every contrast draws a readable screen.**
///
/// The regression this file exists to stop: the colours were fixed light-mode
/// values while the text was drawn in `Contrast::ink`, which follows the
/// scheme. In dark the ink turned light and landed on a light ground — at
/// least one label at a contrast of **zero** against its own background.
///
/// Read as the distance between the ink and each surface it is drawn on, in
/// every combination a person can choose.
#[test]
fn every_scheme_and_contrast_draws_a_readable_screen() {
    for contrast in [crate::Contrast::AsDesigned, crate::Contrast::High] {
        for scheme in [alo_appearance::Scheme::Light, alo_appearance::Scheme::Dark] {
            let it = TheColours::of(contrast, scheme);
            let apart = |a: [u8; 3], b: [u8; 3]| {
                u32::from(a[0]).abs_diff(u32::from(b[0]))
                    + u32::from(a[1]).abs_diff(u32::from(b[1]))
                    + u32::from(a[2]).abs_diff(u32::from(b[2]))
            };
            for (what, surface) in [
                ("the ground", it.ground),
                ("the entry", it.entry),
                ("the monogram", it.monogram),
            ] {
                assert!(
                    apart(it.ink, surface) > 200,
                    "{contrast:?}/{scheme:?}: ink on {what} is only {} apart, which is a \
                     screen somebody cannot read",
                    apart(it.ink, surface)
                );
            }
            // The placeholder is softer than the ink by design, and must still
            // be legible on the field it sits in.
            assert!(
                apart(it.muted, it.entry) > 90,
                "{contrast:?}/{scheme:?}: the placeholder is only {} from the field",
                apart(it.muted, it.entry)
            );
        }
    }
}
