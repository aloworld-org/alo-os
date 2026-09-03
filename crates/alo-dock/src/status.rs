//! The status area: which end of the dock it is at, and which way it runs.
//!
//! `docs/features.md` puts the clock, the battery, the network and — at v0.5 —
//! the egress indicator *at the far end of the dock, wherever the dock is*. What
//! goes in it is that release's; what *the far end* means is this one's, because
//! it is the other half of the v0.01 promise that a dock reflows rather than
//! being a horizontal bar somebody turned sideways.
//!
//! # The far end is a fact about reading, not about the screen
//!
//! A dock running across the screen is a row, and the far end of a row is the
//! end you reach last. In every official EU language that is the right; in
//! Arabic, Hebrew or Persian it is the left. `alo-strings` already knows which
//! way a language is read and `docs/features.md` promises the shell is
//! right-to-left ready *so that adding a language later is translation rather
//! than rework* — so the status area asks, now, while nothing needs the answer.
//!
//! A dock running down the screen is a column, and a column is read downwards in
//! every script alo OS ships or is likely to be given. So its far end is the
//! bottom, in both directions, and that asymmetry is deliberate rather than an
//! oversight: mirroring a column because the language mirrors a row would put
//! the clock above the applications for readers who do not expect it there.
//!
//! **The dock's own edge does not move with the reading.** *Left* in
//! [`crate::Edge`] is the physical left of the screen for everybody. Where a
//! person put their dock is furniture; which end of it they reach last is
//! reading.

use alo_strings::Direction;

use crate::along::Along;

/// One end of the dock.
///
/// All four exist because they are the ends a dock can have; only three of them
/// can ever be a *far* end, and [`StatusArea`]'s tests are what say which.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum End {
    /// The left-hand end of a dock that runs across the screen.
    Left,
    /// The right-hand end of a dock that runs across the screen.
    Right,
    /// The upper end of a dock that runs down the screen.
    Top,
    /// The lower end of a dock that runs down the screen.
    Bottom,
}

impl End {
    /// The end at the other end.
    #[must_use]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
        }
    }
}

/// Where the status area sits along the dock, and which way its own contents
/// run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StatusArea {
    /// Which way its contents run, which is the dock's own direction.
    runs: Along,
    /// Which end of the dock it is at.
    at: End,
}

impl StatusArea {
    /// Where the status area is on a dock running this way, read by somebody
    /// reading this way.
    #[must_use]
    pub const fn of(along: Along, reading: Direction) -> Self {
        let at = match (along, reading) {
            (Along::Across, Direction::LeftToRight) => End::Right,
            (Along::Across, Direction::RightToLeft) => End::Left,
            (Along::Down, _) => End::Bottom,
        };
        Self { runs: along, at }
    }

    /// Which way its contents run.
    ///
    /// The dock's own direction, which is the whole of *the status area
    /// reflows*: a dock down the side of the screen stacks its status items
    /// rather than keeping a row of them and turning the row.
    #[must_use]
    pub const fn runs(self) -> Along {
        self.runs
    }

    /// Which end of the dock it is at.
    #[must_use]
    pub const fn at(self) -> End {
        self.at
    }

    /// Which end the applications start from, which is the other one.
    #[must_use]
    pub const fn applications_start_at(self) -> End {
        self.at.opposite()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The status area runs the way the dock runs.** This is the reflow: a
    /// dock down the side of the screen has a column of status items, not a row
    /// of them rotated.
    #[test]
    fn the_status_area_runs_the_way_the_dock_does() {
        for reading in [Direction::LeftToRight, Direction::RightToLeft] {
            for along in [Along::Across, Along::Down] {
                assert_eq!(StatusArea::of(along, reading).runs(), along);
            }
        }
    }

    /// **A row's far end follows the reading.** Somebody reading Arabic reaches
    /// the left of a row last, so that is where the clock belongs — asked now,
    /// while nothing in the Union needs it, which is the point of the promise.
    #[test]
    fn the_far_end_of_a_row_is_the_end_the_reader_reaches_last() {
        assert_eq!(
            StatusArea::of(Along::Across, Direction::LeftToRight).at(),
            End::Right
        );
        assert_eq!(
            StatusArea::of(Along::Across, Direction::RightToLeft).at(),
            End::Left
        );
    }

    /// **A column is read downwards whichever way the language is read**, so a
    /// vertical dock's far end is its bottom in both directions. Mirroring it
    /// would put the clock above the applications for readers who do not expect
    /// it there.
    #[test]
    fn a_column_does_not_turn_over_when_the_reading_does() {
        for reading in [Direction::LeftToRight, Direction::RightToLeft] {
            let status = StatusArea::of(Along::Down, reading);
            assert_eq!(status.at(), End::Bottom, "{reading:?}");
            assert_eq!(status.applications_start_at(), End::Top, "{reading:?}");
        }
    }

    /// The applications start from the end the status area is not at, whatever
    /// that end is — so nothing is laid out from an end that is already taken.
    #[test]
    fn the_applications_start_from_the_other_end() {
        for reading in [Direction::LeftToRight, Direction::RightToLeft] {
            for along in [Along::Across, Along::Down] {
                let status = StatusArea::of(along, reading);
                assert_ne!(status.applications_start_at(), status.at());
                assert_eq!(status.applications_start_at().opposite(), status.at());
            }
        }
    }

    /// The top is never a far end: a row's far end is one of its sides, and a
    /// column's is its bottom.
    #[test]
    fn no_dock_puts_its_status_area_at_the_top() {
        for reading in [Direction::LeftToRight, Direction::RightToLeft] {
            for along in [Along::Across, Along::Down] {
                assert_ne!(StatusArea::of(along, reading).at(), End::Top);
            }
        }
    }
}
