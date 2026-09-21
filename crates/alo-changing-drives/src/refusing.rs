//! What a person reads when a drive was opened or finished with, and when it
//! was not.
//!
//! One sentence each, and each refusal carries only what its sentence fills in:
//! the name of the drive, as the drive gave it. The broker answers in one word
//! from a closed list and has no person in front of it; this is where that word
//! becomes words, in the reader's language.

use alo_broker::{Answer, SystemVerb};
use alo_drives::NotAnswering;
use alo_record::AtTheBroker;
use alo_strings::{Filling, Said, Strings};

use crate::wanted::Change;
use crate::words;

/// Why nothing was done to a drive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotChanged {
    /// What was handed over was not an approved change to a drive.
    NotADriveChange,
    /// The disk service could not be asked what is plugged in.
    DrivesNotAnswering,
    /// Nothing plugged in has this name.
    NonePluggedInCalled(String),
    /// More than one thing plugged in has this name.
    MoreThanOneCalled(String),
    /// It is one of the machine's own disks.
    PartOfThisMachine(String),
    /// There is nothing on it this machine recognises.
    NothingToOpen(String),
    /// It is open already.
    AlreadyOpen(String),
    /// The approval was refused where the change is made.
    ApprovalNotAccepted,
    /// It was accepted, and the machine could not finish it.
    CouldNotFinish(String),
    /// The machine has stopped writing down changes, so it makes none.
    NotBeingKept,
    /// Nothing was there to make the change.
    NothingMakesChanges,
}

impl From<NotAnswering> for NotChanged {
    fn from(_: NotAnswering) -> Self {
        Self::DrivesNotAnswering
    }
}

impl NotChanged {
    /// What a person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let (word, called) = match self {
            Self::NotADriveChange => (words::NOT_A_DRIVE_CHANGE, None),
            Self::DrivesNotAnswering => (words::DRIVES_NOT_ANSWERING, None),
            Self::NonePluggedInCalled(called) => (words::NONE_PLUGGED_IN_CALLED, Some(called)),
            Self::MoreThanOneCalled(called) => (words::MORE_THAN_ONE_CALLED, Some(called)),
            Self::PartOfThisMachine(called) => (words::PART_OF_THIS_MACHINE, Some(called)),
            Self::NothingToOpen(called) => (words::NOTHING_TO_OPEN, Some(called)),
            Self::AlreadyOpen(called) => (words::ALREADY_OPEN, Some(called)),
            Self::ApprovalNotAccepted => (words::APPROVAL_NOT_ACCEPTED, None),
            Self::CouldNotFinish(called) => (words::COULD_NOT_FINISH, Some(called)),
            Self::NotBeingKept => (words::NOT_BEING_KEPT, None),
            Self::NothingMakesChanges => (words::NOTHING_MAKES_CHANGES, None),
        };
        strings.say(&word.key(), &filled(called.map(String::as_str)))
    }

    /// What the broker's answer to a change to the drive called this means.
    ///
    /// # Errors
    /// The refusal, for every answer but `carried`.
    pub fn from_the_brokers(answer: Answer, called: &str) -> Result<(), Self> {
        match answer {
            Answer::Carried => Ok(()),
            Answer::NotKept => Err(Self::NotBeingKept),
            Answer::Refused(AtTheBroker::NotCarried) => {
                Err(Self::CouldNotFinish(called.to_owned()))
            }
            // Everything else the door refuses is about the approval, or about
            // who asked — which, for the one process that asks, is the same
            // fact: this approval did not make this change. Named one by one so
            // a reason added to the broker's list meets this line.
            Answer::Refused(
                AtTheBroker::NotTheAgentService
                | AtTheBroker::NotARequest
                | AtTheBroker::NotOneOfItsVerbs
                | AtTheBroker::NotApproved
                | AtTheBroker::ApprovalSpent
                | AtTheBroker::ApprovalLapsed,
            ) => Err(Self::ApprovalNotAccepted),
        }
    }
}

/// What a person reads once the drive called this was opened or finished with.
///
/// Nothing, for a verb that is not one of the drives': a surface with no
/// sentence says nothing rather than the wrong thing.
#[must_use]
pub fn changed_said(verb: &SystemVerb, called: &str, strings: &Strings) -> Option<Said> {
    let word = match Change::of(verb)? {
        Change::Open => words::READY,
        Change::FinishWith => words::SAFE_TO_UNPLUG,
    };
    Some(strings.say(&word.key(), &filled(Some(called))))
}

