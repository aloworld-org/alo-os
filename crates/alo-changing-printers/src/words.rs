//! Every string this crate can say, and the English beside each one.
//!
//! Three groups: the three verbs an agent proposes a change to the printers
//! through; what became of a change; and why one was not made. Every refusal
//! says **what did not happen and what a person does about it**.
//!
//! # Nothing a person reads names the machinery
//!
//! Not the printing service, not the part of the machine that makes changes
//! with authority over the whole of it, not a queue, an address, a socket, a
//! key or *root*. A person sets up a printer, removes one, or chooses the one
//! their machine prints on; `docs/features.md` says so in those words, and a
//! test at the bottom of this file reads every sentence for the words that
//! would undo it. A printer's name is data, filled into `{printer}` exactly as
//! the printer gave it, and never written into a sentence.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported so this
/// crate's files name it as `crate::words::Word`.
pub use alo_strings::Word;

/// The gap holding a printer's name, and the argument that names one.
pub const PRINTER: &str = "printer";

// ---------------------------------------------------------------------------
// The three verbs.
// ---------------------------------------------------------------------------

/// What proposing a printer be set up is for.
pub const ADD_PURPOSE: Word = Word::saying(
    "changing-printers.verb.add-printer.purpose",
    "propose setting up a printer this machine found, so it can print on it",
)
.noting(
    "What an assistant's ability to propose setting up a printer is for, shown in a list of what \
     it can do. It only ever proposes: nothing is set up until the person approves.",
);

/// What proposing a printer be removed is for.
pub const REMOVE_PURPOSE: Word = Word::saying(
    "changing-printers.verb.remove-printer.purpose",
    "propose removing a printer that is set up on this machine",
)
.noting(
    "What an assistant's ability to propose removing a printer is for, shown in a list of what it \
     can do. Nothing is removed until the person approves.",
);

/// What proposing the printer this machine prints on is for.
pub const DEFAULT_PURPOSE: Word = Word::saying(
    "changing-printers.verb.set-default-printer.purpose",
    "propose which of the printers set up on this machine it prints on",
)
.noting(
    "What an assistant's ability to propose which printer this machine prints on is for, shown in \
     a list of what it can do. Nothing changes until the person approves.",
);

/// The argument naming the printer, for all three verbs.
pub const THE_PRINTER: Word = Word::saying(
    "changing-printers.verb.printer",
    "the printer, by the name it gave itself",
)
.noting(
    "Describes one part of a proposal: which printer. A printer's name is the one it announced, \
     usually its maker and model, like Brother HL-L2350DW series.",
);

/// The sentence a person approves to set a printer up.
pub const ADD_SENTENCE: Word = Word::saying(
    "changing-printers.verb.add-printer.sentence",
    "set up the printer {printer}, so this machine can print on it",
)
.noting(
    "The sentence a person approves before an assistant's proposal to set up a printer is carried \
     out. {printer} is the name the printer gave itself and is not translated. Approving it sets \
     that one printer up once, and makes it the one this machine prints on.",
);

/// The sentence a person approves to remove a printer.
pub const REMOVE_SENTENCE: Word = Word::saying(
    "changing-printers.verb.remove-printer.sentence",
    "remove the printer {printer} from this machine",
)
.noting(
    "The sentence a person approves before an assistant's proposal to remove a printer is carried \
     out. {printer} is the printer's name and is not translated. Approving it removes that one \
     printer, once; nothing can be printed on it afterwards until it is set up again.",
);

/// The sentence a person approves to choose the printer this machine prints on.
pub const DEFAULT_SENTENCE: Word = Word::saying(
    "changing-printers.verb.set-default-printer.sentence",
    "print on {printer} from now on",
)
.noting(
    "The sentence a person approves before an assistant's proposal to change which printer this \
     machine prints on is carried out. {printer} is the printer's name and is not translated.",
);

// ---------------------------------------------------------------------------
// What became of a change.
// ---------------------------------------------------------------------------

/// A printer was removed.
pub const REMOVED: Word = Word::saying(
    "changing-printers.removed",
    "{printer} is removed from this machine. Nothing more can be printed on it until it is set \
     up again",
)
.noting("Said once a printer the person chose to remove is gone. {printer} is its name.");

/// A printer is now the one this machine prints on.
pub const MADE_DEFAULT: Word = Word::saying(
    "changing-printers.made-default",
    "This machine prints on {printer} from now on",
)
.noting("Said once the person's choice of printer has been made. {printer} is the printer's name.");

