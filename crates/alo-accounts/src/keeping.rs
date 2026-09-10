//! The store on the disk: where it is, who may have written it, and how it
//! is replaced without ever being half a file.
//!
//! # Who may write it
//!
//! The machine description's rule, kept for the machine description's
//! reason: whoever can rewrite this file names who signs in. So before a
//! byte of it is parsed —
//!
//! - the path is **not a symbolic link** (`O_NOFOLLOW`);
//! - it belongs to **root or to the login reading it**, and to nobody else;
//! - **nobody else can write it** — group- or world-writable is refused.
//!
//! Checked on the open file rather than on the path, so the store that was
//! checked and the store that is read cannot be two different files.
//!
//! # Replaced whole or not at all
//!
//! [`kept`] writes a sibling file (`accounts.toml.new`, mode `0600`), syncs
//! it, and renames it over the store. A machine that loses power mid-write
//! boots with the store it had — a torn accounts file would be a machine
//! nobody can sign in to, which on a workstation is a machine that is off.
//!
//! The folder is **not** created here. `/etc/alo` is the image's
//! (`agentd.toml` ships in it); a missing folder means this is not an alo OS
//! machine, and making one would turn a typo in a path into a second store
//! nobody is reading — `alo-agentd`'s rule about the record's folder.

use std::io::{Read, Write};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};

use crate::refusing::NotKept;
use crate::store::Accounts;

/// Where the machine's accounts are.
///
/// Beside the machine description, because the two files together are what
/// the machine says about who it belongs to.
pub const THE_ACCOUNTS: &str = "/etc/alo/accounts.toml";

/// The mode bits that let the group or the world write.
const OTHERS_MAY_WRITE: u32 = 0o022;

/// The mode a fresh store is created with: the owner and nobody else.
const OURS_ALONE: u32 = 0o600;

/// The store at this path, believed and read.
///
/// # Errors
///
/// [`NotKept::NotThere`] when no store exists — the first-boot state, told
/// apart from every failure; [`NotKept::ALink`] for a symbolic link;
/// [`NotKept::SomebodyElses`] and [`NotKept::WritableByOthers`] for a file
/// whoever else could have written; and everything [`Accounts::read`]
/// refuses about the text itself.
pub fn found(at: &Path) -> Result<Accounts, NotKept> {
    let mut file = match std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(o_nofollow())
        .open(at)
    {
        Ok(file) => file,
        Err(why) if why.kind() == std::io::ErrorKind::NotFound => {
            return Err(NotKept::NotThere { at: at.to_owned() });
        }
        // `ELOOP` is what `O_NOFOLLOW` answers a link with; the named
        // `ErrorKind` for it is not yet stable, so the number is compared.
        Err(why) if why.raw_os_error() == Some(rustix::io::Errno::LOOP.raw_os_error()) => {
            return Err(NotKept::ALink { at: at.to_owned() });
        }
        Err(why) => {
            return Err(NotKept::NotRead {
                at: at.to_owned(),
                why: why.to_string(),
            });
        }
    };
    let seen = file.metadata().map_err(|why| NotKept::NotRead {
        at: at.to_owned(),
        why: why.to_string(),
    })?;
    believed(at, seen.uid(), seen.mode(), us())?;
    let mut text = String::new();
    file.read_to_string(&mut text)
        .map_err(|why| NotKept::NotRead {
            at: at.to_owned(),
            why: why.to_string(),
        })?;
    Accounts::read(&text)
}

/// The store, written whole to this path.
///
/// # Errors
///
/// [`NotKept::NotWritten`] naming what the machine said — including a folder
/// that is not there, which is refused rather than made.
pub fn kept(at: &Path, accounts: &Accounts) -> Result<(), NotKept> {
    let text = accounts.written()?;
    let fresh = a_sibling_of(at);
    // A stale sibling from a write that died is removed so `create_new`
    // below can insist the one being written is ours alone.
    if let Err(why) = std::fs::remove_file(&fresh)
        && why.kind() != std::io::ErrorKind::NotFound
    {
        return Err(NotKept::NotWritten {
            at: at.to_owned(),
            why: why.to_string(),
        });
    }
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(OURS_ALONE)
        .custom_flags(o_nofollow())
        .open(&fresh)
        .map_err(|why| NotKept::NotWritten {
            at: at.to_owned(),
            why: why.to_string(),
        })?;
    file.write_all(text.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|why| NotKept::NotWritten {
            at: at.to_owned(),
            why: why.to_string(),
        })?;
    drop(file);
    std::fs::rename(&fresh, at).map_err(|why| NotKept::NotWritten {
        at: at.to_owned(),
        why: why.to_string(),
    })
}

/// Whether a file with this owner and mode is one to believe, as a rule of
/// its own so every branch of it is testable without root.
fn believed(at: &Path, owner: u32, mode: u32, us: u32) -> Result<(), NotKept> {
    if owner != 0 && owner != us {
        return Err(NotKept::SomebodyElses {
            at: at.to_owned(),
            owner,
        });
    }
    if mode & OTHERS_MAY_WRITE != 0 {
        return Err(NotKept::WritableByOthers {
            at: at.to_owned(),
            mode: mode & 0o777,
        });
    }
    Ok(())
}

/// The user this process runs as, asked of the kernel rather than of an
/// environment.
fn us() -> u32 {
    rustix::process::geteuid().as_raw()
}

