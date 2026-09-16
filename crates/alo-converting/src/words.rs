//! Every string this crate can say, and the English beside each one.
//!
//! Four groups: a copy that was made, what it could not carry, why no copy was
//! made, and the verb. Every sentence about a copy says **what happened to the
//! original** as well, because the fear a person has at that moment is for the
//! thing they were sent.
//!
//! # Nothing a person reads names the machinery
//!
//! Not the engine, not a format's internal name, not a filter, a socket or a
//! service. A test at the bottom of this file reads every sentence for them and
//! for a hedge: what a copy lost has been found by the time any of these is
//! said, and *probably* would be the spinner this crate exists to replace.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported so this
/// crate's files name it as `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// A copy that was made.
// ---------------------------------------------------------------------------

/// A copy was made.
pub const CONVERTED: Word = Word::saying(
    "converting.converted",
    "{copy} is a PDF copy of {file}, made on this machine. {file} is exactly as it was",
)
.noting(
    "Said when a document someone was sent has been converted. {copy} and {file} are file names, \
     not translated. The second sentence matters: the person's original is never changed or \
     replaced, and they should be able to trust that without checking.",
);

/// The copy carried everything.
pub const CARRIED_EVERYTHING: Word = Word::saying(
    "converting.carried.everything",
    "Nothing was lost: every font, field, comment and linked picture in the original was checked, \
     and the copy carries all of it",
)
.noting(
    "Said after a conversion only when both the original and the copy were checked and nothing \
     differs. It must not read like a general reassurance: it is a finding, and a conversion that \
     could not be checked never says it.",
);

/// The copy did not carry everything, before the list of what.
pub const CARRIED_NOT_EVERYTHING: Word = Word::saying(
    "converting.carried.not-everything",
    "The copy does not carry everything in the original. This is what is different",
)
.noting(
    "Said after a conversion, directly before a list of sentences, each naming one thing the copy \
     could not carry over. Plain and not alarming: the copy is still usable.",
);

// ---------------------------------------------------------------------------
// What a copy could not carry.
// ---------------------------------------------------------------------------

/// A font was substituted.
pub const NOT_CARRIED_FONT: Word = Word::saying(
    "converting.not-carried.font",
    "The font {font} is not on this machine, so the text set in it is shown in a similar font \
     instead, and lines and pages can break in different places",
)
.noting(
    "One item in the list of what a copy could not carry. {font} is the name of a typeface as the \
     document gave it, such as Garamond, and is not translated. The second half is what the person \
     would notice.",
);

/// A field whose value depends on the moment is fixed.
pub const NOT_CARRIED_FIELD: Word = Word::saying(
    "converting.not-carried.field",
    "The original has {field} that changes by itself when the document is opened. The copy shows \
     it as it was at the moment it was converted, and it will not change",
)
.noting(
    "One item in the list of what a copy could not carry. {field} is one of the field names below, \
     such as \"a date\", written to be read inside this sentence.",
);

/// Macros were not carried.
pub const NOT_CARRIED_MACROS: Word = Word::saying(
    "converting.not-carried.macros",
    "The original contains macros. They were not run, and they are not in the copy",
)
.noting(
    "One item in the list of what a copy could not carry. Macros are small programs inside a \
     document; this machine never runs them from a document someone was sent.",
);

/// A linked picture was not fetched.
pub const NOT_CARRIED_LINKED_PICTURE: Word = Word::saying(
    "converting.not-carried.linked-picture",
    "The original shows a picture kept somewhere else rather than inside the document. It was not \
     fetched, so the copy does not have it. Whoever sent the document can send the picture itself",
)
.noting(
    "One item in the list of what a copy could not carry. The picture is only linked to — on the \
     sender's own computer or a server — and converting never reaches out to get it, because that \
     would send a request somewhere nobody chose.",
);

/// Linked data was not fetched.
pub const NOT_CARRIED_LINKED_DATA: Word = Word::saying(
    "converting.not-carried.linked-data",
    "The original takes some of its contents from another file kept somewhere else. It was not \
     fetched, so the copy shows what the original held when it was last saved",
)
.noting(
    "One item in the list of what a copy could not carry: data linked from another file, such as \
     a table from a spreadsheet. Converting never reaches out to get it.",
);

