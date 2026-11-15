//! **The window alo acts in wears alo's presence for the turn it acts in** —
//! for a request the person typed in the Bar, where alo chose the window
//! itself.
//!
//! `crate::alo_in_a_window` marks a window the person *offered* with *Help
//! with this window*. ADR 0143 promises more: *while alo is working in a
//! window, that window wears the agent's presence … because a person watching
//! their editor type must be able to tell whose hands are on it*. A window alo
//! picks for itself is learned only when alo acts in it, so this is told at
//! those moments, by the code that carries each act out:
//!
//! - **a file alo opened comes in front** in the window alo may type into
//!   (`crate::a_file_alo_opened_comes_in_front`);
//! - **alo types, saves or asks whether it may type** in the window in front
//!   (`crate::typing_into_the_window_in_front`) — and only once every check
//!   that it is alo's to type into has held, so a refused act marks nothing.
//!
//! What it wears is the owner's ruling of 2026-11-15 — *if the window is
//! being operated by alo it should just have a border with one of our brand
//! colors* — so the teal edge and alo's mark on a tab of it, and no band
//! ([`crate::alo_in_a_window::Wears::TheBorder`]), drawn by the one painter
//! stage 3 built (`crate::alos_band_in_a_window`) from the one answer
//! `Server::alo_in_this_window` gives — never a second copy.
//!
//! # Whose turn
//!
//! The door alo acts through carries no task (`alo_acting`), and `alo-agentd`
//! serves one turn at a time, so the act is the **one task at work** — open,
//! and *alo is working* — at that moment. With none, nothing is marked: an act
//! from alo's terminal is not a task's. **With two at work, nothing is marked
//! either**, because a border cannot say which task, and the wrong one is worse
//! than none; the task windows still say both are
//! working. That is a reading put to the owner.
//!
//! # For the turn, and no longer
//!
//! A window is the task's for the turn alo acted in it: through a change
//! waiting for the person's OK (navy, no edge) and through *Stopping…*, and let
//! go once a frame sees that turn over — complete, stopped, its task window
//! closed — or the window gone. A follow-up is a new turn, and wears the window
//! again only if alo acts in it again. Each going on and coming off is said on
//! the log by the window's application, never its title
//! (`alo_acting::said_on_the_log`), so a walk can check the edge on the screen
//! between the two.

use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

use crate::the_tasks_a_person_opened::{TheTasks, WhichTask};

/// The most windows held at once; alo acts in one or two a turn, so the bound
/// is for a caller with a bug.
const THE_MOST: usize = 16;

/// One window alo acted in, for one task's turn.
#[derive(Debug)]
struct ActedIn {
    /// The task whose turn it was.
    which: WhichTask,
    /// The window.
    surface: WlSurface,
    /// Its application, as the log says it.
    application: String,
}

/// The windows alo acted in this turn, oldest first.
#[derive(Debug, Default)]
pub struct TheWindowsAloActsIn {
    /// Each window, and whose turn.
    acting: Vec<ActedIn>,
}

impl TheWindowsAloActsIn {
    /// How many windows alo is acting in now.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.acting.len()
    }

    /// The tasks acting in this window, newest first.
    pub(crate) fn tasks_in(&self, surface: &WlSurface) -> impl Iterator<Item = WhichTask> + '_ {
        let surface = surface.clone();
        self.acting
            .iter()
            .rev()
            .filter(move |acted| acted.surface == surface)
            .map(|acted| acted.which)
    }

    /// Let go of each window whose turn is over, or which has gone, and answer
    /// their applications, one each.
    pub(crate) fn looked_at(&mut self, alive: &[WlSurface], tasks: &TheTasks) -> Vec<String> {
        let (over, still): (Vec<_>, Vec<_>) = std::mem::take(&mut self.acting)
            .into_iter()
            .partition(|acted| {
                !alive.contains(&acted.surface) || !the_turn_goes_on(tasks, acted.which)
            });
        self.acting = still;
        over.into_iter().map(|acted| acted.application).collect()
    }
}

/// Whether this task's turn is still going on: open, and working, waiting on
/// the person, or stopping — the three ways a window wears alo.
fn the_turn_goes_on(tasks: &TheTasks, which: WhichTask) -> bool {
    tasks.is_open(which)
        && tasks
            .the(which)
            .is_some_and(|task| crate::alo_in_a_window::wears_alo(task.what_is_happening()))
}

/// **The one task at work**: open, and *alo is working*. [`None`] for none,
/// and for two or more, which cannot be told apart.
fn the_one_at_work(tasks: &TheTasks) -> Option<WhichTask> {
    let mut at_work = (0..tasks.each().len())
        .filter_map(|at| tasks.the_task_numbered(at))
        .filter(|which| tasks.is_open(*which))
        .filter(|which| {
            tasks
                .the(*which)
                .is_some_and(|task| task.what_is_happening() == alo_tasks::words::ALO_IS_WORKING)
        });
    let one = at_work.next()?;
    at_work.next().is_none().then_some(one)
}

impl crate::Server {
    /// **alo is acting in this window**: it wears the task at work, for that
    /// task's turn. Answers which task, or [`None`] when no one task is at work
    /// and nothing was marked.
    ///
    /// Called by the code that carries an act out, once it holds that the
    /// window is alo's to act in.
    pub fn alo_acts_in(&mut self, surface: &WlSurface) -> Option<WhichTask> {
        let which = the_one_at_work(&self.the_tasks)?;
        let held = &mut self.the_windows_alo_acts_in.acting;
        if held
            .iter()
            .any(|acted| acted.which == which && acted.surface == *surface)
        {
            return Some(which);
        }
        if held.len() >= THE_MOST {
            held.remove(0);
        }
        // **Asked only of a window on the screen**: one that is not has no
        // application to name, and asking its state would read nothing.
        let application = if self.mapped_surfaces().any(|it| it == surface) {
            self.the_app_id_of(surface).unwrap_or_default()
        } else {
            String::new()
        };
        eprintln!(
            "alo-shell: {}",
            alo_acting::said_on_the_log::alo_wears_a_window_of(&application)
        );
        self.the_windows_alo_acts_in.acting.push(ActedIn {
            which,
            surface: surface.clone(),
            application,
        });
        Some(which)
    }

    /// **Once a frame**: let go of each window whose turn is over or which has
    /// gone, saying so on the log.
    pub fn the_windows_alo_acts_in_are_looked_at(&mut self) {
        if self.the_windows_alo_acts_in.acting.is_empty() {
            return;
        }
        let alive: Vec<WlSurface> = self.mapped_surfaces().cloned().collect();
        for application in self
            .the_windows_alo_acts_in
            .looked_at(&alive, &self.the_tasks)
        {
            eprintln!(
                "alo-shell: {}",
                alo_acting::said_on_the_log::alos_work_is_over_in_a_window_of(&application)
            );
        }
    }
}

#[cfg(test)]
#[path = "the_window_alo_acts_in_tests.rs"]
mod tests;
