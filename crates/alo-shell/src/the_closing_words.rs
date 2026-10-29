//! What a task window says when the person closes it while alo is working
//! (ADR 0139): the question, and its two answers. `crate::asking_before_closing`
//! decides when it is asked.

use alo_strings::{Vocabulary, Word};

/// The question, on a card at the end of the conversation.
pub const ALO_IS_STILL_WORKING_ON_THIS: Word = Word::saying(
    "the-closing.still-working",
    "alo is still working on this. Stop it and close the window, or cancel and keep it open.",
)
.noting(
    "Asked in a task window when the person closes it (its Close control, or Alt+F4) while alo is \
     still working on their request. Two buttons follow: \"Stop and close\" and \"Cancel\". \
     Nothing has stopped or closed yet. \"alo\" is the product's name and stays as it is, in \
     lower case.",
);

/// The answer that stops alo and closes the window once it has stopped.
pub const STOP_AND_CLOSE: Word = Word::saying("the-closing.stop-and-close", "Stop and close")
    .noting(
        "A button. It stops alo — and cancels a question already on its way to a provider — and \
         closes the window only once alo has really stopped. Keep it short, like a button.",
    );

/// The answer that closes nothing.
pub const CANCEL: Word = Word::saying("the-closing.cancel", "Cancel").noting(
    "A button beside \"Stop and close\". It closes nothing and changes nothing: the task keeps \
     running and the window stays open.",
);

/// Every string this file declares, in the order a translator meets them.
pub const THE_CLOSING_WORDS: [Word; 3] = [ALO_IS_STILL_WORKING_ON_THIS, STOP_AND_CLOSE, CANCEL];

/// Why this file's words could not be declared.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TheClosingWordsError {
    /// A word that is not a phrase.
    #[error(transparent)]
    Word(#[from] alo_strings::WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] alo_strings::VocabularyError),
}

/// Add these words atomically, before any translation is loaded.
///
/// # Errors
/// [`TheClosingWordsError`] for a malformed phrase or a key already taken.
pub fn declare_the_closing_words(vocabulary: &mut Vocabulary) -> Result<(), TheClosingWordsError> {
    let mut prepared = vocabulary.clone();
    for word in THE_CLOSING_WORDS {
        prepared.says(word.phrase()?)?;
    }
    *vocabulary = prepared;
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// Every word declares, and twice is refused and changes nothing.
    #[test]
    fn every_word_declares_once() {
        let mut vocabulary = Vocabulary::empty();
        declare_the_closing_words(&mut vocabulary).unwrap();
        for word in THE_CLOSING_WORDS {
            assert!(vocabulary.phrase(&word.key()).is_some(), "{word:?}");
        }
        let before = vocabulary.how_many();
        assert!(declare_the_closing_words(&mut vocabulary).is_err());
        assert_eq!(vocabulary.how_many(), before);
    }
}
