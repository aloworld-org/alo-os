//! That a task window is a window on the canvas: pressed where the camera draws
//! it at any pan and zoom, counted by *Show all*, out of the way on another
//! Place, and reached by the view rather than pulled to it.

use std::os::unix::fs::PermissionsExt;

use alo_canvas::{At, Place, Zoom};
use alo_models::InferenceSource;
use alo_tasks::ATask;
use smithay::input::keyboard::XkbConfig;
use smithay::utils::{Physical, Point, Rectangle, Size};

use crate::Server;
use crate::the_tasks_a_person_opened::WhichTask;

/// A seat with a keyboard and a 1280 × 800 output.
fn a_session(what: &str) -> (tempfile::TempDir, Server) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind_keyboard(
        runtime.path(),
        what,
        XkbConfig {
            layout: "us",
            ..XkbConfig::default()
        },
    )
    .expect("a display with a keyboard");
    server.surfaces.popups.output_size = Some(Size::from((1280, 800)));
    (runtime, server)
}

/// A task window, placed on the plane in the view, on the Place being looked
/// at — as a frame places it.
fn a_task(server: &mut Server) -> (WhichTask, Rectangle<i32, Physical>) {
    let which = server.tasks_mut().and_now_this_one(ATask::asked_for(
        "tidy my invoices",
        InferenceSource::ThisMachine,
    ));
    let here = server.the_place_now();
    server.tasks_mut().new_windows_open_on(here);
    let view = Rectangle::new((0, 0).into(), (1280, 800).into());
    let _ = server.tasks_mut().put_whatever_has_nowhere_to_be(
        view,
        alo_strings::Direction::LeftToRight,
        &[],
        100,
    );
    let (_, window) = server
        .the_task_windows_on_the_plane()
        .into_iter()
        .find(|(it, _)| *it == which)
        .expect("the window was placed");
    (which, window)
}

/// Its middle, on the plane.
fn the_middle_of(window: Rectangle<i32, Physical>) -> (i32, i32) {
    (
        window.loc.x + window.size.w / 2,
        window.loc.y + window.size.h / 2,
    )
}

/// **Pressed where it is drawn, at every pan and zoom** — and not where it
/// would be with the canvas unmoved, once the canvas has moved.
#[test]
fn a_task_window_is_pressed_where_the_camera_draws_it() {
    let (_runtime, mut server) = a_session("task-pressed-where-drawn");
    let (which, window) = a_task(&mut server);
    let middle = the_middle_of(window);

    for (looking, zoom) in [
        ((0, 0), 1000),
        ((-2000, -2000), 1000),
        ((-1500, -1500), 500),
    ] {
        assert!(
            server
                .look_at_the_canvas(At::checked(0, 0).expect("the origin"))
                .is_some()
        );
        assert!(
            server
                .zoom_the_canvas(Zoom::of(zoom).expect("a zoom"), (0, 0))
                .is_some()
        );
        assert!(
            server
                .look_at_the_canvas(At::checked(looking.0, looking.1).expect("on the plane"))
                .is_some()
        );
        let seen = server
            .the_task_windows_on_the_screen()
            .into_iter()
            .find(|(it, _)| *it == which)
            .map(|(_, on)| on)
            .expect("on the screen");
        let drawn_middle =
            Point::from((seen.loc.x + seen.size.w / 2, seen.loc.y + seen.size.h / 2));
        assert_eq!(
            server.the_task_window_under(drawn_middle),
            Some(which),
            "looking from {looking:?} at {zoom}, a press on the window as drawn missed it"
        );
        if (looking, zoom) != ((0, 0), 1000) {
            assert_eq!(
                server.the_task_window_under(Point::from(middle)),
                None,
                "looking from {looking:?} at {zoom}, a press where the window would be \
                 unmoved still reached it, so it is pressed where it is not drawn"
            );
        }
    }
}

/// **Zoomed out to half, it is drawn half the size** — the fault the booted
/// machine showed for alo's terminal, which a task window shared.
#[test]
fn zoomed_out_a_task_window_is_drawn_smaller() {
    let (_runtime, mut server) = a_session("task-zoomed");
    let (_which, window) = a_task(&mut server);
    assert!(
        server
            .zoom_the_canvas(Zoom::of(500).expect("a zoom"), (0, 0))
            .is_some()
    );
    let (_, seen) = server
        .the_task_windows_on_the_screen()
        .into_iter()
        .next()
        .expect("on the screen");
    assert_eq!(
        (seen.size.w, seen.size.h),
        (window.size.w / 2, window.size.h / 2)
    );
}

/// **Show all counts it**, and a task window's number is never a client's.
#[test]
fn show_all_counts_a_task_window() {
    let (_runtime, mut server) = a_session("task-counted");
    let (_which, window) = a_task(&mut server);
    let frames = server.the_task_windows_as_frames();
    assert_eq!(frames.len(), 1);
    let frame = frames.first().expect("a frame");
    assert_eq!((frame.at().x, frame.at().y), (window.loc.x, window.loc.y));
    assert!(
        frame.id() > u64::MAX / 2,
        "a task window's number is in a client's range"
    );
}

/// **On another Place it is out of the way**: not on the screen there, not
/// pressed, not taking keys — and Alt+Tab's road takes the person to its Place.
#[test]
fn on_another_place_a_task_window_is_out_of_the_way_until_the_person_goes_to_it() {
    let (_runtime, mut server) = a_session("task-elsewhere");
    let (which, window) = a_task(&mut server);
    server.this_task_took_the_keyboard(which);
    assert!(server.a_task_is_taking_the_keys());

    let elsewhere = Place::FIRST.next().expect("a second Place");
    server.surfaces.place = elsewhere;
    assert!(server.the_task_windows_on_the_screen().is_empty());
    assert_eq!(
        server.the_task_window_under(Point::from(the_middle_of(window))),
        None
    );
    assert!(
        !server.a_task_is_taking_the_keys(),
        "keys went to a task window on a Place nobody is looking at"
    );

    assert!(server.the_view_goes_to_this_task(which));
    assert_eq!(server.the_place_now(), Place::FIRST);
    assert!(server.a_task_is_taking_the_keys());
}

/// **Going to a task window out of sight brings the view to it**, and the
/// window stays where it is.
#[test]
fn going_to_a_task_window_out_of_sight_brings_the_view() {
    let (_runtime, mut server) = a_session("task-travelled-to");
    let (which, window) = a_task(&mut server);
    assert!(server.pan_the_canvas(6_000, 6_000).is_some());
    assert!(server.the_view_goes_to_this_task(which));
    let (_, still) = server
        .the_task_windows_on_the_plane()
        .into_iter()
        .next()
        .expect("still open");
    assert_eq!(still, window, "the window was moved to the view instead");
    let (_, seen) = server
        .the_task_windows_on_the_screen()
        .into_iter()
        .next()
        .expect("on the screen");
    assert!(
        Rectangle::<i32, Physical>::from_size((1280, 800).into()).contains_rect(seen),
        "the window is still out of sight at {seen:?}"
    );
    assert!(
        !server.the_view_goes_to_this_task(which),
        "a window already in sight moved the view"
    );
}
