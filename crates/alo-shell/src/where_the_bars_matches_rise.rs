//! Where the alo Bar's matches go: **a compact list rising from the Dock's own
//! field**, not a panel in the middle of the screen.
//!
//! *The owner's decision of 2026-11-11*: "Clicking 'Find, open, or ask…' on the
//! Dock should not open the same panel as Super+Space. It should allow user to
//! text inside it", and of the matches, *rise just above the field* — a small
//! list that grows upward from the Dock's field as a person types, holding the
//! same matches and the same *Ask alo "…"* row, anchored to the field, with
//! nothing in the middle of the screen. Super+Space puts the keyboard in the
//! same field. This **departs from `561:275` / `561:370`**, the centred
//! launcher approved on 2026-11-01, and
//! `docs/design/the-alo-bar-when-it-is-open.md` records the departure.
//!
//! # Placed by the rules *More apps* is placed by
//!
//! `docs/features.md` promises *More apps* as *compact, non-modal … anchored to
//! that control — above a bottom Dock, below a top one, inward from a side one —
//! inside the display and clear of the panel of minimised windows*. This list
//! keeps every clause of that, anchored to the field instead of the control:
//!
//! - **as wide as *More apps*** — `alo_dock::more_apps::TheOverview::width`, a
//!   logical measure — starting where the field starts;
//! - **the Dock's gap from the bar**, `BESIDE_THE_DOCK`, the room between two
//!   surfaces;
//! - **kept on the display and out of the panel's column** by
//!   `crate::where_a_popover_is_kept`, the function *More apps* now uses too.
//!
//! **The rows are the centred launcher's rows**, at the measures read off
//! `561:275` / `561:370` on 2026-11-01 and declared below: an application's 56,
//! the row that asks alo's 64, 4 between rows and the hairline over the row
//! that asks. Same rows, same order, same keys; only where they are shown has
//! changed. The centred launcher itself is gone — nothing opened it once *Help
//! with this window* moved into the window's own panel — so these measures are
//! declared here, by the one surface that still draws them.
//!
//! # The line that says where the results came from
//!
//! **Carried over from the centred launcher, under what is typed**, at its 18
//! and with its 12 below: *On this computer · 3 applications*, or — on a
//! machine where nothing is chosen to answer a question — that nothing is
//! (`crate::the_bar_finds::nothing_can_answer_a_question`). So **anything typed
//! shows the list**, even where nothing matches and nothing can be asked: the
//! person is told while they are still typing, which is what that sentence
//! exists for, rather than meeting silence on Enter.
//!
//! # A side Dock has no room to type in, so the list carries the line
//!
//! On the left and right Docks the field is a 48-wide slot running down the
//! bar, which cannot hold a line of text. There the list opens **the moment the
//! field takes the keyboard**, with what is typed on its first line, so a
//! person on a side Dock sees their words as somebody on the bottom Dock sees
//! them in the field. On the bottom and top Docks the words are in the field
//! itself and the list is not shown until something is typed.
//!
//! # Small displays
//!
//! The list may grow to **three fifths of the display**, the centred launcher's
//! own bound, and never past the room between the bar and the far edge. Past
//! that the rows scroll, keeping the one the keyboard is on in view, and the row
//! that asks is never scrolled away. A display with no room for even that row
//! draws no list: [`None`], not a list squeezed into nothing.

use smithay::utils::{Physical, Rectangle, Size};

use crate::display_scale::DisplayScale;

/// How tall an application's row is: 56 (`561:324`).
pub(crate) const A_TARGET_IS_TALL: u32 = 56;

/// How tall the row that asks alo is: 64, for its second line (`561:339`).
pub(crate) const THE_ASK_ROW_IS_TALL: u32 = 64;

/// The gap between one row and the next, and either side of the hairline over
/// the row that asks: 4 (`561:338` at 60 under a row of 56, the ask row at 65).
pub(crate) const BETWEEN_ROWS: u32 = 4;

/// How tall the line saying where the results came from is: 18.
const THE_ORIGIN_LINE_IS_TALL: u32 = 18;

