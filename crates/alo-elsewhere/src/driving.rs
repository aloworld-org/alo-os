//! Whether the person's agent may drive one of their machines.
//!
//! [ADR 0079](../../../docs/decisions/0079-a-person-may-hand-their-agent-a-whole-machine.md)
//! settles this, and its terms are the shape of this file rather than a comment
//! above it:
//!
//! - **Off until turned on.** [`Driving::none_given`] is the default, and adding
//!   a machine grants nothing — [`crate::TheMachines`] and this type share no
//!   state at all.
//! - **One named machine at a time.** There is no constructor here taking a set
//!   of machines, and none taking *all* of them. Making the widest grant in the
//!   product the cheapest one to make is the mistake that record rejects.
//! - **Expiring.** The end is not an `Option`. ADR 0001 §3: there is no grant
//!   that outlives the reason it was made, and this is the last grant that
//!   should be the exception.
//! - **Revocable in one action**, by [`Driving::revoke`].
//!
//! # What this grant cannot do, which is the point
//!
//! **It cannot enumerate what it permits.** Every other grant in this product is
//! a list — ADR 0001 §3, *a list, not a rule*. This one is a list of length one
//! whose single entry is a machine, and the verbs behind it are unbounded,
//! because a pointer and a keyboard reach everything that machine's interface
//! offers.
//!
//! So there is no `verbs()` on this type and there will not be one. A method
//! returning a list of what an agent may do on a driven machine would be a
//! fiction that reads like a fact, and ADR 0079 rejects exactly that: *a grant
//! that lies about its own scope is worse than an honest wide one.*
//!
//! What exists instead is [`WHAT_IT_CANNOT_ENUMERATE`], the sentence the person
//! reads at the moment they grant it.

use std::time::SystemTime;

use alo_nearby::MachineId;

use crate::refusing::NotElsewhere;
use crate::words;

/// The key of the sentence a person reads where they grant this.
///
/// Named here rather than only in [`crate::words`] because ADR 0079 makes
/// showing it a term of the decision, not a presentation choice — a caller
/// granting this without saying what it is has not implemented the record.
pub const WHAT_IT_CANNOT_ENUMERATE: &str = words::WHAT_IT_CANNOT_ENUMERATE;

/// The person's agent may drive one named machine, until a moment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MayDrive {
    /// Which machine. One, never a set.
    which: MachineId,
    /// When the person gave it, so the list can say what they did and when.
    given: SystemTime,
    /// When it stops. **Not optional**, and that is ADR 0001 §3 rather than a
    /// convenience.
    ends: SystemTime,
}

impl MayDrive {
    /// A grant over one machine, ending at a moment.
    ///
    /// # Errors
    ///
    /// [`NotElsewhere::EndsBeforeItBegins`] if the end is not after the
    /// beginning. A grant that has already ended is not a safe no-op to accept:
    /// it would sit in the person's list reading like permission they gave,
    /// while permitting nothing, and they would not know which of those two
    /// things they were looking at.
    pub fn given_until(
        which: MachineId,
        given: SystemTime,
        ends: SystemTime,
    ) -> Result<Self, NotElsewhere> {
        if ends <= given {
            return Err(NotElsewhere::EndsBeforeItBegins);
        }
        Ok(Self { which, given, ends })
    }

    /// Which machine this is over.
    #[must_use]
    pub const fn which(&self) -> &MachineId {
        &self.which
    }

    /// When the person gave it.
    #[must_use]
    pub const fn given(&self) -> SystemTime {
        self.given
    }

    /// When it stops.
    #[must_use]
    pub const fn ends(&self) -> SystemTime {
        self.ends
    }

    /// Whether it still stands at this moment.
    #[must_use]
    pub fn stands_at(&self, now: SystemTime) -> bool {
        now < self.ends
    }
}

/// Every machine the person's agent may drive.
///
/// Separate from [`crate::TheMachines`] on purpose: being on the person's list
/// of machines permits nothing, and the two are never read from one place.
#[derive(Debug, Default, Clone)]
pub struct Driving {
    /// One grant per machine, in the order they were given.
    given: Vec<MayDrive>,
}

impl Driving {
    /// Nothing granted, which is what a machine arrives at (ADR 0025).
    #[must_use]
    pub const fn none_given() -> Self {
        Self { given: Vec::new() }
    }

