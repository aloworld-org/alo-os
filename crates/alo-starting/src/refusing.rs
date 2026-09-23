//! What a person reads when this machine's starting was changed, and when it
//! was not — for both changes on this road: *start Windows next time*, and
//! *start this system whenever nobody chooses*.
//!
//! The broker answers in one word from a closed list and has no person in front
//! of it; this is where that word becomes words, in the reader's language. The
//! refusals a surface can find out **before** the door — no Windows on this
//! machine, two of them, a choice that turns out not to start Windows — are
//! separate from the one the door gives back, because a person who is told
//! *there is no Windows here* can stop looking and a person told *it would not
//! be told* can try the way that always works.

use alo_broker::{Answer, SystemVerb};
use alo_record::AtTheBroker;
use alo_strings::{Filling, Said, Strings};

use crate::firmware::NotAnswering;
use crate::offered::the_system_named;
use crate::systems::System;
use crate::wanted::Change;
use crate::words;

/// Why what this machine starts was not changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotChanged {
    /// What was handed over was not an approved change to which system starts.
    NotAStartingChange,
    /// This machine could not be asked what it can start.
    NotAnswering,
    /// Nothing this machine can start is Windows.
    NoWindowsHere,
    /// More than one thing this machine can start is Windows.
    MoreThanOneWindows,
    /// What was approved turned out not to start Windows.
    DoesNotStartWindows,
    /// The approval was refused where the change is made.
    ApprovalNotAccepted,
    /// It was accepted, and this machine would not be told.
    WouldNotBeTold,
    /// It was accepted, and this machine would not keep which system it starts.
    NotRemembered,
    /// The machine has stopped writing down changes, so it makes none.
    NotBeingKept,
    /// Nothing was there to make the change.
    NothingMakesChanges,
}

impl From<NotAnswering> for NotChanged {
    fn from(_: NotAnswering) -> Self {
        Self::NotAnswering
    }
}

