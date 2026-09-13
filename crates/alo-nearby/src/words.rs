//! Every string this crate can say, and the English beside each one.
//!
//! `CLAUDE.md` says hardcoded English is a bug. This is the list that stops it
//! being one here: a key, the sentence in the language the code is written in,
//! and the note a translator needs.
//!
//! # What is on this list, and what is deliberately not
//!
//! **What a pairing permits is here**, one sentence an arm. ADR 0003 says a
//! pairing is *enumerated*, and a list somebody can read to the end only counts
//! as enumerated in a language they read. These are the strings a person looks
//! at while deciding whether to let the machine down the corridor ask this one
//! anything, which makes them the strings in this crate it would be least
//! acceptable to leave in English.
//!
//! **The refusals are here too**, which is why [`crate::NotPaired`] has no
//! `Display`. Somebody reads these having just been told they cannot pair with
//! something, not having just read this file.
//!
//! **Nothing about discovery is here at all.** A machine found on the network
//! has no sentence, because there is nothing to tell anybody: presence is not
//! news, and a notification saying *a machine appeared* would turn an open
//! protocol into a thing that interrupts people in cafés.
//!
//! # The other machine's name is never translated
//!
//! `{machine}` is what a person called the machine when they paired with it, or
//! its identity if they called it nothing. It is somebody's own data, like a
//! filename in `alo-files` and a model's name in `alo-models`, and a
//! translation of it would be an invention.

use alo_strings::Vocabulary;

/// One string a crate can say.
///
/// Re-exported because this crate's own files, and the tests that read this
/// list, name it as `crate::words::Word`.
pub use alo_strings::Word;

// ---------------------------------------------------------------------------
// What a pairing permits — [`crate::MayAskIts`].
//
// Each of these goes in a list a person reads before agreeing to anything, and
// again in the list of pairings they can see afterwards. They are phrases
// rather than sentences, because they appear under a heading naming the other
// machine.
// ---------------------------------------------------------------------------

/// A paired machine may put questions to this one's models.
pub const MAY_ASK_ITS_MODELS: Word = Word::saying(
    "nearby.may.ask-its-models",
    "Ask questions of the models on this machine",
)
.noting(
    "Read while deciding whether to pair with another machine, and again in the list of machines \
     already paired with. \"Ask questions of\" is deliberately the whole of it: a paired machine \
     may ask, and may never act. The models are this machine's own — the reader is the person \
     being asked to share them.",
);

/// A paired machine may reach a workspace this one serves.
pub const MAY_REACH_ITS_WORKSPACE: Word = Word::saying(
    "nearby.may.reach-its-workspace",
    "Reach the workspace this machine serves",
)
.noting(
    "Read in the same list as \"nearby.may.ask-its-models\". A workspace here is the self-hosted \
     one an office runs — mail, documents, calendar — not a window or a desktop. The reader is \
     the person being asked to share it.",
);

// ---------------------------------------------------------------------------
// Why there is no pairing — [`crate::NotPaired`].
//
// Somebody reads these having just been told they cannot pair with something.
// Each one says what is true rather than what went wrong, because in three of
// the four cases nothing did. The fifth is read on the other side of the
// corridor, by whoever a machine without a pairing asked for something.
// ---------------------------------------------------------------------------

/// One machine's person has agreed and the other's has not.
pub const BOTH_MACHINES_HAVE_NOT_AGREED: Word = Word::saying(
    "nearby.not-paired.both-have-not-agreed",
    "Both machines have to agree. Confirm this on the other machine as well.",
)
.noting(
    "The commonest thing that happens here, and usually not a refusal at all: somebody has agreed \
     on one machine and has not yet walked over to the other. It deliberately does not say that \
     anybody refused — the machine cannot tell the difference between a refusal and an answer \
     that has not come yet, and guessing wrong would accuse somebody. Two sentences; the second \
     is what to do.",
);

/// A machine proposing to pair with itself.
pub const A_MACHINE_CANNOT_PAIR_WITH_ITSELF: Word = Word::saying(
    "nearby.not-paired.with-itself",
    "This is the machine you are using.",
)
.noting(
    "Said when somebody picks their own machine out of a list of machines on the network. It is a \
     statement of fact rather than a refusal, because that is what it is: there is nothing to \
     pair, and nothing has gone wrong.",
);

