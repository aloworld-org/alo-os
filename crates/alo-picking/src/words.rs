//! Every string this crate can say, and the English beside each one.
//!
//! Three of them are what a person reads while they are picking — what the
//! picker is for, what picking this folder will do, and the one honest thing
//! to say about a folder with more in it than can be shown. The other seven
//! are refusals: every way a folder cannot be shown, cannot be entered, or
//! cannot be picked.
//!
//! The shape is `alo-overlay`'s and is copied rather than re-decided:
//! constants under one area, `alo_strings::Word` because these are literals in
//! this file, [`declare_into`] for the one vocabulary the machine has, and
//! tests at the bottom holding the list to the rules every other list is held
//! to. Nothing here counts anything, so there is no plural in this file — the
//! test at the bottom is what keeps somebody from writing one by hand.
//!
//! # A note is part of the string
//!
//! A translator works alone, with no alo machine in front of them. *The agent*
//! is the assistant built into alo OS and not a person; *to grant* is for the
//! person to allow the agent to reach one thing; *the picker* is the surface
//! they are looking at while they read these. Where the sentence cannot be
//! translated from its own words, the note says so.
//!
//! # A refusal says what to do next
//!
//! Every one of the seven is read by somebody standing in a folder chooser
//! having just tried something, so each is two clauses: what is so, and then
//! what to do about it. A person told only that something failed has been told
//! they are stuck, and a picker is the one surface where being stuck means the
//! agent can never be granted anything at all.

use alo_strings::Vocabulary;

/// One string a crate can say — `alo-strings`' type, re-exported because this
/// crate's files and the tests that read its list name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// What a person reads while they are picking.
// ---------------------------------------------------------------------------

/// The heading over the picker: what this surface is for.
pub const ASK: Word = Word::saying("picking.ask", "Pick a folder for the agent to reach").noting(
    "The agent is the assistant built into alo OS, not a person. The heading over the folder \
     chooser a person opens to grant the agent a folder. An instruction rather than a title, \
     because it is the whole explanation of what the surface does.",
);

/// What picking the folder in front of the person will do.
pub const COVERS: Word = Word::saying(
    "picking.covers",
    "The agent will be able to reach {folder} and everything inside it, and nothing outside it",
)
.noting(
    "The agent is the assistant built into alo OS, not a person. {folder} is a folder's full path \
     off the person's own disk: it is never translated, and a translation of it would name a \
     different folder. Read immediately before somebody grants a folder, so the last clause is \
     the reassurance and must not be dropped.",
);

/// A folder with more in it than a picker can show at once.
pub const MORE_THAN_SHOWN: Word = Word::saying(
    "picking.more-than-shown",
    "There are more folders here than can be shown. Open one of these to narrow it down, or start \
     somewhere further in",
)
.noting(
    "Shown when a folder holds more folders than the chooser will list at once. It is a fact \
     about the list rather than an error, and the second clause is what a person does about it.",
);

// ---------------------------------------------------------------------------
// The seven refusals.
// ---------------------------------------------------------------------------

/// The picker was asked to start somewhere that is not a full path.
pub const NOT_A_FULL_PATH: Word = Word::saying(
    "picking.not-a-full-path",
    "That is not a place on this machine. Start the picker at a folder named in full, such as \
     your home folder",
)
.noting(
    "The picker is the folder chooser a person picks a folder in. Shown when the chooser is \
     opened at a path that is not written from the top of the disk down, which is alo OS's own \
     wiring being wrong rather than something the person did.",
);

/// The picker was asked to start somewhere that could lead elsewhere.
pub const COULD_LEAD_ELSEWHERE: Word = Word::saying(
    "picking.could-lead-elsewhere",
    "That place is named in a way that could lead somewhere else. Start the picker at the folder \
     itself",
)
.noting(
    "The picker is the folder chooser a person picks a folder in. Shown when the chooser is \
     opened at a path containing a step upwards, which cannot be compared honestly against what \
     is granted and is therefore refused rather than tidied up.",
);

/// The folder is not there any more.
pub const NOTHING_THERE: Word = Word::saying(
    "picking.nothing-there",
    "That folder is not there any more. Go back, and pick another one",
)
.noting(
    "Shown when a folder that was listed a moment ago has been moved or deleted since. Go back \
     means back to the folder above the one that went away.",
);

/// What was opened is not a folder.
pub const NOT_A_FOLDER: Word = Word::saying(
    "picking.not-a-folder",
    "That is not a folder, so there is nothing in it to pick. Pick a folder instead",
)
.noting(
    "Shown when the thing being opened in the chooser turns out to be a file, or a shortcut to \
     somewhere else, rather than a folder. Only a folder can be granted here.",
);

/// This machine would not open the folder.
pub const WOULD_NOT_BE_READ: Word = Word::saying(
    "picking.would-not-be-read",
    "This machine would not open that folder. Pick a folder you can open yourself, or ask \
     whoever looks after this machine",
)
.noting(
    "Shown when the operating system refuses to list a folder for the person who is signed in — \
     usually somebody else's folder, or one the system keeps to itself. The chooser can never \
     show more than the person themselves can open.",
);

