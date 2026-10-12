//! The list of windows an icon with several of them opens.
//!
//! The owner's rule, which this is the second half of:
//!
//! > **Click an app to return to where you last used it. Choose a preview when
//! > you want a different window.**
//!
//! [`crate::clicking`] is the first sentence. Since ADR 0103 it answers
//! [`crate::clicking::WhatAClickDoes::ChooseAWindow`] for an application with
//! more than one window, and **this is what that opens**.
//!
//! # What is here, and what is not
//!
//! [`crate::previews::Previews`] already decides *which windows, in what order,
//! and which one is marked*. [`crate::a_list_the_dock_opens::ThePanel`] already
//! decides *how tall a list of n rows is and which of them are on the screen*.
//! Neither is repeated.
//!
//! What was missing is the one question a caller cannot answer without both:
//! **which window is the row a person just pressed?** A list that scrolls makes
//! the third row on the screen the ninth row in the list, and a caller doing
//! that arithmetic at the press is a caller that will do it differently from
//! the one that drew — which focuses the wrong window, silently, and only when
//! somebody has enough windows to scroll.
//!
//! So it is one type that holds both and answers that question, and the drawing
//! and the press ask the same object.
//!
//! # It is not the panel at the edge
//!
//! The right-hand panel holds windows a person **put aside**, across every
//! application, until they bring them back. This list is **per application**,
//! appears on demand, and is how somebody reaches a particular window —
//! including ones that are not put aside at all. ADR 0102 and
//! `docs/design/the-alo-dock.md` both say so; it is restated here because the
//! two lists look alike and one of them is a permanent surface.

use crate::a_list_the_dock_opens::ThePanel;
use crate::previews::{Preview, Previews};
use crate::room::Room;
use crate::window::WindowId;
use alo_appearance::TextScale;

/// One application's windows, laid out as a list that may scroll.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThePicker {
    /// Which windows, in order, and which is marked.
    previews: Previews,
    /// How tall the list is and which rows are on the screen.
    panel: ThePanel,
}

impl ThePicker {
    /// Lay `previews` out in the room there is for them.
    ///
    /// `room` is what the list may grow into before it scrolls — the distance
    /// between the bar and the far side of the display, which only the caller
    /// knows.
    #[must_use]
    pub fn of(previews: Previews, text: TextScale, room: Room) -> Self {
        let panel = ThePanel::of(previews.how_many(), text, room);
        Self { previews, panel }
    }

    /// How tall it is, how tall a row is, and which rows are on the screen.
    #[must_use]
    pub const fn panel(&self) -> ThePanel {
        self.panel
    }

    /// The windows, in the order they are drawn.
    #[must_use]
    pub fn rows(&self) -> &[Preview] {
        self.previews.rows()
    }

    /// Which application's windows these are.
    ///
    /// The heading names it, and the heading is a sentence somebody says — so
    /// this is here for the lane that says it, not for the drawing.
    #[must_use]
    pub const fn app(&self) -> &crate::window::AppId {
        self.previews.app()
    }

    /// Whether there is anything to choose between.
    ///
    /// **A picker of one window is a picker that should not have opened.**
    /// `clicking` answers `ChooseAWindow` only above one, so this being true is
    /// a caller that opened it for an application whose windows closed in
    /// between — an ordinary race, and the answer is to close rather than to
    /// show somebody a list with one thing on it.
    #[must_use]
    pub fn is_worth_showing(&self) -> bool {
        self.previews.how_many() > 1
    }

    /// Which window the `which`th row **on the screen** names.
    ///
    /// **Counted from what is shown, not from the whole list**, which is the
    /// whole reason this type exists. A list scrolled down three makes the
    /// first row on the screen the fourth window, and a caller that indexed the
    /// list directly would focus the wrong one — silently, and only for
    /// somebody with enough windows to scroll.
    ///
    /// [`None`] for a row that is not on the screen, which includes every row
    /// of an empty list.
    #[must_use]
    pub fn which_window(&self, which: usize) -> Option<WindowId> {
        let (first, showing) = self.panel.showing();
        if which >= showing {
            return None;
        }
        self.previews.rows().get(first + which).map(Preview::id)
    }

