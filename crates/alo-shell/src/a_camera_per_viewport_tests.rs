//! A camera per display: each its own view onto one canvas.
//!
//! `docs/autonomy/more-than-one-display-plan.md` task 7.

use crate::Server;
use alo_displays::{Reported, Socket};

/// A screen of this size in this socket.
fn a_screen(socket: &str, pixels: (u32, u32)) -> Reported {
    Reported::of(
        Socket::named(socket).expect("a socket"),
        None,
        pixels,
        Some((597, 336)),
    )
    .expect("a screen")
}

/// A server holding the arrangement these screens make.
fn server_showing(reported: Vec<Reported>) -> (tempfile::TempDir, Server) {
    use std::os::unix::fs::PermissionsExt as _;
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind(directory.path(), "viewports").expect("a bound socket");
    server.these_screens_are(Some(crate::screens_testing::the_screens(
        reported,
        &alo_displays::Changes::untouched(),
        &alo_appearance::Appearance::shipped(),
        &crate::screens_testing::a_cold_evening(),
    )));
    (directory, server)
}

/// **Two displays hold two cameras, and moving one leaves the other where it
/// was.**
///
/// The acceptance's *each at its own zoom*. Before this there was one camera
/// for the session, so the second display could only ever show the first
/// display's view — mirroring, whatever else had been made per display.
///
/// Written through the per-display accessor rather than the field, because
/// what is being tested is that the two are genuinely separate values and
/// not one value with a second home, which is the shape the plan's constraint
/// forbids.
#[test]
fn two_displays_hold_two_cameras() {
    let (_directory, mut server) = server_showing(vec![
        a_screen("DP-1", (1920, 1080)),
        a_screen("DP-2", (1920, 1080)),
    ]);
    let moved = alo_canvas::Camera::new()
        .panned_by(300, 40)
        .expect("a camera can pan");

    server.surfaces.cameras.insert("DP-2".to_owned(), moved);

    assert_eq!(
        server.surfaces.camera_of("DP-2"),
        moved,
        "the display that was panned did not keep its own view"
    );
    assert_eq!(
        server.surfaces.camera_of("DP-1"),
        alo_canvas::Camera::new(),
        "panning one display moved the other, which is one camera with two homes"
    );
}

/// **A display nobody has panned is looking at the origin at life size.**
///
/// Not an absence: it is where every display looked before any of them had a
/// camera of its own, so a display with no entry behaves exactly as the
/// single camera used to.
#[test]
fn a_display_with_no_camera_looks_at_the_origin() {
    let (_directory, server) = server_showing(vec![a_screen("DP-1", (1920, 1080))]);
    assert_eq!(server.surfaces.camera_of("DP-1"), alo_canvas::Camera::new());
    assert_eq!(
        server.surfaces.camera_of("a-display-that-was-never-here"),
        alo_canvas::Camera::new()
    );
}

/// **With no arrangement there is one camera and `the_camera` is it.**
///
/// Every machine this lane can test. The key is the empty name, which is a
/// detail of the map rather than of the behaviour — what matters is that one
/// display reads back exactly what was written.
#[test]
fn one_display_reads_back_what_was_written() {
    use std::os::unix::fs::PermissionsExt as _;
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind(directory.path(), "one-view").expect("a bound socket");
    assert!(server.the_screens().is_none());
    assert_eq!(server.the_camera(), alo_canvas::Camera::new());

    let moved = alo_canvas::Camera::new()
        .panned_by(-20, 7)
        .expect("a camera can pan");
    server.set_the_camera(moved);

    assert_eq!(
        server.the_camera(),
        moved,
        "a session with one display did not read back its own camera"
    );
}

/// **The display being worked on is the main screen when the pointer is
/// nowhere known.**
///
/// A keyboard command has no position, and the main screen is where the
/// arrangement says a new window opens — so it is where an action nobody
/// placed belongs.
#[test]
fn with_no_pointer_the_main_screen_is_the_one_being_worked_on() {
    let (_directory, server) = server_showing(vec![
        a_screen("DP-1", (1920, 1080)),
        a_screen("DP-2", (1920, 1080)),
    ]);
    let main = server
        .the_screens()
        .expect("an arrangement")
        .each()
        .find(|place| place.is_main())
        .map(|place| place.name().name().to_owned())
        .expect("a main screen");

    assert_eq!(
        server.the_display_being_worked_on(),
        main,
        "an action with no position did not land on the main screen"
    );
}

/// **A write lands on the display being worked on, and nowhere else.**
///
/// The whole point of routing the six assignment sites through one writer:
/// panning while looking at one display must not move another's view.
#[test]
fn a_write_lands_on_one_display_only() {
    let (_directory, mut server) = server_showing(vec![
        a_screen("DP-1", (1920, 1080)),
        a_screen("DP-2", (1920, 1080)),
    ]);
    let worked_on = server.the_display_being_worked_on();
    let moved = alo_canvas::Camera::new()
        .panned_by(11, 22)
        .expect("a camera can pan");

    server.set_the_camera(moved);

    assert_eq!(server.surfaces.camera_of(&worked_on), moved);
    let others: Vec<_> = server
        .the_screens()
        .expect("an arrangement")
        .each()
        .map(|place| place.name().name().to_owned())
        .filter(|named| *named != worked_on)
        .collect();
    assert!(!others.is_empty(), "the fixture has only one display");
    for named in others {
        assert_eq!(
            server.surfaces.camera_of(&named),
            alo_canvas::Camera::new(),
            "writing one display's camera moved {named}"
        );
    }
}