/// A linked template was not fetched.
pub const NOT_CARRIED_LINKED_TEMPLATE: Word = Word::saying(
    "converting.not-carried.linked-template",
    "The original is attached to a template kept somewhere else. It was not fetched, so anything \
     the document takes from it is not in the copy",
)
.noting(
    "One item in the list of what a copy could not carry: a template is a separate file a document \
     gets styles or text from. Converting never reaches out to get it.",
);

/// Some other linked content was not fetched.
pub const NOT_CARRIED_LINKED_SOMETHING_ELSE: Word = Word::saying(
    "converting.not-carried.linked-something-else",
    "The original includes something kept somewhere else rather than inside the document. It was \
     not fetched, so the copy does not have it",
)
.noting(
    "One item in the list of what a copy could not carry, for linked content that is not a \
     picture, data or a template — a video, for example.",
);

/// Comments are not shown.
pub const NOT_CARRIED_COMMENTS: Word = Word::saying(
    "converting.not-carried.comments",
    "The original has comments. The copy does not show them; open the original on a machine with \
     an office application to read them",
)
.noting(
    "One item in the list of what a copy could not carry: notes people left in the margin of the \
     document. They are not lost from the original, only absent from the copy.",
);

/// Tracked changes are not shown.
pub const NOT_CARRIED_TRACKED_CHANGES: Word = Word::saying(
    "converting.not-carried.tracked-changes",
    "The original has tracked changes. The copy does not show which parts were changed, or by whom",
)
.noting(
    "One item in the list of what a copy could not carry: edits recorded so that others can review \
     them. They are not lost from the original, only absent from the copy.",
);

// ---------------------------------------------------------------------------
// The kinds of field, read inside another sentence.
// ---------------------------------------------------------------------------

/// A date field.
pub const FIELD_DATE: Word = Word::saying("converting.field.date", "a date")
    .noting("A field that shows today's date. Read inside another sentence, with its article.");

/// A time field.
pub const FIELD_TIME: Word = Word::saying("converting.field.time", "a time")
    .noting("A field that shows the time now. Read inside another sentence, with its article.");

/// A file name field.
pub const FIELD_FILE_NAME: Word =
    Word::saying("converting.field.file-name", "the document's file name").noting(
        "A field that shows the name the document is saved under. Read inside another sentence.",
    );

/// An author field.
pub const FIELD_AUTHOR: Word = Word::saying("converting.field.author", "a person's name").noting(
    "A field that shows the name of the author or of whoever has the document open. Read inside \
         another sentence, with its article.",
);

/// A merge field.
pub const FIELD_MERGE: Word = Word::saying("converting.field.merge", "a mail merge field").noting(
    "A placeholder filled in from a list of names and addresses when letters are produced. Read \
         inside another sentence, with its article.",
);

/// A formula calling for the current moment.
pub const FIELD_THE_CURRENT_MOMENT: Word = Word::saying(
    "converting.field.the-current-moment",
    "a formula using today's date or the time now",
)
.noting("A spreadsheet formula such as one showing today's date. Read inside another sentence, with its article.");

/// A formula calling for a random number.
pub const FIELD_A_RANDOM_NUMBER: Word = Word::saying(
    "converting.field.a-random-number",
    "a formula using a random number",
)
.noting("A spreadsheet formula whose result is chosen at random each time. Read inside another sentence, with its article.");

// ---------------------------------------------------------------------------
// Why no copy was made.
// ---------------------------------------------------------------------------

/// What was approved was not converting.
pub const NOT_CONVERTING: Word = Word::saying(
    "converting.not-converting",
    "What was approved was not converting a document, so nothing was converted",
)
.noting("Said when an approval for a different action reached converting. Nothing happened.");

/// A kind this machine does not convert.
pub const NOT_A_KIND_THIS_CONVERTS: Word = Word::saying(
    "converting.not-a-kind-this-converts",
    "This is {what}, which this machine does not convert, so nothing was converted and the file is \
     exactly as it was",
)
.noting(
    "{what} is the name of a kind of file, such as \"a GIF image\", read inside this sentence. \
     This machine converts Word documents, Excel spreadsheets and PowerPoint presentations.",
);

