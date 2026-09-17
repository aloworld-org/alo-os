//! Every string this crate can say, and the English beside each one.
//!
//! Four groups.
//!
//! **The names of what a file is.** Every one is read *inside* another sentence
//! — *This is {what}* — so each is lowercase and carries its article, and the
//! note on each says so. They are the names a person already uses for what they
//! were sent: *a Word document*, *a PDF document*, *a PNG image*. None of them is
//! a media type, an extension or the name of anything alo OS runs to read one,
//! because a person who has to learn what `application/vnd.ms-excel` means
//! before they can understand why a file did not open has been handed the
//! machine's problem.
//!
//! **What this machine can do with it.** One sentence for each outcome in
//! [`crate::Outcome`] and each reason in [`crate::Cannot`], so that a sentence
//! can never cover two outcomes and send a person to neither. *This file is
//! damaged* and *this machine does not recognise what this file is* send a person
//! in opposite directions — back to whoever sent it, or to another machine — and
//! they are two sentences for that reason.
//!
//! **What the name got wrong.** A file named as one thing that is another is
//! said to be exactly that, before anything else is said about it.
//!
//! **What would open it.** One sentence for each [`crate::Would`], read straight
//! after the reason a file cannot be opened: a complete copy, a copy without
//! the password, the document itself, another machine or format, or whoever
//! made it. None of them offers to send the file anywhere, and a test holds
//! that.
//!
//! # There is no word here for "probably"
//!
//! Nothing here says *may*, *might* or *probably*, and a test holds that. What a
//! file is has been decided by the time any of these is said, and a sentence
//! that hedges is the spinner this crate exists to replace, written down.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported because this
/// crate's files and the tests that read its list name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// What a file is — [`crate::Kind`], and the two things that are not a kind.
// ---------------------------------------------------------------------------

/// A PDF document.
pub const KIND_PDF: Word = Word::saying("opening.kind.pdf", "a PDF document").noting(
    "The name of a kind of file, read inside another sentence: \"This is {what}\". Lowercase \
     because it sits in the middle of one, with its article because the English sentence needs \
     it; a language that marks this with a case ending or no article should do that instead. \
     PDF is the name people use for these documents and is normally left as it is.",
);

/// A current Word document.
pub const KIND_WORD_DOCUMENT: Word = Word::saying("opening.kind.word-document", "a Word document")
    .noting(
        "The name of a kind of file, read inside another sentence: \"This is {what}\". A document \
         made with Microsoft Word in the format it has saved in since 2007. Use the name people in \
         this language use for these files; the product name Word is normally not translated.",
    );

/// A current Excel workbook.
pub const KIND_EXCEL_WORKBOOK: Word =
    Word::saying("opening.kind.excel-workbook", "an Excel spreadsheet").noting(
        "The name of a kind of file, read inside another sentence: \"This is {what}\". A \
         spreadsheet made with Microsoft Excel in the format it has saved in since 2007. The \
         product name Excel is normally not translated.",
    );

/// A current PowerPoint presentation.
pub const KIND_POWERPOINT_PRESENTATION: Word = Word::saying(
    "opening.kind.powerpoint-presentation",
    "a PowerPoint presentation",
)
.noting(
    "The name of a kind of file, read inside another sentence: \"This is {what}\". A slide \
         presentation made with Microsoft PowerPoint in the format it has saved in since 2007. The \
         product name PowerPoint is normally not translated.",
);

/// An OpenDocument text document.
pub const KIND_OPENDOCUMENT_TEXT: Word = Word::saying(
    "opening.kind.opendocument-text",
    "an OpenDocument text document",
)
.noting(
    "The name of a kind of file, read inside another sentence: \"This is {what}\". A written \
         document in the open standard format for office documents. OpenDocument is the name of \
         that standard and is normally left as it is.",
);

