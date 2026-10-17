//! From *an agent asked what a person could install* to the list, inside the
//! turn.
//!
//! The third executor on this machine and **the first that is a read**.
//! `crate::carrying` carries the six file verbs, `crate::installing_an_application`
//! the installation, `crate::acting_on_an_application` the three application
//! acts — and every one of those is a change that waited for something. This
//! one answers and nothing on the machine moves.
//!
//! # It is the shortest executor here, and that is the whole point
//!
//! There is no tool, no indicator, no bound an organisation set, no boundary and
//! no clock. `alo_software::Catalogue::decided` parses `include_str!` of a file
//! this repository ships and checks it against another, so:
//!
//! - **it cannot fail for want of a machine.** A seat with no rented tool, no
//!   network and no disk answers this read exactly as a working machine does;
//! - **nothing leaves.** Law 1's indicator has nothing to show, because no byte
//!   goes anywhere. The read is not *quiet* egress; it is *no* egress;
//! - **there is no boundary around it** — ADR 0013's boundary is made out of
//!   resolved paths and this verb reaches none, which is
//!   `installing_an_application`'s own argument one verb along.
//!
//! # The one refusal, and why it is unreachable
//!
//! `alo_software::looked_at` refuses an authority for another verb, one that
//! came from an approval, and one whose kind of work is not among the nine.
//! None can be reached from outside: `alo_capability` validated the call
//! against the declaration, `Authorised::read` refuses a change offered as a
//! read, and the options exist before the model does. It is still a `Result`,
//! because a library inside the daemon reporting its own mistake must not take
//! the daemon down with it.
//!
//! # The entry comes back on every road
//!
//! Including the refusal — `crate::carrying`'s promise, kept here: a turn that
//! answered without one would be a thing that happened with no evidence of it.
//! **A read is recorded exactly as a change is.** `CLAUDE.md`'s gate asks for
//! *every execution and every refusal leaves a record*, and it does not say
//! *every change*: a person looking at what their agent did should see the
//! questions it asked as well as the things it altered.

use alo_capability::Authorised;
use alo_record::Entry;
use alo_software::{Catalogue, WhatTheCatalogueHolds};

use crate::machine::Machine;
use crate::refusing::NotDone;

/// Answer what the catalogue holds, and say what happened.
///
/// `now` is not taken, unlike every other executor here: there is no moment in
/// this answer. The catalogue is the same at every moment, and
/// `Entry::ran` takes the authorisation's own `at` — the moment the call was
/// authorised, which is the moment that belongs in a record.
pub(crate) fn looking_at_the_catalogue(
    machine: &Machine<'_>,
    authorised: Authorised,
) -> (Entry, Result<WhatTheCatalogueHolds, NotDone>) {
    let strings = machine.strings();

    // The only reader of the call in this file, as `alo_software::approved` is
    // for the installation.
    let kind = match alo_software::looked_at(&authorised) {
        Ok(kind) => kind,
        Err(not_a_look) => {
            return (
                Entry::ran(&authorised, strings),
                Err(NotDone::CouldNotLookAtTheCatalogue(not_a_look.into())),
            );
        }
    };

    // **The catalogue, read here rather than held by the machine.** Holding one
    // would be a second copy of a compiled-in constant with nothing keeping it
    // in step, and `crate::Machine` is for the things a turn needs that come
    // from outside this process — a tool, an indicator, a resolver. A `const`
    // is not one of those.
    let holds = match Catalogue::decided() {
        Ok(catalogue) => WhatTheCatalogueHolds::under(&catalogue, kind),
        Err(not_catalogued) => {
            // Unreachable on a machine whose tests ran: `alo-software`'s own
            // suite reads this file and refuses it four ways. Said rather than
            // unwrapped, for the reason in this file's header.
            return (
                Entry::ran(&authorised, strings),
                Err(NotDone::CouldNotLookAtTheCatalogue(not_catalogued.into())),
            );
        }
    };

    (Entry::ran(&authorised, strings), Ok(holds))
}
