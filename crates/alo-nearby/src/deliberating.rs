//! How a pairing is made, which is the only way one can come into existence.
//!
//! ADR 0003: *use requires confirmation on both machines.* This file is that
//! sentence with no other road round it. [`Pairing`](crate::Pairing) has no
//! public constructor; the only thing that returns one is
//! [`Deliberating::agreed`], and it refuses unless both people have said yes on
//! their own machine.
//!
//! # Why the refusal is the subject here
//!
//! A pairing nobody can refuse is worse than no pairing at all, so this file
//! carries as many tests about **not** pairing as about pairing. One side
//! agreeing leaves nothing paired, and — the one that matters — *the same side
//! agreeing twice* leaves nothing paired either. That is the shape a
//! well-meaning convenience takes: a machine that asks, gets no answer, asks
//! again, and counts its own two attempts as agreement.
//!
//! # One deliberation per machine, and what crosses between them
//!
//! ADR 0031. A pairing leaves each machine holding a key nobody else has, and
//! the key is agreed here: each side holds a [`Keying`] of its own, the asking
//! machine's [`Offer`] travels inside the [`Proposal`], and the asked machine's
//! offer travels back with its answer. So there is a [`Deliberating`] **on each
//! machine** — [`Deliberating::asking`] on the one that proposed,
//! [`Deliberating::asked`] on the one it was carried to — and each computes the
//! same key from its own private half and the other's offer, over the terms
//! both people were shown. Nothing that would let a third machine compute it
//! crosses the wire.
//!
//! What a third machine *could* do is stand in between at the moment of
//! pairing and hand each side an offer of its own. [`Deliberating::code`] is
//! what stops that: six digits from both identities and both offers, shown on
//! both machines, and two people who read the same six digits are not being
//! intercepted. The surface owes the person that code beside the confirmation.
//!
//! Nothing here reads the clock either. The moment a pairing starts is passed
//! in, the same way `alo_capability::Grant` takes it, so that what a pairing
//! does at a moment can be asked about a moment that is not now.

use std::time::Duration;

use crate::keying::{Code, Keying, Offer, Transcript};
use crate::machine::MachineId;
use crate::pairing::{NotPaired, Pairing};
use crate::permitting::MayAskIts;

/// The longest a pairing may be asked for.
///
/// Thirty days. Not a policy and not configurable — an upper bound, so that
/// *expiring* is a property of the type rather than of whoever filled in the
/// number. A person who wants the machine down the corridor for a year pairs
/// with it again next month, having been reminded that they are doing it.
pub const AT_MOST: Duration = Duration::from_secs(60 * 60 * 24 * 30);

/// What one machine's person is asking another's for.
///
/// Made on the machine that wants something, carried to the machine that has
/// it, and shown to both people before either says yes. It carries the asking
/// machine's [`Offer`], which is public and is the half of the key agreement
/// that travels this way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proposal {
    /// The machine asking.
    asking: MachineId,
    /// The machine being asked.
    asked: MachineId,
    /// What is being asked for, enumerated.
    may: Vec<MayAskIts>,
    /// How long it would last.
    lasting: Duration,
    /// The asking machine's public half.
    offered: Offer,
}

impl Proposal {
    /// One machine's person asking another's, checked before anybody is shown
    /// anything.
    ///
    /// `offered` is the asking machine's [`Offer`], from the [`Keying`] it
    /// keeps for this proposal.
    ///
    /// # Errors
    ///
    /// [`NotPaired::WithItself`] for a machine proposing to itself, which is
    /// not a pairing and would make every later question about *the other
    /// machine* ambiguous; [`NotPaired::NothingAsked`] for an empty list,
    /// because a pairing that permits nothing is a row in a list that does
    /// nothing and confuses everybody who reads it; [`NotPaired::NoTime`] for
    /// zero; and [`NotPaired::TooLong`] past [`AT_MOST`].
    pub fn checked(
        asking: MachineId,
        asked: MachineId,
        may: &[MayAskIts],
        lasting: Duration,
        offered: Offer,
    ) -> Result<Self, NotPaired> {
        if asking == asked {
            return Err(NotPaired::WithItself);
        }
        if may.is_empty() {
            return Err(NotPaired::NothingAsked);
        }
        if lasting.is_zero() {
            return Err(NotPaired::NoTime);
        }
        if lasting > AT_MOST {
            return Err(NotPaired::TooLong);
        }
        let mut may = may.to_vec();
        may.sort_unstable();
        may.dedup();
        Ok(Self {
            asking,
            asked,
            may,
            lasting,
            offered,
        })
    }

