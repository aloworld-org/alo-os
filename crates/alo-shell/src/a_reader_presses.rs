//! What a screen reader's press on the shell's own tree comes to.
//!
//! A press reaches here from `crate::access_action` by way of the desktop's
//! loop, and is handed to **the same door a pointer's press uses** — nothing
//! is decided here that a click does not also decide. So a reader pressing a
//! task window's Close gets exactly what clicking it gets
//! (`crate::closing_a_task_window`): a finished task closes, a task waiting
//! for the person stays open with its question unanswered, and a task alo is
//! still working on stays open until Stop exists. A reader pressing a control
//! on the Dock gets what clicking it or pressing Enter on it gets
//! (`Server::carry_out_what_the_bar_was_asked_for`).

use alo_dock::places::WhatIsHere;

use crate::Server;
use crate::a_press_on_the_dock::what_this_slot_does;
use crate::access_action::Pressed;
use crate::closing_a_task_window::WhatClosingCameTo;

/// What a reader's press came to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatAReaderPressCameTo {
    /// A task window's Close was pressed, and this is what closing it came to.
    ATaskWindow(WhatClosingCameTo),
    /// The Dock did what a click on that control does.
    TheDockDidIt,
    /// Nothing: no Dock drawn now holds that control, or a click on it does
    /// nothing either.
    NothingOnTheDock,
}

impl Server {
    /// A screen reader pressed `pressed` on the tree this machine published.
    pub fn a_reader_pressed(&mut self, pressed: Pressed) -> WhatAReaderPressCameTo {
        match pressed {
            Pressed::CloseTheTask(which) => {
                WhatAReaderPressCameTo::ATaskWindow(self.the_person_closes_a_task(which))
            }
            Pressed::OnTheDock(what) => self.a_reader_pressed_on_the_dock(&what),
        }
    }

    /// A reader pressed `what` on the Dock.
    ///
    /// **Held to a Dock that is drawn now**, as a click is held to landing on
    /// a slot: a tree is a moment old by the time it is pressed, and an
    /// application closed and unpinned in that moment must not be opened again
    /// by a press heard before it went.
    fn a_reader_pressed_on_the_dock(&mut self, what: &WhatIsHere) -> WhatAReaderPressCameTo {
        if !self.a_dock_drawn_now_holds(what) {
            return WhatAReaderPressCameTo::NothingOnTheDock;
        }
        if self.carry_out_what_the_bar_was_asked_for(Some(what_this_slot_does(what))) {
            WhatAReaderPressCameTo::TheDockDidIt
        } else {
            WhatAReaderPressCameTo::NothingOnTheDock
        }
    }
}

#[cfg(test)]
#[path = "a_reader_presses_tests.rs"]
mod tests;
