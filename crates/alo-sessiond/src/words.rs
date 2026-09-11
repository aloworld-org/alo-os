//! Every string this crate can say, and the English beside each one.
//!
//! Four, one for each way a knock can be answered badly. None of them is ever
//! printed by this process: what crosses the wire is the key, and the surface
//! that drew the sign-in is what renders it in the person's own language. That
//! is why they are here rather than in `crate::refusing` — a refusal a person
//! reads and a refusal a service log carries are two different sentences with
//! two different readers, and this file holds the first kind.
//!
//! [`NOT_OPENED`] is the machine refusing. The password was right, the account
//! exists, and `systemd-logind` would not open the session anyway — or the
//! accounts file could not be read at all. From where the person is standing
//! these are one fact: signing in did not finish, and it was not something they
//! typed. Which of them it was is English in [`crate::NotOpened`], for whoever
//! is looking at the machine.
//!
//! [`NOBODY_HERE`] is a number this machine has no account for. A person cannot
//! cause it by typing: the surface only ever asks for the number it has just
//! authenticated. What causes it is a machine whose accounts and whose sign-in
//! surface disagree — so the sentence says which of the two to go and look at.
//!
//! [`NOT_YOURS_TO_ASK`] is something other than the sign-in surface at the door.
//! It is the one sentence here that is a security answer, and it deliberately
//! says nothing about what was asked for: a refusal that named the number would
//! be this component answering *is there an account numbered 1000* to whatever
//! managed to connect.
//!
//! [`ALREADY_SIGNED_IN`] is a second knock while a session is open. The honest
//! thing to tell somebody at that moment is that the machine is already signed
//! in and how to get out of it, because the alternative — a second session
//! nobody asked for — is a machine with two of everything.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported because this
/// crate's files and the tests that read its list name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

/// The machine would not open a session.
pub const NOT_OPENED: Word = Word::saying(
    "signing-in.not-opened",
    "Signing in did not finish: this machine would not start a session. Nothing you typed was \
     wrong, and nothing about your account has changed",
)
.noting(
    "Shown after a correct password, when the part of the machine that starts a session refused \
     to. The person has done nothing wrong and there is nothing for them to type differently, \
     which is the whole point of the second sentence: without it they will try their password \
     again, decide they have forgotten it, and change it. What actually went wrong is written to \
     the machine's log in English for whoever maintains it.",
);

/// There is no account on this machine with that number.
pub const NOBODY_HERE: Word = Word::saying(
    "signing-in.nobody-here",
    "Signing in did not finish: this machine has no account with that number. The accounts on \
     this machine and the screen you signed in at disagree about who you are",
)
.noting(
    "Shown when a session was asked for on behalf of an account number this machine does not \
     have. A person cannot cause this by typing anything — it means two parts of the machine \
     have been set up inconsistently — so the second sentence is addressed to whoever can fix \
     that rather than to the person's memory of their password. \"Number\" is the machine's \
     internal number for an account, not anything the person chose or would recognise.",
);

/// Whatever knocked is not the sign-in surface.
pub const NOT_YOURS_TO_ASK: Word = Word::saying(
    "signing-in.not-yours-to-ask",
    "Something asked this machine to start a session, and it is not the screen people sign in \
     at. Nothing was started",
)
.noting(
    "Shown when a program that is not the sign-in surface asked for a session to be opened. It \
     is a security refusal, and it deliberately says nothing about who was asked for or whether \
     that account exists — naming either would answer a question the asker was refused. Keep it \
     as flat as the English is: it is a statement of what happened, not an accusation and not an \
     alarm.",
);

/// Somebody is signed in already.
pub const ALREADY_SIGNED_IN: Word = Word::saying(
    "signing-in.already-signed-in",
    "This machine is already signed in. Sign out of the session that is open before starting \
     another one",
)
.noting(
    "Shown when a session is asked for while one is already open on this machine. The second \
     sentence is an instruction and must stay one: the person can act on this themselves, which \
     is what makes it different from every other sentence in this file.",
);

/// Every string this crate can say, in the order this file declares them.
pub const EVERY_WORD: [Word; 4] = [NOT_OPENED, NOBODY_HERE, NOT_YOURS_TO_ASK, ALREADY_SIGNED_IN];

/// Why this crate's own words could not be declared.
///
/// Neither can happen to the list above — the tests at the bottom of this file
/// are what say so. It is a `Result` rather than an unwrap because a library
/// that panics on its own string table takes its caller with it, and because
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
pub fn sessiond_words() -> Result<Vocabulary, WordsError> {
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
            assert_eq!(word.key().area(), "signing-in", "{}", word.named());
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it.
    #[test]
    fn the_whole_list_declares() {
        assert_eq!(sessiond_words().unwrap().how_many(), EVERY_WORD.len());
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = sessiond_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **Every sentence is whole.** Each is read at the one moment a person
    /// cannot get into their own machine, and a `{}` in front of somebody at
    /// that moment is the worst available rendering of it.
    #[test]
    fn nothing_here_has_a_gap_in_it() {
        for word in EVERY_WORD {
            assert!(
                word.phrase().unwrap().source().gaps().is_empty(),
                "{} has a gap in it",
                word.named()
            );
        }
    }

    /// **The sentences stand alone**, so each begins one.
    #[test]
    fn every_sentence_begins_a_sentence() {
        for word in EVERY_WORD {
            let first = word.says().chars().next().unwrap();
            assert!(
                first.is_uppercase(),
                "{} does not begin a sentence",
                word.named()
            );
        }
    }

    /// **Every word carries a note.** Not one of them can be translated from
    /// its own words: each is read at a moment the translator has to be able to
    /// picture, and two of them are addressed to somebody other than the person
    /// standing there.
    #[test]
    fn every_word_carries_a_note_for_the_translator() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// **Nothing here counts anything.** A number written into an English
    /// sentence cannot be translated into a language with three plural forms.
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

    /// **The security refusal names nothing it was asked about.** A sentence
    /// that mentioned the account, the number or whether either exists would
    /// answer the question the caller was refused, which is the entire failure
    /// this refusal exists to prevent.
    #[test]
    fn the_refusal_that_is_a_security_answer_gives_nothing_away() {
        for giveaway in ["account", "number", "exist"] {
            assert!(
                !NOT_YOURS_TO_ASK.says().contains(giveaway),
                "the refusal mentions `{giveaway}`"
            );
        }
    }

    /// **The one a person can act on tells them what to do, and the ones they
    /// cannot act on tell them it was not their password.** The two facts each
    /// sentence exists for, held where the translator meets them.
    #[test]
    fn each_sentence_carries_the_fact_it_exists_for() {
        assert!(NOT_OPENED.says().contains("Nothing you typed was wrong"));
        assert!(NOBODY_HERE.says().contains("disagree"));
        assert!(ALREADY_SIGNED_IN.says().contains("Sign out"));
    }
}
