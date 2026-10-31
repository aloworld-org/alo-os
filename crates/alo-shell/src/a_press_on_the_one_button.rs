//! **A press on a task window's one button** — the slot that is Send, or Stop.
//!
//! The button was drawn and nothing could press it (#990). Stop is the reason
//! it is pressable now
//! ([ADR 0139](../../../docs/decisions/0139-stop-reaches-alo-and-cancels-what-it-sent.md)):
//! a press while it shows **Stop** stops the task; a press while it shows
//! anything else is the window's, as a press anywhere else in it is.
//!
//! # One place says where the button is
//!
//! [`the_one_buttons_place`] is what `crate::task_raster` draws the circle at
//! and what a press is tested against, so the button a person sees and the one
//! they can press cannot come apart.

use alo_tasks::TheOneButton;
use alo_tasks::layout::TheRows;
use alo_tasks::measures as m;
use smithay::utils::{Physical, Point, Rectangle};

use crate::display_scale::DisplayScale;

/// **Where a task window's one button is**, for a window at `at` on a display
/// drawn at `scale`, in the same pixels the window is drawn in; [`None`] for a
/// window too small to lay out.
///
/// **Laid out in logical units and drawn in the display's pixels**, as
/// `crate::task_raster` lays out the rest of the window. Until 2026-10-30 this
/// read the window's pixels as logical units, so on a display at 200 per cent
/// the circle was drawn half its size and in the wrong place, and a press was
/// tested against that wrong place.
pub(crate) fn the_one_buttons_place(
    at: Rectangle<i32, Physical>,
    scale: DisplayScale,
) -> Option<Rectangle<i32, Physical>> {
    let px = |of: u32| scale.px(of);
    let controls = TheRows::of(scale.logical(at.size.w), scale.logical(at.size.h))
        .ok()?
        .controls();
    Some(Rectangle::new(
        (
            at.loc.x + px(m::BORDER) + px(controls.the_one_button),
            at.loc.y + px(m::BORDER) + px(controls.from_the_top),
        )
            .into(),
        (px(m::A_CONTROL_IS), px(m::A_CONTROL_IS)).into(),
    ))
}

impl crate::Server {
    /// A press at `at` on the screen, offered to the task windows' one
    /// buttons, newest first. Answers whether one showing **Stop** took it.
    pub(crate) fn a_press_on_a_one_button(&mut self, at: Point<i32, Physical>) -> bool {
        let open = self.tasks().each_open_window();
        // **The scale the windows were last drawn at**, the same answer a
        // press on a task window's edge is measured with.
        let scale = DisplayScale::of_per_cent(self.the_scale_edges_are_measured_at);
        for (which, window) in open.into_iter().rev() {
            let on_it =
                the_one_buttons_place(window, scale).is_some_and(|button| button.contains(at));
            if !on_it {
                continue;
            }
            let shows_stop = self
                .tasks()
                .the(which)
                .is_some_and(|task| task.the_one_button() == TheOneButton::Stop);
            return shows_stop && self.the_person_stops_a_task(which);
        }
        false
    }
}
