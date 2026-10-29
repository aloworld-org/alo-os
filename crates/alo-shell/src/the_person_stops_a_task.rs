//! **The person pressed Stop**: what the window does at once, and what it waits
//! for ([ADR 0139](../../../docs/decisions/0139-stop-reaches-alo-and-cancels-what-it-sent.md)).
//!
//! At the press the task says *stopping* — the request was asked, nothing is
//! claimed — and the Stop is handed to the frame loop, which owns the door to
//! the agent (`crate::stopping_what_was_asked`). *Stopped* is said only when the
//! agent's side has closed (`Server::an_answer_came_back` with `stopped`).
//!
//! # What a Stop never does
//!
//! - **It never answers a change.** A change waiting on this task's card lapses:
//!   neither approved nor declined, the card's two answers go, and the turn it
//!   lived in is stopped on the connection the card kept.
//! - **It never sends a held reply.** One waiting to be sent is given back to
//!   the reply box, so nothing the person typed is lost and nothing goes.

use alo_tasks::TheOneButton;

use crate::the_tasks_a_person_opened::WhichTask;

/// A Stop waiting for the frame loop to carry it to the agent.
#[derive(Debug)]
pub struct AStop {
    /// The task it is for.
    pub which: WhichTask,
    /// The connection a card kept the agent's turn open on, when a change was
    /// waiting; [`None`] when the request is still being read.
    pub held_turn: Option<std::os::unix::net::UnixStream>,
}

impl crate::Server {
    /// **The person pressed Stop in this task.** Answers whether there was
    /// anything to stop: only a task the one button shows Stop for.
    pub(crate) fn the_person_stops_a_task(&mut self, which: WhichTask) -> bool {
        let waiting = &mut self.the_change_waiting;
        let Some(task) = self.the_tasks.the_mut(which) else {
            return false;
        };
        if task.the_one_button() != TheOneButton::Stop {
            return false;
        }
        task.stop();
        if let Some(held) = task.take_held()
            && task.the_draft().is_empty()
        {
            task.the_person_is_typing(held);
        }
        let held_turn = if task.permissions().asking().is_some() {
            waiting.let_go_unanswered(task)
        } else {
            None
        };
        self.stops.push(AStop { which, held_turn });
        true
    }

    /// Take the Stops waiting to be carried to the agent.
    ///
    /// **Taken rather than read**, so one press stops once.
    pub fn take_the_stops(&mut self) -> Vec<AStop> {
        std::mem::take(&mut self.stops)
    }

    /// **The agent has gone after a Stop**: the task says *stopped*, naming
    /// the changes the person allowed before it, and nothing else.
    pub(crate) fn it_really_stopped(&mut self, which: WhichTask) {
        if let Some(task) = self.the_tasks.the_mut(which) {
            let changed = task.what_it_changed();
            task.it_stopped(changed);
        }
        // **And a window *Stop and close* asked for closes now**, and not
        // before (`crate::asking_before_closing`).
        self.close_it_if_it_was_asked(which);
    }
}

#[cfg(test)]
#[path = "the_person_stops_a_task_tests.rs"]
mod tests;
