//! What a task window says when the person closes it while alo is working
//! (ADR 0139): the question, and its three answers.
//! `crate::asking_before_closing` decides when it is asked.

use alo_strings::{Vocabulary, Word};

/// The question, on a card at the end of the conversation.
///
/// **A new key, not a rewording** (ADR 0068): the old sentence,
/// `the-closing.still-working`, offered two ways out, and a correct translation
/// of it would go on offering two while the card offers three.
pub const ALO_IS_STILL_WORKING_ON_THIS: Word = Word::saying(
    "the-closing.still-working-three-ways",
    "alo is still working on this. Stop it and close the window, keep it running in the panel \
     on the right, or cancel and keep the window open.",
)
.noting(
    "Asked in a task window when the person closes it (its Close control, or Alt+F4) while alo is \
     still working on their request. Three buttons follow: \"Stop and close\", \"Keep running\" \
     and \"Cancel\". Nothing has stopped or closed yet. \"The panel on the right\" is the strip \
     down the right-hand edge of the screen where minimised windows and running tasks wait. \
     \"alo\" is the product's name and stays as it is, in lower case.",
);

/// The answer that stops alo and closes the window once it has stopped.
pub const STOP_AND_CLOSE: Word = Word::saying("the-closing.stop-and-close", "Stop and close")
    .noting(
        "A button. It stops alo — and cancels a question already on its way to a provider — and \
         closes the window only once alo has really stopped. Keep it short, like a button.",
    );

/// The answer that leaves alo working and puts the window in the right-hand
/// panel, exactly where minimising puts it.
pub const KEEP_RUNNING: Word = Word::saying("the-closing.keep-running", "Keep running").noting(
    "A button between \"Stop and close\" and \"Cancel\". alo goes on working, and the window \
     leaves the screen for the panel on the right-hand edge, where the task can be opened again \
     or stopped. Nothing is stopped. Keep it short, like a button.",
);

/// The answer that closes nothing.
pub const CANCEL: Word = Word::saying("the-closing.cancel", "Cancel").noting(
    "A button after \"Stop and close\" and \"Keep running\". It closes nothing and changes \
     nothing: the task keeps running and the window stays open.",
);

/// Every string this file declares, in the order a translator meets them.
pub const THE_CLOSING_WORDS: [Word; 4] = [
    ALO_IS_STILL_WORKING_ON_THIS,
    STOP_AND_CLOSE,
    KEEP_RUNNING,
    CANCEL,
];

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
