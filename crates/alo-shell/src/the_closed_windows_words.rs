//! What the person is told when one of alo's own windows could not be drawn
//! and was closed — the owner's answer of 2026-11-06: *a short notice, in the
//! existing notification style*.

use alo_strings::{Vocabulary, Word};

/// The notice, naming the window by its own title.
pub const WAS_CLOSED: Word = Word::saying(
    "shell.window-closed.could-not-be-shown",
    "{window} couldn't be shown on this screen and was closed. Nothing was lost.",
)
.noting(
    "A short notification from alo OS. {window} is the title of one of alo's own windows, as it \
     is shown on that window — for example \"What alo did\" or \"Settings\". The window could not \
     be drawn on this display, so it was closed; nothing the person had was changed or lost.",
);

/// Every sentence this file declares.
pub const THE_CLOSED_WINDOWS_WORDS: [Word; 1] = [WAS_CLOSED];

/// Why this file's words could not be declared.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TheClosedWindowsWordsError {
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
/// [`TheClosedWindowsWordsError`] for a malformed phrase or a key already
/// taken.
pub fn declare_the_closed_windows_words(
    vocabulary: &mut Vocabulary,
) -> Result<(), TheClosedWindowsWordsError> {
    let mut prepared = vocabulary.clone();
    for word in THE_CLOSED_WINDOWS_WORDS {
        prepared.says(word.phrase()?)?;
    }
    *vocabulary = prepared;
    Ok(())
}
