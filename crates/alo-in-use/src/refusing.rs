//! Why the machine could not say what is watching or listening.
//!
//! **There is no silent empty indicator.** This is the whole reason the type
//! exists: an indicator that showed no lines when it could not reach the media
//! server would look exactly like an indicator on a machine where nothing is
//! watching, and that is not a degraded answer — it is the wrong answer, given
//! confidently, about the one thing this indicator is for.
//!
//! So [`crate::InUse::read_from`] answers a `Result`, and every way of failing
//! to read is one of these, and every one of them has a sentence a person can
//! read in their own language.
//!
//! # Where these are read
//!
//! On the indicator itself, in place of the lines — unlike
//! `alo_indicator::NotShown`, whose refusals are read in a log because the
//! thing that is missing is the screen. Here the screen is fine and the answer
//! is missing, so the refusal goes where the answer would have gone.
//!
//! # What each of them carries
//!
//! Two of them carry what the machine said, in English, for whoever is fixing
//! it. That is the same shape `alo_software::Failed` uses and for the same
//! reason: the sentence a person reads is declared and translated, and the
//! detail beside it is a diagnostic that never reaches a screen.

use alo_strings::{Filling, Said, Strings};

use crate::words::{self, Word};

/// Why what is watching or listening could not be read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum NotHeard {
    /// There is nothing on this machine handling sound and video, so there is
    /// no record of open streams to read.
    #[error("nothing on this machine handles sound and video: {said}")]
    NothingHandlesSoundAndVideo {
        /// What the machine said when it was looked for. English, for whoever
        /// is fixing it, and never shown to a person.
        said: String,
    },
    /// It was asked and did not answer.
    #[error("what handles sound and video did not answer: {said}")]
    NoAnswer {
        /// What it said instead. English, for whoever is fixing it.
        said: String,
    },
    /// It answered something this machine could not read.
    #[error("what handles sound and video answered something unreadable: {said}")]
    NotUnderstood {
        /// What was wrong with the answer. English, for whoever is fixing it.
        said: String,
    },
}

impl NotHeard {
    /// The string this crate declares for it: the key a translator's file is
    /// sorted by, and the English beside it.
    #[must_use]
    pub const fn word(&self) -> Word {
        match self {
            Self::NothingHandlesSoundAndVideo { .. } => words::NOTHING_HANDLES_SOUND_AND_VIDEO,
            Self::NoAnswer { .. } => words::NO_ANSWER,
            Self::NotUnderstood { .. } => words::NOT_UNDERSTOOD,
        }
    }

    /// What was said about it in English, for whoever is fixing it.
    ///
    /// Never put in front of a person: what a person reads is [`NotHeard::said`]
    /// and it is answered in their own language.
    #[must_use]
    pub fn diagnosis(&self) -> &str {
        match self {
            Self::NothingHandlesSoundAndVideo { said }
            | Self::NoAnswer { said }
            | Self::NotUnderstood { said } => said,
        }
    }

    /// What to tell the person, in the language they read.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not: a
    /// `Strings` that was never given [`crate::in_use_words`] answers with the
    /// key, marked `Said::is_a_bug`, which is the honest answer to *the shell
    /// forgot to declare what this crate can say*.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    /// Every way of not being heard, so no test below quietly skips one.
    fn every_refusal() -> [NotHeard; 3] {
        [
            NotHeard::NothingHandlesSoundAndVideo {
                said: "pw-dump is not installed".to_owned(),
            },
            NotHeard::NoAnswer {
                said: "no such socket".to_owned(),
            },
            NotHeard::NotUnderstood {
                said: "the record is not a list".to_owned(),
            },
        ]
    }

    /// **Every refusal says something a person could read**, and no two read
    /// the same — somebody told the same sentence for a component that is not
    /// running and one that answered nonsense would go and do the wrong thing.
    #[test]
    fn every_refusal_reads_and_no_two_read_the_same() {
        let strings = in_english();
        let mut seen: Vec<String> = Vec::new();
        for refusal in every_refusal() {
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

    /// **Every refusal says what is watching or listening cannot be shown**,
    /// rather than reading as though nothing were. This is the sentence the
    /// whole type exists for: silence and *nothing is watching* must never be
    /// the same thing on a screen.
    #[test]
    fn no_refusal_can_be_mistaken_for_a_quiet_room() {
        let strings = in_english();
        let quiet = strings.say(&words::NOTHING_IS_IN_USE.key(), &Filling::nothing());
        for refusal in every_refusal() {
            let said = refusal.said(&strings);
            assert_ne!(said.text(), quiet.text());
            assert!(
                said.text().contains("cannot be shown"),
                "{said} does not say that it could not answer"
            );
        }
    }

    /// **The diagnosis is kept and is not the sentence.** Whoever is fixing the
    /// machine needs what it actually said; the person reading the indicator
    /// never sees it.
    #[test]
    fn what_the_machine_said_is_kept_for_whoever_is_fixing_it() {
        let strings = in_english();
        for refusal in every_refusal() {
            assert!(!refusal.diagnosis().is_empty(), "{refusal:?}");
            assert!(
                !refusal.said(&strings).text().contains(refusal.diagnosis()),
                "{refusal:?} puts a diagnostic in front of a person"
            );
        }
    }

    /// **The refusal arrives in the language the person reads** when somebody
    /// has translated it, and says so.
    #[test]
    fn a_refusal_is_read_in_the_language_the_person_reads() {
        let strings = translated(&[(
            words::NO_ANSWER,
            "Was gerade zusieht oder zuhört, kann nicht angezeigt werden: der Teil von alo OS, \
             der für Ton und Video zuständig ist, hat nicht geantwortet. Melden Sie sich ab und \
             wieder an, um ihn neu zu starten",
        )]);
        let said = NotHeard::NoAnswer {
            said: "no such socket".to_owned(),
        }
        .said(&strings);
        assert!(said.is_translated());
        assert!(said.text().starts_with("Was gerade zusieht"));

        // The one nobody translated is still English, and says it is.
        let untranslated = NotHeard::NotUnderstood {
            said: "not a list".to_owned(),
        }
        .said(&strings);
        assert!(!untranslated.is_translated());
        assert!(!untranslated.is_a_bug());
    }
}
