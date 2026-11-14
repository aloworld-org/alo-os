//! Which kind of secret an account signs in with: a PIN, or a password.
//!
//! # Why the store says which
//!
//! The first start offers *Add a PIN?* and, below it, *Use password only*
//! (`docs/features.md`, *Skipping the PIN asks for a password instead*). Either
//! road ends in **one** secret, hashed the same way and checked the same way —
//! a PIN on alo OS is not a second door beside a password, it is the account's
//! only one.
//!
//! What the store did not keep, until 2026-11-14, was **which of the two the
//! person chose**. The sign-in screen therefore had nothing to go on and said
//! *Enter your PIN* to everybody, and three walks on booted machines
//! (2026-11-12, -13, -14) found a person who had pressed *Use password only*
//! being asked every morning for a PIN they never set. The design draws both
//! states (`367:25424` *PIN entered*, `367:25544` *Password*, whose entry
//! reads *Enter your password*), and the screen can only choose between them
//! if the account says.
//!
//! # A store written before this existed
//!
//! Its accounts carry no kind, and they read as [`SecretKind::Password`]. That
//! is the reading that is never false: whatever was typed at the first start is
//! the account's password in the sense every check here uses, and ADR 0150
//! already calls it *the same password, or PIN, they sign in with*. Calling a
//! password a PIN is the fault this file exists to end; calling a PIN a
//! password is only less specific.

use serde::{Deserialize, Serialize};

/// Which kind of secret an account signs in with.
///
/// **It changes nothing about how the secret is checked** — both are hashed
/// and verified identically — only what a person is asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SecretKind {
    /// A PIN, chosen on *Add a PIN?* at the first start.
    Pin,
    /// A password: chosen with *Use password only*, set again with the
    /// recovery key, or written by an alo OS that did not record which.
    #[default]
    Password,
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// One field holding the kind, the way the store writes it.
    #[derive(Debug, Serialize, Deserialize)]
    struct Holding {
        /// The kind.
        kind: SecretKind,
    }

    /// **Each kind is written as one lowercase word**, which is what a person
    /// reading the store by hand sees.
    #[test]
    fn each_kind_is_written_as_one_lowercase_word() {
        for (kind, word) in [
            (SecretKind::Pin, "kind = \"pin\""),
            (SecretKind::Password, "kind = \"password\""),
        ] {
            let written = toml::to_string(&Holding { kind }).unwrap();
            assert_eq!(written.trim(), word);
            assert_eq!(toml::from_str::<Holding>(&written).unwrap().kind, kind);
        }
    }

    /// **A kind nobody declared is refused**, not read as either.
    #[test]
    fn a_kind_nobody_declared_is_refused() {
        for wrong in ["kind = \"fingerprint\"", "kind = \"PIN\"", "kind = 1"] {
            assert!(
                toml::from_str::<Holding>(wrong).is_err(),
                "`{wrong}` was read as a kind of secret"
            );
        }
    }

    /// **Nothing said reads as a password** — the reading that is never false.
    #[test]
    fn nothing_said_is_a_password() {
        assert_eq!(SecretKind::default(), SecretKind::Password);
    }
}
