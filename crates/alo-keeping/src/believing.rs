//! Who may have written a record, asked before a word of it is read.
//!
//! A record is what a person is shown when they ask what their machine did, and
//! what a security team reads afterwards. So whoever can write the file decides
//! what everybody is told happened on this machine — which is the same sentence
//! `alo-accounts` says about its store and `alo-remembering` says about the
//! grants, and it is answered here with exactly their three questions:
//!
//! - the path is **not a symbolic link** (`O_NOFOLLOW`);
//! - it belongs to **root or to the login reading it**, and to nobody else;
//! - **nobody else can write it** — group- or world-writable is refused.
//!
//! Asked of the open file rather than of the path, so the file that was checked
//! and the file that is read cannot be two different files.
//!
//! # It is a second door, not a rule laid over the first
//!
//! [`crate::Reading::at`] reads a record without asking any of this, and it goes
//! on doing so. The daemon opened the record it writes at start-up, holds it
//! open, and shortens the file it is already appending to; a service that
//! stopped writing down what its agent did because a mode bit changed underneath
//! it would be a machine that went quiet about exactly the afternoon somebody
//! would want to read. **A reader being shown an account is the other way
//! round**: nothing is lost by refusing, and what is at stake is a person
//! believing a file somebody else wrote. So [`crate::Reading::believed_at`] is
//! what a surface reads through, and `at` is what the writer's own side uses.
//!
//! # A host that cannot ask does not answer
//!
//! alo OS is Linux. On any other host there is no way to ask who owns a file or
//! who may write it, so nothing here pretends to have asked: the record is not
//! read at all, and the refusal carries the standard library's own word for a
//! thing this platform does not do rather than a sentence invented here.

use std::path::Path;

use crate::failing::NotKept;

/// The bytes of a record whose file this machine is willing to believe.
///
/// # Errors
///
/// [`NotKept::NotThere`] when there is no record there, which is never an empty
/// one; [`NotKept::ALink`], [`NotKept::SomebodyElses`] and
/// [`NotKept::WritableByOthers`] for a file this machine will not read as its
/// own record; [`NotKept::NotRead`] for everything the machine itself refused.
#[cfg(unix)]
pub(crate) fn text_of(path: &Path) -> Result<String, NotKept> {
    use std::io::Read;
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt};

    let mut file = match std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(o_nofollow())
        .open(path)
    {
        Ok(file) => file,
        // `ELOOP` is what `O_NOFOLLOW` answers a link with; the named
        // `ErrorKind` for it is not yet stable, so the number is compared.
        Err(why) if why.raw_os_error() == Some(rustix::io::Errno::LOOP.raw_os_error()) => {
            return Err(NotKept::a_link(path));
        }
        Err(why) => return Err(NotKept::reading(path, &why)),
    };
    let seen = file
        .metadata()
        .map_err(|why| NotKept::reading(path, &why))?;
    believed(path, seen.uid(), seen.mode(), us())?;
    let mut text = String::new();
    file.read_to_string(&mut text)
        .map_err(|why| NotKept::reading(path, &why))?;
    Ok(text)
}

/// The same, on a host that cannot be asked any of it.
///
/// # Errors
///
/// [`NotKept::NotRead`], always. See this module's documentation: a record
/// nobody can say who wrote is not read here.
#[cfg(not(unix))]
pub(crate) fn text_of(path: &Path) -> Result<String, NotKept> {
    Err(NotKept::reading(
        path,
        &std::io::Error::from(std::io::ErrorKind::Unsupported),
    ))
}

/// The mode bits that let the group or the world write.
#[cfg(unix)]
const OTHERS_MAY_WRITE: u32 = 0o022;

/// Whether a file with this owner and mode is one to believe, as a rule of its
/// own so every branch of it is testable without root.
#[cfg(unix)]
fn believed(at: &Path, owner: u32, mode: u32, us: u32) -> Result<(), NotKept> {
    if owner != 0 && owner != us {
        return Err(NotKept::SomebodyElses {
            path: at.display().to_string(),
            owner,
        });
    }
    if mode & OTHERS_MAY_WRITE != 0 {
        return Err(NotKept::WritableByOthers {
            path: at.display().to_string(),
            mode: mode & 0o777,
        });
    }
    Ok(())
}

