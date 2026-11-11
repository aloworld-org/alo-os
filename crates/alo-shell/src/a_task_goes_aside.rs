//! **A running task's window goes into the right-hand panel, and comes back.**
//!
//! `docs/features.md`: *one window per task, and minimising keeps it running —
//! minimising puts its preview, its activity and its Stop control in the
//! right-hand panel*. The owner's approved design `23:2` draws where: a square
//! in the rail and a card beside it, windows and tasks together, newest put
//! aside first, and *a task belongs to the Place it was asked on and returns
//! there*.
//!
//! # Minimising never touches the work
//!
//! Nothing here can stop anything. Putting a task aside changes where its
//! window is — [`crate::the_tasks_a_person_opened::TheTasks::put_aside`] — and
//! adds a number and a Place to the panel; the work goes on in the agent,
//! which nothing here reaches. That is the difference between *minimising
//! keeps it running* being designed and being true.
//!
//! # Asked where the press is, done where the panel is
//!
//! The panel belongs to the desktop, not to the server, so a Minimise pressed
//! on a task window's edge is remembered here and carried out once a frame
//! with the panel lent — the road a window's own Minimise takes
//! (`crate::putting_a_window_aside`).

use alo_put_aside::{Panel, TaskNumber};

use crate::the_tasks_a_person_opened::WhichTask;

impl crate::Server {
    /// **A person asked for this task's window to go into the panel.**
    pub(crate) fn this_task_is_asked_to_go_aside(&mut self, which: WhichTask) {
        if !self.tasks_asked_to_go_aside.contains(&which) {
            self.tasks_asked_to_go_aside.push(which);
        }
    }

    /// **Put every task asked for into the panel.** Answers how many went.
    ///
    /// Taken before anything is tried, so a refusal is not retried for ever;
    /// a desktop with no panel puts nothing aside and keeps the windows where
    /// they are, which is the honest answer rather than a window with nowhere
    /// to be.
    pub fn put_tasks_aside_that_were_asked(&mut self, panel: Option<&mut Panel>) -> usize {
        let asked = std::mem::take(&mut self.tasks_asked_to_go_aside);
        let Some(panel) = panel else {
            return 0;
        };
        asked
            .into_iter()
            .filter(|which| self.put_a_task_aside(panel, *which))
            .count()
    }

    /// **Put a running task's window aside**, into the panel, still running.
    /// Answers whether it went: a window not on the screen cannot.
    pub(crate) fn put_a_task_aside(&mut self, panel: &mut Panel, which: WhichTask) -> bool {
        let Some(place) = self.tasks_mut().put_aside(which) else {
            return false;
        };
        let task = TaskNumber::numbered(which.at());
        if panel.put_a_task_aside(task, place).is_err() {
            // Already there: put the window back, so it is in exactly one place.
            let _ = self.tasks_mut().brought_back(which);
            return false;
        }
        true
    }

    /// **Bring a running task's window back from the panel**, exactly where it
    /// was on the Place it was asked on. Answers whether it came back.
    pub(crate) fn bring_a_task_back(&mut self, panel: &mut Panel, task: TaskNumber) -> bool {
        if panel.bring_a_task_back(task).is_err() {
            return false;
        }
        if self.the_task_card_open == Some(task) {
            self.the_task_card_open = None;
        }
        self.tasks_mut()
            .brought_back(WhichTask::numbered(task.at()))
    }
}
