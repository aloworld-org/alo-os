//! Every string this crate can say, and the English beside each one.
//!
//! Two, and each is the sentence for one of the two moments this crate is
//! about.
//!
//! [`NOT_KEPT`] is the write that failed. `alo_remembering::kept` replaces the
//! file whole or not at all, so the one thing a person needs at that moment is
//! that **nothing moved**: the file is as it was, the daemon was not knocked,
//! and what is granted did not change in either direction. Which failure it
//! was keeps its English inside [`crate::NotChanged`], for whoever is looking
//! at the machine — the person is told the one thing that is theirs to know.
//!
//! [`AT_THE_NEXT_SIGN_IN`] is the change that stood with nobody at the door.
//! It exists because silence there would be ambiguous in the worst direction:
//! a person who revoked something worrying and was told nothing could not
//! tell *the machine handled it* from *nothing happened*. The sentence says
//! the change is safe, when it takes effect everywhere, and that there is
//! nothing more to do — because a machine that leaves a person wondering
//! whether to do it again is a machine that gets the same change made twice.
//!
//! The other four are a pairing's revocation, which the daemon makes rather
//! than this crate, and each is a moment a person would otherwise be told
//! something that is not true. [`UNTIL_A_RESTART`] is a pairing taken away
//! now that comes back after a restart. [`PAIRING_REFUSED`] carries the
//! daemon's own sentence inside it — its one gap — so a refusal is never
//! reported as done. [`NOBODY_KEEPS_PAIRINGS`] is no daemon at all: only the
//! daemon writes the pairings, so nothing was revoked. And
//! [`PAIRING_NOT_ANSWERED`] is a daemon reached that never said, which is the
//! one moment the honest sentence is *look again*.

use alo_strings::{Vocabulary, VocabularyError, WordError};

/// One string a crate can say — `alo-strings`' type, re-exported because this
/// crate's files and the tests that read its list name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

/// The change was not saved, and nothing moved.
pub const NOT_KEPT: Word = Word::saying(
    "changing.not-kept",
    "The change was not saved: the file this machine keeps its grants in would not take it. \
     Everything is as it was — nothing was granted and nothing was revoked",
)
.noting(
    "Shown when writing the machine's own grants file fails — a disk problem, or a file somebody \
     has interfered with. The second sentence is the point and must survive translation: the \
     file is replaced whole or not at all, so a failed save changes nothing in either direction \
     — no new permission exists, and no permission the person tried to take away has been taken \
     away. What exactly went wrong is written to the machine's log in English for whoever \
     maintains it; this sentence is everything the person needs.",
);

/// The change is saved, and it takes effect at the next sign-in.
pub const AT_THE_NEXT_SIGN_IN: Word = Word::saying(
    "changing.at-the-next-sign-in",
    "The change is saved. Nothing is running to hear about it right now, so it takes effect at \
     the next sign-in — there is nothing more to do",
)
.noting(
    "Shown when a grant was made or revoked and no agent service was running to be told. The \
     change (a permission given or taken away) is safely on the machine's own disk and will \
     apply the moment anything starts that reads it. The last clause matters: the person must \
     not be left wondering whether to make the change again.",
);

/// The pairing is revoked for now, and comes back after a restart.
pub const UNTIL_A_RESTART: Word = Word::saying(
    "changing.until-a-restart",
    "The pairing is revoked for now, but this machine could not write that down. After a \
     restart the pairing will stand again, and it will have to be revoked again",
)
.noting(
    "Shown when a person revoked a pairing with another machine and this machine's agent \
     service took it away at once but could not save the change to its disk. The other machine \
     cannot use the pairing now; after this machine restarts it can, until the person revokes it \
     again. Both halves must survive translation: the first is reassurance, the second is the \
     one thing the person still has to do.",
);

