//! Every string this crate can say, and the English beside each one.
//!
//! Four, in two groups.
//!
//! **Two are the list itself.** [`NOTHING_GRANTED`] is what a person reads
//! where the list would be when the machine holds nothing — a sentence rather
//! than an empty list, because an empty list beside a settings heading reads as
//! a screen that failed to load. [`ONE_GRANT`] is the clause a row is made of:
//! who may reach what, with the *what* arriving already worded by
//! `alo-capability`, which is the crate that decides what a grant covers.
//!
//! **Two are what revoking comes back as.** [`REVOKED`] says it has already
//! taken effect, because the one fact a person cannot see from the list
//! closing a row is *when* — and the answer, immediately, is the property
//! `alo_capability::Grants::revoke` actually has. [`ALREADY_GONE`] is the
//! refusal: a row from a list that has moved on lands on nothing, and a
//! machine that said nothing about that would leave somebody believing they
//! revoked a grant that was never the one they meant.
//!
//! # Why the row is a clause and the other three are sentences
//!
//! The row is read inside a list, under a heading, beside the times whoever
//! displays it puts there — it is never announced on its own, so it is
//! lowercase like `alo-overlay`'s clauses. The other three are the whole of
//! what a person is told at that moment, so each is a capitalised sentence,
//! for the reason `alo-indicator`'s readings are.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported because this
/// crate's files and the tests that read its list name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// The list itself — [`crate::Listing`] and [`crate::Seen`].
// ---------------------------------------------------------------------------

/// What stands where the list would be, when nothing is granted.
pub const NOTHING_GRANTED: Word = Word::saying(
    "granted.nothing-granted",
    "Nothing is granted right now. No agent can reach any folder, file or application on this \
     machine, and there is nothing here to revoke",
)
.noting(
    "Shown where the list of grants would be, when the machine holds none — the state every \
     machine starts in, and the state after the last grant is revoked or expires. An agent is an \
     AI assistant such as @files, not a person. It deliberately does not tell anybody to go and \
     grant something: the list of grants is where somebody checks and takes away, and an \
     instruction to grant would read as the machine asking for reach. The overlay's \
     overlay.at-rest.nothing-granted is a different sentence for a different place — a status \
     line that does instruct — and the two are not interchangeable.",
);

/// One row of the list: who may reach what.
pub const ONE_GRANT: Word = Word::saying("granted.one-grant", "{agent} can reach {what}").noting(
    "One row in the list of grants, read inside the list under its heading — a lowercase clause, \
     never announced on its own. {agent} is the agent's own name, such as @files: an AI \
     assistant, not a person, and the name is not translated. {what} arrives already translated \
     — for example \"the folder /home/anna/Invoices and everything in it\" — worded by the part \
     of the system that decides what a grant covers. When it was granted and when it expires are \
     shown beside the row by whoever displays it, so this clause must not try to say them.",
);

// ---------------------------------------------------------------------------
// What revoking comes back as — [`crate::Revoked`].
// ---------------------------------------------------------------------------

/// The grant was revoked, and it has already stopped.
pub const REVOKED: Word = Word::saying(
    "granted.revoked",
    "The grant was revoked. It has already stopped: the next thing the agent asks is refused, \
     with nothing to wait for",
)
.noting(
    "Shown after a person revokes a grant from the list. The agent is an AI assistant such as \
     @files, not a person. The second sentence is the point and must survive translation: \
     revocation takes effect on the very next question the machine is asked, not at the next \
     sign-in or restart, and a person revoking something worrying needs to know that there is \
     nothing further to do.",
);

/// The row was stale: that grant is no longer held.
pub const ALREADY_GONE: Word = Word::saying(
    "granted.already-gone",
    "That grant is no longer held: it has expired or was already revoked. Nothing was changed — \
     the list here was out of date, so look at it again",
)
.noting(
    "Shown when somebody revokes a row from a list the machine has moved past — the grant \
     expired, or was revoked from somewhere else, after the list was drawn. Nothing was changed \
     is literal: the machine's grants are exactly as they were. The last clause is the repair — \
     the list needs re-reading — and it must read as a fact about the list being stale rather \
     than as a fault of the person.",
);

