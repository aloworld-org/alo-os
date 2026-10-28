//! What a task window says about a reply a person sent from its reply box.
//!
//! One sentence today, and it exists because of what the machine could not do:
//! **no road carries a reply to the agent yet.** `alo-agent` takes one request
//! per turn and keeps nothing between turns — *a request is one line because a
//! turn is one request; anything longer is a conversation and this is not one*,
//! in its own words — so a reply has nowhere to go that would let alo know what
//! it answers. `crate::what_becomes_of_a_reply` holds the account and the
//! question it leaves for the owner.

use alo_strings::{Vocabulary, Word};

/// What the conversation says straight after a reply that was not sent.
pub const A_REPLY_DOES_NOT_REACH_ALO_YET: Word = Word::saying(
    "the-reply.does-not-reach-alo-yet",
    "alo did not receive this. Replies in a task window cannot reach alo yet; to ask again, start \
     a new request from the Dock.",
)
.noting(
    "Said in a task window's conversation directly beneath something the person typed in its \
     reply box and sent with Enter. Nothing was sent anywhere and nothing on the computer changed; \
     this must never read as \"alo is thinking\", \"queued\" or \"will answer later\", because it \
     will not. \"The Dock\" is the bar at the bottom of the screen where a person types a request \
     to alo; use the name the Dock has in your language. \"alo\" is the product's name and stays \
     as it is, in lower case.",
);

/// Every string this file declares, in the order a translator meets them.
pub const THE_REPLY_BOXS_WORDS: [Word; 1] = [A_REPLY_DOES_NOT_REACH_ALO_YET];

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
