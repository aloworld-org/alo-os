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
//!
//! # Why the panel's column is an argument and not a combinator
//!
//! The corner this file answers for is at the far end of the output, and the
//! put-aside panel reserves a column at the same end. Until 2026-10-04 this file
//! measured from the output's own width, so on a real draw the indicator's band and
//! the panel's column **shared 104 pixels** — recorded as reconciliation 3 in
//! [ADR 0086](../../../docs/decisions/0086-the-complete-canvas-is-one-current-milestone.md)
//! and measured by `crate::direct_desktop_tests`.
//!
//! The owner's ruling of 2026-09-30, quoted in full in
//! `crate::top_controls_region`, settles the same collision for three surfaces —
//! *Top controls: span the screen up to the reserved right-panel area*, *Right panel:
//! owns that area*, *Bottom Dock: stops before the same area* — and gives the reason:
//! **so one pointer position cannot reveal two surfaces.** The status corner is the
//! fourth surface and the ruling does not name it, because it joined the fixed-control
//! set on 2026-10-02, after the ruling. Applying the named rule to it is not inventing
//! a position: it is the only reading under which the ruling's own clause is true of a
//! real screen.
//!
//! **It is a required parameter rather than a `clear_of(panel)` to call afterwards**,
//! and that is the whole design of it. A combinator is a thing a caller can forget,
//! and this session has catalogued eight pieces of correct, tested, unreachable
//! canvas code — four of them found by deleting the wiring and watching nothing
//! fail. A parameter cannot be forgotten: the two rasters that place a surface at
//! this corner both had to answer for the panel to keep compiling.
//!
//! **And it moved two surfaces, not one.** `crate::in_use_raster` places *your camera
//! is on* at this same corner through this same constructor, and it was under the
//! panel for exactly the same reason. It is not in `FixedControlsDrawn`'s set at all,
//! so nothing would have measured it.

use alo_dock::Layout;
use alo_strings::Direction;
use smithay::utils::{Physical, Rectangle};

