//! What a settings file that did not read says, where there is nobody to ask
//! for words.
//!
//! Six of the things in this crate are read back out of a settings file through
//! `serde(try_from = …)` — a colour, a display's name, a rotation, a schedule, a
//! text size and a time of day — and every one of them is checked again on the
//! way in, because a settings file is a thing a person edits. That check refuses
//! in exactly the words a settings panel would use, and those words are now
//! [`crate::words`]' rather than this crate's.
//!
//! **A deserialiser has no `Strings` and never will.** It is handed a value and
//! a format, not the language the person in front of the machine reads, and
//! `serde` needs the error it answers with to have a `Display`. A sentence
//! composed there would be English that nothing could translate — which is
//! precisely what the ten error types in this crate losing their `Display` was
//! for.
//!
//! So what a refusal writes at that one point is the **key** of the string,
//! rather than the string. Whoever reports a settings file that did not read
//! looks the key up and shows the same words a settings panel shows for the same
//! refusal — one rendering, in the reader's own language, rather than an English
//! line in a log beside a translated line on a screen.
//!
//! `alo-shortcuts` met this first, in `Chord`'s deserialiser, and answered it
//! with a private type of its own. This one is public and shared by six callers:
//! six copies of the same four lines would have been the third pattern this
//! change exists to stop.

use std::fmt;

use alo_strings::{Key, Word};

/// A settings file that did not read, named by the key of the refusal.
///
/// Deliberately says nothing more. It is not the sentence a person is shown —
/// [`crate::ColourError::said`] and the rest are that, in the reader's own
/// language — and turning this into one is done where a `Strings` exists.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotRead {
    /// The string the refusal would have been said with.
    key: Key,
}

impl NotRead {
    /// The refusal this word is for.
    pub(crate) fn about(word: Word) -> Self {
        Self { key: word.key() }
    }

    /// Which string says what was wrong, for whoever looks it up.
    #[must_use]
    pub fn key(&self) -> &Key {
        &self.key
    }
}

impl fmt::Display for NotRead {
    /// The key, and nothing else. A sentence here would be untranslatable
    /// English; see the module documentation.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.key.as_str())
    }
}

impl std::error::Error for NotRead {}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::words;

    /// What it writes is a key, and a key that something declares — so whoever
    /// reads it can look it up and get the sentence a person would have been
    /// shown.
    #[test]
    fn what_it_writes_is_a_key_something_declares() {
        let refused = NotRead::about(words::TOO_QUICK);
        assert_eq!(refused.to_string(), "appearance.rotating.too-quick");
        assert_eq!(refused.key(), &words::TOO_QUICK.key());
        assert!(
            words::appearance_words()
                .unwrap()
                .phrase(refused.key())
                .is_some()
        );
    }

    /// **It is not a sentence, and it must not become one.** A message composed
    /// here would be English in a place where nothing can ask what language the
    /// person reads.
    #[test]
    fn it_says_nothing_but_the_key() {
        for word in words::EVERY_WORD {
            let written = NotRead::about(word).to_string();
            assert_eq!(written, word.key().as_str(), "{}", word.named());
            assert!(!written.contains(' '), "{written} is a sentence");
        }
    }
}