/// An OpenDocument spreadsheet.
pub const KIND_OPENDOCUMENT_SPREADSHEET: Word = Word::saying(
    "opening.kind.opendocument-spreadsheet",
    "an OpenDocument spreadsheet",
)
.noting(
    "The name of a kind of file, read inside another sentence: \"This is {what}\". A spreadsheet \
     in the open standard format for office documents. OpenDocument is the name of that standard \
     and is normally left as it is.",
);

/// An OpenDocument presentation.
pub const KIND_OPENDOCUMENT_PRESENTATION: Word = Word::saying(
    "opening.kind.opendocument-presentation",
    "an OpenDocument presentation",
)
.noting(
    "The name of a kind of file, read inside another sentence: \"This is {what}\". A slide \
     presentation in the open standard format for office documents. OpenDocument is the name of \
     that standard and is normally left as it is.",
);

/// A Word document in the format before 2007.
pub const KIND_OLDER_WORD_DOCUMENT: Word = Word::saying(
    "opening.kind.older-word-document",
    "a Word document in the older format",
)
.noting(
    "The name of a kind of file, read inside another sentence: \"This is {what}\". A document made \
     with Microsoft Word in the format it used before 2007. \"Older\" distinguishes the format, \
     not the age of the document: a file saved in this format yesterday is still in the older \
     format.",
);

/// An Excel workbook in the format before 2007.
pub const KIND_OLDER_EXCEL_WORKBOOK: Word = Word::saying(
    "opening.kind.older-excel-workbook",
    "an Excel spreadsheet in the older format",
)
.noting(
    "The name of a kind of file, read inside another sentence: \"This is {what}\". A spreadsheet \
     made with Microsoft Excel in the format it used before 2007. \"Older\" distinguishes the \
     format, not the age of the spreadsheet.",
);

/// A PowerPoint presentation in the format before 2007.
pub const KIND_OLDER_POWERPOINT_PRESENTATION: Word = Word::saying(
    "opening.kind.older-powerpoint-presentation",
    "a PowerPoint presentation in the older format",
)
.noting(
    "The name of a kind of file, read inside another sentence: \"This is {what}\". A presentation \
     made with Microsoft PowerPoint in the format it used before 2007. \"Older\" distinguishes the \
     format, not the age of the presentation.",
);

/// A rich text document.
pub const KIND_RICH_TEXT: Word = Word::saying("opening.kind.rich-text", "a rich text document")
    .noting(
        "The name of a kind of file, read inside another sentence: \"This is {what}\". A simple \
         formatted document — text with bold, italics and fonts — in the old interchange format \
         most word processors can save in. Use whatever name word processors in this language \
         give that format.",
    );

/// Plain text.
pub const KIND_TEXT: Word = Word::saying("opening.kind.text", "plain text").noting(
    "The name of a kind of file, read inside another sentence: \"This is {what}\". A file of \
     written characters and nothing else — no formatting, no pictures. There is no article in the \
     English because \"This is plain text\" needs none.",
);

/// Plain text in a character set older than Unicode.
pub const KIND_TEXT_IN_AN_OLDER_CHARACTER_SET: Word = Word::saying(
    "opening.kind.text-in-an-older-character-set",
    "plain text written in an older character set",
)
.noting(
    "The name of a kind of file, read inside another sentence: \"This is {what}\". Plain text \
     saved in one of the character sets computers used before one set covered every language — \
     the kind of file where letters such as ä, ł or ş can come out as the wrong symbols. A person \
     meets this most often as a spreadsheet exported from an older program.",
);

/// A PNG image.
pub const KIND_PNG_IMAGE: Word = Word::saying("opening.kind.png-image", "a PNG image").noting(
    "The name of a kind of file, read inside another sentence: \"This is {what}\". A picture in the \
     format PNG, which is its usual name and is normally left as it is.",
);

/// A JPEG image.
pub const KIND_JPEG_IMAGE: Word = Word::saying("opening.kind.jpeg-image", "a JPEG image").noting(
    "The name of a kind of file, read inside another sentence: \"This is {what}\". A picture in the \
     format JPEG — most photographs are — which is its usual name and is normally left as it is.",
);

