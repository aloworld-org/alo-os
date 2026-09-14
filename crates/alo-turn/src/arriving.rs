//! A turn whose verbs arrive from a paired machine, and whose grants are this
//! machine's.
//!
//! ADR 0003's sharpest line, and the one a reader most wants proof of: *an
//! agent on machine A that reaches machine B is bound by the grants made on B,
//! by B's person. A's grants confer nothing on B. Pairing lets A ask; it never
//! lets A act.* This file is B's side of that sentence, and it is deliberately
//! almost nothing: an [`Arriving`] is a [`Turning`] with four things decided
//! differently and everything else the same.
//!
//! # What is decided differently
//!
//! **Whose grants are asked.** The turn belongs to the machine the verb came
//! from — [`alo_nearby::Origin::principal`] is its name on this machine's
//! grants — and it belongs to nothing else. There is no parameter anywhere on
//! this type for the asking machine's grants, so the guarantee that they confer
//! nothing here is not a check that could be skipped but an absence of any
//! place to hand them over. What this machine's person granted **to that
//! machine** is what it may reach, and a grant they made to their own `@files`
//! is a grant to their own `@files`.
//!
//! **What is offered.** Nothing. A local turn begins with what the person had
//! in front of them — a window, a selection, a document, offered at invocation
//! — and a remote one begins with [`alo_context::Context::at_invocation`] and
//! nothing on it, because nobody on this machine invoked anything and *context
//! is offered, never watched*. Nothing on this machine's screen reaches a
//! machine on the network, and there is no door here through which it could.
//!
//! **Which doors there are.** Four of the six: a read, a proposal, an approval
//! and a decline. There is no [`Turning::asking`] on a remote turn, and the
//! [`Turning`] inside is lent out only for reading — [`Arriving::turning`] — so
//! that a remote turn cannot put a question to this machine's models by the
//! back door. A paired machine that may ask this one's models does so through
//! `alo-asking`'s corridor, under the arm of the pairing that says so, and is
//! recorded there as a question answered for another machine.
//!
//! **How a refusal is worded.** The capability model refuses a remote verb
//! exactly as it refuses a local one, with the same value, and this file
//! re-words two of those refusals before anybody reads them — because the
//! sentence *`@files` has not been granted this, grants are made by picking a
//! folder* is true and, carried back to the person who asked, tells them to fix
//! it from a machine where it cannot be fixed. The grant that was not made was
//! not made **here**, and [`crate::words::NOT_GRANTED_HERE`] says so; the grant
//! that ran out ran out here, and [`crate::words::GRANT_HERE_EXPIRED`] says
//! that. Each travels as `alo_capability::Refused::worded_elsewhere` or
//! `alo_capability::ProposalError::NotGrantedElsewhere`, which is the door the
//! capability model already holds open for a refusal whose words are somebody
//! else's — so the screen and the record still render one value, and the
//! reasoning that decided it is untouched.
//!
//! # What is the same
//!
//! Everything a verb goes through. The name and values become a call against
//! the same closed list, or are turned away and written down. The grants are
//! asked when a change is proposed and again at the moment it runs, so one
//! revoked in between stops it. A change waits for one approval, given on this
//! machine by this machine's person, and the approval is spent by being
//! answered. The work runs inside the boundary the kernel imposes — ADR 0013
//! applies on the receiving machine exactly as it does for a local turn, and a
//! turn whose boundary cannot be applied still does not run. And every entry
//! is written before anything is answered, now stamped with where it came from
//! ([`alo_record::Entry::from_another_machine`]), because ADR 0003 asks that
//! *B records it, with A named as the origin*.
//!
//! # And the pairing is asked at every door
//!
//! A pairing is a grant across a machine boundary, *revocable in one action
//! taking effect immediately*, so it is asked at the moment of every door the
//! way the grants are — never borrowed once at the turn's beginning, which
//! would make a remote turn the one thing on the machine nobody could revoke
//! anything during. A pairing that ended between the proposal and the approval
//! stops the change at the moment it would have run, written down as a refusal
//! at the moment with [`crate::words::NO_LONGER_PAIRED`], and a verb arriving
//! after it is refused before the grants are asked.
//!
//! # And the answer leaves under this machine's indicator
//!
//! An answer to a remote read is this machine's data leaving it, and the
//! number a change waits under is a sentence leaving it; law 1 is not
//! suspended for the length of a corridor. So a remote turn has two doors a
//! local one has no use for: [`Arriving::departing`] asks this machine's
//! egress rule about an answer going back to the origin machine and puts it
//! on this machine's indicator, handing back the [`alo_egress::Departing`]
//! that is the only thing meaning *this may leave*; and [`Arriving::returned`]
//! writes the departure down, stamped with where the verb came from, and
//! takes the line off. A rule that refuses — an organisation that said
//! *nothing leaves* — is written down as `held back`, and nothing goes back.
//! The wire that carries the answer is `alo-corridor`'s, and it has no road
//! to its socket that does not pass through the first of these.
//!
//! # What is not here
//!
//! **The wire.** Nothing in this crate opens a socket, and a verb *arrives* at
//! this type as a name and some values the way one arrives at [`Turning`] from
//! `alo-agentd`'s local protocol. `alo-corridor` carries verbs between
//! machines, calls this door and no other, and holds the departure above
//! when it sends an answer back; this crate holds the order around the door
//! and the socket stays where the sockets are.