    /// Where the `which`th row on the screen sits, down from the list's top.
    ///
    /// The same counting as [`Self::which_window`], and they are deliberately
    /// the same arithmetic in one place: a drawing that put the rows at one set
    /// of offsets and a press that read another is the fault this type removes.
    #[must_use]
    pub fn where_a_row_sits(&self, which: usize) -> Option<Room> {
        let (first, showing) = self.panel.showing();
        if which >= showing {
            return None;
        }
        self.panel.where_a_row_sits(first + which)
    }

    /// Scroll the least it can so that the `which`th **window** is on screen.
    ///
    /// Takes an index into the whole list rather than into what is shown,
    /// because the caller wanting this knows which window it means and not
    /// where it currently is — a keyboard moving focus onto a row that is
    /// off the bottom, which is the owner's *focused items scroll into view*.
    #[must_use]
    pub fn showing_window(mut self, which: usize) -> Self {
        self.panel = self.panel.showing_row(which);
        self
    }
}

#[cfg(test)]
#[path = "picking_tests.rs"]
mod tests;

/// How big the picker's panel is, and where its rows sit.
///
/// The same 448 × 536 popover as the overview, with a taller heading — it holds
/// a back control — and a line saying how many windows there are. Measured off
/// `State=Windows` in `docs/design/figma-snapshot/406-26881.xml`:
///
/// ```text
/// heading  25 ..  73   (48)   a 48x48 back control, then the title at 56
/// count    89 .. 137   (48)   "2 windows · Choose one to return"
/// list    153 .. 473  (320)   16 shorter than the overview's, for the heading
/// footer  489 .. 505   (16)
/// ```
#[must_use]
pub fn the_pickers_panel(how_many: usize, room: Room) -> ThePickersPanel {
    use crate::measures::{
        A_WINDOW_ROW_IS_TALL, BETWEEN_APP_CELLS, DOWN_MORE_APPS, INSIDE_MORE_APPS,
        MORE_APPS_IS_AT_MOST_TALL, THE_DOCKS_EDGE, THE_MORE_APPS_FOOTER_IS_TALL,
        THE_PICKERS_HEADING_IS_TALL, THE_WINDOW_COUNT_IS_TALL,
    };
    let fixed = THE_DOCKS_EDGE * 2
        + INSIDE_MORE_APPS * 2
        + THE_PICKERS_HEADING_IS_TALL
        + THE_WINDOW_COUNT_IS_TALL
        + THE_MORE_APPS_FOOTER_IS_TALL
        + DOWN_MORE_APPS * 3;
    let how_many = u32::try_from(how_many).unwrap_or(u32::MAX);
    let whole = how_many
        .saturating_mul(A_WINDOW_ROW_IS_TALL)
        .saturating_add(how_many.saturating_sub(1).saturating_mul(BETWEEN_APP_CELLS));
    // **The same cap as the overview**, and for the same reason: the owner's
    // 536 is about the popover and not about what is in it. The viewport is
    // what gives way, here as there.
    let viewport = MORE_APPS_IS_AT_MOST_TALL
        .saturating_sub(fixed)
        .min(room.as_pixels().saturating_sub(fixed))
        .min(whole);
    ThePickersPanel {
        how_many,
        viewport,
        tall: fixed.saturating_add(viewport),
        whole,
        scrolled: 0,
    }
}

/// The picker's panel, laid out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThePickersPanel {
    /// How many windows are in it.
    how_many: u32,
    /// How tall the scrolling list is.
    viewport: u32,
    /// How tall the whole popover is.
    tall: u32,
    /// How tall every row together is, scrolled or not.
    whole: u32,
    /// How far down the list has been scrolled.
    ///
    /// **This panel had no scroll at all until 2026-10-11**, and the draw drew
    /// every row at its content offset and clipped away whatever fell past the
    /// viewport. So a person with more windows than the list could hold could
    /// see the first few and reach none of the rest — the rows were not hidden
    /// behind a scroll, they were simply not reachable. The design says
    /// otherwise in as many words: *keyboard focus must scroll the focused row
    /// into view.*
    scrolled: u32,
}

