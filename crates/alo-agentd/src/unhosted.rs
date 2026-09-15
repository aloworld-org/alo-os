//! Why this machine advertises no workspace, in what the person can act on.
//!
//! [`NotHosting`] is written for whoever installed the workspace server: it
//! names the file, the owner and the mode, in English, in the service log. The
//! person asking their own machine what it tells the network is somebody else,
//! and eleven ways a file can be wrong are not eleven things they can do. So the
//! eleven arms are grouped here into the three a person **can** act on, each with one
//! sentence in their language that names no path, no owner and no mode:
//!
//! - [`Unhosted::NotTheSystems`] — a link, not a file, not root's, or writable
//!   by somebody else: what says a workspace is installed could have been
//!   written by something other than the system, and reinstalling the server
//!   is what writes it again;
//! - [`Unhosted::NotRead`] — the machine would not read it: a restart, and
//!   failing that a reinstall;
//! - [`Unhosted::NoUsablePort`] — it was read and does not say one port a
//!   workspace can be reached on: not TOML, another key, no port, not a
//!   number, not a port, or the wire's own port — the server's package is what
//!   writes it, so reinstalling or updating it is the act.
//!
//! The grouping is made **once, as the service starts**, from the refusal the
//! start already made, and kept on [`crate::wire::Wire`]: nothing here reads
//! the file, and asking what is advertised cannot cause it to be read again.
//! Keeping the group rather than the refusal is also what keeps the path out of
//! the running service's answer — there is nothing left to leak.

use alo_strings::{Filling, Said, Strings};

use crate::refusing::NotHosting;
use crate::words::{
    A_WORKSPACE_HERE_COULD_NOT_BE_READ, A_WORKSPACE_HERE_IS_NOT_THE_SYSTEMS,
    A_WORKSPACE_HERE_NAMES_NO_USABLE_PORT,
};

/// Why a workspace installed on this machine is not advertised, as the person
/// can act on it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unhosted {
    /// What says a workspace is installed could have been written by something
    /// other than the system.
    NotTheSystems,
    /// What says a workspace is installed could not be read.
    NotRead,
    /// What says a workspace is installed does not name one usable port.
    NoUsablePort,
}

impl Unhosted {
    /// The group a refusal of the workspace file belongs to.
    ///
    /// Exhaustive, so a twelfth way a file can be refused has to be given a
    /// group before this crate compiles.
    #[must_use]
    pub const fn of(refused: &NotHosting) -> Self {
        match refused {
            NotHosting::ALink { .. }
            | NotHosting::NotAFile { .. }
            | NotHosting::NotRoots { .. }
            | NotHosting::WritableByOthers { .. } => Self::NotTheSystems,
            NotHosting::NotRead { .. } => Self::NotRead,
            NotHosting::NotTheShape { .. }
            | NotHosting::AnotherKey { .. }
            | NotHosting::NoPort { .. }
            | NotHosting::NotANumber { .. }
            | NotHosting::NotAPort { .. }
            | NotHosting::TheWiresOwnPort { .. } => Self::NoUsablePort,
        }
    }

    /// What the person is told, in their language.
    ///
    /// No gap in any of the three sentences: nothing about the file — its path,
    /// its owner, its mode or what it said — is put into words a person reads.
    #[must_use]
    pub fn said(self, strings: &Strings) -> Said {
        let word = match self {
            Self::NotTheSystems => A_WORKSPACE_HERE_IS_NOT_THE_SYSTEMS,
            Self::NotRead => A_WORKSPACE_HERE_COULD_NOT_BE_READ,
            Self::NoUsablePort => A_WORKSPACE_HERE_NAMES_NO_USABLE_PORT,
        };
        strings.say(&word.key(), &Filling::nothing())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::hosting::THE_HOSTED_WORKSPACE;
    use crate::testing::in_english;
    use crate::wire::THE_WIRE_PORT;

    /// Every way the file can be refused, each with the group it belongs to.
    fn every_refusal() -> Vec<(NotHosting, Unhosted)> {
        let at = PathBuf::from(THE_HOSTED_WORKSPACE);
        vec![
            (
                NotHosting::ALink { at: at.clone() },
                Unhosted::NotTheSystems,
            ),
            (
                NotHosting::NotAFile { at: at.clone() },
                Unhosted::NotTheSystems,
            ),
            (
                NotHosting::NotRoots {
                    at: at.clone(),
                    owner: 1_000,
                },
                Unhosted::NotTheSystems,
            ),
            (
                NotHosting::WritableByOthers {
                    at: at.clone(),
                    mode: 0o666,
                },
                Unhosted::NotTheSystems,
            ),
            (
                NotHosting::NotRead {
                    at: at.clone(),
                    why: std::io::Error::from(std::io::ErrorKind::PermissionDenied),
                },
                Unhosted::NotRead,
            ),
            (
                NotHosting::NotTheShape {
                    at: at.clone(),
                    why: "expected `=`".to_owned(),
                },
                Unhosted::NoUsablePort,
            ),
            (
                NotHosting::AnotherKey {
                    at: at.clone(),
                    key: "name".to_owned(),
                },
                Unhosted::NoUsablePort,
            ),
            (
                NotHosting::NoPort { at: at.clone() },
                Unhosted::NoUsablePort,
            ),
            (
                NotHosting::NotANumber {
                    at: at.clone(),
                    said: "\"8443\"".to_owned(),
                },
                Unhosted::NoUsablePort,
            ),
            (
                NotHosting::NotAPort {
                    at: at.clone(),
                    port: 65_536,
                },
                Unhosted::NoUsablePort,
            ),
            (
                NotHosting::TheWiresOwnPort {
                    at,
                    port: THE_WIRE_PORT,
                },
                Unhosted::NoUsablePort,
            ),
        ]
    }

    /// **Every refusal of the workspace file is in the group a person can act
    /// on**: the four about who could have written it, the one about reading
    /// it, and the six about what it says.
    #[test]
    fn every_refusal_of_the_workspace_file_is_grouped_into_what_a_person_can_act_on() {
        let refusals = every_refusal();
        assert_eq!(refusals.len(), 11, "a refusal is missing from this test");
        for (refused, group) in &refusals {
            assert_eq!(Unhosted::of(refused), *group, "{refused:?}");
        }
    }

    /// **What the person is told names no path, no owner and no mode**, for
    /// every refusal — and the three groups are three different sentences, each
    /// one this crate declares rather than a key nobody wrote.
    #[test]
    fn what_the_person_is_told_names_no_path_owner_or_mode() {
        let strings = in_english();
        for (refused, _) in every_refusal() {
            let said = Unhosted::of(&refused).said(&strings);
            assert!(!said.is_a_bug(), "{refused:?}: {}", said.text());
            let text = said.text();
            for leaked in [
                "/",
                "workspace.toml",
                "etc",
                "uid",
                "1000",
                "root",
                "666",
                "mode",
                "65536",
                "8443",
                "7610",
                "name",
                "`",
            ] {
                assert!(
                    !text.contains(leaked),
                    "{refused:?} says {leaked:?}: {text}"
                );
            }
            assert_ne!(text, refused.to_string());
        }
        let mut sentences: Vec<String> = [
            Unhosted::NotTheSystems,
            Unhosted::NotRead,
            Unhosted::NoUsablePort,
        ]
        .into_iter()
        .map(|group| group.said(&strings).text().to_owned())
        .collect();
        sentences.sort();
        sentences.dedup();
        assert_eq!(sentences.len(), 3);
    }
}
