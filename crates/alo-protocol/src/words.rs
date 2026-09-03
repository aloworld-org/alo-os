//! Every string this crate can say, and the English beside each one.
//!
//! `CLAUDE.md` says hardcoded English is a bug. This is the list that stops it
//! being one here: a key, the sentence in the language the code is written in,
//! and the note a translator needs. `alo-strings` does the rest.
//!
//! # Every one of these is a refusal, and there are no others
//!
//! A request that is understood produces nothing to say — what happens next is
//! a turn's, and a turn words its own answers. So this list is exactly the
//! seven ways a message can fail to be a request, which is the whole of what
//! item 21 means by *a malformed request is refused in the reader's own
//! language, not dropped*.
//!
//! # Nothing here quotes the message back
//!
//! Not one of these seven has a gap in it, and that is a decision rather than a
//! coincidence. What arrives at this door is bytes somebody or something else
//! wrote, and a refusal that repeated them would put text nobody validated into
//! a sentence a person reads — which is `alo-record`'s *the arguments of a call
//! that never validated are not kept* met at the moment before there is a call
//! at all. The numbers that would have been quoted — how long the message was,
//! what format it claimed — are fields on
//! [`NotUnderstood`](crate::NotUnderstood), for whoever draws them beside the
//! sentence.
//!
//! It also means nothing here counts anything out loud, which is `alo-models`'
//! rule from item 9f kept for the same reason: English's two plural shapes
//! standing in for Polish's three, in a sentence that did not need a number at
//! all.

use alo_strings::Vocabulary;

/// One string a crate can say.
///
/// Lifted into `alo-strings` by item 9d. Re-exported here because this crate's
/// own files, and the tests that read this list, name it as
/// `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// The envelope: what is wrong with the message before anybody looks inside it.
// ---------------------------------------------------------------------------

/// A message longer than this machine will hold.
pub const TOO_LONG: Word = Word::saying(
    "protocol.too-long",
    "that message is longer than this machine will read — send a shorter one",
)
.noting(
    "Said to whatever sent the message, and read by the person who has to fix it. \"This machine\" \
     is the person's own computer. The limit itself is a number this sentence deliberately does \
     not carry, so that no language has to guess at a plural for it.",
);

/// Two messages where one was expected.
pub const MORE_THAN_ONE_MESSAGE: Word = Word::saying(
    "protocol.more-than-one-message",
    "send one message per line — that was more than one",
)
.noting(
    "A message is one line of text. This is said when what arrived had a line break inside it, \
     which would mean everything after the break was silently thrown away.",
);

/// A message from a version of alo OS this one does not know.
pub const FROM_A_NEWER_ALO_OS: Word = Word::saying(
    "protocol.from-a-newer-alo-os",
    "that message comes from a newer alo OS than this one — update this machine",
)
.noting(
    "\"alo OS\" is the product's name and is never translated. Said when a message states a \
     format number this version has never written, which means something newer is talking to \
     something older.",
);

/// A message that names a format nothing ever wrote.
pub const NOT_A_FORMAT: Word = Word::saying(
    "protocol.not-a-format",
    "that message names a format no alo OS has ever written",
)
.noting(
    "\"alo OS\" is the product's name and is never translated. \"Format\" is the number a message \
     states so that a reader knows how to read it. This is said about a number that is not one \
     any version ever used, which usually means the message was not written by an alo OS client \
     at all.",
);

/// A message that could not be read at all.
pub const NOT_READABLE: Word = Word::saying(
    "protocol.not-readable",
    "this machine could not read that message",
)
.noting(
    "Deliberately says nothing about what was wrong with it and quotes none of it back: what \
     arrived is text nobody has checked, and repeating it would put it in front of a person. \
     What a client needs in order to fix it is the written protocol, not this sentence.",
);

// ---------------------------------------------------------------------------
// The two doors: a message that is a request, but not one this caller may make.
// ---------------------------------------------------------------------------

/// An agent reaching for something only a person answers.
pub const NOT_FOR_AN_AGENT: Word = Word::saying(
    "protocol.not-for-an-agent",
    "an agent cannot answer a question that was put to a person",
)
.noting(
    "The most important refusal in this list. Approving a change is the person's answer, and this \
     is what is said when it arrives from the side that proposed it. \"Agent\" is the assistant \
     running on the machine, not the person using it.",
);

