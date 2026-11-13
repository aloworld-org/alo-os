//! **Where Settings' side column is scrolled to**, and the two roads that move
//! it: a wheel over it, and the keyboard or a press choosing a category.
//!
//! The owner, 2026-11-13: *"The side column scrolls"* — on a screen where its
//! entries do not all fit, nothing is hidden or made smaller; the column
//! scrolls (`crate::settings_drawn_side_column`).
//!
//! # A wheel leaves it where it put it; a choice brings the choice into view
//!
//! [`TheSideColumnsScroll::follows`] is the one distinction. A wheel turns it
//! off, so the next frame draws the column where the wheel left it, even with
//! the chosen category out of view — a person reading down the list is not
//! pulled back to the page they are on. Up, Down, Home, End, *Go to* and a
//! press on a category turn it on, so the frame scrolls as little as brings the
//! chosen category whole into view.
//!
//! **Where the frame drew it is written back** after every frame
//! (`Server::settings_was_drawn`), so that bringing a category into
//! view moves the column from where the person sees it, not from wherever a
//! wheel last left a number.
//!
//! # The wheel is the column's only where it can move
//!
//! A wheel over a column that fits does what it did before this file: it
//! reaches the canvas. Over one that scrolls, it is the column's, and it never
//! pans the canvas behind it — even at either end — as a task window keeps its
//! wheel (`crate::a_task_window_scrolls`).

use smithay::input::pointer::AxisFrame;
use smithay::utils::{Physical, Point};

use crate::settings_window::SettingsWindow;

/// Where the side column is scrolled, and whether the next frame brings the
/// chosen category into view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TheSideColumnsScroll {
    /// How far it is scrolled, in the picture's pixels, as the last frame drew
    /// it or the last wheel left it.
    pub(crate) scrolled: i32,
    /// Whether the chosen category is brought whole into view.
    pub(crate) follows: bool,
}

impl TheSideColumnsScroll {
    /// Not scrolled, and following the keyboard: the column as Settings opens.
    pub(crate) const fn at_the_top() -> Self {
        Self {
            scrolled: 0,
            follows: true,
        }
    }

    /// The keyboard, or a press, chose a category: bring it into view.
    pub(crate) const fn a_category_was_chosen(&mut self) {
        self.follows = true;
    }

    /// A wheel moved it `by`, and it can go no further than `most`. Answers
    /// whether it moved.
    pub(crate) fn wheeled(&mut self, by: i32, most: i32) -> bool {
        let before = self.scrolled.clamp(0, most.max(0));
        self.scrolled = before.saturating_add(by).clamp(0, most.max(0));
        self.follows = false;
        self.scrolled != before
    }

    /// The frame drew it `scrolled`.
    pub(crate) const fn was_drawn_at(&mut self, scrolled: i32) {
        self.scrolled = scrolled;
    }
}

/// How far a scroll frame asks the column to move, in the picture's pixels:
/// down for a wheel turned toward the person, as every list scrolls.
///
/// **A notch is 15 of the units an `AxisFrame` carries here**, the figure
/// `crate::a_task_window_scrolls` and `crate::canvas_wheel_zoom` use, and a
/// notch moves `a_notch`: one category and the gap after it.
pub(crate) fn how_far_down(frame: &AxisFrame, a_notch: i32) -> i32 {
    /// A wheel notch, in the units an [`AxisFrame`] carries here.
    const A_NOTCH: f64 = 15.0;
    let down = frame.axis.1 / A_NOTCH * f64::from(a_notch);
    if !down.is_finite() {
        return 0;
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "clamped to a small range first, so the cast cannot truncate"
    )]
    let whole = down.clamp(-1_000_000.0, 1_000_000.0).round() as i32;
    whole
}

impl SettingsWindow {
    /// A wheel moved the side column `by`, no further than `most`. Answers
    /// whether it moved; nothing happens while Settings is shut.
    pub(crate) fn the_side_column_wheeled(&mut self, by: i32, most: i32) -> bool {
        self.open_mut()
            .is_some_and(|open| open.side_column.wheeled(by, most))
    }

    /// The last frame drew the side column `scrolled`.
    pub(crate) fn the_side_column_was_drawn_at(&mut self, scrolled: i32) {
        if let Some(open) = self.open_mut() {
            open.side_column.was_drawn_at(scrolled);
        }
    }
}

impl crate::Server {
    /// A wheel at `at`, a point on the display, offered to Settings' side
    /// column.
    ///
    /// [`None`] where the pointer is not over the side column of an open
    /// Settings, or the column fits and has nowhere to move: the scroll goes
    /// where it went before. Otherwise the column's, whether or not it moved.
    pub(crate) fn a_scroll_over_settings(
        &mut self,
        at: Point<i32, Physical>,
        frame: &AxisFrame,
    ) -> Option<bool> {
        if !self.settings_is_open_here() {
            return None;
        }
        let side = self
            .settings_as_drawn
            .as_ref()?
            .the_side_column_that_scrolls_under(at)?;
        let by = how_far_down(frame, side.a_notch);
        Some(self.settings.the_side_column_wheeled(by, side.most))
    }
}

#[cfg(test)]
#[path = "settings_side_column_scrolls_tests.rs"]
mod tests;

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "a_wheel_reaches_settings_tests.rs"]
mod a_wheel_reaches_settings_tests;
