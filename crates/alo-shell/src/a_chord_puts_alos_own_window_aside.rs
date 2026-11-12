//! **The Minimise chord puts the one of alo's own windows that has the
//! keyboard into the right-hand panel**, by the road its Minimise button takes.
//!
//! The same fault `crate::a_chord_closes_alos_own_window` records for Alt+F4,
//! met before it could happen: the applications' road minimises **the
//! application whose surface holds the seat's keyboard**, and alo's terminal,
//! Settings and the record have no surface to hold it. Left to that road, the
//! chord pressed at Settings would put the application behind Settings aside.
//!
//! **Taken whenever one of them has the keys**, in the order the closing chord
//! asks, so the two chords cannot disagree about which window that is. With
//! none of them holding the keys this answers nothing and the chord minimises
//! the application in front as it always did.
//!
//! *This makes an existing promise true, because the Minimise chord is listed
//! as minimising the window, and alo's windows are windows.*

use alo_put_aside::AlosOwnWindow;
use alo_shortcuts::{Action, Chord, Shortcuts};

impl crate::Server {
    /// Carry out Minimise window on the one of alo's own windows that holds
    /// the keyboard, if this chord is Minimise window and one does.
    ///
    /// Answers whether it was taken; `false` hands the chord on to the
    /// applications' road.
    pub(crate) fn a_chord_puts_alos_own_window_aside(
        &mut self,
        shortcuts: &Shortcuts,
        chord: Chord,
    ) -> bool {
        if shortcuts.action_for(chord) != Some(Action::MinimiseWindow) {
            return false;
        }
        let which = if self.settings_is_taking_the_keys() {
            AlosOwnWindow::Settings
        } else if self.the_record_is_taking_the_keys() {
            AlosOwnWindow::TheRecord
        } else if self.the_terminal_is_taking_the_keys() {
            AlosOwnWindow::TheTerminal
        } else {
            return false;
        };
        self.this_alo_window_is_asked_to_go_aside(which);
        true
    }
}
