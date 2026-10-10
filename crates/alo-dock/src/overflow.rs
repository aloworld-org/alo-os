//! The list the overflow control opens, laid out.
//!
//! **The owner's ruling of 2026-10-10**, asked for because the design file's
//! rows are 39 apart and 39 is below the floor every control in this product is
//! held to:
//!
//! > Overflow rows have a minimum 44px hit-target height. Remove the 39px pitch;
//! > do not create an exception. Allow rows to grow for larger text. Let the
//! > panel grow beyond 313px when space permits, then scroll its list within the
//! > available screen space. Keyboard focus must scroll the focused row into
//! > view.
//!
//! # What this file decides, and what it does not
//!
//! It decides **how tall a row is, how tall the panel is, and which rows are on
//! the screen**. It does not decide where the panel sits — that is the
//! compositor's, which knows the edge and the screen — and it does not know what
//! a name says, because a crate that cannot measure a font has no business
//! deciding how wide a word is.
//!
//! **Nothing here reads a screen.** How much room the panel may take is passed
//! in, which is `crate::layout`'s rule: a settings panel previewing *what would
//! this look like at 200%* asks exactly the question the compositor asks at
//! 200%.
//!
//! # Why the rows scroll by whole rows
//!
//! Because *keyboard focus must scroll the focused row into view* has to be
//! exact. A list scrolled by pixels can leave a focused row half off the top,
//! which is the same bug as a target below 44: the person who most needs the
//! keyboard is the one who cannot see where they are. Scrolling by rows makes
//! *in view* a whole-number question with one answer.

use alo_appearance::TextScale;
use alo_appearance::targets::ENHANCED_TARGET;

use crate::measures::{A_DIVIDER, AN_OVERFLOW_PANEL_IS_WIDE, AROUND_THE_OVERFLOWS_HEADING, GAP};
use crate::room::Room;

/// How tall one row of the overflow list is at this text size.
///
/// **The larger of the standard's floor and what the text needs**, which is the
/// whole of the owner's second clause: *a minimum 44px hit-target height … allow
/// rows to grow for larger text.*
///
/// The floor is `alo_appearance::targets::ENHANCED_TARGET` — WCAG 2.5.5's 44,
/// what every control in alo OS is built to — rather than a number repeated
/// here. A row that only ever took the floor would lose its text at 200%; a row
/// that only ever took the text would be 37 at 100% and below the floor. It is
/// the maximum of the two and never a choice between them.
#[must_use]
pub fn a_row_at(text: TextScale) -> Room {
    let for_the_text = Room::a_line_at(text)
        .as_pixels()
        .saturating_add(GAP.saturating_mul(2));
    Room::pixels(for_the_text.max(ENHANCED_TARGET))
}

/// How much the heading and its rule take above the first row.
#[must_use]
pub fn above_the_rows_at(text: TextScale) -> Room {
    Room::pixels(
        AROUND_THE_OVERFLOWS_HEADING
            .saturating_add(Room::a_line_at(text).as_pixels())
            .saturating_add(AROUND_THE_OVERFLOWS_HEADING)
            .saturating_add(A_DIVIDER),
    )
}

/// The overflow list, laid out for a text size and the room it has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThePanel {
    /// How many rows there are in all.
    how_many: usize,
    /// How tall one row is.
    a_row: Room,
    /// How much the heading and its rule take.
    above_the_rows: Room,
    /// How many rows are on the screen at once.
    showing: usize,
    /// The first row on the screen.
    first_shown: usize,
}

impl ThePanel {
    /// Lay the list out for this many rows, this text size and this much room.
    ///
    /// `room` is how tall the panel may be — the screen less the Dock and the
    /// margins, worked out by whoever knows the edge. The panel takes what its
    /// contents need up to that, **and then scrolls**: there is no 313, and no
    /// other ceiling either.
    ///
    /// **At least one row is always shown**, even where the room does not
    /// honestly allow it. A panel showing none would be a control a person can
    /// press to reach nothing, and a display that small is a layout problem for
    /// `crate::Screen` rather than something to resolve by showing an empty box.
    #[must_use]
    pub fn of(how_many: usize, text: TextScale, room: Room) -> Self {
        let a_row = a_row_at(text);
        let above_the_rows = above_the_rows_at(text);
        let for_rows = room.as_pixels().saturating_sub(above_the_rows.as_pixels());
        let fits = (for_rows / a_row.as_pixels().max(1)) as usize;
        Self {
            how_many,
            a_row,
            above_the_rows,
            // **None when there are none**, and at least one otherwise. The
            // `max(1)` this needed to keep `clamp` legal said a list of no rows
            // showed one, so `is_shown(0)` was true for a row that does not
            // exist — found by this file's own test rather than reasoned about,
            // which is the case for writing the empty one down.
            showing: if how_many == 0 {
                0
            } else {
                fits.clamp(1, how_many)
            },
            first_shown: 0,
        }
    }

