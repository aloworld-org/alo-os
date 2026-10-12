//! The keyboard on a window's edge: reaching its controls, opening its menu,
//! and holding the edge revealed while it is there.
//!
//! §6 of `docs/design/the-external-window-edge.md`. Two files have been waiting
//! on this one and each says so in its own header:
//!
//! | what was waiting | what it was waiting for |
//! |---|---|
//! | `crate::the_window_edge_is_pointed_at` | *nothing in this repository can focus a control yet … every caller passes [`None`]* |
//! | `crate::the_window_edge_reveals` | `Revealing::the_keyboard` and `a_menu` present and fed by nothing |
//!
//! Both are now fed from here, which is the whole point of the file: a focus
//! ring the painter can draw, and the two reveal conditions that keep the edge
//! on the screen while a person is using it without a pointer.
//!
//! # One vocabulary of keys, two destinations
//!
//! A person presses the same six keys whether the keyboard is on the edge or
//! inside its menu, so [`AnEdgeKey`] and
//! [`AMenuKey`](crate::window_menu_keys::AMenuKey) hold the same six and
//! [`AnEdgeKey::as_a_menu_key`] is the one place they meet. Two enumerations
//! rather than one shared enumeration called `AMenuKey`, because a name that
//! says *menu* on the edge's own key would be a small lie in every match arm
//! that reads it.
//!
//! # Where the focus ring is while the menu is open
//!
//! On the Menu control, and the menu's own focused row is a second thing a
//! reader is told — `crate::window_menu_reading` carries it. That is how every
//! menu a person has used behaves: the thing you opened stays lit while you
//! are in what it opened, and Escape puts you back on it rather than nowhere.
//!
//! # What this still does not feed
//!
//! `Revealing`'s fourth condition, `a_drag`. Pressing Move answers
//! [`WhatTheEdgeKeyDid::ItRuns`] and the caller decides, exactly as a press
//! does — and nothing in this repository begins a keyboard move yet, so there
//! is no drag for this file to report. Recorded rather than faked: a
//! `the_edge_is_being_dragged` here that nothing ever called would be the
//! unfed half this file exists to remove.

use alo_dock::revealing::{AMenu, TheKeyboard};

use crate::window_edge::{OnTheEdge, WindowEdge};
use crate::window_menu::{WhatARowDoes, WindowMenu};
use crate::window_menu_keys::{
    AMenuKey, Backwards, Forwards, TheKeyboardInTheMenu, WhatTheKeyDid, Which,
};

/// A key pressed while the keyboard is on a window's edge.
///
/// Named by what a person means rather than by a keysym, for
/// `crate::window_menu_keys`' reason: the chord is the host's to map.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnEdgeKey {
    /// The next control along, or the next row down.
    Next,
    /// The previous one.
    Previous,
    /// The first.
    First,
    /// The last.
    Last,
    /// Press what has the keyboard. Enter, or Space.
    Press,
    /// Leave: close an open menu, or give the edge up altogether. Escape.
    Leave,
}

impl AnEdgeKey {
    /// The same key, for the menu.
    #[must_use]
    pub const fn as_a_menu_key(self) -> AMenuKey {
        match self {
            Self::Next => AMenuKey::Next,
            Self::Previous => AMenuKey::Previous,
            Self::First => AMenuKey::First,
            Self::Last => AMenuKey::Last,
            Self::Press => AMenuKey::Run,
            Self::Leave => AMenuKey::Close,
        }
    }
}

/// What a key press did.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhatTheEdgeKeyDid {
    /// The keyboard is now on this control of the edge.
    TheKeyboardIsOn(OnTheEdge),
    /// The menu opened, with the keyboard on no row yet.
    TheMenuOpened,
    /// The keyboard is now on this row of the open menu.
    TheKeyboardIsOnARow(WhatARowDoes),
    /// This control of the edge was pressed.
    ///
    /// **The control, not an action.** `Move` has no
    /// [`alo_shortcuts::Action`] and the caller decides what a press means,
    /// which is the same arrangement `crate::window_menu_keys` makes for the
    /// menu's rows and for the same recorded reason.
    ItRuns(OnTheEdge),
    /// This row of the menu was pressed. The menu closed and the keyboard is
    /// back on the control that opened it.
    ARowRuns(WhatARowDoes),
    /// The menu closed with nothing run, and the keyboard is back on the
    /// control that opened it.
    TheMenuClosed,
    /// The keyboard left the edge. **Nothing holds the edge revealed now** —
    /// the caller's next `Revealing` must hear that, which is what
    /// [`TheKeyboardOnTheEdge::whether_the_keyboard_holds_it`] answers.
    TheKeyboardLeft,
    /// Nothing happened.
    Nothing,
}

