//! Reader-only vocabulary; dismissing a name never closes its window.
use alo_strings::{Vocabulary, Word};

/// Position within one name. Gaps are bounded one-based decimal page numbers.
pub const READER_POSITION: Word = Word::saying("shell.name-page", "Page {page} of {total}")
    .noting("Position within a long control name. Move both number gaps as your language needs.");
/// Read the preceding part of the same name.
pub const READER_PREVIOUS: Word = Word::saying("shell.name-previous", "Previous page")
    .noting("Read the preceding part of a long control name; does not switch windows.");
/// Read the following part of the same name.
pub const READER_NEXT: Word = Word::saying("shell.name-next", "Next page")
    .noting("Read the following part of a long control name; does not switch windows.");
/// Dismiss only the reader, without executing the control being described.
pub const READER_DISMISS: Word = Word::saying("shell.name-dismiss", "Done reading")
    .noting("Dismiss the control-name reader, leaving the application open and unchanged.");

/// All strings required for a complete reader navigation model.
pub const READER_WORDS: [Word; 4] = [
    READER_POSITION,
    READER_PREVIOUS,
    READER_NEXT,
    READER_DISMISS,
];

/// Invalid declaration or collision while registering reader vocabulary.
#[derive(Debug, thiserror::Error)]
pub enum ReaderWordsError {
    /// Invalid source phrase or translator note.
    #[error(transparent)]
    Word(#[from] alo_strings::WordError),
    /// An existing declaration already owns this key.
    #[error(transparent)]
    Vocabulary(#[from] alo_strings::VocabularyError),
}

/// Add reader wording atomically to the host's vocabulary before translation load.
/// A collision leaves every existing declaration untouched and adds nothing.
pub fn declare_reader_words(vocabulary: &mut Vocabulary) -> Result<(), ReaderWordsError> {
    let mut prepared = vocabulary.clone();
    for word in READER_WORDS {
        prepared.says(word.phrase()?)?;
    }
    *vocabulary = prepared;
    Ok(())
}