/// A GIF image.
pub const KIND_GIF_IMAGE: Word = Word::saying("opening.kind.gif-image", "a GIF image").noting(
    "The name of a kind of file, read inside another sentence: \"This is {what}\". A picture, \
     sometimes a short moving one, in the format GIF, which is its usual name and is normally left \
     as it is.",
);

/// A WebP image.
pub const KIND_WEBP_IMAGE: Word = Word::saying("opening.kind.webp-image", "a WebP image").noting(
    "The name of a kind of file, read inside another sentence: \"This is {what}\". A picture in the \
     format WebP, common on web pages, which is its usual name and is normally left as it is.",
);

/// A zip archive.
pub const KIND_ZIP_ARCHIVE: Word = Word::saying("opening.kind.zip-archive", "a zip archive")
    .noting(
        "The name of a kind of file, read inside another sentence: \"This is {what}\". One file \
         holding other files packed and compressed together — what a person gets when they \
         download several files at once. Use the name people in this language use for these.",
    );

/// A program the machine could run.
pub const A_PROGRAM: Word = Word::saying("opening.appears.a-program", "a program").noting(
    "What a file is, read inside another sentence: \"This file is named as a PDF document, but it \
     is {what}\". A file a computer can run, as opposed to a document it shows.",
);

/// An Office document that is protected with a password.
pub const A_PASSWORD_PROTECTED_DOCUMENT: Word = Word::saying(
    "opening.appears.a-password-protected-document",
    "an Office document protected with a password",
)
.noting(
    "What a file is, read inside another sentence: \"This file is named as a Word document, but it \
     is {what}\". A Word, Excel or PowerPoint file its author locked with a password, so its \
     contents cannot be read without it. Office here means the family of Microsoft programs.",
);

/// A compressed file, which is what a damaged zip-based file is known to be.
pub const CONTAINER_COMPRESSED: Word =
    Word::saying("opening.container.compressed", "a compressed file").noting(
        "What a damaged file is known to be, read inside another sentence: \"This file is damaged: \
         it is {what}, but it has been cut short or does not hold together inside\". Current \
         office documents and zip archives are all compressed files underneath, and a damaged one \
         cannot be looked inside far enough to say which, so the sentence says only what is known.",
    );

/// The storage format older Office documents are kept in.
pub const CONTAINER_OLDER_OFFICE: Word = Word::saying(
    "opening.container.older-office",
    "a file in the format older Office documents are stored in",
)
.noting(
    "What a damaged file is known to be, read inside another sentence: \"This file is damaged: it \
     is {what}, but it has been cut short or does not hold together inside\". The storage format \
     Word, Excel and PowerPoint used before 2007; a damaged one cannot be looked inside far enough \
     to say which of the three it was.",
);

// ---------------------------------------------------------------------------
// What this machine can do with it — [`crate::Outcome`] and [`crate::Cannot`].
// ---------------------------------------------------------------------------

/// It opens as it is.
pub const OPENS_AS_IT_IS: Word = Word::saying(
    "opening.opens-as-it-is",
    "This is {what}, and this machine opens it as it is",
)
.noting(
    "Said before a file is opened, when this machine can show it without changing it. {what} is \
     the name of the kind of file from this same area, already translated — \"a PDF document\", \
     \"a PNG image\". \"As it is\" matters: nothing is converted.",
);

/// It opens by converting a copy.
pub const CONVERTS_A_COPY: Word = Word::saying(
    "opening.converts-a-copy",
    "This is {what}, and this machine opens it by converting a copy into {into}. The original \
     stays exactly as it was, and what the copy could not carry over is said once it has been \
     converted",
)
.noting(
    "Said before a file is converted, when this machine cannot show it as it is but can make a \
     copy in a format it does show. {what} and {into} are names of kinds of file from this same \
     area, already translated. The second sentence is a promise the machine keeps: the file the \
     person was sent is never changed, and after converting they are told exactly what did not \
     survive. Do not soften it into \"some things may be lost\" — nothing here is guessed.",
);

