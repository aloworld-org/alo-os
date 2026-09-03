//! From *this may run* to *this is what happened*, and the entry that says so.
//!
//! A read and an approved change reach this point by different roads — one
//! answers inside the turn, the other waited for a person — and from here they
//! are the same three questions, asked in the same order:
//!
//! 1. **Where does every path this call names really lead, and do the grants
//!    cover that?** [`Touching`] asks it, at the authorisation's own moment.
//!    Containment is decided lexically and touches no disk, so a link out of a
//!    granted folder is caught here and nowhere else.
//! 2. **May anything this call would create be created?** [`Did`] asks it. A
//!    rename invents a name and a move invents one inside a folder, and under a
//!    grant over a single file that name is one nobody granted.
//! 3. **And then the machine does it**, or says why it could not.
//!
//! # This file decides nothing and writes nothing down
//!
//! It answers with the entry and with what the caller gets, and
//! [`crate::Turning`] writes the first before handing over the second. That is
//! the crate's whole promise in one place rather than in three: every road out
//! of here carries an entry, so there is no branch a caller could take that
//! leaves nothing written.
//!
//! The moment is taken off the [`Authorised`] before it is consumed, because it
//! is the moment the grants were asked and every entry about this call belongs
//! at it. A second reading of a clock would be a record saying the machine got
//! round to writing at a different time from the one it decided at.

use alo_capability::{Authorised, Grants};
use alo_files::{Answer, Did, Touching};
use alo_record::Entry;

use crate::machine::Machine;
use crate::refusing::NotDone;

/// Carry an authorised call out, and say what happened.
///
/// The entry comes back on every road, including both refusals: a call the
/// grants stopped at the last moment is a thing that happened, and a call the
/// machine could not manage is one that was attempted and is recorded as one
/// (`alo-files`' rule, kept here).
pub(crate) fn carrying_out(
    machine: &Machine<'_>,
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
    let did = match Did::of(touching, grants, strings) {
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
