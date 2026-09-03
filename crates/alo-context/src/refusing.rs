//! Why something on a person's screen could not be offered to an agent, as a
//! value rather than as a sentence.
//!
//! One type, because everything here refuses the same kind of thing: a **part**
//! of what an invocation offered. Nothing in this file refuses a call — a call
//! is refused by the grants, in their own words, wherever it is made.
//!
//! It is a value with a [`NotOffered::said`], which is item 9e's rule met
//! again: what may be offered never depends on a vocabulary having been loaded,
//! and the words are made once, where the question was answered.
//!
//! # What is not a refusal
//!
//! **Nothing selected, nothing open and nothing in front is not an error.** A
//! person who presses the key on an empty desktop has offered nothing, and an
//! invocation that offered nothing is still an invocation — see
//! [`crate::Context`]. Only something that *was* offered and could not be is
//! refused here.

use alo_strings::{Filling, Said, Strings};

use crate::words;

/// Why a part of what was on the screen could not be offered.
///
/// Read by whoever is looking at why an agent was not told about the document
/// they had open — which is a person in front of the machine — so it is said in
/// their language like everything else here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotOffered {
    /// A window belonging to no application at all.
    NoWindow,
    /// A window whose application could never arrive as an argument.
    NotAnIdentifier {
        /// The identifier as it was offered.
        offered: String,
    },
    /// A document with no path.
    NoDocument,
    /// A relative path, which means a different file depending on where it is
    /// read from.
    NotAFullPath {
        /// The path as it was offered.
        offered: String,
    },
    /// A path containing `..`, which can leave the folder it appears to be in.
    CouldLeadElsewhere {
        /// The path as it was offered.
        offered: String,
    },
    /// The whole machine, offered as though it were a document.
    NotADocument {
        /// The path as it was offered.
        offered: String,
    },
}

impl NotOffered {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub fn word(&self) -> words::Word {
        match self {
            Self::NoWindow => words::NO_WINDOW,
            Self::NotAnIdentifier { .. } => words::NOT_AN_IDENTIFIER,
            Self::NoDocument => words::NO_DOCUMENT,
            Self::NotAFullPath { .. } => words::NOT_A_FULL_PATH,
            Self::CouldLeadElsewhere { .. } => words::COULD_LEAD_ELSEWHERE,
            Self::NotADocument { .. } => words::NOT_A_DOCUMENT,
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics: a `Strings` that was never given
    /// [`crate::context_words`] answers with the key, marked.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = match self {
            Self::NoWindow | Self::NoDocument => Filling::nothing(),
            Self::NotAnIdentifier { offered } => Filling::of("application", offered.clone()),
            Self::NotAFullPath { offered }
            | Self::CouldLeadElsewhere { offered }
            | Self::NotADocument { offered } => Filling::of("document", offered.clone()),
        };
        strings.say(&self.word().key(), &filling)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};

    /// Each refusal says what to do about the thing it refused, and names it.
    #[test]
    fn a_refusal_says_what_to_do_and_names_what_it_refused() {
        let strings = in_english();

        let relative = NotOffered::NotAFullPath {
            offered: "march.pdf".to_owned(),
        }
        .said(&strings);
        assert!(relative.text().contains("march.pdf"), "{relative}");
        assert!(
            relative.text().contains("offer the whole path"),
            "{relative}"
        );

        let upwards = NotOffered::CouldLeadElsewhere {
            offered: "/home/anna/../root/notes.txt".to_owned(),
        }
        .said(&strings);
        assert!(
            upwards.text().contains("could lead somewhere else"),
            "{upwards}"
        );

        let machine = NotOffered::NotADocument {
            offered: "/".to_owned(),
        }
        .said(&strings);
        assert!(machine.text().contains("the whole machine"), "{machine}");
        assert!(
            machine.text().contains("grant over exactly that file"),
            "{machine}"
        );

        let window = NotOffered::NotAnIdentifier {
            offered: "/usr/bin/blender".to_owned(),
        }
        .said(&strings);
        assert!(window.text().contains("/usr/bin/blender"), "{window}");
        assert!(window.text().contains("no folders in it"), "{window}");

        assert!(
            NotOffered::NoWindow
                .said(&strings)
                .text()
                .contains("name the application")
        );
        assert!(
            NotOffered::NoDocument
                .said(&strings)
                .text()
                .contains("or offer nothing")
        );
    }

    /// **The path is the machine's and the words around it are the reader's.**
    /// The same rule a filename is held to in `alo-files`.
    #[test]
    fn the_words_around_a_path_are_translated_and_it_is_not() {
        let strings = translated(&[(
            words::NOT_A_FULL_PATH,
            "{document} ist kein vollständiger Pfad — geben Sie den ganzen Pfad an",
        )]);
        let said = NotOffered::NotAFullPath {
            offered: "march.pdf".to_owned(),
        }
        .said(&strings);
        assert!(said.is_translated());
        assert_eq!(
            said.text(),
            "march.pdf ist kein vollständiger Pfad — geben Sie den ganzen Pfad an"
        );
    }

    /// **A refusal never depends on a string table.** With no words at all it
    /// refuses exactly as firmly and answers with the key, marked.
    #[test]
    fn a_refusal_without_the_words_still_names_the_rule() {
        let strings = Strings::of(alo_strings::Vocabulary::empty());
        let said = NotOffered::NoDocument.said(&strings);
        assert!(said.is_a_bug());
        assert!(
            said.text().contains("context.document.nothing-named"),
            "{said}"
        );
    }

    /// Every variant has a word, and no two variants share one — a refusal
    /// wearing another refusal's sentence would send somebody to the wrong
    /// place.
    #[test]
    fn every_refusal_has_a_sentence_of_its_own() {
        let every = [
            NotOffered::NoWindow,
            NotOffered::NotAnIdentifier {
                offered: String::new(),
            },
            NotOffered::NoDocument,
            NotOffered::NotAFullPath {
                offered: String::new(),
            },
            NotOffered::CouldLeadElsewhere {
                offered: String::new(),
            },
            NotOffered::NotADocument {
                offered: String::new(),
            },
        ];
        let mut named: Vec<String> = every
            .iter()
            .map(|why| why.word().named().to_owned())
            .collect();
        named.sort();
        named.dedup();
        assert_eq!(named.len(), every.len());
    }
}
