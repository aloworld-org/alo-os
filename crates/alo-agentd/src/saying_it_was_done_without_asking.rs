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
//!
//! # A command that failed is not *done*
//!
//! **Since 2026-11-10 a command is answered by `ran_without_asking`**, which
//! says *done* only for a run that worked (`alo_capability::ItDidNotWork`) and
//! otherwise what was tried and why it did not work. The walk of `f028d5ec`
//! read *Done without asking you first … run this command: mkdir
//! /Users/username/Documents/demo* above the model's own *the directory does
//! not exist*: the machine's line claimed a change that was never made.

use alo_capability::ItDidNotWork;
use alo_capability::words::{
    DID_NOT_WORK_AT_FULL_ACCESS, DID_NOT_WORK_AT_FULL_ACCESS_AND_IT_CANNOT_BE_PUT_BACK,
    DONE_AT_FULL_ACCESS, DONE_AT_FULL_ACCESS_AND_IT_CANNOT_BE_PUT_BACK,
};
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

/// **A command run without asking, answered as what became of it**: done, as
/// [`done_without_asking`] says it, when it worked — and otherwise *tried*,
/// with why it did not work, so the person is never told a failed command was
/// done.
///
/// `did` still carries the whole run, status and all, so the model is told
/// the same thing the person is.
#[must_use]
pub fn ran_without_asking(did: Done, sentence: Said, kept: bool, strings: &Strings) -> ToAnAgent {
    let failed = match &did {
        // **With what it said**, so a push nobody had signed in for is told as
        // *you are not signed in to GitHub* and how to sign in, not as a number.
        Done::RanACommand {
            returned,
            took_too_long,
            said,
            ..
        } => ItDidNotWork::of_what_it_said(*returned, *took_too_long, said),
        _ => None,
    };
    let Some(failed) = failed else {
        return done_without_asking(did, sentence, kept, strings);
    };
    let word = if kept {
        DID_NOT_WORK_AT_FULL_ACCESS
    } else {
        DID_NOT_WORK_AT_FULL_ACCESS_AND_IT_CANNOT_BE_PUT_BACK
    };
    let said = strings.say(
        &word.key(),
        &Filling::of("sentence", sentence.into_text()).and_said("why", &failed.said(strings)),
    );
    ToAnAgent::done_without_asking(did, &said)
}
