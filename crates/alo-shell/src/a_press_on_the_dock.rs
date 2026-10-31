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

use alo_dock::a_dock_entry::ANativeAction;
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
    /// **Put the keyboard in the composer**, so the next key typed goes into
    /// the field rather than to whatever had the keyboard before.
    ///
    /// A variant of its own rather than [`Self::Nothing`], which is what it was
    /// until 2026-10-12: a press on the field was *taken and not acted on*,
    /// under a comment saying the field was a surface somebody had to build.
    /// Nothing in this repository had ever accepted a keystroke there, which
    /// left *the composer is the agent's only way in* a promise with no door
    /// behind it.
    TypeIntoTheComposer,
    /// **Lay every window on this Place out at once**, which is what the
    /// control is for: `alo_dock::places::WhatIsHere::ShowAll`'s own rustdoc
    /// says *the control that lays every open window out at once*, and
    /// `crate::canvas_show_all` is that behaviour, built and tested since
    /// before this control was drawn.
    ///
    /// **This closes a gap, and the gap was invisible because there are two
    /// Show-alls.** `Action::ShowAllOnTheCanvas` reaches the same canvas
    /// behaviour by chord and has since it was written, so searching this tree
    /// for *show all* finds a built, tested, called thing and reads as though
    /// the Dock's control were done too. It was not: a press on it answered
    /// [`Self::Nothing`], so the control a person can see did nothing while the
    /// chord nobody is told about worked.
    ///
    /// A variant of its own rather than [`Self::Do`], because an
    /// `ANativeAction` is *an entry a person pinned to their Dock* and this is
    /// one of the Dock's own fixed controls — a different thing, and the owner's
    /// ruling of 2026-10-12 is that each is dispatched through its own handler.
    ShowEveryWindow,
    /// Do this, which the compositor does itself.
    ///
    /// **A separate variant from [`Self::GoTo`]**, which is the owner's ruling
    /// of 2026-10-12 made into a type: *dispatch each through its appropriate
    /// handler.* A native entry carried as an application would reach the
    /// launch path, which would ask the bus to start an application that does
    /// not exist — and the failure would look like a bus problem rather than a
    /// category error.
    Do(ANativeAction),
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
        Some(what) => what_this_slot_does(what),
    }
}

