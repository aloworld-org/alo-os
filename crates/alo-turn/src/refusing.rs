//! What comes back instead, when a turn did not do what it was asked.
//!
//! Eight things, and they are eight rather than one because a caller has a
//! different thing to do about each. Two of them mean the capability model
//! stopped something, three mean the machine could not, one means a question was
//! answered by nobody, and two mean this machine is no longer keeping evidence.
//!
//! | What came back | What happened | What is written down |
//! |---|---|---|
//! | [`NotDone::TurnedAway`] | It never became a call: a verb that is not on the list, or an argument that did not survive validation | `turned away`, with no arguments kept |
//! | [`NotDone::NeverAsked`] | A change the grants already refused, or a read offered for approval — nobody was interrupted | `stopped`, before anybody was asked |
//! | [`NotDone::NotAnswered`] | A number that is not waiting, or a question that stood too long | nothing: see below |
//! | [`NotDone::Refused`] | The grants said no at the moment it would have run | `stopped`, at the moment |
//! | [`NotDone::MachineCouldNot`] | Everything said yes and the disk did not | `ran` — it was attempted |
//! | [`NotDone::NotBounded`] | There was no boundary to run the work inside, so nothing ran | nothing: see below |
//! | [`NotDone::NotRecorded`] | What happened could not be written down | nothing, which is the problem |
//! | [`NotDone::TurnClosed`] | Something earlier in this turn could not be written down | nothing |
//!
//! # None of these is worded here
//!
//! Every one of them carries the value whoever made it handed over, and
//! [`NotDone::said`] asks *that* for a sentence. A turn is the last crate in a
//! chain of five and the one with the least right to describe what the others
//! decided: the call that did not form is `alo-capability`'s to explain, the
//! disk is `alo-files`', the record is `alo-keeping`'. The two sentences with
//! nowhere else to come from are [`NotDone::TurnClosed`] and
//! [`NotDone::NotBounded`], and they are this crate's only strings — the second
//! because a boundary is a mechanism that says nothing to anybody, and
//! [`crate::words`] is the argument for it.
//!
//! # Why a question nobody answered is not written down
//!
//! [`NotDone::NotAnswered`] is the first of two refusals here with no entry
//! behind it, and the absence is deliberate. A proposal is not a thing that
//! happened — it
//! is a thing that was put to somebody — and what the record keeps is its
//! outcome: it ran, or the person declined it, or the grants refused it at the
//! moment. A person who answers *no* has answered, and that is
//! `Entry::declined`. A person who answers nothing has not acted at all, and an
//! entry saying so would be the record starting to keep what the person did
//! rather than what the agent did, which is ADR 0001 §4's watched context
//! arriving through the back door (item 17 refused the same thing about turning
//! an agent off).
//!
//! # And why a turn that could not be bounded is not written down either
//!
//! [`NotDone::NotBounded`] is the second, and it is the same argument about the
//! other end of the journey. A boundary that could not be imposed means nothing
//! was attempted: no control group was made, no entry reached the kernel, no
//! syscall was issued and no grant was consulted. There is nothing for the
//! record to say that would be true — *it ran* is false, and every way this
//! crate has of saying *it was stopped* says the capability model stopped it,
//! which would tell a security review that the grants refused something they
//! were never asked about. That is `alo-files`' rule about a full disk, met at
//! the one place where even the disk was never reached.
//!
//! What is not lost is the knowing. The person is told, in their own language;
//! the reason is English and goes to whoever administers the machine
//! ([`crate::NoBoundary::why`]); and a thread that could not be brought back out
//! is a thing the service stops over rather than logs.

use alo_capability::{AnswerError, CallError, ProposalError, Refused};
use alo_files::Failed;
use alo_keeping::NotKept;
use alo_strings::{Filling, Said, Strings};

use crate::unbounded::NoBoundary;
use crate::went_wrong_asking::WentWrongAsking;
use crate::went_wrong_installing::WentWrongInstalling;
use crate::went_wrong_looking::WentWrongLooking;
use crate::words;

