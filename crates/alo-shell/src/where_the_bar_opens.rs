//! Where the launcher's panel goes, and where everything inside it sits.
//!
//! **The launcher the owner approved on 2026-11-01** — `561:275` at 1440 × 960
//! and `561:370` at 1280 × 800, *Find, open, or ask · the launcher*. Every
//! figure below was read off those two frames on that day and is recorded in
//! `docs/design/the-alo-bar-when-it-is-open.md`.
//!
//! # What is a proportion, what is a measure, and what is a rule
//!
//! The **panel's width and top** are proportions of the display — 47.2 % of its
//! width (680 of 1440, 604 of 1280) and 21.7 % of the way down (208 of 960, 174
//! of 800), centred across. The **air inside it** is a proportion of the panel:
//! its width over 26 (26 at 680, 23 at 604).
//!
//! **Everything a person reads or points at is a measure**, logical and passed
//! through `crate::display_scale::DisplayScale` once: the heading's 40, the
//! field's 56, an application's row of 56 and the row that asks alo of 64, the
//! line of 18 under the field, the 12 between things, the 4 between rows, the
//! key hints' 28 and the 15 of air under them. Both frames draw these at the
//! same sizes on panels of different widths, which is what says they are
//! measures and not shares of the panel.
//!
//! **The panel's height is a rule**: it holds what it shows, and no more —
//! `561:309` is 360 tall at 1440 and `561:404` 357 at 1280, for one result and
//! the row that asks. It stops growing at three fifths of the display, and the
//! list scrolls inside it from there, keeping the row the keyboard is on in
//! view.

use smithay::utils::{Physical, Rectangle, Size};

use crate::display_scale::DisplayScale;

/// How tall a thing a person points at is: the field, and an application's
/// row. 56 in both frames.
pub(crate) const A_TARGET_IS_TALL: u32 = 56;

/// How tall the row that asks alo is: 64, for its second line.
pub(crate) const THE_ASK_ROW_IS_TALL: u32 = 64;

/// The gap between the heading, the field, the line under it, the list and
/// the keys: 12.
pub(crate) const BETWEEN_THINGS: u32 = 12;

/// The gap between one row and the next, and either side of the hairline over
/// the row that asks: 4 (`561:338` at 60 under a row of 56, the ask row at 65).
pub(crate) const BETWEEN_ROWS: u32 = 4;

/// How tall the heading is: 40, the close control's height.
pub(crate) const THE_HEADING_IS_TALL: u32 = 40;

/// The close control: 48 wide, the heading's 40 tall (`561:312`).
pub(crate) const THE_CLOSE_CONTROL_IS: (u32, u32) = (48, 40);

/// How tall the line under the field is: 18.
pub(crate) const THE_ORIGIN_LINE_IS_TALL: u32 = 18;

/// How tall the row of key hints is: 28 (`561:351`).
pub(crate) const THE_KEYS_ARE_TALL: u32 = 28;

/// The air under the key hints, to the panel's edge: 15 in both frames,
/// where the air on every other side is a share of the panel's width.
pub(crate) const UNDER_THE_KEYS: u32 = 15;

/// How much air there is inside the panel, as a share of its width: 26 at 680.
pub(crate) const INSIDE_THE_PANEL: f32 = 26.0 / 680.0;

/// How wide the panel is as a share of the display.
const THE_PANEL_IS_WIDE: f32 = 680.0 / 1440.0;

/// How far down the display the panel's top sits.
const THE_PANEL_BEGINS: f32 = 208.0 / 960.0;

/// The most of the display's height the panel takes before its list scrolls.
const AT_MOST: f32 = 3.0 / 5.0;

/// What the launcher holds, which is all its height depends on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WhatItHolds {
    /// How many applications were found.
    pub(crate) found: usize,
    /// Whether the row that asks alo is offered.
    pub(crate) asks: bool,
    /// Which row the keyboard is on: an application's, or past the last of
    /// them, the row that asks.
    pub(crate) chosen: Option<usize>,
}

/// Where the launcher and everything in it is, on a display of this size.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheBarsPlaces {
    /// The panel itself.
    pub panel: Rectangle<i32, Physical>,
    /// The heading's line, across the panel's inside.
    pub heading: Rectangle<i32, Physical>,
    /// The close control, at the heading's end.
    pub close: Rectangle<i32, Physical>,
    /// The field somebody types into.
    pub field: Rectangle<i32, Physical>,
    /// The line that says where the results came from.
    pub origin: Rectangle<i32, Physical>,
    /// Each application row shown, with which result it is.
    pub rows: Vec<(usize, Rectangle<i32, Physical>)>,
    /// The hairline over the row that asks, where there is one.
    pub hairline: Option<Rectangle<i32, Physical>>,
    /// The row that asks alo, where it is offered.
    pub ask: Option<Rectangle<i32, Physical>>,
    /// The row of key hints.
    pub keys: Rectangle<i32, Physical>,
    /// The scale every measure above went through, so the raster draws at it
    /// rather than asking a second time.
    pub(crate) scale: DisplayScale,
}

