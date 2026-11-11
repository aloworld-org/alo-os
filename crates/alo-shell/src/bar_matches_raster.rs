//! Drawing the alo Bar's matches as a compact list rising from the Dock's field
//! — the owner's decision of 2026-11-11.
//!
//! **The geometry is `crate::where_the_bars_matches_rise`' and the rows are
//! `crate::the_launchers_rows`'**, the same rows the centred panel draws. What
//! is this file's own is the list's panel — white, a hairline, corner 18 and
//! the floating shadow, the centred panel's own surface — and, on a side Dock,
//! the line showing what is typed.
//!
//! # No dimmed screen
//!
//! The centred panel lies over a screen dimmed to 45 %. **This list does not**:
//! it is non-modal, as *More apps* is, so the canvas behind it is not dimmed,
//! not blurred and not disabled, and a press outside it reaches what it lands
//! on (`crate::a_press_on_the_launcher`).

use cosmic_text::FontSystem;
use smithay::utils::{Physical, Rectangle};

use crate::bar_raster::TheBarOnTheDisplay;
use crate::desktop_look::Measure;
use crate::display_scale::DisplayScale;
use crate::painted::Washed;
use crate::settings_canvas::{AShadow, Area, Canvas};
use crate::settings_type::TheType;
use crate::the_launchers_icons::ALaunchersIcon;
use crate::the_launchers_rows::{A_WELLS_CORNER, TheRowsColours, WhereTheRowsAre, said, shrunk};
use crate::where_the_bar_opens::WhatItHolds;
use crate::where_the_bars_matches_rise::{TheFieldOnTheDock, TheMatchesPlaces};

/// The panel's corner, `radius/lg`, the centred panel's own.
const THE_PANELS_CORNER: u32 = 18;

/// The matches as one washed panel, with no veil.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct MatchesPicture {
    /// The list, with everything on it drawn in.
    pub(crate) panel: Washed,
    /// Where everything is, for the press that comes after the draw.
    pub(crate) places: TheMatchesPlaces,
}

/// What the list rises from and keeps clear of.
#[derive(Debug, Clone, Copy)]
pub(crate) struct WhereItRisesFrom {
    /// The Dock's field.
    pub(crate) dock: TheFieldOnTheDock,
    /// The put-aside panel's column.
    pub(crate) clear_of: Option<Rectangle<i32, Physical>>,
}

/// Draw the matches rising from `from`, on a display drawn at `scale`.
///
/// [`None`] when there is nothing to show, or no room to show it
/// (`crate::where_the_bars_matches_rise`).
pub(crate) fn picture(
    bar: &TheBarOnTheDisplay<'_>,
    palette: crate::desktop_look::DesktopPalette,
    from: WhereItRisesFrom,
    fonts: &mut FontSystem,
    scale: DisplayScale,
) -> Option<MatchesPicture> {
    let places = crate::where_the_bars_matches_rise::where_the_matches_rise(
        bar.display,
        scale,
        WhatItHolds {
            found: bar.results.len(),
            asks: bar.asks.is_some(),
            chosen: bar.chosen,
        },
        from.dock,
        from.clear_of,
    )?;
    let px = |logical: u32| scale.px(logical);
    let raised = crate::how_a_panel_is_raised::HowItIsRaised::FLOATING.at(scale.factor());
    let reach = raised.reach();
    let screen: Area = Rectangle::from_size(bar.display);
    let area = Rectangle::new(
        (places.panel.loc.x - reach, places.panel.loc.y - reach).into(),
        (
            places.panel.size.w + reach * 2,
            places.panel.size.h + reach * 2,
        )
            .into(),
    )
    .intersection(screen)?;
    let mut canvas = Canvas::over(area);
    let corner = px(THE_PANELS_CORNER) as f32;
    canvas.shadow(
        places.panel,
        corner,
        AShadow {
            falls: raised.falls(),
            blurs: raised.blurs(),
            spreads: raised.spreads(),
            darkness: raised.darkness(),
        },
    );
    canvas.fill(places.panel, corner, palette.edge, 1.0);
    let line = scale.px(1).max(1);
    canvas.fill(
        shrunk(places.panel, line),
        corner - line as f32,
        palette.dock,
        1.0,
    );

    if let Some(typed) = places.typed {
        what_is_typed(&mut canvas, fonts, bar, typed, palette, scale);
    }
    crate::the_launchers_rows::the_rows(
        &mut canvas,
        fonts,
        bar,
        WhereTheRowsAre {
            rows: &places.rows,
            hairline: places.hairline,
            ask: places.ask,
        },
        palette,
        scale,
    );

    Some(MatchesPicture {
        // **Premultiplied for the renderer** (`crate::painted::Washed`).
        panel: canvas.washed().premultiplied(),
        places,
    })
}

/// The line a side Dock's list shows what is typed on: the field's cool well
/// with the keyboard's outline, the search mark, the words and the caret after
/// them — the centred panel's field (`561:315`) at a row's height, because a
/// side Dock's own field is a 48-wide slot with no room for a line of text.
fn what_is_typed(
    canvas: &mut Canvas,
    fonts: &mut FontSystem,
    bar: &TheBarOnTheDisplay<'_>,
    at: Area,
    palette: crate::desktop_look::DesktopPalette,
    scale: DisplayScale,
) {
    let px = |logical: u32| scale.px(logical);
    let measure = Measure::of(alo_appearance::TextScale::ordinary()).on(scale);
    let colours = TheRowsColours::of(palette);
    canvas.fill(at, px(A_WELLS_CORNER) as f32, palette.badge, 1.0);
    canvas.border(at, px(A_WELLS_CORNER) as f32, px(2) as f32, palette.ink);
    ALaunchersIcon::Search.drawn(
        canvas,
        Rectangle::new(
            (at.loc.x + px(18), at.loc.y + px(18)).into(),
            (px(20), px(20)).into(),
        ),
        colours.secondary,
    );
    let typed_at = (at.loc.x + px(50), at.loc.y + px(16));
    let room = at.size.w - px(50) - px(18);
    canvas.clipped_to(shrunk(at, px(2)));
    // **The end of what is typed, where it is longer than the line**, so the
    // caret and the letter just typed are always the ones in view.
    let metrics = TheType::Introduction.metrics(measure);
    let shown = crate::the_end_that_fits::the_end_that_fits(bar.typed, room, |text| {
        crate::painted_text::how_wide(fonts, text, metrics)
    });
    let typed_wide = said(
        canvas,
        fonts,
        measure,
        shown,
        TheType::Introduction,
        (typed_at.0, typed_at.1, room),
        (palette.badge, palette.ink),
    );
    canvas.fill(
        Rectangle::new(
            (typed_at.0 + typed_wide + px(1), at.loc.y + px(19)).into(),
            (px(2), px(20)).into(),
        ),
        0.0,
        palette.ink,
        1.0,
    );
    canvas.unclipped();
}

#[cfg(test)]
#[path = "bar_matches_raster_tests.rs"]
mod tests;
