//! That the wheel over a task window scrolls its conversation and reaches
//! nothing underneath, that it stops at the oldest words, and that a page key
//! moves a page.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt;

use alo_models::InferenceSource;
use alo_tasks::ATask;
use smithay::backend::input::{Axis, AxisSource};
use smithay::input::keyboard::XkbConfig;
use smithay::input::pointer::AxisFrame;

use super::{APage, how_far_back};
use crate::Server;
use crate::the_tasks_a_person_opened::WhichTask;

/// A wheel turned this far, positive downwards.
fn a_scroll_of(down: f64) -> AxisFrame {
    AxisFrame::new(0)
        .source(AxisSource::Wheel)
        .value(Axis::Vertical, down)
}

/// A session with a keyboard and a pointer, and one placed task window whose
/// conversation a frame measured as able to go 600 back in a view 300 tall.
fn a_long_conversation() -> (tempfile::TempDir, Server, WhichTask) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind_keyboard(
        runtime.path(),
        "a-task-window-scrolls",
        XkbConfig {
            layout: "us",
            ..XkbConfig::default()
        },
    )
    .expect("a display with a keyboard");
    server.enable_pointer().expect("a pointer on this seat");
    let which = server.tasks_mut().and_now_this_one(ATask::asked_for(
        "explain what an invoice is",
        InferenceSource::ThisMachine,
    ));
    let view = smithay::utils::Rectangle::new((0, 0).into(), (1280, 800).into());
    server.tasks_mut().put_whatever_has_nowhere_to_be(
        view,
        alo_strings::Direction::LeftToRight,
        &[],
        100,
    );
    server
        .tasks_mut()
        .the_mut(which)
        .expect("the task is open")
        .scroll_mut()
        .the_frame_measured(600, 300);
    (runtime, server, which)
}

/// Where this task's view starts, for a conversation whose newest is at 600.
fn the_view_starts(server: &Server, which: WhichTask) -> u32 {
    let task = server.tasks().the(which).expect("the task is open");
    task.scroll()
        .the_view_starts(600, task.conversation().each().len())
}

/// Move the pointer to this point inside the task window, or outside it.
fn point_at(server: &mut Server, which: WhichTask, inside: bool) {
    let window = server
        .tasks()
        .where_each_went()
        .get(which.at())
        .copied()
        .flatten()
        .expect("the task window was placed");
    let (x, y) = if inside {
        (window.loc.x + 40, window.loc.y + 200)
    } else if window.loc.x >= 20 {
        (window.loc.x - 10, window.loc.y + 200)
    } else {
        (window.loc.x + window.size.w + 10, window.loc.y + 200)
    };
    server
        .pointer_motion(f64::from(x), f64::from(y), 1)
        .expect("the pointer moved");
}

/// **A wheel pushed away scrolls back by a notch, and the scroll is the task
/// window's.**
#[test]
fn the_wheel_over_a_task_window_scrolls_its_conversation() {
    let (_runtime, mut server, which) = a_long_conversation();
    assert_eq!(
        the_view_starts(&server, which),
        600,
        "the control: at the newest"
    );
    point_at(&mut server, which, true);

    let moved = server
        .pointer_axis(a_scroll_of(-15.0))
        .expect("the scroll was handled");

    assert!(
        moved,
        "a scroll that moved the conversation said it did not"
    );
    assert_eq!(
        the_view_starts(&server, which),
        600 - alo_tasks::measures::A_NOTCH_SCROLLS
    );
}

/// **A wheel away from every task window leaves their conversations alone.**
#[test]
fn the_wheel_elsewhere_scrolls_no_task_window() {
    let (_runtime, mut server, which) = a_long_conversation();
    point_at(&mut server, which, false);

    let _ = server
        .pointer_axis(a_scroll_of(-15.0))
        .expect("the scroll was handled");

    assert_eq!(
        the_view_starts(&server, which),
        600,
        "a task window scrolled from outside it"
    );
}

/// **At the newest, scrolling further forward moves nothing and says so** —
/// and still does not reach anything under the window.
#[test]
fn at_the_newest_a_forward_scroll_moves_nothing() {
    let (_runtime, mut server, which) = a_long_conversation();
    point_at(&mut server, which, true);

    let moved = server
        .pointer_axis(a_scroll_of(15.0))
        .expect("the scroll was handled");

    assert!(!moved, "a scroll at the newest said it moved");
    assert_eq!(the_view_starts(&server, which), 600);
}

/// **A page back moves the view less one line, and a page forward returns.**
#[test]
fn a_page_moves_the_view_less_one_line() {
    let (_runtime, mut server, which) = a_long_conversation();

    assert!(server.a_page_in_the_task(which, APage::Back));
    assert_eq!(
        the_view_starts(&server, which),
        600 - (300 - alo_tasks::measures::A_LINE_OF_WHAT_WAS_SAID)
    );
    assert!(server.a_page_in_the_task(which, APage::Forward));
    assert_eq!(the_view_starts(&server, which), 600);
}

/// **A notch is a notch's worth**, and a scroll that is not a number moves
/// nothing.
#[test]
fn a_notch_is_three_lines_and_nonsense_is_nothing() {
    let notch = i32::try_from(alo_tasks::measures::A_NOTCH_SCROLLS).expect("a small figure");
    assert_eq!(how_far_back(&a_scroll_of(-15.0)), notch);
    assert_eq!(how_far_back(&a_scroll_of(15.0)), -notch);
    let mut nonsense = a_scroll_of(0.0);
    nonsense.axis.1 = f64::NAN;
    assert_eq!(how_far_back(&nonsense), 0);
}