/// Its macros will not run.
pub const MACROS_WILL_NOT_RUN: Word = Word::saying(
    "opening.macros.will-not-run",
    "It contains macros, and they will not run",
)
.noting(
    "Said after the sentence saying a document opens as it is, when the document has macros in it: \
     small programs written into a document that some office programs run when it is opened. On \
     this machine they never run. It is a statement of fact, not a warning that something is \
     wrong with the document.",
);

/// Its macros will neither run nor be carried into the copy.
pub const MACROS_WILL_NOT_BE_CARRIED: Word = Word::saying(
    "opening.macros.will-not-be-carried",
    "It contains macros: they will not run, and they will not be carried into the copy",
)
.noting(
    "Said after the sentence saying a document opens by converting a copy, when the document has \
     macros in it: small programs written into a document that some office programs run when it \
     is opened. They never run on this machine and the converted copy does not contain them; the \
     original still does.",
);

/// The file is empty.
pub const CANNOT_EMPTY: Word = Word::saying(
    "opening.cannot.empty",
    "This file is empty: there is nothing in it to open",
)
.noting(
    "Said when a file has no contents at all. It is deliberately not \"damaged\": nothing inside \
     it is broken, there is simply nothing inside it — often a download or a copy that never \
     finished, which the person can ask for again.",
);

/// Nothing about the file is recognised.
pub const CANNOT_UNRECOGNISED: Word = Word::saying(
    "opening.cannot.unrecognised",
    "This machine does not recognise what this file is, so it has not opened it",
)
.noting(
    "Said when a file's contents match no kind of file this machine knows. It is a statement about \
     this machine, not a fault in the file or the person — the file may be perfectly good and \
     meant for a program this machine does not have. It must not suggest the file is damaged: \
     another sentence says that, and the two send a person in opposite directions.",
);

/// The file is damaged.
pub const CANNOT_DAMAGED: Word = Word::saying(
    "opening.cannot.damaged",
    "This file is damaged: it is {what}, but it has been cut short or does not hold together \
     inside, so it cannot be opened as it is",
)
.noting(
    "Said when a file is recognisably one kind of file but its insides are broken — usually a \
     download or copy that stopped part way. {what} is the name of what it is known to be, from \
     this same area, already translated. It must read as a fact about the file, and it points the \
     person back to whoever sent it: asking for it again is what helps.",
);

/// The file is a program.
pub const CANNOT_A_PROGRAM: Word = Word::saying(
    "opening.cannot.a-program",
    "This file is a program. Opening a file never runs a program, so it has not been opened",
)
.noting(
    "Said when a file somebody tried to open as a document is a program a computer could run. On \
     this machine opening a file only ever shows it; it never starts one. The sentence is calm on \
     purpose — it says what the machine did, not that the person did something dangerous.",
);

/// The file is locked with a password.
pub const CANNOT_PASSWORD_PROTECTED: Word = Word::saying(
    "opening.cannot.password-protected",
    "This is an Office document protected with a password, and this machine cannot open a \
     document protected that way",
)
.noting(
    "Said when a Word, Excel or PowerPoint file has been locked with a password by its author. \
     Office here means the family of Microsoft programs. It says what the machine cannot do, not \
     that the document is broken — the person who sent it can send a copy without the password.",
);

/// Nothing on this machine opens or converts this kind of file.
pub const CANNOT_NOTHING_HERE_OPENS_IT: Word = Word::saying(
    "opening.cannot.nothing-here-opens-it",
    "This is {what}, and nothing on this machine opens it or converts it into something that does",
)
.noting(
    "Said when a file is recognised — this machine knows exactly what it is — but has nothing that \
     shows that kind of file, either as it is or by converting a copy. {what} is the name of the \
     kind of file from this same area, already translated. It is about this machine: the file is \
     fine.",
);

