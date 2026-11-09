//! What a reader is told about an open window panel.
//!
//! The specification's *screen-reader names, roles, selected and expanded
//! states* (`docs/design/the-window-controls.md`, §9), for the panel Window
//! actions opens: its name, which choices are in it, and which one the
//! keyboard is on — told as **selected**, so a person who cannot see the
//! navy picture hears which choice is in hand.
//!
//! # Why this is not `window_edge_reading`
//!
//! That file answers *what can be done to this window*, with no geometry and no
//! state — the list that hangs under each window whether or not its band is
//! revealed. This answers *what is in front of me now*, about one panel that is
//! open. The two change for different reasons, which is law 4.

use alo_access::words::Word;
use alo_access::{Control, Role, State};

use crate::window_menu::{WhatARowDoes, WindowMenu};
use crate::window_menu_keys::TheKeyboardInTheMenu;

/// The panel as a reader meets it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatAReaderMeetsInTheMenu {
    /// The panel itself, named *Window* — its heading.
    ///
    /// [`Role::List`], with the choices under it: the shape
    /// `crate::access_nodes` already uses for the list of open windows. No role
    /// is called `Menu`, and a role is a public surface other people's readers
    /// match on.
    pub panel: Control,
    /// The choices, in keyboard order, each with whether it is the one the
    /// keyboard is on.
    pub rows: Vec<(Control, bool)>,
}

/// What a reader is told about this open panel.
///
/// **The choices come from the panel that was laid out**, so a reader cannot
/// be told about a choice the panel does not draw.
#[must_use]
pub fn what_a_reader_meets(
    menu: &WindowMenu,
    keyboard: Option<TheKeyboardInTheMenu>,
) -> WhatAReaderMeetsInTheMenu {
    let on = keyboard.and_then(TheKeyboardInTheMenu::on);
    WhatAReaderMeetsInTheMenu {
        panel: Control {
            role: Role::List,
            name: alo_access::words::THE_WINDOW_PANEL,
            // **The panel is not pressed; its choices are.**
            state: State::ReadOnly,
            setting: None,
            does: None,
        },
        rows: menu
            .rows
            .iter()
            .map(|row| (a_row(row.does), on == Some(row.does)))
            .collect(),
    }
}

/// One choice, named by what it does, and selectable — one of a few, and the
/// one in hand is told as selected.
fn a_row(does: WhatARowDoes) -> Control {
    Control {
        role: Role::Button,
        name: does.its_own_word(),
        state: State::OneOfAFew,
        setting: None,
        does: None,
    }
}

/// Every word this file can put in front of a reader.
///
/// **So `alo-access`' own identity can count them**: that crate's
/// `EVERY_WORD_ANOTHER_CRATES_SURFACE_SAYS` counts the words it declares that
/// only this crate's surfaces say.
#[must_use]
pub fn every_word_this_surface_says() -> Vec<Word> {
    let mut words = vec![alo_access::words::THE_WINDOW_PANEL];
    for does in [
        WhatARowDoes::Maximise,
        WhatARowDoes::Restore,
        WhatARowDoes::FullScreen,
        WhatARowDoes::ExitFullScreen,
    ] {
        words.push(does.its_own_word());
    }
    words
}

#[cfg(test)]
#[path = "window_menu_reading_tests.rs"]
mod tests;
