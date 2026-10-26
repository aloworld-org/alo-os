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

use alo_shortcuts::{Action, Chord};

use crate::Server;
use crate::a_chord_reaches_its_action::WhatTheChordCameTo;

/// Whether carrying out this action sends the keyboard somewhere other than
/// the task window that held it.
const fn sends_the_keyboard_elsewhere(action: Action) -> bool {
    match action {
        Action::Terminal
        | Action::NextWindow
        | Action::PreviousWindow
        | Action::NextApplication
        | Action::PreviousApplication
        | Action::TheNextArea
        | Action::ThePreviousArea => true,
        Action::TheAgent
        | Action::Launcher
        | Action::Settings
        | Action::WhyIsItWaiting
        | Action::WindowOptions
        | Action::CloseWindow
        | Action::MinimiseWindow
        | Action::MaximiseWindow
        | Action::SnapLeft
        | Action::SnapRight
        | Action::ZoomTheCanvasIn
        | Action::ZoomTheCanvasOut
        | Action::ShowAllOnTheCanvas
        | Action::MoveTheWindowToTheNextPlace
        | Action::GoBackOnThisPlace => false,
    }
}

impl Server {
    /// Carry out a shortcut pressed in the task window holding the keyboard,
    /// and let go of the keyboard if it went somewhere else.
    pub(crate) fn a_chord_in_a_task_window(&mut self, chord: Chord) -> WhatTheChordCameTo {
        let came_to = self.the_chord_does_what_it_names(chord);
        if let WhatTheChordCameTo::Carried(action) = came_to
            && sends_the_keyboard_elsewhere(action)
        {
            self.the_task_gave_the_keyboard_up();
        }
        came_to
    }
}
