//! **The wheel over a first-start list moves through it**, a row a notch.
//!
//! On a screen too short for a whole list a sheet shows a window of it, and the
//! window follows the cursor (`crate::a_window_onto_a_list`). The arrow keys
//! already move the cursor; this is the wheel doing the same, over the rows, so
//! a person with a mouse and no habit of the keyboard reaches every language,
//! region, keyboard and time zone.
//!
//! # The pattern the task window set
//!
//! The design says the content scrolls and does not say how far a notch goes,
//! so this takes the task window's conversation (`crate::a_task_window_scrolls`,
//! #985): **a notch is 15 of the units a frame carries**, and the wheel pushed
//! away goes back up the list. What a notch moves here is a row rather than
//! pixels, because the list is drawn a row at a time. A touchpad's small frames
//! are added up rather than each rounded to nothing, so a slow two-finger
//! scroll still arrives.

use smithay::input::pointer::AxisFrame;

/// A wheel notch, in the units an [`AxisFrame`] carries here — the figure
/// `crate::a_task_window_scrolls` and `crate::canvas_wheel_zoom` use.
const A_NOTCH: f64 = 15.0;

/// The most rows one frame may move, so a frame carrying nonsense moves a
/// list to one of its ends at worst and never loops for long.
const AT_MOST_IN_ONE_FRAME: i32 = 64;

/// What the wheel has turned that has not yet made a whole row.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(crate) struct TheWheelOnASheet {
    /// Part of a notch, carried to the next frame.
    carried: f64,
}

impl TheWheelOnASheet {
    /// **How many rows this frame moves**: positive down the list, negative
    /// back up it. A frame that is not a number moves nothing and forgets
    /// nothing.
    pub(crate) fn turned(&mut self, frame: &AxisFrame) -> i32 {
        let turned = self.carried + frame.axis.1 / A_NOTCH;
        if !turned.is_finite() {
            return 0;
        }
        let whole = turned.trunc();
        self.carried = turned - whole;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "clamped to a small range first, so the cast cannot truncate"
        )]
        let rows = whole.clamp(
            f64::from(-AT_MOST_IN_ONE_FRAME),
            f64::from(AT_MOST_IN_ONE_FRAME),
        ) as i32;
        rows
    }
}

#[cfg(test)]
mod tests {
    use smithay::backend::input::{Axis, AxisSource};

    use super::*;

    /// A scroll frame carrying this much vertical movement.
    fn a_scroll_of(down: f64) -> AxisFrame {
        AxisFrame::new(0)
            .source(AxisSource::Wheel)
            .value(Axis::Vertical, down)
    }

    /// **A notch down is a row down; a notch up is a row up.**
    #[test]
    fn a_notch_is_a_row() {
        let mut wheel = TheWheelOnASheet::default();
        assert_eq!(wheel.turned(&a_scroll_of(15.0)), 1);
        assert_eq!(wheel.turned(&a_scroll_of(-30.0)), -2);
    }

    /// **Small frames add up** rather than each rounding to nothing.
    #[test]
    fn small_frames_add_up() {
        let mut wheel = TheWheelOnASheet::default();
        let moved: i32 = (0..6).map(|_| wheel.turned(&a_scroll_of(5.0))).sum();
        assert_eq!(moved, 2);
    }

    /// **A frame that is not a number moves nothing**, and a huge one is held
    /// to a bound.
    #[test]
    fn nonsense_moves_nothing_and_a_huge_frame_is_bounded() {
        let mut wheel = TheWheelOnASheet::default();
        assert_eq!(wheel.turned(&a_scroll_of(f64::NAN)), 0);
        assert_eq!(wheel.turned(&a_scroll_of(1.0e12)), AT_MOST_IN_ONE_FRAME);
    }
}
