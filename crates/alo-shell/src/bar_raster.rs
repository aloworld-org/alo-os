//! Drawing the alo Bar's open panel: the field, the line under it, and the
//! rows.
//!
//! **The geometry is `crate::where_the_bar_opens`' and the words are
//! `crate::the_bar_finds`'.** This file turns the two into pixels and decides
//! nothing of its own — the fault `crate::dock_raster` records the cost of is a
//! raster with arithmetic of its own, where what a person presses and what
//! lit up are worked out twice and disagree.
//!
//! # The panel is the Dock's own surface, drawn somewhere else
//!
//! `crate::dock_surface` rasterises a rounded panel with its shadow, its
//! hairline and its wells, and takes where it goes as an argument. So the Bar
//! is that surface at the Bar's rectangle, with a well per row — which is why
//! a row looks like the field above it without this file saying so twice.
//!
//! # Every row is drawn, and only the ones that fit are placed
//!
//! `TheBarsPlaces::rows_that_fit` counts whole rows. The design draws a fourth
//! cut off by the panel's edge, which is the list saying it scrolls; the
//! results beyond that are real and are simply not on the screen yet.

use cosmic_text::{FontSystem, Metrics};
use smithay::utils::{Physical, Rectangle, Size};

use crate::painted::{Inked, Washed};
use crate::the_bar_finds::AResult;
use crate::where_the_bar_opens::TheBarsPlaces;

/// What the Bar shows, as a frame hands it over.
#[derive(Debug, Clone, Copy)]
pub struct TheBarOnTheDisplay<'a> {
    /// Where it all goes.
    pub places: TheBarsPlaces,
    /// What a person has typed, which the field shows.
    pub typed: &'a str,
    /// The line under the field.
    pub origin: &'a str,
    /// Every result, in the order to read them.
    pub results: &'a [AResult],
    /// Which row the keyboard is on, if any.
    pub chosen: Option<usize>,
    /// **The display, so the panel's shadow can be kept on it.**
    ///
    /// `crate::dock_surface::ADocksSurface::room` is the screen, and it is
    /// what the rasteriser clamps the texture to so a shadow does not run off
    /// the edge. This field exists because the Bar passed **its own panel's
    /// size** there instead, and the clamp then cut the panel off at its own
    /// width: measured on a booted machine on 2026-10-23, a panel 604 wide was
    /// drawn 276 wide, with the field and the status line running out of it
    /// onto bare canvas. `left = (338 - reach)` and
    /// `right = (338 + 604 + reach).min(604)` — the arithmetic and the pixels
    /// agreed to the pixel.
    ///
    /// `crate::dock_raster` has always passed the display here and says so on
    /// the field: *the display, so the list can be kept on it.* The Bar was
    /// the one caller that did not.
    pub display: Size<i32, Physical>,
}

/// The Bar as one washed panel and the ink on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BarPicture {
    /// The panel, with its rows drawn into it.
    pub(crate) panel: Washed,
    /// Every string on it, already placed.
    pub(crate) inked: Vec<Inked>,
}

/// Draw the Bar.
///
/// [`None`] when the panel could not be rasterised at all, which is the same
/// answer `crate::dock_surface` gives for a surface with no room in it — and
/// the caller draws nothing rather than half a Bar.
pub(crate) fn picture(
    bar: &TheBarOnTheDisplay<'_>,
    palette: crate::desktop_look::DesktopPalette,
    fonts: &mut FontSystem,
) -> Option<BarPicture> {
    let places = bar.places;
    let mut wells = vec![crate::dock_surface::AWell {
        at: places.field,
        corner: alo_dock::measures::THE_COMPOSERS_CORNER,
    }];
    let mut rings: Vec<Rectangle<i32, Physical>> = Vec::new();

    for (which, _) in bar.results.iter().enumerate().take(places.rows_that_fit) {
        let at = row_at(&places, which);
        wells.push(crate::dock_surface::AWell {
            at,
            corner: alo_dock::measures::THE_COMPOSERS_CORNER,
        });
        if bar.chosen == Some(which) {
            rings.push(crate::dock_raster::a_focus_ring(at));
        }
    }

    let panel = crate::dock_surface::surface(crate::dock_surface::ADocksSurface {
        raised: crate::how_a_panel_is_raised::HowItIsRaised::THE_DOCKS.unscaled(),
        panel: places.panel,
        fill: palette.dock,
        edge: palette.edge,
        accent: palette.accent,
        against: alo_dock::Edge::Bottom,
        accent_is: 0,
        badges: &[],
        marks: &[],
        mark: palette.ink,
        badge: palette.badge,
        wells: &wells,
        well_is: palette.badge,
        rings: &rings,
        ring_is: palette.ink,
        in_primary: crate::dock_surface::InPrimary::NOTHING,
        room: (bar.display.w, bar.display.h),
    })?;

    // The sentences, each placed where its box is. `sentence` wraps and
    // `placed` crops, so a long application name is cut rather than running
    // out of its row — which is what the design's own rows do.
    let mut inked = Vec::new();
    let mut say = |text: &str, at: Rectangle<i32, Physical>, size: f32, ink: [u8; 3]| {
        if text.is_empty() {
            return;
        }
        let metrics = Metrics::new(size, size * 1.55);
        let shaped =
            crate::painted_text::sentence(fonts, text, at.size.w, metrics, palette.badge, ink);
        let down = (at.size.h - shaped.height).max(0) / 2;
        if let Some(one) = shaped.placed(at.loc.x, at.loc.y + down, at.size.w) {
            inked.push(one);
        }
    };

    say(bar.typed, inset(places.field), 16.0, palette.ink);
    say(bar.origin, places.origin, 13.0, palette.ink_quiet);
    for (which, result) in bar.results.iter().enumerate().take(places.rows_that_fit) {
        say(
            &result.shown,
            inset(row_at(&places, which)),
            13.0,
            palette.ink,
        );
    }

    Some(BarPicture { panel, inked })
}

/// Where row `which` sits.
fn row_at(places: &TheBarsPlaces, which: usize) -> Rectangle<i32, Physical> {
    let down = places.row_to_row * i32::try_from(which).unwrap_or(0);
    Rectangle::new(
        (places.first_row.loc.x, places.first_row.loc.y + down).into(),
        places.first_row.size,
    )
}

/// A box with the air the design leaves inside a field or a row.
///
/// 16 at the reference, measured off the frame's own `Query` text, and the
/// same on a row — one number, because the design draws a row as the field.
fn inset(at: Rectangle<i32, Physical>) -> Rectangle<i32, Physical> {
    const AIR: i32 = 16;
    Rectangle::new(
        (at.loc.x + AIR, at.loc.y).into(),
        ((at.size.w - AIR * 2).max(1), at.size.h).into(),
    )
}

#[cfg(test)]
#[path = "bar_raster_tests.rs"]
mod tests;
