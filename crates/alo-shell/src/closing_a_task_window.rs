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
//! Two roads reach it, and both end in [`Server::the_person_closes_a_task`]:
//! Alt+F4 while the task window holds the keyboard, and Close on its edge
//! (`crate::a_task_windows_edge`).
//!
//! # The three choices are not built, so a running task does not close
//!
//! Each choice leans on something that does not exist yet: *Stop and close*
//! on a Stop that reaches the agent, *Keep running* on the right-hand panel's
//! place for a task, and all three on a sheet the design has not drawn. Until
//! they exist, **closing a running task is *Cancel***: nothing closes, nothing
//! stops, and a change waiting for the person is neither approved nor
//! declined. That is the one of the three that needs nothing else, and the
//! only behaviour that keeps both halves of the promise — never killed, never
//! lost.
//!
//! **Nothing here can approve anything**: this file never reaches
//! `crate::a_change_waiting_reaches_the_person` or a daemon, and a test holds
//! that a change waiting when a person presses Alt+F4 is still waiting after.
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
    /// Nothing closed, because the task is still running — working, stopping,
    /// or waiting for the person. Nothing about the task changed.
    StillRunning,
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
        if !task.may_close_without_asking() {
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