/// After what the file cannot be, that nothing was converted.
pub const NOTHING_WAS_CONVERTED: Word = Word::saying(
    "converting.nothing-was-converted",
    "Nothing was converted, and the file is exactly as it was",
)
.noting(
    "Said directly after a sentence explaining what is wrong with a file — that it is damaged, for \
     example, or not what its name says.",
);

/// The document could not be read.
pub const UNREADABLE: Word = Word::saying(
    "converting.unreadable",
    "The document could not be read, so nothing was converted. Check that it is still where it was \
     and try again",
)
.noting("Said when the file could not be opened for reading, for example because it was moved.");

/// The document has other names.
pub const MORE_THAN_ONE_NAME: Word = Word::saying(
    "converting.more-than-one-name",
    "This document also appears under another name somewhere else on this machine, so it was not \
     read and nothing was converted. Copy it into the folder first, and convert the copy",
)
.noting(
    "Said when the file is linked from more than one place, which could let an assistant read \
     something from a folder nobody gave it. Copying the file removes the link.",
);

/// A file with the copy's name is already there.
pub const NAME_TAKEN: Word = Word::saying(
    "converting.name-taken",
    "There is already a file called {copy} in that folder, so nothing was converted and nothing \
     was replaced. Choose another folder, or rename that file first",
)
.noting(
    "{copy} is a file name, not translated. This machine never writes over a file to make a copy, \
     and never quietly picks a different name.",
);

/// The copy could not be created.
pub const COPY_NOT_CREATED: Word = Word::saying(
    "converting.copy-not-created",
    "A copy could not be made in that folder, so nothing was converted. Check that the folder is \
     still there and has room",
)
.noting("Said when the new file could not be created, for example because the disk is full.");

/// Nothing on this machine converts.
pub const NOTHING_HERE_CONVERTS: Word = Word::saying(
    "converting.nothing-here-converts",
    "The part of this machine that converts documents did not respond, so nothing was converted. \
     Restarting this machine starts it again",
)
.noting(
    "\"The part of this machine that converts documents\" is deliberate: the person never needs to \
     know what it is called. Documents are never sent anywhere else to be converted instead.",
);

/// The conversion itself failed.
pub const COULD_NOT_CONVERT: Word = Word::saying(
    "converting.could-not-convert",
    "This document could not be converted, so no copy was made and the original is exactly as it \
     was. Whoever sent it can save it again, or send it as a PDF",
)
.noting("Said when converting started and did not produce a copy.");

/// Too large to convert.
pub const TOO_LARGE: Word = Word::saying(
    "converting.too-large",
    "This document is too large to convert on this machine, so no copy was made and the original \
     is exactly as it was. Whoever sent it can send it as a PDF",
)
.noting("Said when a document, or the copy it would become, is larger than this machine converts.");

/// The original could not be checked.
pub const ORIGINAL_NOT_CHECKED: Word = Word::saying(
    "converting.original-not-checked",
    "What is inside this document could not be checked, so this machine could not say what a copy \
     would lose, and no copy was made. The original is exactly as it was",
)
.noting(
    "Said when the document is in a form that could not be examined closely enough to list what a \
     converted copy would leave out. A copy is never shown without that list.",
);

/// Took too long.
pub const TOO_SLOW: Word = Word::saying(
    "converting.too-slow",
    "Converting this document took too long and was stopped, so no copy was made and the original \
     is exactly as it was",
)
.noting("Said when converting ran past the time this machine allows and was stopped.");

/// The copy could not be checked.
pub const COPY_NOT_CHECKED: Word = Word::saying(
    "converting.copy-not-checked",
    "The copy could not be checked for what it left out, so it was not kept: a copy is never given \
     without saying what it lost. The original is exactly as it was",
)
.noting(
    "Said when a copy was produced but could not be examined. It is removed rather than shown with \
     losses nobody could list.",
);

/// The copy could not be written.
pub const COPY_NOT_WRITTEN: Word = Word::saying(
    "converting.copy-not-written",
    "The copy could not be written into that folder, so it was not kept. Check that the folder has \
     room, and try again",
)
.noting("Said when writing the converted copy failed, for example because the disk is full.");