// ---------------------------------------------------------------------------
// What would open it — [`crate::Would`], said after each reason.
// ---------------------------------------------------------------------------

/// A complete copy would open.
pub const WOULD_A_COMPLETE_COPY: Word = Word::saying(
    "opening.would.a-complete-copy",
    "What would open is a complete copy, which whoever has the original can send again",
)
.noting(
    "Said straight after the sentence saying a file is empty or damaged, as the next thing a \
     person reads. It sends them back to whoever sent the file: the file itself is incomplete, and \
     another copy of the original is what helps. It must not suggest another program or another \
     machine — nothing is wrong with this machine, and sending a person there wastes their time.",
);

/// A copy without the password would open.
pub const WOULD_A_COPY_WITHOUT_THE_PASSWORD: Word = Word::saying(
    "opening.would.a-copy-without-the-password",
    "What would open is a copy saved without the password, which whoever sent it can make",
)
.noting(
    "Said straight after the sentence saying a Word, Excel or PowerPoint file is protected with a \
     password this machine cannot open. The author, or whoever sent it, can save a copy with the \
     protection removed. It does not ask the person for the password.",
);

/// The document itself would open.
pub const WOULD_THE_DOCUMENT_ITSELF: Word = Word::saying(
    "opening.would.the-document-itself",
    "If a document was expected, whoever sent it can send the document itself instead",
)
.noting(
    "Said straight after the sentence saying a file is a program, which this machine never runs. \
     A person who was expecting an invoice or a letter and received a program should ask the \
     sender for the actual document. Calm and practical on purpose: it does not call the file \
     dangerous or the person careless.",
);

/// Another machine, or a different format, would open.
pub const WOULD_ANOTHER_MACHINE_OR_FORMAT: Word = Word::saying(
    "opening.would.another-machine-or-format",
    "What would open it is a machine with a program for this kind of file, or a copy saved in a \
     different format by whoever sent it",
)
.noting(
    "Said straight after the sentence saying this machine knows what a file is but has nothing \
     that shows that kind of file. The file is fine: another computer with a suitable program can \
     open it, or the sender can save it in another format. It never offers to send the file \
     anywhere, and it names no particular program.",
);

/// Whoever sent it knows what made it.
pub const WOULD_WHOEVER_MADE_IT: Word = Word::saying(
    "opening.would.whoever-made-it",
    "Whoever sent it can say which program made it, or send a copy saved in a different format",
)
.noting(
    "Said straight after the sentence saying this machine does not recognise what a file is. \
     Because the machine does not know what the file is, it cannot say which machine or program \
     would open it — the sender can. It must not suggest the file is damaged, and it never offers \
     to send the file anywhere to find out.",
);

// ---------------------------------------------------------------------------
// What the name got wrong — [`crate::Decided::NotWhatItsNameSays`].
// ---------------------------------------------------------------------------

/// The file is named as one kind and is another.
pub const NAMED_AS_SOMETHING_ELSE: Word = Word::saying(
    "opening.named-as-something-else",
    "This file is named as {named}, but it is {what}",
)
.noting(
    "Said first, before anything else about a file, when the end of its name says it is one kind \
     of file and its contents are another — a file called \"invoice.pdf\" that is really a \
     picture, or a program. {named} and {what} are names of kinds of file from this same area, \
     already translated. It states both plainly and does not accuse anyone: often it is an honest \
     mistake, sometimes it is not, and the person decides which.",
);

/// The file is named as one kind and is not one, nor anything recognised.
pub const NAMED_AS_WHAT_IT_IS_NOT: Word = Word::saying(
    "opening.named-as-what-it-is-not",
    "This file is named as {named}, but it is not one",
)
.noting(
    "Said first, before anything else about a file, when the end of its name says it is one kind of \
     file and its contents are not that kind, nor any kind this machine recognises. {named} is the \
     name of a kind of file from this same area, already translated, and \"one\" refers back to \
     it.",
);