impl NotChanged {
    /// What a person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let word = match self {
            Self::NotAStartingChange => words::NOT_A_STARTING_CHANGE,
            Self::NotAnswering => words::NOT_ANSWERING,
            Self::NoWindowsHere => words::NO_WINDOWS_HERE,
            Self::MoreThanOneWindows => words::MORE_THAN_ONE_WINDOWS,
            Self::DoesNotStartWindows => words::DOES_NOT_START_WINDOWS,
            Self::ApprovalNotAccepted => words::APPROVAL_NOT_ACCEPTED,
            Self::WouldNotBeTold => words::WOULD_NOT_BE_TOLD,
            Self::NotRemembered => words::NOT_REMEMBERED,
            Self::NotBeingKept => words::NOT_BEING_KEPT,
            Self::NothingMakesChanges => words::NOTHING_MAKES_CHANGES,
        };
        strings.say(&word.key(), &Filling::nothing())
    }

    /// What the broker's answer to this change means.
    ///
    /// **The change is asked for as well as the answer**, because the door's
    /// one word for *it was not carried out* has to become two different
    /// sentences: a person whose machine would not be told to start Windows
    /// once can switch it off and on and choose Windows, and a person whose
    /// machine would not keep which system it starts has a computer that
    /// behaves exactly as it did. One sentence for both would tell one of them
    /// something untrue about their own machine.
    ///
    /// # Errors
    /// The refusal, for every answer but `carried`.
    pub fn from_the_brokers(change: Change, answer: Answer) -> Result<(), Self> {
        match answer {
            Answer::Carried => Ok(()),
            Answer::NotKept => Err(Self::NotBeingKept),
            Answer::Refused(AtTheBroker::NotCarried) => Err(match change {
                Change::RestartIntoWindows => Self::WouldNotBeTold,
                Change::StartByDefault => Self::NotRemembered,
            }),
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
/// Nothing, for a verb that is not this road's: a surface with no sentence says
/// nothing rather than the wrong thing. And nothing for a `starting.default`
/// naming neither of the two systems, which is a verb that could not have been
/// carried out — the identity is compared here exactly as it is where the
/// change is made, so the sentence and the act cannot disagree.
#[must_use]
pub fn changed_said(verb: &SystemVerb, strings: &Strings) -> Option<Said> {
    let word = match (Change::of(verb)?, verb) {
        (Change::RestartIntoWindows, _) => words::WINDOWS_NEXT_TIME,
        (Change::StartByDefault, SystemVerb::StartByDefault(identity)) => {
            match the_system_named(*identity)? {
                System::AloOs => words::NOW_STARTS_ALO_OS,
                System::Windows => words::NOW_STARTS_WINDOWS,
            }
        }
        // `Change::of` answered `StartByDefault`, which only that verb is.
        (Change::StartByDefault, _) => return None,
    };
    Some(strings.say(&word.key(), &Filling::nothing()))
}

/// What a person reads in Settings about which system this machine starts when
/// nobody chooses.
#[must_use]
pub fn starts_at_said(system: System, strings: &Strings) -> Said {
    let word = match system {
        System::AloOs => words::STARTS_AT_ALO_OS,
        System::Windows => words::STARTS_AT_WINDOWS,
    };
    strings.say(&word.key(), &Filling::nothing())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::by_hand::to_start_by_default;
    use alo_broker::Identity;
    use alo_strings::Vocabulary;

    /// Every refusal there is, for the walks below.
    const EVERY_REFUSAL: [NotChanged; 10] = [
        NotChanged::NotAStartingChange,
        NotChanged::NotAnswering,
        NotChanged::NoWindowsHere,
        NotChanged::MoreThanOneWindows,
        NotChanged::DoesNotStartWindows,
        NotChanged::ApprovalNotAccepted,
        NotChanged::WouldNotBeTold,
        NotChanged::NotRemembered,
        NotChanged::NotBeingKept,
        NotChanged::NothingMakesChanges,
    ];

    /// This crate's own sentences, which is every sentence it can say.
    fn in_english() -> Strings {
        let mut vocabulary = Vocabulary::empty();
        words::declare_into(&mut vocabulary).unwrap();
        Strings::of(vocabulary)
    }

    /// A start-up entry, as a digest of one.
    fn an_entry() -> Identity {
        Identity::of_what_was_reported(b"a start-up entry")
    }

    /// **Every refusal is a sentence, not a key**, and every one of them says
    /// that nothing was changed.
    #[test]
    fn every_refusal_is_a_sentence_saying_nothing_was_changed() {
        let strings = in_english();
        for refusal in EVERY_REFUSAL {
            let said = refusal.said(&strings);
            assert!(!said.is_a_bug(), "{refusal:?}: {said}");
            assert!(said.unfilled().is_empty(), "{refusal:?}: {said}");
            assert!(
                said.text().to_lowercase().contains("nothing was changed"),
                "{refusal:?}: {said}"
            );
        }
    }

    /// **No two refusals read the same.** A person meeting *there is no Windows
    /// on this computer* and a person meeting *it would not be told* have
    /// different things to do, and one sentence for both would send one of them
    /// to try again forever.
    #[test]
    fn no_two_refusals_read_the_same() {
        let strings = in_english();
        let mut said: Vec<String> = EVERY_REFUSAL
            .iter()
            .map(|refusal| refusal.said(&strings).text().to_owned())
            .collect();
        said.sort();
        let how_many = said.len();
        said.dedup();
        assert_eq!(said.len(), how_many);
    }

    /// **Only `carried` is carried out.** Every other answer the door can give
    /// is a refusal a person can read.
    #[test]
    fn only_carried_is_carried_out_and_every_other_answer_is_read() {
        for change in [Change::RestartIntoWindows, Change::StartByDefault] {
            assert_eq!(
                NotChanged::from_the_brokers(change, Answer::Carried),
                Ok(())
            );
            assert_eq!(
                NotChanged::from_the_brokers(change, Answer::NotKept),
                Err(NotChanged::NotBeingKept)
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
                    NotChanged::from_the_brokers(change, Answer::Refused(why)),
                    Err(NotChanged::ApprovalNotAccepted),
                    "{why:?}"
                );
            }
        }
    }

    /// **The one word the door has for *it was not carried out* becomes the
    /// sentence of the road it was on.** A machine that would not be told to
    /// start Windows once is a different fact from one that would not keep
    /// which system it starts, and a person reading the wrong one of those is
    /// being told something untrue about their own computer.
    #[test]
    fn not_carried_reads_as_the_road_it_was_on() {
        let strings = in_english();
        let once = NotChanged::from_the_brokers(
            Change::RestartIntoWindows,
            Answer::Refused(AtTheBroker::NotCarried),
        );
        let always = NotChanged::from_the_brokers(
            Change::StartByDefault,
            Answer::Refused(AtTheBroker::NotCarried),
        );
        assert_eq!(once, Err(NotChanged::WouldNotBeTold));
        assert_eq!(always, Err(NotChanged::NotRemembered));
        assert_ne!(
            NotChanged::WouldNotBeTold.said(&strings).text(),
            NotChanged::NotRemembered.said(&strings).text()
        );
    }

    /// **A machine that will not answer is that refusal and no other**, however
    /// it was reached.
    #[test]
    fn a_machine_that_will_not_answer_reads_as_that() {
        assert_eq!(
            NotChanged::from(NotAnswering("nothing there".to_owned())),
            NotChanged::NotAnswering
        );
    }

    /// **What is read afterwards says it is for one start**, and nothing is
    /// said at all about a verb that is not this road's.
    #[test]
    fn what_is_read_afterwards_says_it_is_for_one_start() {
        let strings = in_english();
        let said = changed_said(&Change::RestartIntoWindows.to(an_entry()), &strings).unwrap();
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().to_lowercase().contains("next time"), "{said}");
        assert!(
            changed_said(&SystemVerb::MountDrive(an_entry()), &strings).is_none(),
            "a storage verb said something about which system starts"
        );
    }

    /// **What is read after the default changed names the system it now
    /// starts, and says that it lasts.** The two changes on this road are one
    /// approval apart and mean entirely different things to a person, so their
    /// sentences must not be able to be each other's.
    #[test]
    fn what_is_read_after_the_default_changed_names_the_system() {
        let strings = in_english();
        let once = changed_said(&Change::RestartIntoWindows.to(an_entry()), &strings).unwrap();
        for system in System::BOTH {
            let said = changed_said(&to_start_by_default(system), &strings).unwrap();
            assert!(!said.is_a_bug(), "{said}");
            assert!(said.unfilled().is_empty(), "{said}");
            assert!(
                said.text()
                    .to_lowercase()
                    .contains("whenever nobody chooses"),
                "{said}"
            );
            assert_ne!(said.text(), once.text());
        }
        assert_ne!(
            changed_said(&to_start_by_default(System::AloOs), &strings)
                .unwrap()
                .text(),
            changed_said(&to_start_by_default(System::Windows), &strings)
                .unwrap()
                .text()
        );
    }

    /// **A `starting.default` naming neither system says nothing.** It is a
    /// verb that could not have been carried out, and a surface that said
    /// something about it would be saying it about a change that never
    /// happened.
    #[test]
    fn a_default_naming_neither_system_says_nothing() {
        let strings = in_english();
        assert!(
            changed_said(&SystemVerb::StartByDefault(an_entry()), &strings).is_none(),
            "a verb naming no system said something about which system starts"
        );
    }

    /// **Settings reads one sentence for each system, and they differ.**
    #[test]
    fn settings_reads_one_sentence_for_each_system() {
        let strings = in_english();
        let alo = starts_at_said(System::AloOs, &strings);
        let windows = starts_at_said(System::Windows, &strings);
        for said in [&alo, &windows] {
            assert!(!said.is_a_bug(), "{said}");
            assert!(said.unfilled().is_empty(), "{said}");
        }
        assert_ne!(alo.text(), windows.text());
    }
}
