//! The asking end: a verb an agent on this machine asks for on a paired
//! machine, put there with a proof, under this machine's indicator.
//!
//! # Made only from a pairing, and dialled only where discovery measured
//!
//! [`Crossing::to`] asks this machine's own pairings whether a pairing with
//! the machine named stands now, and there is no other constructor — a
//! machine merely discovered, one whose pairing ended and one whose pairing
//! was revoked are refused alike, and the refusal does not say which, because
//! telling a caller which kind of not-paired a machine is would be telling it
//! how to become paired. The address is [`alo_nearby::Found::where_it_answers`]
//! and nothing a person typed.
//!
//! **No arm of the pairing is asked for.** ADR 0003: the pairing gives
//! standing, not permission. `alo_nearby::MayAskIts` has no arm for a verb
//! and never will; what a verb may do on the other machine is what that
//! machine's person granted there, and it is asked there.
//!
//! # And it is egress, shown first
//!
//! The verb and its arguments are this machine's data leaving it, so the
//! door reaches the wire holding an [`alo_egress::Departing`] that only the
//! indicator makes, after the egress rule in force said yes — under the
//! asking agent's name, as [`alo_egress::Why::Sending`] to the machine by the
//! name the person gave it. What comes back holds that departure, answered
//! or not, so the turn on this machine can write down that the verb left
//! whether or not anything came back: a record of only the answered ones
//! would report a quieter day than the machine had. This crate hands the
//! departure back and writes nothing down, exactly as `alo-asking` does.

use std::net::SocketAddr;
use std::time::SystemTime;

use alo_capability::{Given, Grantee};
use alo_egress::{Departing, Destination, EgressPolicy, Indicator, Leaving, Why};
use alo_nearby::{Found, MachineId, Pairing, Pairings, Proof};

use crate::answered::Answered;
use crate::carried::Carried;
use crate::dialling;
use crate::door::AtTheDoor;
use crate::receiving::{THE_CHANGE_PATH, THE_READ_PATH};
use crate::refusing::{Left, NotCrossed, WentBack};

/// A paired machine, as a place a verb may be put.
///
/// Made only by [`to`](Self::to). Holding one is evidence that two people
/// agreed, that their agreement had not ended at the moment it was made, and
/// that this machine holds the key to prove a verb is its. Consumed by the
/// one verb it puts: one crossing is one departure on the indicator.
#[derive(Debug)]
pub struct Crossing<'a> {
    /// This machine, which the proof names as the sender.
    here: MachineId,
    /// The pairing, which holds the key and names the machine the proof is
    /// for.
    pairing: Pairing,
    /// Where discovery measured that machine answers.
    to: SocketAddr,
    /// The name the person here gave it, for the indicator and the record.
    called: &'a str,
    /// The agent asking, whose name the departure is under.
    agent: Grantee,
}

/// A verb that crossed, and what came back.
///
/// Holds the departure until [`ended`](Self::ended) spends it, so the line
/// on the indicator stays up until the caller has written the departure
/// down — the order `alo_turn::Turning::asking` keeps for a question.
#[derive(Debug)]
pub struct Crossed {
    /// The departure the indicator is showing.
    departing: Departing,
    /// What the other machine answered.
    answered: Answered,
}

impl<'a> Crossing<'a> {
    /// The machine discovery found, if a pairing with it stands at `now`.
    ///
    /// `here` is this machine; `called` is what the person here called the
    /// other machine, which decides nothing and is what they read; `agent` is
    /// the agent asking.
    ///
    /// # Errors
    ///
    /// [`NotCrossed::NotPairedWithIt`] when no pairing with that machine
    /// stands at `now`.
    pub fn to(
        pairings: &Pairings,
        here: &MachineId,
        found: &Found,
        called: &'a str,
        agent: &Grantee,
        now: SystemTime,
    ) -> Result<Self, NotCrossed> {
        let Some(pairing) = pairings.with(&found.machine, now) else {
            return Err(NotCrossed::NotPairedWithIt);
        };
        Ok(Self {
            here: here.clone(),
            pairing: pairing.clone(),
            to: found.where_it_answers(),
            called,
            agent: agent.clone(),
        })
    }

