//! Drawing the launcher — *Find, open, or ask* — as the owner approved it on
//! 2026-11-01 (`561:275`, `561:370`): a heading and Close, the field, the line
//! saying where the results came from, the applications found, the row that
//! asks alo, and the keys along the foot, on a panel over a dimmed screen.
//!
//! **The geometry is `crate::where_the_bar_opens`' and the words are the
//! caller's.** This file turns the two into pixels and decides nothing of its
//! own.
//!
//! # One raster, words included
//!
//! The panel is drawn with `crate::settings_canvas`, which draws Settings: a
//! rounded panel with its shadow, rounded wells in any colour, outlines, icons
//! from the design's own files and words shaped **on the ground they sit on**.
//! The last is what put right the grey band: the earlier Bar inked every line on
//! the field's grey and pasted it onto the white panel, so the line under the
//! field sat in a grey rectangle the design does not draw.
//!
//! # The screen behind is dimmed, not blurred
//!
//! `561:308` lays the canvas colour over everything at 45 % and blurs it by 4.
//! The dimming is drawn — `BarPicture::veil`, a translucent fill over the
//! whole display, under the panel and over the windows and the Dock. **The blur
//! is not**: no surface in this shell blurs what is behind it, and reading the
//! frame back to blur it is a renderer change of its own. Recorded in
//! `docs/design/the-alo-bar-when-it-is-open.md`.
//!
//! # At its display's scale
//!
//! Laid out here with the scale the rest of the display is drawn at, so on a
//! 200 per cent screen the launcher is twice the pixels and the same size to the
//! eye as the Dock beside it.

use cosmic_text::FontSystem;
use smithay::utils::{Physical, Rectangle, Size};

use crate::desktop_look::Measure;
use crate::display_scale::DisplayScale;
use crate::painted::Washed;
use crate::settings_canvas::{AShadow, Area, Canvas};
use crate::settings_type::TheType;
use crate::the_bar_finds::AResult;
use crate::the_launchers_icons::ALaunchersIcon;
use crate::the_launchers_rows::{
    A_KEY_CAPS_CORNER, A_WELLS_CORNER, TheRowsColours, WhereTheRowsAre, a_key_cap, centred, said,
    shrunk,
};
use crate::where_the_bar_opens::{TheBarsPlaces, WhatItHolds};

/// Where the Bar is shown.
///
/// **Two places since the owner's decision of 2026-11-11.** Clicking the Dock's
/// field, `Super`+`Space` and `Super`+`A` keep the keyboard in the Dock's own
/// field, with the matches rising from it (`crate::bar_matches_raster`). The
/// centred panel stays for *Help with this window*, which another change is
/// moving into the window's own panel — and until it does, removing the panel
/// here would take that road away.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheBarOpens {
    /// The centred panel over a dimmed screen (`561:275`, `561:370`).
    InTheMiddle,
    /// A compact list rising from the Dock's field.
    AboveTheDocksField,
}

/// The launcher's own words, already said in the person's language.
#[derive(Debug, Clone, Copy)]
pub struct TheLaunchersWords<'a> {
    /// *Find, open, or ask*, the heading.
    pub heading: &'a str,
    /// *Open*, the hint on the chosen application row.
    pub open_it: &'a str,
    /// *choose*, after the Up and Down keys.
    pub choose: &'a str,
    /// *open*, after the Enter key.
    pub open: &'a str,
    /// *close*, after the Escape key.
    pub close: &'a str,
    /// *Esc*, on its key cap.
    pub esc: &'a str,
}

/// The row that asks alo, already said.
#[derive(Debug, Clone, Copy)]
pub struct TheAskRow<'a> {
    /// *Ask alo “files”*.
    pub asking: &'a str,
    /// *a new task*.
    pub what_it_is: &'a str,
    /// Where an answer would come from, in `alo-models`' own sentence.
    pub from: &'a str,
}

