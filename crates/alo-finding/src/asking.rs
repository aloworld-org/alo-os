//! A query that is not a query, refused before anything is searched.
//!
//! The plan's clause: *a query that is not a query — empty, or longer than a
//! sentence — is refused in words rather than answered with everything.* A
//! [`crate::Query`] cannot be built with no part at all, but a part can be
//! empty of anything to look for: a name of no letters, words of no words.
//! And a part can be longer than anything a person would type into a search
//! box, which is not a question about their files. Both are refused here,
//! with a sentence, and the index is not consulted.
//!
//! # The two bounds
//!
//! [`A_SENTENCE`] is how many words a search by contents takes at most, and
//! [`A_NAME`] is how many characters a part of a name — or one word — can be.
//! Thirty-two words is more than a person types to find a file and fewer
//! than a paragraph pasted into the wrong box; two hundred and fifty-five is
//! the longest name every filesystem this crate walks allows, so a part
//! longer than that is in no name.

use alo_strings::{Filling, Said, Strings};

use crate::query::Query;
use crate::words::{self, Word};

/// The most words a search by contents takes: a sentence.
pub const A_SENTENCE: usize = 32;

/// The most characters a part of a name, or one word, can be: a name.
pub const A_NAME: usize = 255;

/// Why a query was refused before anything was searched.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum NotAsked {
    /// No part of the query asks for anything: an empty name, no words, and
    /// no kind or date either. Answering it would be answering with
    /// everything, which is not an answer.
    #[error("nothing was asked, so nothing was searched")]
    Nothing,

    /// The words asked for run past a sentence.
    #[error("{words} words is more than a sentence; a search takes at most {most}")]
    MoreThanASentence {
        /// How many words were asked for.
        words: usize,
        /// The most a search takes: [`A_SENTENCE`].
        most: usize,
    },

    /// The name part, or one word, is longer than any name on the disk.
    #[error("{chars} characters is longer than any name; a search takes at most {most}")]
    LongerThanAName {
        /// How many characters the part is.
        chars: usize,
        /// The most a search takes: [`A_NAME`].
        most: usize,
    },
}

impl NotAsked {
    /// The word this refusal is said with.
    #[must_use]
    pub fn word(&self) -> &'static Word {
        match self {
            Self::Nothing => &words::NOT_ASKED_NOTHING,
            Self::MoreThanASentence { .. } => &words::NOT_ASKED_MORE_THAN_A_SENTENCE,
            Self::LongerThanAName { .. } => &words::NOT_ASKED_LONGER_THAN_A_NAME,
        }
    }

    /// This refusal, in the language the person reads.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let filling = match self {
            Self::Nothing => Filling::nothing(),
            Self::MoreThanASentence { words, most } => {
                Filling::of("words", words.to_string()).and("most", most.to_string())
            }
            Self::LongerThanAName { chars, most } => {
                Filling::of("chars", chars.to_string()).and("most", most.to_string())
            }
        };
        strings.say(&self.word().key(), &filling)
    }
}