/// A pairing that would permit nothing.
pub const A_PAIRING_HAS_TO_PERMIT_SOMETHING: Word = Word::saying(
    "nearby.not-paired.permits-nothing",
    "Choose at least one thing the other machine may ask for.",
)
.noting(
    "Said when somebody has cleared every item in the list of what a pairing would permit. It \
     asks for what is missing rather than describing the state, because the reader is in front of \
     that list and can fix it.",
);

/// A pairing that does not end.
pub const A_PAIRING_HAS_TO_END: Word = Word::saying(
    "nearby.not-paired.has-to-end",
    "Choose how long this pairing lasts. Pairings always end, and can be renewed.",
)
.noting(
    "Covers a pairing asked for with no duration and one asked for with too long a duration — one \
     sentence for both, because what the reader has to do is the same and the second sentence is \
     the reason. \"Always\" is doing real work: it says this is how the product is rather than a \
     limit somebody set, which is what stops the reader looking for a setting that turns it off. \
     Nothing here can turn it off.",
);

/// This machine is not paired with the one that asked it for something.
pub const NOT_PAIRED_WITH_THE_ONE_THAT_ASKED: Word = Word::saying(
    "nearby.not-paired.not-with-the-one-that-asked",
    "This machine is not paired with the one that asked, so nothing it asked for was considered.",
)
.noting(
    "Said on the machine that was asked, about a machine that asked it to do something without a \
     pairing standing between them: one that was only seen on the network, one whose pairing has \
     run out, or one whose pairing was undone. It deliberately does not say which, because saying \
     which would tell the asker how to become paired. \"Was considered\" is the whole of it: no \
     grant was looked at and nobody on this machine was asked anything.",
);

/// The two machines could not agree a key, or this machine's part of the
/// pairing was not the one it offered.
pub const START_THE_PAIRING_AGAIN: Word = Word::saying(
    "nearby.not-paired.start-again",
    "The two machines could not agree a key between them. Start the pairing again.",
)
.noting(
    "Said when the key agreement underneath a pairing fails, which does not happen in the \
     ordinary course of things: the other machine's part did not arrive, arrived twice, or was \
     this machine's own reflected back. Nothing was paired. The second sentence is what to do, \
     and there is nothing else to do.",
);

// ---------------------------------------------------------------------------
// Why a message was not taken as a paired machine's — [`crate::NotProven`].
//
// Read on the machine that was asked, about something that arrived naming a
// paired machine and did not prove it. Nothing it asked for was considered,
// and each sentence says so.
// ---------------------------------------------------------------------------

/// Something arrived naming a paired machine and could not prove it came from
/// there.
pub const NOT_FROM_THE_MACHINE_IT_NAMES: Word = Word::saying(
    "nearby.not-proven.not-from-the-machine-it-names",
    "Something arrived naming a paired machine but could not prove it came from there, so \
     nothing it asked for was considered.",
)
.noting(
    "Said on the machine that was asked, when a message names a machine this one is paired \
     with but was not made with the key that pairing holds — or was made for a different \
     machine. \"Could not prove\" is the whole of it: it may be a stranger presenting a paired \
     machine's identity, and the sentence does not say so because the machine cannot know. \
     \"Was considered\" as in \"nearby.not-paired.not-with-the-one-that-asked\".",
);

/// A proof that has been accepted already.
pub const A_PROOF_ALREADY_USED: Word = Word::saying(
    "nearby.not-proven.already-used",
    "The same message from a paired machine arrived a second time, so the second was not \
     considered.",
)
.noting(
    "Said on the machine that was asked, when a message it already accepted arrives again — \
     which is what somebody replaying a recording off the network looks like, and also what an \
     honest retry of the identical bytes looks like. The sentence describes the fact and blames \
     nobody.",
);