/// Every string this crate can say, in the order this file declares them.
pub const EVERY_WORD: [Word; 4] = [NOTHING_GRANTED, ONE_GRANT, REVOKED, ALREADY_GONE];

/// The gap in [`ONE_GRANT`] that carries the agent's name.
pub const AGENT: &str = "agent";

/// The gap in [`ONE_GRANT`] that carries what the grant covers.
pub const WHAT: &str = "what";

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
pub fn granted_words() -> Result<Vocabulary, WordsError> {
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

    /// The three whole sentences; the row is the one clause.
    const THE_SENTENCES: [Word; 3] = [NOTHING_GRANTED, REVOKED, ALREADY_GONE];

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
            assert_eq!(word.key().area(), "granted", "{}", word.named());
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it.
    #[test]
    fn the_whole_list_declares() {
        assert_eq!(granted_words().unwrap().how_many(), EVERY_WORD.len());
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = granted_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **The only gaps in this crate are the row's two**, and both are in the
    /// row. The three sentences are whole, because each is read at a moment
    /// something already needs explaining, and a `{}` in front of a person at
    /// that moment is the worst available rendering of it.
    #[test]
    fn the_only_gaps_are_the_rows_two() {
        for word in EVERY_WORD {
            let gaps = word.phrase().unwrap().source().gaps().to_vec();
            if word.named() == ONE_GRANT.named() {
                assert_eq!(
                    gaps,
                    [AGENT.to_owned(), WHAT.to_owned()],
                    "{}",
                    word.named()
                );
            } else {
                assert!(gaps.is_empty(), "{} has a gap in it", word.named());
            }
        }
    }

    /// **The sentences stand alone and the row does not.** A sentence is read
    /// with nothing above it, so it is capitalised; the row is read inside a
    /// list, so it is not.
    #[test]
    fn the_sentences_begin_a_sentence_and_the_row_does_not() {
        for word in THE_SENTENCES {
            let first = word.says().chars().next().unwrap();
            assert!(
                first.is_uppercase(),
                "{} does not begin a sentence",
                word.named()
            );
        }
        let first = ONE_GRANT.says().chars().next().unwrap();
        assert!(
            first == '{',
            "the row does not begin with the agent's own name"
        );
    }

    /// **Every sentence that names the agent says the agent is not a person**,
    /// in the note a translator works from — in a language where that answer
    /// decides the grammar of the whole sentence.
    #[test]
    fn every_word_that_names_the_agent_says_what_the_agent_is() {
        for word in EVERY_WORD {
            if !word.says().to_ascii_lowercase().contains("agent") {
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

    /// **Every word carries a note.** None of the four can be translated from
    /// its own words: each is read at a moment the translator has to be able
    /// to picture, and the row's two gaps carry things a translator must not
    /// touch.
    #[test]
    fn every_word_carries_a_note_for_the_translator() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// **Nothing here counts anything.** A number written into an English
    /// sentence cannot be translated into a language with three plural forms;
    /// how many grants there are is the length of the list, which is drawn,
    /// not said.
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

    /// **The refusal says what to do and the stale row is not blamed on the
    /// person.** Somebody whose revocation landed on nothing has to know two
    /// things: nothing changed, and the repair is to look again.
    #[test]
    fn the_refusal_says_nothing_changed_and_what_to_do() {
        assert!(ALREADY_GONE.says().contains("Nothing was changed"));
        assert!(ALREADY_GONE.says().contains("look at it again"));
    }

    /// **The revocation says when it took effect**, because immediately is the
    /// fact a person revoking something worrying most needs — and it is the
    /// property `alo_capability::Grants::revoke` actually has, restated where
    /// the person is.
    #[test]
    fn the_revocation_says_it_has_already_stopped() {
        assert!(REVOKED.says().contains("already stopped"));
    }
}