use std::time::{Duration, SystemTime};

use alo_capability::{
    Given, GrantError, Grantee, Grants, NotAuthorised, NotGranted, ProposalError, ProposalId,
    Refused, Waiting,
};
use alo_egress::{Departing, Destination, EgressPolicy, Leaving, Why};
use alo_files::Answer;
use alo_nearby::{Origin, Pairings};
use alo_record::Entry;
use alo_strings::{Filling, Said, Strings};

use crate::machine::Machine;
use crate::refusing::NotDone;
use crate::turning::Turning;
use crate::unanswered::NoAnswer;
use crate::words;

/// A turn under way for a paired machine, on this machine's grants.
///
/// Deliberately not `Clone`, like the [`Turning`] inside it, and for one more
/// reason: a remote turn that could be copied could be answered from two
/// places, and one approval on this machine is one execution.
#[derive(Debug)]
pub struct Arriving<'a, 'm> {
    /// The turn, belonging to the machine the verbs come from.
    turning: Turning<'a, 'm>,
    /// Which machine that is, as this machine's pairings know it.
    origin: Origin,
}

impl<'a, 'm> Arriving<'a, 'm> {
    /// Begin a turn for verbs arriving from a paired machine.
    ///
    /// `origin` is the machine they come from, which only
    /// [`alo_nearby::Origin::proven`] makes, from a proof the verb carried
    /// (ADR 0031) — so a turn cannot begin here for a machine this one is not
    /// paired with, nor for a stranger presenting a paired machine's identity,
    /// and there is no constructor that skips that. `at` is the moment, passed rather than read as everywhere
    /// in this workspace. Nothing is offered: no window, no selection, no
    /// document of this machine's, and so nothing is granted at the beginning
    /// of this turn and nothing is taken back at its end.
    ///
    /// **A remote turn cannot begin on a machine with no agent** (ADR 0009),
    /// exactly as a local one cannot: it needs the machine's grants, and a
    /// machine where the person declined has none to lend.
    ///
    /// # Errors
    /// [`GrantError`], carried whole from `alo-capability` — which the
    /// principal a pairing names cannot cause, and is answered rather than
    /// unwrapped for the reason every other door gives.
    pub fn beginning(
        origin: &Origin,
        lasting: Duration,
        at: SystemTime,
        grants: &mut Grants,
        machine: &'a mut Machine<'m>,
    ) -> Result<Self, GrantError> {
        Ok(Self {
            turning: Turning::arriving_from(origin, lasting, at, grants, machine)?,
            origin: origin.clone(),
        })
    }

    /// A read from the paired machine, which answers inside the turn.
    ///
    /// The same road as [`Turning::reading`] — the closed list, the grants,
    /// every path resolved and asked about again, the boundary, the record —
    /// with the pairing asked first, at this moment. **The grants asked are
    /// this machine's, for the paired machine's principal**, and there is no
    /// parameter for any other list.
    ///
    /// # Errors
    /// [`NotDone`], and the entry that says so is written before this answers
    /// — [`NotDone::Refused`] when the pairing has ended, worded as such, and
    /// otherwise everything [`Turning::reading`] can answer with.
    pub fn reading(
        &mut self,
        verb: &str,
        given: &[(&str, Given)],
        pairings: &Pairings,
        grants: &Grants,
        now: SystemTime,
    ) -> Result<Answer, NotDone> {
        self.turning.still_open()?;
        let call = self.turning.calling(verb, given, now)?;
        if let Some(said) = self.pairing_ended(pairings, now) {
            let refused = Refused::worded_elsewhere(call, said);
            return self.turning.stopped_at_the_moment(refused, now);
        }
        self.turning.reading_call(call, grants, now)
    }

