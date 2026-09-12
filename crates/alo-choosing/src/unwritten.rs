//! Why a person's change to their own settings was not made, in the words they
//! read.
//!
//! `crate::refusing` is the other direction and they are two types rather than
//! one, because they say two different things and a person acts on the
//! difference. `crate::NotSet` ends *nothing in the file has been used* — the
//! machine is running on no choice at all. Every one of these ends **nothing in
//! your settings has been changed**: the file is exactly as it was, whatever
//! was in it is still in force, and the thing that just failed was the change.
//!
//! Collapsing the two would produce the sentence somebody is most likely to
//! misread. A person told *nothing in the file has been used* after clicking
//! something would reasonably conclude their machine had just forgotten what
//! they chose last month.
//!
//! # Seven reasons, and two of them are somebody else's sentence
//!
//! The two lists a settings file holds are `alo-models`', and so are the
//! refusals about them: the same weights twice, a provider name already taken,
//! an address that is not one, a key that would travel in clear. Those arrive
//! as [`NotWritten::NotWeights`] and [`NotWritten::NotAProvider`] and are said
//! in `alo-models`' own words, exactly as `crate::NotSet::NotAProvider` already
//! does. A second sentence here would be two accounts of one moment, and the
//! one this crate wrote would be the one nobody kept up to date.
//!
//! **They are therefore the two that do not name the file**, because a refusal
//! about a list has no file in it. The five this crate says itself all do, for
//! `crate::refusing`'s reason: on a machine with several logins *your settings*
//! is not a thing anybody can act on.
//!
//! [`NotWritten::NotAProvider`] is also where an address that is not https
//! arrives, whichever door it came through: `crate::holding` asks
//! `alo_models::Provider::checked`'s rule again at the one place a provider is
//! written, and what it refuses is carried here in that crate's sentence —
//! *use https, or a service on this machine.*

use std::path::{Path, PathBuf};

use alo_models::{ProviderError, WeightsError};
use alo_strings::{Filling, Said, Strings};

use crate::words;

/// Why a change to a person's settings was not made.
///
/// Deliberately no `Display`, which is `crate::NotSet`'s rule and item 9b's: a
/// `Display` is one `to_string()` from a screen whose author had no reason to
/// think about language, and every one of these is read by the person whose
/// file it is. The only road to words is [`said`](NotWritten::said).
#[derive(Debug)]
pub enum NotWritten {
    /// The change would have said weights answer this person's questions, and
    /// would have listed none of that name.
    ///
    /// The same disagreement `crate::NotSet::NotBrought` refuses on the way in,
    /// caught before it can reach a disk: `crate::Settings::of` is asked on the
    /// changed value, so a settings panel cannot write a file its own machine
    /// would afterwards refuse whole.
    NotBrought {
        /// Where the settings are.
        at: PathBuf,
        /// What the choice named, exactly as it was given.
        model: String,
    },
    /// The change would have said a provider answers this person's questions,
    /// and their own list has none of that name.
    NoSuchProvider {
        /// Where the settings are.
        at: PathBuf,
        /// What the choice named, exactly as it was given.
        provider: String,
    },
    /// The list of weights would not take these.
    ///
    /// `alo_models::WeightsError`'s own refusal, carried rather than reworded:
    /// it is about a **list**, it has a sentence already, and this crate adds
    /// nothing to it.
    NotWeights {
        /// Where the settings are.
        at: PathBuf,
        /// What the list refused, kept as the fact it is.
        why: WeightsError,
    },
    /// The list of providers would not take this one, or a provider on it is
    /// not one `alo_models::Provider::checked` would have made.
    ///
    /// The second is how an address that is not https is refused at the write:
    /// `alo_models::Provider` has public fields, so a value can arrive that
    /// nothing has judged, and `crate::holding` judges it again in the one
    /// function every door writes through.
    NotAProvider {
        /// Where the settings are.
        at: PathBuf,
        /// What was refused, kept as the fact it is.
        why: ProviderError,
    },
    /// The change would have replaced a provider on this person's list, and
    /// the list has none of that name.
    ///
    /// Refused rather than added, because *change* and *add* are two different
    /// things a person did: a surface that changed a provider it did not have
    /// would be adding one under a button that said something else.
    NothingToChange {
        /// Where the settings are.
        at: PathBuf,
        /// What the change named, exactly as it was given.
        provider: String,
    },
    /// This alo OS could not turn the changed settings into a file it reads
    /// back as the same settings.
    ///
    /// A defect rather than anything a person typed, and it is refused **before
    /// the disk is touched** — `crate::writing` has the argument, which is that
    /// the alternative is a change reported as made and afterwards described
    /// differently by the machine that made it.
    NotExpressible {
        /// Where the settings are.
        at: PathBuf,
        /// What was not expressible, in the English whoever is fixing the
        /// machine reads — never shown to anybody, for
        /// `crate::NotSet::NotRead`'s reason: it is not a sentence and it is
        /// not in anybody's language.
        why: String,
    },
    /// The disk would not take the file.
    NotKept {
        /// Where the settings are.
        at: PathBuf,
        /// What the machine said, in the operating system's own words. Read by
        /// whoever is fixing the machine rather than shown to the person.
        why: String,
    },
}

