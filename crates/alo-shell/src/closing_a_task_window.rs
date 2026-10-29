//! Closing a task window: which ones may close, and what closing does to the
//! keyboard and to the work.
//!
//! `docs/features.md` promises two things about it, and they are read whole:
//!
//! - **Closing a task whose work is over closes it.** Finished, or stopped and
//!   confirmed stopped — `alo_tasks::ATask::may_close_without_asking`.
//! - **Closing a running task asks**, with three named choices — *Stop and
//!   close*, *Keep running*, *Cancel* — and never kills it silently or leaves
//!   it running with no window and no entry in the panel.
//!
//! Three roads reach it, and all end in [`Server::the_person_closes_a_task`]:
//! Alt+F4 while the task window holds the keyboard, and Close on its edge
//! (`crate::a_task_windows_edge`), and a screen reader pressing that Close
//! (`crate::a_reader_presses`).
//!
//! # A running task asks: Stop and close, or Cancel
//!
//! Since 2026-10-29 (ADR 0139) closing a running task asks the person, on a
//! card at the end of its conversation (`crate::asking_before_closing`):
//! *Stop and close* stops alo and closes the window **once alo has really
//! stopped**, and *Cancel* closes nothing. *Keep running* waits for the
//! right-hand panel's place for a task and is not offered until then. A task
//! already stopping is not asked again; it closes once it has stopped.
//!
//! **Nothing here can approve anything**: this file never reaches a daemon,
//! and a change waiting when a person closes the window is neither approved
//! nor declined — under *Stop and close* it lapses with the turn.
//!
//! # The keyboard goes back to what was behind
//!
//! A task window holding the keyboard takes every key at the seat. Closing it
//! lets go, so the next key reaches whatever had the keyboard before the task
//! took it — the same letting-go Escape on an empty reply box does.

use crate::Server;
use crate::the_tasks_a_person_opened::WhichTask;

/// What a person's asking to close a task window came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatClosingCameTo {
    /// The window closed. Its work was already over.
    Closed,
    /// Nothing closed and nothing was asked: the task is already stopping, and
    /// closes only once it has stopped.
    ///
    /// Until 2026-10-29 this was every running task — working, stopping,
    /// or waiting for the person. Nothing about the task changed.
    StillRunning,
    /// **Asked**: the task is running, so the person is asked *Stop and close*
    /// or *Cancel* (`crate::asking_before_closing`, ADR 0139).
    Asked,
    /// There was no open window by that name to close.
    NothingToClose,
}

impl Server {
    /// The person asked to close this task's window.
    ///
    /// Closes it only when its work is over; see this file's header for why a
    /// running task does not close yet.
    pub fn the_person_closes_a_task(&mut self, which: WhichTask) -> WhatClosingCameTo {
        if !self.the_tasks.is_open(which) {
            return WhatClosingCameTo::NothingToClose;
        }
        let Some(task) = self.the_tasks.the(which) else {
            return WhatClosingCameTo::NothingToClose;
        };
        let nobody_asked_it_to_stop = matches!(task.stopping(), alo_tasks::Stopping::NobodyAsked);
        if !task.may_close_without_asking() {
            // **A running task asks**, unless it is already stopping, which
            // closes nothing until it has stopped (ADR 0139).
            if nobody_asked_it_to_stop {
                self.ask_before_closing(which);
                return WhatClosingCameTo::Asked;
            }
            return WhatClosingCameTo::StillRunning;
        }
        if !self.the_tasks.closed(which) {
            return WhatClosingCameTo::NothingToClose;
        }
        if self.the_task_with_the_keyboard == Some(which) {
            self.the_task_gave_the_keyboard_up();
        }
        WhatClosingCameTo::Closed
    }
}

#[cfg(test)]
#[path = "closing_a_task_window_tests.rs"]
mod tests;
