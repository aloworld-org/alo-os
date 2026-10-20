//! That the divider is skipped, and that the count and the index agree.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use alo_dock::{Dock, places::WhatIsHere};

/// A bar on this edge holding `how_many` applications, as the shell lays it out.
///
/// **The edge is a parameter because the divider depends on it.**
/// `alo_dock::places` puts a rule between the applications and the commands on
/// every edge **but the bottom**, where ADR 0101 keeps the composer and *Show
/// all* with no rule — and `Dock::shipped()` is a bottom Dock. A test of the
/// divider that used the shipped edge would be a test with no divider in it.
fn a_bar(how_many: usize, edge: alo_dock::Edge) -> crate::where_the_dock_is::TheDocksPlaces {
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
    let on_the_dock = alo_dock::Holding::nothing().showing(&windows);
    let mut dock = Dock::shipped();
    dock.set_edge(edge);
    crate::where_the_dock_is::where_the_dock_is(&dock, (1920, 1080), &on_the_dock)
        .unwrap_or_else(|why| panic!("a {edge:?} Dock would not lay out: {why:?}"))
}

/// **The divider is not a slot anybody can land on**, and this is the test that
/// would fail if the filter were dropped from either of the two functions.
///
/// It is written as an inequality between two counts rather than a number,
/// because the number of slots on a shipped Dock is not this file's business
/// and would make it fail for a reason that is not its subject.
#[test]
fn the_divider_is_not_one_of_the_slots_the_keyboard_walks() {
    let bar = a_bar(3, alo_dock::Edge::Left);
    let every = bar.places.each().len();
    let dividers = bar
        .places
        .each()
        .iter()
        .filter(|place| matches!(place.what(), WhatIsHere::TheDivider))
        .count();

    // **The control.** Without a divider on the bar this test could not fail
    // however the filter were written, and a shipped Dock with applications on
    // it is specified to have one between them and the commands.
    assert!(
        dividers > 0,
        "this bar has no divider on it, so nothing here measures anything"
    );
    assert_eq!(
        crate::the_bars_pressable_slots::how_many(&bar.places),
        every - dividers,
        "the count includes something a press does not reach"
    );

    // **And every index lands on something pressable**, which is the half the
    // count cannot check: a count can be right while the nth is off by the
    // dividers that came before it.
    for which in 0..crate::the_bars_pressable_slots::how_many(&bar.places) {
        let place = crate::the_bars_pressable_slots::the_one_at(&bar.places, which)
            .unwrap_or_else(|| panic!("slot {which} is inside the count and is not there"));
        assert!(
            place.what().can_be_pressed(),
            "slot {which} is the divider, so the keyboard can land where a press does nothing"
        );
    }
}

/// **Past the end is [`None`]**, which is the Dock that lost an application
/// between the press that moved the keyboard and the frame that draws it.
#[test]
fn a_slot_past_the_end_is_not_there() {
    let bar = a_bar(2, alo_dock::Edge::Bottom);
    let how_many = crate::the_bars_pressable_slots::how_many(&bar.places);
    assert!(
        crate::the_bars_pressable_slots::the_one_at(&bar.places, how_many).is_none(),
        "a slot one past the last one answered with a place"
    );
}
