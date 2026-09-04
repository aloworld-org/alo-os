//! Why a person's settings were not read, in the words they read.
//!
//! Five things can be wrong with the file, and they are five variants rather
//! than one because they send somebody to five different places: a disk that
//! would not give it up, text that is not settings, a file written for a newer
//! alo OS, a model named nothing, and a language nobody could read.
//!
//! # Every one of them names the file
//!
//! A path is data rather than language — the rule `alo-files` holds a filename
//! to and `alo-egress` holds an address to — so it goes in as text and is never
//! translated. It is in every one of these sentences because a person who has
//! been told their settings are wrong needs the one thing that lets them fix
//! it, and on a machine with several logins *your settings* is not that thing.
//!
//! # None of them stops the machine
//!
//! `alo-agentd` refuses to run without a description, because a machine
//! half-described is one nobody notices until it matters. A person's settings
//! are the opposite case: it is their file, it costs nobody else, and a service
//! that would not start because of a typo in it would take away the agent, the
//! record and the grants over a line about a model. So the machine runs, and
//! the person is told this the moment they ask a question.
//!
//! # And a file that is not there is not one of them
//!
//! [`crate::Settings::at`] answers `Settings::untouched` for a file that does
//! not exist, which is a person who has not chosen rather than a mistake.

use std::path::{Path, PathBuf};

use alo_strings::{Filling, LanguageError, Said, Strings};

use crate::words;

/// Why the settings at a path are not settings.
///
/// Deliberately no `Display`, which is item 9b's rule: a `Display` is one
/// `to_string()` from a screen whose author had no reason to think about
/// language, and every one of these is read by the person whose file it is.
/// The only road to words is [`said`](NotSet::said).
#[derive(Debug)]
pub enum NotSet {
    /// The file is there and the disk would not give it up.
    NotRead {
        /// Where it is.
        at: PathBuf,
        /// What the disk said, in the operating system's own words. Read by
        /// whoever is fixing the machine rather than shown to the person, for
        /// `alo_keeping::NotKept`'s reason: it is not a sentence and it is not
        /// in anybody's language.
        why: String,
    },
    /// The text is not settings: a key nobody declared, a value of the wrong
    /// shape, a comma somewhere TOML does not have them.
    NotUnderstood {
        /// Where it is.
        at: PathBuf,
        /// What the parser made of it. Boxed because it is much larger than
        /// every other variant here and this type travels inside a `Result`.
        why: Box<toml::de::Error>,
    },
    /// The file says it is a shape this alo OS does not read.
    AnotherFormat {
        /// Where it is.
        at: PathBuf,
        /// What the file says it is.
        format: u32,
        /// What this alo OS reads.
        reads: u32,
    },
    /// A list was named and no model with it.
    Nameless {
        /// Where it is.
        at: PathBuf,
    },
    /// A language that is not one.
    NotALanguage {
        /// Where it is.
        at: PathBuf,
        /// What was written where a language belongs, exactly as it was
        /// written.
        tag: String,
        /// Why that is not a language.
        why: LanguageError,
    },
}

impl NotSet {
    /// Where the file is.
    #[must_use]
    pub fn at(&self) -> &Path {
        match self {
            Self::NotRead { at, .. }
            | Self::NotUnderstood { at, .. }
            | Self::AnotherFormat { at, .. }
            | Self::Nameless { at }
            | Self::NotALanguage { at, .. } => at,
        }
    }

    /// The string this crate declares for this reason.
    #[must_use]
    pub fn word(&self) -> words::Word {
        match self {
            Self::NotRead { .. } => words::SETTINGS_NOT_READ,
            Self::NotUnderstood { .. } => words::SETTINGS_NOT_UNDERSTOOD,
            Self::AnotherFormat { .. } => words::SETTINGS_FROM_A_NEWER_ALO_OS,
            Self::Nameless { .. } => words::SETTINGS_NAME_NO_MODEL,
            Self::NotALanguage { .. } => words::SETTINGS_NAME_NO_LANGUAGE,
        }
    }

