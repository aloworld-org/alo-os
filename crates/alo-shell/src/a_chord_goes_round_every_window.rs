//! **Super+Tab and Alt+Tab go round every window a person has open** — the
//! applications, then alo's terminal, then each task window — and the one they
//! land on comes in front.
//!
//! The owner's decision of 2026-10-31: *the window you go to comes in front*,
//! and the task window left behind *stays open behind it, one Super+Tab away*.
//! Until then these chords walked the applications alone, so a task window or
//! the terminal that an application had come in front of could not be reached
//! from the keyboard at all — which `docs/features.md` forbids: *no window is
//! buried behind another where the person cannot find it*.
//!
//! # When this answers, and when the old roads still do
//!
//! **Only while the terminal or a task window is open.** With applications
//! alone there is nothing new to walk, and the roads that have always answered
//! these chords — `crate::a_chord_moves_between_applications` for Super+Tab and
//! `Server::switch_window` for Alt+Tab — still do, exactly as before.
//!
//! # The walk
//!
//! Alt+Tab walks windows and Super+Tab applications, in the order those roads
//! already use, then the terminal, then the task windows oldest first. *Next*
//! starts from whatever has the keyboard: the task window holding it, the
//! terminal if it is in front, or the application window focused. One stop is
//! nowhere to go, and nothing moves.

use alo_shortcuts::{Action, Chord, Shortcuts};

use crate::the_tasks_a_person_opened::WhichTask;
use crate::which_window_is_in_front::AKindOfWindow;
use crate::window_switch::WindowSwitchDirection;

/// One place on the walk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AStop {
    /// An application's window, for Alt+Tab.
    AWindow(smithay::reexports::wayland_server::protocol::wl_surface::WlSurface),
    /// An application, for Super+Tab.
    AnApplication(alo_dock::AppId),
    /// alo's own terminal.
    TheTerminal,
    /// A task window.
    ATask(WhichTask),
}

/// The stop after `at` in `stops`, going `direction`; [`None`] when there is
/// nowhere else to go.
///
/// Nothing focused starts at the end the person is going from: the first
/// going forward, the last going back.
pub(crate) fn the_next_stop(
    stops: &[AStop],
    at: Option<&AStop>,
    direction: WindowSwitchDirection,
) -> Option<AStop> {
    let here = at.and_then(|at| stops.iter().position(|stop| stop == at));
    if stops.is_empty() || (stops.len() == 1 && here.is_some()) {
        return None;
    }
    let next = match (here, direction) {
        (None, WindowSwitchDirection::Forward) => 0,
        (None, WindowSwitchDirection::Backward) => stops.len() - 1,
        (Some(here), WindowSwitchDirection::Forward) => (here + 1) % stops.len(),
        (Some(here), WindowSwitchDirection::Backward) => (here + stops.len() - 1) % stops.len(),
    };
    stops.get(next).cloned()
}

impl crate::Server {
    /// The chord that goes round every window, if this is one of the four and
    /// the terminal or a task window is open.
    ///
    /// Answers whether it took the chord. **`true` even when there is nowhere
    /// to go**, for `crate::a_chord_opens_a_terminal`'s reason.
    pub(crate) fn a_chord_goes_round_every_window(
        &mut self,
        shortcuts: &Shortcuts,
        chord: Chord,
    ) -> bool {
        let (direction, by_application) = match shortcuts.action_for(chord) {
            Some(Action::NextWindow) => (WindowSwitchDirection::Forward, false),
            Some(Action::PreviousWindow) => (WindowSwitchDirection::Backward, false),
            Some(Action::NextApplication) => (WindowSwitchDirection::Forward, true),
            Some(Action::PreviousApplication) => (WindowSwitchDirection::Backward, true),
            _ => return false,
        };
        let a_terminal = self
            .alos_own_terminal()
            .is_some_and(|terminal| !terminal.has_finished());
        let tasks: Vec<WhichTask> = self
            .tasks()
            .each_open_window()
            .into_iter()
            .map(|(which, _)| which)
            .collect();
        if !a_terminal && tasks.is_empty() {
            return false;
        }
        let mut stops = self.the_applications_stops(by_application);
        if a_terminal {
            stops.push(AStop::TheTerminal);
        }
        stops.extend(tasks.into_iter().map(AStop::ATask));
        let here = self.the_stop_with_the_keyboard(by_application);
        match the_next_stop(&stops, here.as_ref(), direction) {
            Some(AStop::AWindow(surface)) => {
                let _ = self.the_person_goes_to_this_window(&surface);
            }
            Some(AStop::AnApplication(app)) => self.go_to_this_application(&app),
            Some(AStop::TheTerminal) => self.the_person_went_to(AKindOfWindow::TheTerminal),
            Some(AStop::ATask(which)) => {
                self.the_composer.it_gave_the_keyboard_up();
                self.this_task_took_the_keyboard(which);
            }
            None => {}
        }
        true
    }

    /// The applications' part of the walk, in the order their own road uses.
    fn the_applications_stops(&mut self, by_application: bool) -> Vec<AStop> {
        if by_application {
            return self
                .the_applications_the_keyboard_walks()
                .into_iter()
                .map(AStop::AnApplication)
                .collect();
        }
        self.switch_order.refresh(self.surfaces.buffered());
        self.switch_order
            .in_order()
            .filter(|root| self.surfaces.mapped_toplevel(root).is_some())
            .cloned()
            .map(AStop::AWindow)
            .collect()
    }

    /// Where the keyboard is, as a stop on the walk.
    fn the_stop_with_the_keyboard(&self, by_application: bool) -> Option<AStop> {
        if let Some(which) = self.the_task_with_the_keyboard {
            return Some(AStop::ATask(which));
        }
        if self.the_terminal_is_taking_the_keys() {
            return Some(AStop::TheTerminal);
        }
        let root = self.surfaces.keyboard_root()?;
        if by_application {
            self.the_application_of(&root).map(AStop::AnApplication)
        } else {
            Some(AStop::AWindow(root))
        }
    }
}

#[cfg(test)]
#[path = "a_chord_goes_round_every_window_tests.rs"]
mod tests;
