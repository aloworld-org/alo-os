//! **Every sentence the *Open with* sheet says**, named for translation —
//! frame `25:52`, approved by the owner on 2026-11-12
//! (`docs/design/default-apps-and-open-with.md`).
//!
//! The applications' own names are theirs, as their entries give them, and
//! the address is the link exactly as the asking application gave it; neither
//! is a word of this sheet's.

use alo_strings::{Vocabulary, VocabularyError, Word, WordError};

/// The sheet's title (`25:58`).
pub const OPEN_WITH: Word = Word::saying("shell.open-with.title", "Open with").noting(
    "The title of a small window asking which app should open a link someone clicked: a web \
     address or an email address.",
);

/// The line under the title (`25:59`).
pub const ASKED_TO_OPEN: Word = Word::saying(
    "shell.open-with.asked-to-open",
    "{application} asked to open {address}",
)
.noting(
    "Under the title of the Open with window. {application} is the name of the app the link was \
     clicked in, as it calls itself; {address} is the link exactly as that app gave it. Neither \
     is translated.",
);

/// Under the person's default, for a web link (`25:67`).
pub const YOUR_DEFAULT_FOR_WEB_LINKS: Word = Word::saying(
    "shell.open-with.your-default-for-web-links",
    "Your default for web links",
)
.noting(
    "Under the name of the browser the person chose, in Settings, to open web links — the one \
     already chosen when the window opens.",
);

/// Under the person's default, for an email address.
pub const YOUR_DEFAULT_FOR_EMAIL_ADDRESSES: Word = Word::saying(
    "shell.open-with.your-default-for-email-addresses",
    "Your default for email addresses",
)
.noting(
    "Under the name of the mail app the person chose, in Settings, to open email addresses — the \
     one already chosen when the window opens.",
);

/// Under any other application offered (`25:73`).
pub const INSTALLED: Word = Word::saying("shell.open-with.installed", "Installed")
    .noting("Under the name of another app on this computer that can open the link.");

/// The quiet button (`25:75`).
pub const CANCEL: Word = Word::saying("shell.open-with.cancel", "Cancel")
    .noting("A button: close the window and open nothing.");

/// The navy button (`25:77`).
pub const OPEN: Word = Word::saying("shell.open-with.open", "Open")
    .noting("A button: open the link in the app chosen in the list.");

/// Every sentence the sheet says.
pub const THE_OPEN_WITH_SHEETS_WORDS: [Word; 7] = [
    OPEN_WITH,
    ASKED_TO_OPEN,
    YOUR_DEFAULT_FOR_WEB_LINKS,
    YOUR_DEFAULT_FOR_EMAIL_ADDRESSES,
    INSTALLED,
    CANCEL,
    OPEN,
];

/// Why the sheet's words could not be declared.
#[derive(Debug, thiserror::Error)]
pub enum TheOpenWithSheetsWordsError {
    /// A word that is not a phrase.
    #[error(transparent)]
    Word(#[from] WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] VocabularyError),
}

/// Add the sheet's words, all or none.
///
/// # Errors
/// [`TheOpenWithSheetsWordsError`] for a malformed phrase or a key already
/// taken.
pub fn declare_the_open_with_sheets_words(
    vocabulary: &mut Vocabulary,
) -> Result<(), TheOpenWithSheetsWordsError> {
    let mut prepared = vocabulary.clone();
    for word in THE_OPEN_WITH_SHEETS_WORDS {
        prepared.says(word.phrase()?)?;
    }
    *vocabulary = prepared;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every sentence is the sheet's, carries a note, and declares once.**
    #[test]
    fn every_sentence_is_the_sheets_and_declares_once() {
        for word in THE_OPEN_WITH_SHEETS_WORDS {
            assert!(
                word.key().as_str().starts_with("shell.open-with."),
                "{}",
                word.key().as_str()
            );
            assert!(
                word.phrase().is_ok_and(|phrase| phrase.note().is_some()),
                "{}",
                word.key().as_str()
            );
        }
        let mut vocabulary = Vocabulary::empty();
        assert!(declare_the_open_with_sheets_words(&mut vocabulary).is_ok());
        assert!(
            declare_the_open_with_sheets_words(&mut vocabulary).is_err(),
            "a second declaration collides"
        );
    }
}
