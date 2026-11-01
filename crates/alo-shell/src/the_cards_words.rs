//! The two answers on an approval card, and why they are worded here.
//!
//! The owner's words, recorded in `alo_tasks::permissions` and dated
//! 2026-10-12: *when alo needs permission, show the specific proposed action
//! in the conversation with **Allow once** and **Deny**. Broader access should
//! be a separate, deliberate choice.*
//! `docs/design/the-ask-alo-task-window.md` draws them in a row of two, 144
//! and 104 wide, both 48 high — *the drawn form of equally reachable*.
//!
//! # These are not `alo_approving`'s two, and that is worth knowing
//!
//! `alo_approving::words::{APPROVE, NO}` say **Approve** and **No** for the
//! full-screen approval surface, which is the other road to the same decision:
//! it runs inside the turn machinery, and `crate::approval_screen` is its
//! shell. **So alo OS now words one action two ways**, and that is a real
//! wrinkle rather than a decision taken twice.
//!
//! It is written here rather than resolved because resolving it is a choice
//! about both surfaces and belongs to whoever owns them:
//!
//! - **Keeping both** is defensible while the two surfaces are different
//!   shapes — a card in a conversation and a question over the whole screen.
//! - **Converging** is defensible too, and if it happens these are the ones to
//!   keep, for the reason below.
//!
//! **Why these are the better pair.** *Allow once* carries the rule in the
//! label. `alo_tasks::permissions` names the failure the whole surface is
//! shaped to refuse — *they allowed it once, so stop asking* — and notes that
//! it reads as kindness, which is how an approval becomes a session. A button
//! reading *Approve* leaves all of that to the sentence above it. One reading
//! *Allow once* says it where somebody is about to press.

use alo_strings::{Vocabulary, Word};

/// The answer that lets this one action go ahead.
pub const ALLOW_ONCE: Word = Word::saying("the-card.allow-once", "Allow once").noting(
    "One of the two answers on an approval card, read directly beneath a sentence saying exactly \
     what the agent will do — the agent being the assistant built into alo OS and not a person. \
     **Once is the whole meaning and must survive translation**: it allows that one action and \
     nothing else, it is not a grant, a permission, a session or a setting, and alo will ask \
     again the next time. A translation reading as \"always allow\", \"trust\" or \"allow\" on \
     its own says something alo OS does not offer. Short because the sentence above carries the \
     rest, and because it sits in a row 144 wide beside its pair.",
);

/// The answer that stops it.
pub const DENY: Word = Word::saying("the-card.deny", "Deny").noting(
    "The other of the two answers on an approval card, beneath the same sentence. It is a \
     complete answer: nothing is postponed, nothing is undone and no reason is asked for or \
     recorded, so a translation should not soften it into \"not now\", \"later\" or \"cancel\". \
     It is exactly as wide as its pair on the screen, 144: both are equally reachable and \
     neither is behind anything.",
);

/// The line at the top of an approval card, above the action.
pub const ALO_IS_ASKING_TO: Word = Word::saying("the-card.alo-is-asking-to", "alo is asking to")
    .noting(
        "alo is the assistant built into alo OS, not a person. The small line at the top of an \
         approval card, beside alo's mark; the line under it, in large type, is the action — \
         for example \"rename march.pdf to invoice-march.pdf\" — so this reads as the start of \
         that sentence. Lower case because alo is always written so. A language whose grammar \
         cannot leave the sentence open may close it (\"alo asks for this:\").",
    );

/// What allowing once means, beside the two answers.
pub const ALLOWS_THIS_ONE_ACTION: Word = Word::saying(
    "the-card.allows-this-one-action",
    "Allows this one action. alo asks again next time.",
)
.noting(
    "Beside Allow once and Deny on an approval card. It says what Allow once does and does not \
     do: it is not a permission that lasts, and alo will ask again the next time it wants to do \
     anything like this. alo is the assistant built into alo OS, not a person.",
);

/// What the conversation says once an allowed change was carried out.
pub const IT_WAS_DONE: Word = Word::saying("the-card.it-was-done", "Done.").noting(
    "Said by the agent in the conversation straight after the person pressed Allow once and the \
     machine carried the change out. The change itself is the sentence on the card just above, \
     so this says only that it happened. Say it only as a completed fact — not \"working on it\", \
     not \"will do\".",
);

/// What the conversation says once an allowed change was carried out and
/// nothing was kept to undo it with (ADR 0045's third term: the change runs,
/// *and says it is not undoable*).
pub const IT_WAS_DONE_AND_CANNOT_BE_UNDONE: Word = Word::saying(
    "the-card.it-was-done-and-cannot-be-undone",
    "Done. This change cannot be undone: alo could not keep a copy of your files from before it.",
)
.noting(
    "Said by the agent in the conversation straight after the person pressed Allow once, the \
     machine carried the change out, and it could not first keep a copy of the person's files to \
     go back to. The change is the sentence on the card just above. This is a plain fact, not a \
     risk: it must never soften into \"may not be undoable\" or \"might not\", because the person \
     will act on it. \"alo\" is the product's name and stays as it is.",
);

