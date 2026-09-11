//! Every string this crate can say, and the English beside each one.
//!
//! Four, and none of them is a refusal about a password: those are
//! `alo-accounts`' two and `alo-sessiond`'s four, carried here exactly as they
//! were declared. A greeter that reworded them would be a machine with two
//! accounts of one moment, and the one nearest a password prompt is the worst
//! place to have two.
//!
//! What is declared here is what nobody else is in a position to say:
//!
//! [`MAKE_AN_ACCOUNT`] is the first screen an alo OS machine ever shows. The
//! image ships no accounts (ADR 0024), so *make an account* rather than *sign
//! in* is the honest first sentence, and a machine that showed a password box
//! nobody could ever fill would be one somebody reasonably concludes is broken.
//!
//! [`ACCOUNTS_UNREADABLE`] is a machine whose accounts file is there and will
//! not be believed — a link, somebody else's file, a file others may write, or
//! text that is not a store. Nobody can sign in on it, and offering a password
//! box would be asking for a keystroke that cannot be checked. The sentence is
//! addressed to whoever installed the machine, because the person in front of
//! it cannot fix this by remembering anything.
//!
//! [`NOTHING_LISTENING`] and [`NOTHING_SAID`] are the conversation with the
//! opener failing to happen. They are two sentences rather than one because
//! they send whoever can act on them to two different places: nothing is
//! running, against something is running and did not finish answering. Both
//! begin by saying that nothing the person typed was wrong — without that they
//! will try the password again, decide they have forgotten it, and change it,
//! which is `alo-sessiond`'s own argument for the same clause.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported because this
/// crate's files and the tests that read its list name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

/// This machine has nobody on it yet.
pub const MAKE_AN_ACCOUNT: Word = Word::saying(
    "greeting.make-an-account",
    "Nobody has an account on this machine yet. Make one to sign in — the account you make here \
     is this machine's own, and it is not sent anywhere",
)
.noting(
    "The first screen an alo OS machine ever shows, before anybody has an account. It replaces \
     the sign-in prompt rather than appearing beside it. The second half is a promise about \
     privacy and is the reason the sentence is not simply \"make an account\": the machine is \
     sold on nothing leaving it, and this is the first moment a person is asked to type \
     anything. Keep both halves.",
);

/// The accounts on this machine will not be read.
pub const ACCOUNTS_UNREADABLE: Word = Word::saying(
    "greeting.accounts-unreadable",
    "Nobody can sign in here: this machine's list of accounts is there and cannot be trusted to \
     be read. Whoever installed this machine has to put that right",
)
.noting(
    "Shown instead of a sign-in prompt when the file holding this machine's accounts exists but \
     is refused — it is a link, it belongs to somebody unexpected, others may write to it, or \
     it is not a list of accounts at all. The person in front of the screen cannot fix this by \
     remembering a password, which is why the second sentence sends them to whoever set the \
     machine up. \"Cannot be trusted to be read\" is deliberate: the file was not unreadable, it \
     was refused on purpose.",
);

/// Nothing is listening at the opener's door.
pub const NOTHING_LISTENING: Word = Word::saying(
    "greeting.nothing-listening",
    "Signing in did not finish: the part of this machine that starts a session is not running. \
     Nothing you typed was wrong",
)
.noting(
    "Shown after a correct password when the service that opens a session could not be reached \
     at all — it is not running, or its door is missing. The second sentence is the important \
     one and is the same promise the other sentences at this screen make: the person has done \
     nothing wrong and must not conclude they have forgotten their password. What exactly went \
     wrong is kept in English for whoever maintains the machine.",
);

/// The door was reached and the conversation did not finish.
pub const NOTHING_SAID: Word = Word::saying(
    "greeting.nothing-said",
    "Signing in did not finish: the part of this machine that starts a session was reached and \
     did not answer. Nothing you typed was wrong",
)
.noting(
    "Shown after a correct password when the service that opens a session was reached but the \
     exchange did not complete — it closed the connection, it took too long, or it answered \
     something this machine could not read. It is deliberately a different sentence from the one \
     about that service not running, because the two send whoever maintains the machine to two \
     different places. As there, the second sentence protects the person from concluding they \
     have forgotten their password.",
);

/// Every string this crate can say, in the order this file declares them.
pub const EVERY_WORD: [Word; 4] = [
    MAKE_AN_ACCOUNT,
    ACCOUNTS_UNREADABLE,
    NOTHING_LISTENING,
    NOTHING_SAID,
];

/// Why this crate's own words could not be declared.
///
/// Neither can happen to the list above — the tests at the bottom of this file
/// are what say so. It is a `Result` rather than an unwrap because a library
/// that panics on its own string table takes the surface with it, and because
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
pub fn greeting_words() -> Result<Vocabulary, WordsError> {
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
            assert_eq!(word.key().area(), "greeting", "{}", word.named());
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it.
    #[test]
    fn the_whole_list_declares() {
        assert_eq!(greeting_words().unwrap().how_many(), EVERY_WORD.len());
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = greeting_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **Nothing here has a gap in it.** A gap is the one road text somebody
    /// typed at a sign-in prompt — a name — could take into a sentence a person
    /// reads, and this is the screen where that text is typed.
    #[test]
    fn nothing_here_has_a_gap_a_typed_name_could_enter_by() {
        for word in EVERY_WORD {
            assert!(
                word.phrase().unwrap().source().gaps().is_empty(),
                "{} has a gap in it",
                word.named()
            );
        }
    }

    /// **The sentences stand alone**, so each begins one — they are read on a
    /// screen with nothing else on it.
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

    /// **Both sentences about a machine that would not finish say that nothing
    /// the person typed was wrong.** Without it they retype, conclude they have
    /// forgotten their password, and change it — which is `alo-sessiond`'s
    /// argument for the same clause, and the two screens are the same screen.
    #[test]
    fn a_machine_that_did_not_finish_says_it_was_not_the_password() {
        for word in [NOTHING_LISTENING, NOTHING_SAID] {
            assert!(
                word.says().contains("Nothing you typed was wrong"),
                "{}",
                word.named()
            );
        }
    }

    /// **And they are two sentences rather than one**, because they send
    /// whoever maintains the machine to two different places.
    #[test]
    fn the_two_ways_the_conversation_fails_are_two_sentences() {
        assert_ne!(NOTHING_LISTENING.says(), NOTHING_SAID.says());
        assert!(NOTHING_LISTENING.says().contains("is not running"));
        assert!(NOTHING_SAID.says().contains("did not answer"));
    }

    /// **The first sentence a machine ever shows says the account stays here.**
    /// It is the first moment anybody is asked to type anything on a machine
    /// sold on nothing leaving it.
    #[test]
    fn the_first_screen_says_the_account_is_this_machines_own() {
        assert!(MAKE_AN_ACCOUNT.says().contains("not sent anywhere"));
    }
}
