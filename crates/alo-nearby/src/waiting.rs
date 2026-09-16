//! One proposal waiting on this machine, which is the value a surface shows.
//!
//! Task 7 of the local-network plan owes `alo-shell` *a value with the
//! proposal, the code and the two confirmations on it*. This is that value.
//! A surface reads [`proposal`](Waiting::proposal) for what is asked,
//! [`code`](Waiting::code) for the six digits both people compare,
//! [`confirmed_here`](Waiting::confirmed_here) and
//! [`confirmed_there`](Waiting::confirmed_there) for where the two people
//! stand, and [`until`](Waiting::until) for when it lapses if nobody answers.
//!
//! # What it holds that a surface does not see
//!
//! The [`Deliberating`] underneath — this machine's key once both offers are
//! known — and where the other machine answers, measured by discovery and
//! never typed. Neither is anything a surface needs, and the key is nothing a
//! surface may have. The transitions that change a waiting proposal are
//! [`crate::Proposals`]', and this file is what one of them looks like from
//! outside.

use std::net::SocketAddr;
use std::num::NonZeroU32;
use std::time::SystemTime;

use crate::confirming::Confirmation;
use crate::deliberating::{Deliberating, Proposal, Side};
use crate::keying::Code;
use crate::machine::MachineId;
use crate::pairing::{NotPaired, Pairing};
use crate::proposals::{NotProposed, WHILE_A_PROPOSAL_WAITS};

/// A proposal waiting for two people, as this machine holds it.
///
/// Not `Clone`: it holds this machine's key, and there is one of each
/// proposal on each machine.
#[derive(Debug)]
pub struct Waiting {
    /// The proposal, and where the two people stand on it.
    deliberating: Deliberating,
    /// When it began waiting here, on this machine's clock.
    since: SystemTime,
    /// When it lapses, which is a stated time after it began.
    until: SystemTime,
    /// Where the other machine answers, as discovery measured it.
    other_at: SocketAddr,
    /// The interface of the network it was heard on, where discovery measured
    /// one — what a connection to it is held to
    /// ([`crate::Found::on_the_network`]).
    other_on: Option<NonZeroU32>,
}

impl Waiting {
    /// A proposal that began waiting here at `since`, with the other machine
    /// answering at `other_at` on the network `other_on`.
    ///
    /// A clock so near the end of time that the moment it lapses cannot be
    /// represented lapses at once, which is a true answer about a clock that
    /// far wrong.
    pub(crate) fn begun(
        deliberating: Deliberating,
        since: SystemTime,
        other_at: SocketAddr,
        other_on: Option<NonZeroU32>,
    ) -> Self {
        Self {
            deliberating,
            since,
            until: since.checked_add(WHILE_A_PROPOSAL_WAITS).unwrap_or(since),
            other_at,
            other_on,
        }
    }

    /// The network the other machine was heard on, where discovery measured
    /// one: what a confirmation to it is held to.
    #[must_use]
    pub const fn where_the_other_was_heard(&self) -> Option<NonZeroU32> {
        self.other_on
    }

    /// What is proposed.
    #[must_use]
    pub const fn proposal(&self) -> &Proposal {
        self.deliberating.proposal()
    }

    /// Which of the two machines this one is.
    #[must_use]
    pub const fn on(&self) -> Side {
        self.deliberating.on()
    }

    /// This machine.
    #[must_use]
    pub const fn here(&self) -> &MachineId {
        match self.on() {
            Side::TheOneAsking => self.proposal().asking(),
            Side::TheOneAsked => self.proposal().asked(),
        }
    }

    /// The other machine.
    #[must_use]
    pub const fn other(&self) -> &MachineId {
        match self.on() {
            Side::TheOneAsking => self.proposal().asked(),
            Side::TheOneAsked => self.proposal().asking(),
        }
    }

    /// The six digits both people compare, once the other machine's offer is
    /// known here. `None` until then, and a surface has nothing to confirm
    /// before then.
    #[must_use]
    pub fn code(&self) -> Option<Code> {
        self.deliberating.code()
    }

    /// Whether the person at one of the two machines has confirmed.
    #[must_use]
    pub const fn confirmed(&self, side: Side) -> bool {
        self.deliberating.has_agreed(side)
    }

