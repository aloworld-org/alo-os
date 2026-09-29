//! Every string this crate can say, and the English beside each one.
//!
//! Three of them are the states in [`crate::Reaching`], read in a list of the
//! person's own machines. The fourth and fifth are the two ways adding a machine
//! is refused.
//!
//! # The one that matters is the one nobody plans for
//!
//! [`DID_NOT_ANSWER`] and [`NOT_ASKED_YET`] are two sentences a careless
//! translation would merge, and merging them is the whole fault this crate
//! exists to prevent. *It did not answer* is a machine that may be off, may be
//! elsewhere, may have a broken network — something to act on. *Not asked yet*
//! is a machine added thirty seconds ago and nothing is wrong at all.
//!
//! A translator handed both at once will reasonably wonder whether one word
//! would do. Both notes say outright that it would not, because the person
//! reading the merged version goes looking for a fault that does not exist.
//!
//! The shape is `alo-dock`'s, one crate on: constants, `alo_strings::Word`, and
//! a test at the bottom putting every key back through the vocabulary.

use alo_strings::Vocabulary;

/// One string a crate can say. Re-exported so this crate's own files name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// How a machine is reaching — [`crate::Reaching`]. Read beside each machine in
// the person's list of their own machines.
// ---------------------------------------------------------------------------

/// The key of [`ANSWERED_WORD`], so [`crate::Reaching`] names it without the
/// whole string.
pub const ANSWERED: &str = "elsewhere.reaching.answered";

/// The key of [`DID_NOT_ANSWER_WORD`].
pub const DID_NOT_ANSWER: &str = "elsewhere.reaching.did-not-answer";

/// The key of [`NOT_ASKED_YET_WORD`].
pub const NOT_ASKED_YET: &str = "elsewhere.reaching.not-asked-yet";

/// A machine that answered when this one last asked.
pub const ANSWERED_WORD: Word = Word::saying(ANSWERED, "Answered").noting(
    "Beside one machine in a list of the person's own machines. It means this machine reached \
     that one when it last asked, and nothing more — not that the other machine is idle, healthy \
     or free. Keep it to one or two words: it sits in a row beside a name, not in a sentence.",
);

/// A machine that was asked and did not answer.
pub const DID_NOT_ANSWER_WORD: Word = Word::saying(DID_NOT_ANSWER, "Did not answer").noting(
    "Beside one machine in a list of the person's own machines, and the one row they may need to \
     act on: the machine may be off, may be somewhere else, may have lost its network. It must \
     NOT be merged with the *not asked yet* string into one word for *unavailable*. Somebody \
     reading the merged version for a machine nobody has asked goes looking for a fault that \
     does not exist. The machine is still listed and still theirs; this says what happened when \
     it was asked, not that it is gone.",
);

/// A machine nobody has asked since it was added.
pub const NOT_ASKED_YET_WORD: Word = Word::saying(NOT_ASKED_YET, "Not asked yet").noting(
    "Beside one machine in a list of the person's own machines, just after they add it. Nothing \
     is wrong: adding a machine is not asking it anything. It must NOT be merged with the *did \
     not answer* string — this one is reassuring and that one is not, and a translation that \
     makes them the same word turns a calm state into an alarming one thirty seconds after \
     somebody added a machine that is fine.",
);

// ---------------------------------------------------------------------------
// The two ways adding a machine is refused — [`crate::NotElsewhere`]. Read at
// the moment a person is adding one.
// ---------------------------------------------------------------------------

/// The key of [`UNNAMED_WORD`].
pub const UNNAMED: &str = "elsewhere.not-added.unnamed";

/// The key of [`NAME_TOO_LONG_WORD`].
pub const NAME_TOO_LONG: &str = "elsewhere.not-added.name-too-long";

/// The key of [`ALREADY_ADDED_WORD`].
pub const ALREADY_ADDED: &str = "elsewhere.not-added.already-added";

