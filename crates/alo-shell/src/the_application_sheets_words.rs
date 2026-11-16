//! **Every sentence the sheet says when alo asks for an application**, named
//! for translation — frame `501:35539` *Settings detail / window*, adapted from
//! a window to an application as Settings' own picker was
//! (`docs/design/when-alo-asks-for-an-application.md`).
//!
//! The application's own name is its packager's, as its entry gives it, and
//! is never a word of this sheet's.

use alo_strings::{Vocabulary, VocabularyError, Word, WordError};

/// What fills `{application}` in these sentences.
pub const THE_APPLICATION: &str = "application";

/// The sheet's heading (`501:35565`).
pub const LET_ALO_USE: Word = Word::saying(
    "shell.application-sheet.let-alo-use",
    "Let alo use {application}?",
)
.noting(
    "The title of a small window that appears while alo is doing a task for the person and \
     needs an app it has not been allowed to use. {application} is the app's own name, as it \
     calls itself, and is not translated.",
);

/// The line under the heading (`501:35566`).
pub const ALO_ASKED: Word = Word::saying(
    "shell.application-sheet.alo-asked",
    "alo asked to use {application}. Choose how long.",
)
.noting(
    "Under the title. alo is the assistant's name and is not translated. {application} is the \
     app's own name. The person then chooses one of three lengths of time below.",
);

/// The first length: until the task ends.
pub const THIS_TASK: Word = Word::saying("shell.application-sheet.this-task", "This task").noting(
    "One of three choices of how long alo may use the app: only while the task alo is \
         doing now lasts.",
);

/// Under *This task*.
pub const UNTIL_THIS_TASK_ENDS: Word = Word::saying(
    "shell.application-sheet.until-this-task-ends",
    "alo may use it in any task until this task's window closes or you stop it",
)
.noting(
    "Under the choice 'This task': alo may use the app in any of its tasks, not only this one, \
     and the permission ends when the window of this task closes, or when the person presses \
     Stop in it. alo is the assistant's name and is not translated.",
);

/// The second length: until sign-out.
pub const THIS_SESSION: Word = Word::saying("shell.application-sheet.this-session", "This session")
    .noting(
        "One of three choices of how long alo may use the app: until the person signs out of \
         the computer.",
    );

/// Under *This session*.
pub const UNTIL_YOU_SIGN_OUT: Word = Word::saying(
    "shell.application-sheet.until-you-sign-out",
    "Until you sign out",
)
.noting(
    "Under the choice 'This session': the permission ends when the person signs out of the \
     computer, or when it is next switched on.",
);

/// The third length: until the person takes it back.
pub const REMEMBER: Word = Word::saying("shell.application-sheet.remember", "Remember").noting(
    "One of three choices of how long alo may use the app: from now on, until the person takes \
     it back.",
);

/// Under *Remember*.
pub const UNTIL_YOU_TAKE_IT_BACK: Word = Word::saying(
    "shell.application-sheet.until-you-take-it-back",
    "Until you take it back in Settings",
)
.noting(
    "Under the choice 'Remember': the permission lasts until the person removes it in the \
     Settings app.",
);

/// The note under the choices (`501:35573`).
pub const ONLY_THIS_APPLICATION: Word = Word::saying(
    "shell.application-sheet.only-this-application",
    "Only {application} is included. Press Escape to say not now.",
)
.noting(
    "Under the three choices. {application} is the app's own name. Escape is the key on the \
     keyboard; 'not now' refers to the button of that name, which refuses.",
);

/// The quiet button (`501:35575`): refuse, and nothing is granted.
pub const NOT_NOW: Word = Word::saying("shell.application-sheet.not-now", "Not now")
    .noting("A button: do not let alo use the app. alo is told no, and nothing is allowed.");

/// The navy button (`501:35577`): grant for the length chosen.
pub const LET_ALO_USE_IT: Word = Word::saying(
    "shell.application-sheet.let-alo-use-it",
    "Let alo use {application}",
)
.noting(
    "A button: allow alo to use the app for the length of time chosen above. {application} is \
     the app's own name. alo still asks again before it does anything in the app.",
);

/// Every sentence the sheet says.
pub const THE_APPLICATION_SHEETS_WORDS: [Word; 11] = [
    LET_ALO_USE,
    ALO_ASKED,
    THIS_TASK,
    UNTIL_THIS_TASK_ENDS,
    THIS_SESSION,
    UNTIL_YOU_SIGN_OUT,
    REMEMBER,
    UNTIL_YOU_TAKE_IT_BACK,
    ONLY_THIS_APPLICATION,
    NOT_NOW,
    LET_ALO_USE_IT,
];

/// Why the sheet's words could not be declared.
#[derive(Debug, thiserror::Error)]
pub enum TheApplicationSheetsWordsError {
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
/// [`TheApplicationSheetsWordsError`] for a malformed phrase or a key already
/// taken.
pub fn declare_the_application_sheets_words(
    vocabulary: &mut Vocabulary,
) -> Result<(), TheApplicationSheetsWordsError> {
    let mut prepared = vocabulary.clone();
    for word in THE_APPLICATION_SHEETS_WORDS {
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
        for word in THE_APPLICATION_SHEETS_WORDS {
            assert!(
                word.key().as_str().starts_with("shell.application-sheet."),
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
        assert!(declare_the_application_sheets_words(&mut vocabulary).is_ok());
        assert!(
            declare_the_application_sheets_words(&mut vocabulary).is_err(),
            "a second declaration collides"
        );
    }
}
