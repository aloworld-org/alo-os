//! One answer, and where it came from.
//!
//! **This type is the reason English cannot be shown silently.** A lookup that
//! answered with a `String` would give a Latvian shell an English sentence and
//! nothing anywhere would know it had happened; the release would ship, and the
//! first person to find out would be a person in Latvia. So the answer carries
//! [`CameFrom`], and *shown English because nothing was translated* is a state
//! the caller can see, the development build marks, and a release note counts.
//!
//! There are three things that can have happened, and they are different in
//! kind rather than in degree:
//!
//! - [`CameFrom::Translation`] — somebody translated this, and this is their
//!   sentence. The ordinary case, and the only one that is not a gap.
//! - [`CameFrom::TheSource`] — nobody has translated it yet, so this is the
//!   English the code was written with. Not a bug; work not yet done.
//! - [`CameFrom::NoPhrase`] — the code asked for a key nothing declares. That
//!   *is* a bug, in this repository rather than in a translation, and it is the
//!   one case where a person is shown a key: there is no honest sentence to
//!   show, and a blank space would read like a sentence with nothing to say.

use std::fmt;

use crate::language::Language;

/// One string, ready to be shown, and where it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Said {
    /// The text.
    text: String,
    /// Where it came from.
    came_from: CameFrom,
    /// Gaps in the sentence that the caller gave no value for. Empty is the
    /// ordinary case.
    unfilled: Vec<String>,
}

impl Said {
    /// Made by [`crate::Strings::say`] and by nothing else.
    pub(crate) fn new(text: String, came_from: CameFrom, unfilled: Vec<String>) -> Self {
        Self {
            text,
            came_from,
            unfilled,
        }
    }

    /// The text, which is what goes on the screen.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Where it came from.
    #[must_use]
    pub fn came_from(&self) -> &CameFrom {
        &self.came_from
    }

    /// Whether somebody translated this. `false` means the person is reading
    /// the source language because nobody has translated it yet, or because the
    /// code asked for something nothing says.
    #[must_use]
    pub fn is_translated(&self) -> bool {
        matches!(self.came_from, CameFrom::Translation(_))
    }

    /// Whether this is a mistake in this repository rather than a translation
    /// nobody has done yet.
    #[must_use]
    pub fn is_a_bug(&self) -> bool {
        matches!(self.came_from, CameFrom::NoPhrase) || !self.unfilled.is_empty()
    }

    /// Gaps in the sentence nobody gave a value for, which come out written as
    /// `{name}` rather than disappearing.
    #[must_use]
    pub fn unfilled(&self) -> &[String] {
        &self.unfilled
    }

    /// The text, given away.
    #[must_use]
    pub fn into_text(self) -> String {
        self.text
    }
}

impl fmt::Display for Said {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

/// Where an answer came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CameFrom {
    /// Somebody translated it, into this language. The language is the one the
    /// sentence is actually in, which is not always the one that was asked for:
    /// a person who asked for `pt-BR` and was answered by `pt` is told `pt`,
    /// because that is what is on their screen.
    Translation(Language),
    /// Nobody has translated it, so this is the English the code was written
    /// with.
    TheSource,
    /// The code asked for a key nothing declares. The text is the key.
    NoPhrase,
}

impl fmt::Display for CameFrom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Translation(language) => write!(f, "{language}"),
            Self::TheSource => f.write_str("the source"),
            Self::NoPhrase => f.write_str("nothing"),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    #[test]
    fn a_translated_answer_says_which_language_it_is_in() {
        let said = Said::new(
            "Es ist nicht mehr da".to_owned(),
            CameFrom::Translation(Language::written("de").unwrap()),
            Vec::new(),
        );
        assert_eq!(said.text(), "Es ist nicht mehr da");
        assert!(said.is_translated());
        assert!(!said.is_a_bug());
        assert_eq!(said.came_from().to_string(), "de");
        assert_eq!(said.to_string(), "Es ist nicht mehr da");
    }

    /// **English is never silent.** The sentence may be English, but nothing
    /// about the answer pretends it was translated.
    #[test]
    fn falling_back_to_the_source_is_never_silent() {
        let said = Said::new(
            "It is not there any more".to_owned(),
            CameFrom::TheSource,
            Vec::new(),
        );
        assert!(!said.is_translated());
        assert!(!said.is_a_bug(), "not translated yet is work, not a fault");
        assert_eq!(said.came_from().to_string(), "the source");
    }

    /// A key nothing declares is a mistake in this repository, and it is the one
    /// case where a person is shown a key rather than a sentence.
    #[test]
    fn a_key_nothing_says_is_a_bug_here() {
        let said = Said::new("«files.gone»".to_owned(), CameFrom::NoPhrase, Vec::new());
        assert!(!said.is_translated());
        assert!(said.is_a_bug());
    }

    /// A gap nobody filled is a mistake here too, whatever language the sentence
    /// came from, because the caller and the sentence disagree about what the
    /// sentence is about.
    #[test]
    fn an_unfilled_gap_is_a_bug_here_even_in_a_translation() {
        let said = Said::new(
            "{path} ist kein Ordner".to_owned(),
            CameFrom::Translation(Language::written("de").unwrap()),
            vec!["path".to_owned()],
        );
        assert!(said.is_translated());
        assert!(said.is_a_bug());
        assert_eq!(said.unfilled(), ["path"]);
    }
}
