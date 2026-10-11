//! What a server with no pointing device says about an edge.
//!
//! **The geometry is not retested here.** `crate::window_edge_pointing` owns
//! it and tests it against real layouts — every control at its middle, the
//! margin outside the drawn pill, the gap between targets, the half-open
//! boundary, the drag region, the concealed edge. Asking a seat the same
//! questions would need a display, a pointing device and motion events to
//! reach the same function, which is the reason
//! `a_click_brings_a_window_back_tests` gives for testing its decision rather
//! than its seat.
//!
//! What is left is the one thing only a real server can be wrong about, and
//! the one this method exists for: **a machine with no pointer must highlight
//! nothing**, rather than highlighting whatever sits at the origin.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on a socket that would not bind is the failure being reported"
)]

use super::*;
use crate::Server;
use crate::window_edge::{Decorations, OnTheEdge, edge_of};
use crate::window_edge_pointing::{UnderThePointer, what_is_under};
use smithay::utils::{Logical, Point, Rectangle, Size};

/// An edge whose controls reach the origin, so the absent-pointer answer and
/// the origin answer are different answers.
///
/// Its window starts at `(0, 44)` and is 600 wide, which puts the region at
/// `(0, 0)` — the specimen `crate::window_edge`'s own tests use.
fn an_edge_over_the_origin() -> crate::window_edge::WindowEdge {
    let window = Rectangle::new(
        Point::from((0, crate::window_edge::THE_REGION_IS_TALL)),
        Size::from((600, 400)),
    );
    edge_of(window, Decorations::TheShellDraws, true)
}

/// A server bound in a directory of its own, with no pointing device.
fn a_server() -> (tempfile::TempDir, Server) {
    use std::os::unix::fs::PermissionsExt as _;
    let directory = tempfile::tempdir().expect("a temporary directory");
    // **0700 set rather than assumed.** `Server::bind` refuses a runtime
    // directory that is not exactly mode 0700 (`crate::socket`), and a tempdir
    // arrives at whatever the umask leaves - which is why the first run of
    // this file, written without these two lines, failed with `UnsafeRuntime`
    // and not with anything about an edge. The same two lines
    // `a_camera_per_viewport_tests` uses, for the same bind.
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind(directory.path(), "pointed-at").expect("a bound socket");
    (directory, server)
}

/// **No pointer highlights nothing**, and the origin is a control.
///
/// The second half is the control on this test: `what_is_under` is asked at
/// `(0, 0)` first, so the assertion that follows is known to be capable of
/// failing. If the method ever truncated an absent pointer to the origin — the
/// fault `the_pointer_in_pixels` keeps `pub(crate)` to prevent a host from
/// committing — the window at the top-left of a display would show a lit
/// control that nobody was pointing at.
#[test]
fn a_machine_with_no_pointer_highlights_nothing() {
    let edge = an_edge_over_the_origin();
    let origin: Point<i32, Logical> = Point::from((0, 0));
    assert!(
        matches!(
            what_is_under(&edge, origin),
            Some(UnderThePointer::Control(_)) | Some(UnderThePointer::TheEdgeAndNothingOnIt)
        ),
        "this test only measures something while the origin is on the edge"
    );

    let (_directory, server) = a_server();
    assert_eq!(
        server.where_the_pointer_is_in_logical_pixels(),
        None,
        "a server bound with no pointing device has no pointer"
    );
    assert_eq!(
        server.what_this_edge_is_pointing_at(&edge, None),
        Pointing::default(),
        "a machine with no pointer lit a control"
    );
}

/// Focus is passed through by a real server, pointer or no pointer.
///
/// The host supplies it, so the one way this could go wrong is the method
/// dropping it — which would be silent, because the only visible difference is
/// a focus outline that never appears.
#[test]
fn the_focus_a_host_supplies_survives() {
    let edge = an_edge_over_the_origin();
    let (_directory, server) = a_server();
    let pointing = server.what_this_edge_is_pointing_at(&edge, Some(OnTheEdge::Maximise));
    assert_eq!(pointing.hovered, None, "there is still no pointer");
    assert_eq!(pointing.focused, Some(OnTheEdge::Maximise));
}

/// A concealed edge has nothing to light, which is checked here rather than
/// assumed from the layout.
#[test]
fn a_concealed_edge_lights_nothing_either() {
    let window = Rectangle::new(
        Point::from((0, crate::window_edge::THE_REGION_IS_TALL)),
        Size::from((600, 400)),
    );
    let edge = edge_of(window, Decorations::TheShellDraws, false);
    assert!(
        edge.controls.is_empty(),
        "a concealed edge draws no control"
    );
    let (_directory, server) = a_server();
    assert_eq!(
        server.what_this_edge_is_pointing_at(&edge, None),
        Pointing::default()
    );
}
