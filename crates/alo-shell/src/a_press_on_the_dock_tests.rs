//! **What a press on the Dock does, on all four edges.**
//!
//! The table a compositor would otherwise be needed to exercise: a value in, a
//! value out, for every slot, both gaps and both ends of the bar.

#![expect(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported, and \
              a formatted panic names the edge it was on"
)]

use std::os::unix::fs::PermissionsExt as _;

use alo_dock::{Dock, Edge};

use super::{WhatAPressOnTheDockDoes, what_a_press_on_the_dock_does};
use crate::Server;
use crate::where_the_dock_is::{TheDocksPlaces, where_the_dock_is};

/// A session on this machine's seat, for the one test that needs a `Server`.
///
/// The same shape as `crate::a_key_reaches_settings_tests::a_session` and for
/// the same reason: a `Server` needs a private runtime directory of its own, and
/// a test that shared one with another test would fail for a reason that has
/// nothing to do with what it measures.
fn a_session(what: &str) -> (tempfile::TempDir, Server) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind(runtime.path(), what).expect("a display");
    (runtime, server)
}

/// This many applications on a Dock, built the way `alo-dock` would hand them
/// over rather than assembled by hand.
fn on_the_dock(how_many: usize) -> Vec<alo_dock::OnTheDock> {
    let patch = alo_dock::Patch::of(alo_dock::Spot::at(0, 0), 1, 1).expect("one by one is a patch");
    let mut windows = alo_dock::Windows::none();
    for number in 0..how_many {
        let named = format!("app-{number}");
        windows.opened(alo_dock::Window::of(
            alo_dock::WindowId::numbered(number as u64),
            Some(alo_dock::AppId::named(&named).expect("a named application")),
            &named,
            patch,
            alo_dock::HowItSits::OnTheCanvas,
        ));
    }
    alo_dock::Holding::nothing().showing(&windows)
}

/// The Dock on this edge of a 1920 × 1080 display, holding `how_many`.
fn on_the_edge(edge: Edge, how_many: usize) -> TheDocksPlaces {
    let mut dock = Dock::shipped();
    dock.set_edge(edge);
    where_the_dock_is(&dock, (1920, 1080), &on_the_dock(how_many))
        .unwrap_or_else(|why| panic!("{edge:?} would not lay out: {why:?}"))
}

/// A point on the screen, `along` units down the bar and across its middle.
fn at(dock: &TheDocksPlaces, along: u32) -> smithay::utils::Point<i32, smithay::utils::Physical> {
    let along = i32::try_from(along).expect("a place on a bar fits in an i32");
    let point = if dock.layout.edge().runs_across() {
        (
            dock.band.loc.x + along,
            dock.band.loc.y + dock.band.size.h / 2,
        )
    } else {
        (
            dock.band.loc.x + dock.band.size.w / 2,
            dock.band.loc.y + along,
        )
    };
    point.into()
}

/// **Pressing an application's icon names that application**, on every edge.
#[test]
fn a_press_on_an_icon_names_the_application_it_is() {
    for edge in Edge::EVERY {
        let dock = on_the_edge(edge, 5);
        for (which, place) in dock
            .places
            .each()
            .iter()
            .filter(|place| place.app().is_some())
            .enumerate()
        {
            let middle = place.from_the_start() + place.across() / 2;
            let named = place
                .app()
                .unwrap_or_else(|| panic!("{edge:?}: slot {which} holds no application"))
                .clone();
            assert_eq!(
                what_a_press_on_the_dock_does(&dock, at(&dock, middle)),
                WhatAPressOnTheDockDoes::GoTo(named),
                "{edge:?}: pressing slot {which} did not name its application"
            );
        }
    }
}

