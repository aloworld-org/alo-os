//! Every window on the Place, the task windows included — and a window the
//! person opens over a view a task window fills covering none of it.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;

use alo_models::InferenceSource;
use alo_strings::Direction;
use alo_tasks::ATask;
use smithay::utils::{Logical, Physical, Rectangle};

use crate::Server;

/// The display the walk of 2026-11-08 was on.
const A_DISPLAY: (i32, i32) = (1280, 800);

/// A session with one task window placed in the view, and where it went.
fn a_task_in_the_view(named: &str) -> (tempfile::TempDir, Server, Rectangle<i32, Logical>) {
    let directory = tempfile::tempdir().expect("a temporary directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind(directory.path(), named).expect("a bound socket");
    server.surfaces.popups.output_size = Some(smithay::utils::Size::from(A_DISPLAY));
    let which = server.tasks_mut().and_now_this_one(ATask::asked_for(
        "tidy my invoices",
        InferenceSource::ThisMachine,
    ));
    let view = Rectangle::new((0, 0).into(), A_DISPLAY.into());
    server
        .tasks_mut()
        .put_whatever_has_nowhere_to_be(view, Direction::LeftToRight, &[], 100);
    let task = server
        .tasks()
        .where_each_went()
        .get(which.at())
        .copied()
        .flatten()
        .expect("the control: the task window was placed");
    let task = Rectangle::new(
        (task.loc.x, task.loc.y).into(),
        (task.size.w, task.size.h).into(),
    );
    (directory, server, task)
}

/// **A task window is among the windows on the Place**, which is what alo's
/// own windows are placed against.
#[test]
fn a_task_window_is_one_of_the_windows_on_this_place() {
    let (_directory, server, task) = a_task_in_the_view("task-on-the-place");
    assert!(
        server.the_windows_on_this_place().contains(&task),
        "the task window {task:?} is missing from {:?}",
        server.the_windows_on_this_place()
    );
}

/// **The refusal: a task window on another Place is not in the way here.**
#[test]
fn a_task_window_on_another_place_is_not_in_the_way() {
    let (_directory, mut server, task) = a_task_in_the_view("task-elsewhere");
    server.surfaces.place = alo_canvas::Place::FIRST.next().expect("a second Place");
    assert!(
        !server.the_windows_on_this_place().contains(&task),
        "a task window on another Place was counted here"
    );
}

/// **Settings opened over a view a task window fills covers neither it nor
/// its strip** — until 2026-11-08 the task window was not in the list Settings
/// was placed against, so Settings found the view empty and opened on it.
#[test]
fn settings_opened_over_a_task_window_covers_none_of_it() {
    let (_directory, mut server, task) = a_task_in_the_view("settings-over-a-task");
    let machine = crate::settings_testing::a_persons_machine("settings-over-a-task");
    server.the_settings_places_are(machine.places.clone());
    server.settings = machine.opened();
    assert!(server.settings_is_placed((1152, 688), A_DISPLAY, Direction::LeftToRight));
    let settings = server
        .where_settings_is_on_the_plane()
        .expect("Settings is on this Place");
    let strip = crate::a_window_and_its_strip::ItsStrip::at(1.0, 100).expect("life size");
    assert!(
        !strip.round(settings).overlaps(strip.round(task)),
        "Settings {settings:?} covers the task window {task:?} or its strip"
    );
    let _ =
        server.the_glide_moves_on(std::time::Instant::now() + std::time::Duration::from_secs(5));
    let seen = crate::the_terminal_on_the_plane::on_the_screen(settings, server.the_camera());
    assert!(
        Rectangle::<i32, Physical>::from_size(A_DISPLAY.into()).contains_rect(seen),
        "after the view moved Settings is not in it: {seen:?}"
    );
}
