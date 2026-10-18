//! Every string this crate can say, and the English beside each one.
//!
//! The record itself is not words: an entry holds [`crate::Line`]s that were
//! already worded by whoever made them, in the language the person was told
//! them in. What needs declaring is the **one verb** this crate offers — its
//! purpose, its argument and the sentence a person reads while it runs — and
//! the two sentences its answer always carries.
//!
//! # A note is part of the string
//!
//! A translator works alone, with no product in front of them. Where a
//! sentence cannot be translated from its own words — *the record* is a thing
//! this machine keeps, and *alo itself* is the agent rather than the person —
//! the note says so.

use alo_strings::Vocabulary;

/// One string a crate can say.
///
/// Re-exported because this crate's own files, and the tests that read this
/// list, name it as `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// The one verb: what it does, what a person is shown, and what its argument
// is for. `crate::verbs` declares it from these.
// ---------------------------------------------------------------------------

/// What `what_touched_this_file` does.
pub const WHAT_TOUCHED_THIS_FILE: Word = Word::saying(
    "record.verb.what-touched-this-file.purpose",
    "look in the record for what was done to a file",
)
.noting(
    "The one-sentence purpose of the record verb, shown where a person reads what their agent \
     can do. \"The record\" is the list this machine keeps of everything the agent did and was \
     stopped from doing; it is not the person's own shell history.",
);

/// What a person is shown when `what_touched_this_file` runs.
///
/// It is a read, so nobody is asked to approve it — this is what the shell
/// shows while it happens.
pub const WHAT_TOUCHED_THIS_FILE_SENTENCE: Word = Word::saying(
    "record.verb.what-touched-this-file.sentence",
    "look in the record for what was done to {file}",
)
.noting("{file} is a path and is never translated.");

/// What `what_touched_this_file` is asked about.
pub const WHAT_TOUCHED_THIS_FILE_FILE: Word = Word::saying(
    "record.verb.what-touched-this-file.argument.file",
    "the file to look for in the record",
)
.noting(
    "Shown beside the argument when an agent asks for it. A whole path, as the person would \
     write it, and not part of a name: the record is searched for calls that named this exact \
     file.",
);

// ---------------------------------------------------------------------------
// The two sentences every answer carries, which no shape of the answer can
// leave out. `crate::what_touched_a_file` says why each is not optional.
// ---------------------------------------------------------------------------

/// That the answer covers what alo did, and not what the person did.
pub const ONLY_WHAT_ALO_DID: Word = Word::saying(
    "record.only-what-alo-did",
    "This is what alo did. Commands you ran yourself are not in it.",
)
.noting(
    "Shown with every answer to the record verb, and it is not optional. The record holds what \
     the agent did; a command the person typed into a terminal themselves leaves an entry \
     saying it ran, and that entry does not say which files it changed. So an empty answer \
     means the agent did nothing to the file, and never that nothing did. \"alo\" is the \
     product's name and is never translated.",
);

/// That nothing in the record names this file.
pub const NOTHING_IN_THE_RECORD_TOUCHED_IT: Word = Word::saying(
    "record.nothing-touched-it",
    "Nothing in the record names {file}.",
)
.noting(
    "Said instead of a list when the record holds no call that named this file. {file} is a \
     path and is never translated. Read together with \"record.only-what-alo-did\", which says \
     what the record covers: this sentence reports an empty search and never that the file was \
     untouched.",
);

// ---------------------------------------------------------------------------
// What is said when the verb was asked for wrongly. Both are read by whoever
// wrote the caller rather than by whoever asked.
// ---------------------------------------------------------------------------

/// A verb that is not this crate's to answer.
pub const NOT_THIS_CRATES: Word = Word::saying(
    "record.verb.not-this-crates",
    "nothing in the record does {verb} — it answers what was done, and anything else is \
     somewhere else's to do",
)
.noting("{verb} is the name of a capability and is never translated.");

/// A verb performed without an argument it declares.
pub const MISSING: Word = Word::saying(
    "record.verb.missing",
    "{verb} was performed without {argument}, which it declares — this is a verb to write \
     again, not a call to make again",
)
.noting(
    "Both gaps are names in the code and are never translated. This sentence is read by whoever \
     wrote the verb rather than by whoever asked for it.",
);

/// Everything this crate can say in one sentence each.
pub const EVERY_WORD: [Word; 7] = [
    WHAT_TOUCHED_THIS_FILE,
    WHAT_TOUCHED_THIS_FILE_SENTENCE,
    WHAT_TOUCHED_THIS_FILE_FILE,
    ONLY_WHAT_ALO_DID,
    NOTHING_IN_THE_RECORD_TOUCHED_IT,
    NOT_THIS_CRATES,
    MISSING,
];

/// Why this crate's list could not be declared.
#[derive(Debug, thiserror::Error)]
pub enum WordsError {
    /// A word that is not a phrase.
    #[error(transparent)]
    Word(#[from] alo_strings::WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] alo_strings::VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
///
/// [`WordsError`], which the list above cannot cause.
pub fn record_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// # Errors
///
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
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::{EVERY_WORD, WordsError, declare_into, record_words};
    use alo_strings::{Key, Vocabulary};

    /// Every key in this file is a key, checked here because a key written in
    /// this file cannot arrive from anywhere else.
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(Key::named(word.named()), Ok(word.key()), "{}", word.named());
        }
    }

    /// A key names one string, and the vocabulary holds as many as the list.
    ///
    /// **The count is the list's own length** rather than a number written
    /// here: a number would have to be edited beside every word added, and an
    /// edit that is only ever made to keep a test quiet is not a check.
    #[test]
    fn the_list_declares_into_a_vocabulary_once() {
        assert_eq!(record_words().unwrap().how_many(), EVERY_WORD.len());
        let mut vocabulary = Vocabulary::empty();
        declare_into(&mut vocabulary).unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **Every string here carries a note.** A translator with no product in
    /// front of them cannot tell that *the record* is a thing this machine
    /// keeps, or that an empty answer is not a statement about the file.
    #[test]
    fn every_string_carries_a_note_for_whoever_translates_it() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// Every key is under this crate's own area, so nothing here can collide
    /// with another crate's list by accident.
    #[test]
    fn every_key_is_this_crates_own() {
        for word in EVERY_WORD {
            assert!(
                word.named().starts_with("record."),
                "{} is not under this crate's area",
                word.named()
            );
        }
    }
}
