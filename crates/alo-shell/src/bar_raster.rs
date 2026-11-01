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
//! The dimming is drawn — [`BarPicture::veil`], a translucent fill over the
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

use alo_appearance::Role;
use cosmic_text::FontSystem;
use smithay::utils::{Physical, Rectangle, Size};

use crate::desktop_look::Measure;
use crate::display_scale::DisplayScale;
use crate::painted::Washed;
use crate::settings_canvas::{AShadow, Area, Canvas};
use crate::settings_type::{TheType, written};
use crate::the_bar_finds::AResult;
use crate::the_launchers_icons::ALaunchersIcon;
use crate::where_the_bar_opens::{TheBarsPlaces, WhatItHolds};

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
}

/// The launcher as one washed panel, over a dimmed screen.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BarPicture {
    /// The dimming over the whole display: where, what colour and how much.
    pub(crate) veil: (Rectangle<i32, Physical>, [u8; 3], f32),
    /// The panel, with everything on it drawn in.
    pub(crate) panel: Washed,
    /// Where everything is, for the press that comes after the draw.
    pub(crate) places: TheBarsPlaces,
}

/// How much of the canvas colour lies over the screen: `561:308`'s 45 %.
const THE_VEIL_IS: f32 = 0.45;

/// The panel's corner, `radius/lg`.
const THE_PANELS_CORNER: u32 = 18;

/// The field's, a row's and the close control's corner: 12.
const A_WELLS_CORNER: u32 = 12;

/// An application icon's tile: 32 across, corner 8, 12 in from the row.
const AN_ICON_IS: (u32, u32, u32) = (32, 8, 12);

/// A key cap's corner: 6 (`561:334`).
const A_KEY_CAPS_CORNER: u32 = 6;

/// How much of the accent tints the chosen row: `561:324`'s 12 %.
const THE_CHOSEN_TINT: f32 = 0.12;

