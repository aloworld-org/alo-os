//! Where Tab goes along the bar, where it stops, and when it is not the
//! Dock's key at all.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;

use super::{WhichWay, where_tab_goes};
use crate::Server;
use crate::which_area_has_the_keyboard::AnArea;

/// A server on a socket of its own.
fn server(named: &str) -> (tempfile::TempDir, Server) {
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let bound = Server::bind(directory.path(), named).expect("a bound socket");
    (directory, bound)
}

/// **Tab walks on and Shift+Tab walks back.**
#[test]
fn tab_walks_the_controls_in_order() {
    assert_eq!(where_tab_goes(0, WhichWay::Onward, 5), 1);
    assert_eq!(where_tab_goes(1, WhichWay::Onward, 5), 2);
    assert_eq!(where_tab_goes(2, WhichWay::Back, 5), 1);
    assert_eq!(where_tab_goes(1, WhichWay::Back, 5), 0);
}

/// **It stops at the ends rather than wrapping**, which is the design and not
/// an omission.
///
/// `alo_access::reaching::after_the_last` says focus is never trapped — *Tab
/// from the end of a surface arrives at its beginning* — and that is for a
/// surface a person is **inside of**. The bar is one of five areas in a cycle
/// and F6 is how a person leaves it. Tab past the last slot would either trap
/// them or quietly do what F6 does, and the second is worse: they would arrive
/// somewhere else without having asked to.
#[test]
fn tab_stops_at_the_ends_and_does_not_wrap() {
    assert_eq!(
        where_tab_goes(4, WhichWay::Onward, 5),
        4,
        "Tab past the last control wrapped, so a person is somewhere they did not ask to be"
    );
    assert_eq!(
        where_tab_goes(0, WhichWay::Back, 5),
        0,
        "Shift+Tab before the first control wrapped"
    );
}

/// **A bar with one control has nowhere to go**, and both directions say so.
#[test]
fn one_control_is_where_both_directions_stay() {
    assert_eq!(where_tab_goes(0, WhichWay::Onward, 1), 0);
    assert_eq!(where_tab_goes(0, WhichWay::Back, 1), 0);
}

/// **A bar with nothing on it answers the first slot rather than panicking.**
///
/// `how_many` of zero cannot happen while the bar has its own controls, and
/// this workspace denies indexing a slice: a model that could not be asked
/// about an empty bar would be one that panicked the first time a draw had not
/// happened yet.
#[test]
fn an_empty_bar_is_answered_rather_than_panicked_on() {
    assert_eq!(where_tab_goes(0, WhichWay::Onward, 0), 0);
    assert_eq!(where_tab_goes(3, WhichWay::Back, 0), 2);
}

/// **Tab is not the Dock's key unless the Dock has the keyboard.**
///
/// Every ordinary moment is this one. Tab belongs to the application somebody
/// is typing in, and a shell that took it whenever it felt like it would be a
/// shell a person cannot fill in a form on.
#[test]
fn tab_is_not_taken_when_the_keyboard_is_elsewhere() {
    let (_directory, mut server) = server("tab-elsewhere");
    assert_eq!(
        server.which_area_has_the_keyboard, None,
        "the premise: the keyboard is with the work"
    );

    assert!(
        !server.tab_moves_among_the_bars_controls(WhichWay::Onward),
        "Tab was taken while the keyboard was with the work"
    );
    assert_eq!(
        server.which_slot_on_the_bar, None,
        "Tab moved a slot while the keyboard was elsewhere"
    );
}

/// **With the Dock focused and nothing drawn, Tab is taken and moves nothing.**
///
/// Taken rather than passed on: a person who pressed F6 to reach the Dock is
/// looking at it, and a Tab that reached the application behind would type into
/// something they are not looking at. Nothing is drawn here because no frame
/// has been rendered, which is the honest state of a server a test just bound.
#[test]
fn tab_on_a_dock_that_has_not_been_drawn_is_taken_and_moves_nothing() {
    let (_directory, mut server) = server("tab-undrawn");
    server.the_keyboard_is_now_in(Some(AnArea::TheDock));
    assert_eq!(server.which_slot_on_the_bar, Some(0), "the premise");

    assert!(
        server.tab_moves_among_the_bars_controls(WhichWay::Onward),
        "Tab was passed to the application behind a surface the person is looking at"
    );
    assert_eq!(
        server.how_many_controls_the_bar_has(),
        0,
        "the premise: no frame has laid the bar out"
    );
    assert_eq!(server.which_slot_on_the_bar, Some(0));
}
