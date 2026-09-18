//! What `displays.toml` says when it did not read, where there is nobody to ask
//! for words.
//!
//! Four of the things in this crate come back out of a settings file through
//! `serde(try_from = …)` — a screen's name, a size, a place and a whole
//! arrangement — and every one of them is checked again on the way in, because
//! a settings file is a thing a person edits. `alo-appearance` met this first
//! and its reasoning is the whole of the argument: a deserialiser is handed a
//! value and a format, never the language the person in front of the machine
//! reads, so a sentence composed there would be English nothing could
//! translate.
//!
//! What a refusal writes at that one point is therefore the **key** of the
//! string rather than the string, and whoever reports a file that did not read
//! looks the key up and shows the same words a settings panel shows for the
//! same refusal.

use std::fmt;

use alo_strings::{Key, Word};

/// A settings file that did not read, named by the key of the refusal.
///
/// Deliberately says nothing more. The sentence a person is shown is
/// [`crate::IdentityError::said`] and the rest, in the reader's own language.
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
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.key.as_str())
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

    /// What it writes is a key, and a key this crate declares — so whoever
    /// reads it can look it up and get the sentence a person would have seen.
    #[test]
    fn what_it_writes_is_a_key_this_crate_declares() {
        let refused = NotRead::about(words::NO_MAIN_SCREEN);
        assert_eq!(refused.to_string(), "displays.no-main-screen");
        assert_eq!(refused.key(), &words::NO_MAIN_SCREEN.key());
        assert!(
            words::display_words()
                .unwrap()
                .phrase(refused.key())
                .is_some()
        );
    }

    /// **It is not a sentence, and it must not become one.**
    #[test]
    fn it_says_nothing_but_the_key() {
        for word in words::EVERY_WORD {
            let written = NotRead::about(word).to_string();
            assert_eq!(written, word.key().as_str(), "{}", word.named());
            assert!(!written.contains(' '), "{written} is a sentence");
        }
    }
}
