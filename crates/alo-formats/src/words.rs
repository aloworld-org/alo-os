//! Every string this crate can say, and the English beside each one.
//!
//! Three groups a person meets in different places: the row in Settings that
//! hands the decision back to their language; the two names a day has when it
//! is close — *Today*, *Yesterday* — which head the record's days; and what is
//! said about the person's own `formats.toml`.
//!
//! The shape is `alo-dock`'s: constants, `alo_strings::Word`, and a test at the
//! bottom putting every key back through `Key::named`.
//!
//! # What is deliberately not here
//!
//! **The name of any region, and any date.** What Belgium is called in Maltese
//! is CLDR's answer, and so is how 28 October is written there; both are asked
//! of `icu` in the person's language rather than typed here, where they would
//! be wrong first in the languages nobody in this repository reads.

use alo_strings::Vocabulary;

/// One string a crate can say.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// The row in Settings above the regions.
// ---------------------------------------------------------------------------

/// The row that hands how dates and times are written back to the language.
pub const AS_MY_LANGUAGE_WRITES_THEM: Word = Word::saying(
    "formats.as-my-language-writes-them",
    "As my language writes them",
)
.noting(
    "The first row of a list in Settings where a person picks how dates and times are written. \
     Every other row is a country or region, named by the system in the person's own language. \
     This one means: no region chosen, write dates and times the usual way for the language I \
     read. It is what a fresh machine is on. \"Them\" is dates and times.",
);

// ---------------------------------------------------------------------------
// A day, when it is close enough to have a name.
// ---------------------------------------------------------------------------

/// The heading over what happened today.
pub const TODAY: Word = Word::saying("formats.day.today", "Today").noting(
    "A heading over a list of things that happened on this machine today, such as the record of \
     what alo did. It stands alone above the list, so it is written as a heading in your \
     language. A day further back is headed with its date instead, written by the system.",
);

/// The heading over what happened yesterday.
pub const YESTERDAY: Word = Word::saying("formats.day.yesterday", "Yesterday").noting(
    "A heading over a list of things that happened on this machine yesterday, the day before \
     today where the person is. It stands alone above the list, so it is written as a heading in \
     your language.",
);

// ---------------------------------------------------------------------------
// A region or language that is not one — `crate::NotARegion`.
// ---------------------------------------------------------------------------

/// Something that was asked for as a region is not one.
pub const NOT_A_REGION: Word = Word::saying(
    "formats.not-a-region",
    "{asked} is not a region alo OS can write dates and times for, so nothing has been changed",
)
.noting(
    "{asked} is what was asked for, exactly as it arrived, and is never translated. A region is \
     written as two capital letters, such as BE. \"alo OS\" is the product's name and is never \
     translated.",
);

/// Something that was asked for as a language is not one.
pub const NOT_A_LANGUAGE: Word = Word::saying(
    "formats.not-a-language",
    "{asked} is not a language alo OS can write dates and times in, so nothing has been changed",
)
.noting("{asked} is what was asked for, exactly as it arrived, and is never translated.");

// ---------------------------------------------------------------------------
// The person's own file — `crate::FileNotRead` and `crate::FileNotWritten`.
// ---------------------------------------------------------------------------

/// The file could not be opened.
pub const KEPT_NOT_READ: Word = Word::saying(
    "formats.kept.not-read",
    "your date and time settings at {path} could not be read, so dates and times are written the \
     way your language writes them",
)
.noting(
    "{path} is a file on this machine and is never translated. A disk or a permission rather than \
     anything a person typed.",
);

/// The file is there and is not date and time settings.
pub const KEPT_NOT_UNDERSTOOD: Word = Word::saying(
    "formats.kept.not-understood",
    "your date and time settings at {path} are not settings alo OS can read, so nothing in the \
     file has been used and dates and times are written the way your language writes them",
)
.noting(
    "{path} is a file on this machine and is never translated. The important clause is that \
     nothing in the file was used: alo OS did not take the half it understood. \"alo OS\" is the \
     product's name and is never translated.",
);

/// The file stopped being settings at a line.
pub const KEPT_NOT_UNDERSTOOD_AT: Word = Word::saying(
    "formats.kept.not-understood-at",
    "your date and time settings at {path} stop making sense at line {line}, so nothing in the \
     file has been used and dates and times are written the way your language writes them",
)
.noting(
    "{path} is a file on this machine and is never translated. {line} is a plain whole number, \
     counted from one the way a text editor counts lines.",
);

/// The file says it is a shape this alo OS does not read.
pub const KEPT_ANOTHER_FORMAT: Word = Word::saying(
    "formats.kept.another-format",
    "your date and time settings at {path} were written for a different alo OS than this one, so \
     nothing in the file has been used and dates and times are written the way your language \
     writes them",
)
.noting(
    "{path} is a file on this machine and is never translated. Most often a newer alo OS wrote \
     the file.",
);

