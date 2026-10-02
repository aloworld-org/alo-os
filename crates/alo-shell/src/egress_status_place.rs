//! Where the egress indicator sits: at the far end of the dock, above it.
//!
//! `docs/features.md` puts the indicator in the status area *so "nothing has
//! left this machine" sits where a person already glances*. The dock is along the
//! bottom edge ([ADR
//! 0076](../../../docs/decisions/0076-the-dock-is-fixed-to-the-bottom-edge-and-answers-one-question.md)),
//! and how thick it is is `alo_dock::Layout::thickness`; this file turns that
//! into pixels on one output.
//!
//! The lines are stacked upwards from that corner, clear of the dock. The first
//! line stays nearest the dock and a new one is added above it, so nothing
//! already on the screen moves when something else starts leaving.
//!
//! # What ADR 0076 changed here, and what it did not
//!
//! **Not the position.** A dock along the bottom of an English machine put the
//! indicator at the bottom right before that record and puts it there after it.
//!
//! What changed is where the answer comes from. The far end used to be
//! `alo_dock::StatusArea`'s, and the status area is not the Dock's any more —
//! a clock is not something a person opens. The far end of a *row* was always a
//! question about the way the row is read, so this file asks the reading
//! direction, and the three branches for a dock on the other three edges went
//! with the edges.
//!
//! **The rest of the status area — the clock, the battery, the network — is owed
//! a location and does not have one.** That is `docs/autonomy/evidence-a-person-can-work-on-it-all-day.md`'s
//! entry to carry, not this file's to invent. The egress indicator is not waiting
//! on it, because it has a corner of its own and always did.

use alo_dock::Layout;
use alo_strings::Direction;

/// Which way a row is aligned across the output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Across {
    /// Rows start at this x and run right.
    FromLeft(i32),
    /// Rows end at this x.
    FromRight(i32),
}

/// Which way rows are stacked down the output.
///
/// **One way, since ADR 0076.** `Downwards` was for a dock along the top of the
/// screen, and there is no such dock. It stays an enum rather than becoming a
/// bare `i32` because *which way do these stack* is still the question the type
/// answers, and a second answer is what a surface somewhere other than the
/// bottom would need.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Stacked {
    /// The first row's bottom is at this y, and the next is above it.
    Upwards(i32),
}

/// The corner the indicator grows from, and how much room it has.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Place {
    /// Where rows are aligned across.
    pub(crate) across: Across,
    /// Where rows are stacked from.
    pub(crate) stacked: Stacked,
    /// The widest a row may be.
    pub(crate) room_across: i32,
    /// The tallest the whole stack may be.
    pub(crate) room_down: i32,
}

impl Place {
    /// The same place, with `taken` pixels already used at the corner.
    ///
    /// **What is in use has the corner and what is leaving stacks beyond it.**
    /// ADR 0010 makes the in-use indicator's position one of the three things a
    /// person is given besides a colour, and the reason is that position costs
    /// nothing to learn — which only holds while that stack's origin does not
    /// move. So it keeps the fixed corner and this moves the other one along.
    ///
    /// The egress indicator's own rule is untouched: its first line stays
    /// nearest its origin and a new one is added beyond, so nothing already on
    /// the screen moves when something else starts leaving.
    pub(crate) fn beyond(self, taken: i32) -> Self {
        if taken <= 0 {
            return self;
        }
        Self {
            stacked: match self.stacked {
                Stacked::Upwards(y) => Stacked::Upwards(y - taken),
            },
            room_down: (self.room_down - taken).max(0),
            ..self
        }
    }

    /// The **other** end of the same dock, for a surface that must not cover
    /// the status area.
    ///
    /// Notifications go here. The two indicators own the status corner and are
    /// permanent; a notification is a message that arrives and goes, and one
    /// that covered *what is leaving this machine* or *your camera is on* would
    /// be trading a promise for a convenience. Same edge, same margins, other
    /// end — so a person's eyes learn one side for what is true now and the
    /// other for what just happened.
    pub(crate) fn of_the_other_end(
        layout: Layout,
        size: (i32, i32),
        margin: i32,
        reading: Direction,
    ) -> Self {
        let corner = Self::of(layout, size, margin, reading);
        let (width, height) = size;
        Self {
            across: match corner.across {
                Across::FromLeft(_) => Across::FromRight(width - margin),
                Across::FromRight(_) => Across::FromLeft(margin),
            },
            stacked: match corner.stacked {
                Stacked::Upwards(y) => Stacked::Upwards(y.min(height - margin)),
            },
            ..corner
        }
    }