/// Why a turn did not do what it was asked.
///
/// **No `Display`**, like every refusal a person reads in this workspace: the
/// road to words is [`NotDone::said`], which takes the vocabulary that person
/// reads and answers with something that says whether anybody translated it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotDone {
    /// It never became a call — a verb that is not on the list, or an argument
    /// that did not survive validation.
    TurnedAway(CallError),
    /// A change that was never put to a person: one the grants already refused,
    /// or a read offered where only a change may wait.
    NeverAsked(ProposalError),
    /// A number that is not waiting for an answer, or a question that stood too
    /// long. Nothing was written down — see this module's documentation.
    NotAnswered(AnswerError),
    /// The capability model said no at the moment it would have run: the
    /// grants, asked last, or a path that really leads somewhere nobody
    /// granted.
    Refused(Refused),
    /// Everything permitted it and the machine could not.
    ///
    /// A different fact from every refusal above, and kept apart from them for
    /// the reason `alo-files` keeps `Failed` and `Refused` apart: a record that
    /// called a full disk a refusal would tell a security review the grants
    /// stopped something they did not.
    MachineCouldNot(Failed),
    /// There was no boundary to run the work inside, so nothing ran.
    ///
    /// Not the grants and not the disk: ADR 0015's *a turn whose boundary
    /// cannot be applied does not run*, which is the machine being unable to
    /// keep its own guarantee rather than the capability model keeping it. It
    /// is written down as the machine's own refusal
    /// (`alo_record::Entry::not_bounded`) before it is answered, and a
    /// [`NoBoundary::a_thread_is_still_inside`] is a service that stops.
    NotBounded(NoBoundary),
    /// What happened could not be written down.
    ///
    /// The turn is closed by this, so nothing else will be done under it. For a
    /// read, nothing was handed back that has no record; for a change, the
    /// change has already happened and there is now no evidence of it, which is
    /// why a daemon meeting this has a machine to stop rather than an error to
    /// log.
    NotRecorded(NotKept),
    /// Something earlier in this turn could not be written down, so nothing
    /// more will be done under it.
    TurnClosed,
    /// An installation this machine would not or could not do.
    ///
    /// Kept apart from [`Self::Refused`] and [`Self::MachineCouldNot`] for the
    /// reason those two are kept apart from each other: `alo-software` draws
    /// its own line between a place an organisation does not permit and a tool
    /// that did not answer, it words both itself, and a record that rewrote
    /// either as one of the three above would be telling a reader the file
    /// grants stopped something they say nothing about.
    CouldNotInstall(alo_software::NotDone),
    /// An installation alo OS could not carry out through a fault of its own.
    ///
    /// The installing half of [`Self::NotBounded`]: not the capability model
    /// and not the world, but this machine unable to keep its own working
    /// consistent. [`WentWrongInstalling`] holds the technical reason in
    /// English for whoever administers the machine, and the sentence a person
    /// reads carries none of it.
    CouldNotInstallHere(WentWrongInstalling),
    /// A look at the catalogue alo OS could not answer through a fault of its
    /// own.
    ///
    /// [`Self::CouldNotInstallHere`] for the read beside the installation, and
    /// [`WentWrongLooking`]'s header says what is different: this verb has no
    /// refusals that are not faults, because nothing in the world can stop it
    /// — it parses a string compiled into this binary.
    CouldNotLookAtTheCatalogue(WentWrongLooking),
    /// A question of the record alo OS could not answer.
    ///
    /// Unlike [`Self::CouldNotLookAtTheCatalogue`] beside it, **one of this
    /// one's two causes is not a fault**: a record kept on a disk is a file,
    /// and a file can be unreadable. [`WentWrongAsking`]'s header says which.
    CouldNotAskTheRecord(WentWrongAsking),
}

impl NotDone {
    /// What this says, in the language the person reads.
    ///
    /// Seven of the ten hand the question straight to whoever made the refusal,
    /// so what a person is told about a call that did not form is the same
    /// sentence wherever in alo OS the call was made. The other three are this
    /// crate's own words for this crate's own facts: a turn that closed, a
    /// boundary that could not be applied, and an installation this machine
    /// could not carry out through a fault here.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::TurnedAway(why) => why.said(strings),
            Self::NeverAsked(why) => why.said(strings),
            Self::NotAnswered(why) => why.said(strings),
            Self::Refused(refused) => refused.said(strings),
            Self::MachineCouldNot(failed) => failed.said(strings),
            Self::NotBounded(why) => why.said(strings),
            Self::NotRecorded(why) => why.said(strings),
            Self::TurnClosed => strings.say(&words::TURN_CLOSED.key(), &Filling::nothing()),
            Self::CouldNotInstall(refused) => refused.said(strings),
            Self::CouldNotInstallHere(_) => {
                strings.say(&words::COULD_NOT_INSTALL_HERE.key(), &Filling::nothing())
            }
            Self::CouldNotLookAtTheCatalogue(_) => strings.say(
                &words::COULD_NOT_LOOK_AT_THE_CATALOGUE.key(),
                &Filling::nothing(),
            ),
            Self::CouldNotAskTheRecord(_) => {
                strings.say(&words::COULD_NOT_ASK_THE_RECORD.key(), &Filling::nothing())
            }
        }
    }

    /// Whether the capability model stopped this.
    ///
    /// True of the three refusals it makes and false of everything else, so a
    /// caller can tell *the agent was not allowed to* from *this machine could
    /// not* without matching on seven variants. A closed turn is not a refusal
    /// and neither is a full disk.
    #[must_use]
    pub fn was_refused(&self) -> bool {
        matches!(
            self,
            Self::TurnedAway(_) | Self::NeverAsked(_) | Self::Refused(_) | Self::CouldNotInstall(_)
        )
    }

    /// Whether this turn is over because the record could not be written.
    ///
    /// True of both halves of that one fact — the moment it happened and every
    /// door afterwards — because what a caller does about them is the same
    /// thing, and it is not *try again*.
    #[must_use]
    pub fn is_the_end_of_the_turn(&self) -> bool {
        matches!(self, Self::NotRecorded(_) | Self::TurnClosed)
    }
}