/// `border/strong`, the key caps' line (`561:334`): outside the twelve roles,
/// as `crate::settings_palette` records it, and named once here.
const BORDER_STRONG: [u8; 3] = [0xCB, 0xD5, 0xE1];

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
    let rgb = |role: Role| {
        let it = role.colour();
        [it.red(), it.green(), it.blue()]
    };
    // **The design's light values where the palette is as designed**, and the
    // palette's own ink at high contrast — the substitution
    // `DesktopPalette::edge` makes, for its reason.
    let as_designed = palette.ink_quiet != palette.ink;
    let secondary = if as_designed {
        rgb(Role::TextSecondary)
    } else {
        palette.ink
    };
    let key_line = if as_designed {
        BORDER_STRONG
    } else {
        palette.ink
    };

    let raised = crate::how_a_panel_is_raised::HowItIsRaised::FLOATING.at(scale.factor());
    let reach = raised.reach();
    let screen: Area = Rectangle::from_size(bar.display);
    let area = Rectangle::new(
        (
            places.panel.loc.x - reach,
            places.panel.loc.y - reach,
        )
            .into(),
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
    canvas.fill(shrunk(places.panel, line), corner - line as f32, palette.ground, 1.0);

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
        (palette.ground, palette.ink),
    );
    canvas.fill(places.close, px(A_WELLS_CORNER) as f32, palette.badge, 1.0);
    ALaunchersIcon::Close.drawn(
        &mut canvas,
        centred(places.close, px(16)),
        palette.ink,
    );

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
        (places.origin.loc.x, places.origin.loc.y, places.origin.size.w),
        (palette.ground, palette.ink_quiet),
    );

    // **Each application found** (`561:324`): tinted in the accent where the
    // keyboard is, its icon, its name, what kind of thing it is, and on the
    // chosen row the hint that Enter opens it.
    let tinted = |ground: [u8; 3], accent: [u8; 3]| -> [u8; 3] {
        let mix = |a: u8, b: u8| {
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "a blend of two bytes stays a byte"
            )]
            let it = (f32::from(b) * THE_CHOSEN_TINT + f32::from(a) * (1.0 - THE_CHOSEN_TINT))
                .round() as u8;
            it
        };
        let ([gr, gg, gb], [ar, ag, ab]) = (ground, accent);
        [mix(gr, ar), mix(gg, ag), mix(gb, ab)]
    };
    let chosen_ground = tinted(palette.ground, palette.accent);
    let chosen = bar.chosen.unwrap_or(0);
    for (which, at) in &places.rows {
        let Some(result) = bar.results.get(*which) else {
            continue;
        };
        let is_chosen = *which == chosen;
        let ground = if is_chosen {
            canvas.fill(*at, px(A_WELLS_CORNER) as f32, chosen_ground, 1.0);
            chosen_ground
        } else {
            palette.ground
        };
        let (icon, icon_corner, icon_in) = AN_ICON_IS;
        let tile = Rectangle::new(
            (at.loc.x + px(icon_in), at.loc.y + px(icon_in)).into(),
            (px(icon), px(icon)).into(),
        );
        an_applications_icon(&mut canvas, bar, result, tile, px(icon_corner), palette);
        let name_at = (at.loc.x + px(56), at.loc.y + px(17));
        canvas.clipped_to(*at);
        let hint_room = if is_chosen { px(77) } else { px(14) };
        let room = at.size.w - px(56) - hint_room;
        let name_wide = say(
            &mut canvas,
            fonts,
            &result.name,
            TheType::Name,
            (name_at.0, name_at.1, room),
            (ground, palette.ink),
        );
        let after = format!("\u{b7}  {}", result.kind);
        say(
            &mut canvas,
            fonts,
            &after,
            TheType::Body,
            (
                name_at.0 + name_wide + px(6),
                name_at.1,
                room - name_wide - px(6),
            ),
            (ground, palette.ink_quiet),
        );
        canvas.unclipped();
        if is_chosen {
            // *Open* and the Enter key, 14 from the row's end (`561:332`).
            let cap = Rectangle::new(
                (
                    at.loc.x + at.size.w - px(14) - px(26),
                    at.loc.y + px(19),
                )
                    .into(),
                (px(26), px(18)).into(),
            );
            a_key_cap(&mut canvas, cap, px(A_KEY_CAPS_CORNER), palette.ground, key_line);
            ALaunchersIcon::Enter.drawn(&mut canvas, centred(cap, px(12)), secondary);
            let open_wide = crate::painted_text::how_wide(
                fonts,
                bar.words.open_it,
                TheType::SmallLabel.metrics(measure),
            );
            say(
                &mut canvas,
                fonts,
                bar.words.open_it,
                TheType::SmallLabel,
                (cap.loc.x - px(8) - open_wide, at.loc.y + px(20), open_wide + px(2)),
                (ground, palette.ink_quiet),
            );
        }
    }

    // **The hairline over the row that asks**, and the row itself (`561:339`):
    // alo's mark on its own tile, the request quoted, *a new task*, and on its
    // second line where the answer would come from.
    if let Some(hairline) = places.hairline {
        canvas.fill(hairline, 0.0, palette.edge, 1.0);
    }
    if let (Some(at), Some(asks)) = (places.ask, bar.asks.as_ref()) {
        let is_chosen = chosen == bar.results.len();
        let ground = if is_chosen {
            canvas.fill(at, px(A_WELLS_CORNER) as f32, chosen_ground, 1.0);
            chosen_ground
        } else {
            palette.ground
        };
        let (icon, icon_corner, icon_in) = AN_ICON_IS;
        let tile = Rectangle::new(
            (at.loc.x + px(icon_in), at.loc.y + px(16)).into(),
            (px(icon), px(icon)).into(),
        );
        canvas.fill(
            tile,
            px(icon_corner) as f32,
            rgb(Role::AccentSoft),
            1.0,
        );
        // **Teal, because this row is alo** — ADR 0067's one use for it.
        ALaunchersIcon::Alo.drawn(&mut canvas, tile, rgb(Role::AccentDefault));
        canvas.clipped_to(at);
        let room = at.size.w - px(56) - px(14);
        let asked = say(
            &mut canvas,
            fonts,
            asks.asking,
            TheType::Name,
            (at.loc.x + px(56), at.loc.y + px(12), room),
            (ground, palette.ink),
        );
        let after = format!("\u{b7}  {}", asks.what_it_is);
        say(
            &mut canvas,
            fonts,
            &after,
            TheType::Body,
            (
                at.loc.x + px(56) + asked + px(6),
                at.loc.y + px(12),
                room - asked - px(6),
            ),
            (ground, palette.ink_quiet),
        );
        ALaunchersIcon::Cloud.drawn(
            &mut canvas,
            Rectangle::new(
                (at.loc.x + px(56), at.loc.y + px(37)).into(),
                (px(14), px(14)).into(),
            ),
            palette.ink_quiet,
        );
        say(
            &mut canvas,
            fonts,
            asks.from,
            TheType::SmallLabel,
            (at.loc.x + px(76), at.loc.y + px(36), room - px(20)),
            (ground, palette.ink_quiet),
        );
        canvas.unclipped();
    }

    // **The keys along the foot** (`561:351`): Up and Down, *choose*; Enter,
    // *open*; Esc, *close* — each cap white on a `border/strong` line.
    let keys = places.keys;
    let y = keys.loc.y + px(8);
    let mut x = keys.loc.x;
    for icon in [ALaunchersIcon::Up, ALaunchersIcon::Down] {
        let cap = Rectangle::new((x, y).into(), (px(26), px(18)).into());
        a_key_cap(&mut canvas, cap, px(A_KEY_CAPS_CORNER), palette.ground, key_line);
        icon.drawn(&mut canvas, centred(cap, px(12)), secondary);
        x += px(32);
    }
    x += say(
        &mut canvas,
        fonts,
        bar.words.choose,
        TheType::SmallLabel,
        (x, y + px(1), keys.size.w / 3),
        (palette.ground, palette.ink_quiet),
    ) + px(16);
    let cap = Rectangle::new((x, y).into(), (px(26), px(18)).into());
    a_key_cap(&mut canvas, cap, px(A_KEY_CAPS_CORNER), palette.ground, key_line);
    ALaunchersIcon::Enter.drawn(&mut canvas, centred(cap, px(12)), secondary);
    x += px(32);
    x += say(
        &mut canvas,
        fonts,
        bar.words.open,
        TheType::SmallLabel,
        (x, y + px(1), keys.size.w / 3),
        (palette.ground, palette.ink_quiet),
    ) + px(16);
    let esc_wide = crate::painted_text::how_wide(
        fonts,
        bar.words.esc,
        TheType::SmallLabel.metrics(measure),
    );
    let cap = Rectangle::new(
        (x, keys.loc.y + px(6)).into(),
        (esc_wide + px(14), px(22)).into(),
    );
    a_key_cap(&mut canvas, cap, px(A_KEY_CAPS_CORNER), palette.ground, key_line);
    say(
        &mut canvas,
        fonts,
        bar.words.esc,
        TheType::SmallLabel,
        (cap.loc.x + px(7), cap.loc.y + px(3), esc_wide + px(2)),
        (palette.ground, secondary),
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
        (palette.ground, palette.ink_quiet),
    );

    Some(BarPicture {
        veil: (screen, canvas_colour, THE_VEIL_IS),
        panel: canvas.washed(),
        places,
    })
}