    /// A change from the paired machine, put to **this machine's person** in
    /// one sentence.
    ///
    /// The question goes onto this turn's list, which is what an approval
    /// surface on this machine reads through [`Arriving::turning`]; nobody on
    /// the machine that asked is asked anything, and nothing they answered
    /// there reaches here. The grants are asked now, as for a local change, so
    /// a change this machine's person could never have permitted never
    /// interrupts them — and that is written down.
    ///
    /// # Errors
    /// [`NotDone::TurnedAway`] if nothing formed; [`NotDone::NeverAsked`] if
    /// the pairing has ended, if the grants refused it, or if a read was
    /// offered where only a change waits.
    pub fn proposing(
        &mut self,
        verb: &str,
        given: &[(&str, Given)],
        pairings: &Pairings,
        grants: &Grants,
        standing: Duration,
        now: SystemTime,
    ) -> Result<ProposalId, NotDone> {
        self.turning.still_open()?;
        let call = self.turning.calling(verb, given, now)?;
        if let Some(said) = self.pairing_ended(pairings, now) {
            let why = ProposalError::NotGrantedElsewhere(said);
            let entry = Entry::never_asked(
                &call,
                self.turning.grantee(),
                why.said(self.turning.strings()).text(),
                self.turning.strings(),
                now,
            );
            self.turning.writing_down(entry)?;
            return Err(NotDone::NeverAsked(why));
        }
        self.turning.proposing_call(call, grants, standing, now)
    }

    /// This machine's person approved it, so it runs — once.
    ///
    /// The grants are asked again here, and so is the pairing: a grant revoked
    /// or a pairing undone between the question and the answer stops the
    /// change at the moment it would have run, which is what *takes effect
    /// immediately* means on both lists.
    ///
    /// # Errors
    /// Everything [`Turning::approving`] can answer with, and
    /// [`NotDone::Refused`] worded for a pairing that has ended.
    pub fn approving(
        &mut self,
        id: ProposalId,
        pairings: &Pairings,
        grants: &Grants,
        now: SystemTime,
    ) -> Result<Answer, NotDone> {
        let authorised = self.turning.redeeming(id, grants, now)?;
        if let Some(said) = self.pairing_ended(pairings, now) {
            let refused = Refused::worded_elsewhere(authorised.call().clone(), said);
            return self.turning.stopped_at_the_moment(refused, now);
        }
        self.turning.running(authorised, grants)
    }

    /// This machine's person said no.
    ///
    /// No pairing is asked: a person declining something on their own machine
    /// is answered whatever the network is doing, and *no* is the whole answer.
    ///
    /// # Errors
    /// [`NotDone::NotAnswered`] if that number is not waiting, and
    /// [`NotDone::NotRecorded`] if the refusal could not be written down.
    pub fn declining(&mut self, id: ProposalId, now: SystemTime) -> Result<(), NotDone> {
        self.turning.declining(id, now)
    }

    /// End the turn.
    ///
    /// Nothing was granted at its beginning, so nothing is taken back: this
    /// exists so that a remote turn ends the way a local one does, by being
    /// consumed, and so that the changes it put to somebody and nobody
    /// answered go away with it.
    pub fn ending(self, grants: &mut Grants) {
        let _ = self.ended(grants);
    }

    /// End the turn, and hand the machine back.
    ///
    /// [`Arriving::ending`] with the machine this turn was holding returned,
    /// for the door on the receiving machine that holds one remote turn after
    /// another on one machine it does not own — the next turn begins on the
    /// same borrow. [`Turning::ended`] is the same door on a local turn.
    #[must_use]
    pub fn ended(self, grants: &mut Grants) -> &'a mut Machine<'m> {
        let (taken_back, machine) = self.turning.ended(grants);
        debug_assert!(
            !taken_back,
            "a remote turn ended holding a grant it could not have made"
        );
        machine
    }

