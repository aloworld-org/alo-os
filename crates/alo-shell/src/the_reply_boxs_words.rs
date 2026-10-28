//! What a task window says about a reply a person sent from its reply box.
//!
//! **A reply reaches alo, with the conversation so far**, since ADR 0135 — so
//! these are the three things the window says when that is not the whole
//! story: the reply did not go because a change above is still waiting, or
//! because alo has not answered yet, or it went without the oldest part of a
//! long conversation. `crate::what_becomes_of_a_reply` decides which.

use alo_strings::{Vocabulary, Word};

/// Said beneath a reply that was not sent because a change is still waiting.
pub const A_REPLY_WAITS_FOR_THE_CHANGE: Word = Word::saying(
    "the-reply.waits-for-the-change",
    "alo did not receive this, because a change above is still waiting for your answer. Allow it \
     or deny it, then send this again.",
)
.noting(
    "Said in a task window's conversation directly beneath something the person typed in its \
     reply box and sent with Enter, while a proposed change above still shows \"Allow once\" and \
     \"Deny\". Nothing was sent anywhere and nothing on the computer changed; this must never read \
     as \"queued\" or \"will be sent later\", because it will not. \"Allow\" and \"deny\" refer to \
     the two buttons on that change; use the same words those buttons use in your language. \
     \"alo\" is the product's name and stays as it is, in lower case.",
);

/// Said beneath a reply that was not sent because alo has not answered yet.
pub const A_REPLY_WAITS_FOR_ALO: Word = Word::saying(
    "the-reply.waits-for-alo",
    "alo did not receive this, because alo is still working on what you asked before. Send it \
     again once alo has answered.",
)
.noting(
    "Said in a task window's conversation directly beneath something the person typed in its \
     reply box and sent with Enter, while alo is still answering their previous message. Nothing \
     was sent and nothing on the computer changed; this must never read as \"queued\", because it \
     is not. \"alo\" is the product's name and stays as it is, in lower case.",
);

/// Said in a task's activity when a long conversation was sent in part.
pub const PART_OF_THE_CONVERSATION_WAS_NOT_SENT: Word = Word::saying(
    "the-reply.part-was-not-sent",
    "The oldest part of this conversation was not sent to alo with your reply, because the \
     conversation is longer than alo sends at once. alo was told that part is missing.",
)
.noting(
    "Shown in a task window's activity after the person sent a reply. Each reply is sent to alo \
     together with the conversation before it, so alo knows what the reply refers to; when the \
     conversation is very long, its beginning is left out, and alo is told so. If the provider \
     answering is on the internet, the part that was sent left this computer. \"alo\" is the \
     product's name and stays as it is, in lower case.",
);

/// Every string this file declares, in the order a translator meets them.
pub const THE_REPLY_BOXS_WORDS: [Word; 3] = [
    A_REPLY_WAITS_FOR_THE_CHANGE,
    A_REPLY_WAITS_FOR_ALO,
    PART_OF_THE_CONVERSATION_WAS_NOT_SENT,
];

/// Why this file's words could not be declared.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum TheReplyBoxsWordsError {
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
/// [`TheReplyBoxsWordsError`] for a malformed phrase or a key already taken.
pub fn declare_the_reply_boxs_words(
    vocabulary: &mut Vocabulary,
) -> Result<(), TheReplyBoxsWordsError> {
    // Prepared and swapped, for `crate::the_cards_words`' reason: a declaration
    // that failed halfway must leave the vocabulary as it was.
    let mut prepared = vocabulary.clone();
    for word in THE_REPLY_BOXS_WORDS {
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

    /// Every word declares, which the shell's boot relies on.
    #[test]
    fn every_word_here_can_be_declared() {
        let mut vocabulary = Vocabulary::empty();
        declare_the_reply_boxs_words(&mut vocabulary).unwrap();
        for word in THE_REPLY_BOXS_WORDS {
            assert!(vocabulary.phrase(&word.key()).is_some(), "{word:?}");
        }
    }

    /// Declaring twice is refused and changes nothing.
    #[test]
    fn declaring_the_same_words_twice_is_refused_and_changes_nothing() {
        let mut vocabulary = Vocabulary::empty();
        declare_the_reply_boxs_words(&mut vocabulary).unwrap();
        let before = vocabulary.how_many();
        assert!(declare_the_reply_boxs_words(&mut vocabulary).is_err());
        assert_eq!(vocabulary.how_many(), before);
    }
}
