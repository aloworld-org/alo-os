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
//!
//! # A sentence is as translated as its least translated piece
//!
//! Almost every gap holds data, and data has no language. One kind holds a
//! *word*: an option a verb offers, which is a string somebody translates
//! dropped into the middle of the sentence a person approves
//! (`alo_capability::Offered`). So a sentence's own provenance is not the whole
//! answer any more, and [`Said::gaps_came_from`] carries the rest of it.
//!
//! [`Said::is_translated`] is `true` only when the sentence **and** every word
//! put into it were translated. The alternative was a German approval sentence
//! reading *Blender on the left half of the screen platzieren*, answering that
//! it was translated, and being marked by nothing — which is the failure this
//! whole type exists to make impossible, arriving through a gap instead of
//! through a key.

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
    /// Where each gap filled with a *word* came from. Empty is the ordinary
    /// case, because almost every gap holds data.
    gaps_came_from: Vec<CameFrom>,
}

impl Said {
    /// Made by [`crate::Strings::say`] and by nothing else.
    pub(crate) fn new(text: String, came_from: CameFrom, unfilled: Vec<String>) -> Self {
        Self {
            text,
            came_from,
            unfilled,
            gaps_came_from: Vec::new(),
        }
    }

    /// The same answer, knowing where the words put into it came from.
    ///
    /// Separate from [`Said::new`] because the two lookups that answer without
    /// filling anything — a key nothing declares — have no gaps to say anything
    /// about, and a constructor taking an argument that is always empty there
    /// would read as though they might.
    pub(crate) fn filled_with(mut self, gaps_came_from: Vec<CameFrom>) -> Self {
        self.gaps_came_from = gaps_came_from;
        self
    }

    /// The text, which is what goes on the screen.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Where the sentence itself came from.
    ///
    /// The sentence, not the whole of what is on the screen: a word put into
    /// one of its gaps has a provenance of its own, and
    /// [`Said::gaps_came_from`] is where those are.
    #[must_use]
    pub fn came_from(&self) -> &CameFrom {
        &self.came_from
    }

    /// Where each gap that was filled with a word came from, in the order the
    /// sentence names them.
    ///
    /// Empty whenever every gap held data, which is almost always.
    #[must_use]
    pub fn gaps_came_from(&self) -> &[CameFrom] {
        &self.gaps_came_from
    }

    /// Whether somebody translated this — **all of it**.
    ///
    /// `false` means the person is reading the source language somewhere on
    /// this line: because nobody has translated the sentence yet, because the
    /// code asked for something nothing says, or because a word dropped into
    /// one of its gaps has not been translated. See this module's documentation
    /// for why the last of those counts.
    #[must_use]
    pub fn is_translated(&self) -> bool {
        matches!(self.came_from, CameFrom::Translation(_))
            && self
                .gaps_came_from
                .iter()
                .all(|came_from| matches!(came_from, CameFrom::Translation(_)))
    }

    /// Whether this is a mistake in this repository rather than a translation
    /// nobody has done yet.
    #[must_use]
    pub fn is_a_bug(&self) -> bool {
        matches!(self.came_from, CameFrom::NoPhrase)
            || !self.unfilled.is_empty()
            || self
                .gaps_came_from
                .iter()
                .any(|came_from| matches!(came_from, CameFrom::NoPhrase))
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

    /// **A translated sentence with an English word in it is not a translated
    /// line.** This is the guarantee item 11a is for: without it, the approval
    /// sentence for an arrangement nobody had translated would come back saying
    /// it was German.
    #[test]
    fn a_sentence_is_only_as_translated_as_the_words_put_into_it() {
        let german = CameFrom::Translation(Language::written("de").unwrap());
        let whole = Said::new(
            "Blender auf der linken Bildschirmhälfte platzieren".to_owned(),
            german.clone(),
            Vec::new(),
        )
        .filled_with(vec![german.clone()]);
        assert!(whole.is_translated());
        assert!(!whole.is_a_bug());

        let half = Said::new(
            "Blender on the left half of the screen platzieren".to_owned(),
            german,
            Vec::new(),
        )
        .filled_with(vec![CameFrom::TheSource]);
        assert!(!half.is_translated(), "an English word is still English");
        assert!(!half.is_a_bug(), "not translated yet is work, not a fault");
        assert_eq!(half.gaps_came_from(), [CameFrom::TheSource]);
    }

    /// A word whose key nothing declares is the same bug as a sentence whose
    /// key nothing declares, and it is one wherever it happens to be sitting.
    #[test]
    fn a_word_nothing_says_is_a_bug_wherever_it_is_put() {
        let said = Said::new(
            "Blender «applications.where.left-half» platzieren".to_owned(),
            CameFrom::Translation(Language::written("de").unwrap()),
            Vec::new(),
        )
        .filled_with(vec![CameFrom::NoPhrase]);
        assert!(said.is_a_bug());
        assert!(!said.is_translated());
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