    /// Whether the person at this machine has confirmed.
    #[must_use]
    pub const fn confirmed_here(&self) -> bool {
        self.confirmed(self.on())
    }

    /// Whether the person at the other machine has confirmed, as far as this
    /// machine has been told and could verify.
    #[must_use]
    pub const fn confirmed_there(&self) -> bool {
        self.confirmed(other_side(self.on()))
    }

    /// When it began waiting here.
    #[must_use]
    pub const fn since(&self) -> SystemTime {
        self.since
    }

    /// When it lapses if nobody answers: [`WHILE_A_PROPOSAL_WAITS`] after it
    /// began.
    #[must_use]
    pub const fn until(&self) -> SystemTime {
        self.until
    }

    /// Whether it has lapsed at `now`.
    #[must_use]
    pub fn has_lapsed(&self, now: SystemTime) -> bool {
        now >= self.until
    }

    /// Where the other machine answers: the address discovery measured and
    /// the port it advertised. What a confirmation from here is dialled to.
    #[must_use]
    pub const fn where_the_other_answers(&self) -> SocketAddr {
        self.other_at
    }

    /// The confirmation this machine sends the other, once the code is known
    /// here. `None` before then: there is nothing to have confirmed.
    pub(crate) fn confirmation(&self) -> Option<Confirmation> {
        let key = self.deliberating.key()?;
        let transcript = self.deliberating.transcript()?;
        Some(Confirmation::made(
            key,
            &transcript,
            self.here(),
            self.other(),
        ))
    }

    /// Whether a confirmation that arrived is the other machine's.
    ///
    /// # Errors
    ///
    /// [`NotProposed::NotAnsweredYet`] if the code is not known here, so there
    /// is no key to check it with; [`NotProposed::NotConfirmedByThatMachine`]
    /// if it names the wrong machines or its tag was not made with the key
    /// the other machine holds.
    pub(crate) fn holds(&self, confirmation: &Confirmation) -> Result<(), NotProposed> {
        let (Some(key), Some(transcript)) =
            (self.deliberating.key(), self.deliberating.transcript())
        else {
            return Err(NotProposed::NotAnsweredYet);
        };
        if confirmation.from() != self.other()
            || confirmation.to() != self.here()
            || !confirmation.is_by(key, &transcript)
        {
            return Err(NotProposed::NotConfirmedByThatMachine);
        }
        Ok(())
    }

    /// The person at this machine confirmed.
    pub(crate) fn confirmed_by_the_person_here(self) -> Self {
        let side = self.on();
        Self {
            deliberating: self.deliberating.agreed_at(side),
            ..self
        }
    }

    /// The person at the other machine confirmed, which is said only after
    /// [`holds`](Self::holds) did.
    pub(crate) fn confirmed_by_the_person_there(self) -> Self {
        let side = other_side(self.on());
        Self {
            deliberating: self.deliberating.agreed_at(side),
            ..self
        }
    }

    /// Whether both people have confirmed.
    #[must_use]
    pub const fn is_mutual(&self) -> bool {
        self.deliberating.is_mutual()
    }

    /// The pairing, if both people confirmed, as this machine keeps it.
    ///
    /// # Errors
    ///
    /// As [`Deliberating::agreed`] answers.
    pub(crate) fn kept(self, at: SystemTime) -> Result<Pairing, NotPaired> {
        self.deliberating.agreed(at)
    }

    /// The other machine's offer, for the asked machine to answer with.
    pub(crate) const fn answered(&self) -> Option<&crate::keying::Offer> {
        self.deliberating.answered()
    }

    /// The deliberation, for the one transition that has to consume it.
    pub(crate) fn into_deliberating(
        self,
    ) -> (Deliberating, SystemTime, SocketAddr, Option<NonZeroU32>) {
        (self.deliberating, self.since, self.other_at, self.other_on)
    }
}