impl NotWritten {
    /// Where the settings are.
    #[must_use]
    pub fn at(&self) -> &Path {
        match self {
            Self::NotBrought { at, .. }
            | Self::NoSuchProvider { at, .. }
            | Self::NotWeights { at, .. }
            | Self::NotAProvider { at, .. }
            | Self::NothingToChange { at, .. }
            | Self::NotExpressible { at, .. }
            | Self::NotKept { at, .. } => at,
        }
    }

    /// The string said for this reason.
    ///
    /// Two of them are `alo-models`' rather than this crate's, which is why
    /// [`said`](Self::said) is not simply this key filled in: their sentences
    /// take their own gaps.
    #[must_use]
    pub fn word(&self) -> words::Word {
        match self {
            Self::NotBrought { .. } => words::CHANGE_NOT_BROUGHT,
            Self::NoSuchProvider { .. } => words::CHANGE_NO_SUCH_PROVIDER,
            Self::NotWeights { why, .. } => why.word(),
            Self::NotAProvider { why, .. } => why.word(),
            Self::NothingToChange { .. } => words::CHANGE_NOTHING_TO_CHANGE,
            Self::NotExpressible { .. } => words::CHANGE_NOT_EXPRESSIBLE,
            Self::NotKept { .. } => words::CHANGE_NOT_KEPT,
        }
    }

    /// What a person is told, in the language they read.
    ///
    /// Never fails and never panics: a `Strings` that was never given
    /// [`crate::choosing_words`] answers with the key, marked, and the settings
    /// are unchanged either way.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        // The list's own refusals already have a sentence for this person, and
        // it is filled from the list rather than from here.
        match self {
            Self::NotWeights { why, .. } => return why.said(strings),
            Self::NotAProvider { why, .. } => return why.said(strings),
            Self::NotBrought { .. }
            | Self::NoSuchProvider { .. }
            | Self::NothingToChange { .. }
            | Self::NotExpressible { .. }
            | Self::NotKept { .. } => {}
        }
        let filling = Filling::of("path", self.at().to_string_lossy().into_owned());
        let filling = match self {
            // Both quote back a name exactly as it was given, which is data and
            // is never translated — the rule a filename is held to in
            // `alo-files` and a path in `crate::refusing`.
            Self::NotBrought { model, .. } => filling.and("model", model.clone()),
            Self::NoSuchProvider { provider, .. } | Self::NothingToChange { provider, .. } => {
                filling.and("provider", provider.clone())
            }
            Self::NotWeights { .. }
            | Self::NotAProvider { .. }
            | Self::NotExpressible { .. }
            | Self::NotKept { .. } => filling,
        };
        strings.say(&self.word().key(), &filling)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    /// Where the settings are, in every one of these tests.
    fn somewhere() -> PathBuf {
        PathBuf::from("/home/ada/.config/alo/settings.toml")
    }

    /// Every way a change is not made, so the tests below walk all of them
    /// rather than the two somebody remembered.
    fn every_reason() -> Vec<NotWritten> {
        vec![
            NotWritten::NotBrought {
                at: somewhere(),
                model: "my-finetune".to_owned(),
            },
            NotWritten::NoSuchProvider {
                at: somewhere(),
                provider: "Mistral".to_owned(),
            },
            NotWritten::NotWeights {
                at: somewhere(),
                why: WeightsError::AlreadyBrought("my-finetune".to_owned()),
            },
            NotWritten::NotAProvider {
                at: somewhere(),
                why: ProviderError::AlreadyAdded("Mistral".to_owned()),
            },
            NotWritten::NothingToChange {
                at: somewhere(),
                provider: "Mistral".to_owned(),
            },
            NotWritten::NotExpressible {
                at: somewhere(),
                why: "a provider carried a list of models this file cannot hold".to_owned(),
            },
            NotWritten::NotKept {
                at: somewhere(),
                why: "permission denied".to_owned(),
            },
        ]
    }