impl From<CallError> for NotDone {
    fn from(why: CallError) -> Self {
        Self::TurnedAway(why)
    }
}

impl From<ProposalError> for NotDone {
    fn from(why: ProposalError) -> Self {
        Self::NeverAsked(why)
    }
}

impl From<AnswerError> for NotDone {
    fn from(why: AnswerError) -> Self {
        Self::NotAnswered(why)
    }
}

impl From<Refused> for NotDone {
    fn from(refused: Refused) -> Self {
        Self::Refused(refused)
    }
}

impl From<NotKept> for NotDone {
    fn from(why: NotKept) -> Self {
        Self::NotRecorded(why)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{everything_that_can_come_back, in_english, translated};

    /// **Every one of the seven says something**, in the language the person
    /// reads, and none of them reaches somebody as a key.
    #[test]
    fn every_way_a_turn_can_refuse_says_something() {
        let strings = in_english();
        for not_done in everything_that_can_come_back() {
            let said = not_done.said(&strings);
            assert!(!said.is_a_bug(), "{not_done:?}: {said}");
            assert!(!said.text().is_empty(), "{not_done:?}");
        }
    }

    /// **Seven of the ten are somebody else's words**, and this is the test
    /// that says so: with only this crate's own vocabulary loaded, the three
    /// sentences that are ours read and the rest are keys nothing declares.
    ///
    /// The three are a turn that closed, a boundary that could not be applied,
    /// and an installation alo OS could not carry out through a fault of its
    /// own. Each is ours for the same reason: it is a mechanism failing and
    /// says nothing to anybody else — [`crate::words`] is the argument. The
    /// third arrived with the second verb family on 2026-10-13, and this test
    /// is what noticed, which is what it is for.
    ///
    /// **The fourth arrived on 2026-10-17** with the read beside the
    /// installation, and this test noticed again: alo OS unable to read its own
    /// shipped catalogue is a mechanism failing and says nothing to anybody
    /// else, which is the same argument the third makes.
    #[test]
    fn the_only_sentences_this_crate_says_of_its_own_are_its_own_four() {
        let ours = Strings::of(words::turn_words().unwrap());
        let mut said_by_us = 0;
        for not_done in everything_that_can_come_back() {
            if !not_done.said(&ours).is_a_bug() {
                said_by_us += 1;
            }
        }
        assert_eq!(
            said_by_us, 4,
            "this crate has started saying something somebody else already says"
        );
        assert!(!NotDone::TurnClosed.said(&ours).is_a_bug());
        assert!(
            !NotDone::NotBounded(crate::NoBoundary::because("no kernel".to_owned()))
                .said(&ours)
                .is_a_bug()
        );
        assert!(
            !NotDone::CouldNotInstallHere(crate::WentWrongInstalling::NotAnInstallation(
                alo_software::NotAnInstallation::NotApproved,
            ))
            .said(&ours)
            .is_a_bug()
        );
    }

    /// The capability model saying no and the machine not managing it are
    /// different facts, and a caller can tell them apart without matching on
    /// every variant there is.
    #[test]
    fn a_refusal_and_a_machine_that_could_not_are_different_answers() {
        for not_done in everything_that_can_come_back() {
            let refused = not_done.was_refused();
            let over = not_done.is_the_end_of_the_turn();
            assert!(
                !(refused && over),
                "{not_done:?} is both a refusal and the end of the turn"
            );
            match not_done {
                NotDone::TurnedAway(_)
                | NotDone::NeverAsked(_)
                | NotDone::Refused(_)
                // A place an organisation does not permit is the capability
                // model saying no, as much as a folder nobody granted is.
                | NotDone::CouldNotInstall(_) => {
                    assert!(refused);
                }
                NotDone::NotRecorded(_) | NotDone::TurnClosed => assert!(over),
                NotDone::NotAnswered(_)
                | NotDone::MachineCouldNot(_)
                | NotDone::NotBounded(_)
                // alo OS contradicting itself: not the grants, not the world,
                // and not the end of the turn either.
                | NotDone::CouldNotInstallHere(_)
                // And alo OS unable to read its own shipped catalogue, which
                // is the same kind of fact one verb along.
                | NotDone::CouldNotLookAtTheCatalogue(_)
                | NotDone::CouldNotAskTheRecord(_) => {
                    assert!(!refused && !over);
                }
            }
        }
    }

    /// And the closed turn is translated like everything else — it is read by
    /// the person whose machine has stopped keeping evidence, which is the last
    /// moment to hand somebody a sentence in a language they do not read.
    #[test]
    fn the_sentence_this_crate_says_is_one_a_translator_can_move() {
        let german = translated(&[(
            words::TURN_CLOSED,
            "dieser Vorgang wurde beendet, weil nicht aufgezeichnet werden konnte, was geschehen \
             ist",
        )]);
        let said = NotDone::TurnClosed.said(&german);
        assert!(said.is_translated(), "{said}");
        assert!(said.text().starts_with("dieser Vorgang"), "{said}");
    }
}