    /// An answer is about to go back to the machine the verbs came from:
    /// ask this machine's egress rule, and put it on this machine's
    /// indicator.
    ///
    /// Law 1, on the receiving side of the corridor. What leaves is under the
    /// origin machine's principal — the name its grants here are made to —
    /// as [`alo_egress::Why::Sending`] to the machine by the name this
    /// machine's person gave it. The [`Departing`] handed back is the only
    /// thing that means *this may leave*, and `alo-corridor`'s reply to a
    /// proven verb cannot be written without one; [`Arriving::returned`] is
    /// what it is spent on once the answer has gone.
    ///
    /// # Errors
    /// [`NoAnswer::HeldBack`] when the rule in force refuses — written down
    /// as held back, in the rule's own words, before this answers;
    /// [`NoAnswer::CannotBeShown`] when the machine's name could not be put
    /// on the indicator, in which case nothing left and nothing is written;
    /// [`NoAnswer::NotRecorded`] with `after_it_left` false if the held-back
    /// entry could not be written; [`NoAnswer::TurnClosed`] if this turn has
    /// stopped keeping evidence, because nothing leaves a machine that cannot
    /// write down that it did.
    pub fn departing(
        &mut self,
        policy: &EgressPolicy,
        now: SystemTime,
    ) -> Result<Departing, NoAnswer> {
        if self.turning.is_closed() {
            return Err(NoAnswer::TurnClosed);
        }
        let destination =
            Destination::paired(self.origin.called()).map_err(NoAnswer::CannotBeShown)?;
        let leaving = Leaving::because(self.turning.grantee(), Why::Sending, destination);
        match self
            .turning
            .machine()
            .indicator()
            .beginning(policy, leaving, now)
        {
            Ok(departing) => Ok(departing),
            Err(refused) => {
                let entry = Entry::held_back(&refused, self.turning.strings(), now);
                match self.turning.keeping_stamped(entry) {
                    Ok(()) => Err(NoAnswer::HeldBack(refused)),
                    Err(why) => Err(NoAnswer::NotRecorded {
                        why,
                        after_it_left: false,
                    }),
                }
            }
        }
    }

    /// The answer has gone back: write the departure down, and take the line
    /// off the indicator.
    ///
    /// In that order, as [`Turning::asking`] keeps it: the departure is
    /// written before the line comes off and before the caller hears
    /// anything, and the line comes off whatever the record said, because
    /// the indicator is a statement about now and a connection that has
    /// ended is not leaving. The entry is stamped with where the verb came
    /// from, as every entry of a remote turn is.
    ///
    /// # Errors
    /// [`NoAnswer::NotRecorded`] with `after_it_left` true: the answer went
    /// back and there is no evidence of it, which is law 1's second half
    /// failing, and the turn is closed by it.
    pub fn returned(&mut self, departing: Departing) -> Result<(), NoAnswer> {
        let kept = self.turning.keeping_stamped(Entry::left(&departing));
        self.turning.machine().indicator().ended(departing);
        kept.map_err(|why| NoAnswer::NotRecorded {
            why,
            after_it_left: true,
        })
    }

    /// The machine the verbs come from.
    #[must_use]
    pub const fn origin(&self) -> &Origin {
        &self.origin
    }

    /// The turn, for reading.
    ///
    /// What an approval surface on this machine reads its question off, and
    /// what a daemon asks whether the turn is closed. Lent out for reading
    /// only, on purpose: the doors that do anything are this type's own, and
    /// one door a local turn has — putting a question to a model — a remote
    /// turn does not.
    #[must_use]
    pub const fn turning(&self) -> &Turning<'a, 'm> {
        &self.turning
    }

    /// The paired machine's name on this machine's grants.
    #[must_use]
    pub fn grantee(&self) -> &Grantee {
        self.turning.grantee()
    }

