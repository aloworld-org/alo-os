//! **What keeps a Dock control's name showing, and what dismisses it.**
//!
//! WCAG 1.4.13 asks two things of content that appears on hover or focus, and
//! `crate::dock_tooltip` met neither until 2026-10-29:
//!
//! - **Hoverable.** A person can move the pointer onto the name without it
//!   going. It stays while the pointer is on the control, on the name, or in
//!   the gap between the two ([`crate::dock_tooltip::TheNameAsDrawn::reach`]),
//!   because a name that vanishes as somebody moves to read it is a name only a
//!   steady hand can read — and a magnifier user moves the pointer to see.
//! - **Dismissible.** Escape takes the name away **without moving anything**:
//!   not the pointer, not the keyboard's place on the bar, not which area has
//!   the keyboard, and nothing else the key would have done. The name stays
//!   away while the same control is asked about, and comes back once the
//!   pointer or the keyboard has moved to something else and back.
//!
//! # Why the server decides
//!
//! Both turn on where the pointer and the keyboard are between frames, and
//! those are the seat's. The draw records where it put a name
//! (`Server::the_docks_name_was_drawn`), which is how a pointer over the name
//! can be told from a pointer over the canvas: outside a frame nothing else
//! knows where the name is.
//!
//! # Escape is taken only when there is a name to dismiss
//!
//! `crate::a_key_reaches_the_docks_surfaces` gives the reason in full: a shell
//! that takes Escape is a shell where nothing can be cancelled. So Escape's
//! press is taken here only while a name is drawn, and only Escape, and only
//! its press. A second Escape, with the name gone, goes wherever it went
//! before. The lane asks after every surface that holds the keyboard
//! (`crate::direct_desktop`): Settings, the market, the Bar and a task window
//! are in front of the Dock and a person pressing Escape at one is cancelling
//! that.

use smithay::backend::input::KeyState;

use crate::dock_tooltip::TheNameAsDrawn;
use crate::{InputUpdate, Server};

/// Escape, in evdev's numbering: the code `crate::a_key_reaches_the_docks_surfaces`
/// takes, which is in the same place on every layout.
const ESCAPE: u32 = 1;

impl Server {
    /// Which control on `display`'s Dock the pointer is on, counted as
    /// `crate::the_bars_pressable_slots` counts.
    ///
    /// **The same hit test a press uses** (`Server::the_dock_takes_this_press`),
    /// on the same Dock as it was last drawn, so the name a person reads is the
    /// name of the control a click would press.
    #[must_use]
    pub(crate) fn the_slot_the_pointer_is_on(&self, display: &str) -> Option<usize> {
        let at = self.where_the_pointer_is_in_pixels()?;
        let dock = self.dock_as_drawn.get(display)?;
        let place = dock.places.at(dock.how_far_along(at)?)?;
        crate::the_bars_pressable_slots::which_one_is(&dock.places, place)
    }

    /// The control whose name the pointer asks for: the one it is on, or —
    /// when it is on no control — the one whose name it is over.
    ///
    /// The Dock is asked first, so a pointer that has moved onto a neighbour is
    /// on the neighbour even where the last name's reach covers part of it.
    fn the_name_the_pointer_holds(&self, display: &str) -> Option<usize> {
        if let Some(on) = self.the_slot_the_pointer_is_on(display) {
            return Some(on);
        }
        let at = self.where_the_pointer_is_in_pixels()?;
        let drawn = self.the_docks_name_as_drawn.get(display)?;
        drawn.holds(at).then_some(drawn.which)
    }

    /// The control whose name is asked for on `display`, dismissed or not: the
    /// pointer's first, as the window title's tooltip asks, else the
    /// keyboard's.
    fn the_docks_name_is_asked_for(&self, display: &str) -> Option<usize> {
        self.the_name_the_pointer_holds(display)
            .or(self.which_slot_on_the_bar)
    }

    /// The control whose name is shown beside `display`'s Dock, or [`None`] —
    /// for nothing asked about, and for a name Escape dismissed while that
    /// control is still the one asked about.
    #[must_use]
    pub(crate) fn the_docks_name_is_for(&self, display: &str) -> Option<usize> {
        let asked = self.the_docks_name_is_asked_for(display)?;
        if self.the_docks_name_was_dismissed.get(display) == Some(&asked) {
            return None;
        }
        Some(asked)
    }

    /// Record where this draw put `display`'s name, or that it drew none.
    ///
    /// **[`None`] removes the entry**, for `Server::the_docks_surface_was_drawn`'s
    /// reason: a reach left behind by a name that is gone would keep a name
    /// showing for a pointer over nothing.
    ///
    /// And **a dismissal ends here** once something else is asked about: the
    /// pointer has left the control and its name, or the keyboard has moved.
    /// Asked after the draw rather than at each event, because a draw is the one
    /// moment both the Dock and the name are known.
    pub(crate) fn the_docks_name_was_drawn(
        &mut self,
        display: &str,
        drawn: Option<TheNameAsDrawn>,
    ) {
        match drawn {
            Some(drawn) => {
                self.the_docks_name_as_drawn
                    .insert(display.to_owned(), drawn);
            }
            None => {
                self.the_docks_name_as_drawn.remove(display);
            }
        }
        if let Some(&dismissed) = self.the_docks_name_was_dismissed.get(display)
            && self.the_docks_name_is_asked_for(display) != Some(dismissed)
        {
            self.the_docks_name_was_dismissed.remove(display);
        }
    }

    /// Whether this key dismissed a name shown beside the Dock.
    ///
    /// **True only for Escape's press while a name is drawn**, and then every
    /// name drawn is dismissed and nothing else changes. Every other key, every
    /// release, and Escape with no name showing is false and goes on to
    /// wherever it went before.
    pub(crate) fn a_key_dismisses_the_docks_name(&mut self, code: u32, state: KeyState) -> bool {
        if code != ESCAPE || state != KeyState::Pressed || self.the_docks_name_as_drawn.is_empty() {
            return false;
        }
        let shown = std::mem::take(&mut self.the_docks_name_as_drawn);
        for (display, drawn) in shown {
            self.the_docks_name_was_dismissed
                .insert(display, drawn.which);
        }
        true
    }

    /// The same, for one seat update: whether it was Escape's press and it
    /// dismissed a name.
    ///
    /// An event libinput's translation refuses is not taken here, so the road
    /// it goes on next reports it as it always did.
    pub(crate) fn the_docks_name_takes_this(&mut self, update: &InputUpdate<'_>) -> bool {
        let InputUpdate::Event(event) = update else {
            return false;
        };
        let Ok(Some(crate::DirectSeatEvent::Key(key, _))) =
            crate::libinput_routing::translate(event)
        else {
            return false;
        };
        self.a_key_dismisses_the_docks_name(key.code, key.state)
    }
}

#[cfg(test)]
#[path = "the_docks_name_is_held_tests.rs"]
mod tests;
