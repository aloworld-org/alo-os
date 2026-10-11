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
        // **Taken and not yet acted on**, like `GoTo` above and for the same
        // reason: each is a surface somebody has to build — the composer opens
        // a field a person types into, *Show all* lays every window out, and
        // History opens what they did before. Returning *taken* is still right:
        // a press that fell through to the canvas would move the view when a
        // person aimed at a control.
        //
        // `TheDivider` cannot arrive here at all — `Places::at` refuses it, so
        // a press on the rule between the applications and the commands reaches
        // nothing, which is what a line a person cannot press should do.
        Some(
            WhatIsHere::TheComposer
            | WhatIsHere::ShowAll
            | WhatIsHere::History
            | WhatIsHere::TheDivider,
        ) => WhatAPressOnTheDockDoes::Nothing,
    }
}

/// Whether a window that is coming back has to be brought back first.
///
/// A named pair rather than a `bool` at the call site, for `alo_dock::Pinned`'s
/// reason exactly: *restore it* and *focus it* are both yes-or-no facts about
/// one window and they read alike.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BringItBack {
    /// It was put aside.
    Yes,
    /// It is on the canvas.
    No,
}

/// What pressing one application's icon does, as a value.
///
/// **This exists so the decision can be tested without a machine.** The press
/// itself cannot be: `Server::the_docks_slots_were_drawn` is called from
/// `crate::direct_desktop` and from nowhere else, so the slots a press is
/// classified against only exist on the lane that needs a login seat and real
/// devices. The nested lane draws a Dock and cannot press it, because
/// `submit_with_desktop` is handed no `Server` to record into.
///
/// Rather than leave the whole chain unmeasured, the part that is a decision is
/// a pure function over `alo_dock::Windows` and the part that needs hardware is
/// three lines underneath it. That is the same split
/// `crate::a_key_reaches_settings` describes for its own lane, and it names
/// which half is read and not measured instead of implying both are.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WhatAnIconDoes {
    /// This application has no windows the Dock knows of.
    ///
    /// Not the same as [`Self::Launch`]: it means the press found nothing,
    /// which happens if the icon's application closed between the draw and the
    /// press.
    Nothing,
    /// One window, on the canvas: focus it.
    Focus(alo_dock::WindowId),
    /// One window, put aside: bring it back, then focus it.
    BringItBackThenFocus(alo_dock::WindowId),
    /// Several windows: the person chooses. **Not built** — task 3.
    OpenThePicker,
    /// Nothing open: start it. **Not built** — nothing here can start an
    /// application.
    Launch,
}

