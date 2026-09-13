//! Every string this crate can say, and the English beside each one.
//!
//! An index is names, kinds, dates and words, and a name needs no
//! translation. What needs one is every place a window shows a sentence
//! instead: the kind of a file, why a file has no words in the index, why a
//! folder could not be indexed at all. Each is declared here so that nothing
//! reaches a person without something having asked whether anybody
//! translated it.
//!
//! # A note is part of the string
//!
//! A translator works alone, with no product in front of them. Where a
//! sentence cannot be translated from its own words — *an index* is a thing
//! this machine keeps, *a link* is a technical term — the note says so.

use alo_strings::{Key, Plural, Vocabulary};

/// One string a crate can say.
///
/// Re-exported because this crate's own files, and the tests that read this
/// list, name it as `crate::words::Word`.
pub use alo_strings::Word;

/// One string this crate can say about a number of things.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Counted {
    /// What names it.
    named: &'static str,
    /// The gap the number goes in.
    number: &'static str,
    /// What it says about one thing.
    one: &'static str,
    /// What it says about any other number of things.
    other: &'static str,
    /// What a translator needs to know.
    note: &'static str,
}

impl Counted {
    /// What names it.
    #[must_use]
    pub fn key(&self) -> Key {
        Key::unchecked(self.named)
    }

    /// What names it, as written.
    #[must_use]
    pub const fn named(&self) -> &'static str {
        self.named
    }

    /// What a translator needs to know.
    #[must_use]
    pub const fn note(&self) -> &'static str {
        self.note
    }
}

// ---------------------------------------------------------------------------
// Why there is no index — `crate::NotIndexed`.
// ---------------------------------------------------------------------------

/// The folder was named from somewhere rather than from the root.
pub const NOT_ABSOLUTE: Word = Word::saying(
    "finding.not-absolute",
    "{at} does not say where it is from the root of the machine, so it cannot be indexed.",
)
.noting(
    "Said when a folder to index was named by a relative path, such as \"Documents\" instead \
     of \"/home/ada/Documents\". {at} is the path as it was given and is never translated. A \
     mistake in the code that asked rather than anything a person did.",
);

/// The folder could not be walked at all.
pub const NOT_WALKED: Word = Word::saying("finding.not-walked", "{at} could not be indexed. {why}")
    .noting(
        "Said when the folder a person asked to index could not be opened at all: it is not \
         there, it is a file rather than a folder, or the machine refused. {at} is its path and \
         is never translated. {why} is a whole sentence saying which, already in the reader's \
         language.",
    );

/// An index of one folder offered for another.
pub const NOT_THE_SAME: Word = Word::saying(
    "finding.not-the-same-folder",
    "The index at hand is of {indexed}, not of {asked}.",
)
.noting(
    "Said when an index of one folder was given as the earlier index of a different folder. \
     Both {indexed} and {asked} are paths and are never translated. A mistake in the code that \
     asked rather than anything a person did.",
);

/// The index could not be written.
pub const NOT_KEPT: Word = Word::saying(
    "finding.not-kept",
    "The index could not be written to {at}: {why}",
)
.noting(
    "Said when the file an index is kept in could not be written: the disk is full, the folder \
     is read-only. {at} is the file's path and is never translated. {why} is what the operating \
     system said, in its own words.",
);

/// The index file could not be read.
pub const NOT_OPENED: Word = Word::saying(
    "finding.not-opened",
    "The index at {at} could not be read: {why}",
)
.noting(
    "Said when the file an index is kept in is missing or could not be opened. {at} is the \
     file's path and is never translated. {why} is what the operating system said, in its own \
     words.",
);

/// The file is not an index.
pub const NOT_AN_INDEX: Word = Word::saying(
    "finding.not-an-index",
    "{at} is not an index this machine can read: {why}",
)
.noting(
    "Said when the file where an index should be holds something else: a write that was cut \
     short, another program's file, or an index from a later version of alo OS. {at} is the \
     file's path and is never translated. {why} says what was wrong, as a sentence. The index is \
     simply made again; nothing is lost.",
);

/// There is no home directory to keep an index in.
pub const NOWHERE_TO_KEEP_IT: Word = Word::saying(
    "finding.nowhere-to-keep-it",
    "There is no home directory to keep an index in.",
)
.noting(
    "Said when the session has no home directory at all, which on a working machine does not \
     happen: an index is kept in the person's own directory, and there is none.",
);

