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
//! # What is not in here
//!
//! No cryptography, and no claim of any. What this file decides is **who
//! agreed**. Proving that the machine that agreed is the machine that later
//! asks is a question for the connection between them, is not built, and is not
//! implied anywhere in this crate — `docs/quirks.md` is where that will be
//! recorded when a connection exists to record it about.
//!
//! Nothing here reads the clock either. The moment a pairing starts is passed
//! in, the same way `alo_capability::Grant` takes it, so that what a pairing
//! does at a moment can be asked about a moment that is not now.

use std::time::Duration;

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
/// it, and shown to both people before either says yes.
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
}

impl Proposal {
    /// One machine's person asking another's, checked before anybody is shown
    /// anything.
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

/// A proposal that is waiting for two people.
///
/// Deliberately not `Copy` and deliberately consuming: [`agreed`](Self::agreed)
/// takes the deliberation, so a pairing is made from it once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deliberating {
    /// What is being proposed.
    proposal: Proposal,
    /// Whether the person at the asking machine has said yes.
    asking_agreed: bool,
    /// Whether the person at the asked machine has said yes.
    asked_agreed: bool,
}

impl Deliberating {
    /// A proposal, with nobody having agreed to it yet.
    #[must_use]
    pub const fn of(proposal: Proposal) -> Self {
        Self {
            proposal,
            asking_agreed: false,
            asked_agreed: false,
        }
    }

