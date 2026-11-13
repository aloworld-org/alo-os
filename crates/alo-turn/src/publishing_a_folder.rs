//! From *the person approved this publication*, or *full access allowed it*,
//! to *this folder is on their GitHub Pages site* — `crate::installing_an_application`'s
//! shape, one verb along.
//!
//! # The order, and why the indicator is in the middle of it
//!
//! 1. **Is this authority a publication at all?** `alo_publishing::approved`
//!    asks it — this verb, from an approval or full access, a folder that is
//!    still a folder with something in it, an account GitHub could give.
//! 2. **Does this machine publish at all?** A machine built without
//!    [`crate::WhatPublishingNeeds`] does not offer the verb, so this is the
//!    machine contradicting itself, said rather than unwrapped.
//! 3. **The line goes up on the indicator**, under the rule in force on this
//!    machine — *alo is sending something to github.com* — and only then
//! 4. **`alo_publishing::publish`**, which runs the one command once, as the
//!    person, on the command road.
//! 5. **The line comes down, on every road out of here.**
//!
//! Law 1 is why 3 is where it is: the line is up before the first byte, and a
//! rule that says nothing may leave refuses here with nothing having left.
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
//! The entry is the call, with the sentence the person approved, and — for a
//! publication that did not happen — the reason beside it.

use alo_capability::{Authorised, Grants};
use alo_files::Touching;
use alo_publishing::{NotPublished, Published};
use alo_record::Entry;

use crate::machine::Machine;
use crate::refusing::NotDone;

/// Carry out a publication, and say what happened — with an entry on every
/// road, including every refusal.
pub(crate) fn publishing_a_folder(
    machine: &mut Machine<'_>,
    authorised: Authorised,
    grants: &Grants,
    now: std::time::SystemTime,
) -> (Entry, Result<Published, NotDone>) {
    let strings = machine.strings();
    let refused = |why: NotPublished, authorised: &Authorised| {
        (
            Entry::ran(authorised, strings).telling(&[why.said(strings)]),
            Err(NotDone::CouldNotPublish(why)),
        )
    };
    // **Where the folder really leads, and whether the grants cover that** —
    // `alo_files::Touching`, as for every file verb, so a link inside a granted
    // folder that leads out of it publishes nothing.
    let agent = authorised.under().clone();
    let at = authorised.at();
    let authorised = match Touching::of(authorised, grants, machine.resolving(), strings) {
        Ok(touching) => touching.into_authorised(),
        Err(refused) => {
            let entry = Entry::refused(&refused, &agent, strings, at);
            return (entry, Err(NotDone::Refused(refused)));
        }
    };
    // Step 1.
    let publication = match alo_publishing::approved(&authorised) {
        Ok(publication) => publication,
        Err(why) => return refused(why, &authorised),
    };
    // Step 2.
    let Some((publishing, indicator)) = machine.publishing_and_showing() else {
        return refused(
            NotPublished::NotHere("this machine was built without anything to publish with".into()),
            &authorised,
        );
    };
    let leaving = match alo_publishing::the_departure(authorised.under()) {
        Ok(leaving) => leaving,
        Err(why) => {
            return refused(
                NotPublished::NotHere(format!("github.com could not be shown: {why:?}")),
                &authorised,
            );
        }
    };
    // Step 3. A rule that says nothing may leave this way refuses here, in its
    // own words, with nothing having left.
    let departing = match indicator.beginning(&publishing.rule, leaving, now) {
        Ok(departing) => departing,
        Err(not_permitted) => {
            let said = not_permitted.said(strings);
            return (
                Entry::ran(&authorised, strings).telling(&[said]),
                Err(NotDone::NotPermittedToLeave(not_permitted)),
            );
        }
    };
    // Step 4.
    let done = alo_publishing::publish(
        &departing,
        &publication,
        &publishing.through,
        publishing.runs,
        strings,
    );
    // Step 5. **The line comes down whatever happened.**
    indicator.ended(departing);
    match done {
        Ok(published) => (Entry::ran(&authorised, strings), Ok(published)),
        Err(why) => refused(why, &authorised),
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