/// The gap between that line and the rows under it: 12, the centred launcher's
/// gap between things.
const UNDER_THE_ORIGIN_LINE: u32 = 12;

/// The most of the display's height the list takes before it scrolls: the
/// centred launcher's three fifths.
const AT_MOST: f32 = 3.0 / 5.0;

/// What the list holds, which is all its height depends on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WhatItHolds {
    /// How many applications were found.
    pub(crate) found: usize,
    /// Whether the row that asks alo is offered.
    pub(crate) asks: bool,
    /// Whether the line saying where the results came from is shown — while
    /// something is typed.
    pub(crate) says: bool,
    /// Which row the keyboard is on: an application's, or past the last of
    /// them, the row that asks.
    pub(crate) chosen: Option<usize>,
}

/// Where the field the list rises from is, on the Dock as last drawn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TheFieldOnTheDock {
    /// The field itself.
    pub(crate) field: Rectangle<i32, Physical>,
    /// The Dock's whole band, which the list keeps its gap from.
    pub(crate) band: Rectangle<i32, Physical>,
    /// Which edge the Dock is on, so *inward* has a direction.
    pub(crate) edge: alo_dock::Edge,
}

/// Where the list and everything in it is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TheMatchesPlaces {
    /// The list's own panel.
    pub(crate) panel: Rectangle<i32, Physical>,
    /// The Dock's field it rises from. **A press here is the field's**, not a
    /// press outside the list.
    pub(crate) field: Rectangle<i32, Physical>,
    /// The line that shows what is typed, on a side Dock only.
    pub(crate) typed: Option<Rectangle<i32, Physical>>,
    /// The line saying where the results came from, while something is typed.
    pub(crate) origin: Option<Rectangle<i32, Physical>>,
    /// Each application row shown, with which result it is.
    pub(crate) rows: Vec<(usize, Rectangle<i32, Physical>)>,
    /// The hairline over the row that asks, where there is one.
    pub(crate) hairline: Option<Rectangle<i32, Physical>>,
    /// The row that asks alo, where it is offered.
    pub(crate) ask: Option<Rectangle<i32, Physical>>,
    /// The scale every measure above went through.
    pub(crate) scale: DisplayScale,
}