    /// How wide the panel is: [`AN_OVERFLOW_PANEL_IS_WIDE`], measured, and the
    /// same on all four edges.
    #[must_use]
    pub const fn width(self) -> Room {
        Room::pixels(AN_OVERFLOW_PANEL_IS_WIDE)
    }

    /// How tall the panel is: the heading, and the rows that are on the screen.
    #[must_use]
    pub fn height(self) -> Room {
        Room::pixels(
            self.above_the_rows.as_pixels().saturating_add(
                self.a_row
                    .as_pixels()
                    .saturating_mul(u32::try_from(self.showing).unwrap_or(u32::MAX)),
            ),
        )
    }

    /// How tall one row is.
    #[must_use]
    pub const fn a_row(self) -> Room {
        self.a_row
    }

    /// How much the heading and its rule take above the first row.
    #[must_use]
    pub const fn above_the_rows(self) -> Room {
        self.above_the_rows
    }

    /// How many rows there are in all.
    #[must_use]
    pub const fn how_many(self) -> usize {
        self.how_many
    }

    /// The first row on the screen, and how many are.
    #[must_use]
    pub const fn showing(self) -> (usize, usize) {
        (self.first_shown, self.showing)
    }

    /// Whether some rows are off the screen.
    #[must_use]
    pub const fn scrolls(self) -> bool {
        self.showing < self.how_many
    }

    /// Whether this row is on the screen now.
    #[must_use]
    pub const fn is_shown(self, row: usize) -> bool {
        row >= self.first_shown && row < self.first_shown.saturating_add(self.showing)
    }

    /// How far below the panel's own top this row sits, or [`None`] if it is
    /// scrolled off.
    ///
    /// **`None` and not a number off the end**, because the one caller is a draw
    /// and a row at a negative offset is a row painted over the heading.
    #[must_use]
    pub fn where_a_row_sits(self, row: usize) -> Option<Room> {
        if !self.is_shown(row) {
            return None;
        }
        let down = u32::try_from(row.saturating_sub(self.first_shown)).unwrap_or(u32::MAX);
        Some(Room::pixels(
            self.above_the_rows
                .as_pixels()
                .saturating_add(self.a_row.as_pixels().saturating_mul(down)),
        ))
    }