/// **Pressing the overflow control turns the list over**, on every edge, and is
/// not a press on the application beside it.
///
/// The slot a person reaches for when they have the most open, and the one
/// where getting the hit test wrong costs most: the thing they want is behind
/// it.
#[test]
fn a_press_on_the_control_turns_the_overflow_over_on_every_edge() {
    for edge in Edge::EVERY {
        let dock = on_the_edge(edge, 200);
        let slots = dock.places.each();
        let control = slots
            .iter()
            .find(|place| place.is_the_overflow())
            .unwrap_or_else(|| panic!("{edge:?}: no slots at all"));
        assert!(control.is_the_overflow(), "{edge:?}: the premise");

        assert_eq!(
            what_a_press_on_the_dock_does(&dock, at(&dock, control.from_the_start() + 1)),
            WhatAPressOnTheDockDoes::TurnTheOverflowOver,
            "{edge:?}: the control's first pixel"
        );
        assert_eq!(
            what_a_press_on_the_dock_does(
                &dock,
                at(&dock, control.past_its_end().saturating_sub(1))
            ),
            WhatAPressOnTheDockDoes::TurnTheOverflowOver,
            "{edge:?}: the control's last pixel"
        );

        // And the application before it is still that application, so the
        // control has not swallowed its neighbour's slot.
        // **The last application, not the slot before the control's index.**
        // The composer and the commands sit after the control since
        // 2026-10-11, so counting back two from the end reaches one of them.
        let before = slots
            .iter()
            .rfind(|place| place.app().is_some())
            .unwrap_or_else(|| panic!("{edge:?}: no application before the control"));
        let answer = what_a_press_on_the_dock_does(&dock, at(&dock, before.from_the_start() + 1));
        assert!(
            matches!(answer, WhatAPressOnTheDockDoes::GoTo(_)),
            "{edge:?}: the slot before the control answered {answer:?}"
        );
    }
}

/// **A press on a gap, on the room at the ends, or off the bar does nothing.**
///
/// `alo_dock::places`' rule, carried through: *a gap that reached its nearest
/// neighbour would mean a person aiming at the space between two applications
/// opening one of them.*
#[test]
fn a_press_that_is_not_on_a_slot_does_nothing() {
    for edge in Edge::EVERY {
        let dock = on_the_edge(edge, 5);
        let slots = dock.places.each();

        // The room before the first slot.
        assert_eq!(
            what_a_press_on_the_dock_does(&dock, at(&dock, 0)),
            WhatAPressOnTheDockDoes::Nothing,
            "{edge:?}: the room at the bar's start"
        );

        // Every gap between one slot and the next.
        for pair in slots.windows(2) {
            let [before, _after] = pair else {
                unreachable!("windows(2) yields pairs")
            };
            assert_eq!(
                what_a_press_on_the_dock_does(&dock, at(&dock, before.past_its_end())),
                WhatAPressOnTheDockDoes::Nothing,
                "{edge:?}: the gap after a slot ending at {}",
                before.past_its_end()
            );
        }

        // The room after the last slot, and a point off the band entirely.
        let last = slots
            .last()
            .unwrap_or_else(|| panic!("{edge:?}: no slots at all"));
        assert_eq!(
            what_a_press_on_the_dock_does(&dock, at(&dock, last.past_its_end())),
            WhatAPressOnTheDockDoes::Nothing,
            "{edge:?}: the room at the bar's end"
        );
        assert_eq!(
            what_a_press_on_the_dock_does(&dock, (960, 540).into()),
            WhatAPressOnTheDockDoes::Nothing,
            "{edge:?}: the middle of the canvas"
        );

        // **The premise**: a press on a slot does something, so the five above
        // are not all `Nothing` because nothing ever is.
        let first = slots
            .first()
            .unwrap_or_else(|| panic!("{edge:?}: no slots at all"));
        assert_ne!(
            what_a_press_on_the_dock_does(&dock, at(&dock, first.from_the_start())),
            WhatAPressOnTheDockDoes::Nothing,
            "{edge:?}: nothing on this bar answers at all, so this test proves nothing"
        );
    }
}