/// Work out where the matches rise to on a display of `display`, drawn at
/// `scale`, from `dock`'s field, clear of `clear_of`.
///
/// [`None`] when there is nothing to show — nothing typed on a bottom or top
/// Dock — and when the display has no room for the list at all.
pub(crate) fn where_the_matches_rise(
    display: Size<i32, Physical>,
    scale: DisplayScale,
    holds: WhatItHolds,
    dock: TheFieldOnTheDock,
    clear_of: Option<Rectangle<i32, Physical>>,
) -> Option<TheMatchesPlaces> {
    let px = |logical: u32| scale.px(logical);
    let carries_the_line = !dock.edge.runs_across();
    if holds.found == 0 && !holds.asks && !holds.says && !carries_the_line {
        return None;
    }
    let gap = px(alo_dock::measures::BESIDE_THE_DOCK);
    let air = px(alo_dock::measures::GAP);
    let across = scale.room(alo_dock::more_apps::TheOverview::width());
    let inner = across - air * 2;
    let a_row = px(A_TARGET_IS_TALL);
    let row_to_row = a_row + px(BETWEEN_ROWS);
    let a_line = scale.px(1).max(1);

    // **The room between the bar and the far side**, which the list may grow
    // into, and the centred launcher's own bound on top of that.
    let headroom = match dock.edge {
        alo_dock::Edge::Bottom => dock.band.loc.y - gap,
        alo_dock::Edge::Top => display.h - (dock.band.loc.y + dock.band.size.h) - gap,
        alo_dock::Edge::Left | alo_dock::Edge::Right => display.h,
    };
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a share of a display's pixels is well inside an i32"
    )]
    let most = headroom.min((display.h as f32 * AT_MOST).round() as i32);

    let typed_line = if carries_the_line {
        a_row + px(BETWEEN_ROWS)
    } else {
        0
    };
    let asking = if holds.asks {
        let above = if holds.found > 0 {
            px(BETWEEN_ROWS) * 2 + a_line
        } else {
            0
        };
        above + px(THE_ASK_ROW_IS_TALL)
    } else {
        0
    };
    let origin_line = if holds.says {
        px(THE_ORIGIN_LINE_IS_TALL) + px(UNDER_THE_ORIGIN_LINE)
    } else {
        0
    };
    let room_for_rows = most - air * 2 - typed_line - origin_line - asking;
    if room_for_rows < 0 {
        return None;
    }
    // Whole rows only: a row half drawn is a row a person cannot read.
    let fit = usize::try_from((room_for_rows + px(BETWEEN_ROWS)) / row_to_row)
        .unwrap_or(0)
        .min(holds.found);
    if holds.found > 0 && fit == 0 && !holds.asks {
        return None;
    }
    let rows_tall = if fit == 0 {
        0
    } else {
        i32::try_from(fit).unwrap_or(0) * row_to_row - px(BETWEEN_ROWS)
    };
    let tall = air * 2 + typed_line + origin_line + rows_tall + asking;
    // **A list with nothing under its last line has no gap under that line**:
    // the line saying where the results came from, or a side Dock's typed line.
    let tall = if fit == 0 && asking == 0 {
        if holds.says {
            tall - px(UNDER_THE_ORIGIN_LINE)
        } else {
            tall - px(BETWEEN_ROWS)
        }
    } else {
        tall
    };

    // **Above a bottom Dock, below a top one, inward from a side one**, starting
    // where the field starts.
    let origin = match dock.edge {
        alo_dock::Edge::Bottom => (dock.field.loc.x, dock.band.loc.y - gap - tall),
        alo_dock::Edge::Top => (dock.field.loc.x, dock.band.loc.y + dock.band.size.h + gap),
        alo_dock::Edge::Left => (dock.band.loc.x + dock.band.size.w + gap, dock.field.loc.y),
        alo_dock::Edge::Right => (dock.band.loc.x - gap - across, dock.field.loc.y),
    };
    let panel = crate::where_a_popover_is_kept::kept_on_the_display(
        origin,
        (across, tall),
        (display.w, display.h),
        clear_of,
        gap,
    );
    let left = panel.loc.x + air;
    let mut y = panel.loc.y + air;
    let typed = carries_the_line.then(|| {
        let at = Rectangle::new((left, y).into(), (inner, a_row).into());
        y += a_row + px(BETWEEN_ROWS);
        at
    });
    let where_from = holds.says.then(|| {
        let at = Rectangle::new(
            (left, y).into(),
            (inner, px(THE_ORIGIN_LINE_IS_TALL)).into(),
        );
        y += px(THE_ORIGIN_LINE_IS_TALL) + px(UNDER_THE_ORIGIN_LINE);
        at
    });

    // **The row the keyboard is on is always in view**: an application's row
    // past the last that fits scrolls the list.
    let first = match holds.chosen {
        Some(chosen) if chosen < holds.found && fit > 0 && chosen >= fit => chosen + 1 - fit,
        _ => 0,
    };
    let rows: Vec<(usize, Rectangle<i32, Physical>)> = (first..first + fit)
        .map(|which| {
            let at = Rectangle::new((left, y).into(), (inner, a_row).into());
            y += row_to_row;
            (which, at)
        })
        .collect();
    let mut bottom = rows.last().map_or(y, |(_, at)| at.loc.y + at.size.h);
    let mut hairline = None;
    let ask = holds.asks.then(|| {
        if !rows.is_empty() {
            let line = Rectangle::new(
                (left, bottom + px(BETWEEN_ROWS)).into(),
                (inner, a_line).into(),
            );
            bottom = line.loc.y + line.size.h + px(BETWEEN_ROWS);
            hairline = Some(line);
        }
        Rectangle::new(
            (left, bottom).into(),
            (inner, px(THE_ASK_ROW_IS_TALL)).into(),
        )
    });

    Some(TheMatchesPlaces {
        panel,
        field: dock.field,
        typed,
        origin: where_from,
        rows,
        hairline,
        ask,
        scale,
    })
}

#[cfg(test)]
#[path = "where_the_bars_matches_rise_tests.rs"]
mod tests;
