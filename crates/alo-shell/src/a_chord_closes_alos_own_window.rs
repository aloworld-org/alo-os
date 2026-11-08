//! **Alt+F4 closes the one of alo's own windows that has the keyboard**, by
//! the road its Close button takes.
//!
//! Settings → Keyboard and shortcuts lists *Close the window: Alt F4*, and
//! `docs/design/the-shortcuts-and-the-edges.md` says the same. Walking a
//! booted machine on 2026-11-08, Alt+F4 in alo's terminal with `sleep 300`
//! running asked nothing and the terminal stayed — while the Close button on
//! the same window asked *"A command is still running. Close anyway?"*.
//!
//! # Why it did nothing, and what it did instead
//!
//! The chord was carried out on the road every application window takes,
//! `Server::dispatch_window_shortcut`, which closes **the application whose
//! surface holds the seat's keyboard**. alo's terminal, Settings and the record
//! are drawn by this compositor and have no surface to hold it: taking the keys
//! leaves the seat's focus where it was, on purpose (`crate::a_key_reaches_settings`
//! has the reason). So Alt+F4 at one of them asked the application **behind**
//! it to close — and in that walk the file manager behind the terminal was gone
//! after the press. The same fault `crate::a_task_window_hands_on_a_chord`
//! records fixing for task windows on 2026-10-28.
//!
//! # What it does now
//!
//! Exactly what each window's Close does, through the same function:
//!
//! - **the terminal**: `Server::the_person_closes_the_terminal` — idle, it
//!   ends; with a command running, the question opens with the keyboard on
//!   Cancel;
//! - **Settings**: `Server::the_person_closes_settings`, which gives up a
//!   provider form that is open, as its Close does;
//! - **the record**: `Server::the_person_closes_the_record`.
//!
//! A task window holding the keyboard never reaches this road:
//! `crate::a_task_window_hands_on_a_chord` answers Alt+F4 for it first.
//!
//! **Taken whenever one of them has the keys**, so the chord can never fall
//! through to an application the person cannot see. With none of them holding
//! the keys this answers nothing, and Alt+F4 closes the application in front as
//! it always did.
//!
//! *This makes an existing promise true, because Alt+F4 is listed and
//! designed as closing the window, and alo's windows are windows.*

use alo_shortcuts::{Action, Chord, Shortcuts};

impl crate::Server {
    /// Carry out Close window on the one of alo's own windows that holds the
    /// keyboard, if this chord is Close window and one does.
    ///
    /// Answers whether it was taken; `false` hands the chord on to the
    /// applications' road.
    pub(crate) fn a_chord_closes_alos_own_window(
        &mut self,
        shortcuts: &Shortcuts,
        chord: Chord,
    ) -> bool {
        if shortcuts.action_for(chord) != Some(Action::CloseWindow) {
            return false;
        }
        // In the order `crate::a_chord_goes_round_every_window` asks where the
        // keyboard is, so the two cannot disagree about which window that is.
        if self.settings_is_taking_the_keys() {
            self.the_person_closes_settings();
            return true;
        }
        if self.the_record_is_taking_the_keys() {
            self.the_person_closes_the_record();
            return true;
        }
        if self.the_terminal_is_taking_the_keys() {
            let _ = self.the_person_closes_the_terminal();
            return true;
        }
        false
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "a_chord_closes_alos_own_window_tests.rs"]
mod tests;