// ---------------------------------------------------------------------------
// Why a file has no words in the index — `crate::Contents`.
// ---------------------------------------------------------------------------

/// A kind of file with no reader.
pub const NOT_TEXT: Word = Word::saying(
    "finding.contents.not-text",
    "Not a kind of file whose words can be read.",
)
.noting(
    "Shown beside a file that is an image, a PDF, an archive or bytes of no known kind, whose \
     words the index does not hold. A search by name, kind or date still finds it; a search by \
     words does not, and says so. A phrase rather than a sentence: it stands beside a file.",
);

/// A file that could not be read.
pub const NOT_READ: Word = Word::saying(
    "finding.contents.not-read",
    "Its words could not be read: {why}",
)
.noting(
    "Shown beside a file the machine would not let the index open, usually because it belongs \
     to another person. {why} is what the operating system said, in its own words. A phrase \
     rather than a sentence: it stands beside a file.",
);

/// A file larger than an index reads.
pub const TOO_BIG: Word = Word::saying(
    "finding.contents.too-big",
    "Larger than the {most} bytes an index reads, so its words were not read.",
)
.noting(
    "Shown beside a file too large for its words to be indexed. {most} is the limit in bytes, \
     a number. A phrase rather than a sentence: it stands beside a file.",
);

// ---------------------------------------------------------------------------
// What a file is — `crate::Kind`.
// ---------------------------------------------------------------------------

/// Text.
pub const KIND_TEXT: Word = Word::saying("finding.kind.text", "Text").noting(
    "The kind of a file whose bytes are readable text of any language: notes, code, a letter. \
     A label in a column of kinds.",
);

/// A PDF document.
pub const KIND_PDF: Word = Word::saying("finding.kind.pdf", "A PDF document").noting(
    "The kind of a file in the PDF format. \"PDF\" is a proper name and is never translated. A \
     label in a column of kinds.",
);

/// A PNG image.
pub const KIND_PNG: Word = Word::saying("finding.kind.png", "A PNG image")
    .noting("An image in the PNG format. \"PNG\" is a proper name and is never translated.");

/// A JPEG image.
pub const KIND_JPEG: Word = Word::saying("finding.kind.jpeg", "A JPEG image")
    .noting("An image in the JPEG format. \"JPEG\" is a proper name and is never translated.");

/// A GIF image.
pub const KIND_GIF: Word = Word::saying("finding.kind.gif", "A GIF image")
    .noting("An image in the GIF format. \"GIF\" is a proper name and is never translated.");

/// A zip archive.
pub const KIND_ZIP: Word = Word::saying("finding.kind.zip", "A zip archive").noting(
    "A file in the zip format, which is also what most office documents are on the disk. \
     \"zip\" is a proper name and is never translated.",
);

/// A program.
pub const KIND_PROGRAM: Word = Word::saying("finding.kind.program", "A program")
    .noting("A file the machine could run. A label in a column of kinds.");

/// Bytes of no known kind.
pub const KIND_BYTES: Word = Word::saying(
    "finding.kind.bytes",
    "A file of a kind this machine does not know",
)
.noting(
    "The kind of a file whose bytes are neither text nor any format the index recognises. Not \
     a fault: the file is whole, and the machine simply cannot say what it is.",
);

/// An empty file.
pub const KIND_EMPTY: Word = Word::saying("finding.kind.empty", "An empty file")
    .noting("A file with nothing in it at all. A label in a column of kinds.");

/// A file that could not be read.
pub const KIND_UNREAD: Word = Word::saying("finding.kind.unread", "A file that could not be read")
    .noting(
        "The kind shown for a file the machine would not let the index open, so nothing about its \
     bytes is known. Read with \"finding.contents.not-read\", which says why.",
    );

/// A folder.
pub const KIND_FOLDER: Word = Word::saying("finding.kind.folder", "A folder")
    .noting("A folder. A label in a column of kinds.");

/// A link.
pub const KIND_LINK: Word = Word::saying("finding.kind.link", "A link").noting(
    "A name that points at something else on the disk, which the index never follows. \
     \"Link\" is the technical term and may be translated as such.",
);

/// Something that is neither a file nor a folder.
pub const KIND_OTHER: Word = Word::saying(
    "finding.kind.other",
    "Something that is not a file or a folder",
)
.noting(
    "A device, a socket or a pipe, which a folder can hold and an index has nothing to say about.",
);

// ---------------------------------------------------------------------------
// What the walk did not reach — `crate::Covered`.
// ---------------------------------------------------------------------------