/// The file could not be read at all, so nothing was decided.
pub const UNREADABLE: Word = Word::saying(
    "opening.unreadable",
    "This file could not be read, so nothing has been decided about it",
)
.noting(
    "Said when the machine could not read a file at all — it was removed while being looked at, or \
     the disk it is on stopped answering. It is not a statement about what the file is: nothing \
     was learned about it, and the sentence must not suggest the file is damaged or unsupported.",
);

/// Every name of what a file is, in the order [`crate::Kind::EVERY`] lists
/// kinds, followed by the two that are not kinds and the two containers.
pub const THE_NAMES: [Word; 22] = [
    KIND_PDF,
    KIND_WORD_DOCUMENT,
    KIND_EXCEL_WORKBOOK,
    KIND_POWERPOINT_PRESENTATION,
    KIND_OPENDOCUMENT_TEXT,
    KIND_OPENDOCUMENT_SPREADSHEET,
    KIND_OPENDOCUMENT_PRESENTATION,
    KIND_OLDER_WORD_DOCUMENT,
    KIND_OLDER_EXCEL_WORKBOOK,
    KIND_OLDER_POWERPOINT_PRESENTATION,
    KIND_RICH_TEXT,
    KIND_TEXT,
    KIND_TEXT_IN_AN_OLDER_CHARACTER_SET,
    KIND_PNG_IMAGE,
    KIND_JPEG_IMAGE,
    KIND_GIF_IMAGE,
    KIND_WEBP_IMAGE,
    KIND_ZIP_ARCHIVE,
    A_PROGRAM,
    A_PASSWORD_PROTECTED_DOCUMENT,
    CONTAINER_COMPRESSED,
    CONTAINER_OLDER_OFFICE,
];

/// Every sentence, which stands on its own.
pub const THE_SENTENCES: [Word; 13] = [
    OPENS_AS_IT_IS,
    CONVERTS_A_COPY,
    MACROS_WILL_NOT_RUN,
    MACROS_WILL_NOT_BE_CARRIED,
    CANNOT_EMPTY,
    CANNOT_UNRECOGNISED,
    CANNOT_DAMAGED,
    CANNOT_A_PROGRAM,
    CANNOT_PASSWORD_PROTECTED,
    CANNOT_NOTHING_HERE_OPENS_IT,
    NAMED_AS_SOMETHING_ELSE,
    NAMED_AS_WHAT_IT_IS_NOT,
    UNREADABLE,
];

/// What would open a file this machine cannot, in the order
/// [`crate::Would::EVERY`] lists them. Each stands on its own and is read
/// straight after the sentence saying why.
pub const THE_REMEDIES: [Word; 5] = [
    WOULD_A_COMPLETE_COPY,
    WOULD_A_COPY_WITHOUT_THE_PASSWORD,
    WOULD_THE_DOCUMENT_ITSELF,
    WOULD_ANOTHER_MACHINE_OR_FORMAT,
    WOULD_WHOEVER_MADE_IT,
];