    /// What is being proposed, for the surface that shows it to a person.
    #[must_use]
    pub const fn proposal(&self) -> &Proposal {
        &self.proposal
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

    /// The pairing, if both people agreed, as the machine on `kept_on`'s side
    /// keeps it.
    ///
    /// This is the only thing in this workspace that returns a
    /// [`Pairing`](crate::Pairing).
    ///
    /// **One agreement, two lists, each naming the other machine.** A pairing
    /// is between two machines and each keeps its own row about it — ADR
    /// 0003's *visible* is a list on each machine, and *revocable in one
    /// action* is either person's — so the row the asking machine keeps names
    /// the one it asked, and the row the asked machine keeps names the one
    /// that asked it. Until 2026-09-13 there was one row and it always named
    /// the asked machine, which was right on the machine that asked and, on
    /// the one that was asked, a pairing with itself: the first thing built on
    /// the asked side ([`crate::Origin`]) found that nothing it was asked by
    /// was paired, and the side is now named rather than assumed.
    ///
    /// # Errors
    ///
    /// [`NotPaired::OnlyOneSideAgreed`] when one machine's person has said yes
    /// and the other has not — including when the same person has said yes
    /// twice. The refusal names no side: what is true is that the machines have
    /// not agreed, and saying *they refused* about a person who has simply not
    /// answered yet would be a sentence the machine cannot support.
    pub fn agreed(self, kept_on: Side, at: std::time::SystemTime) -> Result<Pairing, NotPaired> {
        if !self.is_mutual() {
            return Err(NotPaired::OnlyOneSideAgreed);
        }
        let other = match kept_on {
            Side::TheOneAsking => self.proposal.asked,
            Side::TheOneAsked => self.proposal.asking,
        };
        Pairing::between(other, &self.proposal.may, at, self.proposal.lasting)
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

    /// A proposal for the tests that are not about proposing.
    fn a_proposal() -> Proposal {
        Proposal::checked(
            one(),
            another(),
            &[MayAskIts::Models],
            Duration::from_secs(86_400),
        )
        .unwrap()
    }

    /// A moment for the tests that are not about time.
    fn a_moment() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    /// **Both people say yes, and there is a pairing.**
    #[test]
    fn a_pairing_is_made_when_both_machines_people_agree() {
        let pairing = Deliberating::of(a_proposal())
            .agreed_at(Side::TheOneAsking)
            .agreed_at(Side::TheOneAsked)
            .agreed(Side::TheOneAsking, a_moment())
            .unwrap();
        assert_eq!(*pairing.with(), another());
        assert!(pairing.permits(MayAskIts::Models, a_moment()));
    }

    /// **One side alone leaves nothing paired** — either side, which is why the
    /// test is both.
    #[test]
    fn one_machines_person_agreeing_alone_pairs_nothing() {
        for alone in [Side::TheOneAsking, Side::TheOneAsked] {
            let refused = Deliberating::of(a_proposal())
                .agreed_at(alone)
                .agreed(Side::TheOneAsking, a_moment())
                .unwrap_err();
            assert_eq!(refused, NotPaired::OnlyOneSideAgreed, "{alone:?}");
        }
    }

    /// **And the same side agreeing twice leaves nothing paired**, which is the
    /// shape a well-meaning convenience takes: a machine that asks, hears
    /// nothing, asks again, and counts its own two attempts as agreement.
    #[test]
    fn one_machine_agreeing_twice_is_not_two_machines_agreeing() {
        let refused = Deliberating::of(a_proposal())
            .agreed_at(Side::TheOneAsking)
            .agreed_at(Side::TheOneAsking)
            .agreed(Side::TheOneAsking, a_moment())
            .unwrap_err();
        assert_eq!(refused, NotPaired::OnlyOneSideAgreed);
    }

    /// Nobody agreeing is the same refusal, said the same way.
    #[test]
    fn nobody_agreeing_pairs_nothing() {
        let refused = Deliberating::of(a_proposal())
            .agreed(Side::TheOneAsking, a_moment())
            .unwrap_err();
        assert_eq!(refused, NotPaired::OnlyOneSideAgreed);
    }

    /// A machine cannot pair with itself: it is not a pairing, and it would
    /// make every later question about *the other machine* ambiguous.
    #[test]
    fn a_machine_cannot_pair_with_itself() {
        let refused =
            Proposal::checked(one(), one(), &[MayAskIts::Models], Duration::from_secs(60))
                .unwrap_err();
        assert_eq!(refused, NotPaired::WithItself);
    }

    /// A pairing that permits nothing is a row in a list that does nothing, and
    /// confuses everybody who reads it.
    #[test]
    fn a_pairing_that_would_permit_nothing_is_not_proposed() {
        let refused =
            Proposal::checked(one(), another(), &[], Duration::from_secs(60)).unwrap_err();
        assert_eq!(refused, NotPaired::NothingAsked);
    }

    /// **It ends**, and there is no way to ask for one that does not: zero is
    /// refused at one end and a month at the other.
    #[test]
    fn a_pairing_lasting_no_time_or_forever_is_refused() {
        assert_eq!(
            Proposal::checked(one(), another(), &[MayAskIts::Models], Duration::ZERO).unwrap_err(),
            NotPaired::NoTime
        );
        assert_eq!(
            Proposal::checked(
                one(),
                another(),
                &[MayAskIts::Models],
                AT_MOST + Duration::from_secs(1)
            )
            .unwrap_err(),
            NotPaired::TooLong
        );
        assert!(
            Proposal::checked(one(), another(), &[MayAskIts::Models], AT_MOST).is_ok(),
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
        assert_eq!(a_proposal().lasting(), Duration::from_secs(86_400));
    }

    /// **Each machine keeps a row naming the other one.** The same agreement,
    /// kept on the asked machine, names the machine that asked — which is what
    /// lets the asked machine later answer whether something arriving from
    /// there is from a machine it is paired with.
    #[test]
    fn the_same_agreement_names_the_other_machine_on_each_side() {
        let on_the_asking = Deliberating::of(a_proposal())
            .agreed_at(Side::TheOneAsking)
            .agreed_at(Side::TheOneAsked)
            .agreed(Side::TheOneAsking, a_moment())
            .unwrap();
        let on_the_asked = Deliberating::of(a_proposal())
            .agreed_at(Side::TheOneAsking)
            .agreed_at(Side::TheOneAsked)
            .agreed(Side::TheOneAsked, a_moment())
            .unwrap();

        assert_eq!(on_the_asking.with(), a_proposal().asked());
        assert_eq!(on_the_asked.with(), a_proposal().asking());
        assert_ne!(on_the_asking.with(), on_the_asked.with());
        // And everything else about the two rows is the same agreement.
        assert_eq!(on_the_asking.may(), on_the_asked.may());
        assert_eq!(on_the_asking.made(), on_the_asked.made());
        assert_eq!(on_the_asking.ends(), on_the_asked.ends());
    }
}
