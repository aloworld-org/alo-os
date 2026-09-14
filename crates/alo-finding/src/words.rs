//! Every string this crate can say, and the English beside each one.
//!
//! An index is names, kinds, dates and words, and a name needs no
//! translation. What needs one is every place a window shows a sentence
//! instead: the kind of a file, why a file has no words in the index, why a
//! folder could not be indexed at all, what a search did not look at, and why
//! a query was not one. Each is declared here so that nothing
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

/// The folder is not on the list of indexed folders.
pub const NEVER_INDEXED: Word = Word::saying(
    "finding.never-indexed",
    "{at} was never indexed, so there is no index of it to search.",
)
.noting(
    "Said when a folder is asked about that is not on the list of folders the person asked to \
     have indexed. {at} is the folder's path and is never translated. Nothing is wrong: the \
     folder can be indexed, and then it can be searched.",
);

/// The index file could not be removed.
pub const NOT_REMOVED: Word = Word::saying(
    "finding.not-removed",
    "The index at {at} could not be removed: {why}",
)
.noting(
    "Said when a folder was to be forgotten and the file its index is kept in could not be \
     deleted. {at} is the file's path and is never translated. {why} is what the operating \
     system said, in its own words. The folder stays on the list until the file is gone.",
);

/// The list of indexed folders could not be read.
pub const LIST_NOT_READ: Word = Word::saying(
    "finding.list-not-read",
    "The list of indexed folders at {at} could not be read: {why}",
)
.noting(
    "Said when the file that lists which folders are indexed is there and could not be \
     opened. {at} is the file's path and is never translated. {why} is what the operating \
     system said, in its own words.",
);

/// The file is not a list of indexed folders.
pub const NOT_A_LIST: Word = Word::saying(
    "finding.not-a-list",
    "{at} is not a list of indexed folders this machine can read: {why}",
)
.noting(
    "Said when the file that should list which folders are indexed holds something else: a \
     write that was cut short, another program's file, or a list from a later version of alo \
     OS. {at} is the file's path and is never translated. {why} says what was wrong, as a \
     sentence.",
);

