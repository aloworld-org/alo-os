//! The one thing done to a password, and the decoy that keeps time.
//!
//! Argon2id, through the RustCrypto crate, with a fresh salt from the
//! kernel's generator for every hash. Rented rather than written, because a
//! key derivation function this repository wrote itself would be the security
//! bug the rest of it is organised to avoid — this is the one dependency in
//! this crate that exists *because* writing it ourselves would be worse.
//!
//! # Every hash here costs the same
//!
//! [`Hashed::of`] and [`Hashed::nobody`] use one set of parameters —
//! `Argon2::default()`, which is Argon2id v19 at the cost the crate's authors
//! keep current — and [`Hashed::read`] refuses any stored string that is not
//! an argon2id hash. That is not tidiness: `crate::store` verifies an unknown
//! name against [`Hashed::nobody`]'s decoy so that it takes as long as a
//! wrong password, and *as long* is only true while everything verified in
//! this crate does the same work. A store carrying a cheaper hash would be a
//! store whose unknown names answer at a different speed.
//!
//! # What a hash is not
//!
//! It is not a password, and nothing here turns it back into one — but it is
//! still the thing an offline guess runs against, so no error and no `Debug`
//! in this crate writes one out.

use argon2::Argon2;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};

use crate::refusing::CannotHash;

/// What an argon2id hash names itself in a PHC string.
const ARGON2ID: &str = "argon2id";

/// How many random bytes salt one password.
const A_SALT: usize = 16;

/// How many random bytes make the decoy password nobody knows.
const A_SECRET_NOBODY_KNOWS: usize = 32;

/// One password, hashed — the only form a password ever takes at rest.
///
/// Holds a PHC string (`$argon2id$…`) that has been through
/// [`PasswordHash::new`] at construction, so [`Hashed::verifies`] cannot meet
/// a string it cannot parse and fail fast — a fast failure on one account
/// would be a timing difference between accounts.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct Hashed(String);

impl std::fmt::Debug for Hashed {
    /// Says that it is a hash and nothing of which — a hash is what an
    /// offline guess runs against, and `Debug` is what ends up in logs.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Hashed(argon2id)")
    }
}

impl Hashed {
    /// This password, hashed with a fresh salt.
    ///
    /// # Errors
    ///
    /// [`CannotHash`] when the kernel's randomness is unreachable or the hash
    /// itself fails — both machine states, neither caused by the password.
    pub(crate) fn of(password: &str) -> Result<Self, CannotHash> {
        let salt = salted::<A_SALT>()?;
        let hashed = Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|why| CannotHash {
                why: why.to_string(),
            })?;
        Ok(Self(hashed.to_string()))
    }

    /// The hash an unknown name is verified against, so that *no such
    /// account* takes exactly as long as *wrong password*.
    ///
    /// Its password is random bytes nobody kept, so nothing verifies against
    /// it and nothing ever will; its parameters are [`Hashed::of`]'s, which
    /// is the whole point.
    ///
    /// # Errors
    ///
    /// [`CannotHash`], as [`Hashed::of`].
    pub(crate) fn nobody() -> Result<Self, CannotHash> {
        let secret = salted::<A_SECRET_NOBODY_KNOWS>()?;
        Self::of(secret.as_str())
    }

    /// A hash as the store wrote it down, believed only if it is a whole
    /// argon2id PHC string.
    ///
    /// Anything else — a truncated line, another algorithm, a password typed
    /// in raw — answers [`None`], and the store refuses the file rather than
    /// keeping an account nothing could ever sign in to at the right speed.
    pub(crate) fn read(written: &str) -> Option<Self> {
        let parsed = PasswordHash::new(written).ok()?;
        // A PHC string may legally omit its salt or its hash, and a truncated
        // line does; such a string verifies nothing and fails *fast*, which
        // would make one account answer at a different speed from the rest.
        if parsed.algorithm.as_str() != ARGON2ID || parsed.salt.is_none() || parsed.hash.is_none() {
            return None;
        }
        Some(Self(written.to_owned()))
    }

    /// The PHC string, for the store to write down.
    pub(crate) fn as_written(&self) -> &str {
        &self.0
    }

    /// Whether this password is the one hashed here.
    ///
    /// The parse cannot fail — every constructor has already parsed the
    /// string — and if it somehow did, the answer is *no* rather than a
    /// panic beside a password prompt.
    pub(crate) fn verifies(&self, offered: &str) -> bool {
        match PasswordHash::new(&self.0) {
            Ok(parsed) => Argon2::default()
                .verify_password(offered.as_bytes(), &parsed)
                .is_ok(),
            Err(_unparseable) => false,
        }
    }
}