    /// The machine this turn holds.
    ///
    /// `pub(crate)`, for [`crate::answering_for`]: a question from a paired
    /// machine is answered by the machine and inside no turn, and while this
    /// turn holds the machine the record and the indicator are behind it. A
    /// public door here would be a way to reach the machine past every door
    /// this turn has.
    pub(crate) fn machine(&mut self) -> &mut Machine<'m> {
        self.turning.machine()
    }

    /// The changes this turn is waiting for this machine's person to answer.
    pub fn waiting_at(&self, now: SystemTime) -> impl Iterator<Item = &Waiting> {
        self.turning.waiting_at(now)
    }

    /// One change this turn put to somebody, by number, whether or not it
    /// still stands.
    #[must_use]
    pub fn proposed(&self, id: ProposalId) -> Option<&Waiting> {
        self.turning.proposed(id)
    }

    /// What became of the change waiting under `number`, at `now` — for
    /// the asking machine, which was told the number and nothing more.
    ///
    /// [`Turning::became`], on the turn this is: a read of this turn's own
    /// memory, asking the grants nothing and running nothing.
    #[must_use]
    pub fn became(&self, number: u64, now: SystemTime) -> crate::became::Became {
        self.turning.became(number, now)
    }

    /// Whether this turn has stopped because something could not be written
    /// down.
    #[must_use]
    pub fn is_closed(&self) -> bool {
        self.turning.is_closed()
    }

    /// Whether a thread of this service went into a boundary and stayed there.
    #[must_use]
    pub fn a_thread_is_lost(&self) -> bool {
        self.turning.a_thread_is_lost()
    }

    /// Write down that the person here was not able to have their grants
    /// read again, while this remote turn holds the machine.
    ///
    /// [`Turning::the_grants_were_not_read_again`], on the turn this is, and
    /// for the daemon's reason: the person's own side says *what is granted
    /// has changed* whether or not a turn is under way, and a remote turn
    /// holding the machine is not a reason for that refusal to go unwritten.
    /// It is the person's own entry: it names no agent, as the local door
    /// does not, and it is not stamped with this turn's origin, because the
    /// other machine caused nothing about it.
    ///
    /// # Errors
    ///
    /// [`alo_keeping::NotKept`] when the record could not be written, and
    /// the turn is closed by it, exactly as the local door answers.
    pub fn the_grants_were_not_read_again(
        &mut self,
        why: &Said,
        now: SystemTime,
    ) -> Result<(), alo_keeping::NotKept> {
        self.turning.the_grants_were_not_read_again(why, now)
    }

    /// Write down that a pairing with `with` was kept on this machine, while
    /// this remote turn holds it.
    ///
    /// [`crate::Machine::a_pairing_was_kept`] with a remote turn in front of
    /// it: a confirmation from a third machine can complete a pairing while
    /// another machine's turn is under way here, and the record is behind
    /// the machine this turn holds. The entry is the machine's own — no
    /// agent, and not stamped with this turn's origin, because the machine
    /// whose turn this is caused nothing about it.
    ///
    /// # Errors
    ///
    /// [`alo_keeping::NotKept`] when the record could not be written, and
    /// the turn is closed by it.
    pub fn a_pairing_was_kept(
        &mut self,
        with: &str,
        now: SystemTime,
    ) -> Result<(), alo_keeping::NotKept> {
        self.turning.keeping(Entry::paired(with, now))
    }

    /// Write down that the person opened a workspace discovery found, while
    /// this remote turn holds the machine.
    ///
    /// [`crate::Machine::a_workspace_was_opened`] with a remote turn in front
    /// of it. The entry is the machine's own — no agent, and not stamped with
    /// this turn's origin, because the machine whose turn this is caused
    /// nothing about it: the person here opened it.
    ///
    /// # Errors
    ///
    /// [`alo_keeping::NotKept`] when the record could not be written, and
    /// the turn is closed by it.
    pub fn a_workspace_was_opened(
        &mut self,
        workspace: &str,
        answers_at: &str,
        now: SystemTime,
    ) -> Result<(), alo_keeping::NotKept> {
        self.turning
            .keeping(Entry::a_workspace_was_opened(workspace, answers_at, now))
    }

    /// The sentence for a pairing that has ended, if it has.
    ///
    /// `None` while the pairing stands, which is the ordinary answer and costs
    /// one look at the list.
    fn pairing_ended(&self, pairings: &Pairings, now: SystemTime) -> Option<Said> {
        if pairings.paired_with(self.origin.machine(), now) {
            return None;
        }
        Some(no_longer_paired(&self.origin, self.turning.strings()))
    }
}

/// A refusal by this machine's grants, worded for a verb that came from
/// another machine — or handed back as it was, for everything else.
///
/// The value is not re-decided: what is refused was decided by the grants and
/// stays refused. What changes is the sentence, for the two refusals whose
/// local wording sends the person who asked to fix something on the wrong
/// machine. The rest — a change offered where a read was expected, a path that
/// really leads elsewhere, a machine with no agent — say what is true on any
/// machine and are left alone.
pub(crate) fn worded_here(refused: Refused, origin: Option<&Origin>, strings: &Strings) -> Refused {
    let Some(origin) = origin else {
        return refused;
    };
    let said = match refused.why() {
        NotAuthorised::NotGranted(why) => said_here(why, origin, strings),
        NotAuthorised::NotGrantedElsewhere(_) | NotAuthorised::ChangeWaits { .. } => None,
    };
    match said {
        Some(said) => Refused::worded_elsewhere(refused.call().clone(), said),
        None => refused,
    }
}

