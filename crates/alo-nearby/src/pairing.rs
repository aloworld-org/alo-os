//! One pairing, and the list of them a person can see and undo.
//!
//! ADR 0003: *pairings behave like grants — enumerated, visible, revocable in
//! one action, and expiring.* Each of those four words is a property enforced
//! here rather than checked by whoever remembers to:
//!
//! - **enumerated** — what a pairing permits is a list of
//!   [`MayAskIts`](crate::MayAskIts), each with a sentence a person reads;
//! - **visible** — [`Pairings::every`] is the whole list, in the order it was
//!   made, with nothing hidden from it;
//! - **revocable in one action** — [`Pairings::revoke`] takes effect at the
//!   next question, and there is no second step;
//! - **expiring** — a pairing is made with a moment and a duration, there is no
//!   variant meaning *for ever*, and [`Pairing::permits`] asks the time it is
//!   given rather than the clock.
//!
//! # Being paired is not a fact about the network
//!
//! Nothing here can be reached from a [`Found`](crate::Found). A machine that
//! has been discovered, that shares this machine's network, that shares its
//! wireless password, or that this machine paired with last month is — in every
//! one of those four cases — a machine [`Pairings::permits`] answers `false`
//! about. That is ADR 0003's *being on the same network is not authority*, and
//! `a_pairing_is_made_by_two_people_and_by_nothing_else.rs` holds each case
//! under its own name.

use std::time::{Duration, SystemTime};

use crate::keying::PairingKey;
use crate::machine::MachineId;
use crate::permitting::MayAskIts;
use crate::words;

/// Why there is no pairing.
///
/// **No `Display`**, and so not a `std::error::Error`, for the reason
/// `alo_capability::GrantError` has none: the only road to words is
/// [`said`](NotPaired::said), which takes the strings the person in front of
/// the machine reads. A `Display` here would be an English sentence one
/// `to_string()` away from a settings panel whose author had no reason to think
/// about it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum NotPaired {
    /// One machine's person has agreed and the other's has not.
    ///
    /// It names no side on purpose. What is true is that the machines have not
    /// agreed; saying *they refused* about somebody who has not answered yet is
    /// a sentence the machine cannot support.
    OnlyOneSideAgreed,
    /// A machine proposing to pair with itself.
    WithItself,
    /// A pairing that would permit nothing at all.
    NothingAsked,
    /// A pairing lasting no time.
    NoTime,
    /// A pairing longer than [`crate::AT_MOST`].
    TooLong,
    /// A pairing whose end is past anything this machine can represent.
    NoEnd,
    /// This machine is not paired with the one that asked.
    ///
    /// The refusal on the **asked** side of ADR 0003, made by
    /// [`Proof::made`](crate::Proof::made)'s callers and by
    /// [`Proven::checked`](crate::Proven::checked): a machine that was merely
    /// discovered, one whose pairing has ended, and one whose pairing was
    /// revoked are all this, with no arm that distinguishes them, because
    /// telling a caller which kind of not-paired it is would be telling it how
    /// to become paired.
    NotWithThatMachine,
    /// The two machines could not agree a key (ADR 0031).
    ///
    /// Not reachable from an [`Offer`](crate::Offer) this crate read, and
    /// answered rather than unwrapped.
    NoKey,
    /// The offer this machine holds the private half of is not the one in
    /// the proposal, or the offer that came back is this machine's own.
    ///
    /// The second is what an attacker reflecting a machine's offer back at it
    /// looks like, and the first is a program that lost track of which
    /// keying went with which proposal. Both are refused before anybody is
    /// shown a code.
    NotTheOfferMade,
}

impl NotPaired {
    /// The sentence a person reads for this.
    #[must_use]
    pub const fn word(self) -> alo_strings::Word {
        match self {
            Self::OnlyOneSideAgreed => words::BOTH_MACHINES_HAVE_NOT_AGREED,
            Self::WithItself => words::A_MACHINE_CANNOT_PAIR_WITH_ITSELF,
            Self::NothingAsked => words::A_PAIRING_HAS_TO_PERMIT_SOMETHING,
            Self::NoTime | Self::NoEnd | Self::TooLong => words::A_PAIRING_HAS_TO_END,
            Self::NotWithThatMachine => words::NOT_PAIRED_WITH_THE_ONE_THAT_ASKED,
            Self::NoKey | Self::NotTheOfferMade => words::START_THE_PAIRING_AGAIN,
        }
    }

