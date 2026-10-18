//! From *an agent asked what was done to a file* to the answer, inside the
//! turn.
//!
//! The fourth executor on this machine and **the second that is a read**:
//! `crate::carrying` carries the six file verbs,
//! `crate::installing_an_application` the installation,
//! `crate::acting_on_an_application` the three application acts, and
//! `crate::looking_at_the_catalogue` answers what could be installed. This one
//! answers a question about what already happened.
//!
//! `docs/features.md`, starred: *Find a command by what it did.* The owner
//! decided on 2026-10-18 that the surface is **a question to alo**, which alo
//! answers from the record — so the agent needs a verb, and this is where that
//! verb is carried out.
//!
//! # It needs no boundary, and that is not an oversight
//!
//! ADR 0013's boundary is built out of resolved paths, and this verb opens
//! none: the path it carries is matched against what the record already wrote
//! down, as text, and nothing on the disk at that path is touched. The same
//! argument `looking_at_the_catalogue` makes one verb along.
//!
//! **What it does need is the grant**, and it has it before arriving here:
//! `alo_record::verbs` declares `Requires::grants_over(["file"])`, so a call
//! about a path nobody granted never becomes an [`Authorised`] and never
//! reaches this file. That is where the permission is enforced — not in what
//! is drawn afterwards.
//!
//! # Nothing leaves
//!
//! Law 1's indicator has nothing to show. The record is read from the machine
//! the question was asked on, and no byte goes anywhere.
//!
//! # The entry comes back on every road, including the refusals
//!
//! `crate::carrying`'s promise, kept here: **a read is recorded exactly as a
//! change is.** `CLAUDE.md`'s gate asks for *every execution and every refusal
//! leaves a record*, and it does not say *every change* — a person looking at
//! what their agent did should see the questions it asked about them too.
//!
//! There is a second reason here that the other executors do not have: this
//! verb reads the record, so an agent asking it repeatedly about paths is
//! something a person should be able to see — and they can, because each
//! asking is itself an entry.

use alo_capability::Authorised;
use alo_record::{Entry, WhatTouchedIt, asked_about, what_touched};

use crate::machine::Machine;
use crate::refusing::NotDone;
use crate::went_wrong_asking::WentWrongAsking;

/// Answer what the record says was done to a file, and say what happened.
///
/// `now` is not taken, for `looking_at_the_catalogue`'s reason: `Entry::ran`
/// takes the authorisation's own `at`, which is the moment the call was
/// authorised and the moment that belongs in a record.
pub(crate) fn asking_the_record(
    machine: &Machine<'_>,
    authorised: Authorised,
) -> (Entry, Result<WhatTouchedIt, NotDone>) {
    let strings = machine.strings();

    // The only reader of the call here, as `alo_software::looked_at` is for
    // the catalogue: an executor that read an argument itself would be a
    // second opinion about what arrived.
    let file = match asked_about(&authorised) {
        Ok(file) => file.to_path_buf(),
        Err(not_its_question) => {
            return (
                Entry::ran(&authorised, strings),
                Err(NotDone::CouldNotAskTheRecord(WentWrongAsking::from(
                    not_its_question,
                ))),
            );
        }
    };

    // **The record read back, rather than one the machine holds in memory.**
    // A daemon appends each entry to a file and keeps nothing, so the thing
    // that can answer a question about what happened is the file — and a file
    // can refuse, which is the one outcome here that is not a fault.
    let record = match machine.written_down() {
        Ok(record) => record,
        Err(not_read_back) => {
            return (
                Entry::ran(&authorised, strings),
                Err(NotDone::CouldNotAskTheRecord(WentWrongAsking::from(
                    not_read_back,
                ))),
            );
        }
    };

    // **The path as the person wrote it**, which is the path the record holds:
    // `Written::Path` kept what the call carried, and `What::touched` compares
    // the two exactly. Nothing here resolves anything — a resolution would
    // compare a path this machine worked out against one written down before
    // it, and the two can differ for reasons that have nothing to do with the
    // question.
    let found = what_touched(&record, &file.to_string_lossy());
    (Entry::ran(&authorised, strings), Ok(found))
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    clippy::indexing_slicing,
    reason = "in a test, a panic on an unexpected None or Err, or an index past a length the \
              test just asserted, is the failure being reported"
)]
#[path = "asking_the_record_tests.rs"]
mod tests;
