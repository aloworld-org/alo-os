//! Every string this crate can say, and the English beside each one.
//!
//! Three. Two are the two halves of ADR 0009's sentence that nobody else in
//! this workspace owns: **a machine that cannot reach a model says so once,
//! where it happened, and continues.** The *says so* half already has words —
//! `alo_answering::Failed::said` names what went wrong and where, and
//! `alo_answering::Failed::nothing_was_sent` is the reassurance that goes with
//! it. What had never been written is the line that turns a report into
//! something a person can act on:
//!
//! - [`THE_AGENT_CANNOT_ANSWER`] is the **heading**. Every sentence
//!   `alo-answering` says is about a *question* — *nothing answered on this
//!   machine* — and a person who pressed a key expecting an assistant is not
//!   told by any of them which part of their computer has stopped.
//! - [`CARRY_ON`] is the **and continues**. It is the whole of the anti-nag
//!   promise said out loud: nothing on this machine is waiting for the person
//!   to fix this, so there is nothing to come back to and nothing to be
//!   reminded about.
//!
//! The third is the same promise about a different thing. `docs/features.md`:
//! *a model too large for the memory in this laptop is said so plainly, once —
//! and then run anyway.* `alo-models` says the *plainly*; [`RUNS_THEM_ANYWAY`]
//! is the *once — and then run anyway*, read under it, and it is held to the
//! same rule the two above are held to: it sells nothing, and it leans toward
//! nothing — not a smaller model, not a catalogued one, not a provider.
//!
//! # What is deliberately not here
//!
//! **Nothing that asks anybody to buy anything.** Not a price, not a provider,
//! not a link, not a *top up to continue*. ADR 0009 rejects the greyed-out
//! panel as an advertisement wearing a disabled state, and a line offering to
//! sell somebody credit at the moment their balance emptied is the same
//! advertisement wearing a helpful tone. The test at the bottom of this file is
//! what keeps the next line added here to that rule, and it is written as a
//! refusal because a rule nobody can fail is a rule nobody keeps.
//!
//! **Nothing about a particular reason.** These two lines are said about all
//! eight of `alo_answering::WentWrong`'s reasons without changing, because the
//! reason is already said by the line between them. A pair of strings that
//! varied with the reason would be this crate wording a failure a second time,
//! which is the mistake `alo-turn`'s vocabulary is written to avoid.
//!
//! # A note is part of the string
//!
//! A translator works alone, with no alo machine in front of them. *The agent*
//! is the assistant built into alo OS and is a piece of software rather than a
//! person; *right now* is about this instant rather than about the day. Where
//! the sentence cannot be translated from its own words, the note says so.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported because this
/// crate's files and the tests that read its list name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// What a person is told, in the order they read it — [`crate::ToldOnce`].
// ---------------------------------------------------------------------------

/// The heading: which part of this machine has stopped.
pub const THE_AGENT_CANNOT_ANSWER: Word = Word::saying(
    "telling.the-agent-cannot-answer",
    "The agent cannot answer right now",
)
.noting(
    "The heading of what a person is shown when a question could not be answered, so it is a \
     capitalised sentence standing on its own above the lines that say what went wrong. The agent \
     is the assistant built into alo OS: it is a piece of software and not a person, and a \
     language that inflects for that should treat it as a thing. \"Right now\" is about this \
     moment rather than about the day, and it is the point of the sentence: nothing here is \
     broken for good.",
);

/// The last line: what this means for the rest of the machine, which is nothing.
pub const CARRY_ON: Word = Word::saying(
    "telling.carry-on",
    "You can carry on. Nothing else on this machine depends on the agent, and nothing here is \
     waiting for you to do anything about this",
)
.noting(
    "The last line of what a person is shown when a question could not be answered, read directly \
     under the line that says what went wrong. The agent is the assistant built into alo OS: it \
     is a piece of software and not a person. The second half is a promise and should survive \
     translation whole — the machine will not raise this again by itself, so the person is not \
     being asked to come back to it. \"Carry on\" means continue with whatever you were doing.",
);

/// The last line of a warning about size: it is said once, and the weights run.
pub const RUNS_THEM_ANYWAY: Word = Word::saying(
    "telling.runs-them-anyway",
    "That is said once: alo OS will run these weights whenever you choose them, and will not \
     raise their size again by itself",
)
.noting(
    "The last line of what a person is shown when weights they chose are larger than this \
     machine's memory, read directly under the line that says so. \"alo OS\" is the product's \
     name and is never translated. Both halves are promises and must survive whole: the weights \
     run — this is not a refusal and not advice to pick something smaller — and the machine will \
     not bring their size up again on its own. \"By itself\" is the point: if the person asks, \
     they are answered.",
);

