//! Every string this crate can say, and the English beside each one.
//!
//! `CLAUDE.md` says hardcoded English is a bug. This is the list that stops it
//! being one here: a key, the sentence in the language the code is written in,
//! and the note a translator needs.
//!
//! # What is on this list, and what is deliberately not
//!
//! **What the other machine said at its door is here**, one sentence a word.
//! A refusal crosses the corridor as one word from a closed list, because the
//! receiving machine words its refusals in its own person's language and the
//! person who asked reads their own — so the sentence for each word is said
//! on the asking machine, from this list.
//!
//! **What is not here is anything about the proof.** *Not paired*, *not from
//! the machine it names*, *already used* and *from another moment* are
//! `alo-nearby`'s sentences, said there for a proof judged there, and the
//! same four are what the asking person reads when the other machine judged
//! theirs. One sentence per fact, wherever the fact was found.
//!
//! # The other machine's name is never translated
//!
//! `{machine}` is what the person called the other machine when they paired
//! with it, or its identity if they called it nothing. It is somebody's own
//! data, like a filename in `alo-files`, and a translation of it would be an
//! invention.

use alo_strings::Vocabulary;

/// One string a crate can say.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// What the other machine said at its door — [`crate::AtTheDoor`].
//
// Read on the machine that asked, by the person whose agent asked, having
// just been told that something did not happen on the other machine. Each
// says what is true and, where there is something to do, what.
// ---------------------------------------------------------------------------

/// The message reached the other machine with no proof on it.
pub const NO_PROOF: Word = Word::saying(
    "corridor.at-the-door.no-proof",
    "{machine} received this with nothing to show it came from here. Nothing was done.",
)
.noting(
    "Said when a verb reached the other machine without the proof every verb carries — a fault \
     in the software rather than in anything the person did. {machine} is the name the reader \
     gave the other machine.",
);

/// What arrived at the other machine was not a verb it could read.
pub const NOT_A_VERB: Word = Word::saying(
    "corridor.at-the-door.not-a-verb",
    "{machine} could not read what was sent as a verb. Nothing was done.",
)
.noting(
    "Said when the other machine could not read the message as a verb and its arguments. \
     {machine} is the name the reader gave the other machine.",
);

/// The other machine is holding a turn for a different machine.
pub const ANOTHER_TURN_IS_OPEN: Word = Word::saying(
    "corridor.at-the-door.another-turn-is-open",
    "{machine} is busy with another machine at the moment. Try again shortly.",
)
.noting(
    "Said when the other machine was already carrying out verbs for a third machine. The second \
     sentence is what to do. {machine} is the name the reader gave the other machine.",
);

/// The other machine's person has not granted this.
pub const NOT_GRANTED_THERE: Word = Word::saying(
    "corridor.at-the-door.not-granted-there",
    "This has not been granted on {machine}. Only the person at that machine can grant it.",
)
.noting(
    "Said when the other machine's own grants refused the verb — ADR 0003: what a machine may do \
     on another is what that machine's person granted. The second sentence says whose decision \
     it is, so the reader does not look for a setting on their own machine. {machine} is the \
     name the reader gave the other machine.",
);

/// A change was asked for as a read, or a read as a change.
pub const THE_WRONG_DOOR: Word = Word::saying(
    "corridor.at-the-door.the-wrong-door",
    "{machine} refused this: a change has to wait for approval there, and a read does not wait.",
)
.noting(
    "Said when a verb that changes something was asked for as a read, or a read was put forward \
     for approval. A fault in the software that asked rather than in anything the person did. \
     {machine} is the name the reader gave the other machine.",
);

/// It never became a call on the other machine.
pub const TURNED_AWAY: Word = Word::saying(
    "corridor.at-the-door.turned-away",
    "{machine} does not do that, or not with what was given. Nothing was done.",
)
.noting(
    "Said when the other machine has no such verb, or its arguments did not pass. {machine} is \
     the name the reader gave the other machine.",
);

