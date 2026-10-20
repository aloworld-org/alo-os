//! Which slots on the Dock's bar a person can land on, counted one way.
//!
//! The bar holds applications, the composer, *Show all*, History, *More apps*
//! and a divider between the applications and the commands. **The divider is
//! not one of them** — `alo_dock::places::WhatIsHere::can_be_pressed` says so,
//! and `alo_dock::places`' own rule is that a press on a gap does nothing
//! deliberately.
//!
//! # Why this is a file and not three `filter`s
//!
//! Three things count these slots and they have to agree exactly:
//! `Server::which_slot_on_the_bar` is an index into them, written by F6 and by
//! Tab, read by Enter to decide what to carry out, and now read by
//! `crate::dock_raster` to decide which slot the focus ring goes round.
//!
//! **If the ring counted differently from Enter, the control that lights up
//! would not be the control that runs.** `crate::window_control_input`'s header
//! records what one such divergence cost — only the keyboard was migrated, the
//! button stayed broken, and a keyboard test passed throughout. Counting in one
//! place means the three cannot drift apart; counting in three means nothing
//! would fail if they did.

use alo_dock::places::{APlace, Places};

/// How many slots on this bar a press reaches.
pub(crate) fn how_many(places: &Places) -> usize {
    pressable(places).count()
}

/// The slot the keyboard is on, counting only the ones a press reaches.
///
/// [`None`] when the bar has fewer slots than that — a Dock that lost an
/// application between the press that moved the keyboard and the frame that
/// draws it, which is ordinary and not an error.
pub(crate) fn the_one_at(places: &Places, which: usize) -> Option<&APlace> {
    pressable(places).nth(which)
}

/// Every slot a press reaches, in the order they sit along the bar.
fn pressable(places: &Places) -> impl Iterator<Item = &APlace> {
    places
        .each()
        .iter()
        .filter(|place| place.what().can_be_pressed())
}

#[cfg(test)]
#[path = "the_bars_pressable_slots_tests.rs"]
mod tests;