/// Work out where the launcher goes on a display this size, drawn at `scale`,
/// holding `holds`.
///
/// [`None`] for a display too small to hold the panel and one row at all — a
/// real answer rather than a clamp: a launcher squeezed onto a screen it does
/// not fit is a surface a person cannot use, and the caller draws none.
pub(crate) fn where_the_bar_opens(
    display: Size<i32, Physical>,
    scale: DisplayScale,
    holds: WhatItHolds,
) -> Option<TheBarsPlaces> {
    let px = |logical: u32| scale.px(logical);
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a share of a display's pixels is well inside an i32"
    )]
    let share = |of: i32, share: f32| (of as f32 * share).round() as i32;
    let across = share(display.w, THE_PANEL_IS_WIDE);
    let inset = share(across, INSIDE_THE_PANEL).max(1);
    let inner = across - inset * 2;
    let left = (display.w - across) / 2;
    let top = share(display.h, THE_PANEL_BEGINS);
    if inner <= px(THE_CLOSE_CONTROL_IS.0) {
        return None;
    }

    let heading = Rectangle::new(
        (left + inset, top + inset).into(),
        (inner, px(THE_HEADING_IS_TALL)).into(),
    );
    let close = Rectangle::new(
        (
            left + inset + inner - px(THE_CLOSE_CONTROL_IS.0),
            top + inset,
        )
            .into(),
        (px(THE_CLOSE_CONTROL_IS.0), px(THE_CLOSE_CONTROL_IS.1)).into(),
    );
    let field_top = heading.loc.y + heading.size.h + px(BETWEEN_THINGS);
    let field = Rectangle::new(
        (left + inset, field_top).into(),
        (inner, px(A_TARGET_IS_TALL)).into(),
    );
    let origin_top = field_top + field.size.h + px(BETWEEN_THINGS);
    let origin = Rectangle::new(
        (left + inset, origin_top).into(),
        (inner, px(THE_ORIGIN_LINE_IS_TALL)).into(),
    );
    let list_top = origin_top + origin.size.h + px(BETWEEN_THINGS);

    // **What is under the list, and what the list may have.** The keys and the
    // air under them always; the hairline and the ask row when it is offered.
    let after_the_list = px(BETWEEN_THINGS) + px(THE_KEYS_ARE_TALL) + px(UNDER_THE_KEYS);
    let asking = if holds.asks {
        let above = if holds.found > 0 {
            px(BETWEEN_ROWS) * 2 + scale.px(1).max(1)
        } else {
            0
        };
        above + px(THE_ASK_ROW_IS_TALL)
    } else {
        0
    };
    let a_row = px(A_TARGET_IS_TALL);
    let row_to_row = a_row + px(BETWEEN_ROWS);
    let most_down = share(display.h, AT_MOST);
    let room_for_rows = most_down - (list_top - top) - after_the_list - asking;
    if room_for_rows < 0 || (holds.found > 0 && room_for_rows < a_row) {
        return None;
    }
    // Whole rows only: a row half drawn is a row a person cannot read.
    let fit = usize::try_from((room_for_rows + px(BETWEEN_ROWS)) / row_to_row)
        .unwrap_or(0)
        .min(holds.found);
    // **The row the keyboard is on is always in view**: an application's row
    // past the last that fits scrolls the list, and the row that asks is never
    // scrolled away.
    let first = match holds.chosen {
        Some(chosen) if chosen < holds.found && fit > 0 && chosen >= fit => chosen + 1 - fit,
        _ => 0,
    };
    let mut y = list_top;
    let rows: Vec<(usize, Rectangle<i32, Physical>)> = (first..first + fit)
        .map(|which| {
            let at = Rectangle::new((left + inset, y).into(), (inner, a_row).into());
            y += row_to_row;
            (which, at)
        })
        .collect();
    // Back to the bottom of the last row shown.
    let mut bottom = rows
        .last()
        .map_or(list_top, |(_, at)| at.loc.y + at.size.h);
    let mut hairline = None;
    let ask = holds.asks.then(|| {
        if !rows.is_empty() {
            let line = Rectangle::new(
                (left + inset, bottom + px(BETWEEN_ROWS)).into(),
                (inner, scale.px(1).max(1)).into(),
            );
            bottom = line.loc.y + line.size.h + px(BETWEEN_ROWS);
            hairline = Some(line);
        }
        let at = Rectangle::new(
            (left + inset, bottom).into(),
            (inner, px(THE_ASK_ROW_IS_TALL)).into(),
        );
        bottom = at.loc.y + at.size.h;
        at
    });
    let keys = Rectangle::new(
        (left + inset, bottom + px(BETWEEN_THINGS)).into(),
        (inner, px(THE_KEYS_ARE_TALL)).into(),
    );
    let down = keys.loc.y + keys.size.h + px(UNDER_THE_KEYS) - top;
    let panel = Rectangle::new((left, top).into(), (across, down).into());

    Some(TheBarsPlaces {
        panel,
        heading,
        close,
        field,
        origin,
        rows,
        hairline,
        ask,
        keys,
        scale,
    })
}

#[cfg(test)]
#[path = "where_the_bar_opens_tests.rs"]
mod tests;
