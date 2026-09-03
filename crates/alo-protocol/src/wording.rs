//! A sentence as it crosses the socket, and whether anybody translated it.
//!
//! Everything a client shows a person about what the daemon did is worded by
//! the daemon: a refusal made in `alo-capability`, a sentence a change was
//! proposed under, the clause saying where an answer came from. The daemon
//! holds the vocabulary, so the daemon renders — and what goes on the wire is
//! text.
//!
//! # Text alone would throw away the one thing item 9 built
//!
//! `alo_strings::Said` exists because a lookup that answered with a `String`
//! would hand a Latvian shell an English sentence with nothing anywhere knowing
//! it had happened. A socket that carried only the text would put that hole
//! back at the last boundary before a person reads the sentence, and it would
//! put it there for **every** string in the workspace at once.
//!
//! So a sentence crosses with [`CameFrom`] beside it, and a shell can mark what
//! nobody has translated in exactly the way a development build does. The three
//! states are `alo_strings::Said`'s own, read off it rather than decided here:
//! somebody translated all of this, nobody has translated it yet, or the daemon
//! asked for a string nothing declares — which is a bug in alo OS rather than a
//! translation nobody has done.
//!
//! # Nothing here is composed
//!
//! A [`Wording`] is made from a `Said` and from nothing else. There is no
//! constructor that takes a bare string, because a bare string is what somebody
//! reaches for at four in the afternoon when the vocabulary is inconvenient.

use alo_strings::Said;
use serde::{Deserialize, Serialize};

/// One sentence, ready to be shown, and where it came from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Wording {
    /// The sentence, in whatever language the daemon rendered it in.
    text: String,
    /// Whether anybody translated it.
    came_from: CameFrom,
}

/// Where a sentence that crossed the socket came from.
///
/// `alo_strings::CameFrom`'s three cases, narrowed to what a client can act on:
/// which language a translation was in is the daemon's business, and the shell
/// asked for its own.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CameFrom {
    /// Somebody translated this, all of it, including any word put into a gap.
    Translation,
    /// Nobody has translated it yet, so this is the language the code was
    /// written in. Not a bug: work not yet done, and countable as such.
    TheSource,
    /// The daemon asked for a string nothing declares, so what is in [`Wording`]
    /// is a key. A bug in alo OS, and the one case where showing it plainly is
    /// better than showing a blank line.
    NoSentence,
}

impl Wording {
    /// This sentence, as it goes on the wire.
    ///
    /// The three states are read off the `Said` rather than judged here:
    /// `is_a_bug` first, because a key that happens to have been translated is
    /// still a key.
    #[must_use]
    pub fn of(said: &Said) -> Self {
        let came_from = if said.is_a_bug() {
            CameFrom::NoSentence
        } else if said.is_translated() {
            CameFrom::Translation
        } else {
            CameFrom::TheSource
        };
        Self {
            text: said.text().to_owned(),
            came_from,
        }
    }

    /// The sentence, which is what goes on the screen.
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Where it came from.
    #[must_use]
    pub fn came_from(&self) -> CameFrom {
        self.came_from
    }

    /// Whether somebody translated this.
    ///
    /// What a shell marks in a development build, and what a release note
    /// counts — the same question `alo_strings::Said::is_translated` answers,
    /// asked on the other side of a socket.
    #[must_use]
    pub fn is_translated(&self) -> bool {
        matches!(self.came_from, CameFrom::Translation)
    }

    /// Whether this is a mistake in alo OS rather than a translation nobody has
    /// done yet.
    #[must_use]
    pub fn is_a_bug(&self) -> bool {
        matches!(self.came_from, CameFrom::NoSentence)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, translated};
    use crate::words;
    use alo_strings::{Filling, Strings, Vocabulary};

    /// One sentence this crate really declares, looked up in whatever
    /// vocabulary a test hands over.
    fn said(strings: &Strings) -> alo_strings::Said {
        strings.say(&words::NOT_READABLE.key(), &Filling::nothing())
    }

    /// **A translated sentence says it was translated**, which is the whole of
    /// why this type is not a `String`.
    #[test]
    fn a_translated_sentence_crosses_as_translated() {
        let german = translated(&[(
            words::NOT_READABLE,
            "dieser Rechner konnte diese Nachricht nicht lesen",
        )]);
        let wording = Wording::of(&said(&german));
        assert_eq!(wording.came_from(), CameFrom::Translation);
        assert!(wording.is_translated());
        assert!(!wording.is_a_bug());
        assert!(wording.text().starts_with("dieser Rechner"));
    }

    /// **English shown because nobody translated it says so**, so a shell in
    /// Latvia can mark it rather than showing it as though it were the
    /// person's own language.
    #[test]
    fn a_sentence_nobody_translated_says_that_it_is_the_source() {
        let wording = Wording::of(&said(&in_english()));
        assert_eq!(wording.came_from(), CameFrom::TheSource);
        assert!(!wording.is_translated());
        assert!(!wording.is_a_bug());
    }

    /// A key nothing declares crosses as what it is: a bug in alo OS, marked as
    /// one, rather than a sentence a person is left to make sense of.
    #[test]
    fn a_key_nothing_declares_crosses_as_a_bug() {
        let nothing = Strings::of(Vocabulary::empty());
        let wording = Wording::of(&said(&nothing));
        assert_eq!(wording.came_from(), CameFrom::NoSentence);
        assert!(wording.is_a_bug());
        assert!(!wording.is_translated());
        assert!(wording.text().contains("protocol."));
    }

    /// What is written is what is read back, so a shell and a daemon built from
    /// this crate cannot disagree about whether a person is reading their own
    /// language.
    #[test]
    fn a_sentence_reads_back_as_what_was_written() {
        for sentence in [said(&in_english()), said(&Strings::of(Vocabulary::empty()))] {
            let wording = Wording::of(&sentence);
            let written = serde_json::to_string(&wording).unwrap();
            let back: Wording = serde_json::from_str(&written).unwrap();
            assert_eq!(back, wording);
        }
    }
}
