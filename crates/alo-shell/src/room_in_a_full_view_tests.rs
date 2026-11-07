//! The one rule every window the person opens follows into a full view, and
//! that every one of alo's own windows reaches it.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;
use std::time::Instant;

use smithay::utils::{Logical, Point, Rectangle, Size};

use super::AnOpening;
use crate::Server;

/// A server on a socket of its own.
fn a_server(named: &str) -> (tempfile::TempDir, Server) {
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind(directory.path(), named).expect("a bound socket");
    (directory, server)
}

/// A rectangle on the plane.
type APlaneRect = Rectangle<i32, Logical>;

/// A view one window fills, the window filling it, and where rules 3 to 7
/// would put the next one: over it.
fn a_full_view() -> (APlaneRect, Vec<APlaneRect>, Point<i32, Logical>) {
    let view = Rectangle::new((0, 0).into(), (1440, 960).into());
    (view, vec![view], Point::from((64, 64)))
}

/// The opening, for a window of 640 by 480.
fn opening<'a>(
    view: Rectangle<i32, Logical>,
    open: &'a [Rectangle<i32, Logical>],
) -> AnOpening<'a> {
    AnOpening {
        view,
        open,
        wanted: Size::from((640, 480)),
        fixed: &[],
        reading: alo_strings::Direction::LeftToRight,
    }
}

/// **A full view, the default: empty space, and the view glides there.**
#[test]
fn a_full_view_finds_empty_space_and_the_view_glides() {
    let (_directory, mut server) = a_server("full-glides");
    let (view, open, first) = a_full_view();
    let at = server.room_for_what_the_person_opened(first, &opening(view, &open), Instant::now());
    let window = Rectangle::new(at, Size::from((640, 480)));
    assert!(
        !open.iter().any(|it| it.overlaps(window)),
        "it covers a window"
    );
    assert!(server.the_view_is_gliding(), "the view did not set off");
}

/// **The refusals keep rules 3 to 7's answer, and the view stays**: *Open in
/// front* chosen, and a view with room.
#[test]
fn open_in_front_or_room_in_view_keeps_the_first_answer() {
    let (_in_front, mut server) = a_server("in-front");
    server.when_there_is_no_room_is(alo_appearance::WhenThereIsNoRoom::OpenInFront);
    let (view, open, first) = a_full_view();
    assert_eq!(
        server.room_for_what_the_person_opened(first, &opening(view, &open), Instant::now()),
        first
    );
    assert!(!server.the_view_is_gliding());

    let (_roomy, mut roomy) = a_server("roomy");
    assert_eq!(
        roomy.room_for_what_the_person_opened(first, &opening(view, &[]), Instant::now()),
        first,
        "a view with room moved the window"
    );
    assert!(!roomy.the_view_is_gliding());
}

/// **Less motion asked for: the view jumps** to the space.
#[test]
fn with_reduced_motion_the_view_jumps() {
    let (_directory, mut server) = a_server("full-jumps");
    server.the_motion_is_reduced(true);
    let (view, open, first) = a_full_view();
    let at = server.room_for_what_the_person_opened(first, &opening(view, &open), Instant::now());
    assert!(!server.the_view_is_gliding());
    let camera = server.the_camera().at();
    let seen = Rectangle::<i32, Logical>::new((camera.x, camera.y).into(), view.size);
    assert!(
        seen.contains_rect(Rectangle::new(at, Size::from((640, 480)))),
        "the view did not jump to the new window"
    );
}

/// **Every one of alo's own windows reaches the rule** — the owner's *"Yes, the
/// same"* — in the part of each file that ships, not in a test.
#[test]
fn the_terminal_settings_the_record_and_task_windows_all_ask_the_rule() {
    let shipped = |text: &'static str| text.split("#[cfg(test)]").next().unwrap_or_default();
    for (named, text, asks) in [
        (
            "the terminal",
            include_str!("the_terminal_on_the_plane.rs"),
            "self.room_for_what_the_person_opened(",
        ),
        (
            "Settings",
            include_str!("settings_on_the_plane.rs"),
            "self.room_for_what_the_person_opened(",
        ),
        (
            "the record",
            include_str!("record_on_the_plane.rs"),
            "self.room_for_what_the_person_opened(",
        ),
        (
            "an application",
            include_str!("a_new_window_is_placed.rs"),
            "self.room_for_what_the_person_opened(",
        ),
        (
            "a task window",
            include_str!("direct_desktop.rs"),
            "server.the_task_windows_find_room(",
        ),
    ] {
        assert!(
            shipped(text).contains(asks),
            "{named} does not ask the rule"
        );
    }
}
