//! The file this machine really keeps its pairings in, as the thing the
//! network's lock writes through.
//!
//! `crate::rereading`'s twin for the other file in `/var/lib/alo`, with the
//! roles the other way round: the grants are written by the person's side and
//! only ever **read** by this service, and the pairings are written by this
//! service — at the moment a pairing is kept or revoked — and read by it once,
//! at start, in `src/main.rs`. So [`ThePairingsFile`] implements
//! [`crate::network::KeepingPairings`], which writes and reads nothing, and
//! the reading is `main`'s alone: what travels below `main` is a value and a
//! way to write the list whole, and there is no road from either door to a
//! path.
//!
//! **Nothing an agent sends reaches it.** Every request that changes the
//! pairings is a person's (`alo_protocol::FromAPerson`) and is refused on the
//! agent's door before it reaches `crate::pairing`; the one other road is a
//! confirmation arriving on the wire, which completes a pairing only if this
//! machine's person confirmed the same one (`crate::hearing`).

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use alo_nearby::Pairings;
use alo_remembering::NotRemembered;

use crate::network::KeepingPairings;

/// The file this machine keeps its pairings in.
///
/// Holds a path and nothing else, and the path is `src/main.rs`'s — the one
/// place in this service that names `alo_remembering::THE_PAIRINGS`.
#[derive(Debug, Clone)]
pub struct ThePairingsFile {
    /// Where the pairings are.
    at: PathBuf,
}

impl ThePairingsFile {
    /// The pairings at this path.
    #[must_use]
    pub fn at(at: &Path) -> Self {
        Self { at: at.to_owned() }
    }

    /// Where the pairings are, for a service log and for a test that has to
    /// read the file back as a restart would.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.at
    }
}

impl KeepingPairings for ThePairingsFile {
    /// `alo-remembering`'s own writing: whole, owner-only, and never half a
    /// file.
    fn keep(&self, pairings: &Pairings, now: SystemTime) -> Result<(), NotRemembered> {
        alo_remembering::pairings_kept(&self.at, pairings, now)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use alo_nearby::{MayAskIts, Pairings};

    use super::*;
    use crate::testing::{a_directory_of_our_own, noon, paired_between, reception, the_studio};

    /// **What the service writes is what a restart reads back**, through
    /// `alo-remembering`'s own reader with its three rules.
    #[test]
    fn what_the_service_writes_is_what_a_restart_reads_back() {
        let at = a_directory_of_our_own("pairings-file").join("pairings.toml");
        let file = ThePairingsFile::at(&at);
        let (_, on_studio) =
            paired_between(reception(), the_studio(), &[MayAskIts::Models], noon());
        let mut pairings = Pairings::none();
        pairings.keep(on_studio);
        file.keep(&pairings, noon()).unwrap();

        let back = alo_remembering::pairings_remembered(file.path(), noon()).unwrap();
        assert!(back.paired_with(&reception(), noon()));
        assert_eq!(
            back, pairings,
            "the row that came back is not the row that went down"
        );
    }

    /// A folder that is not there is refused in `alo-remembering`'s words,
    /// and nothing is made.
    #[test]
    fn a_folder_that_is_not_there_is_refused_rather_than_made() {
        let nowhere = a_directory_of_our_own("pairings-nowhere")
            .join("not-made")
            .join("pairings.toml");
        let file = ThePairingsFile::at(&nowhere);
        assert!(matches!(
            file.keep(&Pairings::none(), noon()),
            Err(NotRemembered::NotWritten { .. })
        ));
        assert!(!nowhere.parent().unwrap().exists());
    }
}