/// The daemon refused the revocation, in its own words.
pub const PAIRING_REFUSED: Word = Word::saying(
    "changing.pairing-refused",
    "The pairing was not revoked. This machine's agent service said: {told}",
)
.noting(
    "Shown when a person revoked a pairing with another machine and this machine's agent \
     service refused. {told} is the service's own sentence, already translated — for example \
     that nothing is paired with that machine — and is not translated again. The first sentence \
     matters most: the person must not believe the pairing is gone.",
);

/// No daemon was running to revoke the pairing, so nothing was revoked.
pub const NOBODY_KEEPS_PAIRINGS: Word = Word::saying(
    "changing.nobody-keeps-pairings",
    "The pairing was not revoked: the service that keeps this machine's pairings is not running, \
     and nothing else may change them. It stands as it was",
)
.noting(
    "Shown when a person revoked a pairing with another machine and this machine's agent service \
     was not running. Only that service writes the list of pairings, deliberately, so nothing \
     else could take the pairing away. The last sentence is the point: the pairing still exists.",
);

/// The daemon was reached and did not say what became of the revocation.
pub const PAIRING_NOT_ANSWERED: Word = Word::saying(
    "changing.pairing-not-answered",
    "This machine's agent service did not say whether the pairing was revoked. Look at the list \
     of pairings again before relying on it",
)
.noting(
    "Shown when a person revoked a pairing with another machine and the agent service was \
     reached but never answered. The pairing may or may not be gone, and the sentence must not \
     suggest either: the person is asked to look at the list, which says which it is.",
);

/// Every string this crate can say, in the order this file declares them.
pub const EVERY_WORD: [Word; 6] = [
    NOT_KEPT,
    AT_THE_NEXT_SIGN_IN,
    UNTIL_A_RESTART,
    PAIRING_REFUSED,
    NOBODY_KEEPS_PAIRINGS,
    PAIRING_NOT_ANSWERED,
];

/// The name of the gap [`PAIRING_REFUSED`] carries the daemon's sentence in.
pub const TOLD: &str = "told";

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
pub fn changing_words() -> Result<Vocabulary, WordsError> {
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
            assert_eq!(word.key().area(), "changing", "{}", word.named());
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it.
    #[test]
    fn the_whole_list_declares() {
        assert_eq!(changing_words().unwrap().how_many(), EVERY_WORD.len());
    }

    /// A vocabulary that already holds one of these keeps its own, and nothing
    /// is quietly replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = changing_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **Every sentence is whole but the one that carries the daemon's.**
    /// Each is read at a moment something already needs explaining, and a
    /// `{}` in front of a person at that moment is the worst available
    /// rendering of it — so the one gap there is, is named and is the
    /// daemon's own sentence.
    #[test]
    fn nothing_here_has_a_gap_but_the_daemons_sentence() {
        for word in EVERY_WORD {
            let phrase = word.phrase().unwrap();
            let gaps = phrase.source().gaps();
            if word.named() == PAIRING_REFUSED.named() {
                assert_eq!(gaps, vec![TOLD.to_owned()], "{}", word.named());
            } else {
                assert!(gaps.is_empty(), "{} has a gap in it", word.named());
            }
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

    /// **Every word carries a note.** Neither can be translated from its own
    /// words: each is read at a moment the translator has to be able to
    /// picture.
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

    /// **The failure says nothing moved and the success says what to do** —
    /// which is nothing. The two facts a person is owed at these two moments,
    /// held where the translator meets them.
    #[test]
    fn each_sentence_carries_the_fact_it_exists_for() {
        assert!(NOT_KEPT.says().contains("nothing was granted"));
        assert!(NOT_KEPT.says().contains("nothing was revoked"));
        assert!(AT_THE_NEXT_SIGN_IN.says().contains("next sign-in"));
        assert!(AT_THE_NEXT_SIGN_IN.says().contains("nothing more to do"));
        assert!(UNTIL_A_RESTART.says().contains("revoked again"));
        assert!(PAIRING_REFUSED.says().contains("was not revoked"));
        assert!(NOBODY_KEEPS_PAIRINGS.says().contains("was not revoked"));
        assert!(PAIRING_NOT_ANSWERED.says().contains("Look at the list"));
    }
}
