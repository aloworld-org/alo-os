//! **Every sentence the GitHub sign-in sheet says of its own**, named for
//! translation — as drawn in the proposal the owner approved on 2026-11-13
//! (`docs/design/github-sign-in.md`, *The sheet*).
//!
//! Three of the sheet's sentences are Settings' words already, because the
//! Accounts page says them too: the sentence over the code
//! (`crate::the_settings_words::GITHUB_WILL_ASK_FOR_THIS_CODE`), *What it
//! risks* and the owner's sentence under it
//! (`crate::the_settings_words::GITHUB_RISKS`). One sentence, one key, so the
//! page and the sheet can never say the same thing two ways. The one-time code,
//! the page's address and the person's GitHub login are gh's and GitHub's, and
//! are never translated.

use alo_strings::{Vocabulary, VocabularyError, Word, WordError};

/// The title while signing in.
pub const SIGN_IN_TO_GITHUB: Word = Word::saying("shell.github-sign-in.title", "Sign in to GitHub")
    .noting(
        "The title of a small window that signs this computer in to GitHub, where people keep \
         code. GitHub is a brand name: keep it as it is.",
    );

/// The quiet button beside the code.
pub const COPY: Word = Word::saying("shell.github-sign-in.copy", "Copy").noting(
    "A button beside a one-time code, such as 4F2A-9C1D: put the code on the clipboard so it can \
     be pasted. One short word.",
);

/// The quiet button that stops signing in.
pub const CANCEL: Word = Word::saying("shell.github-sign-in.cancel", "Cancel")
    .noting("A button: stop signing in to GitHub and close the window. Nobody is signed in.");

/// The navy button that opens GitHub's page again.
pub const OPEN_THE_PAGE: Word = Word::saying("shell.github-sign-in.open-the-page", "Open {page}")
    .noting(
        "A button: open GitHub's sign-in page in the web browser, where the person types the code. \
         {page} is the page's web address without https://, such as github.com/login/device. It \
         is not translated.",
    );

/// The title once signed in.
pub const SIGNED_IN_TO_GITHUB_AS: Word = Word::saying(
    "shell.github-sign-in.signed-in-as",
    "Signed in to GitHub as {login}",
)
.noting(
    "The title of the window once signing in to GitHub worked. {login} is the person's GitHub \
     user name, written exactly as GitHub writes it.",
);

/// The line beside the check once signed in.
pub const SIGNED_IN_AND_WHERE_TO_SIGN_OUT: Word = Word::saying(
    "shell.github-sign-in.signed-in",
    "Signed in. You can sign out in Settings → Accounts.",
)
.noting(
    "Under the title once signing in to GitHub worked. 'Settings' and 'Accounts' are the names of \
     the Settings window and its Accounts page: use the same words as those.",
);

/// The navy button that closes the sheet once signed in.
pub const DONE: Word = Word::saying("shell.github-sign-in.done", "Done")
    .noting("A button: close the window after signing in to GitHub worked. One short word.");

/// The line under the title when the sheet rose by itself: what failed, and
/// that alo will not try again by itself (the owner, 2026-11-13: *"Wait for my
/// press"*).
pub const A_COMMAND_NEEDED_IT: Word = Word::saying(
    "shell.github-sign-in.a-command-needed-it",
    "A command alo ran did not work because you are not signed in to GitHub. Sign in, and then \
     ask alo again: alo won't try again by itself.",
)
.noting(
    "Under the title of a small window that appeared by itself after something alo did for the \
     person failed because this computer is not signed in to GitHub. Nothing has started yet: \
     the person may press Sign in, or Cancel. 'ask alo again' matters — alo will not repeat the \
     command on its own. GitHub and alo are names: keep them as they are.",
);

/// The navy button on the sheet that rose by itself: signing in begins.
pub const SIGN_IN: Word = Word::saying("shell.github-sign-in.sign-in", "Sign in").noting(
    "A button: begin signing this computer in to GitHub. Nothing starts until it is pressed. Two \
     short words.",
);

/// Every sentence the sheet says of its own.
pub const THE_GITHUB_SHEETS_WORDS: [Word; 9] = [
    SIGN_IN_TO_GITHUB,
    A_COMMAND_NEEDED_IT,
    SIGN_IN,
    COPY,
    CANCEL,
    OPEN_THE_PAGE,
    SIGNED_IN_TO_GITHUB_AS,
    SIGNED_IN_AND_WHERE_TO_SIGN_OUT,
    DONE,
];

/// Why the sheet's words could not be declared.
#[derive(Debug, thiserror::Error)]
pub enum TheGitHubSheetsWordsError {
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
/// [`TheGitHubSheetsWordsError`] for a malformed phrase or a key already
/// taken.
pub fn declare_the_github_sheets_words(
    vocabulary: &mut Vocabulary,
) -> Result<(), TheGitHubSheetsWordsError> {
    let mut prepared = vocabulary.clone();
    for word in THE_GITHUB_SHEETS_WORDS {
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
        for word in THE_GITHUB_SHEETS_WORDS {
            assert!(
                word.key().as_str().starts_with("shell.github-sign-in."),
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
        assert!(declare_the_github_sheets_words(&mut vocabulary).is_ok());
        assert!(
            declare_the_github_sheets_words(&mut vocabulary).is_err(),
            "a second declaration collides"
        );
    }
}