    /// What a person is told, in the language they read.
    ///
    /// Which is, on a machine whose settings say the language and will not
    /// read, whichever language the machine was already showing — the one place
    /// in alo OS where a refusal cannot be in the language the refusal is
    /// about. That is stated rather than solved: it is the honest consequence
    /// of a person's language living in the file that did not load, and the
    /// alternative is a machine that guesses at a language from a file it has
    /// just refused to believe.
    ///
    /// Never fails and never panics: a `Strings` that was never given
    /// [`crate::choosing_words`] answers with the key, marked, and the settings
    /// are unread either way.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = Filling::of("path", self.at().to_string_lossy().into_owned());
        let filling = match self {
            Self::NotALanguage { tag, .. } => filling.and("language", tag.clone()),
            Self::NotRead { .. }
            | Self::NotUnderstood { .. }
            | Self::AnotherFormat { .. }
            | Self::Nameless { .. } => filling,
        };
        strings.say(&self.word().key(), &filling)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};
    use crate::written::THE_FORMAT;

    /// Where the file is, in every one of these tests.
    fn somewhere() -> PathBuf {
        PathBuf::from("/home/ada/.config/alo/settings.toml")
    }

    /// Every way a settings file is not settings, so the tests below can walk
    /// all of them rather than the two somebody remembered.
    fn every_reason() -> Vec<NotSet> {
        vec![
            NotSet::NotRead {
                at: somewhere(),
                why: "permission denied".to_owned(),
            },
            NotSet::NotUnderstood {
                at: somewhere(),
                why: Box::new(toml::from_str::<toml::Table>("=").unwrap_err()),
            },
            NotSet::AnotherFormat {
                at: somewhere(),
                format: 7,
                reads: THE_FORMAT,
            },
            NotSet::Nameless { at: somewhere() },
            NotSet::NotALanguage {
                at: somewhere(),
                tag: "Deutsch".to_owned(),
                why: alo_strings::Language::written("Deutsch").unwrap_err(),
            },
        ]
    }

    /// **Every refusal names the file**, because on a machine with several
    /// logins *your settings* is not something a person can act on.
    #[test]
    fn every_refusal_names_the_file_somebody_has_to_open() {
        let strings = in_english();
        for reason in every_reason() {
            let said = reason.said(&strings);
            assert!(
                said.text().contains("/home/ada/.config/alo/settings.toml"),
                "{said}"
            );
            assert!(!said.is_a_bug(), "{said}");
            assert_eq!(reason.at(), somewhere());
        }
    }

    /// **Five reasons, five sentences.** A machine that said the same thing
    /// about a disk that would not answer and a language nobody could read
    /// would be sending somebody to the wrong place twice.
    #[test]
    fn no_two_reasons_share_a_sentence() {
        let strings = in_english();
        let mut said: Vec<String> = every_reason()
            .iter()
            .map(|reason| reason.said(&strings).into_text())
            .collect();
        said.sort();
        said.dedup();
        assert_eq!(said.len(), 5);
    }

    /// **What was written where a language belongs is quoted back**, exactly as
    /// it was written and never translated: it is what somebody typed, and a
    /// sentence that did not name it would leave them looking for it.
    #[test]
    fn a_language_that_is_not_one_is_quoted_back_as_it_was_written() {
        let said = NotSet::NotALanguage {
            at: somewhere(),
            tag: "Deutsch".to_owned(),
            why: alo_strings::Language::written("Deutsch").unwrap_err(),
        }
        .said(&in_english());
        assert!(said.text().contains("Deutsch"), "{said}");
    }

    /// And every one of them is read in the language the machine is showing,
    /// which is this crate's ordinary rule with one honest exception written
    /// into [`NotSet::said`].
    #[test]
    fn a_refusal_is_read_in_the_language_the_machine_is_showing() {
        let strings = translated(&[(
            words::SETTINGS_NOT_UNDERSTOOD,
            "Ihre Einstellungen in {path} sind keine Einstellungen",
        )]);
        let said = NotSet::NotUnderstood {
            at: somewhere(),
            why: Box::new(toml::from_str::<toml::Table>("=").unwrap_err()),
        }
        .said(&strings);
        assert!(said.is_translated(), "{said}");
        assert!(said.text().contains("Einstellungen"), "{said}");
        assert!(said.text().contains("settings.toml"), "{said}");
    }
}
