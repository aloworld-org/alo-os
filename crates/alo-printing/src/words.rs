//! Every string this crate can say, and the English beside each one.
//!
//! Five groups: what a printer is called when it has not said, where a found
//! printer is, setting one up, what is wrong when one stops, and printing a
//! document. Every sentence says **what happened and what a person does about
//! it**, because a printer that stopped is the moment this whole crate exists
//! for and a sentence that only names the trouble has done half the job.
//!
//! # Nothing a person reads names the machinery
//!
//! Not the printing service's name, not a queue, not a driver's, not a protocol,
//! not a status code. A test at the bottom of this file reads every sentence for
//! them, and for a hedge: what is wrong has been decided by the time any of
//! these is said.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported so this
/// crate's files name it as `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// A printer, and where it is.
// ---------------------------------------------------------------------------

/// A printer that gave no name this machine can show.
pub const A_PRINTER_WITHOUT_A_NAME: Word = Word::saying(
    "printing.a-printer-without-a-name",
    "a printer that has not given a name this machine can show",
)
.noting(
    "The name of a printer, read inside another sentence: \"Set up {printer}, so this machine can \
     print on it\". Used when a printer announced itself with no name, or with one that has \
     characters in it that cannot be shown. Lowercase, with its article, because it sits inside a \
     sentence.",
);

/// A printer found on the local network.
pub const FOUND_ON_THE_NETWORK: Word = Word::saying(
    "printing.found.on-the-network",
    "{printer}, on your network",
)
.noting(
    "One line in the list of printers this machine found. {printer} is the name the printer \
         gave itself — usually its maker and model, which is not translated. \"Your network\" is \
         the network this machine is connected to, at home or at work.",
);

/// A printer found connected to this machine.
pub const FOUND_ON_THIS_MACHINE: Word = Word::saying(
    "printing.found.on-this-machine",
    "{printer}, connected to this machine",
)
.noting(
    "One line in the list of printers this machine found. {printer} is the name the printer gave \
     itself. Connected means by a cable, directly to this computer.",
);

// ---------------------------------------------------------------------------
// Setting one up.
// ---------------------------------------------------------------------------

/// What a person is asked before a printer is set up.
pub const SET_UP_PROPOSED: Word = Word::saying(
    "printing.set-up.proposed",
    "Set up {printer}, so this machine can print on it",
)
.noting(
    "What a person chooses before a printer they found is added to this machine. Nothing is added \
     until they choose it: a printer is a place documents can be sent, and this machine never adds \
     one by itself. {printer} is the name the printer gave itself.",
);

/// A printer has been set up.
pub const SET_UP_DONE: Word = Word::saying(
    "printing.set-up.done",
    "{printer} is set up, and this machine prints on it",
)
.noting("Said once a printer the person chose has been added. {printer} is its name.");

/// A printer that needs its maker's program.
pub const CANNOT_SET_UP_NEEDS_ITS_MAKERS_PROGRAM: Word = Word::saying(
    "printing.set-up.needs-its-makers-program",
    "{printer} can only be set up with a program from the company that made it, and this machine \
     does not install programs like that, so it has not been set up. A printer that prints without \
     one, as most printers sold in recent years do, sets up here by itself",
)
.noting(
    "Said when a person chose a printer that only works with software its manufacturer supplies. \
     {printer} is the name the printer gave itself, which is not translated. This machine never \
     installs that software, on purpose. The last sentence tells the person what kind of printer \
     does work, so they are not left with only a refusal.",
);

/// A printer that did not answer while it was being set up.
pub const CANNOT_SET_UP_DID_NOT_ANSWER: Word = Word::saying(
    "printing.set-up.did-not-answer",
    "{printer} did not answer when this machine asked how to print on it, so it has not been set \
     up. Check that it is switched on and connected, then set it up again",
)
.noting(
    "Said when setting up a printer failed because the printer itself did not reply. {printer} is \
     the name the printer gave itself, which is not translated. A printer describes itself when it \
     is added, and this one did not. The advice is the thing a person can do at the printer.",
);

/// Setting up was not allowed for this account.
pub const CANNOT_SET_UP_NOT_PERMITTED: Word = Word::saying(
    "printing.set-up.not-permitted",
    "This machine does not allow printers to be added from this account, so {printer} has not \
     been set up. Someone who looks after this machine can set it up",
)
.noting(
    "Said when the person's account is not one allowed to add printers — usually on a machine an \
     organisation manages. {printer} is the name of the printer they chose, which is not \
     translated. It is a statement about how the machine is configured, not a fault.",
);

