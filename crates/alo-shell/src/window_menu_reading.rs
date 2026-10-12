//! What a reader is told about an open window menu.
//!
//! The contract's *give the menu an accessible name*, and the rest of what a
//! person who cannot see the panel needs: which rows are in it, which of them
//! will act on this window, which one has the keyboard, and the window's own
//! title.
//!
//! # Why this is not `window_edge_reading`
//!
//! That file answers *what can be done to this window*, at every width, with
//! no geometry and no state — it is the list that hangs under each window in
//! `crate::access_nodes` whether or not the edge is revealed, because what a
//! person can reach is not what a pointer is near. This file answers a
//! different question: *what is in front of me now*, about one menu that is
//! open. The two would change for different reasons, which is law 4.
//!
//! They must agree about the actions, and
//! `every_row_the_menu_offers_is_an_action_the_edge_announces` is that
//! agreement as a test rather than as a hope.
//!
//! # The title is a string and cannot be a [`Control`]
//!
//! A `Control`'s name is an [`alo_access::Word`] — a phrase with a key, which
//! the person's language is looked up by. A window's title is whatever the
//! application called itself, already in whatever language it chose, and no
//! vocabulary can hold it. `crate::access_nodes` meets the same wall for the
//! list of open windows and resolves it the same way: the node's name is a
//! plain string there, and [`WhatAReaderMeetsInTheMenu::title`] is a plain
//! string here.
//!
//! **So the title is carried beside the controls rather than as one**, and
//! this file does not word the case where an application named nothing —
//! `access_nodes` already falls back to `alo_access::words::AN_APPLICATION`
//! for exactly that, and a second fallback here would be a second answer to
//! one question.
//!
//! # Every row, including the ones that will not act
//!
//! *Explain unavailable actions accessibly.* A reader meets five rows on every
//! window and hears [`State::ReadOnly`] on the ones this window does not
//! allow, which is *read and nothing more* — true, and still only half of what
//! is owed. It is not told **why**: that a window with one size cannot be
//! resized. `crate::window_menu::WhatARowDoes::how_a_reader_finds_it` records
//! that debt where the state is decided, and it is still owed here.

use alo_access::words::Word;
use alo_access::{Control, Role, State};

use crate::window_menu::{WhatARowDoes, WindowMenu};
use crate::window_menu_keys::TheKeyboardInTheMenu;

/// The menu as a reader meets it: the panel, its title, its rows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhatAReaderMeetsInTheMenu {
    /// The panel itself, named *Window menu*.
    ///
    /// [`Role::List`] — *a list of things* — with the rows under it, which is
    /// the shape `crate::access_nodes` already uses for the list of open
    /// windows. No [`Role`] is called `Menu` and none was added: a role is a
    /// public surface other people's readers match on, and *a list of things
    /// a reader walks* is what this is and what `List` already says.
    pub panel: Control,
    /// The window's own title, as selectable and copyable text.
    ///
    /// [`None`] where the application named nothing. See this file's header
    /// for why this is a string rather than a [`Control`], and why the
    /// fallback phrase is the caller's.
    pub title: Option<String>,
    /// The five rows, in the design's order, available and unavailable alike.
    pub rows: Vec<Control>,
    /// Which row has the keyboard, where one does.
    ///
    /// **Carried beside the rows rather than inside one**, because `Control`
    /// has no focused field and inventing one would change a surface every
    /// other reader in this repository already matches on.
    pub the_keyboard_is_on: Option<WhatARowDoes>,
}

/// What a reader is told about this open menu.
///
/// **The rows come from the menu that was laid out**, not from a list this
/// file holds, so a reader cannot be told about a row the panel does not draw
/// — which is the fault ADR 0089 is about, in the other direction.
#[must_use]
pub fn what_a_reader_meets(
    menu: &WindowMenu,
    keyboard: Option<TheKeyboardInTheMenu>,
    title: Option<&str>,
) -> WhatAReaderMeetsInTheMenu {
    WhatAReaderMeetsInTheMenu {
        panel: Control {
            role: Role::List,
            name: alo_access::words::THE_WINDOW_MENU,
            // **The panel is not pressed; its rows are.** A list carrying
            // `CanBeUsed` would tell a reader the panel itself does
            // something, and then a person would look for what.
            state: State::ReadOnly,
            setting: None,
            does: None,
        },
        title: title.map(str::to_owned),
        rows: menu
            .rows
            .iter()
            .map(|row| a_row(row.does, row.available))
            .collect(),
        the_keyboard_is_on: keyboard.and_then(TheKeyboardInTheMenu::on),
    }
}

/// One row, named by what it does.
///
/// ADR 0089's rule, kept the way `crate::window_edge_reading` keeps it: the
/// name is the action's own word where there is an action, and one of
/// `alo-access`' own words where there is not. `WhatARowDoes::its_own_word`
/// holds that choice for both files, so the name a reader says in the menu and
/// the name it says for the window cannot come to differ.
fn a_row(does: WhatARowDoes, available: bool) -> Control {
    Control {
        role: Role::Button,
        name: does.its_own_word(),
        state: WhatARowDoes::how_a_reader_finds_it(available),
        setting: None,
        does: does.what_it_does(),
    }
}

/// Every word this file can put in front of a reader.
///
/// **So `alo-access`' own identity can count them.** That crate holds
/// `EVERY_WORD_ANOTHER_CRATES_SURFACE_SAYS` — words it declares that only
/// another crate's surface says — and `THE_WINDOW_MENU` is the third. A list
/// here is what lets a test ask *is it said*, rather than a reader finding out
/// that it is not.
#[must_use]
pub fn every_word_this_surface_says() -> Vec<Word> {
    let mut words = vec![alo_access::words::THE_WINDOW_MENU];
    for does in [
        WhatARowDoes::Minimise,
        WhatARowDoes::Maximise,
        WhatARowDoes::Restore,
        WhatARowDoes::Move,
        WhatARowDoes::Resize,
        WhatARowDoes::Close,
    ] {
        words.push(does.its_own_word());
    }
    words
}

#[cfg(test)]
#[path = "window_menu_reading_tests.rs"]
mod tests;
