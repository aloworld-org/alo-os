//! That a person can reply in a task window, that Enter never cancels the work
//! they are replying to, and that Escape clears the box rather than the task.
#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on a socket that will not bind, or on a task this file \
              just opened, is the failure being reported"
)]

use std::os::unix::fs::PermissionsExt as _;

use alo_models::InferenceSource;
use alo_tasks::conversation::Turn;
use alo_tasks::{ATask, TheOneButton};

use super::WhatTheReplyBoxDid;
use crate::Server;
use crate::composer_keys::{ComposerKey, Composing};
use crate::the_tasks_a_person_opened::WhichTask;

/// A server with one task open, which alo is working in.
fn a_server_with_a_task(named: &str) -> (tempfile::TempDir, Server, WhichTask) {
    let directory = tempfile::tempdir().expect("a private directory");
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700))
        .expect("a private runtime directory");
    let mut server = Server::bind(directory.path(), named).expect("a bound socket");
    let mut task = ATask::asked_for("Tidy my invoices folder", InferenceSource::ThisMachine);
    task.alo_says(Turn::from_alo("I'll look through them."));
    task.and_then(alo_tasks::Act::doing("reading your folder"));
    let which = server.tasks_mut().and_now_this_one(task);
    (directory, server, which)
}

/// Type a phrase into the task's reply box.
fn type_in(server: &mut Server, which: WhichTask, words: &str) {
    for letter in words.chars() {
        server.a_key_reached_a_task_window(which, ComposerKey::Letter(letter));
    }
}

/// **A person types a reply and sends it**: out of the box, and handed on —
/// `crate::what_becomes_of_a_reply` decides whether it joins the conversation
/// now or is held until alo has answered.
#[test]
fn a_reply_is_typed_sent_and_handed_on() {
    let (_directory, mut server, which) = a_server_with_a_task("reply-sent");
    let before = server
        .tasks()
        .the(which)
        .expect("the task")
        .conversation()
        .each()
        .len();

    type_in(&mut server, which, "Keep the receipts together");
    let what = server.a_key_reached_a_task_window(which, ComposerKey::Ask);

    assert_eq!(
        what,
        WhatTheReplyBoxDid::TheyReplied("Keep the receipts together".to_owned())
    );
    let task = server.tasks().the(which).expect("the task");
    assert_eq!(task.conversation().each().len(), before);
    assert!(
        task.the_draft().is_empty(),
        "the box is empty after sending"
    );
}

/// **Enter sends a reply while the one button says Stop, and does not stop it.**
///
/// *Conversation accepts user replies while work runs.* The dangerous case: a
/// person answering a working task must not cancel it with the key that sends.
#[test]
fn enter_replies_while_the_button_says_stop_and_does_not_stop_the_work() {
    let (_directory, mut server, which) = a_server_with_a_task("reply-while-working");
    assert_eq!(
        server
            .tasks()
            .the(which)
            .expect("the task")
            .the_one_button(),
        TheOneButton::Stop,
        "the fixture must be a working task, or this proves nothing"
    );

    type_in(&mut server, which, "Four slides is enough");
    server.a_key_reached_a_task_window(which, ComposerKey::Ask);

    let task = server.tasks().the(which).expect("the task");
    assert!(
        matches!(task.stopping(), alo_tasks::Stopping::NobodyAsked),
        "sending a reply asked the task to stop"
    );
    assert!(
        task.activity().doing_now().is_some(),
        "the work it was doing was ended by a reply"
    );
}

/// **An empty box sends nothing**, so Enter on a blank line does not add a
/// turn saying nothing.
#[test]
fn enter_on_an_empty_box_says_nothing() {
    let (_directory, mut server, which) = a_server_with_a_task("reply-empty");
    let before = server
        .tasks()
        .the(which)
        .expect("the task")
        .conversation()
        .each()
        .len();

    let what = server.a_key_reached_a_task_window(which, ComposerKey::Ask);

    assert_eq!(what, WhatTheReplyBoxDid::Nothing);
    assert_eq!(
        server
            .tasks()
            .the(which)
            .expect("the task")
            .conversation()
            .each()
            .len(),
        before
    );
}

