//! What a press on the Dock does.
//!
//! **Nothing pressed the Dock until this file.** Measured on 2026-10-10 across
//! the 2531 `.rs` files outside `alo-dock`: `alo_dock::clicking` had no caller,
//! `alo_dock::menu` had none, and `alo_dock::Places` was named twice, both in
//! `crate::dock_raster` and both for drawing — so `Places::at`, the hit test
//! that crate exists for, was never called by anything. The Dock was drawn and
//! was not pressable.
//!
//! # The decision is separated from the act, and that is the whole shape
//!
//! [`what_a_press_on_the_dock_does`] takes where the Dock is and how far along
//! the press landed, and returns what should happen. It touches nothing. The
//! `Server` then does it, and `crate::direct_desktop` does the parts that need a
//! desktop.
//!
//! That is not tidiness. A press is the one input a test cannot conjure without
//! a compositor, a seat and a frame, and a decision folded into
//! `Server::pointer_button` could only be tested by running one. Here the whole
//! table is a value in, a value out — so *which slot a press reached* and *what
//! that slot does* are both checkable without a display.
//!
//! # One press, one thing
//!
//! A press reaches exactly one slot or none. The gaps between slots and the room
//! at the bar's ends belong to nobody, which is `alo_dock::places`' own rule:
//! *pressing a gap does nothing, deliberately — a gap that reached its nearest
//! neighbour would mean a person aiming at the space between two applications
//! opening one of them.*

use alo_dock::places::WhatIsHere;
use alo_dock::window::AppId;

use crate::where_the_dock_is::TheDocksPlaces;

/// What a press on the Dock should do.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WhatAPressOnTheDockDoes {
    /// Nothing: the press was not on a slot, or was not on the Dock at all.
    Nothing,
    /// Open or close the list of applications the bar had no room for.
    ///
    /// **Both, from one control**, because a person who opened a list and
    /// changed their mind reaches for the thing they just pressed. A control
    /// that only opened would need a second way out for the commonest case.
    TurnTheOverflowOver,
    /// Go to this application.
    ///
    /// **Carried but not yet acted on**, and this variant says so rather than
    /// being left out. `alo_dock::clicking` is the behaviour — *click an app to
    /// return to where you last used it* — and it needs the windows and the
    /// canvas camera, which is its own change. Leaving the variant out would
    /// have made a press on an icon indistinguishable from a press on a gap, and
    /// then the hit test would have been written twice.
    GoTo(AppId),
}

/// What a press at this point on the screen does.
///
/// `None` from `how_far_along` — a press that is not on the band — is
/// [`WhatAPressOnTheDockDoes::Nothing`], and so is a press on a gap between
/// slots or on the room at the bar's ends.
pub(crate) fn what_a_press_on_the_dock_does(
    dock: &TheDocksPlaces,
    at: smithay::utils::Point<i32, smithay::utils::Physical>,
) -> WhatAPressOnTheDockDoes {
    let Some(along) = dock.how_far_along(at) else {
        return WhatAPressOnTheDockDoes::Nothing;
    };
    match dock.places.at(along).map(alo_dock::places::APlace::what) {
        None => WhatAPressOnTheDockDoes::Nothing,
        Some(WhatIsHere::TheOverflow) => WhatAPressOnTheDockDoes::TurnTheOverflowOver,
        Some(WhatIsHere::Application(app)) => WhatAPressOnTheDockDoes::GoTo(app.clone()),
    }
}

impl crate::Server {
    /// Record where this draw put one display's Dock.
    ///
    /// Called from the draw, because the draw is the only place that knows. The
    /// same shape as `Server::the_panel_was_drawn` and
    /// `Server::the_fixed_controls_were_drawn`: what the draw knew, carried to a
    /// question asked later.
    pub(crate) fn the_docks_slots_were_drawn(&mut self, display: &str, drawn: TheDocksPlaces) {
        self.dock_as_drawn.insert(display.to_owned(), drawn);
    }

    /// Whether the list of applications the bar had no room for is open.
    #[must_use]
    pub(crate) const fn the_overflow_is_open(&self) -> bool {
        self.the_overflow_is_open
    }

    /// Let the Dock answer this press, and say whether it did.
    ///
    /// **`true` means the Dock took it**, and the caller stops: a press that
    /// opened an application must not also land on whatever is behind the bar.
    ///
    /// Every display's Dock is asked, because the pointer's location is global
    /// and a person may have one on each. The first that claims the point wins,
    /// and no two can claim it — the bands are on different displays.
    pub(crate) fn the_dock_takes_this_press(
        &mut self,
        at: smithay::utils::Point<i32, smithay::utils::Physical>,
    ) -> bool {
        let answer = self
            .dock_as_drawn
            .values()
            .map(|dock| what_a_press_on_the_dock_does(dock, at))
            .find(|answer| *answer != WhatAPressOnTheDockDoes::Nothing);
        match answer {
            None | Some(WhatAPressOnTheDockDoes::Nothing) => false,
            Some(WhatAPressOnTheDockDoes::TurnTheOverflowOver) => {
                self.the_overflow_is_open = !self.the_overflow_is_open;
                true
            }
            // **Taken, and nothing done with it yet.** `alo_dock::clicking` is
            // the behaviour — *click an app to return to where you last used
            // it* — and it needs the windows and the canvas camera. Returning
            // `true` is still the right answer: the press was on the Dock, and
            // letting it fall through to the canvas would move the view when a
            // person aimed at an icon, which is worse than it doing nothing.
            Some(WhatAPressOnTheDockDoes::GoTo(_)) => true,
        }
    }

    /// Close the overflow list, if it is open.
    ///
    /// **A press anywhere else closes it**, which is what a person expects of
    /// every list on this machine and what `docs/design/the-alo-dock.md` says of
    /// the Dock's surfaces: *Escape closes whatever Dock surface is open.*
    pub(crate) const fn the_overflow_closes(&mut self) {
        self.the_overflow_is_open = false;
    }
}

#[cfg(test)]
#[path = "a_press_on_the_dock_tests.rs"]
mod tests;