/// A person's shell reaching for something only an agent asks.
pub const NOT_FOR_A_PERSON: Word = Word::saying(
    "protocol.not-for-a-person",
    "that is something an agent asks during a turn, not an answer a person gives",
)
.noting(
    "\"A turn\" is one exchange, from the moment the person calls the agent until it is over. \
     Said when a request an agent makes arrives on the side a person's screen speaks over.",
);

/// Everything this crate can say, in one list.
pub const EVERY_WORD: [Word; 7] = [
    TOO_LONG,
    MORE_THAN_ONE_MESSAGE,
    FROM_A_NEWER_ALO_OS,
    NOT_A_FORMAT,
    NOT_READABLE,
    NOT_FOR_AN_AGENT,
    NOT_FOR_A_PERSON,
];

/// Why this crate's own list could not be declared.
///
/// Not a refusal a person reads — it keeps its English and its `Display` for
/// the reason `alo-capability`'s `VerbError` does: whoever reads it is whoever
/// is fixing the list.
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

/// This crate's words, as a vocabulary of their own.
///
/// For a test, and for anything holding only this list. A machine loads one
/// vocabulary that every crate declares into, which is [`declare_into`].
///
/// # Errors
/// [`WordsError`], which the list above cannot cause.
pub fn protocol_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put this crate's words into a vocabulary something else is building.
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
    use alo_strings::Key;
    use std::collections::BTreeSet;

    /// Every key here is a key, which is what `Word::key` going through
    /// `Key::unchecked` owes each declaring crate.
    #[test]
    fn every_key_this_crate_writes_is_a_key() {
        for word in EVERY_WORD {
            assert!(Key::named(word.named()).is_ok(), "{}", word.named());
        }
    }

    /// A key means one string, so the list may not name one twice.
    #[test]
    fn no_key_is_declared_twice() {
        let named: BTreeSet<&str> = EVERY_WORD.iter().map(|word| word.named()).collect();
        assert_eq!(named.len(), EVERY_WORD.len());
    }

    /// Every one of them is under this crate's own area of the key space, so a
    /// machine loading eleven lists has no collision to discover at runtime.
    #[test]
    fn every_key_is_in_this_crates_own_area() {
        for word in EVERY_WORD {
            assert!(word.named().starts_with("protocol."), "{}", word.named());
        }
    }

    /// The list declares, and declaring the same list twice is refused rather
    /// than replacing what is there.
    #[test]
    fn the_list_declares_once_and_refuses_a_second_time() {
        let mut vocabulary = protocol_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **Nothing this crate says quotes the message back**, and the shape that
    /// keeps it true is that no sentence here has a gap at all. A gap is the
    /// only road text off a socket could take into a sentence a person reads.
    #[test]
    fn no_sentence_here_has_a_gap_in_it() {
        for word in EVERY_WORD {
            assert!(!word.says().contains('{'), "{}", word.named());
            assert!(!word.says().contains('}'), "{}", word.named());
        }
    }

    /// **Nothing here counts anything**, which is item 9f's rule kept: the two
    /// numbers a reader might want are fields on the refusal, for whoever draws
    /// them beside the sentence in their own language.
    #[test]
    fn nothing_this_crate_says_counts_something() {
        let counting = protocol_words().unwrap();
        assert_eq!(counting.counted().count(), 0);
        for word in EVERY_WORD {
            assert!(
                !word.says().chars().any(|char| char.is_ascii_digit()),
                "{}",
                word.named()
            );
        }
    }

    /// Every one of these needs a translator to know something the sentence
    /// does not tell them — which side said it, or what a word of ours means.
    #[test]
    fn every_word_here_carries_a_note() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// The two that divide the doors say **who** may not ask, because a person
    /// told only that something was refused cannot tell whether their own
    /// screen or the agent is at fault.
    #[test]
    fn the_two_door_refusals_name_the_side_that_asked() {
        assert!(NOT_FOR_AN_AGENT.says().contains("agent"));
        assert!(NOT_FOR_AN_AGENT.says().contains("person"));
        assert!(NOT_FOR_A_PERSON.says().contains("agent"));
        assert!(NOT_FOR_A_PERSON.says().contains("person"));
    }
}