    /// Give the agent one machine.
    ///
    /// # Errors
    ///
    /// [`NotElsewhere::AlreadyDriving`] if a grant over that machine already
    /// exists, whether or not it still stands.
    ///
    /// **Refused rather than replaced**, and this is a decision. A person who
    /// believes they are granting something is, on a silent replace, extending
    /// the widest grant in the product — and extending it by an amount nothing
    /// told them. Renewing is [`Self::extend_until`], which is named for what
    /// it does.
    pub fn give(&mut self, grant: MayDrive) -> Result<(), NotElsewhere> {
        if self.holds(grant.which()) {
            return Err(NotElsewhere::AlreadyDriving);
        }
        self.given.push(grant);
        Ok(())
    }

    /// Move an existing grant's end later, and say what it was before.
    ///
    /// Answers `None` when there is no grant over that machine — extending
    /// nothing does not quietly become granting something.
    ///
    /// # Errors
    ///
    /// [`NotElsewhere::EndsBeforeItBegins`] if the new end is not after the
    /// moment the grant was given.
    pub fn extend_until(
        &mut self,
        which: &MachineId,
        ends: SystemTime,
    ) -> Result<Option<SystemTime>, NotElsewhere> {
        let Some(grant) = self.given.iter_mut().find(|grant| grant.which() == which) else {
            return Ok(None);
        };
        if ends <= grant.given {
            return Err(NotElsewhere::EndsBeforeItBegins);
        }
        let was = grant.ends;
        grant.ends = ends;
        Ok(Some(was))
    }

    /// Take it back, in one action, and say whether there was one to take.
    pub fn revoke(&mut self, which: &MachineId) -> bool {
        let was = self.given.len();
        self.given.retain(|grant| grant.which() != which);
        self.given.len() != was
    }

    /// Take every grant back, and say how many there were.
    ///
    /// One action for *stop everything*, because a person wanting that wants it
    /// now rather than once per machine.
    pub fn revoke_everything(&mut self) -> usize {
        let how_many = self.given.len();
        self.given.clear();
        how_many
    }

    /// **The one question the rest of the system asks.**
    ///
    /// False for a machine with no grant, false after the end, and false after
    /// a revocation — and it takes the moment rather than reading a clock, so
    /// the answer can be tested at any moment rather than only at this one.
    #[must_use]
    pub fn may_drive(&self, which: &MachineId, now: SystemTime) -> bool {
        self.given
            .iter()
            .any(|grant| grant.which() == which && grant.stands_at(now))
    }

    /// Whether a grant over this machine exists, standing or ended.
    #[must_use]
    pub fn holds(&self, which: &MachineId) -> bool {
        self.given.iter().any(|grant| grant.which() == which)
    }

