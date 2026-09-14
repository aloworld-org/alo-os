//! The names file on the disk: where it is, and who is believed about it.
//!
//! What a name is and how the list is written down are `crate::named` and
//! `crate::names`; this is where that text lives, under the rule every file in
//! this crate is held to (`crate::believing`).
//!
//! # `/var/lib/alo/machine-names.toml`, beside the pairings
//!
//! The same folder as the pairings file and the grants, with the same trust:
//! not a symbolic link, owned by root or by the login reading it, writable by
//! nobody else, written `0600` and replaced whole. A name decides nothing, and
//! it is still held to that, because it is what a person reads on an indicator
//! and in a record: whoever could rewrite this file could put one machine's name
//! on another machine's evidence.
//!
//! A file of its own rather than a field in the pairings file, for
//! `crate::names`' reason: a pairings row is exactly what two people made, and
//! a name is one person's.

use std::path::Path;
use std::time::SystemTime;

use alo_nearby::Pairings;

use crate::believing::{read_believed, replaced_whole};
use crate::names::MachineNames;
use crate::refusing::NotRemembered;

/// Where a machine keeps the names its person gave the machines it is paired
/// with.
pub const THE_MACHINE_NAMES: &str = "/var/lib/alo/machine-names.toml";

/// The names kept at this path, believed, read, and only for machines a pairing
/// with stands at `now`.
///
/// # Errors
///
/// [`NotRemembered::NotThere`] when nobody has named anything yet — told apart
/// from every failure; [`NotRemembered::ALink`],
/// [`NotRemembered::SomebodyElses`] and [`NotRemembered::WritableByOthers`]
/// for a file somebody else could have written; and
/// [`NotRemembered::NotMachineNames`] for everything `crate::names::read`
/// refuses about the text.
pub fn machine_names_remembered(
    at: &Path,
    pairings: &Pairings,
    now: SystemTime,
) -> Result<MachineNames, NotRemembered> {
    let text = read_believed(at)?;
    Ok(crate::names::read(&text, pairings, now)?)
}

/// These names, written whole to this path.
///
/// # Errors
///
/// [`NotRemembered::NotWritten`] naming what the machine said — including a
/// folder that is not there, which is refused rather than made.
pub fn machine_names_kept(at: &Path, names: &MachineNames) -> Result<(), NotRemembered> {
    let text = crate::names::written(names)?;
    replaced_whole(at, &text)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use alo_nearby::MachineId;

    use super::*;
    use crate::believing::OURS_ALONE;
    use crate::named::MachineName;
    use crate::testing::{a_folder_of_our_own, noon, the_studio_paired_with_reception};

    /// Reception, named.
    fn named() -> MachineNames {
        let mut names = MachineNames::none();
        names.name(
            MachineId::read("0f1e2d3c4b5a69788796a5b4c3d2e1f0").unwrap(),
            MachineName::checked("the reception machine").unwrap(),
        );
        names
    }

    /// **What was kept is found again, owner-only**, while the pairing
    /// stands.
    #[test]
    fn names_kept_are_found_with_nobody_else_able_to_write_them() {
        let folder = a_folder_of_our_own("names-round-trip");
        let at = folder.join("machine-names.toml");
        machine_names_kept(&at, &named()).unwrap();
        let mode = std::fs::metadata(&at).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, OURS_ALONE, "the names went down mode {mode:o}");
        let (pairings, _) = the_studio_paired_with_reception();
        assert_eq!(
            machine_names_remembered(&at, &pairings, noon()).unwrap(),
            named()
        );
    }

    /// **Nobody has named anything yet** is told apart from a failure, and a
    /// file somebody else could write, or a link, is refused whole.
    #[test]
    fn names_not_there_or_not_believable_are_told_apart() {
        let folder = a_folder_of_our_own("names-untrusted");
        let (pairings, _) = the_studio_paired_with_reception();
        assert!(matches!(
            machine_names_remembered(&folder.join("machine-names.toml"), &pairings, noon()),
            Err(NotRemembered::NotThere { .. })
        ));
        let real = folder.join("machine-names.toml");
        machine_names_kept(&real, &named()).unwrap();
        let link = folder.join("linked.toml");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        assert!(matches!(
            machine_names_remembered(&link, &pairings, noon()),
            Err(NotRemembered::ALink { .. })
        ));
        std::fs::set_permissions(&real, std::fs::Permissions::from_mode(0o646)).unwrap();
        assert!(matches!(
            machine_names_remembered(&real, &pairings, noon()),
            Err(NotRemembered::WritableByOthers { mode: 0o646, .. })
        ));
    }
}
