//! The chord that moves the keyboard from one part of the screen to the next.
//!
//! `alo_shortcuts::Action::TheNextArea` and `ThePreviousArea` are what the two
//! are **called**; this is what they **do** — the division
//! `alo_shortcuts::action`'s own header draws, and the shape
//! `crate::settings_command` already has for `⊞`+I.
//!
//! # Why this is the first thing the Dock's keyboard work needed
//!
//! The Dock's surfaces have taken the arrows, Enter and Escape since
//! 2026-10-11. **Nothing could give them the keyboard**: a person had to open
//! a surface with a pointer before any of it was reachable, which is the
//! opposite of what keyboard work is for. `docs/features.md`'s *the keyboard
//! reaches every part of the screen* is the promise, and F6 is the whole of
//! how a person starts.
//!
//! # An empty area is passed over rather than stopped at
//!
//! `crate::which_area_has_the_keyboard` holds the rule and the reason. What
//! this file adds is the answer to *has it anything*, which is a question only
//! the `Server` can answer and only for this moment.

use alo_shortcuts::{Action, Chord, Shortcuts};

use crate::which_area_has_the_keyboard::{AnArea, WhichWay, where_the_keyboard_goes};

impl crate::Server {
    /// Move the keyboard on or back, if this chord is one of the two.
    ///
    /// Answers whether the chord was taken. A chord that is not one of these
    /// is **not** taken, and goes the road it already went — this is a layer
    /// over the others rather than a gate in front of them.
    pub(crate) fn a_chord_moves_the_keyboard_between_areas(
        &mut self,
        shortcuts: &Shortcuts,
        chord: Chord,
    ) -> bool {
        let pressed = match shortcuts.action_for(chord) {
            Some(Action::TheNextArea) => WhichWay::Onward,
            Some(Action::ThePreviousArea) => WhichWay::Back,
            _ => return false,
        };
        let has_anything = |area: AnArea| self.has_anything_to_act_on(area);
        let now = where_the_keyboard_goes(self.which_area_has_the_keyboard, pressed, &has_anything);
        self.the_keyboard_is_now_in(now);
        true
    }

    /// Put the keyboard in this area, and leave the one it came from as it was
    /// found.
    ///
    /// **Arriving on the Dock lands on its first slot**, because a person who
    /// pressed F6 to reach it has asked to be somewhere rather than to be
    /// nowhere in particular — and a screen reader with nothing focused has
    /// nothing to read out. Leaving it forgets the slot: coming back is a new
    /// arrival, not a resumption, and remembering would put the keyboard on an
    /// icon that may not be there any more.
    pub(crate) fn the_keyboard_is_now_in(
        &mut self,
        area: crate::which_area_has_the_keyboard::TheFocusedArea,
    ) {
        self.which_area_has_the_keyboard = area;
        self.which_slot_on_the_bar = match area {
            Some(AnArea::TheDock) => Some(0),
            _ => None,
        };
    }

    /// **Escape leaves**, which is the last of the four keys
    /// `docs/features.md` promises with F6: the keyboard goes back to the work.
    ///
    /// Answers whether it left, so the caller takes the key only then. Only
    /// the bar itself is left this way — a surface the Dock opened is closed by
    /// Escape first (`WhatTheDockWasAskedFor::when_escape_is_pressed`), and
    /// with the keyboard not on the Dock Escape is the application's, which is
    /// the narrowness `crate::a_key_reaches_the_docks_surfaces` argues for.
    ///
    /// Until 2026-10-30 nothing but a second F6 left: the twelfth Demo 1 walk
    /// pressed Escape twice on the Dock and Enter still pressed its Settings
    /// button.
    pub(crate) fn escape_leaves_the_dock(&mut self) -> bool {
        if self.which_area_has_the_keyboard != Some(AnArea::TheDock)
            || self.the_dock_was_asked_for.is_anything()
        {
            return false;
        }
        self.the_keyboard_is_now_in(None);
        true
    }

    /// Whether this area has anything a person could act on **now**.
    ///
    /// Asked per press rather than remembered: a panel fills and empties while
    /// somebody works, and a remembered answer is a list of what was true when
    /// they last pressed F6.
    ///
    /// **Three of the four answer `false` today and say so here rather than
    /// being left out of the walk.** Notifications and the status area have no
    /// focusable controls in this shell yet, and the panel's own previews take
    /// no keyboard. Writing them as `false` keeps the cycle's shape complete —
    /// the areas exist, the walk knows them, and each becomes reachable in the
    /// change that gives it something to reach. The work is not here at all:
    /// it is the absence the walk returns to.
    pub(crate) fn has_anything_to_act_on(&self, area: AnArea) -> bool {
        match area {
            // The bar always has its own controls — the composer, Show all,
            // History and More apps — whether or not any application is open.
            AnArea::TheDock => true,
            AnArea::ThePanel | AnArea::Notifications | AnArea::TheStatusArea => false,
        }
    }
}

#[cfg(test)]
#[path = "a_chord_moves_the_keyboard_between_areas_tests.rs"]
mod tests;
