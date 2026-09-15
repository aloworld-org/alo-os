//! Every proposal waiting on this machine, and each thing that can happen to
//! one — decided here, with no socket in sight.
//!
//! This is the pairing wire with the wire taken out. Each transition takes
//! what arrived and the moment, and answers with what to send back or what
//! was kept; `receiving.rs` and `crossing.rs` are what carries those answers
//! over a connection, and neither decides anything this file does not. That
//! split is what lets every refusal here be held by a test with no port in
//! it, and the wire tests hold only that the road is walked.
//!
//! # What is refused, and in what order
//!
//! A proposal that arrives is refused **before anybody is shown anything**
//! when it names a machine other than the one it arrived at, when it came
//! from an address discovery never measured, and when one from the same
//! machine is already waiting. (One whose offer does not read never reaches
//! here: `carried.rs` refuses it as a line.) A confirmation that arrives is
//! refused when nothing is waiting for it, when it came from somewhere other
//! than where the other machine answers, and when it was not made with the
//! key only the other machine holds. None of those consults anybody on this
//! machine and none of them is written anywhere.
//!
//! # Nothing is kept until both people have confirmed
//!
//! A [`Pairing`] comes out of exactly two transitions —
//! [`Proposals::confirmed_here`] and [`Proposals::confirmation_arrived`] —
//! and out of either only when it is the second of the two confirmations.
//! Whoever holds the [`crate::Pairings`] keeps what comes out, and that is
//! the first moment anything about a proposal is something that happened to
//! this machine: a proposal that lapsed, was refused or was withdrawn leaves
//! no row and no record, because it is not something that happened.
//!
//! # A proposal lapses
//!
//! [`WHILE_A_PROPOSAL_WAITS`] after it began, on each machine's own clock,
//! and every transition here forgets what has lapsed before it looks — so a
//! lapsed proposal is gone at the next question rather than at a sweep, the
//! way an expired pairing is.

use std::net::IpAddr;
use std::time::{Duration, SystemTime};

use crate::confirming::Confirmation;
use crate::deliberating::{Deliberating, Proposal, Side};
use crate::keying::{Keying, Offer};
use crate::machine::MachineId;
use crate::pairing::{NotPaired, Pairing};
use crate::presence::Found;
use crate::refusing::NotNearby;
use crate::waiting::Waiting;
use crate::words;

/// How long a proposal waits for an answer and two confirmations.
///
/// Ten minutes. Long enough for one person to walk to the other machine and
/// for the other to be found; short enough that a proposal nobody answered
/// is not a row somebody finds tomorrow. Stated here, once, and read by
/// [`Waiting::until`] so a surface can say when.
pub const WHILE_A_PROPOSAL_WAITS: Duration = Duration::from_secs(10 * 60);

/// The most a refusal read off the wire is kept of, in characters.
const AT_MOST_A_REASON: usize = 64;

/// Why a proposal, an answer or a confirmation was not taken.
///
/// **No `Display`**, for the reason [`NotPaired`] has none: the road to words
/// is [`said`](NotProposed::said), in the language of the person reading —
/// who is the person that proposed, since nobody at the machine that refused
/// was shown anything.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum NotProposed {
    /// The proposal names a machine other than the one it is on: it arrived
    /// naming another machine as asked, or was made here naming another as
    /// asking.
    ForAnotherMachine,
    /// It came from an address discovery never measured the other machine at.
    NotFromWhereItWasFound,
    /// A proposal between these two machines is already waiting.
    AlreadyWaiting,
    /// The surface could not show it to anybody, so it was not answered.
    NobodyToShowItTo,
    /// Nothing is waiting between these two machines.
    NothingWaiting,
    /// The other machine has not answered, so there is no code to confirm.
    NotAnsweredYet,
    /// A confirmation arrived that was not made by the other machine.
    NotConfirmedByThatMachine,
    /// The other machine refused, for a reason spelt on the wire as this.
    RefusedThere(String),
    /// What [`Proposal::checked`] or the agreement underneath refused.
    NotPaired(NotPaired),
    /// The network, the randomness or the reading refused underneath.
    Underneath(NotNearby),
}

