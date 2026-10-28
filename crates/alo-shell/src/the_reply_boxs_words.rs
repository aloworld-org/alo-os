//! What a task window says about a reply a person sent from its reply box.
//!
//! **A reply reaches alo, with the conversation so far** (ADR 0136). These are
//! the three things the window says when it does not go at once: it is held
//! until alo has answered, or until the person has answered a change above, or
//! it cannot go at all because the conversation has grown too long for one
//! question to carry. `crate::what_becomes_of_a_reply` decides which.

use alo_strings::{Vocabulary, Word};

/// Said under a held reply while alo is still answering.
pub const HELD_UNTIL_ALO_ANSWERS: Word = Word::saying(
    "the-reply.held-until-alo-answers",
    "Waiting to be sent. This goes to alo, with the conversation so far, as soon as alo has \
     answered. To change or delete it, press Backspace in the empty reply box.",
)
.noting(
    "Shown in a task window directly under something the person typed and sent while alo was \
     still answering their previous message. It has not been sent yet; it is sent by itself, \
     once alo's answer arrives. \"Backspace\" is the key that deletes the character before the \
     cursor; use the name a keyboard in your language prints on it. \"alo\" is the product's \
     name and stays as it is, in lower case.",
);

/// Said under a held reply while a change above waits for the person.
pub const HELD_UNTIL_YOU_ANSWER: Word = Word::saying(
    "the-reply.held-until-you-answer",
    "Waiting to be sent. This goes to alo, with the conversation so far, once you have allowed or \
     denied the change above. To change or delete it, press Backspace in the empty reply box.",
)
.noting(
    "Shown in a task window directly under something the person typed and sent while a proposed \
     change above still shows \"Allow once\" and \"Deny\". It has not been sent yet; it is sent \
     by itself once they press one of the two. \"Allowed\" and \"denied\" refer to those two \
     buttons; use the same words they use in your language. \"Backspace\" is the key that \
     deletes the character before the cursor. \"alo\" is the product's name and stays as it is, \
     in lower case.",
);

/// Said beneath a reply that could not be sent: the conversation is too long
/// for one question to carry.
pub const TOO_LONG_TO_SEND: Word = Word::saying(
    "the-reply.too-long-to-send",
    "alo did not receive this, because this conversation is now too long to send to alo in one \
     go. Start a new one from the Dock to continue.",
)
.noting(
    "Said in a task window's conversation beneath something the person sent. Each reply is sent \
     together with the whole conversation before it, and this one has grown past what this \
     computer can send in one request, so nothing was sent and nothing was left out. \"The Dock\" \
     is the bar at the bottom of the screen where a person types a request to alo; use the name \
     the Dock has in your language. \"alo\" is the product's name and stays as it is, in lower \
     case.",
);

/// Every string this file declares, in the order a translator meets them.
pub const THE_REPLY_BOXS_WORDS: [Word; 3] = [
    HELD_UNTIL_ALO_ANSWERS,
    HELD_UNTIL_YOU_ANSWER,
    TOO_LONG_TO_SEND,
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