/// What one slot of the bar does, whoever asked.
///
/// **Split from the hit test on 2026-10-18**, so that the keyboard reaches it
/// too: a press finds the slot by where the pointer is and Enter finds it by
/// which one the keyboard is on, and *what that slot means* must not be two
/// answers. It was one answer only because there was one caller — the bar had
/// no keyboard performer at all.
pub(crate) fn what_this_slot_does(what: &WhatIsHere) -> WhatAPressOnTheDockDoes {
    match what {
        WhatIsHere::TheOverflow => WhatAPressOnTheDockDoes::TurnTheOverflowOver,
        WhatIsHere::Application(app) => WhatAPressOnTheDockDoes::GoTo(app.clone()),
        WhatIsHere::ANativeAction(action) => WhatAPressOnTheDockDoes::Do(*action),
        // **Taken and not yet acted on**, like `GoTo` above and for the same
        // reason: each is a surface somebody has to build — *Show all* lays
        // every window out, and History opens what they did before. Returning
        // *taken* is still right: a press that fell through to the canvas would
        // move the view when a person aimed at a control.
        //
        // **The composer left this group on 2026-10-12** and has a variant of
        // its own, because the field it opens now exists.
        //
        // `TheDivider` cannot arrive here at all — `Places::at` refuses it, so
        // a press on the rule between the applications and the commands reaches
        // nothing, which is what a line a person cannot press should do.
        WhatIsHere::TheComposer => WhatAPressOnTheDockDoes::TypeIntoTheComposer,
        // **`ShowAll` is answered; `History` is not, and that is a gap rather
        // than a decision.**
        //
        // `WhatIsHere::ShowAll` says what it is — *the control that lays every
        // open window out at once* — so it was never unspecified, and
        // `crate::canvas_show_all` has been that behaviour all along. What was
        // missing was this arm. Recorded here because the reason it was missed
        // is worth keeping: **there are two Show-alls**, the chord
        // (`alo_shortcuts::Action::ShowAllOnTheCanvas`) and this control, and
        // the chord's road was built first. Every search for *show all* found
        // it.
        //
        // **History stays `Nothing`, and that is not the same kind of hole.**
        // Nothing in this repository records what a person did before — no
        // model, no store, no surface — so there is nothing for the control to
        // open. Answering *taken* is still right: a press that fell through to
        // the canvas would move the view when a person aimed at a control.
        //
        // `TheDivider` cannot arrive here at all — `Places::at` refuses it — so
        // it is kept in this arm only because the match must be exhaustive.
        WhatIsHere::ShowAll => WhatAPressOnTheDockDoes::ShowEveryWindow,
        WhatIsHere::History | WhatIsHere::TheDivider => WhatAPressOnTheDockDoes::Nothing,
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
    /// Several windows: the person chooses.
    ///
    /// **This said *not built* until 2026-10-19 and both of these were
    /// carried out twenty lines below it.** `Server::go_to_this_application`
    /// records the ask on `Server::the_dock_was_asked_for`, and the rows are
    /// worked out at every draw by `Server::what_the_dock_has_open` rather
    /// than kept, so a window that closes while the list is open leaves it.
    /// `crate::what_the_dock_has_open_tests` holds that surface's own table.
    ///
    /// What is still owed is not the picker: it is the **travelling**, which
    /// [`what_a_press_on_an_icon_does`] collapses into *focus it* because the
    /// windows it is handed carry placeholder geometry.
    OpenThePicker,
    /// Nothing open: start it.
    ///
    /// **This said *nothing here can start an application* until 2026-10-19,
    /// and the road had been there since the press was written.**
    /// `Server::go_to_this_application` puts the ask on
    /// `Server::asked_to_start`; `crate::an_application_is_started` sends it
    /// off the input path through the sender this machine was given.
    ///
    /// **The limit, as of 2026-10-31:** the shipped desktop's sender starts a
    /// person's ask over the bus where the entry says `DBusActivatable=true`
    /// and from the entry's own `Exec` line otherwise (`alo_launching`, wired
    /// in `alo-desktop`'s `starting_what_a_person_pressed`). An entry with
    /// neither, or whose `Exec` line the specification calls invalid, is not
    /// started, and that refusal is returned and said rather than worked
    /// around.
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

    /// Where this display's open surface was drawn, or that none was.
    ///
    /// **[`None`] removes the entry rather than leaving the last one**, which is
    /// what keeps this from going stale: the frame that closes a popover is the
    /// frame that forgets where it was. An entry kept would be a press taken by
    /// a surface no longer on the screen — and taken is the worst of the three
    /// answers, because the press would not reach the canvas either.
    pub(crate) fn the_docks_surface_was_drawn(
        &mut self,
        display: &str,
        drawn: Option<crate::where_the_docks_surface_is::TheSurfaceAsDrawn>,
    ) {
        match drawn {
            Some(drawn) => {
                self.the_docks_surface_as_drawn
                    .insert(display.to_owned(), drawn);
            }
            None => {
                self.the_docks_surface_as_drawn.remove(display);
            }
        }
    }

    /// Let the open surface answer this press, and say whether it did.
    ///
    /// **Asked before the band**, because the surface is drawn in front of it
    /// and a popover that opens over the bar must take a press that lands on
    /// both. `Dismiss` answers `false`, which is the right answer and not a
    /// gap: a press outside an open surface closes it **and** does the thing it
    /// landed on, which is what every list on this machine does.
    /// `crate::pointer` carries out the closing, on any press the Dock does not
    /// take, and did so before this surface could be pressed at all — so this
    /// returns the press rather than closing anything itself.
    ///
    /// `false` when nothing is open.
    fn the_open_surface_takes_this_press(
        &mut self,
        at: smithay::utils::Point<i32, smithay::utils::Physical>,
    ) -> bool {
        use crate::where_the_docks_surface_is::WhatAPressOnTheSurfaceDoes as Does;
        // Every display's surface is asked, for `the_dock_takes_this_press`'s
        // reason: the pointer's location is global. Only one can be open at a
        // time today, because one field holds what the Dock was asked for — but
        // reading them all costs nothing and does not assume that.
        let answer = self
            .the_docks_surface_as_drawn
            .values()
            .map(|drawn| {
                crate::where_the_docks_surface_is::what_a_press_on_the_surface_does(drawn, at)
            })
            .find(|answer| *answer != Does::Dismiss);
        match answer {
            // Nothing open, or outside every open surface. **Not closed here**,
            // although this is the dismissal case: `crate::pointer` already
            // closes whatever is open when the Dock does not take a press.
            // Closing it here as well would be one rule in two places, and the
            // second copy is the one that gets it wrong when the first changes.
            None | Some(Does::Dismiss) => false,
            Some(Does::TakenAndNothingDone) => true,
            Some(Does::Choose(which)) => {
                self.choose_from_the_open_surface(which);
                true
            }
        }
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

    /// How many slots every display's Dock has told this seat it drew.
    ///
    /// **Read by what the machine says about its own input**, because nought
    /// here and a Dock on the screen are the same picture and opposite faults.
    /// The slots are computed in the draw and recorded from it, so a seat that
    /// has none has either not drawn yet or has drawn without recording —
    /// and the second is a Dock a person cannot press.
    pub(crate) fn how_many_dock_slots_were_drawn(&self) -> usize {
        self.dock_as_drawn
            .values()
            .map(|dock| dock.places.each().len())
            .sum()
    }

    /// Do what the keyboard's own slot on the bar asks for, and say whether
    /// anything was done.
    ///
    /// **Enter, when the keyboard is on the bar and no Dock surface is open.**
    /// The pointer's road is [`Self::the_dock_takes_this_press`]; this is the
    /// same act reached the other way, through the one performer both share.
    ///
    /// **The index is counted the way Tab counts it** — only slots
    /// `WhatIsHere::can_be_pressed` answers for, so the rule between the
    /// applications and the commands is skipped by both. A keyboard that
    /// counted the divider would land one short of every icon after it, and
    /// nothing on the screen would say why.
    ///
    /// **The fewest across every display**, for `how_many_controls_the_bar_has`'s
    /// reason exactly: which display the keyboard is on is not something this
    /// shell tracks, and stopping short of an icon is recoverable where
    /// pressing a slot that is not drawn is not.
    pub(crate) fn the_keyboard_takes_the_slot_it_is_on(&mut self) -> bool {
        let Some(which) = self.which_slot_on_the_bar else {
            return false;
        };
        let answer = self
            .dock_as_drawn
            .values()
            .filter_map(|dock| {
                // **Counted by `crate::the_bars_pressable_slots` and not here**,
                // because `crate::dock_raster` draws the focus ring round the
                // same index and a second `filter` is how the control that
                // lights up stops being the control that runs.
                crate::the_bars_pressable_slots::the_one_at(&dock.places, which)
                    .map(|place| what_this_slot_does(place.what()))
            })
            .find(|answer| *answer != WhatAPressOnTheDockDoes::Nothing);
        self.carry_out_what_the_bar_was_asked_for(answer)
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
        // **The open surface first**, because it is drawn in front of the bar.
        // A press on a cell of *More apps* that also lands over the band must
        // open that application rather than whatever slot is underneath.
        if self.the_open_surface_takes_this_press(at) {
            return true;
        }
        let answer = self
            .dock_as_drawn
            .values()
            .map(|dock| what_a_press_on_the_dock_does(dock, at))
            .find(|answer| *answer != WhatAPressOnTheDockDoes::Nothing);
        self.carry_out_what_the_bar_was_asked_for(answer)
    }

    /// Carry out what a slot on the bar was asked for, and say whether
    /// anything was.
    ///
    /// **One performer for the pointer and the keyboard**, which is the rule
    /// `crate::the_keyboard_on_the_docks_surfaces` states for the Dock's open
    /// surfaces and which the bar itself did not keep. Until 2026-10-18 the
    /// bar had **no** keyboard performer at all:
    /// `Server::which_slot_on_the_bar` was written by F6 and by Tab and read
    /// by nothing but Tab, so a person could walk the Dock with the keyboard
    /// and press nothing on it. `docs/design/the-shortcuts-and-the-edges.md`
    /// says *activate a focused control — Enter*, and the market's entry is on
    /// that bar: with the pointer road broken on a booted machine, the Dock
    /// could not be reached at all.
    ///
    /// Split out rather than copied, because two performers is how a keyboard
    /// and a pointer come to do different things to one slot —
    /// `crate::window_control_input` records what that cost once: only the
    /// keyboard was migrated and the button stayed broken while a keyboard
    /// test passed. A screen reader's press is the third road, and comes here
    /// too (`crate::a_reader_presses`).
    pub(crate) fn carry_out_what_the_bar_was_asked_for(
        &mut self,
        answer: Option<WhatAPressOnTheDockDoes>,
    ) -> bool {
        match answer {
            None | Some(WhatAPressOnTheDockDoes::Nothing) => false,
            Some(WhatAPressOnTheDockDoes::TurnTheOverflowOver) => {
                use crate::what_the_dock_has_open::WhatTheDockWasAskedFor as Asked;
                self.the_dock_was_asked_for =
                    if matches!(self.the_dock_was_asked_for, Asked::MoreApps { .. }) {
                        Asked::Nothing
                    } else {
                        // **Opening the overview closes a window list**, which
                        // is what one field rather than two guarantees: there
                        // is no state in which both are open.
                        //
                        // **Nothing focused**, because a pointer opened it. The
                        // first arrow key focuses the first application; a ring
                        // drawn before any key is pressed would say the
                        // keyboard is somewhere it is not.
                        Asked::MoreApps { focused: None }
                    };
                true
            }
            Some(WhatAPressOnTheDockDoes::GoTo(app)) => {
                self.go_to_this_application(&app);
                true
            }
            Some(WhatAPressOnTheDockDoes::Do(action)) => {
                self.do_this_native_action(action);
                true
            }
            Some(WhatAPressOnTheDockDoes::ShowEveryWindow) => {
                // **The camera is discarded, and that is what the chord does
                // too.** `show_all_on_the_canvas` moves the view itself and
                // hands back where it moved to, for a caller that wants to say
                // so; `dispatch_canvas_command` ignores it the same way.
                //
                // **`None` is not a failure.** It means there was nothing on
                // this Place to fit, and pressing *Show all* on an empty Place
                // is an ordinary thing to do — the screen already shows there
                // is nothing there. The press is still taken, because a press
                // on a control must never reach the canvas underneath it.
                let _ = self.show_all_on_the_canvas();
                true
            }
            Some(WhatAPressOnTheDockDoes::TypeIntoTheComposer) => {
                // **What was typed is kept**, which is why this is not a
                // `TheComposer::empty()`: somebody who pressed the field, went
                // to read a document and came back is writing one request.
                self.the_composer.it_took_the_keyboard();
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
    /// gives every window a one-by-one patch at the origin, and
    /// `what_a_press_on_an_icon_does` hands over a one-by-one view to match, so
    /// `TheView::already_shows` compares a placeholder with a placeholder and
    /// always answers *yes, already shown*. **Travelling on that would be
    /// motion decided by a placeholder**, which is worse than not travelling —
    /// a view that jumps somewhere arbitrary is harder to recover from than one
    /// that stays put.
    ///
    /// # Two things are owed, and this comment used to name one
    ///
    /// It said *what is owed is the window's real patch reaching this builder*.
    /// **That is half of it, and a reader who did only that half would ship a
    /// bug rather than a feature.** Measured 2026-10-14:
    ///
    /// 1. **The real patch**, which the compositor has:
    ///    `Server::the_frames_on_the_plane` builds one per surface from
    ///    `window_buffer_origin` and `scene::geometry`, rounding outwards. So
    ///    this half is a wiring job.
    /// 2. **Which Place that patch is on**, which **nothing in `alo-dock` can
    ///    hold.** `alo_dock::Window` carries `at: Patch` and no place;
    ///    `alo_dock::TheView` carries one `Patch` and no place; `Patch` itself
    ///    is a corner and two extents. The crate does not depend on
    ///    `alo-canvas` at all, where `Frame` **does** carry a `Place`.
    ///
    /// Why the second half is not a detail: `already_shows` is
    /// `patch.wholly_inside(self.showing)`, a pure rectangle test. Given real
    /// geometry and no Place, a window on **another** Place at coordinates
    /// inside the current view reads as *already shown* and the Dock does
    /// nothing, while one on another Place outside the view reads as *travel
    /// there* and the shell pans **this** Place to coordinates where that
    /// window is not. Both are wrong, and the second moves a person's view to
    /// an empty region for a press they aimed at a window they can see.
    ///
    /// **The gap is already recorded where a reader of `alo-dock` meets it**,
    /// and was before this: `alo_dock::words::A_WINDOW_IS_OPEN`'s note says the
    /// design shows *Open · Work* and the Dock draws only *Open*, because
    /// `previews::Preview` *knows the patch a window sits on and not which
    /// Place that patch belongs to* — *the Place is owed*. The same missing
    /// noun blocks the picker's sentence and this travel, which is worth
    /// knowing because one change answers both.
    ///
    /// So what is owed is a **decision** before it is a wiring job: either
    /// `alo-dock` learns the canvas's `Place` — widening a published surface —
    /// or travel is decided in the compositor and the Dock stops answering
    /// `travels` at all. `docs/features.md` promises *travels to it, to the
    /// Place and position it already has*, and that sentence names both.
    pub(crate) fn go_to_this_application(&mut self, app: &alo_dock::AppId) {
        let windows = self.the_windows_the_dock_sees();
        let (id, back) = match what_a_press_on_an_icon_does(&windows, app) {
            // **Several windows: open the list and let the person say which.**
            // The rows are not built here — `Self::what_the_dock_has_open`
            // works them out at every draw from the windows there are then, so
            // a window that closes while the list is open leaves it.
            WhatAnIconDoes::OpenThePicker => {
                self.the_dock_was_asked_for =
                    crate::what_the_dock_has_open::WhatTheDockWasAskedFor::TheWindowsOf {
                        app: app.clone(),
                        focused: None,
                    };
                return;
            }
            // **Nothing open: it starts.** Recorded rather than done, because
            // `alo_portals::activated` waits for the bus to start the
            // application and this is the compositor's input loop —
            // `Server::asked_to_start` carries the whole reason.
            WhatAnIconDoes::Launch => {
                self.asked_to_start.push(app.clone());
                // **And the surfaces close**, as they do for a window: a person
                // who chose an application from the overview has finished with
                // the overview, whether or not the application answers.
                self.the_overflow_closes();
                return;
            }
            // A press that found no window, which happens when the icon's
            // application closed between the draw and the press.
            WhatAnIconDoes::Nothing => return,
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

    /// Carry out a native entry's action.
    ///
    /// **Done here and now, unlike a launch.** `WhatAnIconDoes::Launch` records
    /// an ask because `alo_portals::activated` waits for the bus to start an
    /// application and this is the compositor's input loop. Opening Settings
    /// waits for nothing: it is a window this compositor draws itself, and
    /// `Server::the_chord_does_what_it_names` already opens it on this same
    /// road when a person presses `⊞`+I.
    ///
    /// **The window and the places are taken out and put back**, for that
    /// method's reason exactly: `SettingsWindow::opened_by_hand` wants
    /// `&mut SettingsWindow` and `&SettingsPlaces` while this holds
    /// `&mut self`. Nothing in the call replaces either, so what is put back is
    /// what was taken.
    ///
    /// **A session nobody has told where settings are kept opens none**, and
    /// the press is still taken. The sign-in screen, the nested lane and every
    /// test are in that position, and a press that fell through to the canvas
    /// would move the view when a person aimed at an icon.
    pub(crate) fn do_this_native_action(&mut self, action: alo_dock::ANativeAction) {
        // **The surfaces close first**, as they do for a window and for a
        // launch: a person who chose something from the overview has finished
        // with the overview, whether or not this machine can carry it out.
        self.the_overflow_closes();
        match action {
            alo_dock::ANativeAction::Settings => {
                let Some(places) = self.settings_places.take() else {
                    return;
                };
                let mut window = std::mem::take(&mut self.settings);
                // `opened_by_hand` is the road a **person** takes, and the only
                // one this file knows: no agent, no grant, no approval. A press
                // on one's own Dock is that person.
                let _opened = window.opened_by_hand(&places, std::time::SystemTime::now());
                window.the_zones_are_those_of(self.the_formats());
                self.settings = window;
                self.settings_places = Some(places);
            }
            alo_dock::ANativeAction::Applications => self.the_market_opens(),
        }
    }

    /// Open the window of what a person can install.
    ///
    /// **This is the first production caller `crate::MarketWindow` has ever
    /// had.** The model landed with none, its raster landed with none, and
    /// `market: None` was passed at every site that built a frame — a window
    /// that laid itself out, refused correctly and could not be reached by any
    /// person. Five surfaces in this crate were found in that state in one day.
    ///
    /// **The catalogue is read on each open rather than held on the seat**, for
    /// `MarketWindow::opened`'s own reason: it takes ownership and `close`
    /// drops it, so holding a second copy would be two answers to *what can a
    /// person install* with nothing keeping them in step. It is a file this
    /// crate ships, parsed into eight entries; the cost is not what a person
    /// pressing an icon would notice.
    ///
    /// **A catalogue this machine cannot read opens no window, and the press is
    /// still taken.** There is nowhere here to say why: `do_this_native_action`
    /// is on the Dock's road and this crate's `desktop_source` guard holds that
    /// road to no sentence of its own. A press that fell through to the canvas
    /// would move the view when a person aimed at an icon, which is worse than
    /// a press that appears to do nothing. The refusal is
    /// `alo_software::NotCatalogued` and it is a fault in a file this
    /// repository ships, not in anything a person did — so it cannot happen on
    /// a machine whose tests ran, and `alo-software`'s own suite is what says
    /// so.
    fn the_market_opens(&mut self) {
        let Ok(catalogue) = alo_software::Catalogue::decided() else {
            return;
        };
        // **What the machine has, as somebody told this seat** — never read
        // here. Reading it runs the sandbox's own tool and waits for it, and
        // this is the compositor's input loop; `Server::what_this_machine_has`
        // carries the argument and what an untold seat costs.
        let already = self.what_this_machine_has.clone();
        self.market.opened(catalogue, already);
    }

    /// Close the overflow list, if it is open.
    ///
    /// **A press anywhere else closes it**, which is what a person expects of
    /// every list on this machine and what `docs/design/the-alo-dock.md` says of
    /// the Dock's surfaces: *Escape closes whatever Dock surface is open.*
    pub(crate) fn the_overflow_closes(&mut self) {
        self.the_dock_was_asked_for =
            crate::what_the_dock_has_open::WhatTheDockWasAskedFor::Nothing;
        // **And the search is over.** A person who closes the overview and
        // opens it again has started a new search: finding it still narrowed
        // by something they typed a minute ago would be a Dock that has
        // quietly hidden most of their applications, with the only clue a few
        // letters in a field they are not looking at.
        //
        // Cleared on close rather than on open, so that there is no moment at
        // which the surface is open holding text from last time.
        let _was = self.what_was_typed_into_more_apps.cleared();
    }
}

#[cfg(test)]
#[path = "a_press_on_the_dock_tests.rs"]
mod tests;
