//! Every string this crate can say, and the English beside each one.
//!
//! Five of them, and they divide in two. Two are **the answers themselves** —
//! the two things a person may do about a change that is waiting — and three
//! are **refusals**: why the question could not be put to anybody, and what a
//! person is told when they answer with nothing in front of them.
//!
//! The shape is `alo-picking`'s and is copied rather than re-decided: constants
//! under one area, `alo_strings::Word` because these are literals in this file,
//! [`declare_into`] for the one vocabulary the machine has, and tests at the
//! bottom holding the list to the rules every other list is held to. Nothing
//! here counts anything, so there is no plural in this file — the test at the
//! bottom is what keeps somebody from writing one by hand.
//!
//! # The sentence itself is not on this list, and that is the point
//!
//! What a person approves is the change, worded by the crate that validated it
//! — `alo_capability::Proposal::sentence`, filled from the arguments the call
//! was really made with. It is not declared here, could not be, and there is no
//! string on this list that describes a change: a surface able to word a
//! proposal would be a machine with two accounts of what it is about to do, and
//! the one a person read would be the one that was not checked.
//!
//! There is no heading over it either. The sentence is the whole of what is
//! being asked, and a title above it would be a second sentence about the same
//! moment — one more thing to translate, and one more thing that can drift away
//! from what the machine will actually do.
//!
//! # A note is part of the string
//!
//! A translator works alone, with no alo machine in front of them. *The agent*
//! is the assistant built into alo OS and not a person; *the desktop* is the
//! graphical session rather than a piece of furniture; *a screen* is a physical
//! display. Where the sentence cannot be translated from its own words, the
//! note says so.

use alo_strings::Vocabulary;

/// One string a crate can say — `alo-strings`' type, re-exported because this
/// crate's files and the tests that read its list name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// The two answers — [`crate::Asked::approve_said`] and
// [`crate::Asked::no_said`].
//
// They are the whole of what a person may do about a proposal, and they are
// short because they are read directly beneath the sentence that says what will
// happen. Everything that describes the change is in that sentence.
// ---------------------------------------------------------------------------

/// The answer that carries the change out: one approval, one execution.
pub const APPROVE: Word = Word::saying("approving.approve", "Approve").noting(
    "The answer that lets one change go ahead, read directly beneath a sentence describing exactly \
     what the agent will do — the agent being the assistant built into alo OS and not a person. It \
     approves that one change and nothing else: it is never a session, a permission or a setting, \
     so a translation that reads as \"always allow\" or \"trust\" would say something alo OS does \
     not offer. Short because the sentence above it carries the whole meaning.",
);

/// The answer that is a whole answer: no, and nobody is asked why.
pub const NO: Word = Word::saying("approving.no", "No").noting(
    "The answer that stops one change, read directly beneath a sentence describing exactly what \
     the agent would do — the agent being the assistant built into alo OS and not a person. It is \
     a complete answer and no reason is asked for or recorded, so a translation should not soften \
     it into \"not now\" or \"cancel\": nothing is postponed and nothing is undone, the change \
     simply does not happen. Short because the sentence above it carries the whole meaning.",
);

// ---------------------------------------------------------------------------
// The three refusals — [`crate::NotAsked`] and [`crate::NotAnswered`].
//
// The first two are read somewhere other than the surface the question would
// have been on, because there is no such surface: a service log while a session
// starts, or a text console. They are still declared and still translated,
// because a person reading a log on their own machine reads their own language.
// ---------------------------------------------------------------------------

/// What [`crate::NotAsked::NoCompositor`] says: nothing is drawing a screen, so
/// there is nowhere to put the question.
pub const NO_COMPOSITOR: Word = Word::saying(
    "approving.no-compositor",
    "The agent has a change waiting for your answer and it cannot be put to you: the desktop is \
     not running. Sign in to the desktop, and ask the agent again",
)
.noting(
    "The agent is the assistant built into alo OS, not a person. The desktop is the graphical \
     session a person signs into. Read when the part of alo OS that draws a screen is not running \
     at all — during start-up, or on a text console — so it is read in a log or on a console \
     rather than on the surface it is about. Nothing has happened to the person's machine: a \
     change nobody could be asked about is a change that does not run.",
);

/// What [`crate::SurfaceRefused::NothingToShowOn`] says: something is drawing,
/// and there is no screen to draw on.
pub const NOTHING_TO_SHOW_ON: Word = Word::saying(
    "approving.nothing-to-show-on",
    "The agent has a change waiting for your answer and it cannot be put to you: no screen is \
     connected. Connect a screen, and ask the agent again",
)
.noting(
    "The agent is the assistant built into alo OS, not a person. A screen here is a physical \
     display. Read when the desktop is running with no display to draw on — before a monitor is \
     plugged in, or after the last one went away. Nothing has happened to the person's machine: a \
     change nobody could be asked about is a change that does not run.",
);

/// What [`crate::NotAnswered::NothingToAnswer`] says: an answer arrived with no
/// question in front of the person.
pub const NOTHING_TO_ANSWER: Word = Word::saying(
    "approving.nothing-to-answer",
    "No change is waiting for your answer here. Ask the agent again if you still want it",
)
.noting(
    "The agent is the assistant built into alo OS, not a person. Read when somebody answers and \
     there is no question in front of them — most often because the same question was already \
     answered a moment ago, which is what stops one approval running a change twice. It reports \
     nothing having gone wrong with the machine.",
);