// ---------------------------------------------------------------------------
// Why a change was not made.
// ---------------------------------------------------------------------------

/// No printer found by that name.
pub const NONE_FOUND_CALLED: Word = Word::saying(
    "changing-printers.refused.none-found-called",
    "No printer called {printer} was found, so nothing was changed. Check that it is switched on \
     and connected, then choose it from the printers in Settings",
)
.noting(
    "Said when a person approved setting up a printer by its name and no printer this machine can \
     find now has that name. {printer} is the name that was approved.",
);

/// No printer set up by that name.
pub const NONE_SET_UP_CALLED: Word = Word::saying(
    "changing-printers.refused.none-set-up-called",
    "No printer called {printer} is set up on this machine, so nothing was changed. The printers \
     in Settings show the ones that are",
)
.noting(
    "Said when a person approved removing or choosing a printer by its name and no printer set up \
     on this machine has that name. {printer} is the name that was approved.",
);

/// More than one printer has that name.
pub const MORE_THAN_ONE_CALLED: Word = Word::saying(
    "changing-printers.refused.more-than-one-called",
    "More than one printer is called {printer}, so nothing was changed. Choose the one you mean \
     from the printers in Settings",
)
.noting(
    "Said when a person approved a change to a printer by its name and two printers have that \
     name, which happens in an office with two of the same model. {printer} is the name.",
);

/// The printing service could not be asked.
pub const PRINTING_NOT_ANSWERING: Word = Word::saying(
    "changing-printers.refused.printing-not-answering",
    "This machine's printing is not answering, so nothing was changed. Restart the machine, then \
     try again",
)
.noting(
    "Said when the part of this machine that prints could not be asked which printers there are.",
);

/// The approval was not accepted.
pub const APPROVAL_NOT_ACCEPTED: Word = Word::saying(
    "changing-printers.refused.approval-not-accepted",
    "This change was not made, because its approval was not accepted: it was used already, came \
     too late, or was not given for this change. Nothing was changed. Ask for it again, and \
     approve it when you are asked",
)
.noting(
    "Said when a person's approval of a change to the printers reached the part of the machine \
     that makes such changes and was refused there. An approval works once, for one change, and \
     only for about a minute after it is given.",
);

/// The change was accepted and could not be finished.
pub const COULD_NOT_FINISH: Word = Word::saying(
    "changing-printers.refused.could-not-finish",
    "This machine could not finish the change to {printer}. Open the printers in Settings to see \
     how they are now",
)
.noting(
    "Said when an approved change to a printer was started and the machine could not complete it, \
     for example because the printer went away. {printer} is the printer's name.",
);

/// Nothing is being written down, so nothing is changed.
pub const NOT_BEING_KEPT: Word = Word::saying(
    "changing-printers.refused.not-being-kept",
    "This machine has stopped keeping its record of changes to its settings, so it makes none. \
     Nothing was changed. Restart the machine",
)
.noting(
    "Said when the machine could not write down a change before making it. It makes no change it \
     cannot write down, so it makes none until it is restarted.",
);

/// Nothing is there to make the change.
pub const NOTHING_MAKES_CHANGES: Word = Word::saying(
    "changing-printers.refused.nothing-makes-changes",
    "The part of this machine that changes its settings is not running, so nothing was changed. \
     Restart the machine",
)
.noting(
    "Said when a change a person approved could not be handed to the part of the machine that \
     makes changes to its settings for everybody who uses it.",
);

/// The printer is described by a name the verb's argument cannot hold.
pub const NOT_A_PRINTER_CHANGE: Word = Word::saying(
    "changing-printers.refused.not-a-printer-change",
    "This was not a change to the printers, so nothing was changed",
)
.noting(
    "Said when something other than an approved change to the printers was handed to the part of \
     the machine that changes them. A person should never see it; if they do, the machine is \
     wrong rather than them.",
);

/// Every word that names something inside another sentence.
pub const THE_NAMES: [Word; 4] = [ADD_PURPOSE, REMOVE_PURPOSE, DEFAULT_PURPOSE, THE_PRINTER];