/// Every string this crate can say: the names, the sentences, then what would
/// open a file this machine cannot.
pub const EVERY_WORD: [Word; 40] = [
    KIND_PDF,
    KIND_WORD_DOCUMENT,
    KIND_EXCEL_WORKBOOK,
    KIND_POWERPOINT_PRESENTATION,
    KIND_OPENDOCUMENT_TEXT,
    KIND_OPENDOCUMENT_SPREADSHEET,
    KIND_OPENDOCUMENT_PRESENTATION,
    KIND_OLDER_WORD_DOCUMENT,
    KIND_OLDER_EXCEL_WORKBOOK,
    KIND_OLDER_POWERPOINT_PRESENTATION,
    KIND_RICH_TEXT,
    KIND_TEXT,
    KIND_TEXT_IN_AN_OLDER_CHARACTER_SET,
    KIND_PNG_IMAGE,
    KIND_JPEG_IMAGE,
    KIND_GIF_IMAGE,
    KIND_WEBP_IMAGE,
    KIND_ZIP_ARCHIVE,
    A_PROGRAM,
    A_PASSWORD_PROTECTED_DOCUMENT,
    CONTAINER_COMPRESSED,
    CONTAINER_OLDER_OFFICE,
    OPENS_AS_IT_IS,
    CONVERTS_A_COPY,
    MACROS_WILL_NOT_RUN,
    MACROS_WILL_NOT_BE_CARRIED,
    CANNOT_EMPTY,
    CANNOT_UNRECOGNISED,
    CANNOT_DAMAGED,
    CANNOT_A_PROGRAM,
    CANNOT_PASSWORD_PROTECTED,
    CANNOT_NOTHING_HERE_OPENS_IT,
    NAMED_AS_SOMETHING_ELSE,
    NAMED_AS_WHAT_IT_IS_NOT,
    UNREADABLE,
    WOULD_A_COMPLETE_COPY,
    WOULD_A_COPY_WITHOUT_THE_PASSWORD,
    WOULD_THE_DOCUMENT_ITSELF,
    WOULD_ANOTHER_MACHINE_OR_FORMAT,
    WOULD_WHOEVER_MADE_IT,
];

/// The gap holding what a file is.
pub const WHAT: &str = "what";

/// The gap holding what a copy is converted into.
pub const INTO: &str = "into";

/// The gap holding what a file's name says it is.
pub const NAMED: &str = "named";