/// A proof from a moment too far from this machine's.
pub const A_PROOF_FROM_ANOTHER_MOMENT: Word = Word::saying(
    "nearby.not-proven.from-another-moment",
    "A message from a paired machine was stamped with a time more than two minutes from this \
     machine's, so it was not considered. Check both machines' clocks.",
)
.noting(
    "Said on the machine that was asked, when the moment in a message is further from this \
     machine's clock than a proof may be. \"Two minutes\" is the window the code uses and must \
     survive translation as a number; the second sentence is what to do, and is the one \
     refusal here a person can fix.",
);

// ---------------------------------------------------------------------------
// Why a proposal did not go through — [`crate::NotProposed`].
//
// Read on the machine whose person proposed, or whose person confirmed, about
// something the wire refused. Nobody at the machine that refused was shown
// anything, and several of these say so, because the reader's next question
// is whether the other person saw it.
// ---------------------------------------------------------------------------

/// The proposal named a machine other than the one it was on.
pub const A_PROPOSAL_FOR_ANOTHER_MACHINE: Word = Word::saying(
    "nearby.not-proposed.for-another-machine",
    "The proposal named a different machine from the one it reached, so nobody there was shown \
     it. Choose the machine again.",
)
.noting(
    "Said on the machine that proposed, when the machine at the other end says the proposal \
     named some other machine — the address it was found at now belongs to a different machine, \
     usually. The second sentence is what to do.",
);

/// The proposal came from an address the other machine has not seen this one
/// at.
pub const A_PROPOSAL_FROM_AN_ADDRESS_NOT_SEEN: Word = Word::saying(
    "nearby.not-proposed.from-an-address-not-seen",
    "The other machine has not seen this one on the network, so nobody there was shown the \
     proposal. Make sure both machines can see each other, then try again.",
)
.noting(
    "Said on the machine that proposed, when the other machine refuses because its own search \
     of the network never found this machine at the address the proposal came from. Nothing is \
     wrong with either machine; the second sentence is what to do.",
);

/// A proposal between these two machines is already waiting.
pub const A_PROPOSAL_IS_ALREADY_WAITING: Word = Word::saying(
    "nearby.not-proposed.already-waiting",
    "A proposal between these two machines is already waiting. Answer that one, or let it \
     lapse.",
)
.noting(
    "Said when somebody proposes to a machine a proposal is already waiting with, in either \
     direction. \"Lapse\" is the word for a proposal that nobody answered and that goes away \
     on its own after a stated time.",
);

/// Nobody at the other machine to show the proposal to.
pub const NOBODY_AT_THE_OTHER_MACHINE: Word = Word::saying(
    "nearby.not-proposed.nobody-at-the-other-machine",
    "Nobody is at the other machine to show the proposal to.",
)
.noting(
    "Said on the machine that proposed, when the other machine could not put the proposal in \
     front of a person — nobody signed in, usually. A statement of fact rather than a refusal.",
);

/// No proposal is waiting between these two machines.
pub const NO_PROPOSAL_IS_WAITING: Word = Word::saying(
    "nearby.not-proposed.nothing-waiting",
    "No proposal is waiting between these two machines. Start the pairing again.",
)
.noting(
    "Said when somebody confirms a proposal that is no longer there: it lapsed, was withdrawn, \
     or was already completed. The second sentence is what to do.",
);

/// The other machine has not answered yet, so there is no code to confirm.
pub const THE_OTHER_MACHINE_HAS_NOT_ANSWERED_YET: Word = Word::saying(
    "nearby.not-proposed.not-answered-yet",
    "The other machine has not answered yet, so there is no code to confirm.",
)
.noting(
    "Said on the machine that proposed, when its person tries to confirm before the other \
     machine's answer has arrived. The code is the six digits both people compare, and it \
     cannot exist before the answer.",
);

/// A confirmation arrived that was not from the machine being paired with.
pub const A_CONFIRMATION_NOT_FROM_THAT_MACHINE: Word = Word::saying(
    "nearby.not-proposed.not-confirmed-by-that-machine",
    "A confirmation arrived that did not come from the machine being paired with, so it was \
     not counted.",
)
.noting(
    "Said when something on the network sent a confirmation it could not have made without \
     being the other machine. The sentence describes the fact and blames nobody, because the \
     machine cannot know who sent it.",
);

