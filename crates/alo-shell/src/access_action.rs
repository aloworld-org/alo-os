//! **A control a screen reader can press, and the road the press takes.**
//!
//! `crate::access_bus` answers what every node *is*. This answers what a few of
//! them *do*: `org.a11y.atspi.Action`, served beside `Accessible` on each node
//! that carries a [`Pressed`], so a reader that finds the control can also
//! press it — Orca's "activate", or any client calling `DoAction`.
//!
//! # A press is handed over, never acted on here
//!
//! The bus answers on its own thread and the shell's state lives on the
//! compositor's. So `DoAction` does one thing: it puts *what was pressed* on a
//! channel, and the desktop's loop takes it off on its next frame and hands it
//! to `Server::a_reader_pressed`, which goes through the same door a click
//! does. Nothing on this side can close, approve or decline anything, which is
//! what keeps a reader's press to exactly the rules a pointer's press follows.
//!
//! `DoAction` answers `true` when the press was handed over, which is what
//! every toolkit's bridge means by it: the action was accepted, not that its
//! outcome is known. A running task that does not close is still a press that
//! was accepted, as a click on its Close is.
//!
//! # The protocol's names are not words
//!
//! `GetName` answers `click`, the name AT-SPI clients look for on a button and
//! the one GTK's bridge uses. It is an identifier on the wire, as a role number
//! is, and no person is told it. What a person is told is
//! `GetLocalizedName`, which is the control's own name in their language — the
//! word drawn on it — so pressing Close is announced as *Close*.

use std::sync::mpsc::Sender;

use crate::the_tasks_a_person_opened::WhichTask;

/// The interface a pressable node answers besides `Accessible`.
pub(crate) const ACTION: &str = "org.a11y.atspi.Action";

/// The action's name on the wire: what a client asks for, never what a person
/// hears.
const CLICK: &str = "click";

/// **What a reader pressed**, as the shell acts on it.
///
/// One variant, because one control on the tree can be pressed today: a task
/// window's Close (the owner's decision of 2026-10-28). A window edge's
/// controls on an application window are announced and not yet pressable; see
/// `crate::access_nodes`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pressed {
    /// Close on this task window's edge.
    CloseTheTask(WhichTask),
}

/// One pressable node, answering `org.a11y.atspi.Action`.
pub(crate) struct Pressable {
    /// What pressing it asks the shell to do.
    pub(crate) pressed: Pressed,
    /// Its name in the person's language, which is the node's own name.
    pub(crate) said: String,
    /// Where a press is handed to the compositor's loop.
    pub(crate) to: Sender<Pressed>,
}

#[zbus::interface(name = "org.a11y.atspi.Action")]
impl Pressable {
    /// How many actions it has: one, being pressed.
    #[zbus(property, name = "NActions")]
    fn n_actions(&self) -> i32 {
        1
    }

    /// Nothing: the name is the whole of what pressing it does.
    fn get_description(&self, _index: i32) -> String {
        String::new()
    }

    /// The action's name on the wire, for the one action there is.
    fn get_name(&self, index: i32) -> String {
        if index == 0 {
            CLICK.to_owned()
        } else {
            String::new()
        }
    }

    /// The action's name as a person hears it: the control's own name.
    fn get_localized_name(&self, index: i32) -> String {
        if index == 0 {
            self.said.clone()
        } else {
            String::new()
        }
    }

    /// No key is announced here. Alt+F4 closes the task window that has the
    /// keyboard, not the one a reader is on, so naming it would be a promise
    /// about the wrong window.
    fn get_key_binding(&self, _index: i32) -> String {
        String::new()
    }

    /// Every action, as name, description and key — the localized name, as
    /// at-spi2's own bridge answers it.
    fn get_actions(&self) -> Vec<(String, String, String)> {
        vec![(self.said.clone(), String::new(), String::new())]
    }

    /// Press it: hand the press to the compositor's loop.
    ///
    /// `false` for any action but the one there is, and when nothing is left
    /// to take the press — a desktop that has gone. Never `true` for a press
    /// that was not handed over.
    fn do_action(&self, index: i32) -> bool {
        index == 0 && self.to.send(self.pressed).is_ok()
    }
}

#[cfg(test)]
#[path = "access_action_tests.rs"]
mod tests;
