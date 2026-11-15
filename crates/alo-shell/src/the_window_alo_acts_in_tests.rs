//! The window alo acts in wears alo for the turn it acts in, on no other
//! window, and is bare again when that turn ends — and the refusals: no task
//! at work, two at once, a task whose window was closed.
//!
//! **What is substituted.** The windows are protocol identities that were
//! never mapped, so which window an act really reached is not here; that is
//! `tests/typing_into_the_window_in_front`'s, on real clients, and the walk's
//! on a real VSCodium. What these hold is the rule: which task a window wears,
//! when, and when it stops.

#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;

use alo_models::InferenceSource;
use alo_tasks::ATask;
use alo_tasks::conversation::Turn;
use smithay::input::keyboard::XkbConfig;
use smithay::reexports::wayland_server::{
    Client, DataInit, Dispatch, Display, DisplayHandle, Resource,
    protocol::wl_surface::{self, WlSurface},
};

use crate::Server;
use crate::alo_in_a_window::HowAloIsHere;
use crate::the_tasks_a_person_opened::WhichTask;

/// Surfaces with identities and nothing else: never dispatched.
struct Identities;
impl Dispatch<WlSurface, ()> for Identities {
    fn request(
        _: &mut Self,
        _: &Client,
        surface: &WlSurface,
        _: wl_surface::Request,
        _: &(),
        _: &DisplayHandle,
        _: &mut DataInit<'_, Self>,
    ) {
        surface.post_error(0u32, "an identity fixture takes no requests");
    }
}

/// Two windows, and what keeps them alive.
fn two_windows() -> (
    Display<Identities>,
    std::os::unix::net::UnixStream,
    WlSurface,
    WlSurface,
) {
    let display = Display::<Identities>::new().expect("a display");
    let mut handle = display.handle();
    let (socket, peer) = std::os::unix::net::UnixStream::pair().expect("a socket pair");
    let client = handle
        .insert_client(socket, std::sync::Arc::new(()))
        .expect("a client");
    let one = client
        .create_resource::<WlSurface, (), Identities>(&handle, 1, ())
        .expect("a surface");
    let two = client
        .create_resource::<WlSurface, (), Identities>(&handle, 1, ())
        .expect("a surface");
    (display, peer, one, two)
}

