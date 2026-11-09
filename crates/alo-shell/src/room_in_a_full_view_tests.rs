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
        strip: crate::a_window_and_its_strip::ItsStrip::at(100),
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

/// What is drawn of a window on the plane: the window and the title strip
/// `window_edge::edge_of` lays out above it — the layout the painter draws —
/// with the camera at the origin, at `zoom` on a display at `scale` per cent,
/// through the same conversions the draw makes: the window onto the screen by
/// the zoom, into the edge's units at the edge's scale (the display's times
/// the zoom, since the owner's *"Title bars shrink with zoom"*), and back.
fn drawn(window: Rectangle<i32, Logical>, zoom: f64, scale: u32) -> Rectangle<f64, Logical> {
    let edges_scale = crate::an_edge_in_its_displays_units::a_scale_through_the_zoom(scale, zoom);
    #[expect(
        clippy::cast_possible_truncation,
        reason = "a test window on a test display is well inside an i32"
    )]
    let on_the_screen = |plane: i32| (f64::from(plane) * zoom).round() as i32;
    let screen = Rectangle::<i32, Logical>::new(
        (on_the_screen(window.loc.x), on_the_screen(window.loc.y)).into(),
        (
            on_the_screen(window.loc.x + window.size.w) - on_the_screen(window.loc.x),
            on_the_screen(window.loc.y + window.size.h) - on_the_screen(window.loc.y),
        )
            .into(),
    );
    let edge = crate::window_edge::edge_of(
        crate::an_edge_in_its_displays_units::a_window_in_the_edges_units(screen, edges_scale),
        crate::window_edge::Decorations::TheShellDraws,
        true,
        Size::from((1_000_000, 1_000_000)),
    );
    let strip = edge.strip;
    let back = |units: i32| f64::from(units) * f64::from(edges_scale) / 100.0 / zoom;
    let strip = Rectangle::<f64, Logical>::new(
        (back(strip.loc.x), back(strip.loc.y)).into(),
        (back(strip.size.w), back(strip.size.h)).into(),
    );
    strip.merge(window.to_f64())
}

/// How far apart two drawn rectangles are: the larger of the gaps across and
/// down, negative where they overlap.
fn apart(one: Rectangle<f64, Logical>, other: Rectangle<f64, Logical>) -> f64 {
    let across =
        (other.loc.x - (one.loc.x + one.size.w)).max(one.loc.x - (other.loc.x + other.size.w));
    let down =
        (other.loc.y - (one.loc.y + one.size.h)).max(one.loc.y - (other.loc.y + other.size.h));
    across.max(down)
}

/// **A window the person opens beside alo's terminal covers neither the
/// terminal nor its strip, and keeps the gap from both — at the zoom it opened
/// at and at every other** — the walk of 2026-11-08, where Dolphin opened
/// under the terminal with its strip drawn 14 pixels into the terminal, and
/// the owner's answer the same day: *"Title bars shrink with zoom"*.
///
/// The terminal as it was on that 1280 × 800 display and Dolphin's window
/// geometry, opened at a half, life size and twice, on displays at 100, 150
/// and 200 per cent, and each looked at afterwards at all three zooms. What is
/// drawn of each is measured with the painter's own layout of the strip, so a
/// placer that kept the gap to the window and not its strip, or a strip that
/// did not zoom with its window, fails here.
#[test]
fn a_window_opened_beside_alos_terminal_keeps_the_gap_from_its_strip_at_every_zoom() {
    let terminal = Rectangle::<i32, Logical>::new((178, 92).into(), (924, 614).into());
    let wanted = Size::<i32, Logical>::from((760, 566));
    let zooms = [0.5, 1.0, 2.0];
    for scale in [100, 150, 200] {
        for zoom in zooms {
            let (_directory, mut server) = a_server("beside-the-terminal");
            #[expect(
                clippy::cast_possible_truncation,
                reason = "a display divided by a zoom of at least a half fits an i32"
            )]
            let view = Rectangle::<i32, Logical>::new(
                (0, 0).into(),
                ((1280.0 / zoom) as i32, (800.0 / zoom) as i32).into(),
            );
            let strip = crate::a_window_and_its_strip::ItsStrip::at(scale);
            let reading = alo_strings::Direction::LeftToRight;
            let first =
                strip.where_a_window_opens(view, &[terminal], Some(terminal), wanted, reading, &[]);
            let at = server.room_for_what_the_person_opened(
                first,
                &AnOpening {
                    view,
                    open: &[terminal],
                    wanted,
                    fixed: &[],
                    reading,
                    strip,
                },
                Instant::now(),
            );
            let gap = f64::from(crate::where_a_window_opens::A_SMALL_GAP);
            for seen_at in zooms {
                let new = drawn(Rectangle::new(at, wanted), seen_at, scale);
                let old = drawn(terminal, seen_at, scale);
                assert!(
                    apart(new, old) >= gap - 1.0,
                    "opened at zoom {zoom} on {scale} %, seen at {seen_at}: {new:?} is {} from                      {old:?}, not the gap of {gap}",
                    apart(new, old)
                );
            }
        }
    }
}

/// **Every placer takes the strip into account** — in the part of each file
/// that ships. A placer handed windows without their strips is the fault of
/// 2026-11-08 arriving by a sixth road.
#[test]
fn every_placer_counts_the_strip_above_each_window() {
    let shipped = |text: &'static str| text.split("#[cfg(test)]").next().unwrap_or_default();
    for (named, text) in [
        ("the terminal", include_str!("the_terminal_on_the_plane.rs")),
        ("Settings", include_str!("settings_on_the_plane.rs")),
        ("the record", include_str!("record_on_the_plane.rs")),
        ("an application", include_str!("a_new_window_is_placed.rs")),
        (
            "a task window",
            include_str!("the_task_windows_find_room.rs"),
        ),
    ] {
        let text = shipped(text);
        assert!(
            text.contains("ItsStrip::at("),
            "{named} does not count the strip above each window"
        );
        assert!(
            !text.contains("crate::where_a_window_opens("),
            "{named} asks rules 3 to 7 about windows without their strips"
        );
    }
}
