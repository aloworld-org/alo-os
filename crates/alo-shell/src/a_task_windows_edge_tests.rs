//! That a task window's edge carries Close alone, at the design's figures and
//! scaled, and that a press on Close follows Alt+F4's rules.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;

use alo_models::InferenceSource;
use alo_tasks::ATask;
use alo_tasks::conversation::Turn;
use smithay::backend::input::ButtonState;
use smithay::utils::{Logical, Rectangle, Size};

use super::the_edge_of_a_task_window;
use crate::Server;
use crate::an_edge_in_its_displays_units::a_window_in_the_edges_units as in_the_edges_units;
use crate::the_tasks_a_person_opened::WhichTask;
use crate::window_edge::OnTheEdge;

/// A room no edge is clamped by.
fn anywhere() -> Size<i32, Logical> {
    Size::from((10_000, 10_000))
}

/// A session with a pointer, and one task window placed on a 1280 × 800 view.
fn a_task_window() -> (tempfile::TempDir, Server, WhichTask) {
    a_task_window_on((1280, 800))
}

/// The same, on a view of `size`.
fn a_task_window_on(size: (i32, i32)) -> (tempfile::TempDir, Server, WhichTask) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind_keyboard(
        runtime.path(),
        "a-task-windows-edge",
        smithay::input::keyboard::XkbConfig {
            layout: "us",
            ..smithay::input::keyboard::XkbConfig::default()
        },
    )
    .expect("a display with a keyboard");
    server.enable_pointer().expect("a pointer on this seat");
    let which = server.tasks_mut().and_now_this_one(ATask::asked_for(
        "tidy my invoices",
        InferenceSource::ThisMachine,
    ));
    let view = Rectangle::new((0, 0).into(), size.into());
    server.tasks_mut().put_whatever_has_nowhere_to_be(
        view,
        alo_strings::Direction::LeftToRight,
        &[],
    );
    (runtime, server, which)
}

/// Press the primary button in the middle of this task's Close, through the
/// pointer road a person's press takes.
fn press_close(server: &mut Server, which: WhichTask) -> bool {
    press_close_at(server, which, 100)
}

/// The same, on a display drawn at `scale` per cent: the press goes to the
/// middle of Close **where it is drawn there**, which is its place in the
/// edge's units multiplied back by the scale, as `EdgePicture::of` draws it.
fn press_close_at(server: &mut Server, which: WhichTask, scale: u16) -> bool {
    server.the_scale_edges_are_measured_at = scale;
    let window = server
        .tasks()
        .each_open_window()
        .into_iter()
        .find(|(one, _)| *one == which)
        .map(|(_, at)| at)
        .expect("the window is open");
    let edge = the_edge_of_a_task_window(in_the_edges_units(window, u32::from(scale)), anywhere());
    let close = edge
        .controls
        .iter()
        .find(|control| control.does == OnTheEdge::Close)
        .expect("a Close on the edge")
        .target;
    let drawn = |units: i32| units * i32::from(scale) / 100;
    let (x, y) = (
        drawn(close.loc.x) + drawn(close.size.w) / 2,
        drawn(close.loc.y) + drawn(close.size.h) / 2,
    );
    server
        .pointer_motion(f64::from(x), f64::from(y), 1)
        .expect("the pointer moved");
    server
        .pointer_button(0x110, ButtonState::Pressed, 2)
        .expect("the press was handled")
}

/// **Close, and nothing else, at the design's 44 × 44 at the trailing end of a
/// 44-tall edge that sits on the window.**
#[test]
fn the_edge_carries_close_alone_at_the_designs_figures() {
    let window = Rectangle::new((200, 124).into(), (1040, 800).into());
    let edge = the_edge_of_a_task_window(window, anywhere());

    let does: Vec<OnTheEdge> = edge.controls.iter().map(|control| control.does).collect();
    assert_eq!(
        does,
        [OnTheEdge::Close],
        "minimise, maximise or move was drawn"
    );
    assert_eq!(edge.region.size.h, 44);
    assert_eq!(
        edge.region.loc.y + edge.region.size.h,
        window.loc.y,
        "the edge does not meet the window"
    );
    let close = edge.controls.first().expect("Close").target;
    assert_eq!(close.size, Size::from((44, 44)));
    assert!(
        close.loc.x + close.size.w <= window.loc.x + window.size.w
            && close.loc.x + close.size.w >= window.loc.x + window.size.w - 8,
        "Close is not at the trailing end: {close:?}"
    );
    assert_eq!(edge.title, None, "a task window's edge drew a title");
}

/// **At twice the scale the edge still meets the window**, because the
/// window's pixels are halved into the edge's units and doubled back.
#[test]
fn at_twice_the_scale_the_edge_still_meets_the_window() {
    let pixels = Rectangle::new((400, 248).into(), (2080, 1600).into());
    let units = in_the_edges_units(pixels, 200);

    assert_eq!(units, Rectangle::new((200, 124).into(), (1040, 800).into()));
    let edge = the_edge_of_a_task_window(units, anywhere());
    assert_eq!(
        (edge.region.loc.y + edge.region.size.h) * 2,
        pixels.loc.y,
        "scaled back, the edge's foot is not the window's top"
    );
}

