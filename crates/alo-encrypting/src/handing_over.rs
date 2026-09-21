//! Where a secret is put while a rented tool is being given it, and why that
//! place is in memory.
//!
//! `cryptsetup` and `systemd-cryptenroll` take a secret from a file or from a
//! terminal, and the terminal is not available to a program: measured in the
//! pinned base, `systemd-cryptenroll --password` with standard input closed does
//! not fail, it waits — two and a half minutes before it was killed
//! ([ADR 0056](../../../docs/decisions/0056-a-sealed-disks-promise-is-shown-on-a-machine-with-a-chip.md),
//! measurement 6, and `docs/quirks.md`). So an install hands a secret over in a
//! file, and the only question worth arguing about is *which file*.
//!
//! # `/run`, and never the disk being encrypted
//!
//! [`THE_ONE_PLACE`] is under `/run`, which on this machine is a `tmpfs`: it is
//! memory with a path, it is not on any disk, and it does not survive a restart.
//! A recovery key written there is gone when the install finishes whether or not
//! anybody remembered to remove it, and *the key is never stored on the disk it
//! recovers* stops being a rule somebody follows and becomes a fact about where
//! the file was. Each of these is made readable by nobody but its owner
//! ([`ONLY_ITS_OWNER_MAY_READ_IT`]) — the rented tool complains in as many words
//! when it is not, which is how the number here was measured rather than
//! guessed.
//!
//! # Four, and there is no fifth
//!
//! This crate names the four secrets an enrolment moves and nothing else. A
//! caller cannot ask for a file of its own: there is no constructor that takes a
//! path, so there is no road from a name somebody typed to a file a root program
//! reads.

use std::fmt;

/// The one directory a secret is ever put in, which is memory rather than a
/// disk.
pub const THE_ONE_PLACE: &str = "/run/alo-encrypting";

/// The mode each of these files is made with: its owner reads it and nobody
/// else does.
///
/// `systemd-cryptenroll` says so itself when it is not — *has 0644 mode that is
/// too permissive* — and a warning from a rented tool about the way we handed it
/// a key is a bug of ours.
pub const ONLY_ITS_OWNER_MAY_READ_IT: u32 = 0o600;

/// A secret on its way to a rented tool, and the file it travels in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ASecretOnItsWay {
    /// The installer's own first key: made by the installer, shown to nobody,
    /// and wiped out of the volume at the end of the road.
    TheInstallersFirstKey,
    /// What this person will open the machine with, while it is being enrolled.
    ThePersonsSecret,
    /// What they are changing it to, while it is replacing the old one.
    ThePersonsNewSecret,
    /// The recovery key, while it is opening a volume nothing else opens.
    TheRecoveryKey,
}

impl ASecretOnItsWay {
    /// Every one of them, so that a caller making the directory ready has a
    /// list rather than a memory.
    pub const ALL: [Self; 4] = [
        Self::TheInstallersFirstKey,
        Self::ThePersonsSecret,
        Self::ThePersonsNewSecret,
        Self::TheRecoveryKey,
    ];

    /// Where the file is.
    #[must_use]
    pub const fn where_it_is(self) -> &'static str {
        match self {
            Self::TheInstallersFirstKey => "/run/alo-encrypting/the-installers-first-key",
            Self::ThePersonsSecret => "/run/alo-encrypting/the-persons-secret",
            Self::ThePersonsNewSecret => "/run/alo-encrypting/the-persons-new-secret",
            Self::TheRecoveryKey => "/run/alo-encrypting/the-recovery-key",
        }
    }
}

impl fmt::Display for ASecretOnItsWay {
    /// Says which of the four, and never what is in it. These names reach a
    /// service log; the secrets do not.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::TheInstallersFirstKey => "the installer's first key",
            Self::ThePersonsSecret => "the person's secret",
            Self::ThePersonsNewSecret => "the person's new secret",
            Self::TheRecoveryKey => "the recovery key",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Every one of them is in the one place, and that place is memory.**
    #[test]
    fn every_secret_travels_through_one_directory_in_memory() {
        assert!(THE_ONE_PLACE.starts_with("/run/"));
        for secret in ASecretOnItsWay::ALL {
            let at = secret.where_it_is();
            assert!(at.starts_with(&format!("{THE_ONE_PLACE}/")), "{at}");
            assert!(
                !at.contains("..") && at.matches('/').count() == 3,
                "{at} leaves the one place"
            );
        }
    }

    /// **No two of them share a file**, which is what would let one secret be
    /// read where another was meant.
    #[test]
    fn no_two_secrets_share_a_file() {
        let mut places: Vec<&str> = ASecretOnItsWay::ALL
            .iter()
            .map(|secret| secret.where_it_is())
            .collect();
        places.sort_unstable();
        places.dedup();
        assert_eq!(places.len(), ASecretOnItsWay::ALL.len());
    }

    /// The mode is the owner's alone, and the number is written down so that
    /// changing it is a decision.
    #[test]
    fn only_the_owner_reads_one() {
        assert_eq!(ONLY_ITS_OWNER_MAY_READ_IT, 0o600);
    }

    /// **Naming one says which it is and never what is in it**, because these
    /// names reach a service log.
    #[test]
    fn naming_a_secret_says_which_and_not_what() {
        for secret in ASecretOnItsWay::ALL {
            let said = secret.to_string();
            assert!(!said.is_empty());
            assert!(!said.contains("/run"), "{said}");
        }
        assert_eq!(
            ASecretOnItsWay::TheRecoveryKey.to_string(),
            "the recovery key"
        );
    }
}