/// There was no boundary on the other machine to run it inside.
pub const NOT_BOUNDED_THERE: Word = Word::saying(
    "corridor.at-the-door.not-bounded",
    "{machine} could not put a boundary around this, so it did not run.",
)
.noting(
    "Said when the other machine could not impose the kernel boundary every verb runs inside \
     (ADR 0013, ADR 0015) and so ran nothing. {machine} is the name the reader gave the other \
     machine.",
);

/// Everything permitted it and the other machine could not.
pub const MACHINE_COULD_NOT: Word = Word::saying(
    "corridor.at-the-door.machine-could-not",
    "{machine} could not do this — the file may be gone, or its disk may be full.",
)
.noting(
    "Said when the other machine's grants permitted the verb and the machine itself failed. \
     Deliberately not a refusal: nothing said no. {machine} is the name the reader gave the \
     other machine.",
);

/// The request was for something other than this wire.
pub const NOT_FOR_THIS_WIRE: Word = Word::saying(
    "corridor.at-the-door.not-for-this-wire",
    "{machine} did not recognise the request. Its alo OS may be older or newer than this one.",
)
.noting(
    "Said when the other machine answered that it has no such door. \"alo OS\" is the product's \
     name and is not translated. {machine} is the name the reader gave the other machine.",
);

// ---------------------------------------------------------------------------
// What came back that was not an answer — [`crate::WentBack`].
// ---------------------------------------------------------------------------

/// The other machine answered with something this one could not read.
pub const NOT_AN_ANSWER: Word = Word::saying(
    "corridor.not-crossed.not-an-answer",
    "{machine} answered with something this machine could not read. Nothing is known about \
     what happened there.",
)
.noting(
    "Said when the reply was neither an answer nor a refusal this machine knows — a newer \
     version, or something else on that port. The second sentence is deliberately unsure: the \
     verb may or may not have run. {machine} is the name the reader gave the other machine.",
);

/// Everything this crate can say, in the order this file declares them.
pub const EVERY_WORD: [Word; 10] = [
    NO_PROOF,
    NOT_A_VERB,
    ANOTHER_TURN_IS_OPEN,
    NOT_GRANTED_THERE,
    THE_WRONG_DOOR,
    TURNED_AWAY,
    NOT_BOUNDED_THERE,
    MACHINE_COULD_NOT,
    NOT_FOR_THIS_WIRE,
    NOT_AN_ANSWER,
];

/// Why this crate's list could not be declared.
///
/// Not a refusal a person reads — it is read by whoever is fixing the list
/// above, so it keeps its English and its `Display`.
#[derive(Debug, thiserror::Error)]
pub enum WordsError {
    /// A word that is not a phrase.
    #[error(transparent)]
    Word(#[from] alo_strings::WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] alo_strings::VocabularyError),
}

/// Everything this crate can say, as a vocabulary of its own.
///
/// # Errors
///
/// [`WordsError`], which the list above cannot cause.
pub fn corridor_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// # Errors
///
/// [`WordsError::List`] if the vocabulary already holds this key — nothing is
/// replaced, because a key means one string and whoever declared it first
/// said what that string is.
pub fn declare_into(vocabulary: &mut Vocabulary) -> Result<(), WordsError> {
    for word in EVERY_WORD {
        vocabulary.says(word.phrase()?)?;
    }
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::{EVERY_WORD, corridor_words};

    /// Every word declares, every key is this crate's, and every sentence
    /// but none of the proof's names the other machine.
    #[test]
    fn every_word_declares_under_this_crates_area_and_names_the_machine() {
        let vocabulary = corridor_words().unwrap();
        assert_eq!(vocabulary.how_many(), EVERY_WORD.len());
        for word in EVERY_WORD {
            let key = word.key();
            assert!(key.as_str().starts_with("corridor."), "{}", key.as_str());
            assert!(word.says().contains("{machine}"), "{}", key.as_str());
        }
    }

    /// The list can be declared into a vocabulary beside `alo-nearby`'s,
    /// which it reaches for the proof's four sentences, without a clash.
    #[test]
    fn the_list_lives_beside_the_nearby_list_without_a_clash() {
        let mut vocabulary = alo_nearby::nearby_words().unwrap();
        super::declare_into(&mut vocabulary).unwrap();
        assert!(super::declare_into(&mut vocabulary).is_err());
    }
}