    /// Whether this crate words a reason itself, or carries `alo-models`'.
    fn ours(reason: &NotWritten) -> bool {
        match reason {
            NotWritten::NotBrought { .. }
            | NotWritten::NoSuchProvider { .. }
            | NotWritten::NothingToChange { .. }
            | NotWritten::NotExpressible { .. }
            | NotWritten::NotKept { .. } => true,
            NotWritten::NotWeights { .. } | NotWritten::NotAProvider { .. } => false,
        }
    }

    /// **Every reason reaches the person as a sentence**, and every one this
    /// crate words itself says the settings are as they were — the half a
    /// person acts on, because their machine is still answering questions the
    /// way it was a moment ago and the thing that failed was the change.
    #[test]
    fn every_refusal_says_the_settings_did_not_move() {
        let strings = in_english();
        for reason in every_reason() {
            let said = reason.said(&strings);
            assert!(!said.is_a_bug(), "{said}");
            assert_eq!(reason.at(), somewhere());
            if ours(&reason) {
                assert!(
                    said.text()
                        .contains("nothing in your settings has been changed"),
                    "{said}"
                );
            }
        }
    }

    /// **The five sentences this crate says name the file somebody has to
    /// open.** The other two are `alo-models`' own, are about a list rather
    /// than a file, and have no path to name — which is stated here rather than
    /// noticed later, because it is the one place this crate's refusals are not
    /// all alike.
    #[test]
    fn the_reasons_this_crate_words_itself_all_name_the_file() {
        let strings = in_english();
        for reason in every_reason() {
            let said = reason.said(&strings);
            assert_eq!(
                said.text().contains("/home/ada/.config/alo/settings.toml"),
                ours(&reason),
                "{said}"
            );
        }
    }

    /// **Seven reasons, seven sentences.** A machine that said the same thing about
    /// a disk that would not take the file and a choice naming weights nobody
    /// brought would be sending somebody to the wrong place.
    #[test]
    fn no_two_reasons_share_a_sentence() {
        let strings = in_english();
        let reasons = every_reason();
        let mut said: Vec<String> = reasons
            .iter()
            .map(|reason| reason.said(&strings).into_text())
            .collect();
        said.sort();
        said.dedup();
        assert_eq!(said.len(), reasons.len());
        assert_eq!(said.len(), 7);
    }

    /// **A name is quoted back exactly as it was given**, which is what lets
    /// somebody see which of the two they got wrong.
    #[test]
    fn the_name_a_change_used_is_quoted_back_as_it_was_given() {
        let strings = in_english();
        assert!(
            NotWritten::NotBrought {
                at: somewhere(),
                model: "my-finetune".to_owned(),
            }
            .said(&strings)
            .text()
            .contains("my-finetune")
        );
        assert!(
            NotWritten::NoSuchProvider {
                at: somewhere(),
                provider: "Mistral".to_owned(),
            }
            .said(&strings)
            .text()
            .contains("Mistral")
        );
        assert!(
            NotWritten::NothingToChange {
                at: somewhere(),
                provider: "Mistral".to_owned(),
            }
            .said(&strings)
            .text()
            .contains("Mistral")
        );
    }

    /// **An address that is not https is said in `alo-models`' words**, which
    /// tell the person what to do rather than that alo OS has a fault: the
    /// refusal carried here is that crate's own, and its sentence names https
    /// and the one exception.
    #[test]
    fn an_address_that_is_not_https_is_said_as_what_to_do() {
        let said = NotWritten::NotAProvider {
            at: somewhere(),
            why: ProviderError::InsecureEndpoint,
        }
        .said(&in_english());
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("https"), "{said}");
        assert!(said.text().contains("this machine"), "{said}");
        assert!(!said.text().contains("alo OS could not"), "{said}");
    }

    /// **Nothing a maintainer is told reaches the person.** What the disk said
    /// is in the operating system's words and in nobody's language, and it is
    /// not in the sentence.
    #[test]
    fn what_the_disk_said_is_not_what_the_person_reads() {
        let said = NotWritten::NotKept {
            at: somewhere(),
            why: "ENOSPC: no space left on device".to_owned(),
        }
        .said(&in_english());
        assert!(!said.text().contains("ENOSPC"), "{said}");
    }

    /// And every one of them is read in the language the machine is showing.
    #[test]
    fn a_refusal_is_read_in_the_language_the_machine_is_showing() {
        let strings = translated(&[(
            words::CHANGE_NOT_KEPT,
            "Ihre Einstellungen in {path} konnten nicht geschrieben werden",
        )]);
        let said = NotWritten::NotKept {
            at: somewhere(),
            why: "permission denied".to_owned(),
        }
        .said(&strings);
        assert!(said.is_translated(), "{said}");
        assert!(said.text().contains("geschrieben"), "{said}");
        assert!(said.text().contains("settings.toml"), "{said}");
    }
}