/// **The control is pressable whether or not anything has overflowed.**
///
/// **This asserted the opposite until 2026-10-11**, which was a rule this lane
/// invented: the design's resting Dock has `More apps and canvas options` on it
/// with nothing in the overflow at all. So the control is always there and
/// always answers; what changes is whether the list it opens has anything in
/// it, and `dock_raster` already refuses to draw an empty one.
#[test]
fn the_control_answers_whether_or_not_anything_overflowed() {
    for edge in Edge::EVERY {
        let dock = on_the_edge(edge, 4);
        assert!(dock.over.is_empty(), "{edge:?}: the premise, nothing over");

        let control = dock
            .places
            .each()
            .last()
            .unwrap_or_else(|| panic!("{edge:?}: no slots at all"));
        assert!(
            control.is_the_overflow(),
            "{edge:?}: the last slot is not the control"
        );
        assert_eq!(
            what_a_press_on_the_dock_does(&dock, at(&dock, control.from_the_start() + 1)),
            WhatAPressOnTheDockDoes::TurnTheOverflowOver,
            "{edge:?}: a Dock that fits has no control to press"
        );
    }
}

/// A window of `app`, numbered `number`, sitting however it sits.
fn a_window(number: u64, app: &str, sits: alo_dock::HowItSits) -> alo_dock::Window {
    alo_dock::Window::of(
        alo_dock::WindowId::numbered(number),
        alo_dock::AppId::named(app).ok(),
        "",
        alo_dock::Patch::of(alo_dock::Spot::at(0, 0), 1, 1).unwrap(),
        sits,
    )
}

/// **Every answer pressing an icon can give**, in one case list.
///
/// The decision half of the press, which is the half a machine is not needed
/// for. See [`super::what_a_press_on_an_icon_does`] for why it is a separate
/// function at all: the slots a press is classified against are recorded only
/// by `crate::direct_desktop`, so the press itself cannot be driven here.
#[test]
fn what_pressing_an_icon_decides() {
    use super::WhatAnIconDoes;
    use alo_dock::HowItSits::{OnTheCanvas, PutAside};

    let app = alo_dock::AppId::named("an.app").unwrap();
    let cases: &[(&str, &[alo_dock::HowItSits], WhatAnIconDoes)] = &[
        ("nothing open", &[], WhatAnIconDoes::Launch),
        (
            "one on the canvas",
            &[OnTheCanvas],
            WhatAnIconDoes::Focus(alo_dock::WindowId::numbered(0)),
        ),
        (
            "one put aside",
            &[PutAside],
            WhatAnIconDoes::BringItBackThenFocus(alo_dock::WindowId::numbered(0)),
        ),
        (
            "two on the canvas",
            &[OnTheCanvas, OnTheCanvas],
            WhatAnIconDoes::OpenThePicker,
        ),
        (
            "one of each — a minimised window still counts as one of several",
            &[OnTheCanvas, PutAside],
            WhatAnIconDoes::OpenThePicker,
        ),
        (
            "both put aside",
            &[PutAside, PutAside],
            WhatAnIconDoes::OpenThePicker,
        ),
    ];

    for (what, sits, expected) in cases {
        let mut windows = alo_dock::Windows::none();
        for (number, how) in sits.iter().enumerate() {
            windows.opened(a_window(number as u64, "an.app", *how));
        }
        assert_eq!(
            super::what_a_press_on_an_icon_does(&windows, &app),
            *expected,
            "{what}"
        );
    }
}

