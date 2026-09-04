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
//! the crate's whole promise in one place rather than in three: every road out
//! of here carries an entry, so there is no branch a caller could take that
//! leaves nothing written.
//!
//! **The one road that carries no entry does not come out of here at all**, and
//! that is why it is an `Err` on this function rather than an entry that might
//! be absent. A turn that could not be bounded did not run: no cgroup, no
//! syscall, nothing asked of the disk and no grant consulted. [`crate::NoAnswer`]
//! and [`crate::NotDone::NotAnswered`] are the same shape — the record keeps
//! what happened on this machine, and this is a machine that could not do
//! anything at all rather than an agent that was stopped.
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
use crate::unbounded::NoBoundary;

/// Carry an authorised call out inside a boundary, and say what happened.
///
/// The entry comes back on every road that got as far as the machine, including
/// both refusals: a call the grants stopped at the last moment is a thing that
/// happened, and a call the machine could not manage is one that was attempted
/// and is recorded as one (`alo-files`' rule, kept here).
///
/// # Errors
/// [`NoBoundary`] when there was none to run this inside. Nothing was done and
/// nothing is written down — see this module's documentation.
pub(crate) fn carrying_out(
    machine: &mut Machine<'_>,
    authorised: Authorised,
    grants: &Grants,
) -> Result<(Entry, Result<Answer, NotDone>), NoBoundary> {
    let at = authorised.at();
    let agent = authorised.under().clone();
    let strings = machine.strings();

    let touching = match Touching::of(authorised, grants, machine.resolving(), strings) {
        Ok(touching) => touching,
        Err(refused) => {
            let entry = Entry::refused(&refused, &agent, strings, at);
            return Ok((entry, Err(NotDone::Refused(refused))));
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
            return Ok((
                Entry::ran(&authorised, strings),
                Err(NotDone::MachineCouldNot(failed)),
            ));
        }
    };

    let done = machine
        .bounding()
        .carrying_out(&reaching, Doing::of(touching, grants, strings))?;
    let did = match done {
        Ok(did) => did,
        Err(refused) => {
            let entry = Entry::refused(&refused, &agent, strings, at);
            return Ok((entry, Err(NotDone::Refused(refused))));
        }
    };

    let (authorised, outcome) = did.into_parts();
    Ok((
        Entry::ran(&authorised, strings),
        outcome.map_err(NotDone::MachineCouldNot),
    ))
}
