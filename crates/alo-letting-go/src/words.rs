//! Every string this crate can say, and the English beside each one.
//!
//! Eight, and all eight are about one file: `undo.toml`, the person's own
//! window, when it is there and does not read or a change to it could not be
//! written. The shape is `alo-sleeping`'s and is copied rather than re-decided,
//! because a person meeting the same refusal about two of their own files
//! should meet the same sentence.
//!
//! # What is deliberately not on this list
//!
//! **Anything the unit says.** The privileged unit that removes an expired
//! snapshot says what it did to the journal, in English, for whoever
//! administers the machine — the shape both of `alo-brokerd`'s unit programs
//! already have. It has no person in front of it and no language to say
//! anything in.
//!
//! **What a person is told about an undo they have lost.** That is the
//! record's, said by `alo-recounting` from the entry this crate writes, in the
//! words the person approved at the time. A second wording of one moment is the
//! thing `alo-recounting`'s own list refuses by name.
//!
//! **A number.** Nothing here counts anything. ADR 0045's second term asks that
//! a person be told *which turns* lost their undo rather than how many, so
//! there is no plural in this file and no gap for one.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported because this
/// crate's files name it as `crate::words::Word`.
pub use alo_strings::Word;

/// The disk or a permission stopped the file being read at all.
pub const KEPT_NOT_READ: Word = Word::saying(
    "letting-go.kept.not-read",
    "how far back you asked this machine to keep what the agent changed, at {path}, could not be \
     read, so it keeps what alo OS ships",
)
.noting(
    "{path} is a file on this machine and is never translated. The agent is the assistant built \
     into alo OS and not a person. A disk or a permission rather than anything the person typed. \
     \"alo OS\" is the product's name and is never translated.",
);

/// The file is there and is not this setting.
pub const KEPT_NOT_UNDERSTOOD: Word = Word::saying(
    "letting-go.kept.not-understood",
    "how far back you asked this machine to keep what the agent changed, at {path}, is not \
     something alo OS can read, so nothing in the file has been used and it keeps what alo OS \
     ships",
)
.noting(
    "{path} is a file on this machine and is never translated. The important clause is that \
     nothing in the file was used: alo OS did not take the half it understood.",
);

/// The file stopped being this setting at a line.
pub const KEPT_NOT_UNDERSTOOD_AT: Word = Word::saying(
    "letting-go.kept.not-understood-at",
    "how far back you asked this machine to keep what the agent changed, at {path}, stops making \
     sense at line {line}, so nothing in the file has been used and it keeps what alo OS ships",
)
.noting(
    "{path} is a file on this machine and is never translated. {line} is a plain whole number, \
     counted from one the way a text editor counts lines.",
);

/// The file says it is a shape this alo OS does not read.
pub const KEPT_ANOTHER_FORMAT: Word = Word::saying(
    "letting-go.kept.another-format",
    "how far back you asked this machine to keep what the agent changed, at {path}, was written \
     for a different alo OS than this one, so nothing in the file has been used and it keeps what \
     alo OS ships",
)
.noting(
    "{path} is a file on this machine and is never translated. Most often a newer alo OS wrote \
     the file.",
);

/// The file names something that is not this setting.
pub const KEPT_UNKNOWN_KEY: Word = Word::saying(
    "letting-go.kept.unknown-key",
    "how far back you asked this machine to keep what the agent changed, at {path}, says {key}, \
     which is not something alo OS can change about it, so nothing in the file has been used",
)
.noting(
    "{path} is a file on this machine and {key} is a word as it was typed into it; neither is \
     translated. The key is named because it is what a person has to find in the file to fix it.",
);

/// The disk would not take the changed file.
pub const KEPT_NOT_WRITTEN: Word = Word::saying(
    "letting-go.kept.not-written",
    "how far back this machine keeps what the agent changed could not be written to {path}, so \
     nothing about it has changed",
)
.noting(
    "{path} is a file on this machine and is never translated. A full disk or a folder the person \
     cannot write. The second clause is what they act on: the file is exactly as it was.",
);

/// The change could not be written as a file this alo OS reads back.
pub const KEPT_NOT_EXPRESSIBLE: Word = Word::saying(
    "letting-go.kept.not-expressible",
    "this alo OS could not write that change to {path} in a way it can read back again, so \
     nothing about how far back it keeps what the agent changed has changed",
)
.noting(
    "{path} is a file on this machine and is never translated. A fault in alo OS rather than \
     anything the person did, said plainly: the change was refused before the file was touched.",
);

/// The file a change would replace is there and did not read, so it was kept.
pub const KEPT_NOT_REPLACED: Word = Word::saying(
    "letting-go.kept.not-replaced",
    "how far back you asked this machine to keep what the agent changed, at {path}, could not be \
     read, so that change has not been written over it and nothing has changed — correct the \
     file, or put this back as alo OS ships it",
)
.noting(
    "{path} is a file on this machine and is never translated. Said when a person changes this in \
     Settings while the file, most often one they edited by hand, does not read: alo OS keeps \
     their file rather than replacing it, so a typing mistake in it is not lost. The last clause \
     gives the two ways on.",
);

/// Every string this crate can say, in the order a translator meets them.
pub const EVERY_WORD: [Word; 8] = [
    KEPT_NOT_READ,
    KEPT_NOT_UNDERSTOOD,
    KEPT_NOT_UNDERSTOOD_AT,
    KEPT_ANOTHER_FORMAT,
    KEPT_UNKNOWN_KEY,
    KEPT_NOT_WRITTEN,
    KEPT_NOT_EXPRESSIBLE,
    KEPT_NOT_REPLACED,
];

/// Why this crate's own words could not be declared.
///
/// Neither can happen to the list above — the tests at the bottom of this file
/// are what say so.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase.
    #[error(transparent)]
    Word(#[from] WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn letting_go_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// # Errors
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced.
pub fn declare_into(vocabulary: &mut Vocabulary) -> Result<(), WordsError> {
    for word in EVERY_WORD {
        vocabulary.says(word.phrase()?)?;
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    /// What we ship is held to the rule everybody else is held to.
    #[test]
    fn every_key_is_a_key_under_this_crates_area() {
        for word in EVERY_WORD {
            assert_eq!(
                alo_strings::Key::named(word.named()),
                Ok(word.key()),
                "{}",
                word.named()
            );
            assert_eq!(word.key().area(), "letting-go", "{}", word.named());
        }
    }

    /// The list declares once, each key once, and a second declaration
    /// replaces nothing.
    #[test]
    fn the_whole_list_declares_once() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
        let mut vocabulary = letting_go_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert!(matches!(
            declare_into(&mut vocabulary),
            Err(WordsError::List(_))
        ));
    }

    /// Every word carries a note: none of these can be translated from its own
    /// words alone.
    #[test]
    fn every_word_carries_a_note_for_the_translator() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// **Only a file's place, a line and a key are ever filled in.** No
    /// sentence here has room for a number of snapshots, a folder an agent
    /// touched, or anything a turn did.
    #[test]
    fn nothing_but_a_path_a_line_or_a_key_is_filled_in() {
        let allowed: BTreeSet<&str> = ["path", "line", "key"].into();
        for word in EVERY_WORD {
            for gap in word.phrase().unwrap().source().gaps() {
                assert!(allowed.contains(gap.as_str()), "{}: {gap}", word.named());
            }
        }
    }
}