/// Every string this crate can say, in the order a translator meets them.
pub const EVERY_WORD: [Word; 5] = [
    APPROVE,
    NO,
    NO_COMPOSITOR,
    NOTHING_TO_SHOW_ON,
    NOTHING_TO_ANSWER,
];

/// The two answers a person may give, which are labels rather than sentences.
///
/// Named apart from the refusals so the tests below can hold each half to the
/// rule that is actually its own: a label is short and has no full stop in it,
/// and a refusal says what to do next.
pub const EVERY_ANSWER: [Word; 2] = [APPROVE, NO];

/// Every way this crate refuses, each of which is read having just tried
/// something.
pub const EVERY_REFUSAL: [Word; 3] = [NO_COMPOSITOR, NOTHING_TO_SHOW_ON, NOTHING_TO_ANSWER];

/// Why this crate's own words could not be declared.
///
/// None of these can happen to the list above — the tests at the bottom of this
/// file are what say so. It is a `Result` rather than an unwrap because a
/// library that panics on its own string table takes the shell with it, and
/// because [`declare_into`] can genuinely fail against a vocabulary that
/// already holds one of these keys.
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
pub fn approving_words() -> Result<Vocabulary, WordsError> {
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
            assert_eq!(word.key().area(), "approving", "{}", word.named());
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it.
    #[test]
    fn the_whole_list_declares() {
        assert_eq!(approving_words().unwrap().how_many(), EVERY_WORD.len());
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = approving_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **The two halves of the list are the whole of it.** A word added here
    /// and classified as neither an answer nor a refusal fails this rather than
    /// quietly escaping both sets of rules below.
    #[test]
    fn every_word_is_either_an_answer_or_a_refusal() {
        let named: BTreeSet<&str> = EVERY_ANSWER
            .iter()
            .chain(EVERY_REFUSAL.iter())
            .map(|word| word.named())
            .collect();
        assert_eq!(named.len(), EVERY_WORD.len());
        for word in EVERY_WORD {
            assert!(named.contains(word.named()), "{}", word.named());
        }
    }

    /// **Every refusal is a whole sentence with nothing to fill in** — a gap in
    /// a refusal would put `{}` in front of a person at the exact moment
    /// something already went wrong. The two answers have no gaps either: a
    /// label with a value in it would be a button whose text moves.
    #[test]
    fn nothing_here_needs_anything_filled_in() {
        for word in EVERY_WORD {
            assert!(
                word.phrase().unwrap().source().gaps().is_empty(),
                "{}",
                word.named()
            );
        }
    }

    /// **Every refusal says what to do.** Somebody told only that a change
    /// could not be put to them has been told they are stuck, at the one moment
    /// where being stuck means the agent can never do anything at all. Each is
    /// two clauses — what is so, and then the action.
    #[test]
    fn every_refusal_says_what_to_do() {
        for (word, verb) in [
            (NO_COMPOSITOR, "Sign in"),
            (NOTHING_TO_SHOW_ON, "Connect"),
            (NOTHING_TO_ANSWER, "Ask the agent"),
        ] {
            assert!(
                word.says().contains(verb),
                "{} does not tell anybody to {verb}",
                word.named()
            );
            assert!(
                word.says().split_once(". ").is_some(),
                "{} is one clause, so it reports without instructing",
                word.named()
            );
        }
    }

    /// **The two answers are labels, not sentences.** Each is read directly
    /// beneath the sentence that carries the meaning, so neither ends in a full
    /// stop and neither grows into an explanation of its own — an explanation
    /// here would be a second description of the change, in a vocabulary that
    /// never saw the arguments.
    #[test]
    fn the_two_answers_are_labels_rather_than_sentences() {
        for word in EVERY_ANSWER {
            assert!(!word.says().contains('.'), "{}", word.named());
            assert!(word.says().len() < 32, "{}", word.named());
            assert!(!word.says().trim().is_empty(), "{}", word.named());
        }
        assert_ne!(APPROVE.says(), NO.says());
    }

    /// **Every word carries a note for the translator.** None of these can be
    /// translated from its own words alone: two are single words whose meaning
    /// is entirely in where they are read, and three name something — the
    /// agent, the desktop, a screen — that has a meaning here.
    #[test]
    fn every_word_carries_a_note_for_the_translator() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// **Every sentence that names the agent says the agent is not a person.**
    /// The grammar of the whole sentence depends on the answer in a good many
    /// languages, and a translator who guesses wrong writes alo OS a person
    /// into every one of them.
    #[test]
    fn every_word_that_names_the_agent_says_what_the_agent_is() {
        for word in EVERY_WORD {
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
    /// three plural forms; this crate has nothing to count, and this is what
    /// keeps a plural from being written by hand if it ever does.
    #[test]
    fn nothing_said_here_counts_anything() {
        for word in EVERY_WORD {
            assert!(
                !word.says().chars().any(|char| char.is_ascii_digit()),
                "{}",
                word.named()
            );
        }
    }

    /// **Nothing here describes a change.** The sentence a person approves is
    /// the call's own, and a string on this list that named a verb or a place
    /// would be this crate starting to word proposals — the one thing its
    /// documentation promises it does not do.
    #[test]
    fn nothing_here_describes_a_change() {
        for word in EVERY_WORD {
            for describing in ["move ", "delete ", "rename ", "file ", "folder "] {
                assert!(
                    !word.says().to_lowercase().contains(describing),
                    "{} describes a change",
                    word.named()
                );
            }
        }
    }
}