    /// The same panel, scrolled the least it can be to put this row on the
    /// screen.
    ///
    /// **The least it can be**, so that moving focus down a list by one moves
    /// the list by one and not to the row's own top. A list that jumped would
    /// lose a person their place every time they arrowed past the bottom, which
    /// is the thing the owner's clause is about.
    ///
    /// A row past the end leaves the panel alone rather than scrolling to
    /// nowhere.
    #[must_use]
    pub fn showing_row(mut self, row: usize) -> Self {
        if row >= self.how_many {
            return self;
        }
        if row < self.first_shown {
            self.first_shown = row;
        } else if !self.is_shown(row) {
            self.first_shown = row.saturating_sub(self.showing.saturating_sub(1));
        }
        self
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    //! The owner's second clause, one test each where a test can hold one.
    //!
    //! **Inline rather than in a file of its own**, which is this crate's habit
    //! and is load-bearing: `alo-choosing`'s i18n guard cuts a file at its first
    //! `#[cfg(test)]`, so an assertion message in an included file reads as
    //! English a person could meet. It is not — it is a sentence for whoever
    //! broke the test.

    use super::*;

    fn text(percent: u16) -> TextScale {
        TextScale::percent(percent).unwrap()
    }

    /// Room enough for anything, for the tests that are not about scrolling.
    fn plenty() -> Room {
        Room::pixels(10_000)
    }

    /// **A row is never below the floor, at any text size.**
    ///
    /// *Overflow rows have a minimum 44px hit-target height. Remove the 39px
    /// pitch; do not create an exception.* The floor is
    /// `alo_appearance::targets::ENHANCED_TARGET`, which every other control in
    /// alo OS is built to, and the design file's own rows were 39.
    ///
    /// Walked over the whole text range rather than at a size somebody thought
    /// to try, because the fault this forbids is a row that is tall enough at
    /// 100% and not at some size in between.
    #[test]
    fn no_text_size_makes_a_row_shorter_than_a_control_may_be() {
        let (smallest, largest) = TextScale::range();
        for percent in smallest..=largest {
            let row = a_row_at(text(percent)).as_pixels();
            assert!(
                row >= ENHANCED_TARGET,
                "at {percent}% a row is {row} and a control is built to {ENHANCED_TARGET}"
            );
            assert_ne!(
                row, 39,
                "at {percent}% a row is the design file's 39, which the owner removed"
            );
        }
    }

    /// **A row grows with the text**, which the floor alone would not give.
    ///
    /// *Allow rows to grow for larger text.* A row pinned at 44 would clip its
    /// name at 200% — the size EN 301 549 requires a layout to survive — so this
    /// asserts the growth and names the size where it starts.
    #[test]
    fn a_row_grows_once_the_text_needs_more_than_the_floor() {
        let at_ordinary = a_row_at(text(100));
        let at_the_standards_size = a_row_at(text(200));
        assert!(
            at_the_standards_size > at_ordinary,
            "a row is {} at 100% and {} at 200%, so a name is drawn in a box that did not grow \
             for it",
            at_ordinary.as_pixels(),
            at_the_standards_size.as_pixels()
        );

        // And it never shrinks on the way, which a `max` of two growing
        // measurements could still get wrong if either did.
        let (smallest, largest) = TextScale::range();
        let mut before = Room::pixels(0);
        for percent in smallest..=largest {
            let now = a_row_at(text(percent));
            assert!(now >= before, "a row got shorter at {percent}%");
            before = now;
        }
    }

    /// **The panel is as tall as its contents, with no ceiling of its own.**
    ///
    /// *Let the panel grow beyond 313px when space permits.* The 313 was six
    /// rows at the pitch the owner removed; there is no height constant in this
    /// crate and this is the test that says so.
    #[test]
    fn the_panel_grows_past_the_three_hundred_and_thirteen_it_was_drawn_at() {
        let many = ThePanel::of(20, text(100), plenty());
        assert!(!many.scrolls(), "the premise: there is room for all twenty");
        assert!(
            many.height().as_pixels() > 313,
            "twenty rows came to {}, so something is still capping the panel at the drawn height",
            many.height().as_pixels()
        );
        assert_eq!(
            many.height().as_pixels(),
            many.above_the_rows().as_pixels() + 20 * many.a_row().as_pixels(),
            "the panel is not the heading and its rows"
        );

        // A short list is short: the panel follows its contents downward too,
        // rather than reserving room for rows that are not there.
        let few = ThePanel::of(3, text(100), plenty());
        assert!(few.height() < many.height());
    }

    /// **When the room runs out the list scrolls, and nothing is dropped.**
    ///
    /// *…then scroll its list within the available screen space.* The panel
    /// stays inside the room it was given, shows as many rows as fit, and still
    /// knows about every row there is.
    #[test]
    fn a_panel_with_too_little_room_scrolls_rather_than_overflowing_the_screen() {
        let room = Room::pixels(300);
        let panel = ThePanel::of(40, text(100), room);

        assert!(panel.scrolls(), "forty rows in 300 pixels did not scroll");
        assert!(
            panel.height().as_pixels() <= room.as_pixels(),
            "the panel is {} tall in {} of room",
            panel.height().as_pixels(),
            room.as_pixels()
        );
        assert_eq!(
            panel.how_many(),
            40,
            "rows were dropped rather than scrolled"
        );

        let (first, showing) = panel.showing();
        assert_eq!(first, 0);
        assert!(showing > 0 && showing < 40);

        // Every shown row is inside the panel, and the ones past them are not
        // given a place at all.
        for row in 0..40 {
            match panel.where_a_row_sits(row) {
                Some(down) => assert!(
                    down.as_pixels() + panel.a_row().as_pixels() <= panel.height().as_pixels(),
                    "row {row} sits at {} in a panel {} tall",
                    down.as_pixels(),
                    panel.height().as_pixels()
                ),
                None => assert!(!panel.is_shown(row)),
            }
        }
    }

    /// **Keyboard focus scrolls the focused row into view**, in both directions,
    /// and by the least it can.
    ///
    /// The owner's words, and the half that is easy to get wrong is *the least
    /// it can*: a list that jumped to put the focused row at its top would lose
    /// a person their place every time they arrowed past the bottom.
    #[test]
    fn focusing_a_row_brings_it_into_view_without_losing_the_place() {
        let panel = ThePanel::of(40, text(100), Room::pixels(300));
        let (_, showing) = panel.showing();
        assert!((2..40).contains(&showing), "the premise: {showing} shown");

        // Down past the bottom: the row is the last shown, not the first.
        let just_past = panel.showing_row(showing);
        assert!(just_past.is_shown(showing));
        assert_eq!(
            just_past.showing().0,
            1,
            "arrowing one past the bottom scrolled by more than one row"
        );
        assert!(
            just_past.is_shown(1),
            "the row above the focused one was scrolled away, so a person lost their place"
        );

        // Far down, then back up: both ends work and the second does not need
        // the first.
        let far = panel.showing_row(39);
        assert!(far.is_shown(39));
        assert_eq!(
            far.showing().0,
            40 - showing,
            "the last page is not aligned"
        );
        let back = far.showing_row(0);
        assert!(back.is_shown(0));
        assert_eq!(back.showing().0, 0);

        // A row already on the screen moves nothing.
        assert_eq!(panel.showing_row(0), panel);

        // A row that does not exist moves nothing either, rather than scrolling
        // to a place with no row in it.
        assert_eq!(panel.showing_row(40), panel);
        assert_eq!(panel.showing_row(usize::MAX), panel);
    }

    /// **A panel with room for everything does not scroll**, which is the
    /// premise the scrolling tests rest on.
    #[test]
    fn a_panel_with_room_for_everything_shows_everything() {
        let panel = ThePanel::of(6, text(100), plenty());
        assert!(!panel.scrolls());
        assert_eq!(panel.showing(), (0, 6));
        for row in 0..6 {
            assert!(panel.is_shown(row));
            assert!(panel.where_a_row_sits(row).is_some());
        }
        assert_eq!(panel.showing_row(5), panel, "nothing to scroll");
    }

    /// **A panel on a screen with almost no room still shows one row.**
    ///
    /// A control a person can press to reach nothing is worse than a panel that
    /// is too tall for its box, and a display that small is `crate::Screen`'s
    /// problem rather than this file's.
    #[test]
    fn a_panel_with_no_room_at_all_still_shows_one_row() {
        for room in [0_u32, 1, 10, 44] {
            let panel = ThePanel::of(9, text(100), Room::pixels(room));
            assert_eq!(panel.showing().1, 1, "with {room} of room");
            assert!(panel.is_shown(0));
            assert!(panel.scrolls());
        }
        // And a list with no rows in it asks for none, rather than one that is
        // not there.
        let empty = ThePanel::of(0, text(100), plenty());
        assert!(!empty.is_shown(0));
        assert!(empty.where_a_row_sits(0).is_none());
        assert!(!empty.scrolls());
    }

    /// **The width is the measured one and does not move with the text.**
    ///
    /// 248 on all four edges, which is what keeps it a number rather than a
    /// proportion. A name that is too long for it is wrapped or trimmed by
    /// whoever can measure a font; it does not widen the panel, or the list
    /// would be a different width on every machine.
    #[test]
    fn the_panel_is_the_measured_width_at_every_text_size() {
        for percent in [75, 100, 200, 300] {
            let panel = ThePanel::of(6, text(percent), plenty());
            assert_eq!(panel.width().as_pixels(), AN_OVERFLOW_PANEL_IS_WIDE);
            assert_eq!(panel.width().as_pixels(), 248);
        }
    }
}