/// A name that is not one of the folders being shown.
pub const NOT_SHOWN_HERE: Word = Word::saying(
    "picking.not-shown-here",
    "There is no folder of that name here. Pick one of the folders shown",
)
.noting(
    "Shown when something asks the chooser to open a name that is not in the list in front of the \
     person. Names are matched exactly, so a folder whose name differs only in capital letters is \
     a different folder.",
);

/// There is nothing above where the picker is standing.
pub const NOTHING_ABOVE: Word = Word::saying(
    "picking.nothing-above",
    "This is as far up as the picker goes. Open one of the folders shown, and pick from there",
)
.noting(
    "The picker is the folder chooser a person picks a folder in. Shown when somebody asks to go \
     up from the very top of the disk, where there is nothing above to go to.",
);

/// A pick at the top of the disk, which would be the whole machine.
pub const THE_WHOLE_MACHINE: Word = Word::saying(
    "picking.the-whole-machine",
    "The agent is never granted the whole machine. Open a folder, and pick the folder you \
     actually mean",
)
.noting(
    "The agent is the assistant built into alo OS, not a person. To grant is for the person to \
     allow the agent to reach one thing. Shown when somebody tries to pick the very top of the \
     disk: alo OS has no such grant at all, by design, and the sentence says so as a rule rather \
     than as a failure.",
);

/// Every string this crate can say, in the order a translator meets them.
pub const EVERY_WORD: [Word; 11] = [
    ASK,
    COVERS,
    MORE_THAN_SHOWN,
    NOT_A_FULL_PATH,
    COULD_LEAD_ELSEWHERE,
    NOTHING_THERE,
    NOT_A_FOLDER,
    WOULD_NOT_BE_READ,
    NOT_SHOWN_HERE,
    NOTHING_ABOVE,
    THE_WHOLE_MACHINE,
];

/// The gap [`COVERS`] leaves for the folder a person is about to grant.
///
/// Named here rather than written twice: the sentence above and the code that
/// fills it are one thing, and a typo between them would put the key in front
/// of a person at the moment they are deciding what to grant.
pub const THE_FOLDER: &str = "folder";

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
    Word(#[from] alo_strings::WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] alo_strings::VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn picking_words() -> Result<Vocabulary, WordsError> {
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
    /// [`Word::key`] does not check, because a key written in this file cannot
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

    /// Every one of them is in the area a reader can sort by, which is what
    /// lets one vocabulary hold every crate's strings.
    #[test]
    fn everything_this_crate_says_says_it_is_this_crate() {
        for word in EVERY_WORD {
            assert_eq!(word.key().area(), "picking", "{}", word.named());
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it.
    #[test]
    fn the_whole_list_declares() {
        let vocabulary = picking_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = picking_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **Only the sentence about a particular folder has a gap in it.** A
    /// refusal with a gap could arrive with a hole in it at the exact moment
    /// something has already gone wrong, and a heading with one would be a
    /// heading nobody can fill.
    #[test]
    fn only_the_sentence_that_names_a_folder_needs_anything_filled_in() {
        for word in EVERY_WORD {
            let phrase = word.phrase().unwrap();
            let gaps = phrase.source().gaps();
            if word == COVERS {
                assert_eq!(gaps, vec![THE_FOLDER], "{}", word.named());
            } else {
                assert!(gaps.is_empty(), "{} has a gap in it", word.named());
            }
        }
    }

    /// **Every refusal says what to do next.** Each is two clauses — what is
    /// so, and then the action — so none can shrink into a bare report of a
    /// failure without failing here.
    #[test]
    fn every_refusal_says_what_to_do_next() {
        for word in [
            NOT_A_FULL_PATH,
            COULD_LEAD_ELSEWHERE,
            NOTHING_THERE,
            NOT_A_FOLDER,
            WOULD_NOT_BE_READ,
            NOT_SHOWN_HERE,
            NOTHING_ABOVE,
            THE_WHOLE_MACHINE,
        ] {
            assert!(
                word.says().split_once(". ").is_some(),
                "{} is one clause, so it reports without saying what to do",
                word.named()
            );
        }
    }

    /// **Every word carries a note.** None of these can be translated from its
    /// own words alone: each is shown at a moment the translator has to be
    /// able to picture, and several name things — the agent, a grant, the
    /// picker — that have a meaning here.
    #[test]
    fn every_word_carries_a_note_for_the_translator() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// **Every sentence that names the agent says the agent is not a person.**
    /// A translator with no alo machine in front of them would otherwise have
    /// to guess, and in several languages the guess decides the grammar of the
    /// whole sentence.
    #[test]
    fn every_word_that_names_the_agent_says_what_the_agent_is() {
        for word in EVERY_WORD {
            if !word.says().contains("agent") {
                continue;
            }
            assert!(
                word.note()
                    .is_some_and(|note| note.contains("not a person")),
                "{} does not say what the agent is",
                word.named()
            );
        }
    }

    /// **Nothing here counts anything.** A number written into an English
    /// sentence is a sentence that cannot be translated into a language with
    /// three plural forms, and this crate has no plural to put one in — which
    /// is why the folder that holds more than can be shown says so without
    /// saying how many.
    #[test]
    fn nothing_here_counts_anything() {
        for word in EVERY_WORD {
            assert!(
                !word.says().chars().any(|char| char.is_ascii_digit()),
                "{}",
                word.named()
            );
        }
    }
}
