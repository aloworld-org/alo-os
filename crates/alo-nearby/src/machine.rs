//! The one thing a machine says about itself, and the only thing it keeps.
//!
//! # Why it is random rather than derived
//!
//! The obvious identity for a machine is one it already has: a MAC address, a
//! disk serial, a TPM key, the name somebody typed when Windows was installed.
//! Every one of them is refused here, for two different reasons.
//!
//! A **serial outlives a reinstall**, and an identifier that survives somebody
//! wiping their machine is a tracker. Somebody who reinstalls alo OS to stop
//! being recognised has to actually stop being recognised, and the only way
//! that is true is if the identity lives in a file the reinstall removes.
//!
//! A **name a person chose is a person's name**. `DISAN-LAPTOP` on a café
//! network tells everybody in the café who is in the café. ADR 0003 says
//! discovery reveals presence and nothing else, and a hostname is not nothing.
//!
//! So it is sixteen random bytes from the kernel, written once, read
//! afterwards. Stable across a restart, which is what
//! [`remembered_at`](MachineId::remembered_at) is for; gone when the
//! installation is; and derived from nothing about the machine, which is why
//! two identities made on the same machine in two places differ.

use std::fmt::{self, Display, Formatter};
use std::path::Path;

use crate::refusing::{NotNearby, because};

/// How many bytes of randomness an identity is.
///
/// Sixteen: the same width as a UUID, because the thing being avoided is two
/// machines on one network drawing the same identity, and at that width it does
/// not happen.
const BYTES: usize = 16;

/// How many characters an identity reads as, being two per byte.
const CHARACTERS: usize = BYTES * 2;

/// Which machine, and nothing else about it.
///
/// It is a DNS label as it stands — thirty-two lowercase hexadecimal
/// characters, well under the sixty-three a label may be — which is what lets
/// the advertisement name the machine without inventing a second spelling for
/// it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MachineId(String);

impl MachineId {
    /// A new identity, from the kernel's randomness.
    ///
    /// # Errors
    ///
    /// [`NotNearby::NoRandomness`] if the kernel will not give any. There is no
    /// fallback generator: a predictable machine identity is worse than a
    /// machine that will not start, and every random byte in this workspace
    /// comes through one door.
    pub fn made() -> Result<Self, NotNearby> {
        let mut bytes = [0_u8; BYTES];
        getrandom::fill(&mut bytes).map_err(|why| NotNearby::NoRandomness(why.to_string()))?;
        let mut said = String::with_capacity(CHARACTERS);
        for byte in bytes {
            use fmt::Write as _;
            // Writing to a `String` cannot fail; the result is taken rather
            // than ignored so that no `Result` is dropped silently.
            write!(&mut said, "{byte:02x}")
                .map_err(|why| NotNearby::NoRandomness(why.to_string()))?;
        }
        Ok(Self(said))
    }

    /// An identity somebody else said, checked before it is believed.
    ///
    /// # Errors
    ///
    /// [`NotNearby::NotAMachineIdentity`] for anything that is not thirty-two
    /// lowercase hexadecimal characters. A packet arrives from a stranger, so
    /// what it calls itself is checked here and not further in: without this,
    /// a machine on the network could name itself `..` or a person's name and
    /// that string would travel into whatever displays it.
    pub fn read(said: &str) -> Result<Self, NotNearby> {
        let hexadecimal = said.len() == CHARACTERS
            && said
                .chars()
                .all(|letter| letter.is_ascii_digit() || ('a'..='f').contains(&letter));
        if hexadecimal {
            Ok(Self(said.to_owned()))
        } else {
            Err(NotNearby::NotAMachineIdentity(said.to_owned()))
        }
    }

    /// The identity this machine keeps at `at`, making one the first time.
    ///
    /// This is what *stable across a restart* means in practice: the file is
    /// the whole of the stability, so a person who deletes it is a new machine
    /// to everybody, which is the point of it being a file.
    ///
    /// # Errors
    ///
    /// [`NotNearby::IdentityUnreadable`] if the file cannot be read or written,
    /// and [`NotNearby::NotAMachineIdentity`] if what is in it is not an
    /// identity — which is refused rather than replaced, because silently
    /// making a new one would change who this machine is to every machine it
    /// had paired with.
    pub fn remembered_at(at: &Path) -> Result<Self, NotNearby> {
        match std::fs::read_to_string(at) {
            Ok(said) => Self::read(said.trim()),
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => {
                let made = Self::made()?;
                if let Some(holding) = at.parent() {
                    std::fs::create_dir_all(holding).map_err(|why| {
                        NotNearby::IdentityUnreadable {
                            at: at.to_path_buf(),
                            why: because(&why),
                        }
                    })?;
                }
                std::fs::write(at, made.as_str()).map_err(|why| NotNearby::IdentityUnreadable {
                    at: at.to_path_buf(),
                    why: because(&why),
                })?;
                Ok(made)
            }
            Err(why) => Err(NotNearby::IdentityUnreadable {
                at: at.to_path_buf(),
                why: because(&why),
            }),
        }
    }

