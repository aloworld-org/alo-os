//! **The window alo acts in wears alo's presence for the turn  acts in** —
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
//! serves one turn at a time, so the act is the **task at work** — open,
//! and *alo is working* — at that moment. With none, nothing is marked: an act
//! from alo's terminal is not a task's. **With two at work the window is
//! marked anyway**, for the one asked most recently — the owner, 2026-11-15:
//! *a window alo types into is never left unmarked*. The border says alo, which
//! is true whichever task it was.
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
    /// What alo's work is sending, by application, as the indicator says it.
    sending: Vec<(String, alo_egress::HowItIsShown)>,
    /// The windows whose teal border was drawn since the corner last asked.
    drawn_round: Vec<WlSurface>,
}

impl TheWindowsAloActsIn {
    /// How many windows alo is acting in now.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.acting.len()
    }

    /// The tasks acting in this window, newest first, each with what its
    /// application is sending, if anything.
    pub(crate) fn tasks_in(
        &self,
        surface: &WlSurface,
    ) -> impl Iterator<Item = (WhichTask, Option<alo_egress::HowItIsShown>)> + '_ {
        let surface = surface.clone();
        self.acting
            .iter()
            .rev()
            .filter(move |acted| acted.surface == surface)
            .map(|acted| {
                let sending = self
                    .sending
                    .iter()
                    .find(|(application, _)| *application == acted.application)
                    .map(|(_, how)| *how);
                (acted.which, sending)
            })
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

/// **The task at work**: open, and *alo is working* — the one asked most
/// recently where two are, because a window alo acts in is never left unmarked
/// (the owner, 2026-11-15), and [`None`] where none is.
fn the_task_at_work(tasks: &TheTasks) -> Option<WhichTask> {
    (0..tasks.each().len())
        .rev()
        .filter_map(|at| tasks.the_task_numbered(at))
        .filter(|which| tasks.is_open(*which))
        .find(|which| {
            tasks
                .the(*which)
                .is_some_and(|task| task.what_is_happening() == alo_tasks::words::ALO_IS_WORKING)
        })
}

impl crate::Server {
    /// **alo is acting in this window**: it wears the task at work, for that
    /// task's turn. Answers which task, or [`None`] when no task is at work
    /// and nothing was marked.
    ///
    /// Called by the code that carries an act out, once it holds that the
    /// window is alo's to act in. `application` is the application alo asked
    /// to act in, by the identifier it used — what the log says, because an
    /// Electron program names its own Wayland window something shorter.
    pub fn alo_acts_in(&mut self, surface: &WlSurface, application: &str) -> Option<WhichTask> {
        let which = the_task_at_work(&self.the_tasks)?;
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
        eprintln!(
            "alo-shell: {}",
            alo_acting::said_on_the_log::alo_wears_a_window_of(application)
        );
        self.the_windows_alo_acts_in.acting.push(ActedIn {
            which,
            surface: surface.clone(),
            application: application.to_owned(),
        });
        Some(which)
    }

    /// **What alo's work is sending this frame**, by application, as the
    /// indicator says it — told by the desktop that holds alo's copies, once a
    /// frame, and read by the border of the window alo works in.
    pub fn what_alos_work_is_sending(&mut self, sending: Vec<(String, alo_egress::HowItIsShown)>) {
        self.the_windows_alo_acts_in.sending = sending;
    }

    /// **alo's teal border was drawn round this window**, on some display,
    /// this frame — told by the painter (`crate::every_windows_edge`), which
    /// is the only one that knows a border was not left off a screen.
    pub fn alos_border_was_drawn_round(&mut self, surface: &WlSurface) {
        let drawn = &mut self.the_windows_alo_acts_in.drawn_round;
        if !drawn.contains(surface) && drawn.len() < THE_MOST {
            drawn.push(surface.clone());
        }
    }

    /// **The applications whose copy's sending the border says, so the corner
    /// does not** — the owner, 2026-11-15: *the border is enough*. Each is the
    /// application of a window alo acts in whose teal edge was drawn since
    /// this was last asked **and** is still drawn now: alo acting, not waiting
    /// on the person (navy, no edge), and the turn not over. Anything else is
    /// left in the corner, so nothing alo's copy sends is ever shown nowhere.
    ///
    /// Asked once a frame, before the corner is told what to show; the
    /// drawing it read is let go, so a border not drawn again is not counted
    /// again.
    pub fn the_applications_wearing_alos_border(&mut self) -> Vec<String> {
        let drawn = std::mem::take(&mut self.the_windows_alo_acts_in.drawn_round);
        let mut wearing: Vec<String> = Vec::new();
        for surface in &drawn {
            let edged = self.alo_in_this_window(surface).is_some_and(|here| {
                here.wears == crate::alo_in_a_window::Wears::TheBorder
                    && here.how != crate::alo_in_a_window::HowAloIsHere::WaitingForThePerson
            });
            if !edged {
                continue;
            }
            for acted in &self.the_windows_alo_acts_in.acting {
                if acted.surface == *surface && !wearing.contains(&acted.application) {
                    wearing.push(acted.application.clone());
                }
            }
        }
        wearing
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