impl NotProposed {
    /// The sentence a person reads for this.
    #[must_use]
    pub fn word(&self) -> alo_strings::Word {
        match self {
            Self::ForAnotherMachine => words::A_PROPOSAL_FOR_ANOTHER_MACHINE,
            Self::NotFromWhereItWasFound => words::A_PROPOSAL_FROM_AN_ADDRESS_NOT_SEEN,
            Self::AlreadyWaiting => words::A_PROPOSAL_IS_ALREADY_WAITING,
            Self::NobodyToShowItTo => words::NOBODY_AT_THE_OTHER_MACHINE,
            Self::NothingWaiting => words::NO_PROPOSAL_IS_WAITING,
            Self::NotAnsweredYet => words::THE_OTHER_MACHINE_HAS_NOT_ANSWERED_YET,
            Self::NotConfirmedByThatMachine => words::A_CONFIRMATION_NOT_FROM_THAT_MACHINE,
            Self::RefusedThere(_) => words::THE_OTHER_MACHINE_DID_NOT_ACCEPT,
            Self::NotPaired(why) => why.word(),
            Self::Underneath(_) => words::THE_OTHER_MACHINE_COULD_NOT_BE_REACHED,
        }
    }

    /// This refusal, in the language the person reads.
    #[must_use]
    pub fn said(&self, strings: &alo_strings::Strings) -> alo_strings::Said {
        strings.say(&self.word().key(), &alo_strings::Filling::nothing())
    }

    /// This refusal as the machine that refused writes it back: one lowercase
    /// word, so the machine that asked can say the same thing in its person's
    /// language.
    #[must_use]
    pub const fn on_the_wire(&self) -> &'static str {
        match self {
            Self::ForAnotherMachine => "for-another-machine",
            Self::NotFromWhereItWasFound => "not-from-where-it-was-found",
            Self::AlreadyWaiting => "already-waiting",
            Self::NobodyToShowItTo => "nobody-to-show-it-to",
            Self::NothingWaiting => "nothing-waiting",
            Self::NotAnsweredYet => "not-answered-yet",
            Self::NotConfirmedByThatMachine => "not-confirmed-by-that-machine",
            Self::RefusedThere(_) => "refused",
            Self::NotPaired(_) => "not-paired",
            Self::Underneath(_) => "not-readable",
        }
    }

    /// A refusal the other machine wrote back, read.
    ///
    /// A word this crate did not write is kept as [`RefusedThere`](Self::RefusedThere),
    /// cut to a length a log can hold: the other machine refused, and what it
    /// said is not a sentence anybody here reads.
    #[must_use]
    pub fn off_the_wire(said: &str) -> Self {
        let said = said.trim();
        match said {
            "for-another-machine" => Self::ForAnotherMachine,
            "not-from-where-it-was-found" => Self::NotFromWhereItWasFound,
            "already-waiting" => Self::AlreadyWaiting,
            "nobody-to-show-it-to" => Self::NobodyToShowItTo,
            "nothing-waiting" => Self::NothingWaiting,
            "not-answered-yet" => Self::NotAnsweredYet,
            "not-confirmed-by-that-machine" => Self::NotConfirmedByThatMachine,
            "not-readable" => Self::Underneath(NotNearby::NotAMessage(
                "the other machine could not read what this one sent".to_owned(),
            )),
            _ => Self::RefusedThere(said.chars().take(AT_MOST_A_REASON).collect()),
        }
    }
}

/// Every proposal waiting on this machine.
#[derive(Debug)]
pub struct Proposals {
    /// This machine, which every proposal here names on one side.
    here: MachineId,
    /// In the order they began waiting, at most one per other machine.
    waiting: Vec<Waiting>,
}

impl Proposals {
    /// No proposals waiting, on this machine.
    #[must_use]
    pub const fn on(here: MachineId) -> Self {
        Self {
            here,
            waiting: Vec::new(),
        }
    }

    /// This machine.
    #[must_use]
    pub const fn here(&self) -> &MachineId {
        &self.here
    }

    /// Every proposal waiting, in the order they began — the whole list, for
    /// the surface that shows them.
    #[must_use]
    pub fn every(&self) -> &[Waiting] {
        &self.waiting
    }

    /// The proposal waiting between this machine and `other`, if one is.
    #[must_use]
    pub fn with(&self, other: &MachineId) -> Option<&Waiting> {
        self.index_of(other).and_then(|at| self.waiting.get(at))
    }

    /// Forget every proposal that has lapsed at `now`, answering how many.
    ///
    /// Every other transition does this first, so a caller need not; it is
    /// public for a surface that wants its list current without asking
    /// anything else.
    pub fn lapsed(&mut self, now: SystemTime) -> usize {
        let before = self.waiting.len();
        self.waiting.retain(|waiting| !waiting.has_lapsed(now));
        before.saturating_sub(self.waiting.len())
    }

