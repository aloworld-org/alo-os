//! The file this machine really keeps the names of paired machines in, as the
//! thing [`crate::names::TheNames`] writes through.
//!
//! `crate::keeping_pairings`' twin for the file beside it, with the same roles:
//! written by this service at the moment the person names a machine, takes a
//! name away, or revokes a pairing — and read once, at start, in `src/main.rs`.
//! [`TheNamesFile`] writes and reads nothing else, and holds the one path
//! `main` gave it.
//!
//! **Nothing an agent sends reaches it.** Every request that changes a name is
//! a person's (`alo_protocol::FromAPerson`), refused on the agent's door before
//! it reaches `crate::naming_machines`.

use std::path::{Path, PathBuf};

use alo_remembering::{MachineNames, NotRemembered};

use crate::names::KeepingNames;

/// The file this machine keeps the names of paired machines in.
///
/// Holds a path and nothing else, and the path is `src/main.rs`'s — the one
/// place in this service that names `alo_remembering::THE_MACHINE_NAMES`.
#[derive(Debug, Clone)]
pub struct TheNamesFile {
    /// Where the names are.
    at: PathBuf,
}

impl TheNamesFile {
    /// The names at this path.
    #[must_use]
    pub fn at(at: &Path) -> Self {
        Self { at: at.to_owned() }
    }

    /// Where the names are, for a service log and for a test that reads the
    /// file back as a restart would.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.at
    }
}

impl KeepingNames for TheNamesFile {
    /// `alo-remembering`'s own writing: whole, owner-only, never half a file.
    fn keep(&self, names: &MachineNames) -> Result<(), NotRemembered> {
        alo_remembering::machine_names_kept(&self.at, names)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alo_nearby::{MayAskIts, Pairings};
    use alo_remembering::MachineName;

    use super::*;
    use crate::testing::{a_directory_of_our_own, noon, paired_between, reception, the_studio};

    /// **What the service writes is what a restart reads back**, through
    /// `alo-remembering`'s own reader, while the pairing stands.
    #[test]
    fn what_the_service_writes_is_what_a_restart_reads_back() {
        let file =
            TheNamesFile::at(&a_directory_of_our_own("names-file").join("machine-names.toml"));
        let mut names = MachineNames::none();
        names.name(
            reception(),
            MachineName::checked("the reception machine").unwrap(),
        );
        file.keep(&names).unwrap();
        let (_, on_studio) =
            paired_between(reception(), the_studio(), &[MayAskIts::Models], noon());
        let mut pairings = Pairings::none();
        pairings.keep(on_studio);
        assert_eq!(
            alo_remembering::machine_names_remembered(file.path(), &pairings, noon()).unwrap(),
            names
        );
    }

    /// A folder that is not there is refused in `alo-remembering`'s words.
    #[test]
    fn a_folder_that_is_not_there_is_refused_rather_than_made() {
        let nowhere = a_directory_of_our_own("names-nowhere")
            .join("not-made")
            .join("machine-names.toml");
        assert!(matches!(
            TheNamesFile::at(&nowhere).keep(&MachineNames::none()),
            Err(NotRemembered::NotWritten { .. })
        ));
        assert!(!nowhere.parent().unwrap().exists());
    }
}