    /// This refusal, in the language the person reads.
    #[must_use]
    pub fn said(self, strings: &alo_strings::Strings) -> alo_strings::Said {
        strings.say(&self.word().key(), &alo_strings::Filling::nothing())
    }
}

/// A pairing two people made.
///
/// There is no public constructor. The only thing that returns one is
/// [`Deliberating::agreed`](crate::Deliberating::agreed), which refuses unless
/// both machines' people have said yes — so a `Pairing` in hand is evidence
/// that they did, and that this machine holds the key the two of them agreed
/// (ADR 0031). The key is on the row, so revoking the row is revoking the key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pairing {
    /// The other machine.
    with: MachineId,
    /// What it may ask this one for, enumerated and without repeats.
    may: Vec<MayAskIts>,
    /// When the two people agreed. Shown in the list, so a person can see what
    /// they did and when.
    made: SystemTime,
    /// When it stops. Not optional, on purpose.
    ends: SystemTime,
    /// The key both machines hold and nobody else does. Never shown, never
    /// written on any wire, and printed as nothing.
    key: PairingKey,
}

impl Pairing {
    /// Made from an agreement, which is the only caller this has.
    ///
    /// `pub(crate)` rather than `pub`: the check that both people agreed lives
    /// in `deliberating.rs`, and a public constructor here would be a road
    /// round it.
    ///
    /// # Errors
    ///
    /// [`NotPaired::NoEnd`] if the moment it would stop is past anything this
    /// machine can represent, which is a clock set to the end of time rather
    /// than anything a person did.
    pub(crate) fn between(
        with: MachineId,
        may: &[MayAskIts],
        at: SystemTime,
        lasting: Duration,
        key: PairingKey,
    ) -> Result<Self, NotPaired> {
        let ends = at.checked_add(lasting).ok_or(NotPaired::NoEnd)?;
        Ok(Self {
            with,
            may: may.to_vec(),
            made: at,
            ends,
            key,
        })
    }

    /// The key, for the file that makes proofs and the file that checks them.
    pub(crate) const fn key(&self) -> &PairingKey {
        &self.key
    }

    /// The other machine.
    #[must_use]
    pub const fn with(&self) -> &MachineId {
        &self.with
    }

    /// What it may ask this machine for.
    #[must_use]
    pub fn may(&self) -> &[MayAskIts] {
        &self.may
    }

    /// When the two people agreed.
    #[must_use]
    pub const fn made(&self) -> SystemTime {
        self.made
    }

    /// When it stops.
    #[must_use]
    pub const fn ends(&self) -> SystemTime {
        self.ends
    }

    /// Whether this pairing permits `what` at `now`.
    ///
    /// The moment is given rather than read, so that what a pairing does can be
    /// asked about a moment that is not this one — and so that nothing in this
    /// crate has an opinion about what time it is.
    #[must_use]
    pub fn permits(&self, what: MayAskIts, now: SystemTime) -> bool {
        now < self.ends && self.may.contains(&what)
    }
}

/// Every pairing this machine has, which is the list a person sees.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Pairings {
    /// In the order they were made, which is the order they are shown.
    made: Vec<Pairing>,
}

impl Pairings {
    /// No pairings at all, which is what a machine starts with and what it has
    /// again after the last one is revoked.
    #[must_use]
    pub const fn none() -> Self {
        Self { made: Vec::new() }
    }

    /// Keep a pairing two people made.
    ///
    /// A second pairing with a machine already paired with **replaces** the
    /// first rather than joining it. Two rows about one machine would be a list
    /// a person cannot act on: revoking the one they can see would leave the
    /// one they cannot.
    pub fn keep(&mut self, pairing: Pairing) {
        self.made.retain(|already| already.with != pairing.with);
        self.made.push(pairing);
    }

