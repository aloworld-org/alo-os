//! What a screen reader's press on the shell's own tree comes to.
//!
//! A press reaches here from `crate::access_action` by way of the desktop's
//! loop, and is handed to **the same door a pointer's press uses** — nothing
//! is decided here that a click does not also decide. So a reader pressing a
//! task window's Close gets exactly what clicking it gets
//! (`crate::closing_a_task_window`): a finished task closes, a task waiting
//! for the person stays open with its question unanswered, and a task alo is
//! still working on stays open until Stop exists.

use crate::Server;
use crate::access_action::Pressed;
use crate::closing_a_task_window::WhatClosingCameTo;

impl Server {
    /// A screen reader pressed `pressed` on the tree this machine published.
    pub fn a_reader_pressed(&mut self, pressed: Pressed) -> WhatClosingCameTo {
        match pressed {
            Pressed::CloseTheTask(which) => self.the_person_closes_a_task(which),
        }
    }
}

#[cfg(test)]
#[path = "a_reader_presses_tests.rs"]
mod tests;