    /// Where the proposal with `other` is in the list.
    fn index_of(&self, other: &MachineId) -> Option<usize> {
        self.waiting
            .iter()
            .position(|waiting| waiting.other() == other)
    }

    /// A proposal made here, to a machine discovery found, begins waiting.
    ///
    /// `keying` is the one whose offer the proposal carries. Nothing has
    /// crossed the wire yet; `crossing::propose` is what carries it.
    ///
    /// # Errors
    ///
    /// [`NotProposed::ForAnotherMachine`] if the proposal does not name this
    /// machine as asking or does not name the found machine as asked;
    /// [`NotProposed::AlreadyWaiting`] if a proposal between the two is
    /// already waiting, in either direction; and [`NotProposed::NotPaired`]
    /// as [`Deliberating::asking`] answers.
    pub fn proposed(
        &mut self,
        proposal: Proposal,
        keying: Keying,
        to: &Found,
        now: SystemTime,
    ) -> Result<&Waiting, NotProposed> {
        self.lapsed(now);
        if proposal.asking() != &self.here || proposal.asked() != &to.machine {
            return Err(NotProposed::ForAnotherMachine);
        }
        if self.with(proposal.asked()).is_some() {
            return Err(NotProposed::AlreadyWaiting);
        }
        let deliberating =
            Deliberating::asking(proposal, keying).map_err(NotProposed::NotPaired)?;
        self.waiting
            .push(Waiting::begun(deliberating, now, to.where_it_answers()));
        self.waiting.last().ok_or(NotProposed::NothingWaiting)
    }

    /// Forget the proposal with `other`, answering whether there was one.
    ///
    /// For the asking machine whose proposal the other refused, and for a
    /// person on either machine who has decided against it. Nothing is sent
    /// and nothing is written: a proposal withdrawn is one that did not
    /// happen, and the other machine's copy lapses on its own.
    pub fn withdrawn(&mut self, other: &MachineId) -> bool {
        let before = self.waiting.len();
        self.waiting.retain(|waiting| waiting.other() != other);
        self.waiting.len() != before
    }

    /// The asked machine's offer came back to the proposal made here.
    ///
    /// From here the code is known, and the surface has something to show.
    ///
    /// # Errors
    ///
    /// [`NotProposed::NothingWaiting`] if no proposal made here to `from` is
    /// waiting; [`NotProposed::NotPaired`] as
    /// [`Deliberating::answered_with`] answers — after which the proposal is
    /// gone, because an answer that was this machine's own offer reflected
    /// back is not one to go on waiting for.
    pub fn answered(
        &mut self,
        from: &MachineId,
        offer: Offer,
        now: SystemTime,
    ) -> Result<&Waiting, NotProposed> {
        self.lapsed(now);
        let at = self
            .index_of(from)
            .filter(|&at| {
                self.waiting
                    .get(at)
                    .is_some_and(|waiting| waiting.on() == Side::TheOneAsking)
            })
            .ok_or(NotProposed::NothingWaiting)?;
        let (deliberating, since, other_at) = self.waiting.remove(at).into_deliberating();
        let answered = deliberating
            .answered_with(offer)
            .map_err(NotProposed::NotPaired)?;
        self.waiting
            .insert(at, Waiting::begun(answered, since, other_at));
        self.waiting.get(at).ok_or(NotProposed::NothingWaiting)
    }

    /// A proposal arrived here, from the address `from`, with `found` being
    /// what discovery on this machine has measured.
    ///
    /// What is returned is what the surface shows, and its
    /// [`Waiting::code`] is known already; the offer to answer with is on it.
    ///
    /// # Errors
    ///
    /// In this order, and before anybody is shown anything:
    /// [`NotProposed::ForAnotherMachine`] when it names a machine other than
    /// this one as asked; [`NotProposed::NotFromWhereItWasFound`] when
    /// discovery here has not measured the asking machine at `from`;
    /// [`NotProposed::AlreadyWaiting`] when a proposal between the two is
    /// already waiting; and [`NotProposed::Underneath`] if this machine can
    /// draw no randomness for its own offer.
    pub fn arrived(
        &mut self,
        proposal: Proposal,
        from: IpAddr,
        found: &[Found],
        now: SystemTime,
    ) -> Result<&Waiting, NotProposed> {
        self.lapsed(now);
        if proposal.asked() != &self.here {
            return Err(NotProposed::ForAnotherMachine);
        }
        let Some(seen) = found.iter().find(|seen| {
            seen.machine == *proposal.asking() && seen.addresses().any(|at| at == from)
        }) else {
            return Err(NotProposed::NotFromWhereItWasFound);
        };
        if self.with(proposal.asking()).is_some() {
            return Err(NotProposed::AlreadyWaiting);
        }
        let keying = Keying::fresh().map_err(NotProposed::Underneath)?;
        let deliberating = Deliberating::asked(proposal, keying);
        self.waiting
            .push(Waiting::begun(deliberating, now, seen.where_it_answers()));
        self.waiting.last().ok_or(NotProposed::NothingWaiting)
    }