/// What the launcher shows, as a frame hands it over.
#[derive(Debug, Clone, Copy)]
pub struct TheBarOnTheDisplay<'a> {
    /// What a person has typed, which the field shows.
    pub typed: &'a str,
    /// The line under the field.
    pub origin: &'a str,
    /// Every result, in the order to read them.
    pub results: &'a [AResult],
    /// Which row the keyboard is on: an application's, or one past the last
    /// of them, the row that asks.
    pub chosen: Option<usize>,
    /// **The display**, so the panel's shadow and the dimming are kept on it.
    pub display: Size<i32, Physical>,
    /// The launcher's own words.
    pub words: TheLaunchersWords<'a>,
    /// The row that asks alo, where alo may be asked — never while the person
    /// has set AI to *Not at all*, nor while nothing is chosen to answer.
    pub asks: Option<TheAskRow<'a>>,
    /// What each application draws, for its row's icon.
    pub artwork: Option<&'a alo_dock::artwork::TheArtwork>,
    /// Where it is shown.
    pub opens: TheBarOpens,
}

/// The launcher as one washed panel, over a dimmed screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BarPicture {
    /// The dimming over the whole display: where, what colour and how much.
    /// The opacity is in per cent, so the picture compares exactly.
    pub(crate) veil: (Rectangle<i32, Physical>, [u8; 3], u8),
    /// The panel, with everything on it drawn in.
    pub(crate) panel: Washed,
    /// Where everything is, for the press that comes after the draw.
    pub(crate) places: TheBarsPlaces,
}

/// How much of the canvas colour lies over the screen: `561:308`'s 45 %.
const THE_VEIL_IS: u8 = 45;

/// The panel's corner, `radius/lg`.
const THE_PANELS_CORNER: u32 = 18;