/// The same, for a change refused before anybody was asked.
pub(crate) fn proposal_worded_here(
    why: ProposalError,
    origin: Option<&Origin>,
    strings: &Strings,
) -> ProposalError {
    let Some(origin) = origin else {
        return why;
    };
    match &why {
        ProposalError::NotGranted(not_granted) => {
            said_here(not_granted, origin, strings).map_or(why, ProposalError::NotGrantedElsewhere)
        }
        ProposalError::NotGrantedElsewhere(_)
        | ProposalError::ReadDoesNotWait { .. }
        | ProposalError::NoTime
        | ProposalError::NoEnd => why,
    }
}

/// The grants' refusal, said for the machine that was asked.
///
/// `None` for a machine with no agent at all, whose sentence is about this
/// machine and right as it stands.
fn said_here(why: &NotGranted, origin: &Origin, strings: &Strings) -> Option<Said> {
    let machine = Filling::of("machine", origin.called());
    match why {
        NotGranted::Never { wanted, .. } => Some(strings.say(
            &words::NOT_GRANTED_HERE.key(),
            &wanted.fills("wanted", machine, strings),
        )),
        NotGranted::Lapsed { reach, wanted, .. } => Some(
            strings.say(
                &words::GRANT_HERE_EXPIRED.key(),
                &wanted
                    .fills("wanted", machine, strings)
                    .and_said("reach", &reach.said(strings)),
            ),
        ),
        NotGranted::NoAgent { .. } => None,
    }
}