    /// Every pairing, in the order they were made.
    ///
    /// The whole list. Nothing is hidden from it, and there is no second list
    /// somewhere else — *visible* is a property of there being one of these.
    #[must_use]
    pub fn every(&self) -> &[Pairing] {
        &self.made
    }

    /// Undo a pairing, in one action.
    ///
    /// Answers whether there was one to undo, so a surface can say *that is
    /// already gone* rather than reporting a success about nothing. There is no
    /// second step, no confirmation owed to the other machine, and no way for
    /// the other machine to decline: a person revoking on their own machine is
    /// finished when this returns.
    pub fn revoke(&mut self, with: &MachineId) -> bool {
        let before = self.made.len();
        self.made.retain(|pairing| pairing.with != *with);
        self.made.len() != before
    }

    /// Whether a machine may ask this one for `what`, at `now`.
    ///
    /// **This is the only door.** A machine that was discovered, that shares
    /// this network, that shares its wireless password, or that this machine
    /// paired with last month reaches this function and is answered `false`,
    /// because none of those four things puts a pairing in this list.
    #[must_use]
    pub fn permits(&self, with: &MachineId, what: MayAskIts, now: SystemTime) -> bool {
        self.made
            .iter()
            .any(|pairing| pairing.with == *with && pairing.permits(what, now))
    }

    /// Whether a pairing with that machine stands at `now`, whatever it
    /// permits.
    ///
    /// The question the **asked** side of ADR 0003 puts to this list
    /// ([`crate::Origin::proven`]): not *may it ask for this*, which
    /// [`Pairings::permits`] answers, but *is it somebody this machine can
    /// name at all*. A verb it asks for is then decided by the grants this
    /// machine's person made to it, and by nothing on the pairing — which is
    /// why there is no [`MayAskIts`] arm for a verb and no arm is asked about
    /// here. The four cases above answer `false` here for the same reason they
    /// do there.
    #[must_use]
    pub fn paired_with(&self, with: &MachineId, now: SystemTime) -> bool {
        self.with(with, now).is_some()
    }

    /// The pairing with that machine, if one stands at `now`.
    ///
    /// The row itself, for whatever makes a proof to that machine or checks
    /// one from it (ADR 0031): the key is on the row, and a row that has ended
    /// or was revoked is not here to be borrowed from.
    #[must_use]
    pub fn with(&self, with: &MachineId, now: SystemTime) -> Option<&Pairing> {
        self.made
            .iter()
            .find(|pairing| pairing.with == *with && now < pairing.ends)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use std::time::{Duration, SystemTime};

    use super::{Pairing, Pairings};
    use crate::keying::PairingKey;
    use crate::machine::MachineId;
    use crate::permitting::MayAskIts;

    /// The machine every test here pairs with.
    fn another() -> MachineId {
        MachineId::read("aaaabbbbccccddddeeeeffff00001111").unwrap()
    }

    /// A machine nobody paired with.
    fn a_stranger() -> MachineId {
        MachineId::read("99998888777766665555444433332222").unwrap()
    }

    /// A moment to reason from.
    fn a_moment() -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
    }

    /// A pairing for a day.
    fn for_a_day() -> Pairing {
        Pairing::between(
            another(),
            &[MayAskIts::Models],
            a_moment(),
            Duration::from_secs(86_400),
            PairingKey::of([7; 32]),
        )
        .unwrap()
    }

    /// **It expires.** The same question, the same pairing, one second past the
    /// end, and the answer is no.
    #[test]
    fn a_pairing_stops_permitting_anything_when_it_ends() {
        let pairing = for_a_day();
        assert!(pairing.permits(MayAskIts::Models, a_moment()));
        assert!(pairing.permits(MayAskIts::Models, a_moment() + Duration::from_secs(86_399)));
        assert!(!pairing.permits(MayAskIts::Models, a_moment() + Duration::from_secs(86_400)));
        assert!(!pairing.permits(MayAskIts::Models, a_moment() + Duration::from_secs(999_999)));
    }