/// Every string this crate can say, in the order a person reads them.
///
/// There is no countable one, and there is unlikely ever to be: a telling is
/// about one unavailability, and a machine that counted how many times it had
/// not been able to reach a model would be keeping a tally in order to show
/// somebody a number, which is a nag with arithmetic in it.
pub const EVERY_WORD: [Word; 3] = [THE_AGENT_CANNOT_ANSWER, CARRY_ON, RUNS_THEM_ANYWAY];

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
pub fn telling_words() -> Result<Vocabulary, WordsError> {
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

    /// Every one of them is in the area a reader can sort by, which is what
    /// lets one vocabulary hold every crate's strings.
    #[test]
    fn everything_this_crate_says_says_it_is_this_crate() {
        for word in EVERY_WORD {
            assert_eq!(word.key().area(), "telling", "{}", word.named());
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it.
    #[test]
    fn the_whole_list_declares() {
        assert_eq!(telling_words().unwrap().how_many(), EVERY_WORD.len());
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = telling_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **Nothing this crate says asks anybody to buy anything.** ADR 0009
    /// rejected the greyed-out panel as an advertisement wearing a disabled
    /// state, and a line offering to sell somebody credit at the moment their
    /// balance emptied is the same advertisement wearing a helpful tone. The
    /// notes are searched as well as the sentences, because a note is what a
    /// translator writes the sentence from.
    #[test]
    fn nothing_here_asks_anybody_to_buy_anything() {
        for word in EVERY_WORD {
            let read =
                format!("{} {}", word.says(), word.note().unwrap_or_default()).to_ascii_lowercase();
            for selling in [
                "buy", "credit", "top up", "top-up", "upgrade", "subscri", "purchase", "payment",
                "pay ", "billing", "plan", "price", "trial",
            ] {
                assert!(
                    !read.contains(selling),
                    "{} says \"{selling}\", which is this crate selling something",
                    word.named()
                );
            }
        }
    }

    /// **Every sentence that names the agent says the agent is not a person.**
    /// The two about a question name it, and in a language where the answer
    /// decides the grammar of the whole sentence a translator cannot guess.
    /// The one about size does not name it, because it is not about the
    /// agent: the weights answer questions whether or not they ever get a
    /// turn.
    #[test]
    fn every_word_that_names_the_agent_says_what_the_agent_is() {
        let mut naming_it = 0;
        for word in EVERY_WORD {
            if !word.says().contains("agent") {
                continue;
            }
            naming_it += 1;
            assert!(
                word.note()
                    .is_some_and(|note| note.contains("not a person")),
                "{} does not say what the agent is",
                word.named()
            );
        }
        assert_eq!(naming_it, 2);
    }

    /// **Nothing said about size leans anywhere.** The line under a warning is
    /// held to `alo-models`' own list of nudges, because it is read beside that
    /// crate's sentence about somebody's own weights and the two must not
    /// disagree about whose decision that was.
    #[test]
    fn the_line_about_size_nudges_toward_nothing() {
        let read = format!(
            "{} {}",
            RUNS_THEM_ANYWAY.says(),
            RUNS_THEM_ANYWAY.note().unwrap_or_default()
        )
        .to_ascii_lowercase();
        for nudge in alo_models::words::NUDGES {
            assert!(
                !read.contains(nudge),
                "the line about size says \"{nudge}\""
            );
        }
        assert!(!read.contains("catalogue"));
        assert!(read.contains("will run these weights"));
    }

    /// **Both are whole sentences with nothing to fill in.** A gap in a line
    /// read at the moment something already went wrong would put `{}` in front
    /// of a person at the worst possible moment, and there is nothing to fill
    /// one from: what varies between two tellings is the line between these
    /// two, which `alo-answering` words.
    #[test]
    fn neither_line_has_a_gap_in_it() {
        for word in EVERY_WORD {
            assert!(
                word.phrase().unwrap().source().gaps().is_empty(),
                "{}",
                word.named()
            );
        }
    }

    /// **Nothing here counts anything.** A number written into an English
    /// sentence is a sentence that cannot be translated into a language with
    /// three plural forms — and a telling that counted would be counting how
    /// often the machine had failed, which is a tally kept in order to show
    /// somebody a number.
    #[test]
    fn nothing_here_counts_anything() {
        for word in EVERY_WORD {
            assert!(
                !word.says().chars().any(|letter| letter.is_ascii_digit()),
                "{}",
                word.named()
            );
        }
    }

    /// **Every word carries a note.** Neither can be translated from its own
    /// words alone: each is shown at a moment the translator has to be able to
    /// picture, and each names the agent, which has a meaning here.
    #[test]
    fn every_word_carries_a_note_for_the_translator() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// **Each of them stands on its own.** The heading is announced by a screen
    /// reader with nothing above it and the last line is read after a sentence
    /// somebody else worded, so both are capitalised sentences rather than
    /// clauses in a list.
    #[test]
    fn each_line_begins_a_sentence() {
        for word in EVERY_WORD {
            let first = word.says().chars().next().unwrap();
            assert!(
                first.is_uppercase(),
                "{} does not begin a sentence",
                word.named()
            );
        }
    }
}