    /// The machine asking.
    #[must_use]
    pub const fn asking(&self) -> &MachineId {
        &self.asking
    }

    /// The machine being asked.
    #[must_use]
    pub const fn asked(&self) -> &MachineId {
        &self.asked
    }

    /// What is being asked for, in order and without repeats.
    #[must_use]
    pub fn may(&self) -> &[MayAskIts] {
        &self.may
    }

    /// How long it would last, which is stated here rather than assumed
    /// anywhere later.
    #[must_use]
    pub const fn lasting(&self) -> Duration {
        self.lasting
    }

    /// The asking machine's public half.
    #[must_use]
    pub const fn offered(&self) -> &Offer {
        &self.offered
    }
}

/// Which machine a person is standing in front of.
///
/// A `bool` would have done, and would have been the bug: `confirm(true)` reads
/// as *yes* rather than as *the machine that asked*, and the mistake being
/// guarded against is precisely a machine counting its own agreement twice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// The machine that made the proposal.
    TheOneAsking,
    /// The machine the proposal was carried to.
    TheOneAsked,
}

/// A proposal that is waiting for two people, on one of the two machines.
///
/// Deliberately not `Clone` and deliberately consuming: it holds this
/// machine's private half, and [`agreed`](Self::agreed) takes the deliberation,
/// so a pairing is made from it once.
#[derive(Debug)]
pub struct Deliberating {
    /// What is being proposed.
    proposal: Proposal,
    /// Which machine this deliberation is on.
    on: Side,
    /// This machine's part of the key.
    keying: Keying,
    /// The asked machine's public half, once known: its own, on the asked
    /// machine; what came back, on the asking one.
    answered: Option<Offer>,
    /// Whether the person at the asking machine has said yes.
    asking_agreed: bool,
    /// Whether the person at the asked machine has said yes.
    asked_agreed: bool,
}

impl Deliberating {
    /// The proposal, on the machine that made it, with the keying whose offer
    /// it carries.
    ///
    /// # Errors
    ///
    /// [`NotPaired::NotTheOfferMade`] if `keying`'s offer is not the one in
    /// the proposal: a program that lost track of which keying went with which
    /// proposal would otherwise agree a key the other machine never will.
    pub fn asking(proposal: Proposal, keying: Keying) -> Result<Self, NotPaired> {
        if keying.offer() != &proposal.offered {
            return Err(NotPaired::NotTheOfferMade);
        }
        Ok(Self {
            proposal,
            on: Side::TheOneAsking,
            keying,
            answered: None,
            asking_agreed: false,
            asked_agreed: false,
        })
    }

    /// The proposal, on the machine it was carried to, with a keying of that
    /// machine's own — whose offer is the answer it sends back.
    #[must_use]
    pub fn asked(proposal: Proposal, keying: Keying) -> Self {
        let answered = Some(keying.offer().clone());
        Self {
            proposal,
            on: Side::TheOneAsked,
            keying,
            answered,
            asking_agreed: false,
            asked_agreed: false,
        }
    }

    /// The asked machine's offer, carried back to the asking machine.
    ///
    /// # Errors
    ///
    /// [`NotPaired::NotTheOfferMade`] on the asked machine, whose answer is
    /// its own and not something that arrives — and on either machine if the
    /// offer is the asking machine's own reflected back, which is what an
    /// attacker in between who cannot make a key of its own would send.
    pub fn answered_with(mut self, offer: Offer) -> Result<Self, NotPaired> {
        if self.on == Side::TheOneAsked || offer == self.proposal.offered {
            return Err(NotPaired::NotTheOfferMade);
        }
        self.answered = Some(offer);
        Ok(self)
    }

    /// What is being proposed, for the surface that shows it to a person.
    #[must_use]
    pub const fn proposal(&self) -> &Proposal {
        &self.proposal
    }

    /// Which machine this deliberation is on.
    #[must_use]
    pub const fn on(&self) -> Side {
        self.on
    }

    /// The asked machine's offer, if it is known here yet.
    ///
    /// On the asked machine it is that machine's own, from the start; on the
    /// asking machine it is what [`answered_with`](Self::answered_with) was
    /// given. The wire carries it from the first to the second.
    #[must_use]
    pub const fn answered(&self) -> Option<&Offer> {
        self.answered.as_ref()
    }