/// Decide what pressing `app`'s icon does, given what is open.
///
/// `alo_dock::clicking` is the rule and this is the translation of its answer
/// into what a compositor can carry out. The two differ in one place and it is
/// deliberate: **`WhatAClickDoes::travels` is not carried across.**
///
/// `Server::the_windows_the_dock_sees` gives every window a one-by-one patch at
/// the origin because it cannot see canvas geometry, so `TheView::already_shows`
/// is comparing placeholders and `BringForward` against `TravelTo` is decided by
/// one. **Travelling on a placeholder would be motion chosen at random**, which
/// is worse than not travelling: a view that stays put is somewhere a person
/// already understands. What is owed is each window's real patch reaching that
/// builder, and until it does this collapses the two into *focus it*.
pub(crate) fn what_a_press_on_an_icon_does(
    windows: &alo_dock::Windows,
    app: &alo_dock::AppId,
) -> WhatAnIconDoes {
    // The placeholder view, handed over openly rather than hidden in a default.
    let Ok(everywhere) = alo_dock::Patch::of(alo_dock::Spot::at(0, 0), 1, 1) else {
        return WhatAnIconDoes::Nothing;
    };
    let did =
        alo_dock::clicking::what_a_click_does(windows, app, alo_dock::TheView::showing(everywhere));
    match did {
        alo_dock::clicking::WhatAClickDoes::ChooseAWindow => WhatAnIconDoes::OpenThePicker,
        alo_dock::clicking::WhatAClickDoes::OpenAWindowNearTheView => WhatAnIconDoes::Launch,
        other => match other.window() {
            None => WhatAnIconDoes::Nothing,
            Some(id) if other.brings_one_back() => WhatAnIconDoes::BringItBackThenFocus(id),
            Some(id) => WhatAnIconDoes::Focus(id),
        },
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
    pub(crate) fn what_the_dock_has_open(&self) -> crate::WhatTheDockHasOpen {
        // **Worked out now, from the windows there are now**, rather than
        // remembered from when the list was opened. The person chose an
        // application; which windows it has is a fact about this moment, and
        // one that changes while its list is up.
        crate::what_the_dock_has_open::what_that_means_now(
            &self.the_dock_was_asked_for,
            &self.the_windows_the_dock_sees(),
            alo_appearance::TextScale::default(),
            // **The room is not decided here.** How much of the display a list
            // may take is the draw's, which knows where the bar is; this is the
            // widest bound there is, so the draw's own is the one that binds.
            alo_dock::Room::pixels(u32::MAX),
        )
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
                use crate::what_the_dock_has_open::WhatTheDockWasAskedFor as Asked;
                self.the_dock_was_asked_for =
                    if matches!(self.the_dock_was_asked_for, Asked::MoreApps) {
                        Asked::Nothing
                    } else {
                        // **Opening the overview closes a window list**, which
                        // is what one field rather than two guarantees: there
                        // is no state in which both are open.
                        Asked::MoreApps
                    };
                true
            }
            Some(WhatAPressOnTheDockDoes::GoTo(app)) => {
                self.go_to_this_application(&app);
                true
            }
        }
    }

    /// Carry out `alo_dock::clicking`'s answer for one application.
    ///
    /// **The press was already taken whatever happens here.** A press on an
    /// icon must not fall through to the canvas, which would move the view when
    /// a person aimed at a Dock, so this returns nothing and the caller answers
    /// `true` either way.
    ///
    /// # What is done, and what is named rather than done
    ///
    /// `what_a_click_does` gives five answers and this acts on three of them:
    ///
    /// - **it was put aside** — brought back, then focused. The whole of *click
    ///   an app to return to where you last used it* for the case a person
    ///   notices most, because the window is not on the screen at all.
    /// - **it is open** — focused. `activate_window` raises and focuses; this
    ///   does not move the window, which the rule forbids.
    /// - **several windows** — the picker, which does not exist. Task 3 of
    ///   `docs/autonomy/the-dock-more-apps-and-the-minimised-panel.md`.
    ///   `alo_dock::Previews` is the model and no surface draws it, so the
    ///   press is taken and nothing happens — which is better than falling back
    ///   on *focus the one used last*, the rule ADR 0103 replaced.
    /// - **nothing open** — launch, which needs something that can start an
    ///   application. Nothing in this workspace can.
    ///
    /// **The canvas does not travel.** `WhatAClickDoes::travels` says when it
    /// should and this ignores it, deliberately: `the_windows_the_dock_sees`
    /// gives every window a one-by-one patch at the origin, because it cannot
    /// see canvas geometry, so `TheView::already_shows` is comparing
    /// placeholders. **Travelling on that would be motion decided by a
    /// placeholder**, which is worse than not travelling — a view that jumps
    /// somewhere arbitrary is harder to recover from than one that stays put.
    /// What is owed is the window's real patch reaching this builder.
    fn go_to_this_application(&mut self, app: &alo_dock::AppId) {
        let windows = self.the_windows_the_dock_sees();
        let (id, back) = match what_a_press_on_an_icon_does(&windows, app) {
            // **Several windows: open the list and let the person say which.**
            // The rows are not built here — `Self::what_the_dock_has_open`
            // works them out at every draw from the windows there are then, so
            // a window that closes while the list is open leaves it.
            WhatAnIconDoes::OpenThePicker => {
                self.the_dock_was_asked_for =
                    crate::what_the_dock_has_open::WhatTheDockWasAskedFor::TheWindowsOf(
                        app.clone(),
                    );
                return;
            }
            // `Launch` is named above: nothing here can start an application.
            // `Nothing` is a press that found no window, which happens when the
            // icon's application closed between the draw and the press.
            WhatAnIconDoes::Nothing | WhatAnIconDoes::Launch => return,
            WhatAnIconDoes::Focus(id) => (id, BringItBack::No),
            WhatAnIconDoes::BringItBackThenFocus(id) => (id, BringItBack::Yes),
        };
        // **Going to a window closes whatever the Dock had open**, which is the
        // owner's *dismisses on … completed selection*. Done before the window
        // is reached rather than after, so a refusal below does not leave a
        // list open over the application a person just asked for.
        self.the_overflow_closes();
        // **Looked up now, not remembered from the draw.** Ids are the window
        // list's own indices and a window closing renumbers the rest, so a
        // surface kept since the frame could be a different window by the time
        // a finger lands.
        let Some(surface) = self.the_window_numbered(id).cloned() else {
            return;
        };
        if matches!(back, BringItBack::Yes) {
            // A window that will not come back is not then focused: focusing a
            // hidden window would leave a person with the keyboard somewhere
            // they cannot see.
            if self.set_window_minimized(&surface, false).is_err() {
                return;
            }
        }
        // A refusal here is a window that went away between the lookup and now,
        // which is an ordinary race and not something to end a session over.
        let _ = self.activate_window(&surface);
    }

    /// Close the overflow list, if it is open.
    ///
    /// **A press anywhere else closes it**, which is what a person expects of
    /// every list on this machine and what `docs/design/the-alo-dock.md` says of
    /// the Dock's surfaces: *Escape closes whatever Dock surface is open.*
    pub(crate) fn the_overflow_closes(&mut self) {
        self.the_dock_was_asked_for =
            crate::what_the_dock_has_open::WhatTheDockWasAskedFor::Nothing;
    }
}

#[cfg(test)]
#[path = "a_press_on_the_dock_tests.rs"]
mod tests;