/// The printing service did not answer while a printer was being set up.
pub const CANNOT_SET_UP_SERVICE_NOT_ANSWERING: Word = Word::saying(
    "printing.set-up.service-not-answering",
    "The part of this machine that prints did not respond, so {printer} has not been set up. \
     Restarting this machine starts it again",
)
.noting(
    "Said when the machine's own printing system did not reply while a printer was being added. \
     {printer} is the name of the printer being added, which is not translated. \"The part of \
     this machine that prints\" is deliberate: the person never needs to know what it is called.",
);

/// No printer has been set up.
pub const NO_PRINTER_YET: Word = Word::saying(
    "printing.no-printer-yet",
    "No printer is set up on this machine yet. Open Printers in Settings to find one",
)
.noting(
    "Said when a person prints and no printer has ever been chosen. Printers and Settings are the \
     names of the places in this machine's own interface, and should be the same words used \
     there.",
);

// ---------------------------------------------------------------------------
// How a printer is — `crate::Condition` and `crate::Stopped`.
// ---------------------------------------------------------------------------

/// The printer is ready.
pub const READY: Word = Word::saying("printing.ready", "The printer is ready")
    .noting("Said when a person asks how the printer is and nothing is wrong with it.");

/// Out of paper.
pub const OUT_OF_PAPER: Word = Word::saying(
    "printing.stopped.out-of-paper",
    "The printer is out of paper. Put paper in its tray, and documents it has already taken print \
     once it has some",
)
.noting(
    "Said when the printer stopped because it has no paper. The second sentence is what the person \
     does, and what happens after: documents the printer accepted before it stopped come out by \
     themselves. It does not promise that about a document the printer did not accept — another \
     sentence says that one needs printing again.",
);

/// Jammed.
pub const JAMMED: Word = Word::saying(
    "printing.stopped.jammed",
    "Paper is stuck in the printer. Open it where its own screen or lights point, gently take the \
     stuck paper out, and close it again",
)
.noting(
    "Said when paper has jammed inside the printer. Printers show where the jam is in different \
     ways, which is why the sentence points at the printer's own screen or lights rather than \
     naming a part.",
);

/// Out of ink or toner.
pub const OUT_OF_INK: Word = Word::saying(
    "printing.stopped.out-of-ink",
    "The printer has run out of ink or toner. Put in a new cartridge, and documents it has already \
     taken print once it has one",
)
.noting(
    "Said when the printer stopped because a cartridge is empty. Ink and toner are both named \
     because the person knows which their printer uses and the machine is not always told. As with \
     paper, only documents the printer accepted before it stopped come out by themselves.",
);

/// The printer refused the document.
pub const REFUSED_THE_JOB: Word = Word::saying(
    "printing.stopped.refused-the-job",
    "The printer refused this document, so nothing was printed. Sending it again will not change \
     that: check in Printers, in Settings, that the printer is taking documents, and if it is, \
     print a PDF copy of the document instead",
)
.noting(
    "Said when a printer was reached and would not print this particular document — it does not \
     take this kind of file, or it has been set not to take any. \"Sending it again will not \
     change that\" matters: it stops the person pressing print five more times.",
);

/// Not answering: this machine's printing service did not respond.
pub const NOT_ANSWERING_THE_SERVICE: Word = Word::saying(
    "printing.stopped.not-answering.the-service",
    "The printer is not answering: the part of this machine that prints did not respond when it \
     was asked. Restarting this machine starts it again",
)
.noting(
    "Said when the machine could not find out anything about the printer because its own printing \
     system did not reply. It says what was tried — asking that part of the machine — and what \
     the person can do.",
);

/// Not answering: the printer said nothing this machine can act on.
pub const NOT_ANSWERING_THE_PRINTER: Word = Word::saying(
    "printing.stopped.not-answering.the-printer",
    "The printer is not answering: this machine asked how it is, and it said nothing this machine \
     can act on. Check that it is switched on and connected, by its cable or to the same network \
     as this machine, and look at its own screen or lights",
)
.noting(
    "Said when the printer stopped for a reason that is not out of paper, jammed, out of ink or a \
     refused document — or did not reply at all. It says what the machine tried, asking the \
     printer, and never repeats whatever code the printer sent, because nobody can act on a code.",
);

/// Not answering: the printer is no longer set up.
pub const NOT_ANSWERING_NOT_SET_UP: Word = Word::saying(
    "printing.stopped.not-answering.not-set-up",
    "The printer is not answering: this machine looked for how it was set up and did not find it. \
     Set it up again in Printers, in Settings",
)
.noting(
    "Said when the printer the person uses has been removed from this machine since it was set up. \
     It says what was tried and what to do.",
);

// ---------------------------------------------------------------------------
// Printing a document.
// ---------------------------------------------------------------------------