/// What the conversation says once the person said no.
pub const YOU_SAID_NO: Word = Word::saying(
    "the-card.you-said-no",
    "You said no, so nothing was changed.",
)
.noting(
    "Said by the agent in the conversation straight after the person pressed Deny and the \
         machine recorded it. Both halves matter: it was the person's choice, and nothing on the \
         computer changed because of the request. It is not an error and should not read as one, \
         and it does not ask why.",
);

/// What the conversation says when the machine gave no answer at all.
pub const NO_WORD_BACK: Word = Word::saying(
    "the-card.no-word-back",
    "This computer did not say what happened to that change.",
)
.noting(
    "Said in the conversation when the person answered and the part of alo OS that carries changes \
     out never replied. It must not suggest the change was or was not made: nobody knows, and that \
     is the whole message.",
);

/// What the conversation says when alo stopped without an answer or a change.
///
/// Beside the card's own outcomes because it is the same kind of line: the last
/// one in a conversation, saying how a turn ended. Without it a turn that gave
/// up left the person's request as the last thing said, and the window read
/// *alo is working* for as long as it stayed open (the fifth Demo 1 walk,
/// 2026-10-27).
pub const IT_STOPPED_WITHOUT_FINISHING: Word = Word::saying(
    "the-card.it-stopped-without-finishing",
    "alo stopped without finishing this.",
)
.noting(
    "Said by the agent in the conversation when it stopped without an answer and without anything \
     to approve: it could not get further, or this computer could not be reached. The reason is \
     shown beside the conversation, so this says only that it stopped. It must not claim anything \
     was or was not changed, and it is not an apology.",
);

/// Every string this file declares, in the order a translator meets them.
pub const THE_CARDS_WORDS: [Word; 9] = [
    ALO_IS_ASKING_TO,
    ALLOW_ONCE,
    DENY,
    ALLOWS_THIS_ONE_ACTION,
    IT_WAS_DONE,
    IT_WAS_DONE_AND_CANNOT_BE_UNDONE,
    YOU_SAID_NO,
    NO_WORD_BACK,
    IT_STOPPED_WITHOUT_FINISHING,
];

/// Why this file's words could not be declared.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TheCardsWordsError {
    /// A word that is not a phrase.
    #[error(transparent)]
    Word(#[from] alo_strings::WordError),
    /// A key the vocabulary already has.
    #[error(transparent)]
    List(#[from] alo_strings::VocabularyError),
}

/// Add this surface's wording atomically, before any translation is loaded.
///
/// # Errors
/// [`TheCardsWordsError`] for a malformed phrase or a key already taken.
pub fn declare_the_cards_words(vocabulary: &mut Vocabulary) -> Result<(), TheCardsWordsError> {
    // **Prepared and swapped**, which is every other `declare_` in this crate:
    // a declaration that failed halfway would leave a vocabulary holding one
    // answer and not the other, and the missing one would be found on a screen
    // with a change waiting behind it.
    let mut prepared = vocabulary.clone();
    for word in THE_CARDS_WORDS {
        prepared.says(word.phrase()?)?;
    }
    *vocabulary = prepared;
    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// Both declare, which the shell's boot relies on.
    #[test]
    fn every_word_here_can_be_declared() {
        let mut vocabulary = Vocabulary::empty();
        declare_the_cards_words(&mut vocabulary).unwrap();
        for word in THE_CARDS_WORDS {
            assert!(vocabulary.phrase(&word.key()).is_some(), "{word:?}");
        }
    }

    /// Declaring twice is refused rather than silently replacing, and the
    /// first failure leaves the vocabulary exactly as it was.
    #[test]
    fn declaring_the_same_words_twice_is_refused_and_changes_nothing() {
        let mut vocabulary = Vocabulary::empty();
        declare_the_cards_words(&mut vocabulary).unwrap();
        let before = vocabulary.clone();
        assert!(declare_the_cards_words(&mut vocabulary).is_err());
        assert_eq!(vocabulary.how_many(), before.how_many());
    }

    /// **The two answers are two different keys.** One key behind both would
    /// put the same word on a row where a person chooses between them.
    #[test]
    fn the_two_answers_are_not_one_word() {
        assert_ne!(ALLOW_ONCE.key(), DENY.key());
        assert_ne!(ALLOW_ONCE.says(), DENY.says());
    }
}
