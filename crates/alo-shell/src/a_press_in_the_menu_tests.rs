//! A press in the menu, and the one case reachable without a client.
//!
//! **What is asked here and what is not.** Every road into
//! `a_press_in_the_menu` needs a `WlSurface`, which needs a client, a socket
//! pair and a dispatch loop — `a_frame_per_display_tests` shows what that
//! costs for one surface. So what is tested is the case a bound server can
//! reach: a machine with no pointer, and a machine with no menu open. The
//! rows' own geometry is `window_menu_tests`' and what they draw is
//! `window_menu_paint_tests`'.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on a socket that would not bind is the failure being reported"
)]

use crate::Server;
use smithay::backend::input::ButtonState;

/// A server bound in a directory of its own, with no pointing device.
fn a_server(named: &str) -> (tempfile::TempDir, Server) {
    use std::os::unix::fs::PermissionsExt as _;
    let directory = tempfile::tempdir().expect("a temporary directory");
    // `Server::bind` refuses a runtime directory that is not exactly 0700,
    // and a tempdir arrives at whatever the umask leaves.
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind(directory.path(), named).expect("a bound socket");
    (directory, server)
}

/// **A release is never the menu's.** A row acts on its press, and a release
/// belongs to whatever gesture owns the pointer.
#[test]
fn a_release_is_never_the_menus() {
    let (_directory, mut server) = a_server("menu-release");
    assert!(!server.a_press_in_the_menu(ButtonState::Released));
}

/// **With no menu open, a press is not the menu's** — so it falls through to
/// the edge and the window beneath, which is the whole reason this answers a
/// boolean rather than consuming everything.
#[test]
fn a_press_with_no_menu_open_is_not_the_menus() {
    let (_directory, mut server) = a_server("menu-none-open");
    assert!(
        server.keyboard_on_edges.is_empty(),
        "the fixture has no menu"
    );
    assert!(!server.a_press_in_the_menu(ButtonState::Pressed));
}

/// **A machine with no pointer presses nothing**, which is the rule every
/// reader of the pointer in this crate carries: an absent pointer must stay
/// distinguishable from a pointer at the origin.
#[test]
fn a_machine_with_no_pointer_presses_nothing_in_a_menu() {
    let (_directory, mut server) = a_server("menu-no-pointer");
    assert!(
        server.where_the_pointer_is_in_logical_pixels().is_none(),
        "this fixture is supposed to have no pointing device"
    );
    assert!(!server.a_press_in_the_menu(ButtonState::Pressed));
}
