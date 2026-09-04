//! Every string this crate can say, and the English beside it.
//!
//! `CLAUDE.md` says hardcoded English is a bug. This is the list that stops it
//! being one here, and it holds two strings.
//!
//! # Two, in the crate that joins five others together
//!
//! Everything a person reads during a turn was already somebody's to say, and
//! this crate deliberately does not say any of it a second time:
//!
//! | What a person reads | Whose string it is |
//! |---|---|
//! | *archive /home/anna/Invoices into backup.zip* | the verb's own declaration, by way of `alo-capability` |
//! | *there is no verb called delete_everything* | `alo-capability`, when nothing formed |
//! | *@files has not been granted /etc/shadow* | `alo-capability`, when the grants refused |
//! | *there is nothing at that path* | `alo-files`, when the machine could not |
//! | *nothing has been written to the record* | `alo-keeping`, when the disk did not |
//!
//! A second rendering of any of those would be a machine able to describe one
//! moment two ways, which is the failure the 9-series spent seven items
//! removing. What is left is the two things none of them knows about, because
//! both are facts about the turn rather than about what was asked of it: that
//! it stopped, and that there was no boundary to run its work inside.
//!
//! # The second one was `alo-bounding`'s, and moving it is the same rule
//!
//! [`NOT_BOUNDED`] said what it says today in `alo-bounding`'s own list, where
//! nothing could look it up: that crate is Linux, so its words are not in the
//! vocabulary `alo-saying` collects, and a machine would have had to be told to
//! declare them on top. A sentence that reaches a person only if whoever
//! assembled the process remembered is a sentence that reaches somebody as a
//! key. It lives here because **this is the crate that tells the person** — a
//! boundary is a mechanism and says nothing to anybody — and because a portable
//! crate's refusal has to be sayable on every host this crate compiles for.
//!
//! # Nothing here counts, and nothing here has a gap
//!
//! There is no [`alo_strings::Plural`], for the reason `alo-models` and
//! `alo-asking` give: a sentence that counted would be English's two shapes
//! standing in for Polish's three. And the one sentence has no gap in it, so
//! there is nothing a translation could drop and nothing anybody outside alo OS
//! wrote that could be put into one.

use alo_strings::Vocabulary;

/// One string a crate can say.
///
/// Re-exported because this crate's own files, and the tests that read this
/// list, name it as `crate::words::Word`.
pub use alo_strings::Word;

/// This turn stopped, because what happened on it could not be written down.
pub const TURN_CLOSED: Word = Word::saying(
    "turn.closed",
    "this turn has stopped, because what happened on it could not be written down",
)
.noting(
    "Read by somebody whose machine has stopped keeping evidence of what its agent did. It is not \
     the agent being refused and must not sound like it: nothing was disallowed, and the reason \
     the record could not be written is said separately, in the record's own words.",
);

/// There was no boundary to run this turn's work inside, so nothing was done.
///
/// One sentence for every reason there is. What a person can act on is what is
/// true of all of them — nothing happened, nothing was refused either, and the
/// machine rather than their agent is what has to be looked at — and the reason
/// itself is a fact about a kernel, kept in English for whoever administers the
/// machine and deliberately not worked into this.
pub const NOT_BOUNDED: Word = Word::saying(
    "turn.not-bounded",
    "nothing was done: this machine cannot hold an agent inside what you granted it, so it will \
     not let one act at all — ask whoever set this machine up to look at it",
)
.noting(
    "Shown to a person whose agent did nothing because alo OS could not put the boundary around \
     it that keeps it inside the folders and files they granted. Nothing was attempted and \
     nothing was refused: this is a fault in the machine rather than an answer to anything the \
     person or the agent asked for. The reason is a separate, technical sentence kept in English \
     for whoever administers the machine, and must not be worked into this one.",
);

/// Every string this crate can say, in the order this file declares them.
///
/// The array is what a test reads down and what [`declare_into`] walks, so a
/// word declared above and left out here is a string nothing can look up.
pub const EVERY_WORD: [Word; 2] = [TURN_CLOSED, NOT_BOUNDED];

/// Why this crate's own list could not be declared.
///
/// Not a refusal a person reads — it is read by whoever is fixing the list
/// above, so it keeps its English and its `Display` for the reason
/// `alo_files::Declaring` does.
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
pub fn turn_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// The shell has one vocabulary and every crate adds its own to it, which is
/// what the area at the front of a key is for.
///
/// # Errors
/// [`WordsError::List`] if the vocabulary already holds this key — nothing is
/// replaced, because a key means one string and whoever declared it first said
/// what that string is.
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
    use alo_strings::Key;

    /// **What we ship is held to the rule everybody else is held to.**
    /// [`Word::key`] does not check, because a key written in this file cannot
    /// arrive from anywhere; this is the test that makes that true.
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(
                Key::named(word.named()),
                Ok(word.key()),
                "{}: {}",
                word.named(),
                Key::named(word.named()).unwrap_err()
            );
        }
    }

    /// Every one of them is in the area a reader can sort by, which is what
    /// lets one vocabulary hold every crate's strings.
    #[test]
    fn everything_this_crate_says_says_it_is_this_crate() {
        for word in EVERY_WORD {
            assert_eq!(word.key().area(), "turn", "{}", word.named());
        }
    }

    /// The list declares, and nothing about it is refused by the crate that
    /// receives it.
    #[test]
    fn the_whole_list_declares() {
        let vocabulary = turn_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        assert_eq!(vocabulary.counted().count(), 0);
    }

    /// A vocabulary that already holds it keeps its own, and nothing is quietly
    /// replaced.
    #[test]
    fn a_key_already_taken_is_not_replaced() {
        let mut vocabulary = turn_words().unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **The sentence has no gap in it**, which is the strong form of what this
    /// crate holds: a turn carries the person's own document, the agent's
    /// arguments and the model's words, and none of them can reach a string of
    /// ours because there is nowhere in one for them to go.
    #[test]
    fn the_sentence_has_no_gap_for_anything_to_be_put_into() {
        for word in EVERY_WORD {
            assert!(!word.says().contains('{'), "{}", word.named());
        }
    }

    /// It carries a note, because it is read at the one moment somebody is most
    /// likely to think their agent was refused something.
    #[test]
    fn it_carries_a_note() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }
}
