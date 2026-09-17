//! Every string this crate can say, and the English beside each one.
//!
//! Two kinds: what the pointer says while a person is still holding something —
//! the label that tells them what letting go would do, and what a screen reader
//! announces as they cross a window — and why nothing arrived.
//!
//! The shape is `alo-shortcuts`' and is copied rather than re-decided.
//!
//! # The sentence this crate exists for
//!
//! [`ASK_ABOUT_THIS`] and [`OFFERED_FOR_THIS_QUESTION`] are the two the product
//! is answerable for. A person dragging an invoice onto the agent has to be able
//! to read, in their own language, that they have offered it **for this question
//! and not handed the agent their folder** — because that is the promise ADR
//! 0001 §3 makes, and a promise nobody can read is a promise in a policy
//! document.
//!
//! # No sentence here names an application or a file
//!
//! A refusal is about what somebody dragged and where they dropped it, and none
//! of these says which file or which window. The shell knows both — it drew
//! them a moment ago — and a sentence that repeated them would be alo OS
//! reading a person their own filename back. What an *application* got wrong is
//! kept beside the sentence in [`crate::uri_list::NotAFileList`], in English,
//! for whoever is fixing that application.

use alo_strings::Vocabulary;

/// One string a crate can say.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// What letting go would do — [`crate::Over`]. Labels, shown while the thing is
// still held, so a person can let go somewhere else.
// ---------------------------------------------------------------------------

/// [`crate::Over::WouldHandItOver`], copying.
pub const COPY_HERE: Word = Word::saying("handing.would.copy-here", "Copy here").noting(
    "Shown beside the pointer while a person is dragging something over a window that will take \
     it: letting go puts a copy here and leaves the original where it was. Nothing has moved yet \
     when this is shown.",
);

/// [`crate::Over::WouldHandItOver`], moving.
pub const MOVE_HERE: Word = Word::saying("handing.would.move-here", "Move here").noting(
    "As handing.would.copy-here, except that letting go moves the thing rather than copying it: \
     it will no longer be where it was dragged from.",
);

/// [`crate::Over::WouldOfferItForThisQuestion`].
pub const ASK_ABOUT_THIS: Word = Word::saying("handing.would.ask-about-this", "Ask about this")
    .noting(
        "Shown while a person drags a file or some text over the agent's own panel: letting go \
         offers it to the agent for the question they are about to ask. It does not give the \
         agent the file itself or the folder it is in — handing.offered-for-this-question is the \
         sentence that says so once they have let go.",
    );

/// [`crate::Over::WouldNotTakeIt`].
pub const WILL_NOT_TAKE_IT: Word = Word::saying(
    "handing.would.not-take-it",
    "This window does not take this",
)
.noting(
    "Shown while a person drags something over a window that cannot accept it — a picture over a \
     window that only takes text. Letting go here does nothing.",
);

/// What a person is told once a file has been dropped on the agent's surface.
pub const OFFERED_FOR_THIS_QUESTION: Word = Word::saying(
    "handing.offered-for-this-question",
    "offered for this question only — the agent has not been given the file, or the folder it is \
     in",
)
.noting(
    "Shown after a person drops a file onto the agent's panel. It is the promise alo OS makes \
     about a drop: what the agent can read is what was dropped, while this one question lasts, \
     and dropping something never gives the agent lasting access to anything. Translators: *the \
     agent* is the assistant built into the machine.",
);

// ---------------------------------------------------------------------------
// Why nothing arrived — [`crate::NotHanded`]. Each says what did not happen,
// because a person reading it has just let go of something.
// ---------------------------------------------------------------------------

/// [`crate::NotHanded::NothingHere`].
pub const NOTHING_HERE: Word = Word::saying(
    "handing.refused.nothing-here",
    "there is nothing here to drop this on, so it has stayed where it was",
)
.noting("Said when a person lets go over the empty desktop or over something that takes nothing.");

/// [`crate::NotHanded::WillNotTakeIt`].
pub const WILL_NOT_TAKE_IT_AFTER: Word = Word::saying(
    "handing.refused.will-not-take-it",
    "this window does not take what you dropped on it, so nothing has moved",
)
.noting(
    "Said when a person lets go over a window that cannot accept the sort of thing they were \
     dragging. alo OS does not convert it into something that window would take, because that \
     would hand it something the application it came from never said it could produce.",
);

/// [`crate::NotHanded::NotThatForm`].
pub const NOT_THAT_FORM: Word = Word::saying(
    "handing.refused.not-that-form",
    "this cannot be handed over in that form, so nothing has moved",
)
.noting(
    "Said when something asks for what was dropped in a form the application it came from never \
     offered. What has gone wrong is usually in the application that asked.",
);

/// [`crate::NotHanded::DidNotHandItOver`].
pub const DID_NOT_HAND_IT_OVER: Word = Word::saying(
    "handing.refused.did-not-hand-it-over",
    "the application this came from did not hand it over, so nothing has moved",
)
.noting(
    "Said when the application a person dragged from stopped, or failed, between the moment they \
     picked the thing up and the moment they let go.",
);

/// [`crate::NotHanded::NotAFileList`].
pub const NOT_A_FILE_LIST: Word = Word::saying(
    "handing.refused.not-a-file-list",
    "these files could not be handed over, because the application they came from did not say \
     properly which files they were",
)
.noting(
    "Said when the list of files an application put up cannot be read. Nothing is delivered from \
     part of it: three files out of four would be a drop that lost one without saying so.",
);

/// [`crate::NotHanded::TheQuestionIsOver`].
pub const THE_QUESTION_IS_OVER: Word = Word::saying(
    "handing.refused.the-question-is-over",
    "the question this was dropped into is over, so it has not been read",
)
.noting(
    "Said when something tries to read a file that was dropped on the agent's panel after that \
     question has finished. What is dropped on the agent lasts for one question and no longer.",
);

/// Every string this crate can say, in the order a translator meets them.
pub const EVERY_WORD: [Word; 11] = [
    COPY_HERE,
    MOVE_HERE,
    ASK_ABOUT_THIS,
    WILL_NOT_TAKE_IT,
    OFFERED_FOR_THIS_QUESTION,
    NOTHING_HERE,
    WILL_NOT_TAKE_IT_AFTER,
    NOT_THAT_FORM,
    DID_NOT_HAND_IT_OVER,
    NOT_A_FILE_LIST,
    THE_QUESTION_IS_OVER,
];

/// Why this crate's own words could not be declared.
///
/// Nothing in the list above can cause one; the tests at the bottom of this file
/// are what say so.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
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
/// [`WordsError`], which the list above cannot cause.
pub fn handing_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// # Errors
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced.
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
    use std::collections::BTreeSet;

    /// Every key written here is a key by the rule every other key is held to.
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(
                alo_strings::Key::named(word.named()),
                Ok(word.key()),
                "{}",
                word.named()
            );
            assert_eq!(word.key().area(), "handing", "{}", word.named());
        }
    }

    /// No two words share a key, and the whole list declares.
    #[test]
    fn the_whole_list_declares_and_no_two_are_named_the_same() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
        let vocabulary = handing_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        let again = declare_into(&mut handing_words().unwrap()).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// Every word carries a note, and none has a gap: nothing here is filled in.
    #[test]
    fn every_word_carries_a_note_and_names_nothing() {
        for word in EVERY_WORD {
            assert!(
                word.note().is_some_and(|note| !note.trim().is_empty()),
                "{}",
                word.named()
            );
            assert!(
                word.phrase().unwrap().source().gaps().is_empty(),
                "{}",
                word.named()
            );
        }
    }
}
