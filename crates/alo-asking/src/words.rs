//! Every string this crate can say, and the English beside each one.
//!
//! `CLAUDE.md` says hardcoded English is a bug. This is the list that stops it
//! being one here: a key, the sentence in the language the code is written in,
//! and the note a translator needs. `alo-strings` does the rest.
//!
//! # It is the shortest list in this workspace, and that is the design
//!
//! Two strings, in the crate that talks to the network more than any other.
//! Everything a person reads around a question was already somebody else's to
//! say, and this crate deliberately does not say any of it a second time:
//!
//! | What a person reads | Whose string it is |
//! |---|---|
//! | *@mail is asking a question of alo, in the EU* | `alo-egress`, on the indicator |
//! | *by alo, in the EU* | `alo-models`, beside the answer |
//! | *nothing was answered by alo, in the EU — it answered 503…* | `alo-answering`, when it failed |
//! | *nothing was sent anywhere, and nothing will be unless you say so* | `alo-answering`, always |
//! | *this machine is set to keep questions in the building…* | `alo-models`, when the rule refused |
//!
//! A second rendering of any of those would be a machine able to describe one
//! moment two ways, which is the failure the 9-series spent six items
//! removing. What is left for this crate to say is the only thing none of them
//! knows about: that there is nothing here to ask with yet.
//!
//! **Item 18a added a second door and no string.** Everything a person reads
//! about a question answered on this machine was already somebody's:
//! *on this machine* is `alo-models`', *nothing answered on this machine* is
//! `alo-answering`'s, and the refusals [`crate::Miswired`] makes are read by
//! whoever wired the door rather than by anybody using the machine. A list that
//! grew with every path would be a list that had started saying things twice.
//!
//! # Nothing here counts, and nothing here quotes
//!
//! There is no [`alo_strings::Plural`], for the reason `alo-models` gives: a
//! sentence that said *2 models* would be English's two shapes standing in for
//! Polish's three. And no gap in this file holds text anybody outside alo OS
//! wrote — not the question, not the model's name, not a syllable of what a
//! provider answered.

use alo_strings::Vocabulary;

/// One string a crate can say.
///
/// Re-exported because this crate's own files, and the tests that read this
/// list, name it as `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// What was typed is not a question yet — [`crate::NotAQuestion`].
//
// Read by somebody looking at the box they were about to ask something in, so
// both of them say what to do rather than what is missing.
// ---------------------------------------------------------------------------

/// Nothing was asked.
pub const NOTHING_TO_ASK: Word = Word::saying(
    "asking.question.nothing",
    "there is nothing to ask yet — write the question first",
)
.noting(
    "Read beside an empty box. It is not a failure and must not sound like one: nothing has gone \
     wrong and nothing has been sent.",
);

/// No model was named to answer it.
pub const NO_MODEL_NAMED: Word = Word::saying(
    "asking.question.no-model",
    "choose a model for this question to be answered by",
)
.noting(
    "A model is the thing that answers — one on this machine, or one the provider offers. The name \
     itself is the provider's or the catalogue's and is never translated, which is why it is not \
     in this sentence.",
);

/// Every string this crate can say, in the order this file declares them.
///
/// The array is what a test reads down and what [`declare_into`] walks, so a
/// word declared above and left out here is a string nothing can look up.
pub const EVERY_WORD: [Word; 2] = [NOTHING_TO_ASK, NO_MODEL_NAMED];

/// Why this crate's own list could not be declared.
///
/// Not a refusal a person reads — it is read by whoever is fixing the list
/// above, so it keeps its English and its `Display` for the reason
/// `alo_models::CatalogueError` does.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase: a sentence that is not one, or a note that
    /// could not be attached.
    #[error(transparent)]
    Word(#[from] alo_strings::WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] alo_strings::VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn asking_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// The shell has one vocabulary and every crate adds its own to it, which is
/// what the area at the front of a key is for.
///
/// # Errors
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced, because a key means one string and whoever declared it
/// first said what that string is.
pub fn declare_into(vocabulary: &mut Vocabulary) -> Result<(), WordsError> {
    for word in EVERY_WORD {
        vocabulary.says(word.phrase()?)?;
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use alo_strings::Key;
    use std::collections::BTreeSet;

    /// **What we ship is held to the rule everybody else is held to.**
    /// [`Word::key`] does not check, because a key written in this file cannot
    /// arrive from anywhere; this is the test that makes that true.
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(
                Key::named(word.named()),
                Ok(word.key()),
                "{}: {}",
                word.named(),
                Key::named(word.named()).unwrap_err()
            );
        }
    }

    /// A key names one string. Two words sharing one would mean whichever was
    /// declared second is a string nobody can reach.
    #[test]
    fn no_two_words_are_named_the_same() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// Every one of them is in the area a reader can sort by, which is what
    /// lets one vocabulary hold every crate's strings.
    #[test]
    fn everything_this_crate_says_says_it_is_this_crate() {
        for word in EVERY_WORD {
            assert_eq!(word.key().area(), "asking", "{}", word.named());
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it.
    #[test]
    fn the_whole_list_declares() {
        let vocabulary = asking_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert_eq!(vocabulary.counted().count(), 0);
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = asking_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **Neither sentence has a gap in it**, and that is the strong form of
    /// this crate's rule about what it holds: a question and an answer are the
    /// two things it carries and the two things no sentence of its own can be
    /// handed, so a translation cannot invent a gap to put either into —
    /// `alo-strings` refuses a gap the source does not have.
    #[test]
    fn neither_sentence_has_a_gap_for_anything_to_be_put_into() {
        for word in EVERY_WORD {
            assert!(!word.says().contains('{'), "{}", word.named());
        }
    }

    /// **Nothing this crate says counts anything.** A gap named for a quantity
    /// would be English's two shapes standing in for Polish's three, and the
    /// plural rules are not written from memory.
    #[test]
    fn nothing_this_crate_says_counts_something() {
        let counting = asking_words().unwrap();
        assert_eq!(counting.counted().count(), 0);
    }

    /// Both carry a note. Neither is a sentence a translator can work out from
    /// its own words: one is read beside an empty box and must not sound like
    /// a failure, and the other is about a name that is never translated.
    #[test]
    fn both_of_them_carry_a_note() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }
}