/// `words` in `style` at (`x`, `y`), no wider than `wide`, shaped on `ground`
/// in `ink` — the ground it is drawn on, so no band of another colour is
/// pasted with it. Answers how wide the ink is, for what follows it.
fn said(
    canvas: &mut Canvas,
    fonts: &mut FontSystem,
    measure: Measure,
    words: &str,
    style: TheType,
    (x, y, wide): (i32, i32, i32),
    (ground, ink): ([u8; 3], [u8; 3]),
) -> i32 {
    if words.is_empty() || wide <= 0 {
        return 0;
    }
    let shaped = written(fonts, words, style, wide, measure, (ground, ink)).cropped_to_its_ink();
    canvas.words(x, y, &shaped);
    shaped.width
}

/// An application's icon on its row: its own artwork where this machine knows
/// it, and otherwise the Dock's fallback — its first letter on the cool tile.
fn an_applications_icon(
    canvas: &mut Canvas,
    bar: &TheBarOnTheDisplay<'_>,
    result: &AResult,
    tile: Area,
    corner: i32,
    palette: crate::desktop_look::DesktopPalette,
) {
    let crate::the_bar_finds::WhatARowDoes::OpensAnApplication(identifier) = &result.does;
    let artwork = bar.artwork.and_then(|artwork| {
        alo_dock::AppId::named(identifier)
            .ok()
            .and_then(|app| artwork.of(&app).cloned())
    });
    canvas.fill(tile, corner as f32, palette.badge, 1.0);
    if let Some(picture) = artwork {
        let (Ok(across), Ok(down)) = (u32::try_from(tile.size.w), u32::try_from(tile.size.h))
        else {
            return;
        };
        let scaled = picture.scaled_to(across, down);
        canvas.picture(tile.loc.x, tile.loc.y, tile.size.w, scaled.pixels());
    }
}

/// A key cap: a white well inside a one-unit line.
fn a_key_cap(canvas: &mut Canvas, cap: Area, corner: i32, ground: [u8; 3], line: [u8; 3]) {
    canvas.fill(cap, corner as f32, line, 1.0);
    canvas.fill(shrunk(cap, 1), (corner - 1).max(0) as f32, ground, 1.0);
}

/// A square of `side` centred in `area`.
fn centred(area: Area, side: i32) -> Area {
    Rectangle::new(
        (
            area.loc.x + (area.size.w - side) / 2,
            area.loc.y + (area.size.h - side) / 2,
        )
            .into(),
        (side, side).into(),
    )
}

/// `area` less `by` on every side.
fn shrunk(area: Area, by: i32) -> Area {
    Rectangle::new(
        (area.loc.x + by, area.loc.y + by).into(),
        ((area.size.w - by * 2).max(0), (area.size.h - by * 2).max(0)).into(),
    )
}

#[cfg(test)]
#[path = "bar_raster_tests.rs"]
mod tests;

/// The launcher at a display's scale against the same launcher at one.
#[cfg(test)]
#[path = "bar_raster_scale_tests.rs"]
mod scale_tests;