    /// The identity as it is written, on the wire and in the file.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for MachineId {
    fn fmt(&self, into: &mut Formatter<'_>) -> fmt::Result {
        into.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{CHARACTERS, MachineId};
    use crate::refusing::NotNearby;

    #[expect(
        clippy::unwrap_used,
        reason = "in a test, a panic on an unexpected Err is the failure being reported"
    )]
    mod made {
        use super::{CHARACTERS, MachineId, NotNearby};

        /// An identity reads as what the wire expects: a label, all of it
        /// hexadecimal.
        #[test]
        fn an_identity_is_thirty_two_hexadecimal_characters() {
            let made = MachineId::made().unwrap();
            assert_eq!(made.as_str().len(), CHARACTERS);
            assert!(made.as_str().chars().all(|of| of.is_ascii_hexdigit()));
            assert_eq!(made.as_str(), made.as_str().to_lowercase());
        }

        /// **It is not derived from this machine.** Two identities made one
        /// after the other on one machine differ, which no serial, MAC address
        /// or hostname could manage — the test is what keeps somebody later
        /// from "improving" this into something stable and traceable.
        #[test]
        fn two_identities_made_on_this_machine_are_not_the_same() {
            let one = MachineId::made().unwrap();
            let other = MachineId::made().unwrap();
            assert_ne!(one, other);
        }

        /// A stranger's word for itself is checked before it is believed,
        /// because it travels onward into whatever displays it.
        #[test]
        fn what_is_not_an_identity_is_refused_rather_than_carried() {
            for not_one in [
                "",
                "disan-laptop",
                "Disan's MacBook",
                "../../etc/passwd",
                "00112233445566778899AABBCCDDEEFF",
                "00112233445566778899aabbccddee",
                "00112233445566778899aabbccddeeff00",
            ] {
                let refused = MachineId::read(not_one).unwrap_err();
                assert_eq!(
                    refused,
                    NotNearby::NotAMachineIdentity(not_one.to_owned()),
                    "`{not_one}` was taken as a machine identity"
                );
            }
        }

        /// One this crate made is one it reads back, which is the round trip
        /// the wire depends on.
        #[test]
        fn an_identity_this_machine_made_is_one_it_reads_back() {
            let made = MachineId::made().unwrap();
            assert_eq!(MachineId::read(made.as_str()).unwrap(), made);
        }
    }

    #[expect(
        clippy::unwrap_used,
        reason = "in a test, a panic on an unexpected Err is the failure being reported"
    )]
    mod remembered {
        use super::{MachineId, NotNearby};

        /// A directory of this test's own, so two tests never share a file.
        fn somewhere_of_our_own(called: &str) -> std::path::PathBuf {
            let at = std::env::temp_dir().join(format!(
                "alo-nearby-{called}-{}-{:?}",
                std::process::id(),
                std::thread::current().id()
            ));
            drop(std::fs::remove_dir_all(&at));
            at
        }

        /// **Stable across a restart.** Asking twice is asking the file, and
        /// the second answer is the first.
        #[test]
        fn the_identity_is_the_same_the_second_time_it_is_asked_for() {
            let at = somewhere_of_our_own("stable").join("machine");
            let first = MachineId::remembered_at(&at).unwrap();
            let second = MachineId::remembered_at(&at).unwrap();
            assert_eq!(first, second);
            drop(std::fs::remove_dir_all(at.parent().unwrap()));
        }

        /// **And gone when the installation is.** A second installation on the
        /// same machine is a different machine to everybody, which is the whole
        /// reason the identity is a file rather than a serial.
        #[test]
        fn a_second_installation_on_this_machine_is_a_different_machine() {
            let one = somewhere_of_our_own("installed-once").join("machine");
            let other = somewhere_of_our_own("installed-again").join("machine");
            assert_ne!(
                MachineId::remembered_at(&one).unwrap(),
                MachineId::remembered_at(&other).unwrap()
            );
            drop(std::fs::remove_dir_all(one.parent().unwrap()));
            drop(std::fs::remove_dir_all(other.parent().unwrap()));
        }

        /// A file holding something that is not an identity is refused rather
        /// than replaced: a new one would change who this machine is to every
        /// machine it had paired with, which is a thing a person should be told
        /// about rather than have done for them.
        #[test]
        fn a_file_holding_something_else_is_refused_rather_than_replaced() {
            let at = somewhere_of_our_own("edited").join("machine");
            std::fs::create_dir_all(at.parent().unwrap()).unwrap();
            std::fs::write(&at, "disan-laptop").unwrap();

            let refused = MachineId::remembered_at(&at).unwrap_err();

            assert_eq!(
                refused,
                NotNearby::NotAMachineIdentity("disan-laptop".to_owned())
            );
            assert_eq!(std::fs::read_to_string(&at).unwrap(), "disan-laptop");
            drop(std::fs::remove_dir_all(at.parent().unwrap()));
        }

        /// Trailing whitespace from an editor is not what somebody meant to
        /// write, and is the one thing forgiven here.
        #[test]
        fn a_newline_at_the_end_of_the_file_is_forgiven() {
            let at = somewhere_of_our_own("newline").join("machine");
            let made = MachineId::made().unwrap();
            std::fs::create_dir_all(at.parent().unwrap()).unwrap();
            std::fs::write(&at, format!("{made}\n")).unwrap();
            assert_eq!(MachineId::remembered_at(&at).unwrap(), made);
            drop(std::fs::remove_dir_all(at.parent().unwrap()));
        }
    }
}
