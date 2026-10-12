//! What a Dock is filled with, and the two empties that are not the same.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;

use alo_dock::AppId;
use alo_dock::pinning::{ARole, Discovery, ThePins};

use crate::Server;

/// A server on a socket of its own.
fn server(named: &str) -> (tempfile::TempDir, Server) {
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let bound = Server::bind(directory.path(), named).expect("a bound socket");
    (directory, bound)
}

/// An application the Dock will take.
fn app(name: &str) -> AppId {
    AppId::named(name).expect("a plain identifier is an application the Dock takes")
}

/// A completed reading filling both roles a scan can answer.
fn both_roles() -> Discovery {
    Discovery::Completed(vec![
        (ARole::Files, app("org.kde.dolphin")),
        (ARole::Browser, app("org.mozilla.firefox")),
    ])
}

/// **A fresh machine's Dock is pinned to what discovery found.**
///
/// The assertion is on the identifiers and their order, not on a count: a
/// filling that pinned two of something would satisfy a count and is exactly
/// the kind of wrong this is here to catch.
#[test]
fn a_completed_reading_fills_the_dock_with_what_it_found() {
    let (_directory, mut server) = server("a-completed-reading");

    server.its_dock_is_filled_from(&both_roles());

    let pinned: Vec<&str> = server
        .what_the_dock_is_pinned_to()
        .each_pinned()
        .map(AppId::name)
        .collect();
    assert_eq!(
        pinned,
        vec!["org.kde.dolphin", "org.mozilla.firefox"],
        "the Dock was not pinned to the applications discovery found, in the roles' order"
    );
}

/// **A failed reading pins nothing — and that is not the same as a machine
/// with nothing.**
///
/// Both leave an empty Dock this boot and the difference is what gets written
/// down, which nothing writes yet. Asserted here anyway, because the day
/// persistence lands this is the test that says which of the two may be
/// stored, and a test written then would be written against the code.
#[test]
fn a_failed_reading_pins_nothing() {
    let (_directory, mut server) = server("a-failed-reading");
    server.its_dock_is_filled_from(&both_roles());

    server.its_dock_is_filled_from(&Discovery::Failed);

    assert_eq!(
        server.what_the_dock_is_pinned_to().each_pinned().count(),
        0,
        "a failed reading left pins on the Dock"
    );
    assert!(
        !ThePins::NeverSet
            .what_to_pin(&Discovery::Failed)
            .should_be_written_down(),
        "a failed reading asked to be written down, which would empty a person's Dock for good"
    );
    assert!(
        ThePins::NeverSet
            .what_to_pin(&Discovery::Completed(Vec::new()))
            .should_be_written_down(),
        "a machine that really has nothing installed was not written down, so it would be \
         rediscovered every boot"
    );
}

/// **A reading that has not finished pins nothing either**, and for the other
/// reason: it is waited for rather than reported.
#[test]
fn a_reading_still_running_pins_nothing() {
    let (_directory, mut server) = server("still-running");
    server.its_dock_is_filled_from(&both_roles());

    server.its_dock_is_filled_from(&Discovery::StillLooking);

    assert_eq!(
        server.what_the_dock_is_pinned_to().each_pinned().count(),
        0,
        "a reading that had not finished left pins on the Dock"
    );
}

/// **A machine that really has nothing installed gets an empty Dock**, which is
/// a real answer and not a failure.
#[test]
fn a_machine_with_nothing_installed_is_pinned_to_nothing() {
    let (_directory, mut server) = server("nothing-installed");

    server.its_dock_is_filled_from(&Discovery::Completed(Vec::new()));

    assert_eq!(
        server.what_the_dock_is_pinned_to().each_pinned().count(),
        0,
        "a machine with nothing installed was pinned to something"
    );
}

/// **The Dock draws the pins, and the keyboard walks the same list.**
///
/// This is the assertion the change needed and did not have. While the two
/// Dock lists were only reachable by passing a `Holding`, three call sites
/// across two files each passed `Holding::nothing()` — and putting the
/// constant back at the draw's seam left **854 of 854 tests passing**. The
/// producer was tested, the consumer was tested, and nothing tested that they
/// were joined.
///
/// A pinned application **with no window** is what makes it checkable: it is on
/// the Dock only because it was pinned, so a list built from a constant does
/// not contain it, and the count below is 0 instead of 1.
#[test]
fn what_is_pinned_reaches_both_the_draw_and_the_keyboard() {
    use crate::what_the_dock_has_open::WhatTheDockWasAskedFor as Asked;
    let (_directory, mut server) = server("pins-reach-both");

    server.its_dock_is_filled_from(&Discovery::Completed(vec![(
        ARole::Browser,
        app("org.mozilla.firefox"),
    )]));

    let drawn = server.what_this_persons_dock_holds();
    let on_the_dock: Vec<&str> = drawn.iter().map(|on| on.app().name()).collect();
    assert_eq!(
        on_the_dock,
        vec!["org.mozilla.firefox"],
        "an application this person pinned is not on the Dock that is drawn"
    );

    let grid = server.what_this_persons_more_apps_holds();
    let in_more_apps: Vec<&str> = grid.iter().map(|on| on.app().name()).collect();
    assert_eq!(
        in_more_apps,
        vec!["org.mozilla.firefox"],
        "an application this person pinned is not in the grid that is drawn"
    );

    server.the_dock_was_asked_for = Asked::MoreApps { focused: None };
    let (how_many, across) = server.what_the_keyboard_moves_through();
    assert_eq!(
        how_many, 1,
        "the arrow keys walk a different list from the grid a person is looking at"
    );
    assert!(across > 0, "a grid of no items wide");
}