/// Draw the launcher on a display drawn at `scale`, its canvas `canvas`.
///
/// [`None`] when the display is too small to hold it
/// (`crate::where_the_bar_opens`): the caller draws nothing rather than half a
/// launcher.
pub(crate) fn picture(
    bar: &TheBarOnTheDisplay<'_>,
    palette: crate::desktop_look::DesktopPalette,
    canvas_colour: [u8; 3],
    fonts: &mut FontSystem,
    scale: DisplayScale,
) -> Option<BarPicture> {
    let places = crate::where_the_bar_opens::where_the_bar_opens(
        bar.display,
        scale,
        WhatItHolds {
            found: bar.results.len(),
            asks: bar.asks.is_some(),
            chosen: bar.chosen,
        },
    )?;
    let px = |logical: u32| scale.px(logical);
    let measure = Measure::of(alo_appearance::TextScale::ordinary()).on(scale);
    let TheRowsColours {
        secondary,
        key_line,
    } = TheRowsColours::of(palette);

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
    // **`bg/surface`, which is the Dock's ground and not a window's.** The
    // window ground is the canvas's cream (`Role::Ground`); `561:276` fills the
    // panel white, as the Dock is, and every word on it is shaped on that.
    canvas.fill(
        shrunk(places.panel, line),
        corner - line as f32,
        palette.dock,
        1.0,
    );

    let say = |canvas: &mut Canvas,
               fonts: &mut FontSystem,
               words: &str,
               style: TheType,
               at: (i32, i32, i32),
               colours: ([u8; 3], [u8; 3])|
     -> i32 { said(canvas, fonts, measure, words, style, at, colours) };

    // **The heading and Close** (`561:310`): the title 5 down its 40, Close a
    // 48 × 40 well at its end with the cross centred in a 16 square.
    let heading = places.heading;
    let title_room = places.close.loc.x - heading.loc.x - px(A_WELLS_CORNER);
    say(
        &mut canvas,
        fonts,
        bar.words.heading,
        TheType::Title,
        (heading.loc.x, heading.loc.y + px(5), title_room),
        (palette.dock, palette.ink),
    );
    canvas.fill(places.close, px(A_WELLS_CORNER) as f32, palette.badge, 1.0);
    ALaunchersIcon::Close.drawn(&mut canvas, centred(places.close, px(16)), palette.ink);

    // **The field** (`561:315`): the cool well with the keyboard's outline,
    // which is drawn because the field has the keyboard whenever this is open;
    // the search mark 18 in; what was typed at 50, and the caret after it.
    let field = places.field;
    canvas.fill(field, px(A_WELLS_CORNER) as f32, palette.badge, 1.0);
    canvas.border(field, px(A_WELLS_CORNER) as f32, px(2) as f32, palette.ink);
    ALaunchersIcon::Search.drawn(
        &mut canvas,
        Rectangle::new(
            (field.loc.x + px(18), field.loc.y + px(18)).into(),
            (px(20), px(20)).into(),
        ),
        secondary,
    );
    let typed_at = (field.loc.x + px(50), field.loc.y + px(16));
    canvas.clipped_to(shrunk(field, px(2)));
    let typed_wide = say(
        &mut canvas,
        fonts,
        bar.typed,
        TheType::Introduction,
        (typed_at.0, typed_at.1, field.size.w - px(50) - px(18)),
        (palette.badge, palette.ink),
    );
    canvas.fill(
        Rectangle::new(
            (typed_at.0 + typed_wide + px(1), field.loc.y + px(19)).into(),
            (px(2), px(20)).into(),
        ),
        0.0,
        palette.ink,
        1.0,
    );
    canvas.unclipped();

    // **The line under the field**, on the panel's own white.
    say(
        &mut canvas,
        fonts,
        bar.origin,
        TheType::Label,
        (
            places.origin.loc.x,
            places.origin.loc.y,
            places.origin.size.w,
        ),
        (palette.dock, palette.ink_quiet),
    );

    // **Each application found, the hairline and the row that asks alo** —
    // one drawing with the list that rises from the Dock's field
    // (`crate::the_launchers_rows`).
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

    // **The keys along the foot** (`561:351`): Up and Down, *choose*; Enter,
    // *open*; Esc, *close* — each cap white on a `border/strong` line.
    let keys = places.keys;
    let y = keys.loc.y + px(8);
    let mut x = keys.loc.x;
    for icon in [ALaunchersIcon::Up, ALaunchersIcon::Down] {
        let cap = Rectangle::new((x, y).into(), (px(26), px(18)).into());
        a_key_cap(
            &mut canvas,
            cap,
            px(A_KEY_CAPS_CORNER),
            palette.dock,
            key_line,
        );
        icon.drawn(&mut canvas, centred(cap, px(12)), secondary);
        x += px(32);
    }
    x += say(
        &mut canvas,
        fonts,
        bar.words.choose,
        TheType::SmallLabel,
        (x, y + px(1), keys.size.w / 3),
        (palette.dock, palette.ink_quiet),
    ) + px(16);
    let cap = Rectangle::new((x, y).into(), (px(26), px(18)).into());
    a_key_cap(
        &mut canvas,
        cap,
        px(A_KEY_CAPS_CORNER),
        palette.dock,
        key_line,
    );
    ALaunchersIcon::Enter.drawn(&mut canvas, centred(cap, px(12)), secondary);
    x += px(32);
    x += say(
        &mut canvas,
        fonts,
        bar.words.open,
        TheType::SmallLabel,
        (x, y + px(1), keys.size.w / 3),
        (palette.dock, palette.ink_quiet),
    ) + px(16);
    let esc_wide =
        crate::painted_text::how_wide(fonts, bar.words.esc, TheType::SmallLabel.metrics(measure));
    let cap = Rectangle::new(
        (x, keys.loc.y + px(6)).into(),
        (esc_wide + px(14), px(22)).into(),
    );
    a_key_cap(
        &mut canvas,
        cap,
        px(A_KEY_CAPS_CORNER),
        palette.dock,
        key_line,
    );
    say(
        &mut canvas,
        fonts,
        bar.words.esc,
        TheType::SmallLabel,
        (cap.loc.x + px(7), cap.loc.y + px(3), esc_wide + px(2)),
        (palette.dock, secondary),
    );
    say(
        &mut canvas,
        fonts,
        bar.words.close,
        TheType::SmallLabel,
        (
            cap.loc.x + cap.size.w + px(6),
            keys.loc.y + px(9),
            keys.size.w / 3,
        ),
        (palette.dock, palette.ink_quiet),
    );

    Some(BarPicture {
        veil: (screen, canvas_colour, THE_VEIL_IS),
        // **Premultiplied for the renderer** (`crate::painted::Washed`).
        panel: canvas.washed().premultiplied(),
        places,
    })
}

#[cfg(test)]
#[path = "bar_raster_tests.rs"]
mod tests;

/// The launcher at a display's scale against the same launcher at one.
#[cfg(test)]
#[path = "bar_raster_scale_tests.rs"]
mod scale_tests;
