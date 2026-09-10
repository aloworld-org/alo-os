//! Every way this crate refuses, and who each refusal is written for.
//!
//! Three audiences, three types, and they are deliberately not one enum. A
//! person typing a password reads [`NotSignedIn`], which has **no `Display`**
//! and speaks only through `crate::words` — `alo_secrets::NotStored` is the
//! precedent and its reason holds here doubled, because the text nearest a
//! password prompt is the text most likely to end up in a screenshot or a log.
//! Whoever stands the machine up reads [`NotKept`] and [`NotCreated`] out of a
//! service log, so those keep their English the way `alo-agentd`'s own
//! refusals do.
//!
//! # One refusal for two different facts, on purpose
//!
//! [`NotSignedIn::Refused`] is a wrong password **and** an unknown name, as
//! one value. Telling them apart — in the variant, the sentence, or the time
//! taken — would let anybody with the sign-in surface enumerate which names
//! have accounts on a machine they do not own. `crate::store` holds the timing
//! half of that promise; this type holds the shape half by having nothing to
//! compare.

use std::path::PathBuf;

use alo_strings::{Filling, Said, Strings};

use crate::words;

/// A password could not be hashed.
///
/// The machine's randomness was unreachable, which on a real machine means
/// something is deeply wrong — this is refused loudly rather than salted from
/// anything less than the kernel's generator.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
#[error("a password could not be hashed: {why}")]
pub struct CannotHash {
    /// What the machine said.
    pub why: String,
}

/// Why an account was not created.
///
/// Read by whoever is standing the machine up — the first-boot surface is the
/// compositor lane's, and until it exists these keep their English for the
/// log, the way `alo_secrets::NotStored` deliberately declared no words before
/// anything rendered them.
#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum NotCreated {
    /// A name an account cannot go under.
    #[error(
        "an account's name is ascii lowercase letters, digits, `-` and `_`, starts with a \
         letter, and is at most 32 long — the shape a Unix login already has to have"
    )]
    NotAName,

    /// A name that already has an account.
    #[error("`{name}` already has an account on this machine")]
    TakenName {
        /// The name both would go under.
        name: String,
    },

    /// A number that already belongs to somebody.
    #[error("uid {uid} already belongs to `{name}`, and one number is one person")]
    TakenNumber {
        /// The number both would carry.
        uid: u32,
        /// Who already carries it.
        name: String,
    },

    /// Uid 0 is root, and the person is never root.
    ///
    /// `alo-agentd` runs as the signed-in person (ADR 0001 §2); an account
    /// created at 0 would be a machine whose agent daemon holds every
    /// authority the capability model exists to withhold.
    #[error("uid 0 is root, and the person alo-agentd runs as is never root (ADR 0001 §2)")]
    Root,

    /// `4294967295` is not a user.
    ///
    /// It is what a Unix call answers when there is no user, and it is what a
    /// script that could not look one up leaves behind — the machine
    /// description refuses it for the same reason.
    #[error("4294967295 is not a user — it is what a Unix call answers when there is none")]
    NobodyAtAll,

    /// An empty password is not a password.
    #[error("an account needs a password, and an empty one is not one")]
    NoPassword,

    /// The machine could not hash the password.
    #[error(transparent)]
    CouldNotHash(#[from] CannotHash),
}