    /// Where this would connect, which is where discovery measured.
    #[must_use]
    pub const fn where_it_would_connect(&self) -> SocketAddr {
        self.to
    }

    /// Put a read to the other machine, which answers inside its turn.
    ///
    /// # Errors
    ///
    /// [`NotCrossed`]: nothing left for the first three arms, and for the
    /// fourth the verb left and this is what came back instead of an answer,
    /// holding the departure for the record.
    pub fn reading(
        self,
        verb: &str,
        given: &[(&str, Given)],
        policy: &EgressPolicy,
        indicator: &mut Indicator,
        now: SystemTime,
    ) -> Result<Crossed, NotCrossed> {
        self.crossing(THE_READ_PATH, verb, given, policy, indicator, now)
    }

    /// Put a change to the other machine, which waits for the person there.
    ///
    /// What comes back is the number it waits under, on that machine; the
    /// person there approves or declines it on their own surface, and
    /// nothing about their answer reaches here.
    ///
    /// # Errors
    ///
    /// As [`reading`](Self::reading).
    pub fn changing(
        self,
        verb: &str,
        given: &[(&str, Given)],
        policy: &EgressPolicy,
        indicator: &mut Indicator,
        now: SystemTime,
    ) -> Result<Crossed, NotCrossed> {
        self.crossing(THE_CHANGE_PATH, verb, given, policy, indicator, now)
    }

    /// The crossing itself: the bytes, the proof over them, the indicator,
    /// the dial, and what came back.
    fn crossing(
        self,
        path: &str,
        verb: &str,
        given: &[(&str, Given)],
        policy: &EgressPolicy,
        indicator: &mut Indicator,
        now: SystemTime,
    ) -> Result<Crossed, NotCrossed> {
        let body = Carried::of(verb, given).said();
        let proof = Proof::made(&self.pairing, &self.here, body.as_bytes(), now).said();
        let destination = Destination::paired(self.called).map_err(NotCrossed::CannotBeShown)?;
        let leaving = Leaving::because(&self.agent, Why::Sending, destination);
        let departing = indicator
            .beginning(policy, leaving, now)
            .map_err(NotCrossed::HeldBack)?;
        let went_back = match dialling::put(self.to, path, &proof, &body) {
            Ok(answered) if answered.status == 200 => match Answered::read(&answered.body) {
                Ok(answered) => {
                    return Ok(Crossed {
                        departing,
                        answered,
                    });
                }
                Err(_) => WentBack::NotAnAnswer(answered.body),
            },
            Ok(answered) => AtTheDoor::off_the_wire(&answered.body)
                .map_or(WentBack::NotAnAnswer(answered.body), WentBack::AtTheDoor),
            Err(went_back) => went_back,
        };
        Err(NotCrossed::Left(Box::new(Left::because(
            departing, went_back,
        ))))
    }
}

impl Crossed {
    /// The departure, for the record: what went, under whose authority, to
    /// where.
    #[must_use]
    pub const fn departing(&self) -> &Departing {
        &self.departing
    }

    /// What the other machine answered.
    #[must_use]
    pub const fn answered(&self) -> &Answered {
        &self.answered
    }

