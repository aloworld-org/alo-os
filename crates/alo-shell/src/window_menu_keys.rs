//! Walking the window menu with the keyboard, and giving focus back.
//!
//! The contract's *every action must be reachable by pointer, touch, keyboard
//! and assistive technology*, for the half that is the keyboard. `#630` laid
//! the menu out and gave it a pointer hit test
//! ([`WindowMenu::what_is_under`](crate::WindowMenu::what_is_under)); this is
//! the road a person who never touches a mouse takes to the same five rows.
//!
//! # Why focus return is in the type and not in a caller's discipline
//!
//! *Return focus to its trigger when dismissed.* A menu that closes and leaves
//! the keyboard nowhere strands whoever arrived by keyboard — they are in a
//! shell with no focused control and no way back but the pointer they do not
//! use. So every outcome that closes the menu **names where the keyboard
//! goes**, in the value it returns, and a caller that drops it will not
//! compile having silently stranded someone: there is no variant meaning
//! *closed, and focus is somebody else's problem*.
//!
//! [`alo_dock::revealing::FocusGoes`] is the same idea for a panel and is not
//! reused, because its two answers are *back to the window* and *nowhere* and
//! neither is true here. The menu's trigger is a control on the window's own
//! edge, so what it gives back is an [`OnTheEdge`], which is a third answer
//! rather than one of those two.
//!
//! # The trigger is carried, not assumed
//!
//! [`the_menu_of`](crate::the_menu_of) refuses to build a menu for an edge with
//! no Menu control, so in practice the trigger is always
//! [`OnTheEdge::Menu`]. It is stored anyway, set when the menu opens, because
//! *the thing that opened it* and *the only control a menu can anchor to* are
//! two facts that happen to agree today, and a keyboard route opened from
//! somewhere else later would otherwise hand focus to a control that was never
//! pressed.
//!
//! # What walks and what does not
//!
//! | key | what it does |
//! |---|---|
//! | Down | the next usable row, wrapping to the first |
//! | Up | the previous usable row, wrapping to the last |
//! | Home, End | the first and the last usable row |
//! | Enter | runs the focused row, closes, gives focus back |
//! | Escape | closes with nothing run, gives focus back |
//!
//! **Unavailable rows are stepped over, never landed on.** They stay drawn and
//! a reader still meets them — `crate::window_menu::MenuRow::available` says
//! why that is — but a person pressing Down four times must reach four things
//! they can use, not stop on a row that will not act. Stepping over is
//! `WindowMenu::what_can_be_used`, which is the same list the pointer road
//! would act on, so the two roads cannot come to differ about which rows work.
//!
//! Tab is not handled here. It is the shell's own between-surfaces key and
//! taking it inside a menu is a decision about the whole focus order, which
//! this file is not the place to make.

use alo_dock::revealing::AMenu;

use crate::window_edge::OnTheEdge;
use crate::window_menu::{WhatARowDoes, WindowMenu};

/// A key pressed while the menu has the keyboard.
///
/// Named by what a person means rather than by a keysym, because the chord is
/// the host's to map: `crate::window_control_reader_keys` draws the same line,
/// and a file holding geometry and a file holding scancodes are two reasons to
/// change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AMenuKey {
    /// The next row down.
    Next,
    /// The previous row up.
    Previous,
    /// The first row.
    First,
    /// The last row.
    Last,
    /// Run the focused row. Enter, or Space.
    Run,
    /// Close with nothing run. Escape.
    Close,
}

/// What a key press did to the open menu.
///
/// Three of the four close it, and **each of those names where the keyboard
/// goes**. See this file's header for why that is in the value rather than in
/// a rule a caller remembers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatTheKeyDid {
    /// The keyboard is now on this row, and the menu is still open.
    TheKeyboardIsOn(WhatARowDoes),
    /// This row acts, the menu closes, and the keyboard goes back to the
    /// control that opened it.
    ///
    /// **The row is what it does, not an action to run.** Move and Resize have
    /// no [`alo_shortcuts::Action`] — `WhatARowDoes::what_it_does` records why
    /// — so a caller reads the row and decides, exactly as the pointer road
    /// does with `WindowMenu::what_is_under`.
    ItRuns {
        /// The row that was pressed.
        does: WhatARowDoes,
        /// Where the keyboard goes.
        focus_goes_back_to: OnTheEdge,
    },
    /// The menu closes with nothing run, and the keyboard goes back.
    ItCloses {
        /// Where the keyboard goes.
        focus_goes_back_to: OnTheEdge,
    },
    /// Nothing happened and the menu is still open.
    ///
    /// Two causes, and both are honest refusals rather than lost presses: a
    /// window that allows nothing has no usable row to move to, and Run with
    /// the keyboard on no row has nothing to run.
    Nothing,
}

/// Where the keyboard is inside one open menu.
///
/// **Held by the row's meaning and not by its index.** A window that cannot be
/// maximised still draws five rows, so an index would be stable — but
/// `WhatARowDoes` survives the menu being laid out again at a different size,
/// which happens whenever the window moves near an output edge, and an index
/// into a freshly built `Vec` is a bet that nothing reordered it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TheKeyboardInTheMenu {
    /// The row with the keyboard, or [`None`] before any key has moved it.
    on: Option<WhatARowDoes>,
    /// The control that opened the menu, and the control focus returns to.
    trigger: OnTheEdge,
}

impl TheKeyboardInTheMenu {
    /// Open, with the keyboard on no row yet.
    ///
    /// **No row rather than the first.** A menu opened with the pointer must
    /// not show a focus ring on Minimise that nobody asked for, and a person
    /// who opened it with the keyboard presses Down once to get there — which
    /// is one press for the keyboard road and none of the pointer road's
    /// phantom highlight. `crate::window_edge_paint` draws from the focused
    /// control, so *no row* and *the first row* are visibly different states.
    #[must_use]
    pub const fn opened_from(trigger: OnTheEdge) -> Self {
        Self { on: None, trigger }
    }

