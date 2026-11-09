//! **A change, carried out at once when the person chose full access, and put
//! to them otherwise**
//! ([ADR 0149](../../../docs/decisions/0149-full-access-is-everything-the-person-can-do.md)).
//!
//! ADR 0149, *What it changes for the person*: *"**At full access, alo can do
//! anything the person's own account can do**, without asking: read, create,
//! change and delete any file the account can, run any command, start, drive
//! and close any application."* And `CLAUDE.md`'s *Reads answer, changes
//! wait*, read to its end: *an approval is not a session at the default
//! setting, and a person may choose otherwise … At every level the run is
//! recorded, the files are snapshotted before it, and what leaves is shown.*
//!
//! # One door, and the list decides which road
//!
//! [`Turning::changing`] is [`Turning::proposing`] with a second road. **Which
//! road is not this crate's to decide**: the service reads the person's level
//! at this moment and hands over either the list it keeps or a copy made at
//! full access (`alo_capability::Grants::at_full_access_for`). On a list that
//! gives this turn's agent full access, a change is carried out now, through
//! `alo_capability::Authorised::at_full_access`; on any other list it is put
//! to the person exactly as [`Turning::proposing`] puts it, and waits for one
//! OK. A read offered here takes the proposing road, which refuses it as it
//! always did.
//!
//! # The order, at full access
//!
//! 1. **The sentence is about something that is there**
//!    (`crate::the_sentence_is_about_something`) — the refusal a model learns
//!    from, as on the proposing road.
//! 2. **The grants, at this moment** — full access widens what a call may
//!    reach, never what one call is; a facility or a terminal the person has
//!    not allowed is still refused, and written down.
//! 3. **The snapshot, before anything runs** (ADR 0045): the caller's `before`
//!    is handed the sentence and answers with the name of the place it kept,
//!    or nothing — one that could not be kept does not stop the change.
//! 4. **One execution**, inside the boundary, written down before it is
//!    answered, stamped as decided at full access, with no approval because
//!    none was asked for (`alo_capability::Authorised::without_the_ok`).
//!
//! Steps 1 and 2 come first so a change that will not run takes no snapshot.

use std::time::{Duration, SystemTime};

use alo_capability::{Authorised, Given, Grants, ProposalId};
use alo_strings::Said;

use crate::refusing::NotDone;
use crate::the_sentence_is_about_something::the_sentence_is_about_something;
use crate::turning::Turning;
use crate::what_a_verb_did::WhatAVerbDid;

/// What a change asked for became.
#[derive(Debug)]
pub enum Changed {
    /// It was carried out without the OK, because the person chose full access.
    Done {
        /// What the machine did.
        did: WhatAVerbDid,
        /// What was done, in the words the person would have been asked in.
        sentence: Said,
    },
    /// It waits for the person's OK, under this number.
    Waiting(ProposalId),
}

/// What is told the sentence of a change about to be carried out at full
/// access, so it can keep the place to return to first — and answers with
/// that place's name, or [`None`] when none was kept.
pub type Before<'b> = dyn FnMut(&Said) -> Option<String> + 'b;

impl Turning<'_, '_> {
    /// **A change**: carried out now when `grants` gives this turn's agent full
    /// access, and put to the person otherwise.
    ///
    /// # Errors
    /// As [`Turning::proposing`] on the road that asks; at full access
    /// [`NotDone::Refused`] for a sentence about nothing there or the grants
    /// saying no, and [`NotDone::MachineCouldNot`] for the account's own
    /// permissions or the disk saying no — each written down before it is
    /// answered.
    pub fn changing(
        &mut self,
        verb: &str,
        given: &[(&str, Given)],
        grants: &Grants,
        standing: Duration,
        before: &mut Before<'_>,
        now: SystemTime,
    ) -> Result<Changed, NotDone> {
        self.still_open()?;
        let call = self.calling(verb, given, now)?;
        if !call.waits_for_approval() || !grants.has_full_access(self.grantee()) {
            return self
                .proposing_call(call, grants, standing, now)
                .map(Changed::Waiting);
        }
        let machine = self.machine();
        let (resolving, strings) = (machine.resolving(), machine.strings());
        if let Err(refused) = the_sentence_is_about_something(&call, resolving, strings) {
            return self.stopped_under(refused, grants, now);
        }
        let authorised = match Authorised::at_full_access(&call, self.grantee(), grants, now) {
            Ok(authorised) => authorised,
            Err(refused) => return self.stopped_under(refused, grants, now),
        };
        let sentence = authorised.sentence(strings);
        if let Some(named) = before(&sentence) {
            self.a_place_was_kept(&named);
        }
        let did = self.running(authorised, grants, now)?;
        Ok(Changed::Done { did, sentence })
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
#[path = "a_change_at_full_access_tests.rs"]
mod tests;