/// **A window put aside is brought back before it is focused, never after.**
///
/// The one thing the decision carries that the rule does not spell out, and the
/// one a reordering would break silently: focusing a hidden window leaves the
/// keyboard somewhere a person cannot see, and the window then appearing does
/// not undo the moment they spent typing into nothing.
///
/// Held as a property of the *answer* because the applier that acts on it needs
/// a compositor — `BringItBackThenFocus` names the order in the variant, so a
/// caller cannot carry it out the other way round without saying so.
#[test]
fn coming_back_is_part_of_the_answer_rather_than_the_callers_idea() {
    use super::WhatAnIconDoes;

    let app = alo_dock::AppId::named("an.app").unwrap();
    let mut windows = alo_dock::Windows::none();
    windows.opened(a_window(0, "an.app", alo_dock::HowItSits::PutAside));

    let did = super::what_a_press_on_an_icon_does(&windows, &app);
    assert!(
        matches!(did, WhatAnIconDoes::BringItBackThenFocus(_)),
        "a window put aside was to be focused without being brought back: {did:?}"
    );
    assert!(
        !matches!(did, WhatAnIconDoes::Focus(_)),
        "and it is not the plain focus answer"
    );
}

/// **Another application's windows decide nothing here.**
///
/// `what_a_click_does` filters by application and this passes the whole list,
/// so a press on a one-window application while six others are open must still
/// focus that one window rather than open a picker.
#[test]
fn another_applications_windows_do_not_change_this_ones_answer() {
    use super::WhatAnIconDoes;

    let app = alo_dock::AppId::named("an.app").unwrap();
    let mut windows = alo_dock::Windows::none();
    windows.opened(a_window(0, "an.app", alo_dock::HowItSits::OnTheCanvas));
    for number in 1..7 {
        windows.opened(a_window(
            number,
            "another.app",
            alo_dock::HowItSits::OnTheCanvas,
        ));
    }

    assert_eq!(
        super::what_a_press_on_an_icon_does(&windows, &app),
        WhatAnIconDoes::Focus(alo_dock::WindowId::numbered(0))
    );
}

/// A Dock holding `how_many` applications and the Settings entry last, which is
/// the fresh arrangement the owner specified: *Files, Browser, Settings*.
fn with_settings_last(how_many: usize) -> Vec<alo_dock::OnTheDock> {
    let patch = alo_dock::Patch::of(alo_dock::Spot::at(0, 0), 1, 1).expect("one by one is a patch");
    let mut windows = alo_dock::Windows::none();
    let mut holding = alo_dock::Holding::nothing();
    for number in 0..how_many {
        let named = format!("app-{number}");
        let app = alo_dock::AppId::named(&named).expect("a named application");
        windows.opened(alo_dock::Window::of(
            alo_dock::WindowId::numbered(number as u64),
            Some(app.clone()),
            &named,
            patch,
            alo_dock::HowItSits::OnTheCanvas,
        ));
        holding.pin(app);
    }
    holding.pin_entry(alo_dock::ADockEntry::Native(
        alo_dock::ANativeAction::Settings,
    ));
    holding.showing(&windows)
}

/// **Pressing the Settings entry asks for the action, not for an application.**
///
/// The owner's ruling of 2026-10-12: *dispatch each through its appropriate
/// handler.* `GoTo` reaches the launch path, which asks the bus to start an
/// application — so a native entry carried as one would send `Activate` to a
/// bus name nothing answers on, and the failure would read as a bus problem
/// rather than as the category error it is.
#[test]
fn a_press_on_a_native_entry_asks_for_the_action() {
    for edge in Edge::EVERY {
        let mut dock = Dock::shipped();
        dock.set_edge(edge);
        let shown = with_settings_last(3);
        let dock = where_the_dock_is(&dock, (1920, 1080), &shown)
            .unwrap_or_else(|why| panic!("{edge:?} would not lay out: {why:?}"));

        let native: Vec<_> = dock
            .places
            .each()
            .iter()
            .filter(|place| place.native().is_some())
            .collect();
        assert_eq!(
            native.len(),
            1,
            "{edge:?}: the bar laid out {} native slots for one native entry",
            native.len()
        );
        let place = native
            .first()
            .unwrap_or_else(|| panic!("{edge:?}: the one native slot is there"));
        let middle = place.from_the_start() + place.across() / 2;

        assert_eq!(
            what_a_press_on_the_dock_does(&dock, at(&dock, middle)),
            WhatAPressOnTheDockDoes::Do(alo_dock::ANativeAction::Settings),
            "{edge:?}: pressing the Settings entry did not ask for the Settings action"
        );
    }
}

