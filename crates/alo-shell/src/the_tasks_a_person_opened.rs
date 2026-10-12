//! The task windows this person has open, and the request that opens one.
//!
//! *Each new task gets its own window; follow-ups stay in that window* — the
//! owner, 2026-10-12. So there are two roads and they are different: a request
//! from the composer **opens** a task, and a reply **continues** one.
//!
//! # Why the shell holds them and does not run them
//!
//! A task is work the agent is doing. This crate shows and never measures —
//! `crates/alo-shell/tests/settings_source.rs` refuses a shipped file here that
//! even names where settings are kept, and the same rule puts inference
//! somewhere else entirely. So what is here is **which tasks exist and which
//! window is which**, and what answers them is `alo-asking`, reached by the
//! binary.
//!
//! # A request opens a window even before anything answers
//!
//! The window appears with the person's own words in it and an empty activity,
//! and that is deliberate: a request that produced nothing until a model
//! replied would leave somebody looking at a Dock, wondering whether their
//! keypress registered. **The first thing a task window shows is proof that the
//! machine heard them.**

use alo_tasks::ATask;

/// Which task a person is looking at or talking to.
///
/// **An index into the list rather than a borrow**, because the `Server` hands
/// out `&mut` to its own fields all over this crate and a task identified by a
/// reference could not be held across one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WhichTask(usize);

impl WhichTask {
    /// Its place in the list, for a caller that has to index.
    #[must_use]
    pub const fn at(self) -> usize {
        self.0
    }
}

/// Every task window this person has open, oldest first.
#[derive(Debug, Default)]
pub struct TheTasks {
    /// The tasks, in the order they were opened.
    ///
    /// **Never reordered.** A list that moved the busy one to the top would
    /// move the window a person was reaching for, which is the instability the
    /// Dock's own ordering rule exists to prevent and is worse here: these are
    /// windows, and a window that moves under a hand is one somebody closes by
    /// accident.
    open: Vec<ATask>,
}

impl TheTasks {
    /// Nobody has asked for anything.
    #[must_use]
    pub fn none_yet() -> Self {
        Self::default()
    }

    /// Take a task somebody built for a request.
    ///
    /// **Built elsewhere and handed over**, because making one needs an
    /// `alo_models::InferenceSource` — which model answers here — and that is a
    /// reading of this person's settings. `crates/alo-shell` shows and never
    /// measures, and its own manifest says `alo-models` is *never a dependency
    /// of the shipped surface, which only draws*. So the binary that read the
    /// setting makes the task and this holds it.
    pub fn and_now_this_one(&mut self, task: ATask) -> WhichTask {
        self.open.push(task);
        WhichTask(self.open.len() - 1)
    }

    /// One task, to read.
    #[must_use]
    pub fn the(&self, which: WhichTask) -> Option<&ATask> {
        self.open.get(which.at())
    }

    /// One task, to change.
    pub fn the_mut(&mut self, which: WhichTask) -> Option<&mut ATask> {
        self.open.get_mut(which.at())
    }

    /// Every task, oldest first.
    #[must_use]
    pub fn each(&self) -> &[ATask] {
        &self.open
    }

    /// How many are open.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.open.len()
    }

    /// Which tasks are waiting for the person to answer something.
    ///
    /// **The question a person asks of a screenful of windows.** Answered here
    /// rather than by each surface, so the panel, the Dock and the window list
    /// cannot come to disagree about which task needs somebody.
    pub fn waiting_for_the_person(&self) -> impl Iterator<Item = WhichTask> {
        self.open
            .iter()
            .enumerate()
            .filter(|(_, task)| task.conversation().is_waiting_for_the_person())
            .map(|(at, _)| WhichTask(at))
    }
}

impl crate::Server {
    /// A person asked alo for something in the composer.
    ///
    /// **Records the asking and opens nothing**, which is the same shape as
    /// `Server::asked_to_start` and for the same reason: making a task needs to
    /// know which model answers here, and that is a reading of this person's
    /// settings rather than anything this crate may do. The binary drains this,
    /// builds the task, and hands it back through
    /// [`TheTasks::and_now_this_one`].
    ///
    /// Answers whether anything was asked. **An empty composer asks nothing**:
    /// a person who presses Enter on a blank field has not asked for anything,
    /// and a window with no request in it is one they have to close.
    pub fn a_person_asked_alo(&mut self, request: &str) -> bool {
        let request = request.trim();
        if request.is_empty() {
            return false;
        }
        self.asked_alo.push(request.to_owned());
        true
    }

    /// Take the requests waiting to become tasks.
    ///
    /// **Taken rather than read**, so one Enter opens one window however many
    /// frames pass before the binary asks — the capability model's *one
    /// approval causes exactly one execution*, applied to the one keypress that
    /// starts everything.
    pub fn take_what_was_asked_of_alo(&mut self) -> Vec<String> {
        std::mem::take(&mut self.asked_alo)
    }

    /// The tasks this person has open.
    #[must_use]
    pub const fn tasks(&self) -> &TheTasks {
        &self.the_tasks
    }

    /// The tasks, to be changed.
    pub const fn tasks_mut(&mut self) -> &mut TheTasks {
        &mut self.the_tasks
    }
}

#[cfg(test)]
#[path = "the_tasks_a_person_opened_tests.rs"]
mod tests;
