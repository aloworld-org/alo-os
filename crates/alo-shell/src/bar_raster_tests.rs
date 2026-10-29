#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
//! **The panel that is rasterised covers the panel that was laid out.**
//!
//! There was no test here at all, and the one construction site of
//! `TheBarOnTheDisplay` is a frame loop no test runs — which is how the Bar
//! shipped drawing its panel at **less than half** the width it had asked for.
//!
//! Measured on a booted machine on 2026-10-23, on a 1280-wide display: the
//! layout asked for a panel 604 wide at x=338, and the texture came back 276
//! wide, ending at x=604. The field and the status line were laid out
//! correctly and ran straight out of the panel onto bare canvas.
//!
//! The cause was one argument. `dock_surface::ADocksSurface::room` is **the
//! screen**, and the rasteriser clamps the texture to it so a shadow cannot
//! run off the display — `right = (x + w + reach).min(room.0)`. The Bar
//! passed its own panel's size there, so the clamp cut the panel off at its
//! own width. `dock_raster` has always passed the display and says so on the
//! field; the Bar was the one caller that did not.

use super::*;
use crate::where_the_bar_opens::where_the_bar_opens;

/// Where the Bar is laid out on `display` at one to one.
fn laid_out(display: Size<i32, Physical>) -> TheBarsPlaces {
    where_the_bar_opens(display, DisplayScale::ONE).expect("this display holds a Bar")
}
use cosmic_text::FontSystem;
use smithay::utils::Size;

/// A palette that does not depend on a person's settings.
fn a_palette() -> crate::desktop_look::DesktopPalette {
    crate::desktop_look::DesktopPalette::of(
        alo_appearance::Scheme::Light,
        alo_appearance::Accent::Indigo.on(alo_appearance::Scheme::Light),
        crate::Contrast::AsDesigned,
    )
    // **Indigo and not deep teal.** Deep teal is `Reserved` — it means the
    // agent acting and a person may not choose it, which `AccentError` says
    // and this test found out by asking for it.
    .expect("indigo is one of the four accents a person may choose")
}

/// The Bar as the frame loop builds it, for one display size at one to one.
fn a_bar(display: Size<i32, Physical>) -> Option<BarPicture> {
    let mut fonts = FontSystem::new();
    picture(
        &TheBarOnTheDisplay {
            typed: "hello",
            origin: "On this computer",
            results: &[],
            chosen: None,
            display,
        },
        a_palette(),
        &mut fonts,
        DisplayScale::ONE,
    )
}

/// **The texture covers the whole panel, on every display size.**
///
/// The far edge is the one that was wrong, and it is the one a clamp against
/// the wrong number cuts, so it is asserted by name rather than by comparing
/// a width — a width can be right while the panel sits in the wrong place.
#[test]
fn the_rasterised_panel_covers_the_panel_that_was_laid_out() {
    for (w, h) in [(1280, 800), (1440, 960), (1920, 1080), (2560, 1440)] {
        let display = Size::from((w, h));
        let places = laid_out(display);
        let drawn = a_bar(display).expect("a Bar that was laid out can be drawn");

        let wanted_right = places.panel.loc.x + places.panel.size.w;
        let drawn_right = drawn.panel.area.loc.x + drawn.panel.area.size.w;
        assert!(
            drawn_right >= wanted_right,
            "on {w}x{h} the layout asked for a panel reaching x={wanted_right} and the texture \
             stops at x={drawn_right} — the panel is cut off and everything inside it draws onto \
             bare canvas"
        );
        let wanted_bottom = places.panel.loc.y + places.panel.size.h;
        let drawn_bottom = drawn.panel.area.loc.y + drawn.panel.area.size.h;
        assert!(
            drawn_bottom >= wanted_bottom,
            "on {w}x{h} the panel is cut off at the bottom: wanted y={wanted_bottom}, \
             got y={drawn_bottom}"
        );
        assert!(
            drawn.panel.area.loc.x <= places.panel.loc.x,
            "on {w}x{h} the texture starts after the panel does, so its leading edge is cut"
        );
    }
}

/// **The field a person types into is inside the panel it is drawn on.**
///
/// The control at the other end. The assertion above would pass on a texture
/// that covered the whole display, which is a different fault — and this one
/// is what a person actually saw: a field laid out past the edge of its own
/// panel.
#[test]
fn the_field_sits_inside_the_panel() {
    let display = Size::from((1280, 800));
    let places = laid_out(display);
    let drawn = a_bar(display).expect("a Bar that was laid out can be drawn");

    let field_right = places.field.loc.x + places.field.size.w;
    let panel_right = drawn.panel.area.loc.x + drawn.panel.area.size.w;
    assert!(
        field_right <= panel_right,
        "the field reaches x={field_right} and the drawn panel ends at x={panel_right}, so what \
         a person types is drawn on bare canvas"
    );
}

/// **The texture is never wider than the display**, which is what `room` is
/// for: a shadow that ran off the screen would be a buffer nobody can show.
#[test]
fn the_texture_stays_on_the_display() {
    let display = Size::from((1280, 800));
    let drawn = a_bar(display).expect("a Bar that was laid out can be drawn");
    assert!(
        drawn.panel.area.loc.x >= 0 && drawn.panel.area.loc.y >= 0,
        "the texture starts off the display"
    );
    assert!(
        drawn.panel.area.loc.x + drawn.panel.area.size.w <= display.w,
        "the texture runs off the right of the display"
    );
    assert!(
        drawn.panel.area.loc.y + drawn.panel.area.size.h <= display.h,
        "the texture runs off the bottom of the display"
    );
}
