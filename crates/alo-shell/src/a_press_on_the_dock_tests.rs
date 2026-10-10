//! **What a press on the Dock does, on all four edges.**
//!
//! The table a compositor would otherwise be needed to exercise: a value in, a
//! value out, for every slot, both gaps and both ends of the bar.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported, and \
              a formatted panic names the edge it was on"
)]

use alo_dock::{Dock, Edge};

use super::{WhatAPressOnTheDockDoes, what_a_press_on_the_dock_does};
use crate::where_the_dock_is::{TheDocksPlaces, where_the_dock_is};

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
        for (which, place) in dock.places.each().iter().enumerate() {
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
            .last()
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
        let before = slots
            .get(slots.len().saturating_sub(2))
            .unwrap_or_else(|| panic!("{edge:?}: no slot before the control"));
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

/// **A bar with room for everything has no control to press.**
///
/// The control exists only when something is behind it, so on an ordinary Dock
/// every press is an application or nothing — and a press cannot turn over a
/// list that is not there.
#[test]
fn a_dock_that_fits_has_no_control_to_press() {
    for edge in Edge::EVERY {
        let dock = on_the_edge(edge, 4);
        assert!(dock.over.is_empty(), "{edge:?}: the premise");
        for place in dock.places.each() {
            let answer = what_a_press_on_the_dock_does(&dock, at(&dock, place.from_the_start()));
            assert_ne!(
                answer,
                WhatAPressOnTheDockDoes::TurnTheOverflowOver,
                "{edge:?}: a Dock with room for everything offered a control"
            );
        }
    }
}
