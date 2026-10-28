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

use alo_strings::Direction;
use alo_tasks::ATask;
use smithay::utils::{Logical, Physical, Point, Rectangle};

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

    /// The task at this place.
    ///
    /// **Only a test makes one of these**, which is why it is `cfg(test)`:
    /// every real one comes back from [`TheTasks::and_now_this_one`], so a task
    /// that is named has been opened. A caller that could mint one could name a
    /// task that does not exist — and `TheTasks::the_mut` would answer `None`,
    /// which is safe but is a question nothing should be able to ask.
    #[cfg(test)]
    #[must_use]
    pub(crate) const fn numbered(at: usize) -> Self {
        Self(at)
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
    /// Where each task's window is, in the same order.
    ///
    /// **Placed once and then kept**, which is the design's own requirement:
    /// *apply default placement once; later redraws or remapping must not
    /// unexpectedly reposition it.* A position recomputed each frame is a
    /// window that jumps when somebody else's opens.
    ///
    /// [`ItsWindow::NowhereYet`] until something places it. A task exists the
    /// moment somebody asks, and the frame that places it is the next one — so
    /// *asked for and not yet anywhere* is a real state rather than a gap.
    ///
    /// **A parallel list, and held to it by `debug_assert` at every door that
    /// changes either.** Two vectors that must stay the same length is a shape
    /// worth distrusting; it is this way because `ATask` is `alo-tasks`' and a
    /// window's position is the shell's, and putting a screen rectangle inside
    /// the model would be the shell's state kept in the wrong crate.
    placed: Vec<ItsWindow>,
}

/// Where one task's window is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ItsWindow {
    /// Asked for, and not yet placed.
    NowhereYet,
    /// On the screen, here.
    At(Rectangle<i32, Physical>),
    /// The person closed it.
    ///
    /// **Its place in the list is kept**, and that is the point of this state
    /// rather than removing the task. A [`WhichTask`] is a place in the list,
    /// and an answer still on its way from the agent carries one — so a list
    /// that closed up the gap would hand the next task's answer to the task
    /// after it. A closed window is never placed, drawn or pressed again.
    Closed,
}

impl ItsWindow {
    /// Where it is on the screen, if anywhere.
    const fn on_the_screen(self) -> Option<Rectangle<i32, Physical>> {
        match self {
            Self::At(at) => Some(at),
            Self::NowhereYet | Self::Closed => None,
        }
    }
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
        self.placed.push(ItsWindow::NowhereYet);
        debug_assert_eq!(self.open.len(), self.placed.len());
        WhichTask(self.open.len() - 1)
    }

    /// Where each task's window is, in the order the tasks opened, and
    /// [`None`] for one not yet placed or closed.
    #[must_use]
    pub fn where_each_went(&self) -> Vec<Option<Rectangle<i32, Physical>>> {
        self.placed
            .iter()
            .map(|window| window.on_the_screen())
            .collect()
    }

    /// The person closed this task's window. Answers whether one was open to
    /// close.
    ///
    /// **Bookkeeping only**: whether a window *may* close is
    /// `crate::closing_a_task_window`'s question, and this is the step after
    /// it has been answered.
    pub(crate) fn closed(&mut self, which: WhichTask) -> bool {
        match self.placed.get_mut(which.at()) {
            Some(window) if *window != ItsWindow::Closed => {
                *window = ItsWindow::Closed;
                true
            }
            _ => false,
        }
    }

    /// Whether this task's window is still open — placed or about to be.
    #[must_use]
    pub fn is_open(&self, which: WhichTask) -> bool {
        self.placed
            .get(which.at())
            .is_some_and(|window| *window != ItsWindow::Closed)
    }

    /// The task window under this point on the screen, if one is.
    ///
    /// **The last opened answers first**, because it is drawn last and so is
    /// in front: a press where two windows overlap belongs to the one a person
    /// can see. A window nothing has placed is not under anything.
    #[must_use]
    pub fn the_one_under(&self, point: Point<i32, Physical>) -> Option<WhichTask> {
        self.placed
            .iter()
            .enumerate()
            .rev()
            .find_map(|(at, placed)| {
                placed
                    .on_the_screen()
                    .filter(|window| window.contains(point))
                    .map(|_| WhichTask(at))
            })
    }

    /// Put the window of every task that has nowhere to be.
    ///
    /// Answers how many were placed, so a caller can tell *nothing to do* from
    /// *a view too small to put one in* — both leave the list unchanged and
    /// only one of them is a machine working as intended.
    ///
    /// **Already-placed windows are not moved**, and are handed to the placer
    /// as the ones already open so a new window lands beside them rather than
    /// on them.
    pub fn put_whatever_has_nowhere_to_be(
        &mut self,
        view: Rectangle<i32, Logical>,
        reading: Direction,
        fixed: &[Rectangle<i32, Logical>],
    ) -> usize {
        debug_assert_eq!(self.open.len(), self.placed.len());
        let mut placed = 0;
        for which in 0..self.placed.len() {
            // **Only a window nothing has placed**: one on the screen stays
            // where it is, and a closed one is never put back.
            if self.placed.get(which) != Some(&ItsWindow::NowhereYet) {
                continue;
            }
            // **Front-to-back: the newest first.** That makes the most recently
            // placed window the active one for the next placement, which is
            // what *beside the active window* means when alo is the one
            // opening them.
            let already: Vec<Rectangle<i32, Logical>> = self
                .placed
                .iter()
                .rev()
                .filter_map(|window| window.on_the_screen())
                .map(|at| {
                    Rectangle::new((at.loc.x, at.loc.y).into(), (at.size.w, at.size.h).into())
                })
                .collect();
            let Some(at) = crate::where_a_task_window_opens::where_one_more_goes(
                view, &already, reading, fixed,
            ) else {
                // A view too small for the window at all. Nothing is placed and
                // nothing is invented: the next frame on a larger display will
                // place it.
                continue;
            };
            if let Some(slot) = self.placed.get_mut(which) {
                *slot = ItsWindow::At(crate::where_a_task_window_opens::as_a_frame_draws_it(at));
                placed += 1;
            }
        }
        placed
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

    /// The task at this place in the list, to change, for a frame handing back
    /// what it measured about the window it drew there.
    ///
    /// **A place, not a [`WhichTask`]**, because the frame read it off the list
    /// it was given; a place nothing is at answers `None`.
    pub(crate) fn at_its_place(&mut self, place: usize) -> Option<&mut ATask> {
        self.open.get_mut(place)
    }

    /// Every task, oldest first — **closed ones included**, because a task's
    /// place in this list is how it is named; [`Self::is_open`] says which.
    #[must_use]
    pub fn each(&self) -> &[ATask] {
        &self.open
    }

    /// How many windows are open: every task the person has not closed.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.placed
            .iter()
            .filter(|window| **window != ItsWindow::Closed)
            .count()
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
            .filter(|(_, task)| task.is_waiting_for_the_person())
            .map(|(at, _)| WhichTask(at))
            .filter(|which| self.is_open(*which))
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
