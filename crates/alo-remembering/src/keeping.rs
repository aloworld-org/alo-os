//! The grants file on the disk: where it is, and the whole-or-nothing
//! replacement.
//!
//! Who may have written it, and how it is replaced without ever being half a
//! file, are [`crate::believing`]'s — one rule for this file and for the
//! pairings beside it (`pairings.rs`), because whoever can rewrite either says
//! what this machine's agent may do.
//!
//! # `/var/lib/alo/grants.toml`, and why it is there
//!
//! It is the person's own state and it has to outlive their session, which
//! rules out `/run` — `/run/user/<uid>` and `/run/alo/<uid>` are both gone at
//! sign-out, and *between one sign-in and the next* is the whole of what this
//! crate is for. `/var/lib/alo` is the directory the image already makes for
//! that: `0700 alo alo` in `image/usr/lib/tmpfiles.d/alo.conf`, beside the
//! record, because what an agent did on somebody's machine is theirs and so is
//! what they let it reach.
//!
//! The path is a constant rather than a key in the machine description, and
//! that is a decision. `[record].path` is configurable because ADR 0004 gives
//! retention to whoever manages the machine; where the grants live is nobody's
//! policy, and a second copy of the answer is a second file for somebody to
//! point somewhere the daemon is not reading.

use std::path::Path;
use std::time::SystemTime;

use alo_capability::Grants;

use crate::believing::{read_believed, replaced_whole};
use crate::refusing::NotRemembered;

/// Where a machine keeps its grants.
///
/// In the folder the image makes for what an agent did on this machine, beside
/// the record — the two files that together say what was allowed and what
/// happened.
pub const THE_GRANTS: &str = "/var/lib/alo/grants.toml";

/// The grants kept at this path, believed, read, and already free of the
/// expired ones.
///
/// # Errors
///
/// [`NotRemembered::NotThere`] when nothing has been granted on this machine
/// yet — told apart from every failure; [`NotRemembered::ALink`] for a symbolic
/// link; [`NotRemembered::SomebodyElses`] and
/// [`NotRemembered::WritableByOthers`] for a file somebody else could have
/// written; and everything [`crate::read`] refuses about the text itself.
pub fn remembered(at: &Path, now: SystemTime) -> Result<Grants, NotRemembered> {
    let text = read_believed(at)?;
    crate::read(&text, now)
}

/// What is granted at this moment, written whole to this path.
///
/// # Errors
///
/// [`NotRemembered::NotWritten`] naming what the machine said — including a
/// folder that is not there, which is refused rather than made — and everything
/// [`crate::written`] refuses about the grants themselves.
pub fn kept(at: &Path, grants: &Grants, now: SystemTime) -> Result<(), NotRemembered> {
    let text = crate::written(grants, now)?;
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
    use crate::testing::{HERS, a_folder_of_our_own, an_hour, granted_in, noon};
    use alo_capability::{Ask, Grantee};
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    /// A machine's grants, kept on the disk in this folder.
    fn grants_kept_in(folder: &Path) -> PathBuf {
        let at = folder.join("grants.toml");
        kept(&at, &granted_in("/home/ada/Invoices"), noon()).unwrap();
        at
    }

    /// **What was kept is found again and still permits what it permitted** — a
    /// restart, as far as the disk is concerned — and it went down with the
    /// owner-only mode.
    #[test]
    fn grants_kept_are_found_with_nobody_else_able_to_write_them() {
        let folder = a_folder_of_our_own("round-trip");
        let at = grants_kept_in(&folder);

        let mode = std::fs::metadata(&at).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, OURS_ALONE, "the grants went down mode {mode:o}");
        assert!(
            !folder.join("grants.toml.new").exists(),
            "the staging file outlived the rename"
        );

        let found = remembered(&at, noon()).unwrap();
        assert!(found.permits(
            &Grantee::named(HERS),
            &Ask::path("/home/ada/Invoices/march.pdf"),
            noon()
        ));
    }

    /// **A machine that has granted nothing yet is told exactly that**, apart
    /// from every other failure: nothing granted is not a broken machine.
    #[test]
    fn a_machine_with_nothing_granted_yet_is_not_an_error_story() {
        let folder = a_folder_of_our_own("not-there");
        let refused = remembered(&folder.join("grants.toml"), noon());
        assert!(matches!(refused, Err(NotRemembered::NotThere { .. })));
    }

    /// **A symbolic link is refused as one**, never followed — a link is a name
    /// somebody can point at a file they own, and whoever writes this file says
    /// what the agent may reach.
    #[test]
    fn grants_behind_a_symbolic_link_are_refused() {
        let folder = a_folder_of_our_own("symlink");
        let real = grants_kept_in(&folder);
        let link = folder.join("linked.toml");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        assert!(matches!(
            remembered(&link, noon()),
            Err(NotRemembered::ALink { .. })
        ));
    }

    /// **A file the group or the world can write is refused**, and nothing is
    /// read out of it.
    #[test]
    fn grants_somebody_else_could_write_are_refused() {
        let folder = a_folder_of_our_own("writable");
        let at = grants_kept_in(&folder);
        std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o666)).unwrap();

        assert!(matches!(
            remembered(&at, noon()),
            Err(NotRemembered::WritableByOthers { mode: 0o666, .. })
        ));
    }

    /// **A folder that is not there is refused, not made** — `/var/lib/alo` is
    /// the image's, and a folder made here would turn a typo into a second list
    /// nobody reads.
    #[test]
    fn a_missing_folder_is_refused_rather_than_made() {
        let folder = a_folder_of_our_own("no-folder");
        let nowhere = folder.join("not-made").join("grants.toml");

        assert!(matches!(
            kept(&nowhere, &Grants::default(), noon()),
            Err(NotRemembered::NotWritten { .. })
        ));
        assert!(!nowhere.parent().unwrap().exists(), "the folder was made");
    }

    /// **A stale staging file does not wedge the next write.** A leftover
    /// `.new` from a write that died is cleared and the keep succeeds.
    #[test]
    fn a_stale_staging_file_does_not_stop_the_next_keep() {
        let folder = a_folder_of_our_own("stale");
        std::fs::write(folder.join("grants.toml.new"), "half a file").unwrap();

        let at = grants_kept_in(&folder);
        assert_eq!(remembered(&at, noon()).unwrap().len(), 1);
    }

    /// **Keeping the grants again replaces them whole**, so a revoked grant is
    /// not left behind in the file it was revoked out of.
    #[test]
    fn keeping_again_replaces_the_file_rather_than_adding_to_it() {
        let folder = a_folder_of_our_own("replaced");
        let at = grants_kept_in(&folder);

        let mut grants = remembered(&at, noon()).unwrap();
        let held = grants.active_at(noon()).next().unwrap().id;
        assert!(grants.revoke(held));
        kept(&at, &grants, noon()).unwrap();

        let after = remembered(&at, noon()).unwrap();
        assert!(after.is_empty(), "a revoked grant was still in the file");
        assert!(!after.permits(
            &Grantee::named(HERS),
            &Ask::path("/home/ada/Invoices/march.pdf"),
            noon() + an_hour()
        ));
    }
}
