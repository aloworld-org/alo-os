//! What comes back instead, when a turn did not do what it was asked.
//!
//! Seven things, and they are seven rather than one because a caller has a
//! different thing to do about each. Two of them mean the capability model
//! stopped something, two mean the machine could not, one means a question was
//! answered by nobody, and two mean this machine is no longer keeping evidence.
//!
//! | What came back | What happened | What is written down |
//! |---|---|---|
//! | [`NotDone::TurnedAway`] | It never became a call: a verb that is not on the list, or an argument that did not survive validation | `turned away`, with no arguments kept |
//! | [`NotDone::NeverAsked`] | A change the grants already refused, or a read offered for approval — nobody was interrupted | `stopped`, before anybody was asked |
//! | [`NotDone::NotAnswered`] | A number that is not waiting, or a question that stood too long | nothing: see below |
//! | [`NotDone::Refused`] | The grants said no at the moment it would have run | `stopped`, at the moment |
//! | [`NotDone::MachineCouldNot`] | Everything said yes and the disk did not | `ran` — it was attempted |
//! | [`NotDone::NotRecorded`] | What happened could not be written down | nothing, which is the problem |
//! | [`NotDone::TurnClosed`] | Something earlier in this turn could not be written down | nothing |
//!
//! # None of these is worded here
//!
//! Every one of them carries the value whoever made it handed over, and
//! [`NotDone::said`] asks *that* for a sentence. A turn is the last crate in a
//! chain of five and the one with the least right to describe what the others
//! decided: the call that did not form is `alo-capability`'s to explain, the
//! disk is `alo-files`', the record is `alo-keeping`'. The one sentence with
//! nowhere else to come from is [`NotDone::TurnClosed`], and it is this crate's
//! only string.
//!
//! # Why a question nobody answered is not written down
//!
//! [`NotDone::NotAnswered`] is the one refusal here with no entry behind it,
//! and the absence is deliberate. A proposal is not a thing that happened — it
//! is a thing that was put to somebody — and what the record keeps is its
//! outcome: it ran, or the person declined it, or the grants refused it at the
//! moment. A person who answers *no* has answered, and that is
//! `Entry::declined`. A person who answers nothing has not acted at all, and an
//! entry saying so would be the record starting to keep what the person did
//! rather than what the agent did, which is ADR 0001 §4's watched context
//! arriving through the back door (item 17 refused the same thing about turning
//! an agent off).

use alo_capability::{AnswerError, CallError, ProposalError, Refused};
use alo_files::Failed;
use alo_keeping::NotKept;
use alo_strings::{Filling, Said, Strings};

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
}

impl NotDone {
    /// What this says, in the language the person reads.
    ///
    /// Six of the seven hand the question straight to whoever made the refusal,
    /// so what a person is told about a call that did not form is the same
    /// sentence wherever in alo OS the call was made.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        match self {
            Self::TurnedAway(why) => why.said(strings),
            Self::NeverAsked(why) => why.said(strings),
            Self::NotAnswered(why) => why.said(strings),
            Self::Refused(refused) => refused.said(strings),
            Self::MachineCouldNot(failed) => failed.said(strings),
            Self::NotRecorded(why) => why.said(strings),
            Self::TurnClosed => strings.say(&words::TURN_CLOSED.key(), &Filling::nothing()),
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
            Self::TurnedAway(_) | Self::NeverAsked(_) | Self::Refused(_)
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

    /// **Six of the seven are somebody else's words**, and this is the test
    /// that says so: with only this crate's own vocabulary loaded, the one
    /// sentence that is ours reads and the rest are keys nothing declares.
    #[test]
    fn the_only_sentence_this_crate_says_of_its_own_is_the_closed_turn() {
        let ours = Strings::of(words::turn_words().unwrap());
        let mut said_by_us = 0;
        for not_done in everything_that_can_come_back() {
            if !not_done.said(&ours).is_a_bug() {
                said_by_us += 1;
            }
        }
        assert_eq!(
            said_by_us, 1,
            "this crate has started saying something somebody else already says"
        );
        assert!(!NotDone::TurnClosed.said(&ours).is_a_bug());
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
                NotDone::TurnedAway(_) | NotDone::NeverAsked(_) | NotDone::Refused(_) => {
                    assert!(refused);
                }
                NotDone::NotRecorded(_) | NotDone::TurnClosed => assert!(over),
                NotDone::NotAnswered(_) | NotDone::MachineCouldNot(_) => {
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