    /// What was not asked for is not permitted, which is what makes the list an
    /// enumeration rather than a label on a door that is open.
    #[test]
    fn a_pairing_permits_what_is_on_its_list_and_nothing_beside_it() {
        let pairing = for_a_day();
        assert!(pairing.permits(MayAskIts::Models, a_moment()));
        assert!(!pairing.permits(MayAskIts::Workspace, a_moment()));
    }

    /// **Revoked in one action, and it takes effect at the next question** —
    /// including one already being asked, since the answer is read from the
    /// list every time rather than cached anywhere.
    #[test]
    fn revoking_takes_effect_at_the_very_next_question() {
        let mut pairings = Pairings::none();
        pairings.keep(for_a_day());
        assert!(pairings.permits(&another(), MayAskIts::Models, a_moment()));

        assert!(pairings.revoke(&another()));

        assert!(!pairings.permits(&another(), MayAskIts::Models, a_moment()));
        assert!(pairings.every().is_empty());
    }

    /// Revoking something already gone says so, rather than reporting a success
    /// about nothing.
    #[test]
    fn revoking_a_pairing_that_is_not_there_says_so() {
        let mut pairings = Pairings::none();
        assert!(!pairings.revoke(&another()));
        pairings.keep(for_a_day());
        assert!(pairings.revoke(&another()));
        assert!(!pairings.revoke(&another()));
    }

    /// A machine nobody paired with is permitted nothing, which is the sentence
    /// ADR 0003 is made of.
    #[test]
    fn a_machine_nobody_paired_with_is_permitted_nothing() {
        let mut pairings = Pairings::none();
        pairings.keep(for_a_day());
        assert!(!pairings.permits(&a_stranger(), MayAskIts::Models, a_moment()));
    }

    /// Pairing again with a machine already paired with replaces the row rather
    /// than adding one, so revoking what a person can see does not leave
    /// something they cannot.
    #[test]
    fn pairing_again_replaces_the_row_rather_than_adding_a_second() {
        let mut pairings = Pairings::none();
        pairings.keep(for_a_day());
        pairings.keep(
            Pairing::between(
                another(),
                &[MayAskIts::Models, MayAskIts::Workspace],
                a_moment(),
                Duration::from_secs(60),
                PairingKey::of([7; 32]),
            )
            .unwrap(),
        );

        assert_eq!(pairings.every().len(), 1);
        assert!(pairings.revoke(&another()));
        assert!(pairings.every().is_empty());
    }

    /// The list shows when each pairing was made, because a person looking at
    /// it is asking what they did and when.
    #[test]
    fn the_list_says_when_each_pairing_was_made_and_when_it_ends() {
        let mut pairings = Pairings::none();
        pairings.keep(for_a_day());
        let one = pairings.every().first().unwrap();
        assert_eq!(one.made(), a_moment());
        assert_eq!(one.ends(), a_moment() + Duration::from_secs(86_400));
    }

    /// **Being paired at all is asked apart from what the pairing permits**,
    /// and answers `false` for a stranger, for a pairing that ended and for one
    /// that was revoked — the four cases the module's own documentation names.
    #[test]
    fn whether_a_machine_is_paired_with_is_asked_apart_from_what_it_may_ask_for() {
        let mut pairings = Pairings::none();
        assert!(!pairings.paired_with(&another(), a_moment()));

        pairings.keep(for_a_day());
        assert!(pairings.paired_with(&another(), a_moment()));
        assert!(pairings.paired_with(&another(), a_moment() + Duration::from_secs(86_399)));
        // Paired, and permitted nothing but what its list says.
        assert!(!pairings.permits(&another(), MayAskIts::Workspace, a_moment()));

        assert!(!pairings.paired_with(&a_stranger(), a_moment()));
        assert!(!pairings.paired_with(&another(), a_moment() + Duration::from_secs(86_400)));

        assert!(pairings.revoke(&another()));
        assert!(!pairings.paired_with(&another(), a_moment()));
    }
}
