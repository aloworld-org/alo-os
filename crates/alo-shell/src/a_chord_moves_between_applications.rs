//! Moving between applications with the keyboard, which two chords named and
//! nothing did.
//!
//! `Super`+`Tab` and `Super`+`Shift`+`Tab` have been bound to
//! `alo_shortcuts::Action::NextApplication` and `PreviousApplication` since the
//! shortcuts were written. Measured 2026-10-14: the shell's dispatch chain had
//! **zero** arms for either, so both were shipped, worded, translated and bound
//! and pressing one was indistinguishable from pressing an unbound key.
//!
//! # What this is, next to `crate::window_switch`
//!
//! `Alt`+`Tab` moves between **windows**; this moves between
//! **applications**, and it is the same mechanism at a coarser grain — which is
//! what those two chords mean on every system a person may be arriving from.
//! So it walks the same list `switch_window` walks, grouped by application:
//! using a different order would make `Super`+`Tab` and `Alt`+`Tab` disagree
//! about what *next* means, and a person would have two mental models for one
//! key.
//!
//! **An application appears once**, at the place its first window appears. A
//! list that repeated an application would make *next* take several presses to
//! leave it, which is the complaint every window switcher that got this wrong
//! has earned.
//!
//! # Where it lands inside the application
//!
//! It does not choose a window itself: `Server::go_to_this_application` does,
//! and that is `alo_dock::clicking`'s rule — *click an app to return to where
//! you last used it*. So the chord and a press on the Dock icon land on the
//! same window, which they must: two roads to one application that chose
//! differently would be two behaviours a person has to learn.
//!
//! # The arithmetic is separate, and that is deliberate
//!
//! [`the_next_application`] takes a list and a current position and answers.
//! It needs no display, no seat and no surfaces, so it is tested against every
//! case that matters — empty, one application, wrapping both ways, and a
//! focused window whose application is not in the list at all. What needs a
//! compositor is only *which* list, and that is the thin part.

use alo_dock::window::AppId;
use alo_shortcuts::{Action, Chord, Shortcuts};

use crate::window_switch::WindowSwitchDirection;

/// The application a press moves to, from the list and where the keyboard is.
///
/// `None` when there is nowhere to go: no applications at all, or only the one
/// already focused — and that second case is a refusal rather than a wrap onto
/// itself, because focusing what is already focused is a keypress that appears
/// to do nothing while having done exactly what it said.
///
/// **A focused application not in the list wraps to the first**, which is the
/// honest answer for a real state: the focused window may be a surface the Dock
/// does not count, and *move to the next application* from outside the list
/// means *start at the beginning*.
#[must_use]
pub(crate) fn the_next_application(
    order: &[AppId],
    focused: Option<&AppId>,
    direction: WindowSwitchDirection,
) -> Option<AppId> {
    if order.len() < 2 {
        // One application is nowhere to go, and zero is nowhere to go from.
        // **Not a wrap onto itself**: see this function's own note.
        return None;
    }
    let at = focused.and_then(|app| order.iter().position(|other| other == app));
    let next = match (at, direction) {
        // Nothing of ours is focused: start at either end, whichever way the
        // person asked to go.
        (None, WindowSwitchDirection::Forward) => 0,
        (None, WindowSwitchDirection::Backward) => order.len() - 1,
        (Some(at), WindowSwitchDirection::Forward) => (at + 1) % order.len(),
        // `+ len` rather than a subtraction that could go below zero, which is
        // the one place this arithmetic could panic in debug and wrap in
        // release.
        (Some(at), WindowSwitchDirection::Backward) => (at + order.len() - 1) % order.len(),
    };
    order.get(next).cloned()
}

impl crate::Server {
    /// The applications this person has open, in the order the keyboard walks
    /// them, each once.
    ///
    /// **The same list `switch_window` walks**, so the two chords agree about
    /// what *next* means. A window whose application this compositor cannot
    /// name is skipped rather than given a placeholder: a chord that focused
    /// *the unnamed one* would be a chord a person cannot predict.
    pub(crate) fn the_applications_the_keyboard_walks(&mut self) -> Vec<AppId> {
        self.switch_order.refresh(self.surfaces.buffered());
        let mut order: Vec<AppId> = Vec::new();
        let surfaces: Vec<_> = self.switch_order.in_order().cloned().collect();
        for surface in surfaces {
            if self.surfaces.mapped_toplevel(&surface).is_none() {
                continue;
            }
            let Some(app) = self.the_application_of(&surface) else {
                continue;
            };
            if !order.contains(&app) {
                order.push(app);
            }
        }
        order
    }

    /// The chord that moves between applications, if this is one of the two.
    ///
    /// Answers whether the chord was one of them, so the caller knows not to
    /// pass it on. **`true` even when there is nowhere to go**, for the reason
    /// `crate::a_chord_opens_a_terminal` gives: the chord is still these
    /// actions', and letting it fall through would hand it to a road with no
    /// arm for it, so a person pressing a bound chord would get the behaviour
    /// of an unbound one.
    ///
    /// Pressing it with one application open, or none, does nothing and is not
    /// a fault — the screen already shows there is nowhere else to be.
    pub(crate) fn a_chord_moves_between_applications(
        &mut self,
        shortcuts: &Shortcuts,
        chord: Chord,
    ) -> bool {
        let direction = match shortcuts.action_for(chord) {
            Some(Action::NextApplication) => WindowSwitchDirection::Forward,
            Some(Action::PreviousApplication) => WindowSwitchDirection::Backward,
            _ => return false,
        };
        let order = self.the_applications_the_keyboard_walks();
        let focused = self
            .surfaces
            .keyboard_root()
            .and_then(|root| self.the_application_of(&root));
        if let Some(next) = the_next_application(&order, focused.as_ref(), direction) {
            self.go_to_this_application(&next);
        }
        true
    }
}

#[cfg(test)]
#[path = "a_chord_moves_between_applications_tests.rs"]
mod tests;
