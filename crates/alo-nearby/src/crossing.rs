//! What this machine carries to the other over the wire: a proposal, and a
//! confirmation.
//!
//! Two functions, each a transition from `proposals.rs` with the dialling
//! from `dialling.rs` in the middle of it, so that a shell has one call to
//! make for *propose to that machine* and one for *the person here
//! confirmed*. Neither decides anything the pure transitions do not.
//!
//! # A confirmation is sent first and counted after
//!
//! [`confirm`] makes the confirmation, dials it, and only then counts the
//! person here as having confirmed. If the other machine could not be
//! reached, nothing here has changed, and the person tries again — rather
//! than this machine holding a confirmation the other never heard, which is
//! the one state that would leave the two machines disagreeing about whether
//! they are paired.

use std::time::{Duration, SystemTime};

use crate::deliberating::Proposal;
use crate::dialling;
use crate::keying::{Keying, Offer};
use crate::machine::MachineId;
use crate::pairing::Pairing;
use crate::permitting::MayAskIts;
use crate::presence::Found;
use crate::proposals::{NotProposed, Proposals};
use crate::receiving::{THE_CONFIRMATION_PATH, THE_PROPOSAL_PATH};
use crate::waiting::Waiting;

/// Propose to a machine discovery found, and bring back its answer.
///
/// Dials [`Found::where_it_answers`] and nothing else. What comes back is the
/// proposal waiting here with the code known, for the surface to show the
/// person beside the confirmation.
///
/// `now` is the moment the proposal begins waiting; the dial takes up to
/// `http::WHILE_THE_WIRE_ANSWERS` after it, which against
/// [`crate::WHILE_A_PROPOSAL_WAITS`] is nothing.
///
/// # Errors
///
/// [`NotProposed`] as [`Proposals::proposed`] answers before anything is
/// sent; [`NotProposed::Underneath`] if the machine could not be reached or
/// what it answered could not be read; and whatever the other machine wrote
/// back, read by [`NotProposed::off_the_wire`]. After any of them nothing is
/// waiting here.
pub fn propose<'a>(
    proposals: &'a mut Proposals,
    to: &Found,
    may: &[MayAskIts],
    lasting: Duration,
    now: SystemTime,
) -> Result<&'a Waiting, NotProposed> {
    let keying = Keying::fresh().map_err(NotProposed::Underneath)?;
    let proposal = Proposal::checked(
        proposals.here().clone(),
        to.machine.clone(),
        may,
        lasting,
        keying.offer().clone(),
    )
    .map_err(NotProposed::NotPaired)?;
    let said = format!("{}\n", proposal.said());
    proposals.proposed(proposal, keying, to, now)?;

    let answered = match dialling::put(to.where_it_answers(), THE_PROPOSAL_PATH, &said) {
        Ok(answered) => answered,
        Err(why) => {
            proposals.withdrawn(&to.machine);
            return Err(NotProposed::Underneath(why));
        }
    };
    if answered.status != 200 {
        proposals.withdrawn(&to.machine);
        return Err(NotProposed::off_the_wire(&answered.body));
    }
    let offer = match Offer::read(answered.body.trim()) {
        Ok(offer) => offer,
        Err(why) => {
            proposals.withdrawn(&to.machine);
            return Err(NotProposed::Underneath(why));
        }
    };
    proposals.answered(&to.machine, offer, now)
}

/// The person here confirmed the proposal with `other`: tell the other
/// machine, and count it once told.
///
/// Dials [`Waiting::where_the_other_answers`] and nothing else. Answers the
/// pairing if this was the second of the two confirmations, in which case it
/// is the caller's to keep.
///
/// # Errors
///
/// [`NotProposed`] as [`Proposals::confirmation_for`] answers before anything
/// is sent; [`NotProposed::Underneath`] if the other machine could not be
/// reached; and whatever it wrote back. After any of them the person here has
/// not been counted as having confirmed, and may try again.
pub fn confirm(
    proposals: &mut Proposals,
    other: &MachineId,
    now: SystemTime,
) -> Result<Option<Pairing>, NotProposed> {
    let confirmation = proposals.confirmation_for(other, now)?;
    let at = proposals
        .with(other)
        .map(Waiting::where_the_other_answers)
        .ok_or(NotProposed::NothingWaiting)?;
    let said = format!("{}\n", confirmation.said());
    let answered =
        dialling::put(at, THE_CONFIRMATION_PATH, &said).map_err(NotProposed::Underneath)?;
    if answered.status != 204 {
        return Err(NotProposed::off_the_wire(&answered.body));
    }
    proposals.confirmed_here(other, now)
}
