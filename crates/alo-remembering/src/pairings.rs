//! The pairings file on the disk: where it is, and the whole-or-nothing
//! replacement.
//!
//! `alo-nearby` decides what a pairing is and how one is written down
//! (`alo_nearby::keeping`): the row two people made, its enumerated list,
//! when it was made and ends, and the key the two machines agreed (ADR 0031).
//! This file decides where that text lives and who may have written it — and
//! it is the same answer the grants get, from [`crate::believing`], because
//! whoever can rewrite this file says which machines may ask this one.
//!
//! # `/var/lib/alo/pairings.toml`, beside the grants and the identity
//!
//! The same folder, for the same reason, and one more: the identity the key
//! is paired *to* (`/var/lib/alo/machine-id`) is already there. A pairing is
//! two things — who this machine is and what it agreed with another — and a
//! machine that kept the two in two stores under two rules would be a machine
//! that could wake with one and not the other.
//!
//! # A pairing that has ended is gone when the list is read
//!
//! [`pairings_remembered`] hands the moment to `alo_nearby::keeping::read`,
//! which drops every row that has ended before the list exists — so a pairing
//! that ran out while the machine was switched off is not on the list the
//! machine wakes with, and its **expiry survives a restart** as a property of
//! the reading rather than of whoever remembers to check. The daemon writes
//! this file itself, at the moment a pairing is kept or revoked, which is the
//! one difference from the grants file: a pairing is made by two people on
//! two machines, and the daemon is the thing that hears the second of them.

use std::path::Path;
use std::time::SystemTime;

use alo_nearby::Pairings;

use crate::believing::{read_believed, replaced_whole};
use crate::refusing::NotRemembered;

/// Where a machine keeps its pairings.
pub const THE_PAIRINGS: &str = "/var/lib/alo/pairings.toml";

/// The pairings kept at this path, believed, read, and already free of the
/// ones that have ended.
///
/// # Errors
///
/// [`NotRemembered::NotThere`] when this machine has paired with nothing yet
/// — told apart from every failure; [`NotRemembered::ALink`] for a symbolic
/// link; [`NotRemembered::SomebodyElses`] and
/// [`NotRemembered::WritableByOthers`] for a file somebody else could have
/// written; and [`NotRemembered::NotPairings`] for everything
/// `alo_nearby::keeping::read` refuses about the text itself.
pub fn pairings_remembered(at: &Path, now: SystemTime) -> Result<Pairings, NotRemembered> {
    let text = read_believed(at)?;
    Ok(alo_nearby::keeping::read(&text, now)?)
}

/// What is paired at this moment, written whole to this path.
///
/// # Errors
///
/// [`NotRemembered::NotWritten`] naming what the machine said — including a
/// folder that is not there, which is refused rather than made — and
/// [`NotRemembered::NotPairings`] for a pairing that cannot be written down.
pub fn pairings_kept(at: &Path, pairings: &Pairings, now: SystemTime) -> Result<(), NotRemembered> {
    let text = alo_nearby::keeping::written(pairings, now)?;
    replaced_whole(at, &text)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::believing::OURS_ALONE;
    use crate::testing::{a_folder_of_our_own, noon, the_studio_paired_with_reception};
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    /// The studio's pairings, kept on the disk in this folder.
    fn pairings_kept_in(folder: &Path) -> PathBuf {
        let at = folder.join("pairings.toml");
        let (pairings, _) = the_studio_paired_with_reception();
        pairings_kept(&at, &pairings, noon()).unwrap();
        at
    }

    /// **What was kept is found again, owner-only**, and the staging file did
    /// not outlive the rename.
    #[test]
    fn pairings_kept_are_found_with_nobody_else_able_to_write_them() {
        let folder = a_folder_of_our_own("pairings-round-trip");
        let at = pairings_kept_in(&folder);
        let mode = std::fs::metadata(&at).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, OURS_ALONE, "the pairings went down mode {mode:o}");
        assert!(!folder.join("pairings.toml.new").exists());
        assert_eq!(pairings_remembered(&at, noon()).unwrap().every().len(), 1);
    }

    /// A machine that has paired with nothing is told exactly that.
    #[test]
    fn a_machine_paired_with_nothing_yet_is_not_an_error_story() {
        let folder = a_folder_of_our_own("pairings-not-there");
        assert!(matches!(
            pairings_remembered(&folder.join("pairings.toml"), noon()),
            Err(NotRemembered::NotThere { .. })
        ));
    }

    /// **A symbolic link and a file somebody else could write are each
    /// refused whole**, and nothing is read out of either.
    #[test]
    fn pairings_behind_a_link_or_writable_by_others_are_refused() {
        let folder = a_folder_of_our_own("pairings-untrusted");
        let real = pairings_kept_in(&folder);
        let link = folder.join("linked.toml");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        assert!(matches!(
            pairings_remembered(&link, noon()),
            Err(NotRemembered::ALink { .. })
        ));
        std::fs::set_permissions(&real, std::fs::Permissions::from_mode(0o666)).unwrap();
        assert!(matches!(
            pairings_remembered(&real, noon()),
            Err(NotRemembered::WritableByOthers { mode: 0o666, .. })
        ));
    }

    /// **A file hand-edited into a pairing this machine would not make is
    /// refused whole**, in `alo-nearby`'s words, and nothing is read.
    #[test]
    fn a_hand_edited_pairing_is_refused_whole() {
        let folder = a_folder_of_our_own("pairings-edited");
        let at = pairings_kept_in(&folder);
        let text = std::fs::read_to_string(&at).unwrap();
        std::fs::write(
            &at,
            text.replace("may = [\"models\"]", "may = [\"everything\"]"),
        )
        .unwrap();
        let refused = pairings_remembered(&at, noon()).unwrap_err();
        assert!(
            matches!(refused, NotRemembered::NotPairings(_)),
            "{refused}"
        );
        assert!(refused.to_string().contains("everything"), "{refused}");
    }

    /// A folder that is not there is refused, not made.
    #[test]
    fn a_missing_folder_is_refused_rather_than_made() {
        let folder = a_folder_of_our_own("pairings-no-folder");
        let nowhere = folder.join("not-made").join("pairings.toml");
        let (pairings, _) = the_studio_paired_with_reception();
        assert!(matches!(
            pairings_kept(&nowhere, &pairings, noon()),
            Err(NotRemembered::NotWritten { .. })
        ));
        assert!(!nowhere.parent().unwrap().exists());
    }
}
