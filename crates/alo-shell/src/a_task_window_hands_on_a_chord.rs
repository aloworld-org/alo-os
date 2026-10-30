//! A shortcut pressed while a task window holds the keyboard.
//!
//! **Every shortcut was swallowed there**, found walking a booted machine on
//! 2026-10-26: with the task window that had just asked a question holding the
//! keyboard, Ctrl+Alt+T opened nothing. A task window intercepts every key so an
//! application behind it hears nothing of a reply — and the shortcuts are read
//! in `crate::keyboard`, on the road that interception skips. So it is carried
//! out here, the same way that road carries it out.
//!
//! # Which shortcuts take the keyboard away from the task
//!
//! **Ordinary focus**, the owner's choice of 2026-10-26: the window a person
//! goes to has the keyboard. A shortcut that sends them to another window —
//! the terminal, the next window or application, the next area — takes the
//! keyboard from the task. One whose surface is drawn **in front of** the task
//! and takes keys before it does — the agent, the Bar, Settings — leaves it,
//! so that closing that surface brings typing back to the task. The rest act on
//! what is already there and leave it as well.
//!
//! Written as one exhaustive match with no catch-all, so a shortcut added
//! tomorrow does not compile until somebody has answered this question for it.
//!
//! # Close window closes the task window
//!
//! **Alt+F4 in a task window closed an application behind it** until
//! 2026-10-28: the shortcut was carried out on the road every other window
//! takes, and that road knows only applications' windows. The window holding
//! the keyboard is the one a person means, so here it closes the task — or,
//! for a task still running, does nothing, which `crate::closing_a_task_window`
//! explains.

use alo_shortcuts::{Action, Chord};

use crate::Server;
use crate::a_chord_reaches_its_action::WhatTheChordCameTo;

/// Whether carrying out this action sends the keyboard somewhere other than
/// the task window that held it: to another window, or — F6 and Shift+F6 — to
/// another area of the screen.
///
/// The window half is `crate::a_chord_sends_the_keyboard_to_a_window`'s, which
/// is exhaustive, so the two cannot disagree about which shortcut goes to a
/// window.
const fn sends_the_keyboard_elsewhere(action: Action) -> bool {
    crate::a_chord_sends_the_keyboard_to_a_window::sends_the_keyboard_to_a_window(action)
        || matches!(action, Action::TheNextArea | Action::ThePreviousArea)
}

impl Server {
    /// Carry out a shortcut pressed in the task window holding the keyboard,
    /// and let go of the keyboard if it went somewhere else.
    pub(crate) fn a_task_window_hands_on_a_chord(&mut self, chord: Chord) -> WhatTheChordCameTo {
        let names = self
            .shortcuts
            .as_ref()
            .and_then(|shortcuts| shortcuts.action_for(chord));
        if names == Some(Action::CloseWindow)
            && let Some(which) = self.the_task_with_the_keyboard
        {
            return match self.the_person_closes_a_task(which) {
                crate::closing_a_task_window::WhatClosingCameTo::Closed
                | crate::closing_a_task_window::WhatClosingCameTo::Asked => {
                    WhatTheChordCameTo::Carried(Action::CloseWindow)
                }
                crate::closing_a_task_window::WhatClosingCameTo::StillRunning
                | crate::closing_a_task_window::WhatClosingCameTo::NothingToClose => {
                    WhatTheChordCameTo::Refused
                }
            };
        }
        let came_to = self.a_chord_is_carried_out(chord);
        if let WhatTheChordCameTo::Carried(action) = came_to
            && sends_the_keyboard_elsewhere(action)
        {
            self.the_task_gave_the_keyboard_up();
        }
        came_to
    }
}