/// The other machine did not accept the proposal, for a reason it wrote in
/// a word this machine does not have.
pub const THE_OTHER_MACHINE_DID_NOT_ACCEPT: Word = Word::saying(
    "nearby.not-proposed.not-accepted-there",
    "The other machine did not accept the proposal. Start the pairing again.",
)
.noting(
    "Said on the machine that proposed, when the other machine refused for a reason this one \
     has no sentence for — a newer version with a refusal this one does not know. The second \
     sentence is what to do.",
);

/// The other machine could not be reached, or what it said could not be
/// read.
pub const THE_OTHER_MACHINE_COULD_NOT_BE_REACHED: Word = Word::saying(
    "nearby.not-proposed.could-not-be-reached",
    "The other machine could not be reached, or did not answer as an alo machine would. Try \
     again.",
)
.noting(
    "Said when the connection to the other machine failed, timed out, or answered with \
     something that is not this product's reply. One sentence for all three, because what the \
     reader does is the same. \"alo\" is the product's name and is not translated.",
);

/// Everything this crate can say.
pub const EVERY_WORD: [Word; 20] = [
    MAY_ASK_ITS_MODELS,
    MAY_REACH_ITS_WORKSPACE,
    BOTH_MACHINES_HAVE_NOT_AGREED,
    A_MACHINE_CANNOT_PAIR_WITH_ITSELF,
    A_PAIRING_HAS_TO_PERMIT_SOMETHING,
    A_PAIRING_HAS_TO_END,
    NOT_PAIRED_WITH_THE_ONE_THAT_ASKED,
    START_THE_PAIRING_AGAIN,
    NOT_FROM_THE_MACHINE_IT_NAMES,
    A_PROOF_ALREADY_USED,
    A_PROOF_FROM_ANOTHER_MOMENT,
    A_PROPOSAL_FOR_ANOTHER_MACHINE,
    A_PROPOSAL_FROM_AN_ADDRESS_NOT_SEEN,
    A_PROPOSAL_IS_ALREADY_WAITING,
    NOBODY_AT_THE_OTHER_MACHINE,
    NO_PROPOSAL_IS_WAITING,
    THE_OTHER_MACHINE_HAS_NOT_ANSWERED_YET,
    A_CONFIRMATION_NOT_FROM_THAT_MACHINE,
    THE_OTHER_MACHINE_DID_NOT_ACCEPT,
    THE_OTHER_MACHINE_COULD_NOT_BE_REACHED,
];

/// Why this crate's list could not be declared.
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
pub fn nearby_words() -> Result<Vocabulary, WordsError> {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary)?;
    Ok(vocabulary)
}

/// Put everything this crate can say into an existing vocabulary.
///
/// # Errors
///
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
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::{EVERY_WORD, WordsError, declare_into, nearby_words};
    use alo_strings::{Key, Vocabulary};

    /// Every key in this file is a key, checked here because a key written in
    /// this file cannot arrive from anywhere else.
    #[test]
    fn every_key_is_a_key() {
        for word in EVERY_WORD {
            assert_eq!(Key::named(word.named()), Ok(word.key()), "{}", word.named());
        }
    }

    /// A key names one string.
    #[test]
    fn the_list_declares_into_a_vocabulary_once() {
        assert!(nearby_words().is_ok());
        let mut vocabulary = Vocabulary::empty();
        declare_into(&mut vocabulary).unwrap();
        let again = declare_into(&mut vocabulary).unwrap_err();
        assert!(matches!(again, WordsError::List(_)), "{again}");
    }

    /// **Every string here carries a note.** A translator with no product in
    /// front of them cannot tell that *ask* is the whole of what a pairing
    /// permits, or that *both machines have to agree* is usually not a refusal.
    #[test]
    fn every_string_carries_a_note_for_whoever_translates_it() {
        for word in EVERY_WORD {
            assert!(word.note().is_some(), "{}", word.named());
        }
    }

    /// Every key is under this crate's own area, so nothing here can collide
    /// with another crate's list by accident.
    #[test]
    fn every_key_is_this_crates_own() {
        for word in EVERY_WORD {
            assert!(
                word.named().starts_with("nearby."),
                "{} is not under this crate's area",
                word.named()
            );
        }
    }
}
