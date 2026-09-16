//! A proxy's password: where it is kept, and the one moment it is held.
//!
//! [ADR 0022](../../../docs/decisions/0022-where-a-providers-key-is-kept.md)
//! decided where a credential lives on an alo OS machine — the keyring, reached
//! over the person's own session bus — and this file is that decision applied
//! to the one credential a proxy has. It is the same split
//! `alo_models::SecretRef` and `alo_models::Secret` are:
//!
//! | | |
//! |---|---|
//! | [`WhereThePasswordIs`] | a name in the keyring, and **what the setting holds** |
//! | [`Password`] | the password itself, held for the length of one road out |
//!
//! **The setting cannot hold a password, because there is no field for one.**
//! [`crate::ProxyAddress`] carries a [`WhereThePasswordIs`] and nothing else, so
//! a proxy setting written to a file, read back, shown in a settings panel or
//! attached to whatever somebody sends to their administrator carries a name to
//! look up and never a credential. That is a property of the shapes rather than
//! of anybody's care, and `crate::setting` holds it with a test.
//!
//! **A password goes in and does not come out** — no accessor, no
//! [`Display`](std::fmt::Display), no `Serialize`, no `Clone`, and a
//! [`Debug`](std::fmt::Debug) written by hand to say nothing. The one thing
//! done with one is put it on the road it belongs to, which is
//! [`crate::Carried`], and that is the single place in this crate where a
//! password becomes text. One function, so that everything which could ever
//! carry one is what `grep` finds.
//!
//! **What this does not claim.** The bytes are not scrubbed from memory when the
//! value is dropped: doing that honestly needs either `unsafe`, which this
//! workspace forbids, or a dependency. `alo_models::Secret` says the same thing
//! and for the same reason, and claiming it without doing it would be worse than
//! not claiming it.

use std::fmt;

use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Serialize};

use crate::words;

/// The most characters a keyring name may be.
///
/// Long enough for any name a machine writes for itself, short enough that a
/// settings file cannot smuggle a paragraph in where a name belongs.
pub const LONGEST_NAME: usize = 255;

/// Why some text is not a name a password can be kept under.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotAName {
    /// Nothing, or only blank space.
    #[error("no name was given for where the password is kept")]
    Nameless,
    /// Blank space or a control character inside it.
    #[error("that name is not one line")]
    NotOneLine,
    /// Longer than a name may be.
    #[error("that name is longer than {LONGEST_NAME} characters")]
    TooLong,
}

/// Where a proxy's password is kept, by the name it is kept under.
///
/// **This is not a password.** It is what the setting holds, and looking one up
/// is the keyring's — `alo-secrets` on a machine that has one. Reading one back
/// reaches nothing and permits nothing: it names a place, and naming a place is
/// all it does.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WhereThePasswordIs(String);

impl WhereThePasswordIs {
    /// The password kept in the keyring under this name.
    ///
    /// # Errors
    /// [`NotAName`], naming the first thing that does not hold. It is checked
    /// rather than trimmed into shape because this name is read back out of a
    /// file that an organisation wrote, and a name with a newline in it is two
    /// names.
    pub fn named(name: &str) -> Result<Self, NotAName> {
        let name = name.trim();
        if name.is_empty() {
            return Err(NotAName::Nameless);
        }
        if name.chars().any(char::is_control) {
            return Err(NotAName::NotOneLine);
        }
        if name.chars().count() > LONGEST_NAME {
            return Err(NotAName::TooLong);
        }
        Ok(Self(name.to_owned()))
    }

    /// The name to look up in the keyring.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Why something typed into the password field is not a password.
///
/// Neither sentence repeats what was typed, and neither may: a password in a
/// refusal is a password in a log. That is also why there is **no `Display`** —
/// the only road to words is [`NotAPassword::said`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotAPassword {
    /// Nothing was typed. A proxy that needs no password is given none at all,
    /// which is a different thing from being given an empty one.
    Blank,
    /// Something in it cannot be sent — a line break pasted along with it, or a
    /// stray control character.
    NotSendable,
}

impl NotAPassword {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(self) -> words::Word {
        match self {
            Self::Blank => words::PASSWORD_BLANK,
            Self::NotSendable => words::PASSWORD_NOT_SENDABLE,
        }
    }

