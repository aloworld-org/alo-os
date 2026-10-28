//! Every string this crate can say, and the English beside each one: the
//! refusals, read by whoever is looking after a machine where making the
//! folders did not work.
//!
//! **The folders' names are not here.** They were, until 2026-10-28, as three
//! words for alo OS's own translators. They are now what `xdg-user-dirs` calls
//! them in the person's language ([`crate::catalogue`], ADR 0137), so a German
//! person's `Dokumente` is the `Dokumente` every other Linux program expects.
//!
//! The shape is `alo-picking`'s and is copied rather than re-decided:
//! constants under one area, `alo_strings::Word` because these are literals in
//! this file, [`declare_into`] for the one vocabulary the machine has, and
//! tests at the bottom holding the list to the rules every other list is held
//! to.
//!
//! # A folder name is held to tighter rules than a sentence
//!
//! It becomes a directory on a disk: no slash, no leading dot, and nothing
//! that is only punctuation. `crate::naming` refuses a name that breaks those
//! rather than making a folder a person cannot see or cannot open, and the
//! refusal names the English key so the catalogue can be fixed.

use alo_strings::Vocabulary;

/// One string a crate can say — `alo-strings`' type, re-exported because this
/// crate's files and the tests that read its list name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// Refusals, read by whoever looks after the machine.
// ---------------------------------------------------------------------------

/// A translated folder name that cannot be a folder.
pub const NOT_A_NAME_A_FOLDER_CAN_HAVE: Word = Word::saying(
    "their-folders.not-a-name-a-folder-can-have",
    "One of the folder names cannot be used as a folder. The machine made the others and left \
     this one out",
)
.noting(
    "Shown to whoever looks after the machine, not to the person signing in. It means a \
     translation of one of the three folder names is empty, contains a slash, begins with a dot, \
     or is only punctuation - so making it would give the person a folder they cannot see or \
     cannot open. The names come from the translations of xdg-user-dirs, the package every Linux \
     desktop names these folders with.",
);

/// The disk would not make the folder.
pub const THE_DISK_WOULD_NOT: Word = Word::saying(
    "their-folders.the-disk-would-not",
    "This machine would not make one of the folders. The person's home folder may be full, or \
     may not be theirs to write in",
)
.noting(
    "Shown to whoever looks after the machine. The operating system refused to create a directory \
     in the person's own home folder, which usually means the disk is full or the home folder \
     belongs to somebody else.",
);

/// The record saying which folder is which could not be written.
pub const THE_RECORD_WAS_NOT_WRITTEN: Word = Word::saying(
    "their-folders.the-record-was-not-written",
    "The folders were made, and the note saying which folder is which was not. Applications will \
     fall back to looking for English names",
)
.noting(
    "Shown to whoever looks after the machine. The folders exist, but the small file that tells \
     other software which folder means documents, which means downloads and which means pictures \
     could not be written - so software will guess, and will guess in English.",
);

/// Every string this crate can say, in the order a translator meets them.
pub const EVERY_WORD: [Word; 3] = [
    NOT_A_NAME_A_FOLDER_CAN_HAVE,
    THE_DISK_WOULD_NOT,
    THE_RECORD_WAS_NOT_WRITTEN,
];

/// Why this crate's own words could not be declared.
///
/// Neither can happen to the list above — the tests at the bottom of this file
/// are what say so. It is a `Result` rather than an unwrap because a library
/// that panics on its own string table takes the shell with it.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase: a sentence that is not one, or a note that
    /// could not be attached.
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
pub fn their_folders_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// The machine has one vocabulary and every crate adds its own to it —
/// `alo-saying` is the one place that calls all of these.
///
/// # Errors
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced, because a key means one string and whoever declared it
/// first said what that string is.
pub fn declare_into(vocabulary: &mut Vocabulary) -> Result<(), WordsError> {
    for word in EVERY_WORD {
        vocabulary.says(word.phrase()?)?;
    }
    Ok(())
}
