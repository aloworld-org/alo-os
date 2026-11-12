//! The alo Bar's rows as they are drawn — each application found, the hairline
//! and the row that asks alo — **wherever the Bar is shown**.
//!
//! Split out of the centred launcher's raster on 2026-11-11, when the owner's
//! decision put the matches in a list rising from the Dock's field
//! (`crate::bar_matches_raster`). The centred launcher was removed the same day,
//! once nothing opened it, so the list is the one surface drawing these rows
//! now, at the measures read off `561:324` and `561:339`.

use alo_appearance::Role;
use cosmic_text::FontSystem;
use smithay::utils::{Physical, Rectangle};

use crate::desktop_look::Measure;
use crate::display_scale::DisplayScale;
use crate::settings_canvas::{Area, Canvas};
use crate::settings_type::{TheType, written};
use crate::the_bar_finds::AResult;
use crate::the_launchers_icons::ALaunchersIcon;
use crate::what_the_bar_shows::TheBarOnTheDisplay;

/// The typed line's and a row's corner: 12.
pub(crate) const A_WELLS_CORNER: u32 = 12;

/// An application icon's tile: 32 across, corner 8, 12 in from the row.
const AN_ICON_IS: (u32, u32, u32) = (32, 8, 12);

/// A key cap's corner: 6 (`561:334`).
pub(crate) const A_KEY_CAPS_CORNER: u32 = 6;

/// How much of the accent tints the chosen row: `561:324`'s 12 %.
const THE_CHOSEN_TINT: f32 = 0.12;

/// `border/strong`, the key caps' line (`561:334`): outside the twelve roles,
/// as `crate::settings_palette` records it, and named once here.
const BORDER_STRONG: [u8; 3] = [0xCB, 0xD5, 0xE1];

/// The colours the rows need beyond the palette's own.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TheRowsColours {
    /// `text/secondary`, or the palette's ink at high contrast.
    pub(crate) secondary: [u8; 3],
    /// A key cap's line.
    pub(crate) key_line: [u8; 3],
}

impl TheRowsColours {
    /// **The design's light values where the palette is as designed**, and the
    /// palette's own ink at high contrast — the substitution
    /// `DesktopPalette::edge` makes, for its reason.
    pub(crate) fn of(palette: crate::desktop_look::DesktopPalette) -> Self {
        let as_designed = palette.ink_quiet != palette.ink;
        Self {
            secondary: if as_designed {
                rgb(Role::TextSecondary)
            } else {
                palette.ink
            },
            key_line: if as_designed {
                BORDER_STRONG
            } else {
                palette.ink
            },
        }
    }
}

/// A role's colour as three bytes.
pub(crate) fn rgb(role: Role) -> [u8; 3] {
    let it = role.colour();
    [it.red(), it.green(), it.blue()]
}

/// Where the rows go, as either face of the Bar laid them out.
#[derive(Debug, Clone, Copy)]
pub(crate) struct WhereTheRowsAre<'a> {
    /// Each application row shown, with which result it is.
    pub(crate) rows: &'a [(usize, Rectangle<i32, Physical>)],
    /// The hairline over the row that asks.
    pub(crate) hairline: Option<Rectangle<i32, Physical>>,
    /// The row that asks alo.
    pub(crate) ask: Option<Rectangle<i32, Physical>>,
}