    /// The six digits both people compare, once both offers are known here.
    ///
    /// **What the surface owes.** Shown beside the confirmation on this
    /// machine, and the person says yes only if the other person reads the
    /// same digits on theirs. `None` until the other machine's offer has
    /// arrived, and a surface has nothing to confirm before then.
    #[must_use]
    pub fn code(&self) -> Option<Code> {
        self.transcript().map(|transcript| transcript.code())
    }

    /// The person in front of one of the two machines said yes.
    ///
    /// Saying yes twice from one side is saying yes once: the field is set
    /// rather than counted, which is what makes *mutual* true however many
    /// times a machine repeats itself.
    #[must_use]
    pub const fn agreed_at(mut self, side: Side) -> Self {
        match side {
            Side::TheOneAsking => self.asking_agreed = true,
            Side::TheOneAsked => self.asked_agreed = true,
        }
        self
    }

    /// Whether both people have agreed.
    #[must_use]
    pub const fn is_mutual(&self) -> bool {
        self.asking_agreed && self.asked_agreed
    }

    /// The pairing, if both people agreed, as this machine keeps it — naming
    /// the other machine, and holding the key the two of them agreed.
    ///
    /// This is the only thing in this workspace that returns a
    /// [`Pairing`](crate::Pairing).
    ///
    /// **One agreement, two lists, each naming the other machine.** A pairing
    /// is between two machines and each keeps its own row about it — ADR
    /// 0003's *visible* is a list on each machine, and *revocable in one
    /// action* is either person's — so the row the asking machine keeps names
    /// the one it asked, and the row the asked machine keeps names the one
    /// that asked it. The two rows hold the same key, computed on each side
    /// from what only that side holds (ADR 0031).
    ///
    /// # Errors
    ///
    /// [`NotPaired::OnlyOneSideAgreed`] when one machine's person has said yes
    /// and the other has not — including when the same person has said yes
    /// twice, and including the asking machine having heard nothing back from
    /// the asked one. The refusal names no side: what is true is that the
    /// machines have not agreed, and saying *they refused* about a person who
    /// has simply not answered yet would be a sentence the machine cannot
    /// support. [`NotPaired::NoKey`] if the agreement fails, which an offer of
    /// the right length does not cause.
    pub fn agreed(self, at: std::time::SystemTime) -> Result<Pairing, NotPaired> {
        if !self.is_mutual() {
            return Err(NotPaired::OnlyOneSideAgreed);
        }
        let (Some(transcript), Some(answered)) = (self.transcript(), self.answered.as_ref()) else {
            return Err(NotPaired::OnlyOneSideAgreed);
        };
        let (other, theirs) = match self.on {
            Side::TheOneAsking => (self.proposal.asked.clone(), answered),
            Side::TheOneAsked => (self.proposal.asking.clone(), &self.proposal.offered),
        };
        let key = self.keying.agreed(theirs, &transcript)?;
        Pairing::between(other, &self.proposal.may, at, self.proposal.lasting, key)
    }

