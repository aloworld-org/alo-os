//! From *this may run* to *this is what happened*, and the entry that says so.
//!
//! A read and an approved change reach this point by different roads — one
//! answers inside the turn, the other waited for a person — and from here they
//! are the same four questions, asked in the same order:
//!
//! 1. **Where does every path this call names really lead, and do the grants
//!    cover that?** [`Touching`] asks it, at the authorisation's own moment.
//!    Containment is decided lexically and touches no disk, so a link out of a
//!    granted folder is caught here and nowhere else.
//! 2. **Where does this execution have to be able to reach?** [`Reaching`]
//!    answers it: the resolved paths, plus the folder above anything the call
//!    would create.
//! 3. **May anything this call would create be created?** [`Did`] asks it, and
//!    it is the one question asked *inside* the boundary. A rename invents a
//!    name and a move invents one inside a folder, and under a grant over a
//!    single file that name is one nobody granted.
//! 4. **And then the machine does it**, or says why it could not.
//!
//! # The boundary is around the verb and not around the entry
//!
//! Steps 3 and 4 happen inside a boundary the kernel imposes ([`Bounding`]) and
//! nothing else does. That is not tidiness in either direction:
//!
//! **Resolving is outside** because a reach is made of resolved paths, so a
//! thread inside a boundary would have to look up what it is not yet allowed to
//! open. **Writing the entry is outside** because the record is a file the
//! grants say nothing about, and a thread bounded across both would be refused
//! the evidence of what it had just done.
//!
//! # This file decides nothing and writes nothing down
//!
//! It answers with the entry and with what the caller gets, and
//! [`crate::Turning`] writes the first before handing over the second. That is
//! the crate's whole promise in one place rather than in three: **every road
//! out of here carries an entry**, so there is no branch a caller could take
//! that leaves nothing written.
//!
//! **That includes the road where there was no boundary**, and until
//! 2026-09-12 it did not. A turn that could not be bounded did not run — no
//! cgroup, no syscall, nothing asked of the disk and no grant consulted — and
//! this file argued that there was nothing true to write. There was: the
//! machine refused to run the agent's turn, which is a refusal, and a record
//! that kept every refusal but the machine's own showed a boundary that had
//! gone as a record that simply stopped. So the road out of a boundary that
//! could not be applied carries [`Entry::not_bounded`], with the sentence the
//! person was shown and the machine's own account of which pin was gone, and
//! [`crate::NotDone::NotBounded`] beside it — the same shape as
//! [`crate::NotDone::MachineCouldNot`], which is the other thing a machine
//! rather than a grant can say no to.
//!
//! The moment is taken off the [`Authorised`] before it is consumed, because it
//! is the moment the grants were asked and every entry about this call belongs
//! at it. A second reading of a clock would be a record saying the machine got
//! round to writing at a different time from the one it decided at.

use alo_capability::{Authorised, Grants};
use alo_files::{Answer, Reaching, Touching};
use alo_record::Entry;

use crate::bounding::Doing;
use crate::machine::Machine;
use crate::refusing::NotDone;

/// Carry an authorised call out inside a boundary, and say what happened.
///
/// The entry comes back on every road, including all three refusals: a call
/// the grants stopped at the last moment is a thing that happened, a call the
/// machine could not manage is one that was attempted and is recorded as one
/// (`alo-files`' rule, kept here), and a call the machine would not run
/// because it could not bound it is the machine's own refusal, written down
/// as such.
pub(crate) fn carrying_out(
    machine: &mut Machine<'_>,
    authorised: Authorised,
    grants: &Grants,
) -> (Entry, Result<Answer, NotDone>) {
    let at = authorised.at();
    let agent = authorised.under().clone();
    let strings = machine.strings();

    let touching = match Touching::of(authorised, grants, machine.resolving(), strings) {
        Ok(touching) => touching,
        Err(refused) => {
            let entry = Entry::refused(&refused, &agent, strings, at);
            return (entry, Err(NotDone::Refused(refused)));
        }
    };

    // Everywhere the work will have to open, worked out before the work can
    // open anything. A call of the six always reaches at least one place, so
    // this refuses only a verb something else declared — and it is the same
    // answer, and the same entry, as the machine failing at it a moment later.
    let reaching = match Reaching::of(&touching) {
        Ok(reaching) => reaching,
        Err(failed) => {
            let authorised = touching.into_authorised();
            return (
                Entry::ran(&authorised, strings),
                Err(NotDone::MachineCouldNot(failed)),
            );
        }
    };

    let done = match machine
        .bounding()
        .carrying_out(&reaching, Doing::of(touching, grants, strings))
    {
        Ok(done) => done,
        Err(no_boundary) => {
            let said = no_boundary.said(strings);
            let entry = Entry::not_bounded(&agent, said.text(), no_boundary.why(), at);
            return (entry, Err(NotDone::NotBounded(no_boundary)));
        }
    };
    let did = match done {
        Ok(did) => did,
        Err(refused) => {
            let entry = Entry::refused(&refused, &agent, strings, at);
            return (entry, Err(NotDone::Refused(refused)));
        }
    };

    let (authorised, outcome) = did.into_parts();
    (
        Entry::ran(&authorised, strings),
        outcome.map_err(NotDone::MachineCouldNot),
    )
}
