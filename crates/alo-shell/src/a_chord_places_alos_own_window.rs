//! **Alt+F3 and Super+Up on alo's terminal and on a task window**: the
//! window's panel, and Maximise or Restore, by the roads its band takes.
//!
//! The same fault `crate::a_chord_closes_alos_own_window` records for Alt+F4:
//! the applications' road acts on **the application whose surface holds the
//! seat's keyboard**, and alo's windows have no surface to hold it. Left to
//! that road, Alt+F3 pressed in the terminal opened the panel of the
//! application behind it. Since the owner's *"Yes add that"* of 2026-11-14 the
//! terminal and the task windows have panels of their own
//! (`crate::alos_windows_panel`), so the chord opens that one.
//!
//! **Settings and the record have no panel yet** — their pages are laid out at
//! one size — so with one of them holding the keys this answers nothing, as it
//! did before.
//!
//! *This makes an existing promise true, because Alt+F3 is listed as showing
//! the window's options and Super+Up as maximising the window, and the
//! terminal and the task windows now have both.*

use alo_put_aside::AlosOwnWindow;
use alo_shortcuts::{Action, Chord, Shortcuts};

use crate::the_bands_over_alos_windows::AlosBand;
use crate::the_edge_takes_the_keyboard::ByWhat;
use crate::which_band::ABand;
use crate::window_menu::WhatARowDoes;

impl crate::Server {
    /// The one of alo's windows with a panel that holds the keyboard: the
    /// terminal, or the task window the keys go to.
    fn alos_window_with_the_keys_and_a_panel(&self) -> Option<AlosBand> {
        if self.the_terminal_is_taking_the_keys() {
            return Some(AlosBand::Its(AlosOwnWindow::TheTerminal));
        }
        if self.settings_is_taking_the_keys() || self.the_record_is_taking_the_keys() {
            return None;
        }
        self.the_task_with_the_keyboard.map(AlosBand::ATask)
    }

    /// Carry out Show the window's options or Maximise window on the one of
    /// alo's windows that holds the keyboard, if this chord is one of them and
    /// one does.
    ///
    /// Answers whether it was taken; `false` hands the chord on to the
    /// applications' road.
    pub(crate) fn a_chord_places_alos_own_window(
        &mut self,
        shortcuts: &Shortcuts,
        chord: Chord,
    ) -> bool {
        let action = shortcuts.action_for(chord);
        if !matches!(action, Some(Action::WindowOptions | Action::MaximiseWindow)) {
            return false;
        }
        let Some(which) = self.alos_window_with_the_keys_and_a_panel() else {
            return false;
        };
        if action == Some(Action::WindowOptions) {
            self.the_menu_was_opened_on_the_band(&ABand::Alos(which), ByWhat::TheKeyboard);
            return true;
        }
        let does = if self.is_alos_window_placed(which) {
            WhatARowDoes::Restore
        } else {
            WhatARowDoes::Maximise
        };
        self.carry_out_the_row_on(&ABand::Alos(which), does);
        true
    }
}
