//! **A change an agent asked for: carried out at once at full access, and put
//! to the person at every other level**
//! ([ADR 0149](../../../docs/decisions/0149-full-access-is-everything-the-person-can-do.md)).
//!
//! ADR 0149, *What it changes for the person*: *"**At full access, alo can do
//! anything the person's own account can do**, without asking: read, create,
//! change and delete any file the account can, run any command, start, drive
//! and close any application."* `docs/features.md`: *alo reaches every file and
//! folder the person's account can, runs any command and drives any
//! application, without asking and without folder grants.*
//!
//! # The level is read here, at the moment of the change
//!
//! Through `crate::the_grants_as_the_level_is_now`, from the person's own file,
//! as a read and a command read it:
//!
//! | the file says | what happens to the change |
//! |---|---|
//! | full access (`at-full-trust`) | carried out now: the snapshot first, one execution, written down as at full access with no approval, and answered `done-without-asking` |
//! | *ask me each time*, the sealed box, **nothing chosen**, **a file that will not read** | put to the person, and it waits for one OK — unchanged |
//!
//! So switching full access off takes effect on the next change: the first was
//! carried out, the second waits.
//!
//! # And the person is told
//!
//! Nobody was asked, so the answer carries the sentence the person would have
//! been asked, inside one that says it was done without asking and why — and
//! whether it can be put back, which depends on whether the snapshot was kept
//! (ADR 0045's third term). The agent tells the person in its reply
//! (`alo_agent::WhatHappened::done_without_asking`), and the model is told the
//! same line as a step, so the turn goes on.
//!
//! # The terminal's own switch
//!
//! `in-the-terminal` (`docs/contracts/how-code-runs-file.md`) lets `alo-ask`
//! approve, on the person's door, a change its own request left waiting. At
//! full access nothing is left waiting, so the switch has nothing to answer
//! and does nothing; at every other level it works as it did.

use std::ffi::OsStr;
use std::path::Path;
use std::time::{Duration, SystemTime};

use alo_capability::{Given, Grants};
use alo_protocol::ToAnAgent;
use alo_strings::{Said, Strings};
use alo_turn::{Changed, Turning};

use crate::running_a_command::Commands;
use crate::saying_it_was_done_without_asking::done_without_asking;
use crate::the_grants_as_the_level_is_now::the_grants_as_the_level_is_now;
use crate::what_a_client_is_told_a_verb_did::as_a_client_sees_it;

/// **A change an agent proposed**, decided by the person's level at this
/// moment: carried out and answered, or left waiting and answered with the
/// sentence the person will be asked — and written down in every case but a
/// question, which is not a thing that happened.
#[expect(
    clippy::too_many_arguments,
    reason = "one request, the turn it belongs to, and everything it can reach; a struct would exist only to be unpacked"
)]
pub fn a_change_asked_for(
    verb: &str,
    given: &[(&str, Given)],
    turning: &mut Turning<'_, '_>,
    grants: &Grants,
    commands: Commands<'_>,
    strings: &Strings,
    standing: Duration,
    now: SystemTime,
) -> ToAnAgent {
    let (here, mut bracket) = commands.into_parts();
    // **A publication at the sealed box is refused before anybody is asked**,
    // and written down (`crate::publishing_at_this_level`).
    if let Some(said) = crate::publishing_at_this_level::not_at_this_level(verb, here, strings) {
        let why = turning.a_change_this_level_cannot_make(verb, given, &said, now);
        return crate::doing::not_done(&why, strings);
    }
    let grants = the_grants_as_the_level_is_now(grants, here, turning.grantee());
    let changed = {
        // **The snapshot, before the change runs** (ADR 0045), asked for only
        // once the change is known to run: a refusal takes none.
        let mut before = |sentence: &Said| {
            let bracket = bracket.as_deref_mut()?;
            bracket.before_a_change(sentence.text());
            bracket
                .taken_at()
                .and_then(Path::file_name)
                .and_then(OsStr::to_str)
                .map(str::to_owned)
        };
        turning.changing(verb, given, &grants, standing, &mut before, now)
    };
    match changed {
        Ok(Changed::Done { did, sentence }) => {
            let kept = bracket.is_some_and(|bracket| bracket.taken_at().is_some());
            done_without_asking(as_a_client_sees_it(&did, now), sentence, kept, strings)
        }
        Ok(Changed::Waiting(number)) => crate::doing::waiting_under(turning, number, strings, now),
        Err(why) => crate::doing::not_done(&why, strings),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "a_change_asked_for_tests.rs"]
mod tests;