/// Draw every application found, the hairline and the row that asks alo onto
/// `canvas`, whose ground is `palette.dock`.
///
/// **Each application found** (`561:324`): tinted in the accent where the
/// keyboard is, its icon, its name, what kind of thing it is, and on the chosen
/// row the hint that Enter opens it. **The row that asks** (`561:339`): alo's
/// mark on its own tile, the request quoted, *a new task*, and on its second
/// line where the answer would come from.
#[expect(
    clippy::too_many_lines,
    reason = "one row of each kind, drawn in the order the design reads them"
)]
pub(crate) fn the_rows(
    canvas: &mut Canvas,
    fonts: &mut FontSystem,
    bar: &TheBarOnTheDisplay<'_>,
    places: WhereTheRowsAre<'_>,
    palette: crate::desktop_look::DesktopPalette,
    scale: DisplayScale,
) {
    let px = |logical: u32| scale.px(logical);
    let measure = Measure::of(alo_appearance::TextScale::ordinary()).on(scale);
    let colours = TheRowsColours::of(palette);
    let chosen_ground = tinted(palette.dock, palette.accent);
    let chosen = bar.chosen.unwrap_or(0);
    for (which, at) in places.rows {
        let Some(result) = bar.results.get(*which) else {
            continue;
        };
        let is_chosen = *which == chosen;
        let ground = if is_chosen {
            canvas.fill(*at, px(A_WELLS_CORNER) as f32, chosen_ground, 1.0);
            chosen_ground
        } else {
            palette.dock
        };
        let (icon, icon_corner, icon_in) = AN_ICON_IS;
        let tile = Rectangle::new(
            (at.loc.x + px(icon_in), at.loc.y + px(icon_in)).into(),
            (px(icon), px(icon)).into(),
        );
        an_applications_icon(canvas, bar, result, tile, px(icon_corner), palette);
        let name_at = (at.loc.x + px(56), at.loc.y + px(17));
        canvas.clipped_to(*at);
        let hint_room = if is_chosen { px(77) } else { px(14) };
        let room = at.size.w - px(56) - hint_room;
        let name_wide = said(
            canvas,
            fonts,
            measure,
            &result.name,
            TheType::Name,
            (name_at.0, name_at.1, room),
            (ground, palette.ink),
        );
        let after = format!("\u{b7}  {}", result.kind);
        said(
            canvas,
            fonts,
            measure,
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
                (at.loc.x + at.size.w - px(14) - px(26), at.loc.y + px(19)).into(),
                (px(26), px(18)).into(),
            );
            a_key_cap(
                canvas,
                cap,
                px(A_KEY_CAPS_CORNER),
                palette.dock,
                colours.key_line,
            );
            ALaunchersIcon::Enter.drawn(canvas, centred(cap, px(12)), colours.secondary);
            let open_wide = crate::painted_text::how_wide(
                fonts,
                bar.words.open_it,
                TheType::SmallLabel.metrics(measure),
            );
            said(
                canvas,
                fonts,
                measure,
                bar.words.open_it,
                TheType::SmallLabel,
                (
                    cap.loc.x - px(8) - open_wide,
                    at.loc.y + px(20),
                    open_wide + px(2),
                ),
                (ground, palette.ink_quiet),
            );
        }
    }

    if let Some(hairline) = places.hairline {
        canvas.fill(hairline, 0.0, palette.edge, 1.0);
    }
    if let (Some(at), Some(asks)) = (places.ask, bar.asks.as_ref()) {
        let is_chosen = chosen == bar.results.len();
        let ground = if is_chosen {
            canvas.fill(at, px(A_WELLS_CORNER) as f32, chosen_ground, 1.0);
            chosen_ground
        } else {
            palette.dock
        };
        let (icon, icon_corner, icon_in) = AN_ICON_IS;
        let tile = Rectangle::new(
            (at.loc.x + px(icon_in), at.loc.y + px(16)).into(),
            (px(icon), px(icon)).into(),
        );
        canvas.fill(tile, px(icon_corner) as f32, rgb(Role::AccentSoft), 1.0);
        // **Teal, because this row is alo** — ADR 0067's one use for it.
        ALaunchersIcon::Alo.drawn(canvas, tile, rgb(Role::AccentDefault));
        canvas.clipped_to(at);
        let room = at.size.w - px(56) - px(14);
        let asked = said(
            canvas,
            fonts,
            measure,
            asks.asking,
            TheType::Name,
            (at.loc.x + px(56), at.loc.y + px(12), room),
            (ground, palette.ink),
        );
        let after = format!("\u{b7}  {}", asks.what_it_is);
        said(
            canvas,
            fonts,
            measure,
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
            canvas,
            Rectangle::new(
                (at.loc.x + px(56), at.loc.y + px(37)).into(),
                (px(14), px(14)).into(),
            ),
            palette.ink_quiet,
        );
        said(
            canvas,
            fonts,
            measure,
            asks.from,
            TheType::SmallLabel,
            (at.loc.x + px(76), at.loc.y + px(36), room - px(20)),
            (ground, palette.ink_quiet),
        );
        canvas.unclipped();
    }
}

/// The chosen row's ground: the accent at [`THE_CHOSEN_TINT`] over `ground`.
fn tinted(ground: [u8; 3], accent: [u8; 3]) -> [u8; 3] {
    let mix = |a: u8, b: u8| {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a blend of two bytes stays a byte"
        )]
        let it =
            (f32::from(b) * THE_CHOSEN_TINT + f32::from(a) * (1.0 - THE_CHOSEN_TINT)).round() as u8;
        it
    };
    let ([gr, gg, gb], [ar, ag, ab]) = (ground, accent);
    [mix(gr, ar), mix(gg, ag), mix(gb, ab)]
}

/// `words` in `style` at (`x`, `y`), no wider than `wide`, shaped on `ground`
/// in `ink` — the ground it is drawn on, so no band of another colour is
/// pasted with it. Answers how wide the ink is, for what follows it.
pub(crate) fn said(
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
pub(crate) fn a_key_cap(
    canvas: &mut Canvas,
    cap: Area,
    corner: i32,
    ground: [u8; 3],
    line: [u8; 3],
) {
    canvas.fill(cap, corner as f32, line, 1.0);
    canvas.fill(shrunk(cap, 1), (corner - 1).max(0) as f32, ground, 1.0);
}

/// A square of `side` centred in `area`.
pub(crate) fn centred(area: Area, side: i32) -> Area {
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
pub(crate) fn shrunk(area: Area, by: i32) -> Area {
    Rectangle::new(
        (area.loc.x + by, area.loc.y + by).into(),
        ((area.size.w - by * 2).max(0), (area.size.h - by * 2).max(0)).into(),
    )
}