/// The document was sent to the printer.
pub const PRINTED: Word = Word::saying(
    "printing.printed",
    "The document has been sent to the printer",
)
.noting(
    "Said once a document has been handed to the printer. \"Sent\" rather than \"printed\" on \
         purpose: the paper has not come out yet, and if the printer then stops, another sentence \
         says why.",
);

/// The printer stopped and did not take this document, which will not print by
/// itself.
pub const NOT_TAKEN: Word = Word::saying(
    "printing.not-taken",
    "The printer did not take this document, so it will not print by itself. Once that is put \
     right, print it again",
)
.noting(
    "Said straight after the sentence saying what is wrong with a printer — out of paper, jammed, \
     out of ink, or not answering — when that stopped it accepting the document the person was \
     printing. \"That\" is whatever the sentence before it named. Without this sentence a person \
     would wait for a page that is never coming.",
);

/// The document is not a kind printers take.
pub const NOT_A_KIND_PRINTERS_TAKE: Word = Word::saying(
    "printing.document.not-a-kind-printers-take",
    "This is {what}, and this machine prints PDF documents, pictures and plain text but not this \
     kind of file as it is, so nothing was printed",
)
.noting(
    "Said when a person prints a file this machine recognises but does not print directly. {what} \
     is the name of the kind of file, already translated, from the words that describe files.",
);

/// The document cannot be opened, so it was not printed.
pub const CANNOT_BE_OPENED: Word = Word::saying(
    "printing.document.cannot-be-opened",
    "Nothing was printed, because this file cannot be opened",
)
.noting(
    "Said after the sentence explaining why a file cannot be opened — that it is empty, damaged, a \
     program and so on. This one only adds that printing did not happen.",
);

/// The document is not what its name says.
pub const NOT_WHAT_ITS_NAME_SAYS: Word = Word::saying(
    "printing.document.not-what-its-name-says",
    "Nothing was printed, because this file is not what its name says",
)
.noting(
    "Said after the sentence explaining that a file's name claims it is one kind of file and it is \
     another. Nothing is printed from a file like that, whatever it really is.",
);

/// The document could not be read.
pub const UNREADABLE: Word = Word::saying(
    "printing.document.unreadable",
    "This file could not be read, so nothing was printed",
)
.noting(
    "Said when the file stopped being readable while it was being printed — removed, or on a disk \
     that stopped answering. It says nothing about what the file is.",
);

/// Sending across the network had not been shown.
pub const NOT_SHOWN_LEAVING: Word = Word::saying(
    "printing.not-shown-leaving",
    "Nothing was printed, because sending the document to a printer on the network had not been \
     shown as leaving this machine",
)
.noting(
    "Said when something tried to print on a printer across the network without the machine first \
     showing, in the place that lists everything leaving this machine, that a document was about to \
     leave. On this machine nothing leaves without being shown, and a print is no exception.",
);

/// What was approved was not printing.
pub const NOT_PRINTING: Word = Word::saying(
    "printing.not-printing",
    "Nothing was printed, because what was approved was not printing a document",
)
.noting(
    "Said when an approval for something else reached printing. A person approves one sentence for \
     one action, and an approval never carries over to another.",
);

// ---------------------------------------------------------------------------
// The verb — `crate::verbs`.
// ---------------------------------------------------------------------------

/// What `print_document` is for.
pub const VERB_PURPOSE: Word = Word::saying(
    "printing.verb.print-document.purpose",
    "print a document on this machine's printer",
)
.noting(
    "What an assistant's printing action does, in the list of everything it can do on this machine. \
     Lowercase because it is listed after the action's name.",
);

/// What the document argument is.
pub const VERB_DOCUMENT: Word = Word::saying(
    "printing.verb.print-document.document",
    "the document to print",
)
.noting(
    "What the one thing an assistant names when it prints is: the document. Read in a list of \
         what the action takes.",
);

/// The sentence a person approves.
pub const VERB_SENTENCE: Word = Word::saying(
    "printing.verb.print-document.sentence",
    "print {document} on this machine's printer",
)
.noting(
    "The sentence a person approves before an assistant prints. {document} is where the file is, \
     exactly as it is written on this machine, and is not translated. Nothing is printed until the \
     person approves this sentence, and approving it prints once.",
);

/// Every word that names something, read inside another sentence.
pub const THE_NAMES: [Word; 3] = [A_PRINTER_WITHOUT_A_NAME, VERB_PURPOSE, VERB_DOCUMENT];