/// The flag that refuses to open a symbolic link, as `OpenOptions` takes it.
#[expect(
    clippy::cast_possible_wrap,
    reason = "O_NOFOLLOW is a flag bit pattern; the kernel reads it as bits either way"
)]
fn o_nofollow() -> i32 {
    rustix::fs::OFlags::NOFOLLOW.bits() as i32
}

/// The path the store is staged at before it replaces the real one.
fn a_sibling_of(at: &Path) -> PathBuf {
    let mut named = at.as_os_str().to_owned();
    named.push(".new");
    PathBuf::from(named)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    /// A folder of this test's own, emptied of any earlier run.
    fn a_folder_of_our_own(what: &str) -> PathBuf {
        let at = std::env::temp_dir().join(format!("alo-accounts-{what}-{}", std::process::id()));
        let _cleared = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).unwrap();
        at
    }

    /// A store with one account, kept on the disk at this folder.
    fn adas_store_kept_in(folder: &Path) -> PathBuf {
        let mut store = Accounts::none().unwrap();
        store
            .created("ada", 1000, "correct horse battery staple")
            .unwrap();
        let at = folder.join("accounts.toml");
        kept(&at, &store).unwrap();
        at
    }

    /// **What was kept is found again and signs in** — a reboot, as far as
    /// the disk is concerned — and it went down with the owner-only mode.
    #[test]
    fn a_store_kept_is_found_with_nobody_else_able_to_write_it() {
        let folder = a_folder_of_our_own("round-trip");
        let at = adas_store_kept_in(&folder);

        let mode = std::fs::metadata(&at).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, OURS_ALONE, "the store went down mode {mode:o}");
        assert!(
            !folder.join("accounts.toml.new").exists(),
            "the staging file outlived the rename"
        );

        let found = found(&at).unwrap();
        assert!(
            found
                .signs_in("ada", "correct horse battery staple")
                .is_ok()
        );
    }

    /// **A machine with no store yet is told exactly that**, apart from every
    /// other failure, because first boot is not an error.
    #[test]
    fn a_machine_with_no_store_yet_is_not_an_error_story() {
        let folder = a_folder_of_our_own("not-there");
        let refused = found(&folder.join("accounts.toml"));
        assert!(matches!(refused, Err(NotKept::NotThere { .. })));
    }

    /// **A symbolic link is refused as one**, never followed — a link is a
    /// name somebody can point at a file they own.
    #[test]
    fn a_store_behind_a_symbolic_link_is_refused() {
        let folder = a_folder_of_our_own("symlink");
        let real = adas_store_kept_in(&folder);
        let link = folder.join("linked.toml");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        assert!(matches!(found(&link), Err(NotKept::ALink { .. })));
    }

    /// **A store the group or the world can write is refused**, because
    /// whoever can write it names who signs in.
    #[test]
    fn a_store_somebody_else_could_write_is_refused() {
        let folder = a_folder_of_our_own("writable");
        let at = adas_store_kept_in(&folder);
        std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o666)).unwrap();

        assert!(matches!(
            found(&at),
            Err(NotKept::WritableByOthers { mode: 0o666, .. })
        ));
    }

    /// **The ownership rule, every branch** — a test cannot chown a file to
    /// somebody else without root, so the rule is a function and this walks
    /// it: root's file is believed, ours is believed, anybody else's is not,
    /// and a believable owner does not excuse writable modes.
    #[test]
    fn only_roots_store_or_our_own_is_believed() {
        let at = Path::new("/etc/alo/accounts.toml");
        assert!(believed(at, 0, OURS_ALONE, 1000).is_ok());
        assert!(believed(at, 1000, OURS_ALONE, 1000).is_ok());
        assert!(matches!(
            believed(at, 1001, OURS_ALONE, 1000),
            Err(NotKept::SomebodyElses { owner: 1001, .. })
        ));
        assert!(matches!(
            believed(at, 0, 0o620, 1000),
            Err(NotKept::WritableByOthers { .. })
        ));
        assert!(matches!(
            believed(at, 1000, 0o602, 1000),
            Err(NotKept::WritableByOthers { .. })
        ));
    }

    /// **A folder that is not there is refused, not made** — `/etc/alo` is
    /// the image's, and a folder made here would turn a typo into a second
    /// store nobody reads.
    #[test]
    fn a_missing_folder_is_refused_rather_than_made() {
        let folder = a_folder_of_our_own("no-folder");
        let nowhere = folder.join("not-made").join("accounts.toml");
        let store = Accounts::none().unwrap();

        assert!(matches!(
            kept(&nowhere, &store),
            Err(NotKept::NotWritten { .. })
        ));
        assert!(!nowhere.parent().unwrap().exists(), "the folder was made");
    }

    /// **A failed or stale staging file does not wedge the next write.** A
    /// leftover `.new` from a write that died is cleared and the keep
    /// succeeds.
    #[test]
    fn a_stale_staging_file_does_not_stop_the_next_keep() {
        let folder = a_folder_of_our_own("stale");
        let at = folder.join("accounts.toml");
        std::fs::write(folder.join("accounts.toml.new"), "half a file").unwrap();

        let mut store = Accounts::none().unwrap();
        store
            .created("ada", 1000, "correct horse battery staple")
            .unwrap();
        kept(&at, &store).unwrap();

        assert!(
            found(&at)
                .unwrap()
                .signs_in("ada", "correct horse battery staple")
                .is_ok()
        );
    }
}