/// **And the applications beside it are still applications.**
///
/// Written as its own test because the dangerous failure is the quiet one: a
/// layout that gave every slot the native kind would pass the test above on its
/// own, and a person's Dock would open Settings whatever they pressed.
#[test]
fn the_applications_beside_a_native_entry_are_still_applications() {
    for edge in Edge::EVERY {
        let mut dock = Dock::shipped();
        dock.set_edge(edge);
        let shown = with_settings_last(3);
        let dock = where_the_dock_is(&dock, (1920, 1080), &shown)
            .unwrap_or_else(|why| panic!("{edge:?} would not lay out: {why:?}"));

        let applications: Vec<_> = dock
            .places
            .each()
            .iter()
            .filter(|place| place.app().is_some())
            .collect();
        assert_eq!(
            applications.len(),
            3,
            "{edge:?}: three applications and the Settings entry laid out {} application slots",
            applications.len()
        );
        for place in applications {
            let middle = place.from_the_start() + place.across() / 2;
            let named = place
                .app()
                .unwrap_or_else(|| panic!("{edge:?}: an application slot holds no application"))
                .clone();
            assert_eq!(
                what_a_press_on_the_dock_does(&dock, at(&dock, middle)),
                WhatAPressOnTheDockDoes::GoTo(named),
                "{edge:?}: an application beside the Settings entry stopped being one"
            );
        }
    }
}

/// **A press on the composer puts the keyboard in the field, on every edge.**
///
/// Until 2026-10-12 this slot answered `Nothing` — *taken and not acted on* —
/// because the field it would open did not exist. It does now, so the press has
/// an outcome of its own, and this asserts it at the slot's first and last
/// pixel rather than at its middle: a hit test that is one pixel out passes a
/// test aimed at the centre.
#[test]
fn a_press_on_the_composer_types_into_it_on_every_edge() {
    for edge in Edge::EVERY {
        let dock = on_the_edge(edge, 5);
        let slots = dock.places.each();
        let composer = slots
            .iter()
            .find(|place| matches!(place.what(), alo_dock::places::WhatIsHere::TheComposer))
            .unwrap_or_else(|| panic!("{edge:?}: the Dock has no composer"));

        assert_eq!(
            what_a_press_on_the_dock_does(&dock, at(&dock, composer.from_the_start())),
            WhatAPressOnTheDockDoes::TypeIntoTheComposer,
            "{edge:?}: the field's first pixel"
        );
        assert_eq!(
            what_a_press_on_the_dock_does(
                &dock,
                at(&dock, composer.past_its_end().saturating_sub(1))
            ),
            WhatAPressOnTheDockDoes::TypeIntoTheComposer,
            "{edge:?}: the field's last pixel"
        );
    }
}

/// **Show all answers on every edge, and on every edge it is really there.**
///
/// The `found` guard is the point of the loop rather than decoration: a Dock
/// that put no *Show all* on this edge would walk zero places, assert nothing,
/// and pass. `WhatIsHere::ShowAll`'s own rustdoc says the bottom Dock has it
/// and the edge Docks put History in its place, so *how many* differs by edge
/// and only *at least one somewhere* is safe to assert here.
#[test]
fn a_press_on_show_all_lays_every_window_out() {
    let mut edges_with_one = 0;
    for edge in Edge::EVERY {
        let dock = on_the_edge(edge, 5);
        let mut found = 0;
        for place in dock.places.each() {
            if matches!(place.what(), alo_dock::places::WhatIsHere::ShowAll) {
                found += 1;
                assert_eq!(
                    what_a_press_on_the_dock_does(&dock, at(&dock, place.from_the_start())),
                    WhatAPressOnTheDockDoes::ShowEveryWindow,
                    "{edge:?}: a press on Show all"
                );
                assert_eq!(
                    what_a_press_on_the_dock_does(
                        &dock,
                        at(&dock, place.past_its_end().saturating_sub(1))
                    ),
                    WhatAPressOnTheDockDoes::ShowEveryWindow,
                    "{edge:?}: Show all's last pixel"
                );
            }
        }
        if found > 0 {
            edges_with_one += 1;
        }
    }
    assert!(
        edges_with_one > 0,
        "no edge put a Show all on its Dock, so this test proves nothing"
    );
}