/// The list of indexed folders could not be written.
pub const LIST_NOT_KEPT: Word = Word::saying(
    "finding.list-not-kept",
    "The list of indexed folders could not be written to {at}: {why}",
)
.noting(
    "Said when the file that lists which folders are indexed could not be written: the disk \
     is full, the folder is read-only. {at} is the file's path and is never translated. {why} \
     is what the operating system said, in its own words.",
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

/// An index with a folder no walk could list to its end.
pub const NOT_WHOLE: Counted = Counted {
    named: "finding.not-whole",
    number: "most",
    one: "A folder under it holds more than one thing at a single level, so the index is not \
          the whole of the folder.",
    other: "A folder under it holds more than {most} things at a single level, so the index is \
            not the whole of the folder.",
    note: "Said once, above an index, when a folder under the one indexed holds more things \
           directly inside it than one walk looks at, which no walk can list to its end. {most} \
           is that limit; a folder holding more than it in subfolders is indexed whole. What was \
           indexed is true, and a search says which folders were left.",
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

// ---------------------------------------------------------------------------
// A query that is not one — `crate::NotAsked`.
// ---------------------------------------------------------------------------

/// Nothing was asked.
pub const NOT_ASKED_NOTHING: Word = Word::saying(
    "finding.not-asked.nothing",
    "Nothing was asked, so nothing was searched: a search needs a name, a kind, a date or some \
     words.",
)
.noting(
    "Said when a search was started with nothing to look for: an empty box, or only spaces or \
     punctuation. Nothing is wrong and nothing was searched; the person types something and \
     searches again.",
);

/// More words than a sentence.
pub const NOT_ASKED_MORE_THAN_A_SENTENCE: Word = Word::saying(
    "finding.not-asked.more-than-a-sentence",
    "{words} words is more than a sentence, and a search takes at most {most}.",
)
.noting(
    "Said when the words to search for run longer than a search takes, which is what happens \
     when a whole paragraph is pasted into the search box. {words} is how many words were given \
     and {most} is the limit; both are numbers.",
);

/// Longer than a name.
pub const NOT_ASKED_LONGER_THAN_A_NAME: Word = Word::saying(
    "finding.not-asked.longer-than-a-name",
    "{chars} characters is longer than any file name, and a search takes at most {most}.",
)
.noting(
    "Said when the part of a name to search for, or a single word, is longer than any file's \
     name can be, so it is in no file. {chars} is how many characters were given and {most} is \
     the limit; both are numbers.",
);

// ---------------------------------------------------------------------------
// What a search did not look at — `crate::NotSearched`.
// ---------------------------------------------------------------------------

/// Nothing outside the indexed folder was searched.
pub const NOT_SEARCHED_OUTSIDE: Word = Word::saying(
    "finding.not-searched.outside",
    "Nothing outside {folder} was searched.",
)
.noting(
    "Shown beside every search answer, first, so that a person knows which folder was \
     searched and that the rest of the disk was not. {folder} is a path and is never \
     translated.",
);

/// A folder the machine would not read.
pub const NOT_SEARCHED_FOLDER_UNREAD: Word = Word::saying(
    "finding.not-searched.folder-unread",
    "{below} could not be read, so nothing in it was searched: {why}",
)
.noting(
    "Shown beside a search answer for each folder inside the searched one that the machine \
     would not let the index open, usually because it belongs to another person. {below} is \
     the folder's path and is never translated. {why} is what the operating system said, in \
     its own words.",
);

/// A folder on another disk.
pub const NOT_SEARCHED_ELSEWHERE: Word = Word::saying(
    "finding.not-searched.elsewhere",
    "{below} is on another disk, so nothing in it was searched.",
)
.noting(
    "Shown beside a search answer for each folder inside the searched one that is really on \
     another disk or drive, which an index does not enter. {below} is the folder's path and is \
     never translated.",
);

/// A folder the index stopped in.
pub const NOT_SEARCHED_NOT_ENTERED: Word = Word::saying(
    "finding.not-searched.not-entered",
    "The index stopped before it had finished {below}, so not all of it was searched.",
)
.noting(
    "Shown beside a search answer for each folder no walk could list to its end, because it \
     holds more things at one level than one walk looks at. Read together with \
     \"finding.not-whole\", which says how many that is. {below} is the folder's path and is \
     never translated.",
);

/// A file the machine would not open.
pub const NOT_SEARCHED_FILE_UNREAD: Word = Word::saying(
    "finding.not-searched.file-unread",
    "{below} could not be read, so it was not searched: {why}",
)
.noting(
    "Shown beside a search answer for each file the machine would not let the index open, so \
     that neither what kind of file it is nor its words could be searched. {below} is the \
     file's path and is never translated. {why} is what the operating system said, in its own \
     words.",
);

/// Files of a kind with no reader.
pub const NOT_SEARCHED_NO_READER: Counted = Counted {
    named: "finding.not-searched.no-reader",
    number: "files",
    one: "One file is of a kind whose words cannot be read, so it was not searched by its words.",
    other: "{files} files are of kinds whose words cannot be read, so they were not searched by \
            their words.",
    note: "Said once beside a search by words, counting the images, PDFs, archives and files of \
           no known kind, whose words the index does not hold. A search by name still finds \
           them. {files} is how many.",
};

/// Files larger than an index reads.
pub const NOT_SEARCHED_TOO_BIG: Counted = Counted {
    named: "finding.not-searched.too-big",
    number: "files",
    one: "One file is larger than an index reads, so it was not searched by its words.",
    other: "{files} files are larger than an index reads, so they were not searched by their \
            words.",
    note: "Said once beside a search by words, counting the files too large for their words to \
           be indexed. A search by name still finds them. {files} is how many.",
};

// ---------------------------------------------------------------------------
// The one verb: what it does, what a person is shown, and what each argument
// is for. `crate::verbs` declares it from these.
// ---------------------------------------------------------------------------

/// What `search_files` does.
pub const SEARCH_FILES: Word = Word::saying(
    "finding.verb.search-files.purpose",
    "search the index of a folder for files by what they are called",
)
.noting(
    "The one-sentence purpose of the search verb, shown where a person reads what their agent \
     can do. \"The index\" is the list this machine keeps of a folder's files, so that a search \
     does not read the disk again.",
);

/// What a person is shown when `search_files` runs. It is a read, so nobody
/// is asked to approve it — it is what the shell shows while it happens.
pub const SEARCH_FILES_SENTENCE: Word = Word::saying(
    "finding.verb.search-files.sentence",
    "search the index of {folder} for files whose name contains {named}",
)
.noting(
    "{folder} is a path and {named} is part of a file's name; neither is translated. {named} \
     is not a pattern and not an expression: this verb interprets nothing.",
);

/// Where `search_files` looks.
pub const SEARCH_FILES_FOLDER: Word = Word::saying(
    "finding.verb.search-files.argument.folder",
    "the folder whose index to search",
)
.noting("Shown beside the argument when an agent asks for it. The folder has to have an index.");

/// What `search_files` looks for.
pub const SEARCH_FILES_NAMED: Word = Word::saying(
    "finding.verb.search-files.argument.named",
    "the words a file's name has to contain",
)
.noting("Shown beside the argument when an agent asks for it. Part of a name, never a pattern.");

/// A verb that is not this crate's to answer.
pub const NOT_THIS_CRATES: Word = Word::saying(
    "finding.verb.not-this-crates",
    "nothing in the index does {verb} — it searches, and anything else is somewhere else's to do",
)
.noting("{verb} is the name of a capability and is never translated.");

/// A verb performed without an argument it declares.
pub const MISSING: Word = Word::saying(
    "finding.verb.missing",
    "{verb} was performed without {argument}, which it declares — this is a verb to write again, \
     not a call to make again",
)
.noting(
    "Both gaps are names in the code and are never translated. This sentence is read by whoever \
     wrote the verb rather than by whoever asked for it.",
);

/// Everything this crate can say in one sentence each.
pub const EVERY_WORD: [Word; 42] = [
    NOT_ASKED_NOTHING,
    NOT_ASKED_MORE_THAN_A_SENTENCE,
    NOT_ASKED_LONGER_THAN_A_NAME,
    NOT_SEARCHED_OUTSIDE,
    NOT_SEARCHED_FOLDER_UNREAD,
    NOT_SEARCHED_ELSEWHERE,
    NOT_SEARCHED_NOT_ENTERED,
    NOT_SEARCHED_FILE_UNREAD,
    NOT_ABSOLUTE,
    NOT_WALKED,
    NOT_THE_SAME,
    NOT_KEPT,
    NOT_OPENED,
    NOT_AN_INDEX,
    NOWHERE_TO_KEEP_IT,
    NEVER_INDEXED,
    NOT_REMOVED,
    LIST_NOT_READ,
    NOT_A_LIST,
    LIST_NOT_KEPT,
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
    SEARCH_FILES,
    SEARCH_FILES_SENTENCE,
    SEARCH_FILES_FOLDER,
    SEARCH_FILES_NAMED,
    NOT_THIS_CRATES,
    MISSING,
];

/// Everything this crate can say about a number of things.
pub const EVERY_COUNTED: [Counted; 4] = [
    NOT_WHOLE,
    UNNAMED,
    NOT_SEARCHED_NO_READER,
    NOT_SEARCHED_TOO_BIG,
];

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
        assert_eq!(finding_words().unwrap().how_many(), 46);
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
