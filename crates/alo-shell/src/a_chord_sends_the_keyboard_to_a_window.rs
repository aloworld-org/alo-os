//! **A shortcut that sends the keyboard to a window takes it off the Dock.**
//!
//! F6 puts the keyboard on the Dock (`crate::which_area_has_the_keyboard`), and
//! from then on the Dock takes Enter, Tab, the arrows and Escape. Nothing took
//! it off again but a second F6. The twelfth Demo 1 walk (2026-10-30) pressed
//! F6, then `Ctrl`+`Alt`+`T`: letters went to the new terminal, and Enter went
//! on pressing the Dock's Settings button behind it, twice, while the person
//! was typing a command.
//!
//! **Ordinary focus**, the owner's choice of 2026-10-26 that
//! `crate::a_task_window_hands_on_a_chord` already carries out for a task
//! window: the window a person goes to has the keyboard. So every shortcut that
//! goes to a window puts the keyboard back with the work, and every key then
//! reaches the one place.
//!
//! Written as one exhaustive match with no catch-all, so a shortcut added
//! tomorrow does not compile until somebody has said where it sends the
//! keyboard.

use alo_shortcuts::{Action, Chord};

use crate::Server;
use crate::a_chord_reaches_its_action::WhatTheChordCameTo;

/// Whether carrying out this action puts the keyboard in a window.
///
/// The terminal opens and takes the keys; the four window and application
/// moves focus another window. The rest either act on what is already there,
/// open a surface drawn in front that takes keys before the Dock does and gives
/// them back when it closes, or — F6 and Shift+F6 — move the keyboard among
/// the areas themselves, which is their own business.
pub(crate) const fn sends_the_keyboard_to_a_window(action: Action) -> bool {
    match action {
        Action::Terminal
        | Action::NextWindow
        | Action::PreviousWindow
        | Action::NextApplication
        | Action::PreviousApplication => true,
        Action::TheNextArea
        | Action::ThePreviousArea
        | Action::TheAgent
        | Action::Launcher
        | Action::Settings
        | Action::SeeWhatAloDid
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
        | Action::ReturnTheCanvasToLifeSize
        | Action::MoveTheWindowToTheNextPlace
        | Action::GoBackOnThisPlace => false,
    }
}

impl Server {
    /// Carry out a chord, and take the keyboard off the Dock if it went to a
    /// window.
    ///
    /// Only a chord that was **carried out** moves it: `Alt`+`Tab` with no
    /// window to go to went nowhere, and a person on the Dock is still there.
    ///
    /// **Not named `a_chord_…`**, because that prefix marks a link *in* the
    /// dispatch chain and `the_road_every_action_takes` requires the chain to
    /// call every one. This calls the chain; it is not a link in it.
    pub(crate) fn carry_out_a_chord(&mut self, chord: Chord) -> WhatTheChordCameTo {
        let came_to = self.the_chord_does_what_it_names(chord);
        if let WhatTheChordCameTo::Carried(action) = came_to
            && sends_the_keyboard_to_a_window(action)
        {
            self.the_keyboard_is_now_in(None);
        }
        came_to
    }
}

#[cfg(test)]
#[path = "a_chord_sends_the_keyboard_to_a_window_tests.rs"]
mod tests;