/// **History is still taken and not acted on**, so answering Show all did not
/// quietly give its neighbour an outcome too.
///
/// It is a different kind of hole and the assertion says which: nothing in this
/// repository records what a person did before, so there is nothing for the
/// control to open. The day something does, this test fails and names itself.
#[test]
fn history_is_still_taken_and_not_acted_on() {
    let mut edges_with_one = 0;
    for edge in Edge::EVERY {
        let dock = on_the_edge(edge, 5);
        for place in dock.places.each() {
            if matches!(place.what(), alo_dock::places::WhatIsHere::History) {
                edges_with_one += 1;
                assert_eq!(
                    what_a_press_on_the_dock_does(&dock, at(&dock, place.from_the_start())),
                    WhatAPressOnTheDockDoes::Nothing,
                    "{edge:?}: a press on History"
                );
            }
        }
    }
    assert!(
        edges_with_one > 0,
        "no edge put a History on its Dock, so this test proves nothing"
    );
}

/// **The divider is unpressable, and the reason is `Places::at` rather than the
/// match arm below it.**
///
/// Both halves are asserted, because only one of them is the mechanism.
/// `Places::at` finds a place that `holds(along)` **and** `can_be_pressed()`,
/// and `WhatIsHere::can_be_pressed` is `!matches!(self, Self::TheDivider)` — so
/// a press on the rule between the applications and the commands resolves to
/// `None` before any arm sees it. The arm that lists `TheDivider` beside
/// `History` exists for exhaustiveness and would answer the same thing; it is
/// not what makes this true.
///
/// **The first draft of this test asserted that no divider was walkable at
/// all** and failed with `left: 3`, because it counted the dividers in
/// `places.each()` — which lists every place, pressable or not — instead of
/// what a press resolves to. The two are different populations and the test
/// named neither; it is written out here because the failure read like a fault
/// in the code and was a fault in the measurement.
#[test]
fn a_press_where_the_divider_is_reaches_nothing_because_at_refuses_it() {
    let mut found = 0;
    for edge in Edge::EVERY {
        let dock = on_the_edge(edge, 5);
        for place in dock.places.each() {
            if !matches!(place.what(), alo_dock::places::WhatIsHere::TheDivider) {
                continue;
            }
            found += 1;
            let along = place.from_the_start();
            assert!(
                dock.places.at(along).is_none(),
                "{edge:?}: `Places::at` handed out a divider, which `can_be_pressed` forbids"
            );
            assert_eq!(
                what_a_press_on_the_dock_does(&dock, at(&dock, along)),
                WhatAPressOnTheDockDoes::Nothing,
                "{edge:?}: a press on the divider"
            );
        }
    }
    assert!(
        found > 0,
        "no edge drew a divider, so this test proves nothing"
    );
}

