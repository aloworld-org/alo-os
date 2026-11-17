//! **Every sentence the file chooser says**, named for translation
//! (`docs/design/the-file-chooser.md`).
//!
//! A folder's and a file's names are the person's own, and the asking
//! application's name is the one its entry gives; none of them is a word of
//! this sheet's.

use alo_strings::{Vocabulary, VocabularyError, Word, WordError};

/// The title, when an application asked to open a file.
pub const OPEN_A_FILE: Word = Word::saying("shell.file-chooser.open-a-file", "Open a file")
    .noting(
        "The title of a small window in which the person chooses a file for an app to open.",
    );

/// The title, when an application asked to save a file.
pub const SAVE_A_FILE: Word = Word::saying("shell.file-chooser.save-a-file", "Save a file")
    .noting(
        "The title of a small window in which the person chooses a folder and a name for an app \
         to save a file under.",
    );

/// The line under the title, when opening.
pub const ASKED_TO_OPEN: Word = Word::saying(
    "shell.file-chooser.asked-to-open",
    "{application} asked to open a file",
)
.noting(
    "Under the title of the window for opening a file. {application} is the name of the app that \
     asked, as it calls itself, and is not translated.",
);

/// The line under the title, when saving.
pub const ASKED_TO_SAVE: Word = Word::saying(
    "shell.file-chooser.asked-to-save",
    "{application} asked to save a file",
)
.noting(
    "Under the title of the window for saving a file. {application} is the name of the app that \
     asked, as it calls itself, and is not translated.",
);

/// Where the person is standing.
pub const IN_THE_FOLDER: Word = Word::saying("shell.file-chooser.in-the-folder", "In {folder}")
    .noting(
        "Above the list of folders and files: which folder the person is looking in. {folder} is \
         the folder's whole path on the computer, such as /home/anna/Documents, and is not \
         translated.",
    );

/// Under the row that goes up.
pub const THE_FOLDER_ABOVE: Word =
    Word::saying("shell.file-chooser.the-folder-above", "The folder above").noting(
        "Under the name of the first row in the list. Choosing it goes up to the folder that \
         holds the one being looked in.",
    );

/// Under a folder's row.
pub const FOLDER: Word = Word::saying("shell.file-chooser.folder", "Folder")
    .noting("Under a name in the list: this is a folder, and choosing it opens it.");

/// Under a file's row.
pub const FILE: Word = Word::saying("shell.file-chooser.file", "File")
    .noting("Under a name in the list: this is a file.");

/// The label over the name, when saving.
pub const NAME: Word = Word::saying("shell.file-chooser.name", "Name")
    .noting("The label of the box where the person types the name to save the file under.");

/// A folder that would not open.
pub const COULD_NOT_OPEN: Word = Word::saying(
    "shell.file-chooser.could-not-open",
    "{folder} could not be opened.",
)
.noting(
    "Said when the person chose a folder that this computer would not open for them, or that is \
     gone. {folder} is the folder's name, not translated. They stay where they were.",
);

/// No name.
pub const NO_NAME: Word = Word::saying(
    "shell.file-chooser.no-name",
    "Type a name to save the file under.",
)
.noting("Said when the person pressed Save with the name box empty. Nothing is saved.");

/// Not one name.
pub const NOT_A_NAME: Word = Word::saying(
    "shell.file-chooser.not-a-name",
    "A name cannot hold a slash, and cannot be . or ..",
)
.noting(
    "Said when the name typed is not a name a file can have: it holds the character /, or is only \
     a dot or two dots. Nothing is saved. Keep the characters / . and .. as they are.",
);

/// A folder's name.
pub const A_FOLDER_HAS_THIS_NAME: Word = Word::saying(
    "shell.file-chooser.a-folder-has-this-name",
    "A folder here already has this name.",
)
.noting(
    "Said when the name typed is the name of a folder in the folder being looked in. Nothing is \
     saved.",
);

/// Replacing.
pub const REPLACES_WHAT_IS_THERE: Word = Word::saying(
    "shell.file-chooser.replaces-what-is-there",
    "A file with this name is here already. Save again to replace it.",
)
.noting(
    "Said when the name typed is the name of a file already in this folder. Pressing Save a second \
     time, without changing the name, replaces that file.",
);

/// The quiet button.
pub const CANCEL: Word = Word::saying("shell.file-chooser.cancel", "Cancel")
    .noting("A button: close the window, and the app is given no file.");

/// The navy button, when opening.
pub const OPEN: Word = Word::saying("shell.file-chooser.open", "Open")
    .noting("A button: open the folder or the file chosen in the list.");

/// The navy button, when saving.
pub const SAVE: Word = Word::saying("shell.file-chooser.save", "Save")
    .noting("A button: save the file under the name typed, in the folder being looked in.");

/// Every sentence the sheet says.
pub const THE_FILE_CHOOSERS_WORDS: [Word; 17] = [
    OPEN_A_FILE,
    SAVE_A_FILE,
    ASKED_TO_OPEN,
    ASKED_TO_SAVE,
    IN_THE_FOLDER,
    THE_FOLDER_ABOVE,
    FOLDER,
    FILE,
    NAME,
    COULD_NOT_OPEN,
    NO_NAME,
    NOT_A_NAME,
    A_FOLDER_HAS_THIS_NAME,
    REPLACES_WHAT_IS_THERE,
    CANCEL,
    OPEN,
    SAVE,
];

/// Why the sheet's words could not be declared.
#[derive(Debug, thiserror::Error)]
pub enum TheFileChoosersWordsError {
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
/// [`TheFileChoosersWordsError`] for a malformed phrase or a key already
/// taken.
pub fn declare_the_file_choosers_words(
    vocabulary: &mut Vocabulary,
) -> Result<(), TheFileChoosersWordsError> {
    let mut prepared = vocabulary.clone();
    for word in THE_FILE_CHOOSERS_WORDS {
        prepared.says(word.phrase()?)?;
    }
    *vocabulary = prepared;
    Ok(())
}

/// The sentence for what just happened.
pub(crate) const fn the_word_for(note: &alo_file_chooser::Note) -> Word {
    use alo_file_chooser::Note;
    match note {
        Note::CouldNotOpen(_) => COULD_NOT_OPEN,
        Note::NoName => NO_NAME,
        Note::NotAName => NOT_A_NAME,
        Note::AFolderHasThisName => A_FOLDER_HAS_THIS_NAME,
        Note::ReplacesWhatIsThere => REPLACES_WHAT_IS_THERE,
    }
}

/// The sentence for what just happened, in the person's language.
pub(crate) fn said(strings: &alo_strings::Strings, note: &alo_file_chooser::Note) -> String {
    let filling = match note {
        alo_file_chooser::Note::CouldNotOpen(folder) => {
            alo_strings::Filling::of("folder", folder.as_str())
        }
        _ => alo_strings::Filling::nothing(),
    };
    strings.say(&the_word_for(note).key(), &filling).into_text()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every sentence is the sheet's, carries a note, and declares once.**
    #[test]
    fn every_sentence_is_the_sheets_and_declares_once() {
        for word in THE_FILE_CHOOSERS_WORDS {
            assert!(
                word.key().as_str().starts_with("shell.file-chooser."),
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
        assert!(declare_the_file_choosers_words(&mut vocabulary).is_ok());
        assert!(
            declare_the_file_choosers_words(&mut vocabulary).is_err(),
            "a second declaration collides"
        );
    }
}
