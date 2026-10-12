//! What the keyboard can reach, and what it does with it.
//!
//! # What is measured here, and what is not
//!
//! A `Server` binds without a compositor, so **every test here has no
//! clients** — no windows, no applications, nothing on either surface. That is
//! the state in which the interesting refusals live: an open popover holding
//! nothing must hand an arrow key back, and Enter with nothing focused must do
//! nothing rather than reach an application behind.
//!
//! What it cannot show is a person moving through four applications and
//! choosing the third. That needs mapped surfaces, which needs a real client —
//! `crate::dock_windows`' own tests say the same of themselves, and the road is
//! `examples/a_real_application.rs` on a machine with a Wayland parent. **This
//! file does not stand in for that**; the movement itself is a pure function
//! and `what_the_dock_has_open_tests.rs` walks every direction of it.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected Err or None is the failure being reported"
)]

use crate::Server;

/// A server on a private socket, with no clients.
fn server(named: &str) -> (tempfile::TempDir, Server) {
    use std::os::unix::fs::PermissionsExt as _;
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind(directory.path(), named).expect("a bound socket");
    (directory, server)
}

/// **The row the keyboard moves by is the row the draw lays out.**
///
/// Two answers to *how many across* is the fault this asks about: the keyboard
/// would step four where the grid drew three, and the ring would land on a cell
/// that is not where a person is looking. `TheOverview::of` takes the count
/// from `how_many_across`, so this holds them to one another over a spread of
/// list lengths rather than over the one the popover happens to have today.
#[test]
fn the_keyboard_moves_by_the_row_the_grid_draws() {
    let across = alo_dock::more_apps::TheOverview::how_many_across();
    assert!(across > 0, "a grid with no columns");
    for (pinned, others) in [(0, 0), (1, 0), (0, 1), (4, 0), (4, 56), (60, 60)] {
        let drawn =
            alo_dock::more_apps::TheOverview::of(pinned, others, alo_dock::Room::pixels(768));
        assert_eq!(
            drawn.columns(),
            across,
            "a grid of {pinned} pinned and {others} others drew a different row"
        );
    }
}

/// **With nothing open the keyboard moves through nothing**, which is what
/// returns every arrow to the application that has the keyboard.
#[test]
fn nothing_open_is_nothing_to_move_through() {
    let (_directory, server) = server("nothing-open");
    let (how_many, across) = server.what_the_keyboard_moves_through();
    assert_eq!(how_many, 0, "something to move through with nothing open");
    assert!(across > 0, "a row of no items wide");
}

/// **An open overview with no applications hands the arrows back.**
///
/// The dangerous shape is the other one: a popover that is open, holds nothing,
/// and swallows every arrow key anyway. A person would be left with a keyboard
/// that does nothing until they found the mouse.
#[test]
fn an_overview_with_no_applications_does_not_swallow_the_arrows() {
    use crate::what_the_dock_has_open::{AKeyMoves, WhatTheDockWasAskedFor as Asked};
    let (_directory, mut server) = server("empty-overview");
    server.the_dock_was_asked_for = Asked::MoreApps { focused: None };
    let (how_many, across) = server.what_the_keyboard_moves_through();
    assert_eq!(how_many, 0, "an application appeared from nowhere");
    for pressed in [
        AKeyMoves::Up,
        AKeyMoves::Down,
        AKeyMoves::Left,
        AKeyMoves::Right,
    ] {
        assert_eq!(
            server
                .the_dock_was_asked_for
                .when_an_arrow_is_pressed(pressed, how_many, across),
            None,
            "{pressed:?} was taken by an empty overview"
        );
    }
}

/// **Enter with nothing focused changes nothing and refuses nothing.**
///
/// Including that the surface stays open: a person who pressed Enter by
/// accident has not lost the popover they were about to use.
#[test]
fn enter_with_nothing_focused_leaves_the_surface_as_it_was() {
    use crate::what_the_dock_has_open::WhatTheDockWasAskedFor as Asked;
    let (_directory, mut server) = server("enter-unfocused");
    server.the_dock_was_asked_for = Asked::MoreApps { focused: None };
    server.the_keyboard_takes_what_is_focused();
    assert_eq!(
        server.the_dock_was_asked_for,
        Asked::MoreApps { focused: None },
        "Enter with nothing focused moved the Dock on"
    );
}

/// **Enter on an item that is no longer there closes nothing and panics not.**
///
/// A focus can outlive what it pointed at — an application closes while its
/// overview is open — and the index is held in the state rather than re-derived
/// each draw. Indexing a shorter list is where a `[which]` would have ended the
/// session; `get` is why it does not, and this is the test that keeps it.
#[test]
fn enter_on_an_item_that_went_away_does_nothing() {
    use crate::what_the_dock_has_open::WhatTheDockWasAskedFor as Asked;
    let (_directory, mut server) = server("enter-stale");
    for asked in [
        Asked::MoreApps { focused: Some(7) },
        Asked::TheWindowsOf {
            app: alo_dock::AppId::named("gone.app").expect("a named application"),
            focused: Some(3),
        },
    ] {
        server.the_dock_was_asked_for = asked.clone();
        server.the_keyboard_takes_what_is_focused();
        assert_eq!(
            server.the_dock_was_asked_for, asked,
            "Enter on a stale focus changed what was open"
        );
    }
}

/// **A window list is a column**, so Up and Down move by one row.
///
/// Asked of the server rather than assumed, because the picker's panel is laid
/// out by a different function from the grid's and nothing but this says the
/// keyboard reads it as a list.
#[test]
fn a_window_list_moves_one_row_at_a_time() {
    use crate::what_the_dock_has_open::WhatTheDockWasAskedFor as Asked;
    let (_directory, mut server) = server("a-list");
    server.the_dock_was_asked_for = Asked::TheWindowsOf {
        app: alo_dock::AppId::named("some.app").expect("a named application"),
        focused: None,
    };
    let (_, across) = server.what_the_keyboard_moves_through();
    assert_eq!(across, 1, "a window list was laid out as a grid");
}
