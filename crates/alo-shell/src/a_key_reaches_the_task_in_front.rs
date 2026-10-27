//! The road a real key takes into the task window holding the keyboard, and the
//! one decision on it: **the card, or the reply box.**
//!
//! # The card gets a key only when all of this is true
//!
//! - **a change is waiting, and it is in this window.** One question stands per
//!   machine and it is held by whichever task is asking; a person may only
//!   answer what is in front of them, so a key in another task's window never
//!   reaches it;
//! - **there is a daemon to tell**;
//! - and the key is **Tab** — moving to the answers — or **Enter, Space or
//!   Escape with an answer already selected**.
//!
//! Everything else is typing, and goes to the reply box. So **Enter with no
//! answer selected sends the reply rather than answering the card**, which is
//! *nothing proceeds on silence* on the one key where it could go wrong: a
//! person who pressed Enter to send what they wrote has not approved anything.
//!
//! # Letting go
//!
//! **Escape on an empty reply box, with no answer selected, gives the keyboard
//! up**, so a person can always get back to their applications. Before that,
//! Escape empties the box, which is what it already did. Clicking an
//! application does not take the keyboard back yet — and the same is true of
//! the Bar, whose `TheComposer::it_gave_the_keyboard_up` has no caller outside
//! its tests. That is its own change.

use crate::approval_keys::ApprovalKey;
use crate::composer_keys::ComposerKey;
use crate::the_tasks_a_person_opened::WhichTask;
use crate::{InputError, InputUpdate, Server};

/// One libinput update while a task window holds the keyboard.
///
/// # Errors
/// As `Server::the_tasks_key`, and as the seat for anything that is not a key.
pub(crate) fn a_key_reaches_the_task(
    server: &mut Server,
    update: InputUpdate<'_>,
    extent: (i32, i32),
    strings: &alo_strings::Strings,
) -> Result<(), InputError> {
    let InputUpdate::Event(event) = update else {
        return server.libinput_update(update, extent);
    };
    // The same reading Settings, the Bar and the clients are handed, so a
    // machine cannot disagree with itself about which key was pressed.
    let Some(crate::DirectSeatEvent::Key(key, _)) = crate::libinput_routing::translate(event)?
    else {
        return server.libinput_update(InputUpdate::Event(event), extent);
    };
    let daemon = server
        .settings_places
        .as_ref()
        .map(|places| alo_changing::TheDaemonsDoor::at(places.daemon()));
    the_key_the_task_took(
        server,
        key,
        daemon
            .as_ref()
            .map(|door| door as &dyn alo_changing::Answering),
        strings,
    )
}

/// One key, from the evdev code inward.
///
/// **Split from the function above so that a test can press a key**, for
/// `crate::a_key_reaches_the_bar`'s reason; and handed the daemon rather than
/// finding it, so a test can stand in for one.
///
/// # Errors
/// As [`a_key_reaches_the_task`].
pub(crate) fn the_key_the_task_took(
    server: &mut Server,
    key: crate::DirectKeyEvent,
    daemon: Option<&dyn alo_changing::Answering>,
    strings: &alo_strings::Strings,
) -> Result<(), InputError> {
    let Some(which) = server.the_task_with_the_keyboard else {
        return Ok(());
    };
    match server.the_tasks_key(key.code, key.state, key.time) {
        Ok(Some((typed, answering))) => {
            server.a_key_reached_the_task(which, typed, answering, daemon, strings);
            Ok(())
        }
        // A release, a repeat, and a code evdev does not have.
        Ok(None) | Err(InputError::InvalidKey) => Ok(()),
        Err(error) => Err(error),
    }
}

impl Server {
    /// One key in the task window holding the keyboard: to the card, to the
    /// reply box, or — Escape on an empty box — letting go.
    pub(crate) fn a_key_reached_the_task(
        &mut self,
        which: WhichTask,
        typed: ComposerKey,
        answering: ApprovalKey,
        daemon: Option<&dyn alo_changing::Answering>,
        strings: &alo_strings::Strings,
    ) {
        let the_question_is_here = self
            .the_tasks
            .the(which)
            .is_some_and(|task| task.permissions().asking().is_some());
        let something_is_selected = self.the_change_waiting.selected().is_some();
        let for_the_card = self.the_change_waiting.is_asking()
            && the_question_is_here
            && match answering {
                ApprovalKey::NextAnswer | ApprovalKey::PreviousAnswer => true,
                ApprovalKey::Choose | ApprovalKey::Decline => something_is_selected,
                ApprovalKey::Nothing => false,
            };
        if for_the_card && let Some(daemon) = daemon {
            // **The fields, not the methods**, so the compiler can see the
            // question and the task are different parts of the server.
            if let Some(task) = self.the_tasks.the_mut(which)
                && let crate::a_change_waiting_reaches_the_person::WhatThePressDid::ItWasAnswered(
                    answered,
                ) = self.the_change_waiting.pressed(answering, task, daemon)
            {
                // **The person is told what their answer did**, which nothing
                // did until 2026-10-26 — `crate::what_an_answer_did` says why.
                // It is also what moves the task off *waiting for your
                // permission*: the conversation no longer ends on a question.
                task.alo_says(alo_tasks::Turn::from_alo(
                    crate::what_an_answer_did::what_an_answer_did(answered.as_ref(), strings),
                ));
            }
            return;
        }
        let the_box_is_empty = self
            .the_tasks
            .the(which)
            .is_none_or(|task| task.the_draft().is_empty());
        if typed == ComposerKey::Clear && the_box_is_empty && !something_is_selected {
            self.the_task_gave_the_keyboard_up();
            return;
        }
        drop(self.a_key_reached_a_task_window(which, typed));
    }
}

#[cfg(test)]
#[path = "a_key_reaches_the_task_in_front_tests.rs"]
mod tests;