/// **The press is taken by the Dock rather than falling through to the canvas**,
/// which is the half of this change that `what_a_press_on_the_dock_does` alone
/// cannot show.
///
/// `the_dock_takes_this_press` answers `false` for
/// [`WhatAPressOnTheDockDoes::Nothing`] and `true` for every variant it acts on,
/// and its caller in `crate::pointer` uses that answer to decide whether the
/// canvas sees the press. So this assertion is exactly the behaviour a person
/// feels: **before this change a press on *Show all* moved the view**, because
/// the control answered `Nothing` and the canvas underneath took it.
///
/// **What this does not prove is that the canvas fitted anything.** A `Server`
/// built here has no output, so `show_all_on_the_canvas` answers `None` and
/// moves nothing — which is the refusal path, and is asserted as that. Nothing
/// in this repository presses the Dock through a real pointer with a display
/// behind it; `GoTo`, `Do` and `TypeIntoTheComposer` are all tested at this same
/// level, so the gap is the road's and not this control's.
#[test]
fn the_dock_takes_a_press_on_show_all() {
    let (_runtime, mut server) = a_session("a-press-on-show-all");
    let dock = on_the_edge(Edge::Bottom, 5);
    let show_all = dock
        .places
        .each()
        .iter()
        .find(|place| matches!(place.what(), alo_dock::places::WhatIsHere::ShowAll))
        .map(|place| place.from_the_start())
        .expect("the bottom Dock carries Show all");
    let where_it_is = at(&dock, show_all);
    let elsewhere = (0, 0).into();
    server.the_docks_slots_were_drawn("a-display", dock);

    assert!(
        server.the_dock_takes_this_press(where_it_is),
        "a press on Show all was not taken, so the canvas underneath would move"
    );
    assert!(
        !server.the_dock_takes_this_press(elsewhere),
        "a press in the corner was taken by the Dock, so this proves nothing about aim"
    );
    assert_eq!(
        server.show_all_on_the_canvas(),
        None,
        "a session with no output fitted something, so the refusal path above is untested"
    );
}

/// **A window far from the view is still only focused, never travelled to —
/// and this test is a canary rather than an approval.**
///
/// `what_a_press_on_an_icon_does` hands `alo_dock::clicking` a one-by-one view
/// at the origin, and `the_windows_the_dock_sees` gives every window a
/// one-by-one patch at the origin, so `TheView::already_shows` compares a
/// placeholder with a placeholder and always answers *already shown*. This
/// asserts that behaviour **so that wiring real geometry breaks it**, which is
/// the point: `Server::go_to_this_application`'s header says two things are
/// owed and only one of them is a wiring job. The other is that nothing in
/// `alo-dock` can hold **which Place** a patch is on, and given real geometry
/// without it, a window on another Place reads as *travel there* and the
/// compositor would pan **this** Place to coordinates where that window is not.
///
/// So when this test fails, the question to answer is not *how do I make it
/// pass* — it is whether the Place reached the Dock too.
///
/// The patch here is deliberately far away and large: if the answer depended on
/// geometry at all, this is the case that would differ from a window at the
/// origin, and it does not.
#[test]
fn a_window_far_from_the_view_is_focused_rather_than_travelled_to() {
    use super::WhatAnIconDoes;

    let app = alo_dock::AppId::named("far.away").expect("a named application");
    let far = alo_dock::Patch::of(alo_dock::Spot::at(100_000, 100_000), 800, 600)
        .expect("a patch far from the origin");
    let near =
        alo_dock::Patch::of(alo_dock::Spot::at(0, 0), 1, 1).expect("one by one at the origin");

    for (which, patch) in [("far away", far), ("at the origin", near)] {
        let mut windows = alo_dock::Windows::none();
        windows.opened(alo_dock::Window::of(
            alo_dock::WindowId::numbered(0),
            Some(app.clone()),
            "a window",
            patch,
            alo_dock::HowItSits::OnTheCanvas,
        ));
        assert_eq!(
            super::what_a_press_on_an_icon_does(&windows, &app),
            WhatAnIconDoes::Focus(alo_dock::WindowId::numbered(0)),
            "{which}: the answer depended on where the window is, which means the view is no \
             longer a placeholder — read `go_to_this_application`'s header before making this pass"
        );
    }
}