/// The side that is not this one.
const fn other_side(side: Side) -> Side {
    match side {
        Side::TheOneAsking => Side::TheOneAsked,
        Side::TheOneAsked => Side::TheOneAsking,
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use std::net::SocketAddr;
    use std::time::Duration;

    use super::Waiting;
    use crate::deliberating::{Deliberating, Side};
    use crate::proposals::{NotProposed, WHILE_A_PROPOSAL_WAITS};
    use crate::testing::{a_moment, a_proposal, both_sides, reception, studio};

    /// Where the tests say the other machine answers.
    fn there() -> SocketAddr {
        "192.168.1.20:7610".parse().unwrap()
    }

    /// **What a surface reads**: the proposal, the code once known, the two
    /// confirmations, and when it lapses.
    #[test]
    fn what_a_surface_reads_is_the_proposal_the_code_and_the_two_confirmations() {
        let (proposal, keying) = a_proposal();
        let waiting = Waiting::begun(
            Deliberating::asking(proposal.clone(), keying).unwrap(),
            a_moment(),
            there(),
            None,
        );
        assert_eq!(waiting.proposal(), &proposal);
        assert_eq!(waiting.on(), Side::TheOneAsking);
        assert_eq!(waiting.here(), &reception());
        assert_eq!(waiting.other(), &studio());
        assert!(waiting.code().is_none());
        assert!(!waiting.confirmed_here() && !waiting.confirmed_there());
        assert_eq!(waiting.since(), a_moment());
        assert_eq!(waiting.until(), a_moment() + WHILE_A_PROPOSAL_WAITS);
        assert_eq!(waiting.where_the_other_answers(), there());
        assert!(!waiting.has_lapsed(a_moment()));
        assert!(!waiting.has_lapsed(a_moment() + WHILE_A_PROPOSAL_WAITS - Duration::from_secs(1)));
        assert!(waiting.has_lapsed(a_moment() + WHILE_A_PROPOSAL_WAITS));
    }

    /// On the asked machine, `here` and `other` are the other way round, and
    /// the code is known from the start.
    #[test]
    fn on_the_asked_machine_the_sides_are_the_other_way_round() {
        let (_, at_studio) = both_sides();
        let waiting = Waiting::begun(at_studio, a_moment(), there(), None);
        assert_eq!(waiting.on(), Side::TheOneAsked);
        assert_eq!(waiting.here(), &studio());
        assert_eq!(waiting.other(), &reception());
        assert!(waiting.code().is_some());
    }

    /// **A confirmation from the other machine holds, and one from anybody
    /// else does not** — and before the answer there is nothing to check it
    /// with.
    #[test]
    fn a_confirmation_holds_only_from_the_other_machine_and_only_once_answered() {
        let (at_reception, at_studio) = both_sides();
        let at_reception = Waiting::begun(at_reception, a_moment(), there(), None);
        let at_studio = Waiting::begun(at_studio, a_moment(), there(), None);

        let from_studio = at_studio.confirmation().unwrap();
        assert!(at_reception.holds(&from_studio).is_ok());
        // Reception's own, presented back to it.
        let its_own = at_reception.confirmation().unwrap();
        assert_eq!(
            at_reception.holds(&its_own).unwrap_err(),
            NotProposed::NotConfirmedByThatMachine
        );
        // A stranger's, from some other agreement.
        let (_, elsewhere) = both_sides();
        let forged = Waiting::begun(elsewhere, a_moment(), there(), None)
            .confirmation()
            .unwrap();
        assert_eq!(
            at_reception.holds(&forged).unwrap_err(),
            NotProposed::NotConfirmedByThatMachine
        );

        let (proposal, keying) = a_proposal();
        let unanswered = Waiting::begun(
            Deliberating::asking(proposal, keying).unwrap(),
            a_moment(),
            there(),
            None,
        );
        assert!(unanswered.confirmation().is_none());
        assert_eq!(
            unanswered.holds(&from_studio).unwrap_err(),
            NotProposed::NotAnsweredYet
        );
    }

    /// The two confirmations are kept apart, and a pairing is kept only from
    /// both.
    #[test]
    fn the_two_confirmations_are_kept_apart_and_a_pairing_needs_both() {
        let (at_reception, _) = both_sides();
        let waiting =
            Waiting::begun(at_reception, a_moment(), there(), None).confirmed_by_the_person_here();
        assert!(waiting.confirmed_here());
        assert!(!waiting.confirmed_there());
        assert!(!waiting.is_mutual());

        let waiting = waiting.confirmed_by_the_person_there();
        assert!(waiting.is_mutual());
        let pairing = waiting.kept(a_moment()).unwrap();
        assert_eq!(pairing.with(), &studio());
    }
}
