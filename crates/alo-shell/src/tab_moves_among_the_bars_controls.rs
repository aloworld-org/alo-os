//! Tab, while the Dock has the keyboard.
//!
//! `docs/design/the-shortcuts-and-the-edges.md`: *move among controls in the
//! focused area — Tab / Shift+Tab*, and *activate a focused control — Enter*.
//! F6 brings the keyboard to the Dock and lands it on the first slot; this is
//! how it moves from there, and what Enter does when it stops.
//!
//! # Which slots, and why the question is already answered
//!
//! `alo_dock::places::WhatIsHere::can_be_pressed` has said which slots a press
//! reaches since the bar was laid out — everything but the rule between the
//! applications and the commands, which is a line nobody can press. **Tab walks
//! exactly that**, rather than a list of its own: a keyboard that stopped
//! somewhere a pointer could not reach, or skipped something a pointer could,
//! would be two answers to *what is on this bar*.
//!
//! # It does not wrap, and that is the design rather than a shortcoming
//!
//! `alo_access::reaching::after_the_last` says focus is never trapped: *Tab
//! from the end of a surface arrives at its beginning*. **That is for a surface
//! a person is inside of.** The bar is one of five areas in a cycle, and F6 is
//! how a person leaves it — so Tab past the last slot would either trap them
//! on the bar or quietly do what F6 does, and the second is worse: a person
//! would arrive somewhere else without having asked to.
//!
//! So Tab stops at the ends. A person at the last slot who presses Tab again
//! stays there, and presses F6 when they mean to leave.

use crate::which_area_has_the_keyboard::AnArea;

/// Which way Tab was pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WhichWay {
    /// Tab: on to the next control.
    Onward,
    /// Shift+Tab: back to the one before.
    Back,
}

/// Where Tab goes among `how_many` controls, from the one at `from`.
///
/// **Stops at the ends** — see this module's header. Answers the same place
/// when there is nowhere to go, so a caller can tell *moved* from *did not*
/// without a second question.
#[must_use]
pub(crate) fn where_tab_goes(from: usize, pressed: WhichWay, how_many: usize) -> usize {
    let last = how_many.saturating_sub(1);
    match pressed {
        WhichWay::Onward => from.saturating_add(1).min(last),
        WhichWay::Back => from.saturating_sub(1),
    }
}

impl crate::Server {
    /// Move the keyboard along the bar, if it is on the bar.
    ///
    /// Answers whether Tab was taken. **It is not taken when the keyboard is
    /// elsewhere**, which is every ordinary moment: Tab belongs to the
    /// application a person is typing in, and a shell that took it whenever it
    /// felt like it would be a shell a person cannot fill in a form on.
    pub(crate) fn tab_moves_among_the_bars_controls(&mut self, pressed: WhichWay) -> bool {
        if self.which_area_has_the_keyboard != Some(AnArea::TheDock) {
            return false;
        }
        let how_many = self.how_many_controls_the_bar_has();
        if how_many == 0 {
            // A bar with nothing on it takes the key and moves nothing, rather
            // than passing it to an application the person is not looking at.
            return true;
        }
        let from = self.which_slot_on_the_bar.unwrap_or(0).min(how_many - 1);
        self.which_slot_on_the_bar = Some(where_tab_goes(from, pressed, how_many));
        true
    }

    /// Which way Tab was pressed, from the seat that knows what is held.
    ///
    /// **Through the seat rather than from a modifier this file tracks**: the
    /// seat's XKB state is the one place that knows whether Shift is down, and
    /// a second answer kept here would be a second answer.
    ///
    /// # Errors
    /// Whatever the seat refuses about the key it was handed.
    pub(crate) fn which_way_tab_was_pressed(
        &mut self,
        code: u32,
    ) -> Result<WhichWay, crate::InputError> {
        let shifted = self.the_docks_tab_key(code)?;
        Ok(if shifted {
            WhichWay::Back
        } else {
            WhichWay::Onward
        })
    }

    /// How many controls the bar has that a person can act on.
    ///
    /// **Asked of the laid-out bar**, which is where the divider is told from
    /// the icons, and recomputed rather than remembered: an application opening
    /// adds a slot, and a remembered count would leave Tab walking past the end
    /// or stopping short of a new icon.
    ///
    /// # The fewest across every display, and why that is the safe answer
    ///
    /// The Dock is laid out per display and the counts can differ: a narrow
    /// screen fits fewer applications before the rest go into the overflow, so
    /// its bar has fewer slots. **Which display the keyboard is on is not
    /// something this shell tracks**, and until it does, taking the fewest is
    /// the one answer that is wrong in a recoverable direction — Tab stops
    /// short of an icon on the wider screen, rather than stopping on a slot
    /// that is not drawn on the narrower one and leaving a person pressing
    /// Enter on nothing.
    ///
    /// A machine with one display — which is every machine this has been run
    /// on — has one count and the question does not arise. The gap is named
    /// here rather than left for whoever first plugs in a second screen.
    pub(crate) fn how_many_controls_the_bar_has(&self) -> usize {
        self.dock_as_drawn
            .values()
            .map(|places| crate::the_bars_pressable_slots::how_many(&places.places))
            .min()
            .unwrap_or(0)
    }
}

#[cfg(test)]
#[path = "tab_moves_among_the_bars_controls_tests.rs"]
mod tests;