    /// The indicator's corner on an output of `size`, for a dock laid out as
    /// `layout` and read `reading`, keeping `margin` pixels from the dock and
    /// the output's edges.
    ///
    /// **The far end of the dock is the corner a person reads last**, which is
    /// the right in a language read left to right and the left in one read the
    /// other way. That used to be `alo_dock::StatusArea`'s answer; ADR 0076 took
    /// the status area off the Dock, and the answer moved here rather than
    /// changing — a dock along the bottom in English put the indicator at the
    /// bottom right before that record and puts it there after it.
    ///
    /// It is asked of the reading direction rather than of the Dock because that
    /// is what the question was always about: the far end of a *row* follows the
    /// way the row is read, and nothing about where the dock sits is involved now
    /// that it sits in one place.
    pub(crate) fn of(layout: Layout, size: (i32, i32), margin: i32, reading: Direction) -> Self {
        let (width, height) = size;
        let thickness = i32::try_from(layout.thickness().as_pixels())
            .unwrap_or(i32::MAX)
            .clamp(0, width.min(height));
        let across = match reading {
            Direction::RightToLeft => Across::FromLeft(margin),
            Direction::LeftToRight => Across::FromRight(width - margin),
        };
        // Clear of the bar, which floats above the bottom edge rather than
        // sitting on it — so the first line starts above the gap as well as
        // above the bar itself.
        let floating =
            i32::try_from(alo_dock::measures::FLOATING_ABOVE_THE_EDGE).unwrap_or(i32::MAX);
        Self {
            across,
            stacked: Stacked::Upwards(height - thickness - floating - margin),
            room_across: width - 2 * margin,
            room_down: height - thickness - floating - 2 * margin,
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_appearance::TextScale;
    use alo_dock::{Dock, Screen};

    /// The dock on a 1920×1080 screen.
    fn laid_out() -> Layout {
        Dock::shipped().layout_on(Screen::of(1920, 1080).unwrap(), TextScale::ordinary())
    }

    /// **The far end of the dock is the corner a person reads last**, and the
    /// lines grow up from it, clear of the dock.
    ///
    /// The exact numbers are what this asserted before ADR 0076 for a dock along
    /// the bottom, unchanged — which is the point of keeping them: the record
    /// moved where the answer comes from and was not supposed to move the
    /// indicator, and a test that had been rewritten alongside could not say so.
    #[test]
    fn it_sits_at_the_end_read_last_and_grows_upwards_clear_of_the_dock() {
        let layout = laid_out();
        let thick = i32::try_from(layout.thickness().as_pixels()).unwrap();

        let place = Place::of(layout, (1920, 1080), 8, Direction::LeftToRight);
        assert_eq!(place.across, Across::FromRight(1912));
        let floating = i32::try_from(alo_dock::measures::FLOATING_ABOVE_THE_EDGE).unwrap();
        assert_eq!(place.stacked, Stacked::Upwards(1080 - thick - floating - 8));

        let mirrored = Place::of(layout, (1920, 1080), 8, Direction::RightToLeft);
        assert_eq!(mirrored.across, Across::FromLeft(8));
        assert_eq!(
            mirrored.stacked, place.stacked,
            "which way a person reads moves it across, never up or down"
        );
    }

    /// The room left over never includes the dock.
    #[test]
    fn the_room_leaves_the_dock_out() {
        for reading in [Direction::LeftToRight, Direction::RightToLeft] {
            let layout = laid_out();
            let thick = i32::try_from(layout.thickness().as_pixels()).unwrap();
            let place = Place::of(layout, (1920, 1080), 8, reading);
            let floating = i32::try_from(alo_dock::measures::FLOATING_ABOVE_THE_EDGE).unwrap();
            assert!(
                place.room_down <= 1080 - thick - floating - 16,
                "{reading:?}"
            );
            assert_eq!(place.room_across, 1920 - 16);
        }
    }

    /// **Notifications go to the other end of the same edge**, so one never
    /// covers *what is leaving this machine*.
    #[test]
    fn the_other_end_is_the_other_end() {
        let layout = laid_out();
        for reading in [Direction::LeftToRight, Direction::RightToLeft] {
            let corner = Place::of(layout, (1920, 1080), 8, reading);
            let other = Place::of_the_other_end(layout, (1920, 1080), 8, reading);
            assert_ne!(corner.across, other.across, "{reading:?}");
        }
    }
}