    /// The transcript both sides derive from, once both offers are known.
    fn transcript(&self) -> Option<Transcript> {
        self.answered.as_ref().map(|answered| {
            Transcript::of(
                &self.proposal.asking,
                &self.proposal.asked,
                &self.proposal.offered,
                answered,
                &self.proposal.may,
                self.proposal.lasting,
            )
        })
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use std::time::{Duration, SystemTime};

    use super::{AT_MOST, Deliberating, Proposal, Side};
    use crate::keying::Keying;
    use crate::machine::MachineId;
    use crate::pairing::NotPaired;
    use crate::permitting::MayAskIts;

    /// One of the two machines in every test here.
    fn one() -> MachineId {
        MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
    }

    /// The other.
    fn another() -> MachineId {
        MachineId::read("aaaabbbbccccddddeeeeffff00001111").unwrap()
    }

    /// A proposal for the tests that are not about proposing, and the keying
    /// whose offer it carries.
    fn a_proposal() -> (Proposal, Keying) {
        let keying = Keying::fresh().unwrap();
        let proposal = Proposal::checked(
            one(),
            another(),
            &[MayAskIts::Models],
            Duration::from_secs(86_400),
            keying.offer().clone(),
        )
        .unwrap();
        (proposal, keying)
    }

    /// Both sides of one proposal, each holding the other's offer, with
    /// nobody having agreed yet: the asking side first.
    fn both_sides() -> (Deliberating, Deliberating) {
        let (proposal, at_asking) = a_proposal();
        let asked = Deliberating::asked(proposal.clone(), Keying::fresh().unwrap());
        let asking = Deliberating::asking(proposal, at_asking)
            .unwrap()
            .answered_with(asked.answered().unwrap().clone())
            .unwrap();
        (asking, asked)
    }

    /// A moment for the tests that are not about time.
    fn a_moment() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    /// **Both people say yes, and there is a pairing** — on each machine,
    /// naming the other, with the same key on both.
    #[test]
    fn a_pairing_is_made_when_both_machines_people_agree() {
        let (asking, asked) = both_sides();
        let on_asking = asking
            .agreed_at(Side::TheOneAsking)
            .agreed_at(Side::TheOneAsked)
            .agreed(a_moment())
            .unwrap();
        let on_asked = asked
            .agreed_at(Side::TheOneAsking)
            .agreed_at(Side::TheOneAsked)
            .agreed(a_moment())
            .unwrap();
        assert_eq!(*on_asking.with(), another());
        assert_eq!(*on_asked.with(), one());
        assert!(on_asking.permits(MayAskIts::Models, a_moment()));
        assert_eq!(on_asking.key(), on_asked.key());
        // And everything else about the two rows is the same agreement.
        assert_eq!(on_asking.may(), on_asked.may());
        assert_eq!(on_asking.made(), on_asked.made());
        assert_eq!(on_asking.ends(), on_asked.ends());
    }

    /// **One side alone leaves nothing paired** — either side, on either
    /// machine, which is why the test is all four.
    #[test]
    fn one_machines_person_agreeing_alone_pairs_nothing() {
        for alone in [Side::TheOneAsking, Side::TheOneAsked] {
            let (asking, asked) = both_sides();
            for on in [asking, asked] {
                let refused = on.agreed_at(alone).agreed(a_moment()).unwrap_err();
                assert_eq!(refused, NotPaired::OnlyOneSideAgreed, "{alone:?}");
            }
        }
    }

    /// **And the same side agreeing twice leaves nothing paired**, which is the
    /// shape a well-meaning convenience takes: a machine that asks, hears
    /// nothing, asks again, and counts its own two attempts as agreement.
    #[test]
    fn one_machine_agreeing_twice_is_not_two_machines_agreeing() {
        let (asking, _) = both_sides();
        let refused = asking
            .agreed_at(Side::TheOneAsking)
            .agreed_at(Side::TheOneAsking)
            .agreed(a_moment())
            .unwrap_err();
        assert_eq!(refused, NotPaired::OnlyOneSideAgreed);
    }

    /// Nobody agreeing is the same refusal, said the same way.
    #[test]
    fn nobody_agreeing_pairs_nothing() {
        let (asking, _) = both_sides();
        assert_eq!(
            asking.agreed(a_moment()).unwrap_err(),
            NotPaired::OnlyOneSideAgreed
        );
    }

    /// **An asking machine that heard nothing back has nothing to agree
    /// with**, however many times both flags are set on it: without the asked
    /// machine's offer there is no key, and a pairing without a key is not
    /// made.
    #[test]
    fn an_asking_machine_that_heard_nothing_back_pairs_nothing() {
        let (proposal, keying) = a_proposal();
        let refused = Deliberating::asking(proposal, keying)
            .unwrap()
            .agreed_at(Side::TheOneAsking)
            .agreed_at(Side::TheOneAsked)
            .agreed(a_moment())
            .unwrap_err();
        assert_eq!(refused, NotPaired::OnlyOneSideAgreed);
    }

    /// **A keying that is not the one the proposal carries is refused**, and
    /// so is an answer on the asked side, and so is an offer reflected back.
    #[test]
    fn the_wrong_keying_or_a_reflected_offer_is_refused_before_any_code_is_shown() {
        let (proposal, _) = a_proposal();
        let refused = Deliberating::asking(proposal.clone(), Keying::fresh().unwrap()).unwrap_err();
        assert_eq!(refused, NotPaired::NotTheOfferMade);

        let asked = Deliberating::asked(proposal.clone(), Keying::fresh().unwrap());
        let refused = asked
            .answered_with(Keying::fresh().unwrap().offer().clone())
            .unwrap_err();
        assert_eq!(refused, NotPaired::NotTheOfferMade);

        let (proposal, keying) = a_proposal();
        let reflected = proposal.offered().clone();
        let refused = Deliberating::asking(proposal, keying)
            .unwrap()
            .answered_with(reflected)
            .unwrap_err();
        assert_eq!(refused, NotPaired::NotTheOfferMade);
    }

    /// **The code is the same on both machines**, is six digits, and is
    /// nothing until the other machine's offer has arrived.
    #[test]
    fn the_code_is_the_same_on_both_machines_and_nothing_before_the_answer() {
        let (proposal, keying) = a_proposal();
        let waiting = Deliberating::asking(proposal, keying).unwrap();
        assert!(waiting.code().is_none());

        let (asking, asked) = both_sides();
        let code = asking.code().unwrap();
        assert_eq!(Some(code.clone()), asked.code());
        assert_eq!(code.digits().len(), 6);
    }

    /// **A machine in between hands each side an offer of its own, and the two
    /// codes differ** — which is what the two people see, and the only thing
    /// they have to see.
    #[test]
    fn a_machine_in_between_shows_the_two_people_different_codes() {
        let (proposal, at_asking) = a_proposal();
        let in_between = Keying::fresh().unwrap();
        // What reaches the asked machine names the interceptor's offer.
        let altered = Proposal::checked(
            one(),
            another(),
            proposal.may(),
            proposal.lasting(),
            in_between.offer().clone(),
        )
        .unwrap();
        let asked = Deliberating::asked(altered, Keying::fresh().unwrap());
        // And what comes back to the asking machine is the interceptor's too.
        let asking = Deliberating::asking(proposal, at_asking)
            .unwrap()
            .answered_with(Keying::fresh().unwrap().offer().clone())
            .unwrap();
        assert_ne!(asking.code(), asked.code());
    }

    /// A machine cannot pair with itself: it is not a pairing, and it would
    /// make every later question about *the other machine* ambiguous.
    #[test]
    fn a_machine_cannot_pair_with_itself() {
        let offer = Keying::fresh().unwrap().offer().clone();
        let refused = Proposal::checked(
            one(),
            one(),
            &[MayAskIts::Models],
            Duration::from_secs(60),
            offer,
        )
        .unwrap_err();
        assert_eq!(refused, NotPaired::WithItself);
    }

    /// A pairing that permits nothing is a row in a list that does nothing, and
    /// confuses everybody who reads it.
    #[test]
    fn a_pairing_that_would_permit_nothing_is_not_proposed() {
        let offer = Keying::fresh().unwrap().offer().clone();
        let refused =
            Proposal::checked(one(), another(), &[], Duration::from_secs(60), offer).unwrap_err();
        assert_eq!(refused, NotPaired::NothingAsked);
    }

    /// **It ends**, and there is no way to ask for one that does not: zero is
    /// refused at one end and a month at the other.
    #[test]
    fn a_pairing_lasting_no_time_or_forever_is_refused() {
        let offer = Keying::fresh().unwrap().offer().clone();
        assert_eq!(
            Proposal::checked(
                one(),
                another(),
                &[MayAskIts::Models],
                Duration::ZERO,
                offer.clone()
            )
            .unwrap_err(),
            NotPaired::NoTime
        );
        assert_eq!(
            Proposal::checked(
                one(),
                another(),
                &[MayAskIts::Models],
                AT_MOST + Duration::from_secs(1),
                offer.clone()
            )
            .unwrap_err(),
            NotPaired::TooLong
        );
        assert!(
            Proposal::checked(one(), another(), &[MayAskIts::Models], AT_MOST, offer).is_ok(),
            "the longest a pairing may be asked for is refused"
        );
    }

    /// What is asked for is a list without repeats, so the sentence a person
    /// reads does not say the same thing twice.
    #[test]
    fn what_is_asked_for_is_said_once_each() {
        let proposal = Proposal::checked(
            one(),
            another(),
            &[MayAskIts::Models, MayAskIts::Models, MayAskIts::Workspace],
            Duration::from_secs(60),
            Keying::fresh().unwrap().offer().clone(),
        )
        .unwrap();
        assert_eq!(
            proposal.may(),
            [MayAskIts::Models, MayAskIts::Workspace].as_slice()
        );
    }

    /// **How long it lasts is stated where it is made**, rather than being a
    /// constant somebody has to go and look up.
    #[test]
    fn how_long_it_would_last_is_part_of_what_is_shown() {
        assert_eq!(a_proposal().0.lasting(), Duration::from_secs(86_400));
    }
}