// ---------------------------------------------------------------------------
// The verb.
// ---------------------------------------------------------------------------

/// What the verb does.
pub const VERB_PURPOSE: Word = Word::saying(
    "converting.verb.convert-document.purpose",
    "convert a document into a PDF copy, and say what the copy could not carry over",
)
.noting(
    "What an assistant's converting action does, in the list of everything it can do on this \
     machine. Lowercase because it is listed after the action's name.",
);

/// What the file argument is.
pub const VERB_FILE: Word = Word::saying(
    "converting.verb.convert-document.file",
    "the document to convert",
)
.noting("One of the two things an assistant names when it converts. Read in a list of what the action takes.");

/// What the into argument is.
pub const VERB_INTO: Word = Word::saying(
    "converting.verb.convert-document.into",
    "the folder the copy is put in",
)
.noting("One of the two things an assistant names when it converts. Read in a list of what the action takes.");

/// The sentence a person approves.
pub const VERB_SENTENCE: Word = Word::saying(
    "converting.verb.convert-document.sentence",
    "convert {file} into a PDF copy in {into}, leaving the original as it is",
)
.noting(
    "The sentence a person approves before an assistant converts a document. {file} and {into} are \
     where the document and the folder are, exactly as written on this machine, and are not \
     translated. Nothing is converted until the person approves, and approving converts once.",
);

// ---------------------------------------------------------------------------

/// Every word that names something, read inside another sentence.
pub const THE_NAMES: [Word; 10] = [
    FIELD_DATE,
    FIELD_TIME,
    FIELD_FILE_NAME,
    FIELD_AUTHOR,
    FIELD_MERGE,
    FIELD_THE_CURRENT_MOMENT,
    FIELD_A_RANDOM_NUMBER,
    VERB_PURPOSE,
    VERB_FILE,
    VERB_INTO,
];

/// Every word that is a line or a sentence of its own.
pub const THE_SENTENCES: [Word; 27] = [
    CONVERTED,
    CARRIED_EVERYTHING,
    CARRIED_NOT_EVERYTHING,
    NOT_CARRIED_FONT,
    NOT_CARRIED_FIELD,
    NOT_CARRIED_MACROS,
    NOT_CARRIED_LINKED_PICTURE,
    NOT_CARRIED_LINKED_DATA,
    NOT_CARRIED_LINKED_TEMPLATE,
    NOT_CARRIED_LINKED_SOMETHING_ELSE,
    NOT_CARRIED_COMMENTS,
    NOT_CARRIED_TRACKED_CHANGES,
    NOT_CONVERTING,
    NOT_A_KIND_THIS_CONVERTS,
    NOTHING_WAS_CONVERTED,
    UNREADABLE,
    MORE_THAN_ONE_NAME,
    NAME_TAKEN,
    COPY_NOT_CREATED,
    NOTHING_HERE_CONVERTS,
    COULD_NOT_CONVERT,
    TOO_LARGE,
    ORIGINAL_NOT_CHECKED,
    TOO_SLOW,
    COPY_NOT_CHECKED,
    COPY_NOT_WRITTEN,
    VERB_SENTENCE,
];

/// Every string this crate can say.
pub const EVERY_WORD: [Word; 37] = [
    FIELD_DATE,
    FIELD_TIME,
    FIELD_FILE_NAME,
    FIELD_AUTHOR,
    FIELD_MERGE,
    FIELD_THE_CURRENT_MOMENT,
    FIELD_A_RANDOM_NUMBER,
    VERB_PURPOSE,
    VERB_FILE,
    VERB_INTO,
    CONVERTED,
    CARRIED_EVERYTHING,
    CARRIED_NOT_EVERYTHING,
    NOT_CARRIED_FONT,
    NOT_CARRIED_FIELD,
    NOT_CARRIED_MACROS,
    NOT_CARRIED_LINKED_PICTURE,
    NOT_CARRIED_LINKED_DATA,
    NOT_CARRIED_LINKED_TEMPLATE,
    NOT_CARRIED_LINKED_SOMETHING_ELSE,
    NOT_CARRIED_COMMENTS,
    NOT_CARRIED_TRACKED_CHANGES,
    NOT_CONVERTING,
    NOT_A_KIND_THIS_CONVERTS,
    NOTHING_WAS_CONVERTED,
    UNREADABLE,
    MORE_THAN_ONE_NAME,
    NAME_TAKEN,
    COPY_NOT_CREATED,
    NOTHING_HERE_CONVERTS,
    COULD_NOT_CONVERT,
    TOO_LARGE,
    ORIGINAL_NOT_CHECKED,
    TOO_SLOW,
    COPY_NOT_CHECKED,
    COPY_NOT_WRITTEN,
    VERB_SENTENCE,
];

