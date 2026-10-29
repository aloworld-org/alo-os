//! Typing into a task window's reply box, and what Enter does there.
//!
//! The Dock's composer opens a task; this is the box inside the window a person
//! answers in. It is the **same key meanings** — `crate::composer_keys` — because
//! they are the same two rules in both places and a second reading of Enter is a
//! second chance to get it wrong.
//!
//! # What the two boxes do not share
//!
//! **The Dock's composer asks for a new task. This one replies to the one it is
//! in.** So the draft lives on the `ATask` rather than on the `Server`: it must
//! survive the window being put aside and brought back, and `alo_tasks`' own
//! note says why it is held there and not by a window.
//!
//! # Enter sends, and never stops
//!
//! *Conversation accepts user replies while work runs* — the interaction
//! contract. So Enter sends a reply even while the one button shows **Stop**,
//! and `ATask::enter_sends_the_draft` is the answer rather than the button's
//! state. A person replying to a working task must not cancel it by pressing
//! the key that sends.

use alo_tasks::ATask;

use crate::composer_keys::ComposerKey;
use crate::the_tasks_a_person_opened::WhichTask;

/// What a key press did to a task window's reply box.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WhatTheReplyBoxDid {
    /// Nothing at all.
    Nothing,
    /// What is in the box changed, so the window has to be drawn again.
    ItChanged,
    /// **A person sent a reply**, and this is what they said.
    ///
    /// Out of the box — a reply still in the box after being sent is one a
    /// person sends twice — and **not yet in the conversation**:
    /// `crate::what_becomes_of_a_reply` decides whether it is sent now or held
    /// until alo has answered, and a held reply is not part of what came before.
    TheyReplied(String),
}

impl crate::Server {
    /// One key press in a task window's reply box.
    ///
    /// Answers [`WhatTheReplyBoxDid::Nothing`] for a task that is not open,
    /// which is a true answer rather than a failure: a window can be closed
    /// between a key being pressed and this being asked.
    pub fn a_key_reached_a_task_window(
        &mut self,
        which: WhichTask,
        key: ComposerKey,
    ) -> WhatTheReplyBoxDid {
        // **A task that is not there does nothing**, and this branch cannot be
        // reached today: `WhichTask` has no constructor outside its own module
        // and nothing removes a task, so every one a caller holds is open.
        //
        // It is here because **closing a task window is promised and not
        // built** — `docs/features.md`'s *closing a running task asks* — and the
        // day it is, a key pressed between a window closing and this being asked
        // will arrive. Written now rather than discovered then, and left
        // untested rather than given a constructor that exists only to reach it:
        // a test-only door into `WhichTask` would make the invalid value
        // somebody can hold.
        let Some(task) = self.tasks_mut().the_mut(which) else {
            return WhatTheReplyBoxDid::Nothing;
        };
        match key {
            ComposerKey::Letter(letter) => typed(task, letter),
            ComposerKey::NewLine => typed(task, '\n'),
            ComposerKey::Erase => erased(task),
            ComposerKey::Ask => replied(task),
            // **Escape clears the box and does not close the window.** A task is
            // work alo is doing; closing its window on a keypress meant to
            // dismiss a half-typed sentence would be the one thing
            // `docs/features.md` says closing a running task must ask about.
            ComposerKey::Clear => {
                if task.the_draft().is_empty() {
                    WhatTheReplyBoxDid::Nothing
                } else {
                    task.the_person_is_typing("");
                    WhatTheReplyBoxDid::ItChanged
                }
            }
            ComposerKey::Nothing => WhatTheReplyBoxDid::Nothing,
        }
    }
}

/// Add one letter to what is in the box.
fn typed(task: &mut ATask, letter: char) -> WhatTheReplyBoxDid {
    if task.the_draft().chars().count() >= THE_MOST_A_REPLY_CAN_BE {
        return WhatTheReplyBoxDid::Nothing;
    }
    let mut draft = task.the_draft().to_owned();
    draft.push(letter);
    task.the_person_is_typing(draft);
    WhatTheReplyBoxDid::ItChanged
}

/// Take the last grapheme back.
fn erased(task: &mut ATask) -> WhatTheReplyBoxDid {
    use unicode_segmentation::UnicodeSegmentation;
    let draft = task.the_draft();
    let Some(last) = draft.graphemes(true).next_back() else {
        // **An empty box gives a held reply back**, to change or to delete:
        // the one way to take back something that has not been sent yet, and
        // nothing is lost by it — the words are in the box (ADR 0138).
        return match task.take_held() {
            Some(held) => {
                task.the_person_is_typing(held);
                WhatTheReplyBoxDid::ItChanged
            }
            None => WhatTheReplyBoxDid::Nothing,
        };
    };
    let keep = draft.len() - last.len();
    let shorter = draft.get(..keep).unwrap_or_default().to_owned();
    task.the_person_is_typing(shorter);
    WhatTheReplyBoxDid::ItChanged
}

/// Enter, with something in the box.
fn replied(task: &mut ATask) -> WhatTheReplyBoxDid {
    if !task.enter_sends_the_draft() {
        return WhatTheReplyBoxDid::Nothing;
    }
    WhatTheReplyBoxDid::TheyReplied(task.take_the_draft())
}

/// The most a reply can be, in characters.
///
/// `crate::the_composer_a_person_types_in`'s bound and its reason: a field with
/// no bound is a field a held key fills memory with. A reply is an answer rather
/// than a whole request, and this is the same generous figure because being
/// stingy here would cut somebody off mid-correction.
const THE_MOST_A_REPLY_CAN_BE: usize = 4_000;

#[cfg(test)]
#[path = "a_key_reaches_a_task_window_tests.rs"]
mod tests;