/// A Dock holding `how_many` applications and the Applications entry last.
fn with_the_market_last(how_many: usize) -> Vec<alo_dock::OnTheDock> {
    let patch = alo_dock::Patch::of(alo_dock::Spot::at(0, 0), 1, 1).expect("one by one is a patch");
    let mut windows = alo_dock::Windows::none();
    let mut holding = alo_dock::Holding::nothing();
    for number in 0..how_many {
        let named = format!("app-{number}");
        let app = alo_dock::AppId::named(&named).expect("a named application");
        windows.opened(alo_dock::Window::of(
            alo_dock::WindowId::numbered(number as u64),
            Some(app.clone()),
            &named,
            patch,
            alo_dock::HowItSits::OnTheCanvas,
        ));
        holding.pin(app);
    }
    holding.pin_entry(alo_dock::ADockEntry::Native(
        alo_dock::ANativeAction::Applications,
    ));
    holding.showing(&windows)
}

/// **Pressing the Applications entry asks for the market, not for Settings.**
///
/// Written beside the Settings test rather than folded into it: the two actions
/// read different words and reach different windows, and a layout that gave
/// either slot the other's action would pass a test that only knew *some*
/// native action was asked for.
#[test]
fn a_press_on_the_market_entry_asks_for_the_market() {
    for edge in Edge::EVERY {
        let mut dock = Dock::shipped();
        dock.set_edge(edge);
        let shown = with_the_market_last(3);
        let dock = where_the_dock_is(&dock, (1920, 1080), &shown)
            .unwrap_or_else(|why| panic!("{edge:?} would not lay out: {why:?}"));

        let place = dock
            .places
            .each()
            .iter()
            .find(|place| place.native().is_some())
            .unwrap_or_else(|| panic!("{edge:?}: the one native slot is there"));
        let middle = place.from_the_start() + place.across() / 2;

        assert_eq!(
            what_a_press_on_the_dock_does(&dock, at(&dock, middle)),
            WhatAPressOnTheDockDoes::Do(alo_dock::ANativeAction::Applications),
            "{edge:?}: pressing the Applications entry did not ask for the market"
        );
    }
}

/// **And carrying that action out opens the window.**
///
/// The one test on this road that needs a `Server`, because the window lives on
/// one. This is the assertion that `crate::MarketWindow`'s production mention
/// count is no longer zero: it was zero when the model landed, zero when its
/// raster landed, and a surface no person can reach is the fault five other
/// shell windows were found in on 2026-10-16.
#[test]
fn carrying_out_the_market_action_opens_the_window() {
    let (_runtime, mut server) = a_session("market-opens");

    assert!(
        !server.market.is_open(),
        "a fresh session has the market open"
    );
    server.do_this_native_action(alo_dock::ANativeAction::Applications);
    assert!(
        server.market.is_open(),
        "the Applications entry was pressed and no window opened"
    );
    // **And it is showing the decided catalogue**, not an empty window: a
    // window that opened on nothing would draw its furniture and no
    // applications, which is the failure a person would read as *there is
    // nothing to install*.
    let crate::MarketShows::Browsing(browsing) = server.market.shows() else {
        panic!("the window opened and is showing nothing");
    };
    assert!(
        browsing.how_many() >= 8,
        "the window opened on {} applications",
        browsing.how_many()
    );
}

/// **Pressing Settings does not open the market, and the other way round.**
///
/// Two one-line handlers beside each other is exactly where a copied `match`
/// arm goes wrong, and the symptom would be a person pressing Applications and
/// getting their settings — which looks like a layout fault rather than a
/// dispatch one.
#[test]
fn each_native_action_opens_only_its_own_window() {
    let (_runtime, mut server) = a_session("market-only-its-own");

    server.do_this_native_action(alo_dock::ANativeAction::Settings);
    assert!(
        !server.market.is_open(),
        "pressing Settings opened the market"
    );

    server.do_this_native_action(alo_dock::ANativeAction::Applications);
    assert!(server.market.is_open());
    // Settings cannot be asserted shut here for a reason worth saying: a
    // session nobody has told where settings are kept opens none, and this
    // one is in that position. What **is** asserted is that the market
    // action did not reach it, which is the question.
    assert!(
        !server.settings.is_open(),
        "pressing Applications opened Settings"
    );
}
