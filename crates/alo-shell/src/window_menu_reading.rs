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
///
/// **A placement preview has a spoken equivalent**: while Left or Right is
/// the choice the keyboard is on, `preview` is what its outline's corner
/// says — *Left half · preview*, or that the window cannot fit — and the
/// choice is named with it, so a reader hears where the window would go
/// before anything moves.
#[must_use]
pub fn what_a_reader_meets(
    menu: &WindowMenu,
    keyboard: Option<TheKeyboardInTheMenu>,
    preview: Option<Word>,
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
            .map(|row| {
                let held = on == Some(row.does);
                let mut control = a_row(row.does);
                if let (true, Some(says), Some(_)) = (held, preview, row.does.the_side()) {
                    control.name = says;
                }
                (control, held)
            })
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
        WhatARowDoes::Left,
        WhatARowDoes::Right,
        WhatARowDoes::Maximise,
        WhatARowDoes::Restore,
        WhatARowDoes::FullScreen,
        WhatARowDoes::ExitFullScreen,
    ] {
        words.push(does.its_own_word());
    }
    words.extend([
        alo_access::words::LEFT_HALF_PREVIEW,
        alo_access::words::RIGHT_HALF_PREVIEW,
        alo_access::words::NO_ROOM_IN_HALF,
    ]);
    words
}

#[cfg(test)]
#[path = "window_menu_reading_tests.rs"]
mod tests;