    /// Every grant, each with whether it still stands at this moment.
    ///
    /// Pairs, for the reason [`crate::TheMachines::each`] gives: a person
    /// reading what they have granted must not be shown a list that quietly
    /// leaves out the ones that have ended, because *nothing is listed* and
    /// *nothing stands* are different facts and only one of them is reassuring.
    pub fn each(&self, now: SystemTime) -> impl Iterator<Item = (&MayDrive, bool)> + '_ {
        self.given
            .iter()
            .map(move |grant| (grant, grant.stands_at(now)))
    }

    /// How many grants exist, standing or ended.
    #[must_use]
    pub fn how_many(&self) -> usize {
        self.given.len()
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn identity(last: char) -> MachineId {
        let mut said = "0123456789abcdef0123456789abcde".to_owned();
        said.push(last);
        MachineId::read(&said).expect("thirty-two hexadecimal characters")
    }

    fn at(seconds: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)
    }

    fn an_hour_from(seconds: u64) -> MayDrive {
        MayDrive::given_until(identity('a'), at(seconds), at(seconds + 3600)).expect("an hour")
    }

    /// **Off until turned on.** ADR 0025: nobody chooses for the person, and
    /// arriving able to be driven by an agent is not something anyone asked for.
    #[test]
    fn nothing_is_granted_to_begin_with() {
        let driving = Driving::none_given();
        assert_eq!(driving.how_many(), 0);
        assert!(!driving.may_drive(&identity('a'), at(1)));
    }

    /// There is no grant that outlives the reason it was made, so a grant with
    /// no end cannot be spelled and one that ends before it begins is refused.
    #[test]
    fn a_grant_that_ends_before_it_begins_is_refused() {
        assert_eq!(
            MayDrive::given_until(identity('a'), at(100), at(100)),
            Err(NotElsewhere::EndsBeforeItBegins)
        );
        assert_eq!(
            MayDrive::given_until(identity('a'), at(100), at(99)),
            Err(NotElsewhere::EndsBeforeItBegins)
        );
        assert!(MayDrive::given_until(identity('a'), at(100), at(101)).is_ok());
    }

    /// It stops on its own, without anybody revoking it.
    #[test]
    fn a_grant_stops_at_its_end_without_anybody_acting() {
        let mut driving = Driving::none_given();
        driving.give(an_hour_from(1000)).expect("given");

        assert!(driving.may_drive(&identity('a'), at(1000)));
        assert!(driving.may_drive(&identity('a'), at(4599)));
        assert!(
            !driving.may_drive(&identity('a'), at(4600)),
            "it still permitted at its own end"
        );
        assert!(!driving.may_drive(&identity('a'), at(9999)));
    }

    /// **Revocable in one action**, and it stops permitting at once.
    #[test]
    fn revoking_stops_it_immediately() {
        let mut driving = Driving::none_given();
        driving.give(an_hour_from(1000)).expect("given");
        assert!(driving.may_drive(&identity('a'), at(1500)));

        assert!(driving.revoke(&identity('a')));
        assert!(!driving.may_drive(&identity('a'), at(1500)));
        assert!(!driving.revoke(&identity('a')), "there was nothing left");
    }

    /// **One machine at a time.** A grant over one says nothing about another.
    #[test]
    fn a_grant_over_one_machine_is_not_a_grant_over_another() {
        let mut driving = Driving::none_given();
        driving.give(an_hour_from(1000)).expect("given");
        assert!(driving.may_drive(&identity('a'), at(1500)));
        assert!(!driving.may_drive(&identity('b'), at(1500)));
    }

    /// **Extending is named for what it does.** A silent replace would let a
    /// person believe they were granting when they were widening.
    #[test]
    fn granting_twice_is_refused_and_extending_is_its_own_act() {
        let mut driving = Driving::none_given();
        driving.give(an_hour_from(1000)).expect("given");
        assert_eq!(
            driving.give(an_hour_from(2000)),
            Err(NotElsewhere::AlreadyDriving)
        );
        assert_eq!(driving.how_many(), 1, "a second row was made");

        let was = driving
            .extend_until(&identity('a'), at(9000))
            .expect("a later end")
            .expect("there was a grant to extend");
        assert_eq!(was, at(4600), "it did not say what the end had been");
        assert!(driving.may_drive(&identity('a'), at(8999)));
    }

    /// Extending nothing does not quietly become granting something.
    #[test]
    fn extending_a_machine_with_no_grant_grants_nothing() {
        let mut driving = Driving::none_given();
        assert_eq!(
            driving
                .extend_until(&identity('a'), at(9000))
                .expect("asked"),
            None
        );
        assert_eq!(driving.how_many(), 0);
        assert!(!driving.may_drive(&identity('a'), at(1)));
    }

    /// A person reading what they granted sees the ended ones too, because
    /// *nothing is listed* and *nothing stands* are different facts.
    #[test]
    fn an_ended_grant_is_still_listed_and_says_it_has_ended() {
        let mut driving = Driving::none_given();
        driving.give(an_hour_from(1000)).expect("given");

        let listed: Vec<_> = driving.each(at(9999)).collect();
        assert_eq!(listed.len(), 1, "an ended grant was left out");
        let (_, stands) = listed.first().expect("one grant was just listed");
        assert!(!stands, "it was listed as still standing");
    }

    /// One action for *stop everything*, because somebody wanting that wants it
    /// now rather than once per machine.
    #[test]
    fn everything_can_be_taken_back_at_once() {
        let mut driving = Driving::none_given();
        driving.give(an_hour_from(1000)).expect("given");
        driving
            .give(MayDrive::given_until(identity('b'), at(1000), at(4600)).expect("an hour"))
            .expect("given");

        assert_eq!(driving.revoke_everything(), 2);
        assert_eq!(driving.how_many(), 0);
        assert!(!driving.may_drive(&identity('a'), at(1500)));
        assert!(!driving.may_drive(&identity('b'), at(1500)));
    }

    /// **The sentence ADR 0079 makes a term of the decision is reachable from
    /// the type that needs it**, so a caller granting this cannot fail to find
    /// what it must say.
    #[test]
    fn the_sentence_about_what_it_cannot_list_is_named_here() {
        assert_eq!(
            WHAT_IT_CANNOT_ENUMERATE,
            "elsewhere.driving.what-it-cannot-enumerate"
        );
    }
}