/// **A press on Close closes a finished task**, and nothing behind it hears
/// the press.
#[test]
fn a_press_on_close_closes_a_finished_task() {
    let (_runtime, mut server, which) = a_task_window();
    server
        .tasks_mut()
        .the_mut(which)
        .expect("open")
        .alo_says(Turn::from_alo("Your invoices are tidy."));

    let delivered = press_close(&mut server, which);

    assert!(
        !delivered,
        "the press reached an application behind the edge"
    );
    assert!(!server.tasks().is_open(which), "Close did not close it");
}

/// **At 200 per cent a press where Close is drawn closes the task** — the
/// owner's report of 2026-10-28: Close was drawn at the display's scale and a
/// press was read against the edge laid out at 100, so the two disagreed.
///
/// Pressed a quarter of the way into the drawn button, which is the part of it
/// the old reading missed: asserted below to lie outside Close as the edge was
/// laid out at 100, so this test fails on the code it replaces.
#[test]
fn at_two_hundred_a_press_where_close_is_drawn_closes_the_task() {
    let (_runtime, mut server, which) = a_task_window_on((2560, 1600));
    server
        .tasks_mut()
        .the_mut(which)
        .expect("open")
        .alo_says(Turn::from_alo("Your invoices are tidy."));
    let window = server
        .tasks()
        .each_open_window()
        .into_iter()
        .find(|(one, _)| *one == which)
        .map(|(_, at)| at)
        .expect("the window is open");
    let close_at = |scale: u32| {
        the_edge_of_a_task_window(in_the_edges_units(window, scale), anywhere())
            .controls
            .iter()
            .find(|control| control.does == OnTheEdge::Close)
            .expect("a Close on the edge")
            .target
    };
    let units = close_at(200);
    let drawn = Rectangle::<i32, Logical>::new(
        (units.loc.x * 2, units.loc.y * 2).into(),
        (units.size.w * 2, units.size.h * 2).into(),
    );
    assert_eq!(
        drawn.size,
        Size::from((88, 88)),
        "Close is drawn twice the design's 44"
    );
    let (x, y) = (
        drawn.loc.x + drawn.size.w / 4,
        drawn.loc.y + drawn.size.h / 4,
    );
    let at_one = close_at(100);
    assert!(
        !(at_one.loc.x..at_one.loc.x + at_one.size.w).contains(&x)
            || !(at_one.loc.y..at_one.loc.y + at_one.size.h).contains(&y),
        "the premise: ({x}, {y}) is outside Close as laid out at 100"
    );
    server.the_scale_edges_are_measured_at = 200;
    server
        .pointer_motion(f64::from(x), f64::from(y), 1)
        .expect("the pointer moved");
    let delivered = server
        .pointer_button(0x110, ButtonState::Pressed, 2)
        .expect("the press was handled");

    assert!(
        !delivered,
        "the press reached an application behind the edge"
    );
    assert!(
        !server.tasks().is_open(which),
        "a press where Close is drawn at 200 did not close it"
    );
}

/// **The same press helper at 200 lands on Close's middle too**, for the
/// refusal: a running task is not closed at 200 either.
#[test]
fn at_two_hundred_close_still_leaves_a_running_task_open() {
    let (_runtime, mut server, which) = a_task_window_on((2560, 1600));
    let delivered = press_close_at(&mut server, which, 200);
    assert!(
        !delivered,
        "the press reached an application behind the edge"
    );
    assert!(server.tasks().is_open(which), "a running task was closed");
}

/// **A press on Close does not close a running task**, and asks nothing to
/// stop — the same refusal Alt+F4 makes.
#[test]
fn a_press_on_close_leaves_a_running_task_open() {
    let (_runtime, mut server, which) = a_task_window();

    let delivered = press_close(&mut server, which);

    assert!(
        !delivered,
        "the press reached an application behind the edge"
    );
    assert!(server.tasks().is_open(which), "a running task was closed");
    assert!(
        !server
            .tasks()
            .the(which)
            .expect("open")
            .stopping()
            .is_waiting(),
        "pressing Close asked the work to stop"
    );
}

/// **A closed window's edge is gone with it**: a press where its Close was
/// is not taken by the edge.
#[test]
fn a_closed_windows_edge_is_gone() {
    let (_runtime, mut server, which) = a_task_window();
    server
        .tasks_mut()
        .the_mut(which)
        .expect("open")
        .alo_says(Turn::from_alo("Done."));
    let window = server
        .tasks()
        .each_open_window()
        .first()
        .map(|(_, at)| *at)
        .expect("the window is open");
    let edge = the_edge_of_a_task_window(in_the_edges_units(window, 100), anywhere());
    let close = edge.controls.first().expect("Close").target;

    press_close(&mut server, which);
    assert!(
        !server.tasks().is_open(which),
        "the control: Close closed it"
    );
    assert!(
        !server.a_press_on_a_task_windows_edge((close.loc.x + 1, close.loc.y + 1).into()),
        "the edge of a closed window still took a press"
    );
}