/// An index that stopped at its bound.
pub const NOT_WHOLE: Counted = Counted {
    named: "finding.not-whole",
    number: "most",
    one: "The index stopped after one thing, so it is not the whole of the folder.",
    other: "The index stopped after {most} things, so it is not the whole of the folder.",
    note: "Said once, above an index, when the folder holds more things than one walk looks at. \
           {most} is that limit. What was indexed is true, and a search says which folders it \
           never entered.",
};

/// Things whose names cannot be shown, left out of the index.
pub const UNNAMED: Counted = Counted {
    named: "finding.unnamed",
    number: "unnamed",
    one: "One thing whose name cannot be shown is not indexed.",
    other: "{unnamed} things whose names cannot be shown are not indexed.",
    note: "Said once, above an index, when the folder holds files or folders whose names contain \
           characters a screen cannot show safely. They are left out rather than shown under a \
           name that could mislead. {unnamed} is how many.",
};

/// Everything this crate can say in one sentence each.
pub const EVERY_WORD: [Word; 23] = [
    NOT_ABSOLUTE,
    NOT_WALKED,
    NOT_THE_SAME,
    NOT_KEPT,
    NOT_OPENED,
    NOT_AN_INDEX,
    NOWHERE_TO_KEEP_IT,
    NOT_TEXT,
    NOT_READ,
    TOO_BIG,
    KIND_TEXT,
    KIND_PDF,
    KIND_PNG,
    KIND_JPEG,
    KIND_GIF,
    KIND_ZIP,
    KIND_PROGRAM,
    KIND_BYTES,
    KIND_EMPTY,
    KIND_UNREAD,
    KIND_FOLDER,
    KIND_LINK,
    KIND_OTHER,
];

/// Everything this crate can say about a number of things.
pub const EVERY_COUNTED: [Counted; 2] = [NOT_WHOLE, UNNAMED];

/// Why this crate's list could not be declared.
#[derive(Debug, thiserror::Error)]
pub enum WordsError {
    /// A word that is not a phrase.
    #[error(transparent)]
    Word(#[from] alo_strings::WordError),
    /// A countable string that is not one.
    #[error(transparent)]
    Counted(#[from] alo_strings::PluralError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] alo_strings::VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
///
/// [`WordsError`], which the list above cannot cause.
pub fn finding_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// # Errors
///
/// [`WordsError::List`] if the vocabulary already holds one of these keys —
/// nothing is replaced, because a key means one string and whoever declared it
/// first said what that string is.
pub fn declare_into(vocabulary: &mut Vocabulary) -> Result<(), WordsError> {
    for word in EVERY_WORD {
        vocabulary.says(word.phrase()?)?;
    }
    for counted in EVERY_COUNTED {
        vocabulary.counts(
            Plural::counting(counted.key(), counted.number, counted.one, counted.other)?
                .noting(counted.note)?,
        )?;
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::{EVERY_COUNTED, EVERY_WORD, WordsError, declare_into, finding_words};
    use alo_strings::{Key, Vocabulary};

    /// Every key in this file is a key, checked here because a key written in
    /// this file cannot arrive from anywhere else.
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(Key::named(word.named()), Ok(word.key()), "{}", word.named());
        }
        for counted in EVERY_COUNTED {
            assert_eq!(
                Key::named(counted.named()),
                Ok(counted.key()),
                "{}",
                counted.named()
            );
        }
    }

    /// A key names one string.
    #[test]
    fn the_list_declares_into_a_vocabulary_once() {
        assert_eq!(finding_words().unwrap().how_many(), 25);
        let mut vocabulary = Vocabulary::empty();
        declare_into(&mut vocabulary).unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **Every string here carries a note.** A translator with no product in
    /// front of them cannot tell that *an index* is a thing the machine
    /// keeps, or that *bytes of no known kind* is not a fault.
    #[test]
    fn every_string_carries_a_note_for_whoever_translates_it() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
        for counted in EVERY_COUNTED {
            assert!(!counted.note().is_empty(), "{}", counted.named());
        }
    }

    /// Every key is under this crate's own area, so nothing here can collide
    /// with another crate's list by accident.
    #[test]
    fn every_key_is_this_crates_own() {
        for word in EVERY_WORD {
            assert!(
                word.named().starts_with("finding."),
                "{} is not under this crate's area",
                word.named()
            );
        }
        for counted in EVERY_COUNTED {
            assert!(
                counted.named().starts_with("finding."),
                "{}",
                counted.named()
            );
        }
    }
}