    /// What this says, in the language the person reads.
    ///
    /// Never fails and never panics, because `alo_strings::Strings` does not: a
    /// `Strings` that was never given [`crate::proxy_words`] answers with the
    /// key, marked, and `Said::is_a_bug`.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

/// A proxy's password, held for one road out and never written down.
pub struct Password(String);

impl Password {
    /// The password somebody has just typed, or the keyring has just answered
    /// with.
    ///
    /// Blank space around it is dropped, because a password pasted out of a
    /// message arrives with a newline on the end. Anything else unprintable is
    /// refused rather than quietly removed: silently changing a credential is
    /// how somebody spends an afternoon on a password that was right all along.
    ///
    /// # Errors
    /// [`NotAPassword`], saying what to do rather than what went wrong.
    pub fn typed(password: &str) -> Result<Self, NotAPassword> {
        let password = password.trim();
        if password.is_empty() {
            return Err(NotAPassword::Blank);
        }
        if password.chars().any(char::is_control) {
            return Err(NotAPassword::NotSendable);
        }
        Ok(Self(password.to_owned()))
    }

    /// The password, for the one caller in this crate that puts it on a road.
    ///
    /// `pub(crate)` on purpose, and it is the only reader of the bytes that
    /// exists: a password can be handed to this crate and cannot be taken back
    /// out of it. [`crate::Carried`] is what uses it, and the test in that file
    /// is what says where it may appear.
    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

/// Says nothing, on purpose.
///
/// A password reaches a log the moment a structure holding one is formatted,
/// and structures get formatted. `alo_models::Secret` writes the same `Debug`
/// by hand for the same reason.
impl fmt::Debug for Password {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Password(…)")
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::in_english;

    /// A name is checked rather than repaired: it comes back out of a file
    /// somebody else wrote.
    #[test]
    fn a_keyring_name_is_checked_and_never_repaired() {
        assert_eq!(
            WhereThePasswordIs::named("  the company proxy  ")
                .unwrap()
                .as_str(),
            "the company proxy"
        );
        assert_eq!(
            WhereThePasswordIs::named("   ").unwrap_err(),
            NotAName::Nameless
        );
        assert_eq!(
            WhereThePasswordIs::named("the company proxy\nand another").unwrap_err(),
            NotAName::NotOneLine
        );
        assert_eq!(
            WhereThePasswordIs::named(&"x".repeat(LONGEST_NAME + 1)).unwrap_err(),
            NotAName::TooLong
        );
    }

    /// **A password formatted into anything says nothing.** This is the test
    /// that keeps a credential out of the first log line somebody adds.
    #[test]
    fn a_password_that_is_formatted_says_nothing() {
        let password = Password::typed("hunter2").unwrap();
        assert_eq!(format!("{password:?}"), "Password(…)");
        assert!(!format!("{password:?}").contains("hunter2"));
    }

    /// A password pasted with a newline on the end is the password; one with a
    /// control character in the middle is refused rather than repaired.
    #[test]
    fn a_pasted_password_is_taken_and_a_broken_one_is_refused() {
        assert_eq!(Password::typed(" hunter2\n").unwrap().as_str(), "hunter2");
        assert_eq!(Password::typed("  ").unwrap_err(), NotAPassword::Blank);
        assert_eq!(
            Password::typed("hunt\u{1b}er2").unwrap_err(),
            NotAPassword::NotSendable
        );
    }

    /// **Neither refusal repeats what was typed**, and both say what to do.
    #[test]
    fn a_refusal_about_a_password_never_carries_one() {
        let strings = in_english();
        for refusal in [NotAPassword::Blank, NotAPassword::NotSendable] {
            let said = refusal.said(&strings);
            assert!(!said.is_a_bug(), "{said}");
            assert!(said.unfilled().is_empty(), "{said}");
        }
        let blank = NotAPassword::Blank.said(&strings);
        assert!(blank.text().contains("password"), "{blank}");
        assert!(!blank.text().contains("hunter2"), "{blank}");
    }

    /// **The setting can hold where a password is and cannot hold one**, which
    /// is what written-down-and-read-back has to keep true.
    #[test]
    fn where_a_password_is_kept_survives_being_written_down() {
        let kept = WhereThePasswordIs::named("the company proxy").unwrap();
        let written = serde_json::to_string(&kept).unwrap();
        assert_eq!(written, "\"the company proxy\"");
        assert_eq!(
            serde_json::from_str::<WhereThePasswordIs>(&written).ok(),
            Some(kept)
        );
    }
}
