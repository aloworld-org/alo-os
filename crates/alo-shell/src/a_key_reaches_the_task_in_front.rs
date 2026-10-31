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
//! # Escape empties the box, and never lets go
//!
//! **The window in front keeps the keyboard** — the owner, 2026-10-30: *like
//! every desktop, the window you can see in front gets your typing.* Escape
//! empties the reply box, and on an empty box does nothing more. The keyboard
//! leaves a task window when a person goes somewhere else — a click on an
//! application, a shortcut that goes to another window — or when the window
//! closes (`crate::closing_a_task_window`).
//!
//! *Withdrawn on 2026-10-30 by the owner's decision:* until then Escape on an
//! empty box with no answer selected gave the keyboard up, "so a person can
//! always get back to their applications". On a booted machine the task window
//! stayed drawn in front, and every key went to the terminal hidden behind it.

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
    /// One key in the task window holding the keyboard: to the closing
    /// question, to the card, or to the reply box.
    pub(crate) fn a_key_reached_the_task(
        &mut self,
        which: WhichTask,
        typed: ComposerKey,
        answering: ApprovalKey,
        daemon: Option<&dyn alo_changing::Answering>,
        strings: &alo_strings::Strings,
    ) {
        // **The question about closing comes first** while it is asked: it is
        // the newest thing on the screen, and Escape there is *Cancel* rather
        // than letting go (ADR 0139).
        if self.a_key_on_the_closing_question(which, answering) {
            return;
        }
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
            // **A reply held for this answer goes now** (ADR 0138).
            self.send_what_was_held(which, strings);
            return;
        }
        // **Escape on an empty box does nothing**: the window in front keeps
        // the keyboard (this file's header). `a_key_reached_a_task_window`
        // answers `Nothing` for it.
        //
        // **A reply that was sent goes to alo with the conversation so far**,
        // or says beneath it why it did not: `crate::what_becomes_of_a_reply`
        // (ADR 0138). Dropped here until 2026-10-28, which left the window
        // reading *alo is working* for ever with nothing asked.
        if let crate::a_key_reaches_a_task_window::WhatTheReplyBoxDid::TheyReplied(said) =
            self.a_key_reached_a_task_window(which, typed)
        {
            self.a_reply_was_sent(which, said, strings);
        }
    }
}

#[cfg(test)]
#[path = "a_key_reaches_the_task_in_front_tests.rs"]
mod tests;
