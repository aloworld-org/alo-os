//! From *the person approved this publication*, or *full access allowed it*,
//! to *this folder is on their GitHub Pages site* — `crate::installing_an_application`'s
//! shape, one verb along.
//!
//! # The order
//!
//! 1. **Is this authority a publication at all?** `alo_publishing::approved`
//!    asks it — this verb, from an approval or full access, a folder that is
//!    still a folder with something in it, an account GitHub could give.
//!    **No folder grant is asked**: the owner, 2026-11-13, *"Approval alone is
//!    enough"* — one approval of the sentence, as for any command.
//! 2. **Does this machine publish at all?** A machine built without
//!    [`crate::WhatPublishingNeeds`] does not offer the verb, so this is the
//!    machine contradicting itself, said rather than unwrapped.
//! 3. **May anything leave for github.com?** The rule in force on this machine
//!    is asked; one that says no refuses here, with nothing sent.
//! 4. **`alo_publishing::publish`**, which runs the one command once, as the
//!    person, on the command road — **counted the way every command alo runs
//!    is** (`crate::while_alos_command_runs`, ADR 0148 as amended 2026-11-13):
//!    started inside a control group of its own, the line *git, a command alo
//!    is running, is sending* up from the first byte the kernel counts — or,
//!    on a machine that cannot count, *alo is running git, and this machine
//!    cannot count what it sends* for the whole run — and down when it ends.
//!    How many bytes left is kept on the record's entry.
//!
//! # No boundary, and no snapshot
//!
//! As with an installation, there is no kernel boundary around this: what it
//! reaches is the person's own `git` and `gh`, run as them on the command road
//! of ADR 0149, and what bounds that is the account's own permissions. And
//! **nothing on this computer changes** — the folder is read, never written
//! (`alo_publishing::the_line`'s header) — so nothing here takes a place to
//! return to: a snapshot beside a publication would put *Put back* on a thing
//! it cannot recall, which is ADR 0110 §2's overstatement.
//!
//! # Every run and every refusal is written down
//!
//! The entry is the call, with the sentence the person approved, how many bytes
//! it sent where they were counted, and — for a publication that did not
//! happen — the reason beside it.

use alo_capability::Authorised;
use alo_publishing::{NotPublished, Published};
use alo_record::Entry;

use crate::machine::Machine;
use crate::refusing::NotDone;
use crate::while_alos_command_runs::WhileItRuns;

/// What the indicator names while a publication's command runs: the program
/// that sends the folder.
const WHAT_SENDS_IT: &str = "git";

/// Carry out a publication, and say what happened — with an entry on every
/// road, including every refusal.
pub(crate) fn publishing_a_folder(
    machine: &mut Machine<'_>,
    authorised: Authorised,
    now: std::time::SystemTime,
) -> (Entry, Result<Published, NotDone>) {
    let strings = machine.strings();
    let refused = |why: NotPublished, authorised: &Authorised| {
        (
            Entry::ran(authorised, strings).telling(&[why.said(strings)]),
            Err(NotDone::CouldNotPublish(why)),
        )
    };
    // Step 1.
    let publication = match alo_publishing::approved(&authorised) {
        Ok(publication) => publication,
        Err(why) => return refused(why, &authorised),
    };
    // Step 2.
    let Some((publishing, bounding, indicator)) = machine.publishing_and_counting() else {
        return refused(
            NotPublished::NotHere("this machine was built without anything to publish with".into()),
            &authorised,
        );
    };
    // Step 3. A rule that says nothing may leave this way refuses here, in its
    // own words, with nothing having left.
    let leaving = match alo_publishing::the_departure(authorised.under()) {
        Ok(leaving) => leaving,
        Err(why) => {
            return refused(
                NotPublished::NotHere(format!("github.com could not be named: {why:?}")),
                &authorised,
            );
        }
    };
    if let Some(not_permitted) = publishing.rule.refusal(&leaving) {
        let said = not_permitted.said(strings);
        return (
            Entry::ran(&authorised, strings).telling(&[said]),
            Err(NotDone::NotPermittedToLeave(not_permitted)),
        );
    }
    // Step 4, counted as every command alo runs is, and the line down on every
    // road out of it.
    let (done, sent) = {
        let mut around =
            WhileItRuns::begun(bounding.a_command_counted(), indicator, WHAT_SENDS_IT, now);
        let done = alo_publishing::publish(
            &publication,
            &publishing.through,
            publishing.runs,
            &mut around,
            strings,
        );
        (done, around.ended())
    };
    match done {
        Ok(published) => (
            Entry::ran(&authorised, strings).having_sent(sent),
            Ok(published),
        ),
        Err(why) => {
            let (entry, refusal) = refused(why, &authorised);
            (entry.having_sent(sent), refusal)
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "publishing_a_folder_tests.rs"]
mod tests;
