//! **When alo's task window closes, the keyboard goes to the window the person
//! sees in front.**
//!
//! The owner's decision of 2026-11-17, in these words: *"When alo's task window
//! closes, the keyboard goes to the window the person sees in front. alo still
//! never takes the keyboard while the person is working elsewhere."* It
//! settles a meeting of two promises found on a walked machine (run
//! 43990841766): a task opened Papers, the person closed the finished task,
//! Papers was the window they saw — and their typing stayed wherever it had
//! been before the task, so Alt+F4 there closed nothing.
//!
//! # What changes, and what does not
//!
//! - **Closing** — or sending aside — the last task window that held the
//!   keyboard, with the applications in front, gives the seat's keyboard to the
//!   application window in front on the Place being looked at: the same window
//!   *Fit the window in front* frames
//!   (`crate::canvas_fit_the_window_in_front`), so the two cannot disagree
//!   about which window that is. Already the owner's rule for task windows
//!   (*like every desktop, the window you can see in front gets your typing*,
//!   2026-10-30); now for an application too.
//! - **With the terminal in front**, nothing more is needed: it takes keys by
//!   being in front, and has no surface to focus.
//! - **A window alo opens still takes nothing.**
//!   `docs/design/where-a-new-window-opens.md`'s rule 8 — *alo must not steal
//!   focus* — is untouched (`crate::a_window_the_person_opened`): this runs
//!   only when the person closes the window they were in.
//! - **A hidden window never gets it**: only a window that is drawn is a
//!   window in front.

use crate::which_window_is_in_front::AKindOfWindow;

impl crate::Server {
    /// The person's task window has gone, and no task window is left to hold
    /// the keyboard: give it to the application window they see in front, if
    /// the applications are what is in front.
    pub(crate) fn the_keyboard_goes_to_the_window_in_front(&mut self) {
        if self.the_kind_in_front() != AKindOfWindow::Applications {
            return;
        }
        let place = self.the_place_now();
        let Some(front) = self
            .mapped_surfaces()
            .find(|surface| crate::canvas_place::the_place_of(surface) == Some(place))
            .cloned()
        else {
            return;
        };
        // A refusal leaves the keyboard where it was, which is what happened
        // before this file: nothing worse than the old behaviour.
        let _ = self.keyboard_focus(Some(&front));
    }
}