impl ThePickersPanel {
    /// How tall the whole popover is. **Never more than the owner's cap.**
    #[must_use]
    pub const fn height(self) -> Room {
        Room::pixels(self.tall)
    }

    /// How tall the scrolling list inside it is.
    #[must_use]
    pub const fn the_list_is_tall(self) -> Room {
        Room::pixels(self.viewport)
    }

    /// Whether there is more than the viewport can hold.
    #[must_use]
    pub const fn scrolls(self) -> bool {
        self.whole > self.viewport
    }

    /// How far down the list has been scrolled.
    #[must_use]
    pub const fn scrolled(self) -> Room {
        Room::pixels(self.scrolled)
    }

    /// The most it can be scrolled, which is everything below the viewport.
    #[must_use]
    pub const fn the_furthest_it_scrolls(self) -> Room {
        Room::pixels(self.whole.saturating_sub(self.viewport))
    }

    /// Scroll the least it can so that the `which`th row is wholly in view.
    ///
    /// **The least it can**, so arrowing one row past the bottom moves the list
    /// one row and not to that row's own top — `ThePanel::showing_row` says why
    /// at greater length, and this is deliberately the same rule: a person
    /// should not lose their place in a list by reaching the end of it.
    ///
    /// `which` indexes the whole list. A row past the end leaves the panel
    /// alone rather than scrolling to nowhere.
    #[must_use]
    pub fn showing(mut self, which: usize) -> Self {
        let Some(top) = self.where_a_row_sits(which).map(Room::as_pixels) else {
            return self;
        };
        let bottom = top.saturating_add(crate::measures::A_WINDOW_ROW_IS_TALL);
        if top < self.scrolled {
            self.scrolled = top;
        } else if bottom > self.scrolled.saturating_add(self.viewport) {
            self.scrolled = bottom.saturating_sub(self.viewport);
        }
        self.scrolled = self
            .scrolled
            .min(self.the_furthest_it_scrolls().as_pixels());
        self
    }

    /// Which row a point in the **list's viewport** lands on.
    ///
    /// `down` is measured from the viewport's own top edge, before the scroll
    /// is undone — this adds [`Self::scrolled`] itself, for
    /// `TheOverview::which_cell_is_at`'s reason exactly.
    ///
    /// [`None`] in the gap between two rows and past the last window. A list is
    /// one column, so there is no across to check.
    #[must_use]
    pub fn which_row_is_at(self, down: Room) -> Option<usize> {
        if down.as_pixels() >= self.viewport {
            return None;
        }
        let into_the_content = down.as_pixels().saturating_add(self.scrolled);
        let stride = crate::measures::A_WINDOW_ROW_IS_TALL
            .saturating_add(crate::measures::BETWEEN_APP_CELLS);
        let row = into_the_content.checked_div(stride)?;
        if into_the_content.saturating_sub(row.saturating_mul(stride))
            >= crate::measures::A_WINDOW_ROW_IS_TALL
        {
            return None;
        }
        if row >= self.how_many {
            return None;
        }
        usize::try_from(row).ok()
    }

    /// Whether the `which`th row is at least partly in view.
    #[must_use]
    pub fn is_shown(self, which: usize) -> bool {
        let Some(top) = self.where_a_row_sits(which).map(Room::as_pixels) else {
            return false;
        };
        top.saturating_add(crate::measures::A_WINDOW_ROW_IS_TALL) > self.scrolled
            && top < self.scrolled.saturating_add(self.viewport)
    }

    /// Where the `which`th row sits inside the list's content.
    ///
    /// **Before any scrolling**, like `TheOverview::where_a_cell_sits` — the
    /// draw subtracts [`Self::scrolled`] once, for the group headings and the
    /// rows together.
    ///
    /// [`None`] past the last window.
    #[must_use]
    pub fn where_a_row_sits(self, which: usize) -> Option<Room> {
        let which = u32::try_from(which).ok()?;
        if which >= self.how_many {
            return None;
        }
        Some(Room::pixels(which.saturating_mul(
            crate::measures::A_WINDOW_ROW_IS_TALL + crate::measures::BETWEEN_APP_CELLS,
        )))
    }
}