/// The file names something that is not a date and time setting.
pub const KEPT_UNKNOWN_KEY: Word = Word::saying(
    "formats.kept.unknown-key",
    "your date and time settings at {path} say {key}, which is not something alo OS can change \
     about how dates and times are written, so nothing in the file has been used",
)
.noting(
    "{path} is a file on this machine and {key} is a word as it was typed into it; neither is \
     translated. The key is named because it is what a person has to find in the file to fix it.",
);

/// The disk would not take the changed file.
pub const KEPT_NOT_WRITTEN: Word = Word::saying(
    "formats.kept.not-written",
    "your date and time settings at {path} could not be written, so nothing about how dates and \
     times are written has been changed",
)
.noting(
    "{path} is a file on this machine and is never translated. A full disk or a folder the person \
     cannot write. The second clause is what they act on: the file is exactly as it was.",
);

/// The change could not be written as a file this alo OS reads back.
pub const KEPT_NOT_EXPRESSIBLE: Word = Word::saying(
    "formats.kept.not-expressible",
    "this alo OS could not write that change into date and time settings at {path} it can read \
     back again, so nothing about how dates and times are written has been changed",
)
.noting(
    "{path} is a file on this machine and is never translated. A fault in alo OS rather than \
     anything the person did, said plainly: the change was refused before the file was touched.",
);

/// The file a change would replace is there and did not read, so it was kept.
pub const KEPT_NOT_REPLACED: Word = Word::saying(
    "formats.kept.not-replaced",
    "your date and time settings at {path} could not be read, so that change has not been written \
     over them and nothing about how dates and times are written has been changed — correct the \
     file, or put these settings back as alo OS ships them",
)
.noting(
    "{path} is a file on this machine and is never translated. Said when a person changes \
     something in Settings while the file, most often one they edited by hand, does not read: alo \
     OS keeps their file rather than replacing it, so a typing mistake in it is not lost. The last \
     clause gives the two ways on — mend the file in an editor, or deliberately put this section \
     back as it ships, which does replace the file.",
);

/// Every string this crate can say.
pub const EVERY_WORD: [Word; 13] = [
    AS_MY_LANGUAGE_WRITES_THEM,
    TODAY,
    YESTERDAY,
    NOT_A_REGION,
    NOT_A_LANGUAGE,
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
/// None of these can happen to the list above — the tests at the bottom of this
/// file are what say so — and [`declare_into`] can genuinely fail against a
/// vocabulary that already holds one of these keys.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase.
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
pub fn formats_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// # Errors
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced, because a key means one string.
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
    use super::*;
    use std::collections::BTreeSet;

    /// **What we ship is held to the rule everybody else is held to.**
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(
                alo_strings::Key::named(word.named()),
                Ok(word.key()),
                "{}",
                word.named()
            );
        }
    }

    /// A key names one string.
    #[test]
    fn no_two_words_are_named_the_same() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// Every one of them is in this crate's area.
    #[test]
    fn everything_this_crate_says_says_it_is_this_crate() {
        for word in EVERY_WORD {
            assert_eq!(word.key().area(), "formats", "{}", word.named());
        }
    }

    /// The list declares, and a second declaration replaces nothing.
    #[test]
    fn the_whole_list_declares_once() {
        let mut vocabulary = formats_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert!(matches!(
            declare_into(&mut vocabulary),
            Err(WordsError::List(_))
        ));
    }

    /// **A heading and a row are names, with nothing to fill in.**
    #[test]
    fn a_heading_and_a_row_name_nothing() {
        for word in [AS_MY_LANGUAGE_WRITES_THEM, TODAY, YESTERDAY] {
            assert!(
                word.phrase().unwrap().source().gaps().is_empty(),
                "{word:?}"
            );
        }
    }

    /// Every sentence about the file names it, and the key or line where that
    /// is what was wrong.
    #[test]
    fn the_strings_about_the_file_have_a_gap_for_it() {
        for (word, gaps) in [
            (KEPT_NOT_READ, &["path"][..]),
            (KEPT_NOT_UNDERSTOOD, &["path"][..]),
            (KEPT_NOT_UNDERSTOOD_AT, &["path", "line"][..]),
            (KEPT_ANOTHER_FORMAT, &["path"][..]),
            (KEPT_UNKNOWN_KEY, &["path", "key"][..]),
            (KEPT_NOT_WRITTEN, &["path"][..]),
            (KEPT_NOT_EXPRESSIBLE, &["path"][..]),
            (NOT_A_REGION, &["asked"][..]),
            (NOT_A_LANGUAGE, &["asked"][..]),
            (KEPT_NOT_REPLACED, &["path"][..]),
        ] {
            let phrase = word.phrase().unwrap();
            for gap in gaps {
                assert!(phrase.source().has(gap), "{} wants {gap}", word.named());
            }
        }
    }
}