/// Whether this query is one: something is asked, and no more than a person
/// asks.
///
/// # Errors
///
/// [`NotAsked`], naming which.
pub(crate) fn checked(query: &Query) -> Result<(), NotAsked> {
    let name = query.name_asked().map(str::trim).unwrap_or_default();
    let words = query.words();
    if name.is_empty() && words.is_empty() && !query.asks_kind_or_date() {
        return Err(NotAsked::Nothing);
    }
    if words.len() > A_SENTENCE {
        return Err(NotAsked::MoreThanASentence {
            words: words.len(),
            most: A_SENTENCE,
        });
    }
    let longest = std::iter::once(name)
        .chain(words.iter().map(String::as_str))
        .map(|part| part.chars().count())
        .max()
        .unwrap_or(0);
    if longest > A_NAME {
        return Err(NotAsked::LongerThanAName {
            chars: longest,
            most: A_NAME,
        });
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use std::time::SystemTime;

    use super::*;
    use crate::kind::Kind;

    /// **A query with nothing in it is refused**, however it was emptied —
    /// and one with a kind or a date in it is a question even with no text.
    #[test]
    fn a_query_that_asks_nothing_is_refused_and_one_with_a_kind_or_date_is_not() {
        assert_eq!(checked(&Query::named("")), Err(NotAsked::Nothing));
        assert_eq!(checked(&Query::named("   ")), Err(NotAsked::Nothing));
        assert_eq!(checked(&Query::saying("")), Err(NotAsked::Nothing));
        assert_eq!(
            checked(&Query::saying(", \u{2014} !")),
            Err(NotAsked::Nothing),
            "punctuation is not a word"
        );
        assert_eq!(
            checked(&Query::named("").and_saying("")),
            Err(NotAsked::Nothing)
        );

        assert_eq!(checked(&Query::named("a")), Ok(()));
        assert_eq!(checked(&Query::saying("contract")), Ok(()));
        assert_eq!(checked(&Query::of_kind(Kind::Pdf)), Ok(()));
        assert_eq!(
            checked(&Query::changed_since(SystemTime::UNIX_EPOCH)),
            Ok(())
        );
        assert_eq!(
            checked(&Query::changed_before(SystemTime::UNIX_EPOCH)),
            Ok(())
        );
        assert_eq!(
            checked(&Query::named("").and_of_kind(Kind::Text)),
            Ok(()),
            "an empty name beside a kind asks for the kind"
        );
    }

    /// **More than a sentence is refused**, at exactly the bound.
    #[test]
    fn more_than_a_sentence_is_refused_at_the_bound() {
        let a_sentence = (0..A_SENTENCE)
            .map(|n| format!("w{n}"))
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(checked(&Query::saying(&a_sentence)), Ok(()));
        let one_more = format!("{a_sentence} more");
        assert_eq!(
            checked(&Query::saying(&one_more)),
            Err(NotAsked::MoreThanASentence {
                words: A_SENTENCE + 1,
                most: A_SENTENCE,
            })
        );
        assert_eq!(
            checked(&Query::saying(&format!("{a_sentence} {a_sentence}"))),
            Ok(()),
            "the same words twice are the same words"
        );
    }

    /// **Longer than a name is refused**, in the name part and in a word,
    /// counted in characters rather than bytes.
    #[test]
    fn longer_than_a_name_is_refused_in_a_name_part_and_in_a_word() {
        let a_name: String = "\u{142}".repeat(A_NAME);
        assert_eq!(checked(&Query::named(&a_name)), Ok(()));
        assert_eq!(checked(&Query::saying(&a_name)), Ok(()));
        let one_more = format!("{a_name}\u{142}");
        let refused = NotAsked::LongerThanAName {
            chars: A_NAME + 1,
            most: A_NAME,
        };
        assert_eq!(checked(&Query::named(&one_more)), Err(refused.clone()));
        assert_eq!(checked(&Query::saying(&one_more)), Err(refused.clone()));
        assert_eq!(
            checked(&Query::of_kind(Kind::Pdf).and_named(&one_more)),
            Err(refused),
            "a kind beside it does not make the name shorter"
        );
    }

    /// Every refusal is a sentence a person reads, with nothing unfilled.
    #[test]
    fn every_refusal_is_said_in_a_sentence() {
        let strings = Strings::of(crate::finding_words().unwrap());
        for refusal in [
            NotAsked::Nothing,
            NotAsked::MoreThanASentence {
                words: 40,
                most: 32,
            },
            NotAsked::LongerThanAName {
                chars: 300,
                most: 255,
            },
        ] {
            let said = refusal.said(&strings);
            assert!(!said.is_a_bug(), "{said}");
            assert!(said.unfilled().is_empty(), "{said}");
            assert!(!said.text().starts_with("finding."), "{said}");
        }
        assert!(
            NotAsked::MoreThanASentence {
                words: 40,
                most: 32
            }
            .said(&strings)
            .text()
            .contains("40 words")
        );
    }
}
