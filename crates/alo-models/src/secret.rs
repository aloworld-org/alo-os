//! A key as it was just typed, for the length of one call.
//!
//! [`provider`](crate::provider) holds a *reference* to a key in the keyring
//! and never a key, so that a settings structure written to disk, logged or
//! attached to a support bundle cannot leak one. This is the other side of that
//! rule, and it exists because of a single moment: **a provider is tested
//! before it is saved**, so at that moment the key is not in the keyring yet.
//! It is in the person's hands, and it has to reach one request without
//! becoming anything that outlives it.
//!
//! So the key goes in and does not come out. There is no accessor, no
//! [`Display`](std::fmt::Display), no `Serialize`, no `Clone`, and the
//! [`Debug`](std::fmt::Debug) is written by hand to say nothing. The one thing
//! that is done with a [`Secret`] is put it on one request to the one address
//! the policy was asked about — see [`crate::trying`].
//!
//! **And a request from another crate is handed to the key, never the key to
//! the crate.** `alo-asking` puts questions to a provider and needs the same
//! header on the same kind of request; giving it the bytes would mean a
//! `pub fn bearer` and the end of everything above. So [`Secret::carried_by`]
//! takes the request and gives it back with the key on it, and what a caller
//! outside this crate can do with a key is *send* it somewhere — never read it,
//! log it, or keep it. What that caller owes, and what no type here can check,
//! is that the request is the one the policy answered about.
//!
//! **What this does not claim.** The bytes are not scrubbed from memory when
//! the value is dropped: doing that honestly needs either `unsafe` or a
//! dependency, and the workspace forbids the first. Claiming it without doing
//! it would be worse than not claiming it, so the promise here is the narrow
//! one that is actually kept — a key is never written down, never rendered, and
//! never travels anywhere but to the provider it belongs to.

use std::fmt;

use alo_strings::{Filling, Said, Strings};
use ureq::RequestBuilder;

use crate::words;

/// Why something typed into the key field is not a key.
///
/// Neither message repeats what was typed, and neither may start: a key in an
/// error is a key in a log. That is also why there is **no `Display`** (item
/// 9f): the only road to words is [`SecretError::said`], and a type in this
/// file with a `Display` is one `to_string()` away from every log line that
/// ever formats something holding one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretError {
    /// Nothing was typed. A provider that needs no key is given none at all,
    /// which is a different thing from being given an empty one.
    Blank,
    /// Something is in it that cannot be sent — a line break pasted along with
    /// the key, or a stray control character. Sending it would either be
    /// refused by the provider or, worse, split the request in two.
    NotSendable,
}