/// A machine cannot be added without a name.
pub const UNNAMED_WORD: Word = Word::saying(UNNAMED, "Give this machine a name you will recognise")
    .noting(
        "Shown where a person is adding one of their own machines and has left the name empty. \
         It asks rather than scolds: the name is for them, not for the machine, and nobody else \
         ever sees it. Not *the name is required* — that says whose rule it is rather than what \
         it is for.",
    );

/// A name past what a list can show.
pub const NAME_TOO_LONG_WORD: Word = Word::saying(
    NAME_TOO_LONG,
    "That name is longer than {at_most} characters, so it will not fit in the list",
)
.noting(
    "Shown while a person is typing a name for one of their own machines. It says what will \
     happen rather than that a rule was broken, because the limit exists so a list of machines \
     stays scannable. {at_most} is a plain whole number with no sign or separator on it — how a \
     number is written belongs to the region rather than to the language.",
);

/// A machine already on the person's list.
pub const ALREADY_ADDED_WORD: Word = Word::saying(
    ALREADY_ADDED,
    "You have already added this machine, under the name {called}",
)
.noting(
    "Shown when a person adds a machine that is already on their list. It names what they called \
     it the first time, because the likeliest reason they are adding it again is that they did \
     not recognise it under that name. Not an error about duplicates — it is the answer to *have \
     I added this one?* {called} is the name they gave it, which is theirs and never translated.",
);

/// Every string this crate can say.
pub const EVERY_WORD: [Word; 6] = [
    ANSWERED_WORD,
    DID_NOT_ANSWER_WORD,
    NOT_ASKED_YET_WORD,
    UNNAMED_WORD,
    NAME_TOO_LONG_WORD,
    ALREADY_ADDED_WORD,
];

/// What can go wrong declaring this crate's words.
///
/// A `Result` rather than an unwrap because a library that panics on its own
/// string table takes the shell with it, and because [`declare_into`] can
/// genuinely fail against a vocabulary that already holds one of these keys.
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

/// A vocabulary holding only this crate's words.
///
/// # Errors
/// [`WordsError::List`] if two of these declared the same key, which the test
/// at the bottom of this file exists to catch first.
pub fn elsewhere_words() -> Result<Vocabulary, WordsError> {
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
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    /// No two words share a key, which a vocabulary would refuse at run time
    /// and this catches at test time.
    #[test]
    fn every_key_is_its_own() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(named.len(), EVERY_WORD.len(), "two words share a key");
    }

    /// Every key is in this crate's own area, so adding them to the shell's one
    /// vocabulary cannot collide with another crate's.
    #[test]
    fn every_key_is_in_this_crates_area() {
        for word in EVERY_WORD {
            assert!(
                word.named().starts_with("elsewhere."),
                "{} is outside this crate's area",
                word.named()
            );
        }
    }

    /// Every string carries a note, because a word with no note is where a
    /// translation goes wrong quietly.
    #[test]
    fn every_word_carries_a_note() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// **The two sentences a translation must not merge say so to the
    /// translator.** This is the fault this crate exists to prevent, one layer
    /// out: if the distinction only lives in Rust, the shipped product loses it
    /// in every language but English.
    #[test]
    fn the_two_states_that_must_not_be_merged_tell_the_translator_so() {
        for word in [DID_NOT_ANSWER_WORD, NOT_ASKED_YET_WORD] {
            let note = word.note().expect("a note");
            assert!(
                note.contains("must NOT be merged"),
                "{} does not warn the translator",
                word.named()
            );
        }
    }

    /// The sentences that take a value name it, so a translator knows what will
    /// be put there.
    #[test]
    fn the_sentences_with_a_gap_in_them_name_it() {
        assert!(NAME_TOO_LONG_WORD.says().contains("{at_most}"));
        assert!(ALREADY_ADDED_WORD.says().contains("{called}"));
    }

    /// They all go into one vocabulary, which is what the shell does with them.
    #[test]
    fn they_all_go_into_one_vocabulary() {
        let vocabulary = elsewhere_words().expect("no key is declared twice");
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
    }
}
