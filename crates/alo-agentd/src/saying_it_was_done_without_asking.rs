//! **What the person is told about a change alo carried out without asking**,
//! because they chose full access
//! ([ADR 0149](../../../docs/decisions/0149-full-access-is-everything-the-person-can-do.md)).
//!
//! One sentence, written by the machine and not by the model: what was done,
//! that nobody was asked and why, and — when no copy from before it could be
//! kept — that it cannot be put back. **Every road a change takes at full
//! access answers with it**: a verb (`crate::a_change_asked_for`) and a
//! command where no verb fits (`crate::running_a_command`). Until 2026-11-09
//! only the first did, and a booted machine showed the cost: *make a folder
//! called demo in my Documents* ran `mkdir` without asking, and the person
//! read only the model's own sentence about it.
//!
//! A command run as administrator is not one of these: the person was asked,
//! for their password, so it is not answered as done without asking.

use alo_capability::words::{DONE_AT_FULL_ACCESS, DONE_AT_FULL_ACCESS_AND_IT_CANNOT_BE_PUT_BACK};
use alo_protocol::{Done, ToAnAgent};
use alo_strings::{Filling, Said, Strings};

/// **`did`, answered as done without asking**: `sentence` is what the person
/// would have been asked about it, and `kept` whether a copy from before it was
/// kept, which decides whether they are told it cannot be put back.
#[must_use]
pub fn done_without_asking(did: Done, sentence: Said, kept: bool, strings: &Strings) -> ToAnAgent {
    let word = if kept {
        DONE_AT_FULL_ACCESS
    } else {
        DONE_AT_FULL_ACCESS_AND_IT_CANNOT_BE_PUT_BACK
    };
    let said = strings.say(&word.key(), &Filling::of("sentence", sentence.into_text()));
    ToAnAgent::done_without_asking(did, &said)
}