    /// Take the line off the indicator, and keep the answer.
    ///
    /// Consumes the departure, so one crossing ends exactly one line.
    #[must_use]
    pub fn ended(self, indicator: &mut Indicator) -> Answered {
        indicator.ended(self.departing);
        self.answered
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use std::time::Duration;

    use alo_capability::{Given, Grantee};
    use alo_egress::{EgressPolicy, Indicator};
    use alo_nearby::{Found, Pairings};

    use super::Crossing;
    use crate::refusing::NotCrossed;
    use crate::testing::{a_moment, paired_between, reception, studio};

    /// The studio, as discovery found it, at an address nothing listens on.
    fn the_studio_found() -> Found {
        Found::seen(studio(), 1, std::net::Ipv4Addr::LOCALHOST.into())
    }

    /// Reception's pairings, holding its row about the studio.
    fn at_reception() -> Pairings {
        let (on_reception, _) = paired_between(reception(), studio());
        let mut pairings = Pairings::none();
        pairings.keep(on_reception);
        pairings
    }

    /// **An unpaired machine is not a place a verb may be put**, however
    /// present it is: one merely found, one whose pairing ended, and one
    /// whose pairing was undone, all the same refusal.
    #[test]
    fn an_unpaired_machine_is_not_a_place_a_verb_may_be_put() {
        let files = Grantee::named("@files");
        assert!(matches!(
            Crossing::to(
                &Pairings::none(),
                &reception(),
                &the_studio_found(),
                "the studio",
                &files,
                a_moment()
            )
            .unwrap_err(),
            NotCrossed::NotPairedWithIt
        ));
        let a_week_later = a_moment() + Duration::from_secs(7 * 86_400);
        assert!(matches!(
            Crossing::to(
                &at_reception(),
                &reception(),
                &the_studio_found(),
                "the studio",
                &files,
                a_week_later
            )
            .unwrap_err(),
            NotCrossed::NotPairedWithIt
        ));
        let mut revoked = at_reception();
        assert!(revoked.revoke(&studio()));
        assert!(matches!(
            Crossing::to(
                &revoked,
                &reception(),
                &the_studio_found(),
                "the studio",
                &files,
                a_moment()
            )
            .unwrap_err(),
            NotCrossed::NotPairedWithIt
        ));
    }

    /// **A rule that says nothing leaves holds the verb back**, and nothing
    /// is dialled: the address nothing listens on is never reached, and the
    /// indicator stays quiet.
    #[test]
    fn a_rule_that_says_nothing_leaves_holds_the_verb_back_before_anything_is_dialled() {
        let crossing = Crossing::to(
            &at_reception(),
            &reception(),
            &the_studio_found(),
            "the studio",
            &Grantee::named("@files"),
            a_moment(),
        )
        .unwrap();
        assert_eq!(
            crossing.where_it_would_connect(),
            the_studio_found().where_it_answers()
        );
        let mut indicator = Indicator::default();
        let held = crossing
            .reading(
                "list_folder",
                &[("folder", Given::text("/a"))],
                &EgressPolicy::NothingLeaves,
                &mut indicator,
                a_moment(),
            )
            .unwrap_err();
        assert!(matches!(held, NotCrossed::HeldBack(_)), "{held:?}");
        assert!(!held.something_left());
        assert!(indicator.is_quiet());
    }

    /// **A verb that left and met nobody comes back holding its departure**,
    /// shown on the indicator until it is ended.
    #[test]
    fn a_verb_that_left_and_met_nobody_comes_back_holding_its_departure() {
        let crossing = Crossing::to(
            &at_reception(),
            &reception(),
            &the_studio_found(),
            "the studio",
            &Grantee::named("@files"),
            a_moment(),
        )
        .unwrap();
        let mut indicator = Indicator::default();
        let left = crossing
            .reading(
                "list_folder",
                &[("folder", Given::text("/a"))],
                &EgressPolicy::InTheBuilding,
                &mut indicator,
                a_moment(),
            )
            .unwrap_err();
        assert!(left.something_left());
        assert_eq!(indicator.showing().len(), 1);
        let NotCrossed::Left(left) = left else {
            unreachable!()
        };
        assert_eq!(left.departing().agent(), &Grantee::named("@files"));
        assert!(matches!(
            left.ended(&mut indicator),
            crate::refusing::WentBack::TheNetwork(_)
        ));
        assert!(indicator.is_quiet());
    }
}