/// The user this process runs as, asked of the kernel rather than of an
/// environment.
#[cfg(unix)]
fn us() -> u32 {
    rustix::process::geteuid().as_raw()
}

/// The flag that refuses to open a symbolic link, as `OpenOptions` takes it.
#[cfg(unix)]
#[expect(
    clippy::cast_possible_wrap,
    reason = "O_NOFOLLOW is a flag bit pattern; the kernel reads it as bits either way"
)]
fn o_nofollow() -> i32 {
    rustix::fs::OFlags::NOFOLLOW.bits() as i32
}

#[cfg(all(test, unix))]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_folder_of_our_own, a_line, an_afternoon, nowhere};
    use std::os::unix::fs::PermissionsExt;

    /// A record on a real disk, written the way this crate writes one.
    fn a_record_in(folder: &Path) -> std::path::PathBuf {
        let at = folder.join("record.jsonl");
        let mut text = "{\"format\":1}\n".to_owned();
        for entry in an_afternoon() {
            text.push_str(&a_line(&entry));
        }
        std::fs::write(&at, text).unwrap();
        std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o600)).unwrap();
        at
    }

    /// A record this machine wrote is read back whole, which is the ordinary
    /// case and the one every refusal below is measured against.
    #[test]
    fn a_record_of_our_own_is_read() {
        let folder = a_folder_of_our_own("believed");
        let text = text_of(&a_record_in(&folder)).unwrap();
        assert!(text.starts_with("{\"format\":1}\n"));
        assert_eq!(text.lines().count(), an_afternoon().len() + 1);
    }

    /// **A record that is not there is still not an empty one.** Believing the
    /// file does not change the one answer this crate must never give.
    #[test]
    fn a_record_that_is_not_there_is_told_apart_from_every_refusal() {
        let folder = a_folder_of_our_own("believed-missing");
        let refused = text_of(&folder.join("record.jsonl")).unwrap_err();
        assert!(matches!(refused, NotKept::NotThere { .. }), "{refused:?}");
    }

    /// **A symbolic link is refused as one, never followed.** A link is a name
    /// somebody can point at a record of their own writing, and what a record
    /// says is what everybody is told happened here.
    #[test]
    fn a_record_behind_a_symbolic_link_is_refused() {
        let folder = a_folder_of_our_own("believed-link");
        let real = a_record_in(&folder);
        let link = folder.join("linked.jsonl");
        std::os::unix::fs::symlink(&real, &link).unwrap();

        assert!(matches!(text_of(&link), Err(NotKept::ALink { .. })));
    }

    /// **A record the group or the world can write is refused**, and not one
    /// line of it is read.
    #[test]
    fn a_record_somebody_else_could_write_is_refused() {
        let folder = a_folder_of_our_own("believed-writable");
        let at = a_record_in(&folder);
        std::fs::set_permissions(&at, std::fs::Permissions::from_mode(0o666)).unwrap();

        assert!(matches!(
            text_of(&at),
            Err(NotKept::WritableByOthers { mode: 0o666, .. })
        ));
    }

    /// **The ownership rule, every branch** — a test cannot chown a file to
    /// somebody else without root, so the rule is a function and this walks it:
    /// root's record is believed, ours is believed, anybody else's is not, and
    /// a believable owner does not excuse a writable mode.
    #[test]
    fn only_roots_record_or_our_own_is_believed() {
        assert!(believed(nowhere(), 0, 0o600, 1000).is_ok());
        assert!(believed(nowhere(), 1000, 0o644, 1000).is_ok());
        assert!(matches!(
            believed(nowhere(), 1001, 0o600, 1000),
            Err(NotKept::SomebodyElses { owner: 1001, .. })
        ));
        assert!(matches!(
            believed(nowhere(), 0, 0o620, 1000),
            Err(NotKept::WritableByOthers { .. })
        ));
        assert!(matches!(
            believed(nowhere(), 1000, 0o602, 1000),
            Err(NotKept::WritableByOthers { .. })
        ));
    }
}