/// **Shift+Enter adds a line and sends nothing.**
#[test]
fn shift_enter_adds_a_line_rather_than_replying() {
    let (_directory, mut server, which) = a_server_with_a_task("reply-newline");
    type_in(&mut server, which, "first");

    let what = server.a_key_reached_a_task_window(which, ComposerKey::NewLine);
    type_in(&mut server, which, "second");

    assert_eq!(what, WhatTheReplyBoxDid::ItChanged);
    assert_eq!(
        server.tasks().the(which).expect("the task").the_draft(),
        "first\nsecond"
    );
}

/// **Enter while an input method is composing replies with nothing.**
///
/// Held through the whole road rather than at the key enum, which is where it
/// is cheapest to prove and least worth proving.
#[test]
fn enter_while_composing_does_not_reply() {
    let (_directory, mut server, which) = a_server_with_a_task("reply-ime");
    type_in(&mut server, which, "\u{3053}\u{3093}");
    let before = server
        .tasks()
        .the(which)
        .expect("the task")
        .conversation()
        .each()
        .len();

    let key = ComposerKey::of(
        smithay::input::keyboard::Keysym::Return,
        &smithay::input::keyboard::ModifiersState::default(),
        Composing::InProgress,
    );
    let what = server.a_key_reached_a_task_window(which, key);

    assert_eq!(what, WhatTheReplyBoxDid::Nothing);
    assert_eq!(
        server
            .tasks()
            .the(which)
            .expect("the task")
            .conversation()
            .each()
            .len(),
        before,
        "a candidate being accepted was sent as a reply"
    );
    assert_eq!(
        server.tasks().the(which).expect("the task").the_draft(),
        "\u{3053}\u{3093}",
        "and what was being composed is still there"
    );
}

/// **Escape clears the box and leaves the task alone.**
///
/// A task is work alo is doing. Closing its window on a key meant to dismiss a
/// half-typed sentence would be the thing `docs/features.md` says closing a
/// running task must ask about.
#[test]
fn escape_clears_the_box_and_not_the_task() {
    let (_directory, mut server, which) = a_server_with_a_task("reply-escape");
    type_in(&mut server, which, "never mind");

    let what = server.a_key_reached_a_task_window(which, ComposerKey::Clear);

    assert_eq!(what, WhatTheReplyBoxDid::ItChanged);
    let task = server.tasks().the(which).expect("the task");
    assert!(task.the_draft().is_empty());
    assert!(
        matches!(task.stopping(), alo_tasks::Stopping::NobodyAsked),
        "Escape stopped the task"
    );
    assert_eq!(server.tasks().how_many(), 1, "Escape closed the window");
}

/// **One backspace takes back one thing a person sees.**
#[test]
fn erasing_takes_back_a_whole_grapheme() {
    let (_directory, mut server, which) = a_server_with_a_task("reply-erase");
    type_in(&mut server, which, "cafe\u{301}");

    server.a_key_reached_a_task_window(which, ComposerKey::Erase);

    assert_eq!(
        server.tasks().the(which).expect("the task").the_draft(),
        "caf",
        "the letter and its accent went together"
    );
}

/// **Backspace in an empty box gives a held reply back**, to change or to
/// delete, and with nothing held it does nothing (ADR 0138).
#[test]
fn backspace_in_an_empty_box_gives_a_held_reply_back() {
    let (_directory, mut server, which) = a_server_with_a_task("held-back");
    assert_eq!(
        server.a_key_reached_a_task_window(which, ComposerKey::Erase),
        WhatTheReplyBoxDid::Nothing,
        "the control: an empty box with nothing held"
    );
    server
        .tasks_mut()
        .the_mut(which)
        .expect("the task")
        .hold("and the April one too");

    let what = server.a_key_reached_a_task_window(which, ComposerKey::Erase);

    assert_eq!(what, WhatTheReplyBoxDid::ItChanged);
    let task = server.tasks().the(which).expect("the task");
    assert_eq!(task.the_draft(), "and the April one too");
    assert_eq!(task.held(), None, "the reply is both held and in the box");
}