/// Where the keyboard is on one window's edge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TheKeyboardOnTheEdge {
    /// Not on this edge at all, which is where every edge starts.
    #[default]
    Nowhere,
    /// On one of the edge's own controls.
    OnAControl(OnTheEdge),
    /// Inside the menu that control opened.
    InTheMenu(TheKeyboardInTheMenu),
}

impl TheKeyboardOnTheEdge {
    /// The control the painter draws a focus ring on.
    ///
    /// The value `crate::Server::what_this_edge_is_pointing_at` takes as
    /// `focused`, which until now was [`None`] at every call site.
    ///
    /// **The trigger stays lit while its menu is open**, which is this file's
    /// header and is why the menu case answers a control rather than nothing.
    #[must_use]
    pub const fn what_the_painter_shows(self) -> Option<OnTheEdge> {
        match self {
            Self::Nowhere => None,
            Self::OnAControl(control) => Some(control),
            Self::InTheMenu(menu) => Some(menu.trigger()),
        }
    }

    /// The open menu, where there is one, for the painter and for a reader.
    #[must_use]
    pub const fn the_menu(self) -> Option<TheKeyboardInTheMenu> {
        match self {
            Self::Nowhere | Self::OnAControl(_) => None,
            Self::InTheMenu(menu) => Some(menu),
        }
    }

    /// Whether the keyboard holds the edge revealed.
    ///
    /// *Keep the edge revealed during keyboard focus.* An edge that concealed
    /// under its own focus ring would leave a person who tabbed to it pressing
    /// a control they cannot see.
    #[must_use]
    pub const fn whether_the_keyboard_holds_it(self) -> TheKeyboard {
        match self {
            Self::Nowhere => TheKeyboard::IsElsewhere,
            Self::OnAControl(_) | Self::InTheMenu(_) => TheKeyboard::IsOnIt,
        }
    }

    /// Whether an open menu holds the edge revealed.
    #[must_use]
    pub const fn whether_a_menu_holds_it(self) -> AMenu {
        match self {
            Self::Nowhere | Self::OnAControl(_) => AMenu::IsNot,
            Self::InTheMenu(_) => AMenu::IsOpen,
        }
    }

    /// The keyboard arrives on this edge, on its first control.
    ///
    /// **The first, not none.** This is a person tabbing *to* the edge, which
    /// is a deliberate arrival and must land somewhere — unlike a menu opened
    /// by pointer, where a ring on the first row would be a highlight nobody
    /// asked for. [`WhatTheEdgeKeyDid::Nothing`] for an edge with no controls,
    /// which is every concealed one: `edge_of` lays a concealed edge out with
    /// none, so arriving must reveal it first.
    pub fn reached(&mut self, edge: &WindowEdge) -> WhatTheEdgeKeyDid {
        match edge.controls.first() {
            Some(control) => {
                *self = Self::OnAControl(control.does);
                WhatTheEdgeKeyDid::TheKeyboardIsOn(control.does)
            }
            None => WhatTheEdgeKeyDid::Nothing,
        }
    }

    /// What this key does.
    ///
    /// `menu` is the laid-out menu, which the caller has because it draws it.
    /// [`None`] is honest for a window whose menu cannot be laid out, and then
    /// a key pressed inside a menu answers [`WhatTheEdgeKeyDid::Nothing`]
    /// rather than guessing at rows.
    pub fn told(
        &mut self,
        key: AnEdgeKey,
        edge: &WindowEdge,
        menu: Option<&WindowMenu>,
    ) -> WhatTheEdgeKeyDid {
        match *self {
            Self::Nowhere => WhatTheEdgeKeyDid::Nothing,
            Self::InTheMenu(keyboard) => self.in_the_menu(key, keyboard, menu),
            Self::OnAControl(control) => self.on_a_control(key, control, edge),
        }
    }

