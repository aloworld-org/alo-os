//! Scrolling a task window's conversation: by the wheel over it, by Page Up and
//! Page Down while it holds the keyboard, and the frame telling each task how
//! far it can go.
//!
//! *Conversation: scrolls on y, clipped on x* — `docs/design/the-ask-alo-task-window.md`.
//! Until 2026-10-28 nothing scrolled it. A conversation always opened at its
//! newest words, so an answer taller than the window showed its last lines and
//! nothing could bring its first ones into view — a walk on a booted machine
//! asked for four hundred words and could read the end of them only.
//!
//! **Where it is held is `alo_tasks::TheScroll`**, on the task, for the reason
//! the draft is held there: a person who scrolled back to read and put the
//! window aside finds it where they were reading.

use smithay::input::pointer::AxisFrame;
use smithay::utils::{Physical, Point};

use crate::the_tasks_a_person_opened::WhichTask;

/// Which way a page key moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum APage {
    /// Page Up: back toward the oldest words.
    Back,
    /// Page Down: toward the newest.
    Forward,
}

/// How far a scroll frame asks to move back, in the conversation's pixels.
///
/// **A notch is 15 of the units an `AxisFrame` carries here** —
/// `crate::canvas_wheel_zoom`'s figure and its reason — and a notch moves
/// `alo_tasks::measures::A_NOTCH_SCROLLS`. **Positive is back**: a wheel pushed
/// away goes up into what was said earlier, which is the direction every
/// document uses, and the axis is positive downwards.
fn how_far_back(frame: &AxisFrame) -> i32 {
    /// A wheel notch, in the units an [`AxisFrame`] carries here.
    const A_NOTCH: f64 = 15.0;
    let back = -frame.axis.1 / A_NOTCH * f64::from(alo_tasks::measures::A_NOTCH_SCROLLS);
    if !back.is_finite() {
        return 0;
    }
    #[expect(
        clippy::cast_possible_truncation,
        reason = "clamped to a small range first, so the cast cannot truncate"
    )]
    let whole = back.clamp(-1_000_000.0, 1_000_000.0).round() as i32;
    whole
}

impl crate::Server {
    /// A scroll over a task window, which scrolls its conversation.
    ///
    /// Answers [`None`] when the pointer is over no task window, so the scroll
    /// goes where it would have gone. **Over one, it never reaches whatever is
    /// underneath**, whether or not anything moved: a task window is drawn over
    /// the applications, and a scroll a person aimed at it is not a scroll of
    /// a document they cannot see.
    pub(crate) fn a_scroll_over_a_task_window(
        &mut self,
        at: Point<i32, Physical>,
        frame: &AxisFrame,
    ) -> Option<bool> {
        let which = self.the_task_window_under(at)?;
        Some(self.the_conversation_scrolled(which, how_far_back(frame)))
    }

    /// Page Up or Page Down in the task window holding the keyboard.
    ///
    /// Answers whether the view moved.
    pub(crate) fn a_page_in_the_task(&mut self, which: WhichTask, page: APage) -> bool {
        let Some(task) = self.tasks().the(which) else {
            return false;
        };
        let by = i32::try_from(task.scroll().a_page()).unwrap_or(i32::MAX);
        let back = match page {
            APage::Back => by,
            APage::Forward => -by,
        };
        self.the_conversation_scrolled(which, back)
    }

    /// Scroll this task's conversation `back`, and say whether it moved.
    fn the_conversation_scrolled(&mut self, which: WhichTask, back: i32) -> bool {
        let Some(task) = self.tasks_mut().the_mut(which) else {
            return false;
        };
        let turns = task.conversation().each().len();
        task.scroll_mut().scrolled(back, turns)
    }

    /// What this frame measured about each task window it drew: how far back
    /// its conversation can go, and how tall its view is.
    ///
    /// **Handed back after the frame**, because only a frame that shaped the
    /// words knows either — this crate's font is in the draw, and nothing
    /// before it can say how many lines an answer wrapped to.
    pub(crate) fn the_frame_measured_the_tasks(
        &mut self,
        drawn: &[crate::task_raster::TaskPicture],
    ) {
        for picture in drawn {
            let (the_most, the_view) = picture.the_conversation;
            if let Some(task) = self.tasks_mut().at_its_place(picture.place) {
                task.scroll_mut().the_frame_measured(the_most, the_view);
            }
        }
    }
}

#[cfg(test)]
#[path = "a_task_window_scrolls_tests.rs"]
mod tests;