/// A session with no task in it.
fn a_session(named: &str) -> (tempfile::TempDir, Server) {
    let runtime = tempfile::tempdir().expect("a runtime directory of this test's own");
    std::fs::set_permissions(runtime.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let server = Server::bind_keyboard(
        runtime.path(),
        named,
        XkbConfig {
            layout: "us",
            ..XkbConfig::default()
        },
    )
    .expect("a display with a keyboard");
    (runtime, server)
}

/// A task the person asked for in the Bar, which alo is working on.
fn a_task_asked(server: &mut Server, request: &str) -> WhichTask {
    server
        .tasks_mut()
        .and_now_this_one(ATask::asked_for(request, InferenceSource::ThisMachine))
}

const ASKED: &str = "Open hello.txt in VSCodium and type a line into it";

/// **The window alo acts in wears the task at work, in the person's own words,
/// and no other window does.**
///
/// What would have to be true for this to pass while broken: the shell not
/// calling `alo_acts_in` where alo acts — which this cannot see, and
/// `tests/typing_into_the_window_in_front` and the walk do.
#[test]
fn the_window_alo_acts_in_wears_the_task_at_work_and_no_other_does() {
    let (_display, _peer, codium, other) = two_windows();
    let (_runtime, mut server) = a_session("alo-acts-in-one-window");
    let which = a_task_asked(&mut server, ASKED);
    assert!(
        server.alo_in_this_window(&codium).is_none(),
        "a window wore alo before alo acted in it"
    );

    assert_eq!(server.alo_acts_in(&codium), Some(which));

    let here = server
        .alo_in_this_window(&codium)
        .expect("the window alo acts in wears alo");
    assert_eq!(here.which, which);
    assert_eq!(here.how, HowAloIsHere::Working);
    assert_eq!(here.status, alo_tasks::words::ALO_IS_WORKING);
    assert_eq!(here.what, ASKED);
    assert_eq!(
        here.wears,
        crate::alo_in_a_window::Wears::TheBorder,
        "a window alo chose wore the band: the owner ruled a border alone"
    );
    assert!(
        server.alo_in_this_window(&other).is_none(),
        "a window alo did not act in was marked"
    );
}

/// **No task at work, no presence**: with no task, or with the only task
/// finished, an act marks nothing — the window is never told it is alo's when
/// no request of the person's is being worked on.
#[test]
fn nothing_is_marked_when_no_task_is_at_work() {
    let (_display, _peer, codium, _other) = two_windows();
    let (_runtime, mut server) = a_session("alo-acts-with-no-task");
    assert_eq!(server.alo_acts_in(&codium), None);
    assert!(server.alo_in_this_window(&codium).is_none());

    let which = a_task_asked(&mut server, ASKED);
    server
        .tasks_mut()
        .the_mut(which)
        .expect("open")
        .alo_says(Turn::from_alo("Done."));
    assert_eq!(server.alo_acts_in(&codium), None);
    assert!(
        server.alo_in_this_window(&codium).is_none(),
        "a finished task's presence was put on a window"
    );
}

/// **Two tasks at work and the window wears neither**: which one acted cannot
/// be told, and a band naming the wrong request, with a Stop that stops the
/// wrong work, is worse than none. The task windows still say both are working.
#[test]
fn two_tasks_at_work_mark_no_window() {
    let (_display, _peer, codium, _other) = two_windows();
    let (_runtime, mut server) = a_session("alo-acts-for-two");
    let _first = a_task_asked(&mut server, ASKED);
    let _second = a_task_asked(&mut server, "Summarise my notes");
    assert_eq!(server.alo_acts_in(&codium), None);
    assert!(server.alo_in_this_window(&codium).is_none());
}

/// **A task whose window the person closed is not the one at work**, so a
/// second task's act is still told apart.
#[test]
fn a_closed_task_is_not_the_one_at_work() {
    let (_display, _peer, codium, _other) = two_windows();
    let (_runtime, mut server) = a_session("alo-acts-after-a-close");
    let first = a_task_asked(&mut server, "Summarise my notes");
    assert!(server.tasks_mut().closed(first));
    let second = a_task_asked(&mut server, ASKED);
    assert_eq!(server.alo_acts_in(&codium), Some(second));
}

/// **Waiting on the person keeps the window, in navy**: a change waiting for
/// their OK is part of the same turn, so the window stays alo's — not acting,
/// so not teal — and the frame does not let it go.
#[test]
fn waiting_for_the_person_keeps_the_window_and_is_not_acting() {
    let (_display, _peer, codium, _other) = two_windows();
    let (_runtime, mut server) = a_session("alo-acts-then-asks");
    let which = a_task_asked(&mut server, ASKED);
    assert_eq!(server.alo_acts_in(&codium), Some(which));
    {
        let task = server.tasks_mut().the_mut(which).expect("open");
        task.is_asking("Save hello.txt");
        task.alo_says(Turn::alo_proposing("Save hello.txt"));
    }
    let over = server
        .the_windows_alo_acts_in
        .looked_at(std::slice::from_ref(&codium), &server.the_tasks);
    assert!(over.is_empty(), "the window was let go mid-turn: {over:?}");
    assert_eq!(
        server.alo_in_this_window(&codium).map(|here| here.how),
        Some(HowAloIsHere::WaitingForThePerson)
    );
}

/// **The presence goes when the turn ends, and a follow-up does not bring it
/// back**: the window was alo's for the turn that acted in it. A follow-up is a
/// new turn, which wears the window again only if alo acts in it again — unlike
/// a window the person offered, which every turn of its task is about.
#[test]
fn the_presence_goes_when_the_turn_ends_and_a_follow_up_does_not_bring_it_back() {
    let (_display, _peer, codium, _other) = two_windows();
    let (_runtime, mut server) = a_session("alo-acts-and-answers");
    let which = a_task_asked(&mut server, ASKED);
    assert_eq!(server.alo_acts_in(&codium), Some(which));
    server
        .tasks_mut()
        .the_mut(which)
        .expect("open")
        .alo_says(Turn::from_alo("It is written and saved."));
    assert!(
        server.alo_in_this_window(&codium).is_none(),
        "the window still wore alo after its turn ended"
    );
    let over = server
        .the_windows_alo_acts_in
        .looked_at(std::slice::from_ref(&codium), &server.the_tasks);
    assert_eq!(over.len(), 1, "the end of the turn was not said once");

    server
        .tasks_mut()
        .the_mut(which)
        .expect("open")
        .the_person_says("thank you");
    assert!(
        server.alo_in_this_window(&codium).is_none(),
        "a follow-up put alo back on a window it has not acted in this turn"
    );
    // Acting again in the new turn wears it again.
    assert_eq!(server.alo_acts_in(&codium), Some(which));
    assert!(server.alo_in_this_window(&codium).is_some());
}

/// **A window that has gone takes its presence with it, said once**, and the
/// task goes on working in its own window.
#[test]
fn a_window_that_has_gone_is_let_go_once_and_the_work_goes_on() {
    let (_display, _peer, codium, _other) = two_windows();
    let (_runtime, mut server) = a_session("alo-acts-in-a-window-that-goes");
    let which = a_task_asked(&mut server, ASKED);
    assert_eq!(server.alo_acts_in(&codium), Some(which));
    let over = server
        .the_windows_alo_acts_in
        .looked_at(&[], &server.the_tasks);
    assert_eq!(over.len(), 1);
    let again = server
        .the_windows_alo_acts_in
        .looked_at(&[], &server.the_tasks);
    assert!(again.is_empty(), "a window let go was said twice");
    assert!(server.alo_in_this_window(&codium).is_none());
    assert_eq!(
        server.tasks().the(which).map(ATask::what_is_happening),
        Some(alo_tasks::words::ALO_IS_WORKING),
        "the window going stopped the work"
    );
}

/// **Acting twice in one window in one turn is one presence**, said on the
/// log once — typing and then saving is one piece of work in one window.
#[test]
fn acting_twice_in_one_turn_is_one_presence() {
    let (_display, _peer, codium, _other) = two_windows();
    let (_runtime, mut server) = a_session("alo-acts-twice");
    let which = a_task_asked(&mut server, ASKED);
    assert_eq!(server.alo_acts_in(&codium), Some(which));
    assert_eq!(server.alo_acts_in(&codium), Some(which));
    assert_eq!(server.the_windows_alo_acts_in.how_many(), 1);
}

/// **A task the person has asked to stop is not at work**, so an act that
/// arrives after Stop marks nothing; and once the window wears a task, Stop
/// keeps it teal, *stopping*, until alo has gone.
#[test]
fn stop_is_shown_and_an_act_after_stop_marks_nothing() {
    let (_display, _peer, codium, other) = two_windows();
    let (_runtime, mut server) = a_session("alo-acts-and-is-stopped");
    let which = a_task_asked(&mut server, ASKED);
    assert_eq!(server.alo_acts_in(&codium), Some(which));
    assert!(server.the_person_stops_a_task(which));
    assert_eq!(
        server.alo_in_this_window(&codium).map(|here| here.how),
        Some(HowAloIsHere::Stopping)
    );
    assert_eq!(
        server.alo_acts_in(&other),
        None,
        "an act after Stop marked a window"
    );
    assert!(server.alo_in_this_window(&other).is_none());
    server.it_really_stopped(which);
    assert!(server.alo_in_this_window(&codium).is_none());
}

/// **Closing the task's window while alo acts takes the presence off the
/// window it acted in**, which the frame then lets go.
#[test]
fn closing_the_task_while_alo_acts_takes_the_presence_off() {
    let (_display, _peer, codium, _other) = two_windows();
    let (_runtime, mut server) = a_session("alo-acts-and-its-task-closes");
    let which = a_task_asked(&mut server, ASKED);
    assert_eq!(server.alo_acts_in(&codium), Some(which));
    assert!(server.tasks_mut().closed(which));
    assert!(server.alo_in_this_window(&codium).is_none());
    let over = server
        .the_windows_alo_acts_in
        .looked_at(std::slice::from_ref(&codium), &server.the_tasks);
    assert_eq!(over.len(), 1);
}
