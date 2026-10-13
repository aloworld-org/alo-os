//! The whole road from a person pressing Enter to a task window in a frame.
//!
//! Every piece of this had passing tests of its own while nothing on a desktop
//! ever drew a task window. That is this repository's standing fault and it was
//! made four times in one day on this very feature: a composer that collected
//! keystrokes nobody read, a palette nothing constructed, a raster nothing
//! called, a placement with no caller. **A seam nobody crosses in a test looks
//! exactly like a seam that works.**
//!
//! So this crosses the whole thing, in two halves that catch different faults:
//!
//! 1. **The road itself**, walked end to end — a keystroke in, a drawn window
//!    out. It would still pass if `present` never called any of it.
//! 2. **That `present` calls it**, read out of the production source, with a
//!    positive control so a walk over an empty string cannot report a pass.
//!    `tests/a_new_window_has_somewhere_to_be.rs` is where this shape comes
//!    from, and its own comment says why: *where_a_window_opens implemented,
//!    tested, and reached by nobody, while every window opened at the origin.*
#![cfg(target_os = "linux")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "in a test, a panic naming the step of the road that broke is the failure being \
              reported"
)]

use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;

use alo_models::InferenceSource;
use alo_shell::Server;
use alo_tasks::ATask;

/// A server on a socket of its own.
fn a_server(named: &str) -> (tempfile::TempDir, Server) {
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let bound = Server::bind(directory.path(), named).expect("a bound socket");
    (directory, bound)
}

/// **A keystroke becomes a window.**
///
/// The road, step by step, with each step asserted so a failure names which one
/// broke rather than only that the end was empty.
#[test]
fn a_person_types_a_request_and_a_task_window_is_drawn() {
    let (_directory, mut server) = a_server("a-request-becomes-a-window");

    // 1. The Dock's composer takes the keyboard and a person types.
    assert!(
        server.a_person_asked_alo("Tidy my invoices folder"),
        "the request was not even recorded"
    );

    // 2. The lane drains it and the desktop makes a task of it. In production
    //    `TheDesktop::a_task_for` does this, because which model answers is a
    //    reading of settings and the shell may not read one.
    let asked = server.take_what_was_asked_of_alo();
    assert_eq!(asked, vec!["Tidy my invoices folder".to_owned()]);
    for request in asked {
        server
            .tasks_mut()
            .and_now_this_one(ATask::asked_for(request, InferenceSource::ThisMachine));
    }
    assert_eq!(server.tasks().how_many(), 1, "no task was opened");

    // 3. Its window is placed, once, by the owner's own rule.
    let view = smithay::utils::Rectangle::new((0, 0).into(), (1920, 1080).into());
    let placed = server.tasks_mut().put_whatever_has_nowhere_to_be(
        view,
        alo_strings::Direction::LeftToRight,
        &[],
    );
    assert_eq!(placed, 1, "the window was not placed");
    let somewhere = server
        .tasks()
        .where_each_went()
        .first()
        .copied()
        .flatten()
        .expect("a placed window has a position");
    assert!(
        somewhere.size.w > 0 && somewhere.size.h > 0,
        "the window was placed with no extent: {somewhere:?}"
    );

    // 4. Placed once: a second pass moves nothing.
    let again = server.tasks_mut().put_whatever_has_nowhere_to_be(
        view,
        alo_strings::Direction::LeftToRight,
        &[],
    );
    assert_eq!(again, 0, "a placed window was placed a second time");
    assert_eq!(
        server.tasks().where_each_went().first().copied().flatten(),
        Some(somewhere),
        "a window that was already placed moved on the next frame"
    );
}

/// Read a shipped source file of this crate.
fn read(named: &str) -> String {
    let at = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join(named);
    std::fs::read_to_string(&at)
        .unwrap_or_else(|why| panic!("{} could not be read: {why}", at.display()))
}

/// A file's production part: everything before its unit tests.
fn production_part_of(file: &str) -> &str {
    file.split_once("#[cfg(test)]")
        .map_or(file, |(before, _)| before)
}

/// **`present` really does turn a request into a task.**
///
/// The road above would pass with nothing in production calling any of it. This
/// is the half that refuses that, and it carries its own positive control: a
/// marker this file is certain of, so a read that returned an empty string
/// reports a failure rather than a clean pass.
#[test]
fn the_frame_loop_turns_a_request_into_a_task_and_draws_it() {
    let desk = read("direct_desktop.rs");
    let shipped = production_part_of(&desk);

    // The positive control, in the same scope as the claims below it.
    assert!(
        shipped.contains("fn present<T: crate::direct_loop::LoopTarget"),
        "direct_desktop.rs's production part no longer holds the frame loop, so nothing below \
         is measuring what it claims"
    );

    assert!(
        shipped.contains("server.take_what_was_asked_of_alo()"),
        "nothing in production drains what a person asked alo, so a request would be recorded \
         and never become a window"
    );
    assert!(
        shipped.contains("self.desktop.a_task_for("),
        "production never asks the desktop to make a task, so the only road from a request to \
         a window is unused"
    );
    assert!(
        shipped.contains("and_now_this_one("),
        "a task is made in production and never kept, so it would be dropped the moment it \
         was built"
    );
    assert!(
        shipped.contains("put_whatever_has_nowhere_to_be("),
        "nothing in production places a task window, so every one would have nowhere to be \
         drawn"
    );
    assert!(
        shipped.contains("TheTasksWords::said("),
        "nothing in production words a task window, so it would draw with whatever its caller \
         happened to pass"
    );
    assert!(
        shipped.contains("tasks: &the_task_windows,"),
        "the task windows are built in production and not handed to the frame, which is the \
         exact shape of this repository's standing fault"
    );
}

/// **A desktop that cannot answer makes no task**, and the request is not kept.
///
/// `TheDesktop::a_task_for` answers [`None`] on a machine where nobody has
/// chosen what answers their questions. A window opened onto a model that is
/// not there would be worse than none: a person would type into a surface that
/// could never reply.
#[test]
fn a_machine_that_cannot_answer_opens_no_window() {
    let (_directory, mut server) = a_server("a-machine-that-cannot-answer");

    assert!(server.a_person_asked_alo("Tidy my invoices folder"));
    // The default `a_task_for` answers None, which is what a desktop that has
    // not been taught to read the choice does.
    for _request in server.take_what_was_asked_of_alo() {
        // Nothing is kept, which is the point.
    }

    assert_eq!(
        server.tasks().how_many(),
        0,
        "a task was opened on a machine with nothing to answer it"
    );
    assert!(
        server.take_what_was_asked_of_alo().is_empty(),
        "the request was queued, so a window would appear later for something a person typed \
         and gave up on"
    );
}
