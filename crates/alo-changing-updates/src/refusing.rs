//! What a person reads when an update was applied or this machine went back,
//! and when it did not.
//!
//! The broker answers in one word from a closed list and has no person in front
//! of it; this is where that word becomes words, in the reader's language. Two
//! of the four answers are sentences `alo-keeping-up` already has, and they are
//! **said** here rather than written again: a person who is told *the update
//! could not be prepared* should read the same line whether the preparing was
//! refused before the door or behind it.

use alo_broker::{Answer, SystemVerb};
use alo_keeping_up::words as keeping_up;
use alo_record::AtTheBroker;
use alo_strings::{Filling, Said, Strings};

use crate::wanted::Change;
use crate::words;

/// Why this machine's system was not changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotChanged {
    /// What was handed over was not an approved change to this machine's
    /// system.
    NotAnUpdateChange,
    /// The approval was refused where the change is made.
    ApprovalNotAccepted,
    /// It was accepted, and the machine could not prepare it.
    ///
    /// **`alo-keeping-up`'s own sentence**, said here rather than written
    /// again: *the update could not be prepared, so nothing was changed*, or
    /// its counterpart for going back. Which of the two depends on which change
    /// was asked for, so the refusal carries it.
    CouldNotPrepare(Change),
    /// The machine has stopped writing down changes, so it makes none.
    NotBeingKept,
    /// Nothing was there to make the change.
    NothingMakesChanges,
}

impl NotChanged {
    /// What a person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let word = match self {
            Self::NotAnUpdateChange => words::NOT_AN_UPDATE_CHANGE,
            Self::ApprovalNotAccepted => words::APPROVAL_NOT_ACCEPTED,
            Self::CouldNotPrepare(Change::Apply) => keeping_up::NOT_PREPARED,
            Self::CouldNotPrepare(Change::GoBack) => keeping_up::GOING_BACK_NOT_PREPARED,
            Self::NotBeingKept => words::NOT_BEING_KEPT,
            Self::NothingMakesChanges => words::NOTHING_MAKES_CHANGES,
        };
        strings.say(&word.key(), &Filling::nothing())
    }

    /// What the broker's answer to this change means.
    ///
    /// # Errors
    /// The refusal, for every answer but `carried`.
    pub fn from_the_brokers(answer: Answer, change: Change) -> Result<(), Self> {
        match answer {
            Answer::Carried => Ok(()),
            Answer::NotKept => Err(Self::NotBeingKept),
            Answer::Refused(AtTheBroker::NotCarried) => Err(Self::CouldNotPrepare(change)),
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

/// What a person reads once the change was carried out.
///
/// Both are `alo-keeping-up`'s: nothing has happened to the machine yet, and
/// what a person needs to read is the promise that it will happen at a restart
/// they make and that their files stay as they are. Nothing, for a verb that is
/// not one of the updates'.
#[must_use]
pub fn changed_said(verb: &SystemVerb, strings: &Strings) -> Option<Said> {
    let word = match Change::of(verb)? {
        Change::Apply => keeping_up::WAITING_FOR_THE_RESTART,
        Change::GoBack => keeping_up::GOING_BACK_AT_THE_RESTART,
    };
    Some(strings.say(&word.key(), &Filling::nothing()))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_broker::Identity;
    use alo_strings::Vocabulary;

    /// This crate's own sentences beside the ones it borrows, which is every
    /// sentence it can say and no more.
    fn in_english() -> Strings {
        let mut vocabulary = Vocabulary::empty();
        words::declare_into(&mut vocabulary).unwrap();
        alo_keeping_up::declare_into(&mut vocabulary).unwrap();
        Strings::of(vocabulary)
    }

    /// A build, as a digest of one.
    fn a_build() -> Identity {
        Identity::of_what_was_reported(b"a build")
    }

    /// **Every refusal is a sentence, not a key.**
    #[test]
    fn every_refusal_is_a_sentence() {
        let strings = in_english();
        for refusal in [
            NotChanged::NotAnUpdateChange,
            NotChanged::ApprovalNotAccepted,
            NotChanged::CouldNotPrepare(Change::Apply),
            NotChanged::CouldNotPrepare(Change::GoBack),
            NotChanged::NotBeingKept,
            NotChanged::NothingMakesChanges,
        ] {
            let said = refusal.said(&strings);
            assert!(!said.is_a_bug(), "{refusal:?}: {said}");
            assert!(said.unfilled().is_empty(), "{refusal:?}: {said}");
            assert!(
                said.text().to_lowercase().contains("nothing was changed")
                    || said
                        .text()
                        .to_lowercase()
                        .contains("nothing else was changed"),
                "{refusal:?}: {said}"
            );
        }
    }

    /// **Applying and going back are told apart when the preparing fails.**
    /// One says the update could not be prepared, the other says going back
    /// could not — and a person reading the wrong one of those would look for
    /// their machine in the wrong direction.
    #[test]
    fn a_failure_to_prepare_names_which_of_the_two_it_was() {
        let strings = in_english();
        let applying = NotChanged::CouldNotPrepare(Change::Apply).said(&strings);
        let going_back = NotChanged::CouldNotPrepare(Change::GoBack).said(&strings);
        assert_ne!(applying.text(), going_back.text());
        assert!(applying.text().contains("update"), "{applying}");
        assert!(going_back.text().contains("Going back"), "{going_back}");
    }

    /// **Only `carried` is carried out.** Every other answer the door can give
    /// is a refusal a person can read.
    #[test]
    fn only_carried_is_carried_out_and_every_other_answer_is_read() {
        assert_eq!(
            NotChanged::from_the_brokers(Answer::Carried, Change::Apply),
            Ok(())
        );
        assert_eq!(
            NotChanged::from_the_brokers(Answer::NotKept, Change::Apply),
            Err(NotChanged::NotBeingKept)
        );
        assert_eq!(
            NotChanged::from_the_brokers(Answer::Refused(AtTheBroker::NotCarried), Change::GoBack),
            Err(NotChanged::CouldNotPrepare(Change::GoBack))
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
                NotChanged::from_the_brokers(Answer::Refused(why), Change::Apply),
                Err(NotChanged::ApprovalNotAccepted),
                "{why:?}"
            );
        }
    }

    /// **What a person reads afterwards keeps the promise the machine makes**:
    /// it applies at a restart they make, and their files stay as they are.
    #[test]
    fn what_is_read_afterwards_keeps_the_promise() {
        let strings = in_english();
        let applied = changed_said(&Change::Apply.to(a_build()), &strings).unwrap();
        let went_back = changed_said(&Change::GoBack.to(a_build()), &strings).unwrap();
        for said in [&applied, &went_back] {
            assert!(!said.is_a_bug(), "{said}");
            assert!(said.text().contains("restart"), "{said}");
            assert!(said.text().contains("files"), "{said}");
        }
        assert_ne!(applied.text(), went_back.text());

        assert!(
            changed_said(&SystemVerb::MountDrive(a_build()), &strings).is_none(),
            "a storage verb said something about an update"
        );
    }
}