    /// The row with the keyboard, for the painter and for a reader.
    #[must_use]
    pub const fn on(self) -> Option<WhatARowDoes> {
        self.on
    }

    /// The control focus returns to when this menu closes.
    #[must_use]
    pub const fn trigger(self) -> OnTheEdge {
        self.trigger
    }

    /// What this key does, and where it leaves the keyboard.
    ///
    /// **The menu is asked, every time, rather than remembered.** Which rows
    /// can be used is a property of the window as it is now — a window that
    /// has just become unresizable must not still offer Resize to the
    /// keyboard — and `WindowMenu` is rebuilt from the window each frame
    /// anyway.
    pub fn told(&mut self, key: AMenuKey, menu: &WindowMenu) -> WhatTheKeyDid {
        let usable: Vec<WhatARowDoes> = menu
            .what_can_be_used()
            .into_iter()
            .map(|row| row.does)
            .collect();
        match key {
            // **Closing works with nothing usable and with no focused row**,
            // because a person who can open a menu must always be able to shut
            // it. An Escape that depended on the window's capabilities would
            // trap whoever opened the menu of a window that allows nothing.
            AMenuKey::Close => WhatTheKeyDid::ItCloses {
                focus_goes_back_to: self.trigger,
            },
            AMenuKey::Run => match self.on {
                // Asked of `usable` and not of `self.on`: a row that could be
                // used when the keyboard landed on it and cannot now must not
                // act. The window changed under the menu, which is the case
                // this re-reading exists for.
                Some(does) if usable.contains(&does) => WhatTheKeyDid::ItRuns {
                    does,
                    focus_goes_back_to: self.trigger,
                },
                Some(_) | None => WhatTheKeyDid::Nothing,
            },
            AMenuKey::First => self.moved_to(usable.first().copied()),
            AMenuKey::Last => self.moved_to(usable.last().copied()),
            AMenuKey::Next => self.moved_to(step(&usable, self.on, Forwards)),
            AMenuKey::Previous => self.moved_to(step(&usable, self.on, Backwards)),
        }
    }

    /// Put the keyboard on this row, where there was one to move to.
    fn moved_to(&mut self, row: Option<WhatARowDoes>) -> WhatTheKeyDid {
        match row {
            Some(does) => {
                self.on = Some(does);
                WhatTheKeyDid::TheKeyboardIsOn(does)
            }
            // **The keyboard is left where it was.** A window allowing nothing
            // has nowhere to move to, and clearing focus on a press that could
            // not move would lose a person's place for pressing a key.
            None => WhatTheKeyDid::Nothing,
        }
    }
}

/// Which way along a short list the keyboard is walking.
///
/// `pub(crate)` and shared with `crate::window_edge_keys`, which walks the
/// edge's controls with the same two answers. One direction type rather than
/// two private ones, because a second copy is the thing that comes to disagree.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Which {
    /// Towards the end of the list.
    Forwards,
    /// Towards the start.
    Backwards,
}

pub(crate) use Which::{Backwards, Forwards};

/// The next usable row in this direction, wrapping, or [`None`] if there are
/// none.
///
/// **Wrapping, because the list is short and circular to a person**: five rows
/// in a panel, where Down from Close reaching Minimise is what every menu a
/// person has used does. `crate::window_control_reader_keys` deliberately does
/// *not* wrap and says so — its pages are a sequence a person is reading
/// through, where wrapping would hide having reached the end. Opposite
/// decisions about different things.
///
/// **From no row, Next gives the first and Previous the last**, so a person
/// who opened the menu reaches either end in one press.
fn step(usable: &[WhatARowDoes], from: Option<WhatARowDoes>, which: Which) -> Option<WhatARowDoes> {
    if usable.is_empty() {
        return None;
    }
    let Some(from) = from else {
        return match which {
            Forwards => usable.first().copied(),
            Backwards => usable.last().copied(),
        };
    };
    // A focused row that is no longer usable has no neighbours to step from, so
    // this starts over rather than guessing which end it was nearer.
    let Some(at) = usable.iter().position(|it| *it == from) else {
        return match which {
            Forwards => usable.first().copied(),
            Backwards => usable.last().copied(),
        };
    };
    let next = match which {
        // `% usable.len()` is safe past the emptiness check above, and the
        // addition cannot overflow: `at` came from `position` on this slice.
        Forwards => (at + 1) % usable.len(),
        Backwards => (at + usable.len() - 1) % usable.len(),
    };
    usable.get(next).copied()
}

/// Whether the edge must stay revealed, told from the menu.
///
/// **This is the join `crate::the_window_edge_reveals` left named and unfed.**
/// That file's header records all four of §5's keep-revealed conditions as
/// already present in [`alo_dock::revealing::Revealing`], and that three of
/// them — keyboard, menu, drag — had nothing to report them because §6's
/// interactions did not exist. An open menu is the first of the three to
/// exist, and this is the one line that reports it.
///
/// *Keep the edge revealed during keyboard focus, dragging and an open menu.*
/// An edge that concealed while its own menu was open would take the menu's
/// anchor out from under it, and the pointer is usually off the edge while
/// a person is reading down the rows — which is exactly the case
/// `AMenu::IsOpen` exists for.
#[must_use]
pub const fn the_edge_stays_revealed(menu: Option<&TheKeyboardInTheMenu>) -> AMenu {
    match menu {
        Some(_) => AMenu::IsOpen,
        None => AMenu::IsNot,
    }
}

#[cfg(test)]
#[path = "window_menu_keys_tests.rs"]
mod tests;