/// Why this crate's own words could not be declared.
///
/// Neither can happen to the list above — the tests at the bottom of this file
/// are what say so. It is a `Result` rather than an unwrap because a library
/// that panics on its own string table takes the shell with it, and because
/// [`declare_into`] can genuinely fail against a vocabulary that already holds
/// one of these keys.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WordsError {
    /// A word that is not a phrase: a sentence that is not one, or a note that
    /// could not be attached.
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
pub fn opening_words() -> Result<Vocabulary, WordsError> {
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

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    /// **What we ship is held to the rule everybody else is held to.**
    /// `Word::key` does not check, because a key written in this file cannot
    /// arrive from anywhere; this is the test that makes that true.
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

    /// A key names one string. Two words sharing one would mean whichever was
    /// declared second is a string nobody can reach.
    #[test]
    fn no_two_words_are_named_the_same() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// Every one of them is in the area a reader can sort by.
    #[test]
    fn everything_this_crate_says_says_it_is_this_crate() {
        for word in EVERY_WORD {
            assert_eq!(word.key().area(), "opening", "{}", word.named());
        }
    }

    /// The three lists are the whole list, so a word added to one of them
    /// without a test being told about it fails here.
    #[test]
    fn the_three_lists_are_the_whole_list() {
        let mut all: Vec<&str> = THE_NAMES.iter().map(Word::named).collect();
        all.extend(THE_SENTENCES.iter().map(Word::named));
        all.extend(THE_REMEDIES.iter().map(Word::named));
        let every: Vec<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(all, every);
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it.
    #[test]
    fn the_whole_list_declares() {
        assert_eq!(opening_words().unwrap().how_many(), EVERY_WORD.len());
    }

    /// A vocabulary that already holds one of these keys keeps its own, and
    /// nothing is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = opening_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **Every gap is one this crate fills, and in the sentences that need
    /// it.** A gap a caller could put anything into is a gap somebody fills
    /// with a file's name or a path, and the sentence then quotes what a
    /// person was sent into a notification.
    #[test]
    fn every_gap_is_one_this_crate_fills() {
        let expected: [(Word, &[&str]); 13] = [
            (OPENS_AS_IT_IS, &[WHAT]),
            (CONVERTS_A_COPY, &[WHAT, INTO]),
            (MACROS_WILL_NOT_RUN, &[]),
            (MACROS_WILL_NOT_BE_CARRIED, &[]),
            (CANNOT_EMPTY, &[]),
            (CANNOT_UNRECOGNISED, &[]),
            (CANNOT_DAMAGED, &[WHAT]),
            (CANNOT_A_PROGRAM, &[]),
            (CANNOT_PASSWORD_PROTECTED, &[]),
            (CANNOT_NOTHING_HERE_OPENS_IT, &[WHAT]),
            (NAMED_AS_SOMETHING_ELSE, &[NAMED, WHAT]),
            (NAMED_AS_WHAT_IT_IS_NOT, &[NAMED]),
            (UNREADABLE, &[]),
        ];
        assert_eq!(expected.len(), THE_SENTENCES.len());
        for (word, gaps) in expected {
            let found = word.phrase().unwrap().source().gaps().to_vec();
            assert_eq!(found, gaps, "{}", word.named());
        }
        for word in THE_NAMES.iter().chain(THE_REMEDIES.iter()) {
            assert!(
                word.phrase().unwrap().source().gaps().is_empty(),
                "{} has a gap in it",
                word.named()
            );
        }
    }

    /// **Every word carries a note.** A name is read inside somebody else's
    /// sentence and a sentence is read at a moment a translator has to be able
    /// to picture; neither can be translated from its own words.
    #[test]
    fn every_word_carries_a_note_for_the_translator() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
        for word in THE_NAMES {
            assert!(
                word.note()
                    .is_some_and(|note| note.contains("read inside another sentence")),
                "{} does not tell a translator where it is read",
                word.named()
            );
        }
    }

    /// **The sentences begin a sentence and the names do not.**
    #[test]
    fn the_sentences_begin_a_sentence_and_the_names_do_not() {
        for word in THE_SENTENCES.iter().chain(THE_REMEDIES.iter()) {
            assert!(
                word.says().chars().next().unwrap().is_uppercase(),
                "{} does not begin a sentence",
                word.named()
            );
        }
        for word in THE_NAMES {
            assert!(
                word.says().chars().next().unwrap().is_lowercase(),
                "{} is read inside a sentence and begins one",
                word.named()
            );
        }
    }

    /// **Nothing here hedges.** The set of outcomes has no *probably* in it,
    /// and a sentence that said one would put it back.
    #[test]
    fn nothing_here_hedges() {
        for word in EVERY_WORD {
            let said = word.says().to_lowercase();
            for hedge in ["probably", "may ", "might", "perhaps", "possibly", "likely"] {
                assert!(
                    !said.contains(hedge),
                    "{} says \"{hedge}\", and nothing here is guessed",
                    word.named()
                );
            }
        }
    }

    /// **Nothing a person reads names the machinery.** Not a media type, not
    /// an extension, not a byte, not a code: a person should not have to learn
    /// how a file is recognised to understand what was recognised.
    #[test]
    fn nothing_a_person_reads_names_the_machinery() {
        for word in EVERY_WORD {
            let said = word.says().to_lowercase();
            for machinery in [
                "mime",
                "application/",
                "extension",
                "signature",
                "magic",
                "byte",
                "header",
                "error",
                "code",
                "format:",
                ".docx",
                ".pdf",
                "library",
                "libreoffice",
                "office suite",
                "return",
                "exit status",
                "cups",
                "application/octet",
            ] {
                assert!(
                    !said.contains(machinery),
                    "{} says \"{machinery}\"",
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

    /// **Nothing a person reads offers to send the file anywhere to find
    /// out.** An offer to upload is the sentence this crate exists to replace,
    /// and on this machine it would be an errand nobody asked for.
    #[test]
    fn nothing_offers_to_send_the_file_anywhere() {
        for word in EVERY_WORD {
            let said = word.says().to_lowercase();
            for offer in [
                "upload",
                "online",
                "internet",
                "website",
                "web ",
                "cloud",
                "service",
                "send it to",
                "send this file",
                "look it up",
                "search",
            ] {
                assert!(
                    !said.contains(offer),
                    "{} says \"{offer}\": a file this machine cannot open is never sent \
                     anywhere to find out",
                    word.named()
                );
            }
        }
    }
}