/// Why the store on the disk was not believed, read, or written.
///
/// Read out of a service log by whoever installs the machine, so it keeps its
/// English. Nothing in it ever carries a password, and nothing in it carries a
/// hash either — a hash is not a password, but it is still a thing an offline
/// guess runs against, and a log line is not where it belongs.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum NotKept {
    /// There is no store at all.
    ///
    /// A machine nobody has made an account on yet — the first-boot state,
    /// answered as itself rather than as an empty store, because *nobody can
    /// sign in yet* and *somebody deleted the accounts* deserve different
    /// sentences from whatever reads this.
    #[error("there is no accounts store at {}", at.display())]
    NotThere {
        /// Where it was looked for.
        at: PathBuf,
    },

    /// The path is a symbolic link, and the store is read only as a real file.
    ///
    /// The machine description's rule, for the machine description's reason: a
    /// link is a name somebody can point at a file they own.
    #[error("{} is a symbolic link, and the accounts store is read only as the file it is", at.display())]
    ALink {
        /// Where the link is.
        at: PathBuf,
    },

    /// The file could not be read.
    #[error("{} could not be read: {why}", at.display())]
    NotRead {
        /// Where it is.
        at: PathBuf,
        /// What the machine said.
        why: String,
    },

    /// The file belongs to somebody other than root or the login reading it.
    ///
    /// Whoever owns this file names who may sign in, so it is believed from
    /// exactly the two owners the machine description is believed from.
    #[error(
        "{} belongs to uid {owner}, and an accounts store is believed only from root or from \
         the login reading it",
        at.display()
    )]
    SomebodyElses {
        /// Where it is.
        at: PathBuf,
        /// Who owns it.
        owner: u32,
    },

    /// The file can be written by its group or by the world.
    #[error(
        "{} is writable by its group or by anyone (mode {mode:o}), so somebody else could name \
         who signs in on this machine",
        at.display()
    )]
    WritableByOthers {
        /// Where it is.
        at: PathBuf,
        /// The mode it has.
        mode: u32,
    },

    /// The file is not the shape an accounts store takes.
    #[error("the accounts store is not the shape one takes: {why}")]
    NotTheShape {
        /// What would not parse, in the parser's words.
        why: String,
    },

    /// A store written for an alo OS this is not.
    ///
    /// Refused rather than guessed at, before any other value in the file —
    /// the machine description's rule, kept for its reason.
    #[error(
        "the accounts store says format {format}, and this alo OS reads only format {}",
        crate::THE_FORMAT
    )]
    AnotherFormat {
        /// The number it says.
        format: u32,
    },

    /// An account whose password field does not hold an argon2id hash.
    ///
    /// A password typed into the store raw is the likeliest way this happens,
    /// and it is refused loudly rather than left as an account nothing could
    /// ever sign in to — only a hash is ever written here.
    #[error(
        "the account `{name}` does not hold an argon2id hash — a password is never written to \
         the store, only its hash"
    )]
    NotAHash {
        /// Whose line it is.
        name: String,
    },

    /// An account in the store is not one this crate would create.
    #[error(transparent)]
    NotAnAccount(#[from] NotCreated),

    /// The store could not be written down.
    #[error("the accounts store could not be written to {}: {why}", at.display())]
    NotWritten {
        /// Where it was going.
        at: PathBuf,
        /// What the machine said.
        why: String,
    },
}

/// Why nobody was signed in.
///
/// **No `Display`.** These are the sentences nearest a password prompt, and a
/// type with a `Display` there is a sentence one `to_string()` from a log line
/// written by somebody who never thought about passwords. What a person reads
/// is [`NotSignedIn::said`], through the machine's vocabulary, in their own
/// language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotSignedIn {
    /// The name and password did not sign anybody in.
    ///
    /// **One value for a wrong password and an unknown name** — see this
    /// file's header. There is nothing on this variant because anything on it
    /// would be the distinction it exists to withhold.
    Refused,

    /// The account is real and is not the person this machine is described as.
    ///
    /// The machine description names the login `alo-agentd` runs as, and a
    /// session under any other number would be a machine whose description and
    /// whose session disagree about who is signed in. Both numbers are carried
    /// because whoever fixes this needs them; neither reaches the sentence a
    /// person reads.
    NotThePerson {
        /// Who authenticated.
        signed_in: u32,
        /// Whom the machine description names.
        described: u32,
    },
}

impl NotSignedIn {
    /// The string this crate declares for this refusal.
    #[must_use]
    pub const fn word(&self) -> words::Word {
        match self {
            Self::Refused => words::NOT_SIGNED_IN,
            Self::NotThePerson { .. } => words::NOT_THIS_MACHINES_PERSON,
        }
    }

    /// What a person is told, in the language they read.
    ///
    /// Never fails and never panics: a `Strings` that was never given this
    /// crate's words answers with the key, marked as the bug it is, and nobody
    /// was signed in either way.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        // No gap in either sentence, deliberately: a gap is the one road a
        // name somebody typed could take into a sentence a person reads.
        strings.say(&self.word().key(), &Filling::nothing())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The refusal a person reads carries nothing to tell two causes
    /// apart.** `Refused` is one value; two of them are always equal, which is
    /// what lets a test one crate up assert that a wrong password and an
    /// unknown name are indistinguishable in shape.
    #[test]
    fn one_refusal_is_always_the_same_refusal() {
        assert_eq!(NotSignedIn::Refused, NotSignedIn::Refused);
    }

    /// **The two refusals say two different sentences**, because they ask a
    /// person to do two different things — retype, or go and fix the machine's
    /// description.
    #[test]
    fn the_two_refusals_are_two_words() {
        let refused = NotSignedIn::Refused;
        let not_the_person = NotSignedIn::NotThePerson {
            signed_in: 1001,
            described: 1000,
        };
        assert_ne!(refused.word().named(), not_the_person.word().named());
    }

    /// **Nothing operator-facing quotes a hash or a password.** The sentences
    /// are fixed at compile time except for paths, names, numbers and what the
    /// machine itself said — checked here for the two variants whose fields
    /// are closest to the secret.
    #[test]
    fn no_refusal_carries_a_secret_into_its_sentence() {
        let not_a_hash = NotKept::NotAHash {
            name: "ada".to_owned(),
        };
        assert!(not_a_hash.to_string().contains("ada"));
        assert!(!not_a_hash.to_string().contains('$'));

        let no_password = NotCreated::NoPassword;
        assert!(no_password.to_string().contains("password"));
    }
}