impl SecretError {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub fn word(self) -> words::Word {
        match self {
            Self::Blank => words::KEY_BLANK,
            Self::NotSendable => words::KEY_NOT_SENDABLE,
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics: a `Strings` that was never given
    /// [`crate::model_words`] answers with the key, marked. Neither sentence
    /// has a gap in it, which is deliberate — there is nothing here that could
    /// be put into one except what was typed.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

/// A key, held for one call and never written down.
pub struct Secret(String);

impl Secret {
    /// The key somebody has just typed or pasted.
    ///
    /// Whitespace around it is dropped, because a key copied out of a web page
    /// arrives with a newline on the end and *that* mistyped key is exactly
    /// what testing a provider exists to catch. Anything else unprintable is
    /// refused rather than quietly removed: silently changing a credential is
    /// how somebody spends an afternoon on a key that was right all along.
    ///
    /// # Errors
    /// [`SecretError`], saying what to do rather than what went wrong.
    pub fn typed(key: &str) -> Result<Self, SecretError> {
        let key = key.trim();
        if key.is_empty() {
            return Err(SecretError::Blank);
        }
        if key.chars().any(char::is_control) {
            return Err(SecretError::NotSendable);
        }
        Ok(Self(key.to_owned()))
    }

    /// How the key travels on the one request it is used for.
    ///
    /// `pub(crate)` on purpose, and it is the only reader of the key that
    /// exists: a key can be handed to this crate and cannot be taken back out
    /// of it. The doctest on [`Secret`]'s module asserts that this does not
    /// compile from outside.
    pub(crate) fn bearer(&self) -> String {
        format!("Bearer {}", self.0)
    }

    /// Put this key on a request, without letting it out.
    ///
    /// The whole of what a crate outside this one can do with a key. It is a
    /// request in and a request out, so the bytes never become a `String`
    /// anybody else holds, and [`bearer`](Self::bearer) stays private with its
    /// `compile_fail` doctest intact.
    ///
    /// Generic over the request's kind because the two callers differ: asking a
    /// provider what it offers is a `GET` with no body, and putting a question
    /// to one is a `POST` with one.
    #[must_use]
    pub fn carried_by<Any>(&self, request: RequestBuilder<Any>) -> RequestBuilder<Any> {
        request.header("authorization", self.bearer())
    }
}

/// Says that there is a key, and nothing about what it is.
///
/// Written by hand rather than derived, because a derived `Debug` would put the
/// key in every error report, panic message and log line that ever formats a
/// structure holding one.
impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Secret(…)")
    }
}

/// A key cannot be read back out of this crate.
///
/// Neither of these uses `?`: [`SecretError`] lost its `Display` in item 9f and
/// so is no longer a `std::error::Error`, and a pair of doctests that failed on
/// a conversion would be a pair testing the conversion.
///
/// ```compile_fail
/// let key = alo_models::Secret::typed("sk-the-real-thing").expect("a key");
/// // `bearer` is pub(crate): there is no way to get the key back.
/// let _ = key.bearer();
/// ```
///
/// The twin that passes, so the pair cannot rot into a test of a typo:
///
/// ```
/// let key = alo_models::Secret::typed("sk-the-real-thing").expect("a key");
/// assert_eq!(format!("{key:?}"), "Secret(…)");
/// ```
#[cfg(doctest)]
pub struct AKeyCannotBeReadBackOut;

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The load-bearing property, and the reason this type exists at all: no
    /// rendering of it can show the key, because nothing can read the key.
    #[test]
    fn a_key_never_appears_in_anything_that_renders_it() {
        let key = Secret::typed("sk-live-0123456789").unwrap();
        let debugged = format!("{key:?}");
        assert_eq!(debugged, "Secret(…)");
        assert!(!debugged.contains("sk-live"), "{debugged}");
    }

    /// A key copied out of a web page arrives with the line around it. That is
    /// the mistyped key this feature exists to catch, so it is fixed rather
    /// than reported — and the fix is visible on the wire, which is where
    /// `trying.rs` asserts it.
    #[test]
    fn the_line_around_a_pasted_key_is_dropped() {
        let key = Secret::typed("  sk-live-0123456789\n").unwrap();
        assert_eq!(key.bearer(), "Bearer sk-live-0123456789");
    }

    /// An empty key is not a key. A provider that needs none is given none.
    #[test]
    fn nothing_typed_is_refused_rather_than_sent_as_an_empty_key() {
        assert_eq!(Secret::typed("   ").unwrap_err(), SecretError::Blank);
        assert_eq!(Secret::typed("").unwrap_err(), SecretError::Blank);
    }

    /// A control character inside a key would either be refused by the
    /// provider or split the request in two at the header. It is refused here,
    /// and it is not silently removed: a credential quietly altered is an
    /// afternoon lost to a key that was right.
    #[test]
    fn a_key_that_cannot_be_sent_is_refused_and_not_quietly_repaired() {
        assert_eq!(
            Secret::typed("sk-live\r\nx-something: else").unwrap_err(),
            SecretError::NotSendable
        );
        assert_eq!(
            Secret::typed("sk-\u{7}live").unwrap_err(),
            SecretError::NotSendable
        );
    }

    /// The words are read by somebody who has just pasted something, so they
    /// say what to do — and neither of them repeats what was pasted.
    #[test]
    fn the_errors_say_what_to_do_without_quoting_the_key() {
        let strings = crate::testing::in_english();
        assert!(
            SecretError::Blank
                .said(&strings)
                .text()
                .contains("paste the key this provider gave you")
        );
        assert!(
            SecretError::NotSendable
                .said(&strings)
                .text()
                .contains("copy it again")
        );
        for error in [SecretError::Blank, SecretError::NotSendable] {
            assert!(!error.said(&strings).text().contains("sk-"), "{error:?}");
        }
    }

    /// **A key never reaches a sentence, in any language.** Neither of these
    /// strings has a gap, so a translation cannot invent one to put the key in
    /// — `alo-strings` refuses a gap the source does not have, and this is the
    /// test that says so about the two strings where it matters most.
    #[test]
    fn neither_sentence_has_a_gap_for_anything_to_be_put_into() {
        for word in [words::KEY_BLANK, words::KEY_NOT_SENDABLE] {
            assert!(!word.says().contains('{'), "{}", word.named());
        }
    }
}
