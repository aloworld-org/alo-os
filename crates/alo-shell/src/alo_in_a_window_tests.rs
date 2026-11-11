//! alo in a window: the presence a task wears on the window it was asked
//! about, on no other, for its turn — and the refusals.
//!
//! **What is substituted.** The windows are protocol identities that were
//! never mapped, so nothing here can show that a real window is found under a
//! real pointer; `tests/help_with_this_window` does that on a real client. What
//! these hold is the rule: which window wears alo, when, and when it stops.

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

use super::HowAloIsHere;
use crate::Server;
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

/// A session, and a task asked about `window`, which alo is working on.
fn a_task_about(window: &WlSurface, what: &str) -> (tempfile::TempDir, Server, WhichTask) {
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
    let which = server.tasks_mut().and_now_this_one(ATask::asked_for(
        "summarise this document",
        InferenceSource::ThisMachine,
    ));
    server.alo_works_in(which, window.clone());
    (runtime, server, which)
}

/// **The window asked about wears alo while it works, and no other window
/// does** — with the task's own status and the person's own words.
#[test]
fn the_window_asked_about_wears_alo_and_no_other_does() {
    let (_display, _peer, notes, other) = two_windows();
    let (_runtime, server, which) = a_task_about(&notes, "alo-wears-one-window");
    let here = server
        .alo_in_this_window(&notes)
        .expect("the window asked about wears alo");
    assert_eq!(here.which, which);
    assert_eq!(here.how, HowAloIsHere::Working);
    assert_eq!(here.status, alo_tasks::words::ALO_IS_WORKING);
    assert_eq!(here.what, "summarise this document");
    assert!(
        server.alo_in_this_window(&other).is_none(),
        "a second window was marked"
    );
}

/// **The presence goes when the turn ends**: alo answered, so nothing marks
/// the window, and a follow-up's turn marks it again.
#[test]
fn the_presence_goes_when_the_turn_ends() {
    let (_display, _peer, notes, _other) = two_windows();
    let (_runtime, mut server, which) = a_task_about(&notes, "alo-goes-with-the-turn");
    server
        .tasks_mut()
        .the_mut(which)
        .expect("open")
        .alo_says(Turn::from_alo("It is about making room."));
    assert!(
        server.alo_in_this_window(&notes).is_none(),
        "the window still wore alo after its turn ended"
    );
    server
        .tasks_mut()
        .the_mut(which)
        .expect("open")
        .the_person_says("and shorter?");
    assert_eq!(
        server.alo_in_this_window(&notes).map(|here| here.how),
        Some(HowAloIsHere::Working)
    );
}

/// **Stop during an approval**: waiting for the person is navy and not
/// acting; Stop makes it *stopping*, the change lapses unanswered, and the
/// band goes only once alo has really gone — stopping is not undoing.
#[test]
fn stop_during_an_approval_lapses_the_change_and_ends_the_presence() {
    let (_display, _peer, notes, _other) = two_windows();
    let (_runtime, mut server, which) = a_task_about(&notes, "alo-stopped-while-asking");
    {
        let task = server.tasks_mut().the_mut(which).expect("open");
        task.is_asking("Replace the first paragraph");
        task.alo_says(Turn::alo_proposing("Replace the first paragraph"));
    }
    assert_eq!(
        server.alo_in_this_window(&notes).map(|here| here.how),
        Some(HowAloIsHere::WaitingForThePerson)
    );
    assert!(server.the_person_stops_a_task(which));
    assert_eq!(
        server.alo_in_this_window(&notes).map(|here| here.how),
        Some(HowAloIsHere::Stopping),
        "the band said the work was over before alo had gone"
    );
    let task = server.tasks().the(which).expect("open");
    assert!(
        task.answers_given().each().is_empty(),
        "the Stop answered the change"
    );
    assert_eq!(server.take_the_stops().len(), 1);
    server.it_really_stopped(which);
    assert!(server.alo_in_this_window(&notes).is_none());
}

/// **A window that closes mid-turn takes its presence with it**, and the
/// turn is not stopped by it: the task is still working, in its own window.
#[test]
fn a_window_that_closes_mid_turn_takes_its_presence_and_leaves_the_turn() {
    let (_display, _peer, notes, _other) = two_windows();
    let (_runtime, mut server, which) = a_task_about(&notes, "alo-window-closed");
    // Never mapped, as a window that has gone is not: the frame lets go.
    server.alo_may_be_asked(true);
    assert!(server.alo_in_this_window(&notes).is_none());
    assert!(server.the_window_a_task_is_about(which).is_none());
    assert_eq!(
        server.tasks().the(which).map(ATask::what_is_happening),
        Some(alo_tasks::words::ALO_IS_WORKING),
        "closing the window stopped the work"
    );
    assert!(server.take_the_stops().is_empty());
}

/// **Help with this window opens nothing while alo may not be asked** — AI
/// set to *Not at all*, or nothing chosen to answer — and nothing for a
/// window that is not on the screen.
#[test]
fn help_opens_nothing_while_alo_may_not_be_asked_or_for_a_window_not_shown() {
    let (_display, _peer, notes, _other) = two_windows();
    let (_runtime, mut server, _which) = a_task_about(&notes, "alo-help-refused");
    server.alo_may_be_asked(false);
    assert!(!server.help_with_this_window(&notes));
    assert!(!server.composer().has_the_keyboard());
    server.alo_may_be_asked(true);
    assert!(
        !server.help_with_this_window(&notes),
        "a window not on the screen was offered"
    );
    assert!(server.the_window_offered_reads().is_none());
    assert!(server.take_what_was_asked_about_a_window().is_empty());
}
