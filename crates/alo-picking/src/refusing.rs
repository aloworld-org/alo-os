//! Every way a folder is not picked, and the sentence a person reads for it.
//!
//! A folder chooser is the one surface on this machine where being stuck means
//! the agent can never be granted anything at all — with nothing granted,
//! `alo-agentd` refuses every verb, correctly and forever. So there is no path
//! through [`crate::Picker`] that answers with silence: every refusal is one of
//! these, and every one of these has a sentence in [`crate::words`] that says
//! what to do next.
//!
//! Like everything else this repository says to a person, the sentences are
//! answered through [`NotPicked::said`] in the language they read. There is no
//! `Display` here that would put English on a screen by accident — the same
//! decision, for the same reason, as `alo_capability::GrantError`.

use alo_strings::{Filling, Said, Strings};

use crate::folders::NotShown;
use crate::words::{self, Word};

/// Why a folder was not shown, not opened, or not picked.
///
/// Seven, and they are kept apart rather than folded into *it did not work*
/// because a person acts differently on each: a folder that went away sends
/// them back, a folder the machine will not open sends them elsewhere or to
/// whoever looks after the machine, and the whole machine is not a mistake at
/// all but a rule alo OS holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotPicked {
    /// The picker was opened somewhere that is not a full path, so it cannot
    /// be compared against what is granted.
    NotAFullPath,
    /// The picker was opened somewhere whose name steps upwards, which can
    /// mean a different folder than it appears to.
    CouldLeadElsewhere,
    /// It is not there — moved or deleted, possibly since it was listed.
    NothingThere,
    /// It is there and it is not a folder.
    NotAFolder,
    /// It is there and this machine would not open it.
    WouldNotBeRead,
    /// A name that is not among the folders being shown.
    NotShownHere,
    /// There is nothing above where the picker is standing.
    NothingAbove,
    /// A pick at the top of the disk, which would be a grant to the whole
    /// machine — and there is no such grant (ADR 0001 §3).
    TheWholeMachine,
}

impl NotPicked {
    /// The string this crate declares for it: the key a translator's file is
    /// sorted by, and the English beside it.
    #[must_use]
    pub fn word(self) -> Word {
        match self {
            Self::NotAFullPath => words::NOT_A_FULL_PATH,
            Self::CouldLeadElsewhere => words::COULD_LEAD_ELSEWHERE,
            Self::NothingThere => words::NOTHING_THERE,
            Self::NotAFolder => words::NOT_A_FOLDER,
            Self::WouldNotBeRead => words::WOULD_NOT_BE_READ,
            Self::NotShownHere => words::NOT_SHOWN_HERE,
            Self::NothingAbove => words::NOTHING_ABOVE,
            Self::TheWholeMachine => words::THE_WHOLE_MACHINE,
        }
    }

    /// What to tell the person, in the language they read.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not:
    /// there is always something to put in front of the person, and where it
    /// came from is on the [`Said`]. A `Strings` that was never given
    /// [`crate::picking_words`] answers with the key, marked `Said::is_a_bug`
    /// — the honest answer to *the shell forgot to declare what this crate can
    /// say*.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

impl From<NotShown> for NotPicked {
    /// What the machine said about a folder, as what the person is told.
    ///
    /// One arm each and no default, so a fourth way of not being shown cannot
    /// arrive here and quietly become somebody else's sentence.
    fn from(not_shown: NotShown) -> Self {
        match not_shown {
            NotShown::NotAFolder => Self::NotAFolder,
            NotShown::WentAway => Self::NothingThere,
            NotShown::WouldNotBeRead => Self::WouldNotBeRead,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    /// Every way of not picking, so no test below quietly skips one.
    const EVERY_REFUSAL: [NotPicked; 8] = [
        NotPicked::NotAFullPath,
        NotPicked::CouldLeadElsewhere,
        NotPicked::NothingThere,
        NotPicked::NotAFolder,
        NotPicked::WouldNotBeRead,
        NotPicked::NotShownHere,
        NotPicked::NothingAbove,
        NotPicked::TheWholeMachine,
    ];

    /// **Every refusal says something a person could read**, and no two ways
    /// of failing read the same — a person told one sentence for a folder that
    /// went away and for one the machine would not open would go and fix the
    /// wrong thing.
    #[test]
    fn every_refusal_reads_and_no_two_read_the_same() {
        let strings = in_english();
        let mut seen: Vec<String> = Vec::new();
        for refusal in EVERY_REFUSAL {
            let said = refusal.said(&strings);
            assert!(!said.text().is_empty(), "{refusal:?} says nothing");
            assert!(!said.is_a_bug(), "{refusal:?} is not declared");
            assert_eq!(said.text(), refusal.word().says());
            assert!(
                !seen.contains(&said.text().to_owned()),
                "two refusals both say {said}"
            );
            seen.push(said.text().to_owned());
        }
    }

    /// **The refusal arrives in the language the person reads** when somebody
    /// has translated it, and says so — the whole of what declaring these
    /// through `alo-strings` buys.
    #[test]
    fn a_refusal_is_read_in_the_language_the_person_reads() {
        let strings = translated(&[(
            words::THE_WHOLE_MACHINE,
            "Der Agent erhält nie den ganzen Rechner. Öffnen Sie einen Ordner und wählen Sie den \
             Ordner, den Sie wirklich meinen",
        )]);
        let said = NotPicked::TheWholeMachine.said(&strings);
        assert!(said.is_translated());
        assert!(said.text().starts_with("Der Agent"), "{said}");

        // The one nobody translated is still English, and says it is.
        let untranslated = NotPicked::NothingThere.said(&strings);
        assert!(!untranslated.is_translated());
        assert!(!untranslated.is_a_bug());
    }

    /// **What the machine said becomes what the person is told**, one fact to
    /// one sentence — a folder that went away is never reported as one the
    /// machine would not open.
    #[test]
    fn each_way_of_not_being_shown_becomes_its_own_refusal() {
        assert_eq!(NotPicked::from(NotShown::NotAFolder), NotPicked::NotAFolder);
        assert_eq!(NotPicked::from(NotShown::WentAway), NotPicked::NothingThere);
        assert_eq!(
            NotPicked::from(NotShown::WouldNotBeRead),
            NotPicked::WouldNotBeRead
        );
    }
}