    /// A key pressed while the keyboard is on one of the edge's controls.
    fn on_a_control(
        &mut self,
        key: AnEdgeKey,
        control: OnTheEdge,
        edge: &WindowEdge,
    ) -> WhatTheEdgeKeyDid {
        let along: Vec<OnTheEdge> = edge.controls.iter().map(|it| it.does).collect();
        match key {
            // **Leaving works from anywhere**, for the reason Escape works in
            // a menu on a window that allows nothing: a person who can reach
            // the edge must always be able to leave it.
            AnEdgeKey::Leave => {
                *self = Self::Nowhere;
                WhatTheEdgeKeyDid::TheKeyboardLeft
            }
            AnEdgeKey::Press => match control {
                // **The Menu control opens the menu**, which is the thing
                // nothing could do: the menu existed, was laid out, had a
                // pointer hit test, and no road led into it.
                OnTheEdge::Menu => {
                    *self = Self::InTheMenu(TheKeyboardInTheMenu::opened_from(OnTheEdge::Menu));
                    WhatTheEdgeKeyDid::TheMenuOpened
                }
                OnTheEdge::Move | OnTheEdge::Minimise | OnTheEdge::Maximise | OnTheEdge::Close => {
                    WhatTheEdgeKeyDid::ItRuns(control)
                }
            },
            AnEdgeKey::First => self.moved_to(along.first().copied()),
            AnEdgeKey::Last => self.moved_to(along.last().copied()),
            AnEdgeKey::Next => self.moved_to(step(&along, control, Forwards)),
            AnEdgeKey::Previous => self.moved_to(step(&along, control, Backwards)),
        }
    }

    /// A key pressed while the keyboard is inside the open menu.
    fn in_the_menu(
        &mut self,
        key: AnEdgeKey,
        mut keyboard: TheKeyboardInTheMenu,
        menu: Option<&WindowMenu>,
    ) -> WhatTheEdgeKeyDid {
        let Some(menu) = menu else {
            return WhatTheEdgeKeyDid::Nothing;
        };
        match keyboard.told(key.as_a_menu_key(), menu) {
            WhatTheKeyDid::TheKeyboardIsOn(row) => {
                *self = Self::InTheMenu(keyboard);
                WhatTheEdgeKeyDid::TheKeyboardIsOnARow(row)
            }
            // **Focus returns to the trigger, and this is where that promise
            // is kept.** `window_menu_keys` names the control; this is the
            // file that puts the keyboard back on it.
            WhatTheKeyDid::ItRuns {
                does,
                focus_goes_back_to,
            } => {
                *self = Self::OnAControl(focus_goes_back_to);
                WhatTheEdgeKeyDid::ARowRuns(does)
            }
            WhatTheKeyDid::ItCloses { focus_goes_back_to } => {
                *self = Self::OnAControl(focus_goes_back_to);
                WhatTheEdgeKeyDid::TheMenuClosed
            }
            // The menu stayed open with the keyboard where it was, so this
            // stays as it is rather than writing back an unchanged value.
            WhatTheKeyDid::Nothing => WhatTheEdgeKeyDid::Nothing,
        }
    }

    /// Put the keyboard on this control, where there was one to move to.
    fn moved_to(&mut self, control: Option<OnTheEdge>) -> WhatTheEdgeKeyDid {
        match control {
            Some(control) => {
                *self = Self::OnAControl(control);
                WhatTheEdgeKeyDid::TheKeyboardIsOn(control)
            }
            None => WhatTheEdgeKeyDid::Nothing,
        }
    }
}

/// The next control in this direction, wrapping.
///
/// Wrapping for `crate::window_menu_keys::step`'s reason: three or four
/// controls in a strip, where Next from Close reaching the first is what a
/// person expects of a short row.
fn step(along: &[OnTheEdge], from: OnTheEdge, which: Which) -> Option<OnTheEdge> {
    if along.is_empty() {
        return None;
    }
    // A control that is no longer on this edge — the window was resized across
    // a breakpoint under the keyboard — starts over rather than guessing which
    // end it was nearer. The same decision the menu makes for a row that
    // stopped being usable.
    let Some(at) = along.iter().position(|it| *it == from) else {
        return along.first().copied();
    };
    let next = match which {
        Forwards => (at + 1) % along.len(),
        Backwards => (at + along.len() - 1) % along.len(),
    };
    along.get(next).copied()
}

#[cfg(test)]
#[path = "window_edge_keys_tests.rs"]
mod tests;
