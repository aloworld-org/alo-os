//! What this compositor does with each thing the gesture recogniser answers.
//!
//! Not what a swipe *is* — how many fingers, how far, in which direction, and
//! which of those a person has turned off are all `alo-desktops`' and are held
//! by that crate's own tests against its deterministic event stream. What is
//! held here is the half that is this crate's: a recognised desktop switch is
//! carried out on every display and the answer shown, and the other two intents
//! are refused rather than half-carried-out.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_desktops::{DisplayId, Promises, Switch};
use alo_dividing::{Area, Point, WindowId, area::Size};
use std::os::unix::fs::PermissionsExt as _;

/// A socket directory this crate will accept: a runtime directory is refused
/// unless only its owner can reach it.
fn a_runtime() -> tempfile::TempDir {
    let directory = tempfile::tempdir().unwrap();
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    directory
}

/// A server with one display on it, at the size of an ordinary screen.
fn a_session_with_a_display() -> (tempfile::TempDir, crate::Server) {
    let directory = a_runtime();
    let mut server = crate::Server::bind(directory.path(), "swipes").unwrap();
    server
        .display_arrived(
            DisplayId::from_compositor(1),
            "a-screen",
            Area::of(Point::at(0, 0), Size::of(1920, 1080)).unwrap(),
            Promises::of(
                WindowId::from_compositor(1),
                WindowId::from_compositor(2),
                WindowId::from_compositor(3),
            )
            .unwrap(),
            &|_| None,
        )
        .unwrap();
    (directory, server)
}

/// Which desktop this display is showing.
fn showing(server: &crate::Server) -> alo_desktops::DesktopId {
    server
        .desktops_on(DisplayId::from_compositor(1))
        .unwrap()
        .current()
}

/// **A recognised swipe switches the desktop, and the answer is the crate's.**
///
/// What this asserts is that the desktop being shown afterwards is the one
/// `alo-desktops` says it switched to — not that it is the second one, or the
/// next one, or any other description of it assembled here. A shell that agreed
/// with the crate by arithmetic would be the second decider all over again.
#[test]
fn a_recognised_swipe_shows_the_desktop_the_crate_switched_to() {
    let (_directory, mut server) = a_session_with_a_display();
    let display = DisplayId::from_compositor(1);
    assert_eq!(
        server.desktops_on(display).unwrap().how_many(),
        1,
        "a display arrived with more than the one desktop it starts with"
    );
    // A second desktop to switch to; adding one is the person's road and not
    // the swipe's.
    let added = server.desktops_on_mut(display).unwrap().add().unwrap();

    assert!(server.carry_out(alo_desktops::gesture_events::Intent::Desktop(Switch::Next)));

    assert_eq!(showing(&server), added, "the swipe showed another desktop");
}

/// **A swipe at the end of the row changes nothing, because the crate refused.**
///
/// There is no wrap here and there is no wrap invented here: one desktop means
/// a next that does not exist, and `alo-desktops` saying so leaves the display
/// showing exactly what it was showing.
#[test]
fn a_swipe_past_the_last_desktop_leaves_the_display_as_it_was() {
    let (_directory, mut server) = a_session_with_a_display();
    let before = showing(&server);

    assert!(server.carry_out(alo_desktops::gesture_events::Intent::Desktop(Switch::Next)));

    assert_eq!(showing(&server), before);
}

/// **A swipe on a session with no display is not an error and not a switch.**
///
/// It is what a machine is between the sign-in screen ending and its first
/// frame. Nothing is refused to anybody, because nobody asked for a desktop
/// that exists.
#[test]
fn a_swipe_before_any_display_has_arrived_does_nothing() {
    let directory = a_runtime();
    let mut server = crate::Server::bind(directory.path(), "no-display").unwrap();

    assert!(server.has_no_display());
    assert!(server.carry_out(alo_desktops::gesture_events::Intent::Desktop(Switch::Next)));
    assert!(server.has_no_display());
}

/// **A pinch is not carried out here, and says so rather than swallowing it.**
///
/// Nothing in this compositor zooms an application. Returning `false` hands the
/// event back to the ordinary road instead of consuming it as a gesture that
/// was handled — a pinch that vanished into a compositor which did nothing with
/// it is the shape of a fault nobody finds.
#[test]
fn a_zoom_is_not_carried_out_because_nothing_here_can() {
    let (_directory, mut server) = a_session_with_a_display();

    assert!(
        !server.carry_out(alo_desktops::gesture_events::Intent::Zoom(2.0)),
        "a zoom was reported as carried out by a compositor that cannot zoom"
    );
}

/// **A scroll is not carried out here, because it is already carried out.**
///
/// `crate::libinput_scroll` sends the axis to the focused client. Doing it here
/// as well would scroll twice for one movement of somebody's fingers, which is
/// the fault `crate::libinput_routing` already warns about for the deprecated
/// axis events.
#[test]
fn a_scroll_is_left_to_the_road_that_already_scrolls() {
    let (_directory, mut server) = a_session_with_a_display();

    assert!(
        !server.carry_out(alo_desktops::gesture_events::Intent::Scroll {
            horizontal: Some(4.0),
            vertical: None,
        })
    );
}
