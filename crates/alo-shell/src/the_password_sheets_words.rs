//! **Every sentence the password sheet says**, named for translation
//! (ADR 0150).
//!
//! The design file has no frame for this sheet yet; these are the sheet's
//! own words until it does, written to the rule every surface here keeps:
//! say what is happening, what it is for, and what the person can do.

use alo_strings::{Vocabulary, VocabularyError, Word, WordError};

/// The heading when the password is for a command alo was asked to run.
pub const ALO_ASKS_TO_RUN_THIS: Word = Word::saying(
    "shell.password-sheet.alo-asks-to-run-this",
    "alo is asking to run this as administrator:",
)
.noting(
    "The first line of the sheet that asks a person for their password. Under it is the exact \
     command alo wants to run with administrator rights, shown as it will run. Administrator \
     means the rights to change the whole machine, not only the person's own files.",
);

/// The heading when something else on the machine asked polkit.
pub const SOMETHING_ASKS: Word = Word::saying(
    "shell.password-sheet.something-asks",
    "Something on this machine is asking for administrator rights:",
)
.noting(
    "The first line of the password sheet when it is not alo asking: a program or a setting on \
     this computer needs administrator rights. Under it is that program's own description of \
     what it wants to do.",
);

/// How to answer.
pub const HOW_TO_ANSWER: Word = Word::saying(
    "shell.password-sheet.how-to-answer",
    "Type the password you sign in with, then press Enter. Escape cancels.",
)
.noting(
    "Under the question on the password sheet. The password is the same one, or the same PIN, \
     the person types to sign in to this computer. Pressing Escape closes the sheet and nothing \
     is done.",
);

/// What was typed is being checked.
pub const CHECKING: Word = Word::saying("shell.password-sheet.checking", "Checking your password…")
    .noting("Shown for the moment between pressing Enter and the answer.");

/// It was not the person's password.
pub const NOT_RIGHT: Word = Word::saying(
    "shell.password-sheet.not-right",
    "That is not the password you sign in with. Try again, or press Escape to cancel.",
)
.noting("Shown when the password typed was wrong. The sheet stays open for another try.");

/// The system's list was not given the password at sign-in.
pub const NOT_IN_STEP: Word = Word::saying(
    "shell.password-sheet.not-in-step",
    "Your password could not be given to the system when you signed in, so it cannot be \
     checked here. Sign out and sign in again, then try again. Escape cancels.",
)
.noting(
    "Shown when the password cannot be checked at all, because when the person signed in the \
     computer could not copy their password to the place the administrator check reads. It is \
     not saying the password was wrong.",
);

/// The checker did not answer.
pub const NO_CHECKER: Word = Word::saying(
    "shell.password-sheet.no-checker",
    "Your password cannot be checked right now: the part of the system that checks it did not \
     answer. Escape cancels.",
)
.noting(
    "Shown when the part of the computer that checks passwords could not be reached. It is not \
     saying the password was wrong.",
);

/// Every sentence the sheet draws.
pub const THE_PASSWORD_SHEETS_WORDS: [Word; 7] = [
    ALO_ASKS_TO_RUN_THIS,
    SOMETHING_ASKS,
    HOW_TO_ANSWER,
    CHECKING,
    NOT_RIGHT,
    NOT_IN_STEP,
    NO_CHECKER,
];

/// Invalid declaration or collision while registering the sheet's wording.
#[derive(Debug, thiserror::Error)]
pub enum ThePasswordSheetsWordsError {
    /// Invalid source phrase or translator note.
    #[error(transparent)]
    Word(#[from] WordError),
    /// An existing declaration already owns this key.
    #[error(transparent)]
    Vocabulary(#[from] VocabularyError),
}

/// Add the sheet's wording atomically, before any translation is loaded.
///
/// # Errors
/// [`ThePasswordSheetsWordsError`] for a malformed phrase or a key already
/// taken.
pub fn declare_the_password_sheets_words(
    vocabulary: &mut Vocabulary,
) -> Result<(), ThePasswordSheetsWordsError> {
    let mut prepared = vocabulary.clone();
    for word in THE_PASSWORD_SHEETS_WORDS {
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
        for word in THE_PASSWORD_SHEETS_WORDS {
            assert!(
                word.key().as_str().starts_with("shell.password-sheet."),
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
        assert!(declare_the_password_sheets_words(&mut vocabulary).is_ok());
        assert!(
            declare_the_password_sheets_words(&mut vocabulary).is_err(),
            "a second declaration collides"
        );
    }
}