/// The sentence for a pairing that ended while a turn was under way.
fn no_longer_paired(origin: &Origin, strings: &Strings) -> Said {
    strings.say(
        &words::NO_LONGER_PAIRED.key(),
        &Filling::of("machine", origin.called()),
    )
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::path::Path;

    use alo_capability::{Ask, Authorised, Grant, Reach};
    use alo_egress::Indicator;
    use alo_files::OnThisMachine;
    use alo_nearby::{
        Deliberating, Keying, MachineId, MayAskIts, Pairing, Proof, Proposal, Seen, Side,
    };
    use alo_record::{Happened, Record};

    use super::*;
    use crate::testing::{NothingIsBounded, files, hour, in_english, listing, noon};

    /// The machine down the corridor.
    fn the_reception() -> MachineId {
        MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap()
    }

    /// This machine.
    fn here() -> MachineId {
        MachineId::read("aaaabbbbccccddddeeeeffff00001111").unwrap()
    }

    /// The pairing with the reception machine, as each side keeps it: the
    /// reception's row first, this machine's second, one key on both.
    fn paired_both_ways() -> (Pairing, Pairing) {
        let at_reception = Keying::fresh().unwrap();
        let proposal = Proposal::checked(
            the_reception(),
            here(),
            &[MayAskIts::Models],
            hour(),
            at_reception.offer().clone(),
        )
        .unwrap();
        let on_here = Deliberating::asked(proposal.clone(), Keying::fresh().unwrap());
        let on_reception = Deliberating::asking(proposal, at_reception)
            .unwrap()
            .answered_with(on_here.answered().unwrap().clone())
            .unwrap();
        (
            on_reception
                .agreed_at(Side::TheOneAsking)
                .agreed_at(Side::TheOneAsked)
                .agreed(noon())
                .unwrap(),
            on_here
                .agreed_at(Side::TheOneAsking)
                .agreed_at(Side::TheOneAsked)
                .agreed(noon())
                .unwrap(),
        )
    }

    /// The reception machine, as a place a verb arrives from — proven, with
    /// a proof made on its own row.
    fn origin() -> Origin {
        let (on_reception, on_here) = paired_both_ways();
        let mut pairings = Pairings::none();
        pairings.keep(on_here);
        let proof = Proof::made(&on_reception, &the_reception(), b"a turn", noon());
        Origin::proven(
            &pairings,
            &here(),
            &proof,
            b"a turn",
            "the reception machine",
            noon(),
            &mut Seen::nothing(),
        )
        .unwrap()
    }

    /// The grants' own refusal of a folder never granted.
    fn never() -> Refused {
        Authorised::read(
            &listing(Path::new("/home/anna/Invoices")),
            &files(),
            &Grants::default(),
            noon(),
        )
        .unwrap_err()
    }

    /// **A refusal from another machine says the grant was not made here**, in
    /// the name the person gave the machine, and does not send anybody to a
    /// folder picker.
    #[test]
    fn a_grant_never_made_is_said_to_have_not_been_made_here() {
        let strings = in_english();
        let here = worded_here(never(), Some(&origin()), &strings);
        assert!(matches!(here.why(), NotAuthorised::NotGrantedElsewhere(_)));
        let said = here.said(&strings);
        assert!(said.text().contains("the reception machine"), "{said}");
        assert!(said.text().contains("on this machine"), "{said}");
        assert!(said.text().contains("/home/anna/Invoices"), "{said}");
        assert!(!said.text().contains("picking a folder"), "{said}");
        assert!(!said.text().contains("machine:"), "{said}");
        assert_eq!(here.call(), never().call());
    }

    /// **A grant that ran out is said to have run out here**, naming what it
    /// was over, and nothing granted elsewhere stands in for it.
    #[test]
    fn a_grant_that_expired_here_is_said_to_have_expired_here() {
        let strings = in_english();
        let mut grants = Grants::default();
        grants.grant(
            Grant::checked(
                "@files",
                Reach::Folder("/home/anna/Invoices".into()),
                noon(),
                hour(),
            )
            .unwrap(),
        );
        let lapsed = Authorised::read(
            &listing(Path::new("/home/anna/Invoices")),
            &files(),
            &grants,
            noon() + hour(),
        )
        .unwrap_err();
        assert!(lapsed.said(&strings).text().contains("grant it again"));

        let said = worded_here(lapsed, Some(&origin()), &strings).said(&strings);
        assert!(said.text().contains("has expired"), "{said}");
        assert!(said.text().contains("on this machine"), "{said}");
        assert!(said.text().contains("the reception machine"), "{said}");
        assert!(said.text().contains("and everything in it"), "{said}");
        assert!(!said.text().contains("grant it again"), "{said}");
    }

    /// **A local turn's refusals are left exactly as they were.** No origin, no
    /// re-wording: this file changes nothing about a verb an agent on this
    /// machine asked for.
    #[test]
    fn a_local_refusal_is_not_touched() {
        let strings = in_english();
        let refused = never();
        let same = worded_here(refused.clone(), None, &strings);
        assert_eq!(same, refused);
        let why = ProposalError::NotGranted(NotGranted::Never {
            agent: "@files".to_owned(),
            wanted: Ask::path("/home/anna/Invoices"),
        });
        assert_eq!(proposal_worded_here(why.clone(), None, &strings), why);
    }

    /// The refusals that say what is true on any machine are left alone even
    /// on a remote turn: a change offered as a read, a question that stands
    /// for no time, and a refusal somebody else already worded.
    #[test]
    fn a_refusal_that_is_true_on_any_machine_keeps_its_words() {
        let strings = in_english();
        let theirs = strings.say(
            &alo_strings::Key::named("files.refused.really-leads-elsewhere").unwrap(),
            &Filling::nothing(),
        );
        let elsewhere = Refused::worded_elsewhere(never().call().clone(), theirs.clone());
        assert_eq!(
            worded_here(elsewhere.clone(), Some(&origin()), &strings),
            elsewhere
        );

        for why in [
            ProposalError::ReadDoesNotWait {
                verb: "list_folder".to_owned(),
            },
            ProposalError::NoTime,
            ProposalError::NoEnd,
            ProposalError::NotGrantedElsewhere(theirs),
        ] {
            assert_eq!(
                proposal_worded_here(why.clone(), Some(&origin()), &strings),
                why
            );
        }
    }

    /// A change refused before anybody was asked is re-worded through the
    /// same door as one refused at the moment, so the two roads a remote
    /// verb can be refused on read the same sentence.
    #[test]
    fn a_change_never_asked_about_is_worded_the_same_way() {
        let strings = in_english();
        let why = ProposalError::NotGranted(NotGranted::Never {
            agent: origin().principal(),
            wanted: Ask::path("/home/anna/Invoices"),
        });
        let here = proposal_worded_here(why, Some(&origin()), &strings);
        assert!(matches!(here, ProposalError::NotGrantedElsewhere(_)));
        assert_eq!(
            here.said(&strings).text(),
            worded_here(never(), Some(&origin()), &strings)
                .said(&strings)
                .text()
        );
    }

    /// The sentence for a pairing that ended names the machine and says that
    /// nothing more it asks for is considered.
    #[test]
    fn a_pairing_that_ended_is_said_in_the_machines_name() {
        let said = no_longer_paired(&origin(), &in_english());
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("no longer paired"), "{said}");
        assert!(said.text().contains("the reception machine"), "{said}");
    }

    /// **An answer going back leaves under this machine's indicator**, as the
    /// origin's principal sending to the machine by the name the person gave
    /// it; once returned it is written down as having left, stamped with
    /// where the verb came from, and the line is off.
    #[test]
    fn an_answer_going_back_is_shown_leaving_and_written_down_as_left() {
        let strings = in_english();
        let mut indicator = Indicator::default();
        let mut record = Record::default();
        let mut bounding = NothingIsBounded;
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();
        let origin = origin();
        let mut grants = Grants::default();
        let mut arriving =
            Arriving::beginning(&origin, hour(), noon(), &mut grants, &mut machine).unwrap();

        let departing = arriving
            .departing(&EgressPolicy::InTheBuilding, noon())
            .unwrap();
        assert_eq!(departing.agent(), arriving.grantee());
        assert_eq!(departing.why(), Why::Sending);
        assert_eq!(
            departing.destination(),
            &Destination::paired("the reception machine").unwrap()
        );
        assert_eq!(arriving.turning().showing().showing().len(), 1);
        assert!(
            arriving
                .turning()
                .showing()
                .showing()
                .first()
                .unwrap()
                .said(&strings)
                .text()
                .contains("the reception machine")
        );

        arriving.returned(departing).unwrap();
        assert!(arriving.turning().showing().is_quiet());
        let machine = arriving.ended(&mut grants);
        assert!(machine.showing().is_quiet());

        assert_eq!(record.len(), 1);
        let entry = record.everything().next().unwrap();
        assert!(entry.happened().caused_egress());
        assert_eq!(
            entry.happened().destination(),
            Some(&Destination::paired("the reception machine").unwrap())
        );
        assert!(
            entry
                .origin()
                .is_some_and(|from| from.is("the reception machine"))
        );
    }

    /// **A rule that says nothing leaves holds the answer back**, written
    /// down as such in the rule's own words with the origin named, and
    /// nothing goes on the indicator.
    #[test]
    fn a_rule_that_says_nothing_leaves_holds_the_answer_back_and_writes_it_down() {
        let strings = in_english();
        let mut indicator = Indicator::default();
        let mut record = Record::default();
        let mut bounding = NothingIsBounded;
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();
        let origin = origin();
        let mut grants = Grants::default();
        let mut arriving =
            Arriving::beginning(&origin, hour(), noon(), &mut grants, &mut machine).unwrap();

        let held = arriving
            .departing(&EgressPolicy::NothingLeaves, noon())
            .unwrap_err();
        assert!(matches!(held, NoAnswer::HeldBack(_)), "{held:?}");
        assert!(arriving.turning().showing().is_quiet());
        assert!(!arriving.is_closed());
        let _ = arriving.ended(&mut grants);

        assert_eq!(record.len(), 1);
        let entry = record.everything().next().unwrap();
        assert!(!entry.happened().caused_egress());
        assert!(matches!(entry.happened(), Happened::HeldBack { .. }));
        assert!(
            entry
                .origin()
                .is_some_and(|from| from.is("the reception machine"))
        );
    }

    /// A turn handed back after ending is the same machine, and the next
    /// remote turn begins on it.
    #[test]
    fn a_turn_that_ended_hands_the_machine_back_for_the_next_one() {
        let strings = in_english();
        let mut indicator = Indicator::default();
        let mut record = Record::default();
        let mut bounding = NothingIsBounded;
        let mut machine = Machine::carrying_out_file_verbs(
            &strings,
            &OnThisMachine,
            &mut bounding,
            &mut indicator,
            &mut record,
        )
        .unwrap();
        let origin = origin();
        let mut grants = Grants::default();
        let first =
            Arriving::beginning(&origin, hour(), noon(), &mut grants, &mut machine).unwrap();
        let machine = first.ended(&mut grants);
        let second = Arriving::beginning(&origin, hour(), noon(), &mut grants, machine).unwrap();
        assert!(!second.is_closed());
        second.ending(&mut grants);
        assert!(record.is_empty());
    }
}
