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

use super::{in_the_edges_units, the_edge_of_a_task_window};
use crate::Server;
use crate::the_tasks_a_person_opened::WhichTask;
use crate::window_edge::OnTheEdge;

/// A room no edge is clamped by.
fn anywhere() -> Size<i32, Logical> {
    Size::from((10_000, 10_000))
}

/// A session with a pointer, and one task window placed on a 1280 × 800 view.
fn a_task_window() -> (tempfile::TempDir, Server, WhichTask) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind(runtime.path(), "a-task-windows-edge").expect("a display");
    server.enable_pointer().expect("a pointer on this seat");
    let which = server.tasks_mut().and_now_this_one(ATask::asked_for(
        "tidy my invoices",
        InferenceSource::ThisMachine,
    ));
    let view = Rectangle::new((0, 0).into(), (1280, 800).into());
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
    let window = server
        .tasks()
        .each_open_window()
        .into_iter()
        .find(|(one, _)| *one == which)
        .map(|(_, at)| at)
        .expect("the window is open");
    let edge = the_edge_of_a_task_window(in_the_edges_units(window, 100), anywhere());
    let close = edge
        .controls
        .iter()
        .find(|control| control.does == OnTheEdge::Close)
        .expect("a Close on the edge")
        .target;
    let (x, y) = (
        close.loc.x + close.size.w / 2,
        close.loc.y + close.size.h / 2,
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