/// The span across the output a corner surface may use, given the panel's column.
///
/// Returned as `(from, to)` in output pixels, half-open on the right, before any
/// margin is taken off either end.
///
/// **Which side the column is on is measured rather than assumed.** The design puts
/// the put-aside panel at the right on every frame, and `Direction::RightToLeft` is
/// the case where assuming it would be wrong without anything saying so — a mirrored
/// session with a mirrored panel would put the column exactly where this file's
/// `FromLeft` rows start. So the column's own `loc.x` decides, and both answers are
/// reachable.
///
/// A column of no extent is no column: an empty panel reserves a rectangle with no
/// width, and the whole output is the room. That is the same treatment
/// `crate::top_controls_region::reserved` gives it, for the same reason — a rectangle
/// of no extent claims no point while still reading as a surface.
fn the_span_across(width: i32, panel: Option<Rectangle<i32, Physical>>) -> (i32, i32) {
    let Some(panel) = panel.filter(|column| column.size.w > 0 && column.size.h > 0) else {
        return (0, width);
    };
    let left = panel.loc.x;
    let right = left.saturating_add(panel.size.w);
    if left <= 0 {
        // Against the left edge, so the room is everything to the right of it.
        (right.clamp(0, width), width)
    } else {
        (0, left.clamp(0, width))
    }
}

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
        panel: Option<Rectangle<i32, Physical>>,
    ) -> Self {
        let corner = Self::of(layout, size, margin, reading, panel);
        let (width, height) = size;
        // The other end of the **room**, not of the output. In a left-to-right
        // session this end is away from the panel and the two are the same number;
        // in a mirrored one it is the end the column is at, which is the case that
        // would have been wrong silently.
        let (from, to) = the_span_across(width, panel);
        Self {
            across: match corner.across {
                Across::FromLeft(_) => Across::FromRight(to - margin),
                Across::FromRight(_) => Across::FromLeft(from + margin),
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
    /// `panel` is the column the put-aside panel reserved, or [`None`] when none is
    /// reserved — the corner stops before it, so the two cannot overlap by
    /// construction rather than by two numbers agreeing. See this file's own note for
    /// why it is a parameter.
    pub(crate) fn of(
        layout: Layout,
        size: (i32, i32),
        margin: i32,
        reading: Direction,
        panel: Option<Rectangle<i32, Physical>>,
    ) -> Self {
        let (width, height) = size;
        let thickness = i32::try_from(layout.thickness().as_pixels())
            .unwrap_or(i32::MAX)
            .clamp(0, width.min(height));
        // **The Dock's thickness is still asked of the whole output.** Narrowing the
        // room is not the same as narrowing the screen: the Dock lays out on the
        // output it is on, and handing it a width short by the panel's column would
        // change how many icons it fits. Only the across-numbers below move.
        let (from, to) = the_span_across(width, panel);
        let across = match reading {
            Direction::RightToLeft => Across::FromLeft(from + margin),
            Direction::LeftToRight => Across::FromRight(to - margin),
        };
        // Clear of the bar, which floats above the bottom edge rather than
        // sitting on it — so the first line starts above the gap as well as
        // above the bar itself.
        let floating =
            i32::try_from(alo_dock::measures::FLOATING_ABOVE_THE_EDGE).unwrap_or(i32::MAX);
        Self {
            across,
            stacked: Stacked::Upwards(height - thickness - floating - margin),
            room_across: (to - from - 2 * margin).max(0),
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

        let place = Place::of(layout, (1920, 1080), 8, Direction::LeftToRight, None);
        assert_eq!(place.across, Across::FromRight(1912));
        let floating = i32::try_from(alo_dock::measures::FLOATING_ABOVE_THE_EDGE).unwrap();
        assert_eq!(place.stacked, Stacked::Upwards(1080 - thick - floating - 8));

        let mirrored = Place::of(layout, (1920, 1080), 8, Direction::RightToLeft, None);
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
            let place = Place::of(layout, (1920, 1080), 8, reading, None);
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
            let corner = Place::of(layout, (1920, 1080), 8, reading, None);
            let other = Place::of_the_other_end(layout, (1920, 1080), 8, reading, None);
            assert_ne!(corner.across, other.across, "{reading:?}");
        }
    }

    /// The column the put-aside panel reserved on a 1280-wide output, matching the
    /// draw `crate::direct_desktop_tests` measured: `x=1168`, 112 wide, full height.
    fn the_panel_column() -> Rectangle<i32, Physical> {
        Rectangle::new(
            smithay::utils::Point::from((1168, 0)),
            smithay::utils::Size::from((112, 720)),
        )
    }

    /// **The corner stops before the panel's column, with the margin kept.**
    ///
    /// The numbers are the ones from the failing draw rather than round ones: the
    /// indicator's band was at `x=905 … 1272` and the column starts at `1168`, so
    /// 104 pixels belonged to two surfaces. A row ending at `1168 - 8` cannot reach
    /// the column whatever it is as wide as, which is the construction the owner's
    /// ruling asks for — not two numbers that happen to agree.
    #[test]
    fn the_corner_stops_before_the_panels_column() {
        let layout = laid_out();
        let panel = the_panel_column();

        let place = Place::of(layout, (1280, 720), 8, Direction::LeftToRight, Some(panel));
        assert_eq!(
            place.across,
            Across::FromRight(1168 - 8),
            "rows must end before the column at x=1168, not before the output at x=1280"
        );
        assert_eq!(
            place.room_across, 1152,
            "the widest row is the room beside the column less both margins"
        );
    }

    /// **Without a column the corner is where it always was**, so the parameter
    /// cannot have quietly moved the ordinary case.
    ///
    /// `None` and a column of no extent are the same answer, and both are reachable:
    /// an empty panel reserves a rectangle with no width on every frame where nothing
    /// is put aside, which is most of them.
    #[test]
    fn no_column_leaves_the_corner_where_it_was() {
        let layout = laid_out();
        let nothing_aside = Rectangle::new(
            smithay::utils::Point::from((1280, 0)),
            smithay::utils::Size::from((0, 720)),
        );
        for reading in [Direction::LeftToRight, Direction::RightToLeft] {
            let absent = Place::of(layout, (1280, 720), 8, reading, None);
            let empty = Place::of(layout, (1280, 720), 8, reading, Some(nothing_aside));
            assert_eq!(
                absent, empty,
                "a column of no extent is no column, {reading:?}"
            );
            assert_eq!(absent.room_across, 1280 - 16, "{reading:?}");
        }
    }

    /// **A mirrored session's column is stepped over rather than stepped under.**
    ///
    /// This is the branch that would have been wrong with no symptom. A right-to-left
    /// session puts the rows at the left, and if the panel is mirrored with them the
    /// column is at the left too — so *stop before the column* has to mean *start
    /// after it* on that side. Asserted because nothing on an English machine would
    /// ever reach it.
    #[test]
    fn a_column_at_the_left_pushes_the_rows_right_of_it() {
        let layout = laid_out();
        let mirrored_column = Rectangle::new(
            smithay::utils::Point::from((0, 0)),
            smithay::utils::Size::from((112, 720)),
        );
        let place = Place::of(
            layout,
            (1280, 720),
            8,
            Direction::RightToLeft,
            Some(mirrored_column),
        );
        assert_eq!(
            place.across,
            Across::FromLeft(112 + 8),
            "rows must start after the column, not at the output's own left edge"
        );
        assert_eq!(place.room_across, 1152);
    }

    /// **The other end follows the room too.**
    ///
    /// Notifications take the end away from the status corner, and on a mirrored
    /// session that end is the one the column is at. `of_the_other_end` derived its
    /// numbers from the output's width, so it had the same fault one surface over.
    #[test]
    fn the_other_end_also_stops_before_the_column() {
        let layout = laid_out();
        let panel = the_panel_column();
        let other = Place::of_the_other_end(
            layout,
            (1280, 720),
            8,
            Direction::RightToLeft,
            Some(panel),
        );
        assert_eq!(
            other.across,
            Across::FromRight(1168 - 8),
            "a notification at the far end must not sit under the panel either"
        );
    }
}
