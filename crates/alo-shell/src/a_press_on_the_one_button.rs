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

/// A logical measure as a pixel count.
fn px(of: u32) -> i32 {
    i32::try_from(of).unwrap_or(i32::MAX)
}

/// **Where a task window's one button is**, for a window at `at`, in the
/// same pixels the window is drawn in; [`None`] for a window too small to lay
/// out.
pub(crate) fn the_one_buttons_place(
    at: Rectangle<i32, Physical>,
) -> Option<Rectangle<i32, Physical>> {
    let wide = u32::try_from(at.size.w.max(0)).unwrap_or(0);
    let tall = u32::try_from(at.size.h.max(0)).unwrap_or(0);
    let controls = TheRows::of(wide, tall).ok()?.controls();
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
        for (which, window) in open.into_iter().rev() {
            let on_it = the_one_buttons_place(window).is_some_and(|button| button.contains(at));
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