/// Every word that is a sentence of its own.
pub const THE_SENTENCES: [Word; 13] = [
    ADD_SENTENCE,
    REMOVE_SENTENCE,
    DEFAULT_SENTENCE,
    REMOVED,
    MADE_DEFAULT,
    NONE_FOUND_CALLED,
    NONE_SET_UP_CALLED,
    MORE_THAN_ONE_CALLED,
    PRINTING_NOT_ANSWERING,
    APPROVAL_NOT_ACCEPTED,
    COULD_NOT_FINISH,
    NOT_BEING_KEPT,
    NOTHING_MAKES_CHANGES,
];

/// Every word, the names first.
pub const EVERY_WORD: [Word; 18] = [
    ADD_PURPOSE,
    REMOVE_PURPOSE,
    DEFAULT_PURPOSE,
    THE_PRINTER,
    ADD_SENTENCE,
    REMOVE_SENTENCE,
    DEFAULT_SENTENCE,
    REMOVED,
    MADE_DEFAULT,
    NONE_FOUND_CALLED,
    NONE_SET_UP_CALLED,
    MORE_THAN_ONE_CALLED,
    PRINTING_NOT_ANSWERING,
    APPROVAL_NOT_ACCEPTED,
    COULD_NOT_FINISH,
    NOT_BEING_KEPT,
    NOTHING_MAKES_CHANGES,
    NOT_A_PRINTER_CHANGE,
];

/// Why this crate's words could not be declared.
#[derive(Debug, thiserror::Error)]
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
pub fn changing_printers_words() -> Result<Vocabulary, WordsError> {
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

    /// Every key is one of this crate's, and names one string.
    #[test]
    fn every_key_is_one_of_this_crates_and_names_one_string() {
        for word in EVERY_WORD {
            assert_eq!(alo_strings::Key::named(word.named()), Ok(word.key()));
            assert_eq!(word.key().area(), "changing-printers", "{}", word.named());
        }
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// The two groups and the one refusal nobody should see are the whole list.
    #[test]
    fn the_groups_are_the_whole_list() {
        let mut all: Vec<&str> = THE_NAMES.iter().map(Word::named).collect();
        all.extend(THE_SENTENCES.iter().map(Word::named));
        all.push(NOT_A_PRINTER_CHANGE.named());
        let every: Vec<&str> = EVERY_WORD.iter().map(Word::named).collect();
        assert_eq!(all, every);
    }

    /// The list declares, and a key already taken is not replaced.
    #[test]
    fn the_list_declares_and_nothing_is_replaced() {
        let mut vocabulary = changing_printers_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert!(matches!(
            declare_into(&mut vocabulary),
            Err(WordsError::List(_))
        ));
    }

    /// **Every word carries a note, and every gap is the printer's name.**
    #[test]
    fn every_word_carries_a_note_and_only_the_printers_name_fills_a_gap() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
            for gap in word.phrase().unwrap().source().gaps() {
                assert_eq!(gap, PRINTER, "{} has a gap called {gap}", word.named());
            }
        }
    }

    /// **Nothing here names the machinery, and nothing hedges.** No printing
    /// service, no broker, no socket, no key, no root, no queue, no address.
    #[test]
    fn nothing_here_names_the_machinery_or_hedges() {
        for word in EVERY_WORD {
            let said = word.says().to_lowercase();
            for forbidden in [
                "cups",
                "ipp",
                "driver",
                "queue",
                "broker",
                "socket",
                "root",
                "token",
                "key",
                "digest",
                "identity",
                "uri",
                "address",
                "daemon",
                "service",
                "luks",
                "tpm",
                "networkmanager",
                "probably",
                "might",
                "perhaps",
                "possibly",
            ] {
                assert!(
                    !said.contains(forbidden),
                    "{} says \"{forbidden}\"",
                    word.named()
                );
            }
        }
    }

    /// **Every refusal says nothing was changed or where to look, and what to
    /// do about it** — a second clause after what happened.
    #[test]
    fn every_refusal_says_what_to_do_about_it() {
        for word in [
            NONE_FOUND_CALLED,
            NONE_SET_UP_CALLED,
            MORE_THAN_ONE_CALLED,
            PRINTING_NOT_ANSWERING,
            APPROVAL_NOT_ACCEPTED,
            COULD_NOT_FINISH,
            NOT_BEING_KEPT,
            NOTHING_MAKES_CHANGES,
        ] {
            let said = word.says();
            assert!(
                said.contains(". ") || said.contains(", then "),
                "{} names what happened and not what to do",
                word.named()
            );
        }
    }
}
