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
///
/// **A publication is told in its own line**, whatever was kept: it is on the
/// internet, and Put back can't recall it (the owner's decision of 2026-11-13,
/// `alo_publishing::what_the_person_is_told`) — never *Done*, which beside a
/// change reads as one Put back can take back.
#[must_use]
pub fn done_without_asking(did: Done, sentence: Said, kept: bool, strings: &Strings) -> ToAnAgent {
    if let Done::Published {
        folder,
        address,
        nothing_new,
    } = &did
    {
        let said = alo_publishing::published_without_asking(
            alo_publishing::WhatWasPublished {
                folder,
                address,
                nothing_new: *nothing_new,
            },
            strings,
        );
        return ToAnAgent::done_without_asking(did, &said);
    }
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
        // **Never plain *Done* beside a snapshot for a command that sent
        // something off this computer** (ADR 0110 §2): the line says Put back
        // returns the files here and cannot recall what left — or, where
        // nothing counted, that it could not tell (`alo_capability::WhatItSent`).
        if kept && let Some(what) = did.what_a_command_sent() {
            let said = strings.say(
                &what.done_at_full_access().key(),
                &Filling::of("sentence", sentence.into_text()),
            );
            return ToAnAgent::done_without_asking(did, &said);
        }
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

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::ran_without_asking;
    use alo_capability::WhatACommandDid;
    use alo_protocol::{Done, ToAnAgent};
    use alo_strings::{Filling, Strings};

    /// The line a command run at full access is answered with, given how
    /// many bytes it was counted sending and whether a copy was kept.
    fn told(sent: Option<u64>, kept: bool) -> String {
        let strings = Strings::of(alo_capability::words::capability_words().unwrap());
        let did = Done::of_a_command(
            &WhatACommandDid::of(Some(0), String::new(), false, false).having_sent(sent),
        );
        let sentence = strings.say(
            &alo_capability::words::RUN_A_COMMAND.key(),
            &Filling::of("command", "git push"),
        );
        match ran_without_asking(did, sentence, kept, &strings) {
            ToAnAgent::DoneWithoutAsking { said, .. } => said.text().to_owned(),
            other => format!("{other:?}"),
        }
    }

    /// **A push at full access, beside a kept copy, is never told as plain
    /// *Done***: Put back returns the files here and cannot recall what left —
    /// or, where nothing counted, the line says it could not tell. One counted
    /// as sending nothing keeps the plain line, and one with no copy kept
    /// already says it can't be put back (ADR 0110 §2).
    #[test]
    fn a_command_that_sent_something_at_full_access_is_not_told_as_wholly_undoable() {
        let plain = "Done without asking you first, because you chose full access in Settings → \
                     Running code: run this command: git push.";
        assert_eq!(told(Some(0), true), plain);
        let sent = told(Some(52_000), true);
        assert!(
            sent.starts_with("Done without asking you first")
                && sent.contains("Put back can return your files on this computer")
                && sent.contains("sent something off this computer while it ran"),
            "{sent}"
        );
        assert!(
            told(None, true).contains("could not count whether it sent anything"),
            "{}",
            told(None, true)
        );
        assert!(told(Some(52_000), false).contains("can't be put back"));
    }
}
