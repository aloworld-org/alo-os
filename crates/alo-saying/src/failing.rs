//! What did not reach anybody's language: a whole file, or one line of one.
//!
//! # This is the one crate that cannot ask for words
//!
//! Everywhere else in this workspace, English in the source is a bug: a refusal
//! is a value, and somebody renders it out of the machine's vocabulary in the
//! language the person reads. Here that is not available, and not because
//! nobody got round to it. **This is what happens when the vocabulary does not
//! load**, and a sentence saying so, looked up in the vocabulary that did not
//! load, would come out as its own key.
//!
//! So these keep their English and their `Display`, and the rule survives
//! rather than being bent: `CLAUDE.md` says hardcoded English is a bug in what
//! a *person using the machine* reads, and nobody using a machine reads this.
//! Whoever reads it is whoever built the image or contributed the translation —
//! `alo_shortcuts::DefaultsError`'s reader, and `alo-agentd`'s `refusing`
//! module's, which keeps its English for the same reason.
//!
//! # None of this stops a machine
//!
//! Every value here travels inside [`crate::Damage`] rather than out of a
//! `Result`, because a machine that would not start over a translation could
//! not tell anybody why: the sentence explaining it is in the file that did not
//! load. A machine with nothing translated speaks English and says so, which is
//! the state alo OS ships in today.

use alo_strings::{Language, Wrongs};

/// Why a translation is not being spoken.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum NotSpoken {
    /// The directory the translations live in could not be read.
    #[error(
        "no translations were loaded: {at} could not be read — {why}. This machine speaks English until it is fixed"
    )]
    NoneHere {
        /// Where they were looked for.
        at: String,
        /// What the machine said about it.
        why: String,
    },

    /// A file that says `.toml` and would not be read.
    #[error("{file} could not be read — {why}")]
    NotRead {
        /// Which file.
        file: String,
        /// What the machine said about it.
        why: String,
    },

    /// A file that is not TOML.
    #[error("{file} is not a translation file — {why}")]
    NotWritten {
        /// Which file.
        file: String,
        /// What was wrong with it, in the words of whatever read it.
        why: String,
    },

    /// A file in a shape this alo OS does not read.
    #[error(
        "{file} says it is in shape {format} and this alo OS reads shape {reads} — it was written for a later one, and guessing at it would show somebody a sentence nobody wrote"
    )]
    FromANewerAlo {
        /// Which file.
        file: String,
        /// What it says it is.
        format: u64,
        /// What this alo OS reads.
        reads: u64,
    },

    /// A language tag that is not one.
    #[error("{file} is written for \"{tag}\", which is not a language — {why}")]
    NotALanguage {
        /// Which file.
        file: String,
        /// What it said the language was.
        tag: String,
        /// Why that is not one.
        why: String,
    },

    /// A key that is not one.
    #[error("{file} has a line called \"{named}\", which is not the name of a string — {why}")]
    NotAKey {
        /// Which file.
        file: String,
        /// What the line was called.
        named: String,
        /// Why that is not a name.
        why: String,
    },

    /// A translation nothing at all could be taken from.
    ///
    /// The two checks in [`crate::loading`] make this unreachable — the second
    /// is asked of what is left after everything the first refused has been
    /// taken out, and checking fewer strings cannot find more wrong with them.
    /// It is reported rather than asserted because law 3 forbids a panic on a
    /// path nobody has proved cannot happen, and a machine speaking English is
    /// a better answer than a machine that stopped.
    #[error("{file} gave nothing that could be shown — {why}")]
    GaveNothing {
        /// Which file.
        file: String,
        /// What was wrong with it.
        why: String,
    },

    /// A second file for a language another file already gave.
    #[error(
        "{file} is a second {language} translation and {already} was read first, so this one was left unread — put the two together into one file, because which of them answered would otherwise depend on the order the directory came back in"
    )]
    AlreadySpoken {
        /// Which file was not read.
        file: String,
        /// The language both are for.
        language: Language,
        /// The file that was read.
        already: String,
    },
}

/// Lines left out of a language that was otherwise loaded.
///
/// **A file is not thrown away over a line.** `alo_strings::Vocabulary::check`
/// refuses a whole translation when anything in it would come out wrong, which
/// is right at the moment somebody contributes one and wrong at the moment a
/// machine loads one: a string renamed in a release would otherwise turn a
/// person's language off entirely, in the release that renamed it, on every
/// machine at once.
///
/// So the lines that would come out wrong are left out, the rest of the
/// language is shown, and what was left out is reported here. The sentences are
/// `alo-strings`' own and are addressed to a translator.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("{file}: {wrongs}")]
pub struct LeftOut {
    /// Which file.
    file: String,
    /// Which lines, and what would have been wrong with each.
    wrongs: Wrongs,
}

impl LeftOut {
    /// Lines left out of one file.
    pub(crate) fn of(file: String, wrongs: Wrongs) -> Self {
        Self { file, wrongs }
    }

    /// Which file they were in.
    #[must_use]
    pub fn file(&self) -> &str {
        &self.file
    }

    /// Which lines, and what would have been wrong with each.
    #[must_use]
    pub fn wrongs(&self) -> &Wrongs {
        &self.wrongs
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::every_way_it_can_fail;

    /// Every one of these is read by somebody who has to go and change
    /// something, so every one of them names the file.
    #[test]
    fn everything_that_did_not_load_says_which_file_it_was() {
        for not_spoken in every_way_it_can_fail() {
            let said = not_spoken.to_string();
            assert!(
                said.contains("de.toml") || said.contains("/usr/share/alo/translations"),
                "{said}"
            );
        }
    }

    /// **A file that was not read is never reported as an empty one.** Each of
    /// these says what to do about it rather than only that something happened,
    /// which is `alo-models`' rule about an error one file over.
    #[test]
    fn everything_that_did_not_load_says_enough_to_act_on() {
        for not_spoken in every_way_it_can_fail() {
            let said = not_spoken.to_string();
            assert!(said.len() > 30, "{said}");
            assert!(!said.ends_with(' '), "{said}");
        }
    }

    /// A machine that found no translations at all says where it looked, so
    /// whoever packaged it can see which directory did not arrive.
    #[test]
    fn a_machine_with_no_translations_at_all_says_where_it_looked() {
        let none = NotSpoken::NoneHere {
            at: "/usr/share/alo/translations".to_owned(),
            why: "no such file or directory".to_owned(),
        };
        assert!(none.to_string().contains("/usr/share/alo/translations"));
        assert!(none.to_string().contains("speaks English"));
    }

    /// A second file for one language names **both**, because the thing to do
    /// about it is to put them together and you cannot do that holding one.
    #[test]
    fn a_second_file_for_one_language_names_both_of_them() {
        let already = NotSpoken::AlreadySpoken {
            file: "german.toml".to_owned(),
            language: Language::written("de").unwrap(),
            already: "de.toml".to_owned(),
        };
        let said = already.to_string();
        assert!(said.contains("german.toml"), "{said}");
        assert!(said.contains("de.toml"), "{said}");
    }
}