/// Fresh random bytes from the kernel, as the string form the hasher takes.
///
/// # Errors
///
/// [`CannotHash`] when the generator is unreachable — refused loudly rather
/// than salted from anything weaker.
fn salted<const BYTES: usize>() -> Result<SaltString, CannotHash> {
    let mut bytes = [0u8; BYTES];
    getrandom::fill(&mut bytes).map_err(|why| CannotHash {
        why: why.to_string(),
    })?;
    SaltString::encode_b64(&bytes).map_err(|why| CannotHash {
        why: why.to_string(),
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **The right password verifies and a wrong one does not**, which is the
    /// whole of what a hash is for.
    #[test]
    fn the_password_hashed_is_the_password_that_verifies() {
        let hashed = Hashed::of("correct horse battery staple").unwrap();
        assert!(hashed.verifies("correct horse battery staple"));
        assert!(!hashed.verifies("correct horse battery stable"));
        assert!(!hashed.verifies(""));
    }

    /// **Two hashes of one password differ**, because every hash gets a fresh
    /// salt — equal hashes would say which two people chose the same password.
    #[test]
    fn one_password_hashed_twice_is_two_different_strings() {
        let once = Hashed::of("the same password").unwrap();
        let twice = Hashed::of("the same password").unwrap();
        assert_ne!(once.as_written(), twice.as_written());
        assert!(once.verifies("the same password"));
        assert!(twice.verifies("the same password"));
    }

    /// **What was written down reads back and still verifies**, which is the
    /// round trip the store depends on.
    #[test]
    fn a_hash_written_down_reads_back_and_verifies() {
        let hashed = Hashed::of("a password").unwrap();
        let read = Hashed::read(hashed.as_written()).unwrap();
        assert!(read.verifies("a password"));
        assert!(!read.verifies("another password"));
    }

    /// **A password typed into the store raw is not a hash**, and neither is
    /// a truncated line — the two shapes a hand-edited store really takes.
    #[test]
    fn what_is_not_an_argon2id_hash_is_refused() {
        assert!(Hashed::read("hunter2").is_none());
        assert!(Hashed::read("").is_none());
        let whole = Hashed::of("a password").unwrap();
        let truncated: String = whole.as_written().chars().take(20).collect();
        assert!(Hashed::read(&truncated).is_none());
    }

    /// **Another algorithm's hash is refused even when it is a valid PHC
    /// string**, because a cheaper hash in the store would make one account
    /// answer at a different speed from the rest.
    #[test]
    fn a_hash_that_is_not_argon2id_is_refused() {
        let pbkdf2 = "$pbkdf2-sha256$i=10000,l=32$c2FsdHNhbHRzYWx0c2FsdA$xYc6dKTKm3ZaSqa9866y2SEOOtsCM5DkjClZ2fJ9Cbw";
        assert!(Hashed::read(pbkdf2).is_none());
    }

    /// **The decoy verifies nothing anybody could type**, because its
    /// password was random bytes nobody kept — and two decoys are two, so no
    /// fixed string could ever be the one that matches everywhere.
    #[test]
    fn the_decoy_answers_no_to_everything_and_is_never_the_same_twice() {
        let nobody = Hashed::nobody().unwrap();
        assert!(!nobody.verifies(""));
        assert!(!nobody.verifies("password"));
        assert_ne!(nobody.as_written(), Hashed::nobody().unwrap().as_written());
    }

    /// **`Debug` says nothing an offline guess could run against.**
    #[test]
    fn a_hash_does_not_write_itself_into_a_log() {
        let hashed = Hashed::of("a password").unwrap();
        assert_eq!(format!("{hashed:?}"), "Hashed(argon2id)");
    }
}