    /// The confirmation this machine sends `other` when the person here
    /// confirms. Nothing changes here: it is counted by
    /// [`confirmed_here`](Self::confirmed_here) once it has been sent.
    ///
    /// # Errors
    ///
    /// [`NotProposed::NothingWaiting`] if nothing is waiting with `other`;
    /// [`NotProposed::NotAnsweredYet`] if the code is not known here yet —
    /// there is nothing to have confirmed.
    pub fn confirmation_for(
        &mut self,
        other: &MachineId,
        now: SystemTime,
    ) -> Result<Confirmation, NotProposed> {
        self.lapsed(now);
        self.with(other)
            .ok_or(NotProposed::NothingWaiting)?
            .confirmation()
            .ok_or(NotProposed::NotAnsweredYet)
    }

    /// The person at this machine confirmed the proposal with `other`, having
    /// been shown the code.
    ///
    /// Answers the pairing if this was the second of the two confirmations,
    /// in which case the proposal is no longer waiting and the pairing is
    /// the caller's to keep; `None` if the other person has not confirmed
    /// yet, in which case it goes on waiting.
    ///
    /// # Errors
    ///
    /// [`NotProposed::NothingWaiting`] and [`NotProposed::NotAnsweredYet`] as
    /// [`confirmation_for`](Self::confirmation_for); [`NotProposed::NotPaired`]
    /// if the pairing could not be made from two confirmations, which two
    /// offers that read do not cause.
    pub fn confirmed_here(
        &mut self,
        other: &MachineId,
        now: SystemTime,
    ) -> Result<Option<Pairing>, NotProposed> {
        self.lapsed(now);
        let at = self.index_of(other).ok_or(NotProposed::NothingWaiting)?;
        if self
            .waiting
            .get(at)
            .is_none_or(|waiting| waiting.code().is_none())
        {
            return Err(NotProposed::NotAnsweredYet);
        }
        let waiting = self.waiting.remove(at).confirmed_by_the_person_here();
        self.settled(at, waiting, now)
    }

    /// A confirmation arrived from the address `from`.
    ///
    /// Answers the pairing if this was the second of the two confirmations,
    /// exactly as [`confirmed_here`](Self::confirmed_here) does.
    ///
    /// # Errors
    ///
    /// In this order: [`NotProposed::ForAnotherMachine`] when it is not for
    /// this machine; [`NotProposed::NothingWaiting`] when nothing is waiting
    /// with the machine it says it is from; [`NotProposed::NotFromWhereItWasFound`]
    /// when it did not come from where that machine answers;
    /// [`NotProposed::NotAnsweredYet`] and
    /// [`NotProposed::NotConfirmedByThatMachine`] as `Waiting::holds`
    /// answers; and [`NotProposed::NotPaired`] as above.
    pub fn confirmation_arrived(
        &mut self,
        confirmation: &Confirmation,
        from: IpAddr,
        now: SystemTime,
    ) -> Result<Option<Pairing>, NotProposed> {
        self.lapsed(now);
        if confirmation.to() != &self.here {
            return Err(NotProposed::ForAnotherMachine);
        }
        let at = self
            .index_of(confirmation.from())
            .ok_or(NotProposed::NothingWaiting)?;
        let waiting = self.waiting.get(at).ok_or(NotProposed::NothingWaiting)?;
        if waiting.where_the_other_answers().ip() != from {
            return Err(NotProposed::NotFromWhereItWasFound);
        }
        waiting.holds(confirmation)?;
        let waiting = self.waiting.remove(at).confirmed_by_the_person_there();
        self.settled(at, waiting, now)
    }