/// The filling for a sentence about the drive called this, or about none.
fn filled(called: Option<&str>) -> Filling {
    called.map_or_else(Filling::nothing, |called| Filling::of(words::DRIVE, called))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_broker::Identity;

    /// The machine's one vocabulary is assembled elsewhere; this crate's own is
    /// what its unit tests read.
    fn in_english() -> Strings {
        Strings::of(words::changing_drives_words().unwrap())
    }

    /// The drive a person plugged in.
    const A_STICK: &str = "Kingston-DataTraveler-1C1B";

    /// **Every refusal is a sentence, not a key**, and every one that names the
    /// drive fills the gap with it.
    #[test]
    fn every_refusal_is_a_sentence_and_names_the_drive_where_it_should() {
        let strings = in_english();
        let named = A_STICK.to_owned();
        for refusal in [
            NotChanged::NotADriveChange,
            NotChanged::DrivesNotAnswering,
            NotChanged::NonePluggedInCalled(named.clone()),
            NotChanged::MoreThanOneCalled(named.clone()),
            NotChanged::PartOfThisMachine(named.clone()),
            NotChanged::NothingToOpen(named.clone()),
            NotChanged::AlreadyOpen(named.clone()),
            NotChanged::ApprovalNotAccepted,
            NotChanged::CouldNotFinish(named.clone()),
            NotChanged::NotBeingKept,
            NotChanged::NothingMakesChanges,
        ] {
            let said = refusal.said(&strings);
            assert!(!said.is_a_bug(), "{refusal:?}: {said}");
            assert!(said.unfilled().is_empty(), "{refusal:?}: {said}");
        }
    }

    /// **A disk service that will not answer is one sentence**, and it is this
    /// machine's fault rather than the drive's.
    #[test]
    fn a_disk_service_that_will_not_answer_is_one_sentence() {
        assert_eq!(
            NotChanged::from(NotAnswering("the bus is not there".to_owned())),
            NotChanged::DrivesNotAnswering
        );
    }

    /// **Only `carried` is carried out.** Every other answer the door can give
    /// is a refusal a person can read, and the one that means *it was started
    /// and not finished* is the one that tells them not to unplug it yet.
    #[test]
    fn only_carried_is_carried_out_and_every_other_answer_is_read() {
        assert_eq!(
            NotChanged::from_the_brokers(Answer::Carried, A_STICK),
            Ok(())
        );
        assert_eq!(
            NotChanged::from_the_brokers(Answer::NotKept, A_STICK),
            Err(NotChanged::NotBeingKept)
        );
        assert_eq!(
            NotChanged::from_the_brokers(Answer::Refused(AtTheBroker::NotCarried), A_STICK),
            Err(NotChanged::CouldNotFinish(A_STICK.to_owned()))
        );
        for why in [
            AtTheBroker::NotTheAgentService,
            AtTheBroker::NotARequest,
            AtTheBroker::NotOneOfItsVerbs,
            AtTheBroker::NotApproved,
            AtTheBroker::ApprovalSpent,
            AtTheBroker::ApprovalLapsed,
        ] {
            assert_eq!(
                NotChanged::from_the_brokers(Answer::Refused(why), A_STICK),
                Err(NotChanged::ApprovalNotAccepted),
                "{why:?}"
            );
        }
    }

    /// **What a person reads afterwards is different for each of the two**, and
    /// a verb that is not one of the drives' is not spoken about at all.
    #[test]
    fn each_of_the_two_says_its_own_thing_and_no_other_verb_says_any_of_it() {
        let strings = in_english();
        let identity = Identity::of_what_was_reported(b"a stick");
        let opened = changed_said(&Change::Open.to(identity), A_STICK, &strings).unwrap();
        let finished = changed_said(&Change::FinishWith.to(identity), A_STICK, &strings).unwrap();
        assert!(opened.text().contains("ready"), "{opened}");
        assert!(finished.text().contains("safe to unplug"), "{finished}");
        assert!(opened.text().contains(A_STICK), "{opened}");
        assert_ne!(opened.text(), finished.text());

        assert!(
            changed_said(&SystemVerb::AddPrinter(identity), A_STICK, &strings).is_none(),
            "a printer verb said something about a drive"
        );
    }
}