/// The gap holding what a file is.
pub const WHAT: &str = "what";

/// The gap holding a font's name.
pub const FONT: &str = "font";

/// The gap holding a kind of field.
pub const FIELD: &str = "field";

/// The gap holding the copy's name.
pub const COPY: &str = "copy";

/// The gap holding the document's name.
pub const FILE: &str = "file";

/// Why this crate's own words could not be declared.
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
pub fn converting_words() -> Result<Vocabulary, WordsError> {
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
    use super::*;
    use std::collections::BTreeSet;

    /// Every key is a key, in this crate's area, and names one string.
    #[test]
    fn every_key_is_one_of_this_crates_and_names_one_string() {
        for word in EVERY_WORD {
            assert_eq!(alo_strings::Key::named(word.named()), Ok(word.key()));
            assert_eq!(word.key().area(), "converting", "{}", word.named());
        }
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// The groups are the whole list, in its order.
    #[test]
    fn the_groups_are_the_whole_list() {
        let mut all: Vec<&str> = THE_NAMES.iter().map(Word::named).collect();
        all.extend(THE_SENTENCES.iter().map(Word::named));
        let every: Vec<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(all, every);
    }

    /// The list declares, and a key already taken is not replaced.
    #[test]
    fn the_list_declares_and_nothing_is_replaced() {
        let mut vocabulary = converting_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert!(matches!(
            declare_into(&mut vocabulary),
            Err(WordsError::List(_))
        ));
    }

    /// **Every word carries a note**, and every gap is one this crate fills.
    #[test]
    fn every_word_carries_a_note_and_only_gaps_this_crate_fills() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
            for gap in word.phrase().unwrap().source().gaps() {
                assert!(
                    [WHAT, FONT, FIELD, COPY, FILE, "into"].contains(&gap.as_str()),
                    "{} has a gap called {gap}",
                    word.named()
                );
            }
        }
    }

    /// **Nothing here hedges and nothing names the machinery.** What a copy
    /// lost has been found by the time it is said.
    #[test]
    fn nothing_here_hedges_or_names_the_machinery() {
        for word in EVERY_WORD {
            let said = word.says().to_lowercase();
            for forbidden in [
                "probably",
                "perhaps",
                "possibly",
                "likely",
                "might",
                concat!("libre", "office"),
                concat!("sof", "fice"),
                "office suite",
                "engine",
                "filter",
                "socket",
                "service",
                "daemon",
                "xml",
                "zip",
                "docx",
                "xlsx",
                "pptx",
                "mime",
                "error",
                "code",
                "upload",
            ] {
                assert!(
                    !said.contains(forbidden),
                    "{} says \"{forbidden}\"",
                    word.named()
                );
            }
            assert!(
                !said.chars().any(|letter| letter.is_ascii_digit()),
                "{} has a number in it",
                word.named()
            );
        }
    }

    /// **Lost nothing and did not check can never read the same**: no sentence
    /// for a copy that could not be checked says nothing was lost.
    #[test]
    fn lost_nothing_and_not_checked_never_read_the_same() {
        for word in [ORIGINAL_NOT_CHECKED, COPY_NOT_CHECKED] {
            assert!(
                !word.says().contains("Nothing was lost"),
                "{}",
                word.named()
            );
            assert_ne!(word.says(), CARRIED_EVERYTHING.says());
        }
        assert!(CARRIED_EVERYTHING.says().contains("checked"));
    }
}