    /// A proposal with one more confirmation on it: kept if that made two,
    /// put back at `at` if not.
    fn settled(
        &mut self,
        at: usize,
        waiting: Waiting,
        now: SystemTime,
    ) -> Result<Option<Pairing>, NotProposed> {
        if waiting.is_mutual() {
            waiting.kept(now).map(Some).map_err(NotProposed::NotPaired)
        } else {
            self.waiting.insert(at, waiting);
            Ok(None)
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};
    use std::time::{Duration, SystemTime};

    use super::{NotProposed, Proposals, WHILE_A_PROPOSAL_WAITS};
    use crate::confirming::Confirmation;
    use crate::deliberating::Proposal;
    use crate::keying::Keying;
    use crate::pairing::NotPaired;
    use crate::permitting::MayAskIts;
    use crate::presence::Found;
    use crate::proof::Proof;
    use crate::proven::Proven;
    use crate::refusing::NotNearby;
    use crate::replaying::Seen;
    use crate::testing::{a_moment, a_proposal, a_stranger, reception, studio};
    use crate::{Pairings, nearby_words};

    /// Where both machines are, in a test with no wire.
    const HERE: IpAddr = IpAddr::V4(Ipv4Addr::LOCALHOST);

    /// The studio, as reception's discovery found it.
    fn the_studio_found() -> Found {
        Found::seen(studio(), 7_610, HERE)
    }

    /// Reception, as the studio's discovery found it.
    fn reception_found() -> Found {
        Found::seen(reception(), 7_611, HERE)
    }

    /// Both machines, with nothing waiting on either.
    fn both() -> (Proposals, Proposals) {
        (Proposals::on(reception()), Proposals::on(studio()))
    }

    /// Reception has proposed and the studio has answered: both machines,
    /// each with the proposal waiting and the code known.
    fn answered() -> (Proposals, Proposals) {
        let (mut at_reception, mut at_studio) = both();
        let (proposal, keying) = a_proposal();
        at_reception
            .proposed(proposal.clone(), keying, &the_studio_found(), a_moment())
            .unwrap();
        let offer = at_studio
            .arrived(proposal, HERE, &[reception_found()], a_moment())
            .unwrap()
            .answered()
            .unwrap()
            .clone();
        at_reception.answered(&studio(), offer, a_moment()).unwrap();
        (at_reception, at_studio)
    }

    /// **The road, with no wire**: proposed, arrived, answered, both codes the
    /// same, each person confirms on their own, and each machine keeps a
    /// pairing only at the second confirmation — a pairing that works, held
    /// by a proof crossing it.
    #[test]
    fn a_pairing_is_kept_on_each_machine_only_after_both_people_confirmed() {
        let (mut at_reception, mut at_studio) = both();
        let (proposal, keying) = a_proposal();
        let waiting = at_reception
            .proposed(proposal.clone(), keying, &the_studio_found(), a_moment())
            .unwrap();
        assert!(waiting.code().is_none(), "a code before the answer");

        let shown = at_studio
            .arrived(proposal, HERE, &[reception_found()], a_moment())
            .unwrap();
        let code_at_the_studio = shown.code().unwrap();
        let offer = shown.answered().unwrap().clone();
        let shown = at_reception.answered(&studio(), offer, a_moment()).unwrap();
        assert_eq!(shown.code().unwrap(), code_at_the_studio);

        // Reception's person confirms; nothing is kept anywhere.
        let from_reception = at_reception
            .confirmation_for(&studio(), a_moment())
            .unwrap();
        assert!(
            at_reception
                .confirmed_here(&studio(), a_moment())
                .unwrap()
                .is_none()
        );
        assert!(
            at_studio
                .confirmation_arrived(&from_reception, HERE, a_moment())
                .unwrap()
                .is_none()
        );
        assert!(at_studio.with(&reception()).unwrap().confirmed_there());
        assert!(!at_studio.with(&reception()).unwrap().confirmed_here());

        // The studio's person confirms: the studio keeps its pairing, and
        // reception keeps its own when the confirmation arrives.
        let from_studio = at_studio
            .confirmation_for(&reception(), a_moment())
            .unwrap();
        let on_studio = at_studio
            .confirmed_here(&reception(), a_moment())
            .unwrap()
            .unwrap();
        let on_reception = at_reception
            .confirmation_arrived(&from_studio, HERE, a_moment())
            .unwrap()
            .unwrap();
        assert!(at_studio.every().is_empty());
        assert!(at_reception.every().is_empty());
        assert_eq!(on_studio.with(), &reception());
        assert_eq!(on_reception.with(), &studio());

        let mut pairings = Pairings::none();
        pairings.keep(on_studio);
        let proof = Proof::made(&on_reception, &reception(), b"a question", a_moment());
        assert!(
            Proven::checked(
                &pairings,
                &studio(),
                &proof,
                b"a question",
                a_moment(),
                &mut Seen::nothing()
            )
            .is_ok()
        );
    }

    /// **The asking side confirming alone keeps nothing**, and neither does
    /// the asked side — on either machine, however many times.
    #[test]
    fn one_side_confirming_alone_keeps_nothing_on_either_machine() {
        let (mut at_reception, at_studio) = answered();
        for _ in 0..2 {
            assert!(
                at_reception
                    .confirmed_here(&studio(), a_moment())
                    .unwrap()
                    .is_none()
            );
        }
        assert_eq!(at_reception.every().len(), 1);
        assert_eq!(at_studio.every().len(), 1);

        let (at_reception, mut at_studio) = answered();
        for _ in 0..2 {
            assert!(
                at_studio
                    .confirmed_here(&reception(), a_moment())
                    .unwrap()
                    .is_none()
            );
        }
        assert_eq!(at_reception.every().len(), 1);
        assert_eq!(at_studio.every().len(), 1);
    }

    /// A proposal made here names this machine as asking and the found
    /// machine as asked, and anything else is refused before it waits.
    #[test]
    fn a_proposal_made_here_names_this_machine_and_the_machine_found() {
        let (mut at_reception, mut at_studio) = both();
        let (proposal, keying) = a_proposal();
        assert_eq!(
            at_studio
                .proposed(proposal.clone(), keying, &the_studio_found(), a_moment())
                .unwrap_err(),
            NotProposed::ForAnotherMachine
        );
        let (proposal, keying) = a_proposal();
        assert_eq!(
            at_reception
                .proposed(
                    proposal,
                    keying,
                    &Found::seen(a_stranger(), 7_610, HERE),
                    a_moment()
                )
                .unwrap_err(),
            NotProposed::ForAnotherMachine
        );
        let (proposal, _) = a_proposal();
        assert_eq!(
            at_reception
                .proposed(
                    proposal,
                    Keying::fresh().unwrap(),
                    &the_studio_found(),
                    a_moment()
                )
                .unwrap_err(),
            NotProposed::NotPaired(NotPaired::NotTheOfferMade)
        );
        assert!(at_reception.every().is_empty() && at_studio.every().is_empty());
    }

    /// **A second proposal between the same machines while the first waits is
    /// refused** — made here twice, arrived twice, and arrived while one made
    /// here to that machine waits.
    #[test]
    fn a_second_proposal_while_the_first_waits_is_refused() {
        let (mut at_reception, mut at_studio) = both();
        let (proposal, keying) = a_proposal();
        at_reception
            .proposed(proposal.clone(), keying, &the_studio_found(), a_moment())
            .unwrap();
        let (again, keying) = a_proposal();
        assert_eq!(
            at_reception
                .proposed(again, keying, &the_studio_found(), a_moment())
                .unwrap_err(),
            NotProposed::AlreadyWaiting
        );

        at_studio
            .arrived(proposal.clone(), HERE, &[reception_found()], a_moment())
            .unwrap();
        let (again, _) = a_proposal();
        assert_eq!(
            at_studio
                .arrived(again, HERE, &[reception_found()], a_moment())
                .unwrap_err(),
            NotProposed::AlreadyWaiting
        );
        assert_eq!(at_studio.every().len(), 1);

        // And the other way round: the studio proposed to reception, and a
        // proposal from reception arrives while that waits.
        let mut at_studio = Proposals::on(studio());
        let studios_keying = Keying::fresh().unwrap();
        let from_the_studio = Proposal::checked(
            studio(),
            reception(),
            &[MayAskIts::Models],
            Duration::from_secs(60),
            studios_keying.offer().clone(),
        )
        .unwrap();
        at_studio
            .proposed(
                from_the_studio,
                studios_keying,
                &reception_found(),
                a_moment(),
            )
            .unwrap();
        assert_eq!(
            at_studio
                .arrived(proposal, HERE, &[reception_found()], a_moment())
                .unwrap_err(),
            NotProposed::AlreadyWaiting
        );
    }

    /// **A proposal that arrived naming another machine is refused** before
    /// anybody is shown anything, and nothing waits.
    #[test]
    fn a_proposal_that_arrived_naming_another_machine_is_refused() {
        let (_, mut at_studio) = both();
        let keying = Keying::fresh().unwrap();
        let for_a_stranger = Proposal::checked(
            reception(),
            a_stranger(),
            &[MayAskIts::Models],
            Duration::from_secs(60),
            keying.offer().clone(),
        )
        .unwrap();
        assert_eq!(
            at_studio
                .arrived(for_a_stranger, HERE, &[reception_found()], a_moment())
                .unwrap_err(),
            NotProposed::ForAnotherMachine
        );
        assert!(at_studio.every().is_empty());
    }

    /// **A proposal from an address discovery never measured is refused** —
    /// from a machine discovery never found, and from a found machine at an
    /// address it was not found at.
    #[test]
    fn a_proposal_from_an_address_discovery_never_measured_is_refused() {
        let (_, mut at_studio) = both();
        let (proposal, _) = a_proposal();
        assert_eq!(
            at_studio
                .arrived(proposal.clone(), HERE, &[], a_moment())
                .unwrap_err(),
            NotProposed::NotFromWhereItWasFound
        );
        let elsewhere = Found::seen(reception(), 7_611, Ipv4Addr::new(192, 168, 1, 40).into());
        assert_eq!(
            at_studio
                .arrived(proposal, HERE, &[elsewhere], a_moment())
                .unwrap_err(),
            NotProposed::NotFromWhereItWasFound
        );
        assert!(at_studio.every().is_empty());
    }

    /// An answer for nothing waiting is refused, and so is one to the asked
    /// side, and so is this machine's own offer reflected back — after which
    /// the proposal is gone.
    #[test]
    fn an_answer_for_nothing_waiting_or_reflected_back_is_refused() {
        let (mut at_reception, mut at_studio) = both();
        let offer = Keying::fresh().unwrap().offer().clone();
        assert_eq!(
            at_reception
                .answered(&studio(), offer.clone(), a_moment())
                .unwrap_err(),
            NotProposed::NothingWaiting
        );
        let (proposal, keying) = a_proposal();
        at_studio
            .arrived(proposal.clone(), HERE, &[reception_found()], a_moment())
            .unwrap();
        assert_eq!(
            at_studio
                .answered(&reception(), offer, a_moment())
                .unwrap_err(),
            NotProposed::NothingWaiting
        );

        at_reception
            .proposed(proposal.clone(), keying, &the_studio_found(), a_moment())
            .unwrap();
        assert_eq!(
            at_reception
                .answered(&studio(), proposal.offered().clone(), a_moment())
                .unwrap_err(),
            NotProposed::NotPaired(NotPaired::NotTheOfferMade)
        );
        assert!(at_reception.every().is_empty());
    }

    /// Confirming before the answer is refused: there is no code to have
    /// confirmed.
    #[test]
    fn confirming_before_the_other_machine_has_answered_is_refused() {
        let (mut at_reception, _) = both();
        let (proposal, keying) = a_proposal();
        at_reception
            .proposed(proposal, keying, &the_studio_found(), a_moment())
            .unwrap();
        assert_eq!(
            at_reception
                .confirmation_for(&studio(), a_moment())
                .unwrap_err(),
            NotProposed::NotAnsweredYet
        );
        assert_eq!(
            at_reception
                .confirmed_here(&studio(), a_moment())
                .unwrap_err(),
            NotProposed::NotAnsweredYet
        );
        assert_eq!(
            at_reception
                .confirmed_here(&a_stranger(), a_moment())
                .unwrap_err(),
            NotProposed::NothingWaiting
        );
    }

    /// **A confirmation is refused** when nothing is waiting for it, when it
    /// is for another machine, when it came from somewhere other than where
    /// the other machine answers, and when it was not made by that machine —
    /// and none of them changes what is waiting.
    #[test]
    fn a_confirmation_that_is_not_the_other_machines_is_refused() {
        let (mut at_reception, mut at_studio) = answered();
        let from_studio = at_studio
            .confirmation_for(&reception(), a_moment())
            .unwrap();
        let from_reception = at_reception
            .confirmation_for(&studio(), a_moment())
            .unwrap();

        // Nothing waiting: a machine with no proposal.
        assert_eq!(
            Proposals::on(reception())
                .confirmation_arrived(&from_studio, HERE, a_moment())
                .unwrap_err(),
            NotProposed::NothingWaiting
        );
        // For another machine: the studio's confirmation to reception,
        // arriving at the studio.
        assert_eq!(
            at_studio
                .confirmation_arrived(&from_studio, HERE, a_moment())
                .unwrap_err(),
            NotProposed::ForAnotherMachine
        );
        // From somewhere other than where the studio answers.
        assert_eq!(
            at_reception
                .confirmation_arrived(
                    &from_studio,
                    Ipv4Addr::new(192, 168, 1, 40).into(),
                    a_moment()
                )
                .unwrap_err(),
            NotProposed::NotFromWhereItWasFound
        );
        // Not made by the studio: reception's own, with the machines swapped
        // so it names the right ones.
        let forged = Confirmation::read(&format!(
            "alo-os/1 confirmed {} {} {}",
            studio(),
            reception(),
            from_reception.said().rsplit(' ').next().unwrap()
        ))
        .unwrap();
        assert_eq!(
            at_reception
                .confirmation_arrived(&forged, HERE, a_moment())
                .unwrap_err(),
            NotProposed::NotConfirmedByThatMachine
        );

        let waiting = at_reception.with(&studio()).unwrap();
        assert!(!waiting.confirmed_here() && !waiting.confirmed_there());
    }

    /// **A proposal nobody answered lapses from both machines within the
    /// stated time**, at the next question rather than at a sweep, and one
    /// that was answered lapses the same way if nobody confirms.
    #[test]
    fn a_proposal_nobody_answered_lapses_from_both_machines_within_the_stated_time() {
        let (mut at_reception, mut at_studio) = both();
        let (proposal, keying) = a_proposal();
        at_reception
            .proposed(proposal.clone(), keying, &the_studio_found(), a_moment())
            .unwrap();
        at_studio
            .arrived(proposal, HERE, &[reception_found()], a_moment())
            .unwrap();
        assert_eq!(
            at_reception.with(&studio()).unwrap().until(),
            a_moment() + WHILE_A_PROPOSAL_WAITS
        );

        let just_before = a_moment() + WHILE_A_PROPOSAL_WAITS - Duration::from_secs(1);
        assert_eq!(at_reception.lapsed(just_before), 0);
        assert_eq!(at_studio.lapsed(just_before), 0);

        let the_stated_time = a_moment() + WHILE_A_PROPOSAL_WAITS;
        assert_eq!(
            at_studio
                .confirmed_here(&reception(), the_stated_time)
                .unwrap_err(),
            NotProposed::NothingWaiting
        );
        assert!(at_studio.every().is_empty());
        assert_eq!(at_reception.lapsed(the_stated_time), 1);
        assert!(at_reception.every().is_empty());
    }

    /// A proposal withdrawn is gone, and withdrawing nothing says so.
    #[test]
    fn a_proposal_withdrawn_is_gone() {
        let (mut at_reception, _) = answered();
        assert!(at_reception.withdrawn(&studio()));
        assert!(!at_reception.withdrawn(&studio()));
        assert!(at_reception.every().is_empty());
    }

    /// Every refusal has a sentence, and every refusal the other machine can
    /// write back is read as itself.
    #[test]
    fn every_refusal_can_be_said_to_a_person_and_read_off_the_wire() {
        let strings = alo_strings::Strings::of(nearby_words().unwrap());
        for refused in [
            NotProposed::ForAnotherMachine,
            NotProposed::NotFromWhereItWasFound,
            NotProposed::AlreadyWaiting,
            NotProposed::NobodyToShowItTo,
            NotProposed::NothingWaiting,
            NotProposed::NotAnsweredYet,
            NotProposed::NotConfirmedByThatMachine,
            NotProposed::RefusedThere("something".to_owned()),
            NotProposed::NotPaired(NotPaired::NotTheOfferMade),
            NotProposed::Underneath(NotNearby::NotAMessage("nothing".to_owned())),
        ] {
            assert!(!refused.said(&strings).text().is_empty(), "{refused:?}");
            let read_back = NotProposed::off_the_wire(refused.on_the_wire());
            match refused {
                NotProposed::RefusedThere(_) | NotProposed::NotPaired(_) => {
                    assert!(matches!(read_back, NotProposed::RefusedThere(_)));
                }
                NotProposed::Underneath(_) => {
                    assert!(matches!(read_back, NotProposed::Underneath(_)));
                }
                ref same => assert_eq!(&read_back, same),
            }
        }
        let long: String = "x".repeat(500);
        let NotProposed::RefusedThere(kept) = NotProposed::off_the_wire(&long) else {
            unreachable!("a word this crate did not write was read as one it did")
        };
        assert_eq!(kept.len(), 64);
    }

    /// The moment is the caller's: nothing here reads a clock.
    #[test]
    fn nothing_here_reads_the_clock() {
        let (mut at_reception, _) = answered();
        let long_ago = SystemTime::UNIX_EPOCH;
        assert_eq!(at_reception.lapsed(long_ago), 0);
        assert_eq!(at_reception.every().len(), 1);
    }
}