/// Every word that is a line or a sentence of its own.
pub const THE_SENTENCES: [Word; 26] = [
    FOUND_ON_THE_NETWORK,
    FOUND_ON_THIS_MACHINE,
    SET_UP_PROPOSED,
    SET_UP_DONE,
    CANNOT_SET_UP_NEEDS_ITS_MAKERS_PROGRAM,
    CANNOT_SET_UP_DID_NOT_ANSWER,
    CANNOT_SET_UP_NOT_PERMITTED,
    CANNOT_SET_UP_SERVICE_NOT_ANSWERING,
    NO_PRINTER_YET,
    READY,
    OUT_OF_PAPER,
    JAMMED,
    OUT_OF_INK,
    REFUSED_THE_JOB,
    NOT_ANSWERING_THE_SERVICE,
    NOT_ANSWERING_THE_PRINTER,
    NOT_ANSWERING_NOT_SET_UP,
    PRINTED,
    NOT_TAKEN,
    NOT_A_KIND_PRINTERS_TAKE,
    CANNOT_BE_OPENED,
    NOT_WHAT_ITS_NAME_SAYS,
    UNREADABLE,
    NOT_SHOWN_LEAVING,
    NOT_PRINTING,
    VERB_SENTENCE,
];

/// Every string this crate can say.
pub const EVERY_WORD: [Word; 29] = [
    A_PRINTER_WITHOUT_A_NAME,
    VERB_PURPOSE,
    VERB_DOCUMENT,
    FOUND_ON_THE_NETWORK,
    FOUND_ON_THIS_MACHINE,
    SET_UP_PROPOSED,
    SET_UP_DONE,
    CANNOT_SET_UP_NEEDS_ITS_MAKERS_PROGRAM,
    CANNOT_SET_UP_DID_NOT_ANSWER,
    CANNOT_SET_UP_NOT_PERMITTED,
    CANNOT_SET_UP_SERVICE_NOT_ANSWERING,
    NO_PRINTER_YET,
    READY,
    OUT_OF_PAPER,
    JAMMED,
    OUT_OF_INK,
    REFUSED_THE_JOB,
    NOT_ANSWERING_THE_SERVICE,
    NOT_ANSWERING_THE_PRINTER,
    NOT_ANSWERING_NOT_SET_UP,
    PRINTED,
    NOT_TAKEN,
    NOT_A_KIND_PRINTERS_TAKE,
    CANNOT_BE_OPENED,
    NOT_WHAT_ITS_NAME_SAYS,
    UNREADABLE,
    NOT_SHOWN_LEAVING,
    NOT_PRINTING,
    VERB_SENTENCE,
];

/// The gap holding a printer's name.
pub const PRINTER: &str = "printer";

/// The gap holding what a file is.
pub const WHAT: &str = "what";

/// Why this crate's own words could not be declared.
///
/// Neither can happen to the list above — the tests at the bottom of this file
/// say so — and [`declare_into`] can genuinely fail against a vocabulary that
/// already holds one of these keys.
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
pub fn printing_words() -> Result<Vocabulary, WordsError> {
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
            assert_eq!(word.key().area(), "printing", "{}", word.named());
        }
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// The two groups are the whole list.
    #[test]
    fn the_two_groups_are_the_whole_list() {
        let mut both: Vec<&str> = THE_NAMES.iter().map(Word::named).collect();
        both.extend(THE_SENTENCES.iter().map(Word::named));
        let every: Vec<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(both, every);
    }

    /// The list declares, and a key already taken is not replaced.
    #[test]
    fn the_list_declares_and_nothing_is_replaced() {
        let mut vocabulary = printing_words().unwrap();
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
                    [PRINTER, WHAT, "document"].contains(&gap.as_str()),
                    "{} has a gap called {gap}",
                    word.named()
                );
            }
        }
    }

    /// **Nothing here hedges and nothing names the machinery.** What is wrong
    /// with a printer has been decided by the time it is said, and a person
    /// never has to learn what anything underneath is called.
    #[test]
    fn nothing_here_hedges_or_names_the_machinery() {
        for word in EVERY_WORD {
            let said = word.says().to_lowercase();
            for forbidden in [
                "probably", "may ", "might", "perhaps", "possibly", "likely", "cups", "ipp",
                "driver", "queue", "protocol", "backend", "status", "code", "error", "usb", "dns",
                "http", "mime",
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

    /// **Every stop says what a person does about it** — a second clause after
    /// what is wrong, never the trouble on its own.
    #[test]
    fn every_stop_says_what_to_do_about_it() {
        for word in [
            OUT_OF_PAPER,
            JAMMED,
            OUT_OF_INK,
            REFUSED_THE_JOB,
            NOT_ANSWERING_THE_SERVICE,
            NOT_ANSWERING_THE_PRINTER,
            NOT_ANSWERING_NOT_SET_UP,
        ] {
            let said = word.says();
            assert!(
                said.contains(". ") || said.contains(": "),
                "{} names the trouble and not what to do",
                word.named()
            );
        }
    }
}
