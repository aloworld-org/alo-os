//! The answers file on the disk: who may have written it, before a byte of it
//! is believed or added to.
//!
//! The rules `alo-remembering` holds the grants and pairings files to
//! (`alo_remembering`'s `believing`), for the same reason: a record whoever
//! could rewrite is a record that says what they like about which application
//! was refused. That crate keeps them private and is not this plan's to change,
//! so the three questions are asked again here, of the open file rather than
//! of the path, so the file checked and the file read cannot be two files:
//!
//! - the path is **not a symbolic link** (`O_NOFOLLOW`);
//! - it is **a regular file**, not a pipe, a device or a folder;
//! - it belongs to **the login reading it**, and **nobody else can write it**.
//!
//! The ownership rule is narrower than it was, and
//! [ADR 0052](../../../docs/decisions/0052-what-a-persons-applications-asked-for-is-the-persons-record.md)
//! is why: the answers file used to be one machine-wide file beside the agent's
//! record, where root's ownership was the ordinary case. It is now **this
//! login's own**, in their own state directory — and a root-owned file there is
//! not the ordinary case, it is evidence that something else wrote a person's
//! record.
//!
//! A file made here is made `0600`. **The folder is not made here**:
//! `crate::where_the_answers_are` makes it, once, inside a directory that
//! already exists.

use std::fs::{File, OpenOptions};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::Path;

use crate::not_recorded::NotRecorded;

/// The mode bits that let the group or the world write.
const OTHERS_MAY_WRITE: u32 = 0o022;

/// The mode the answers file is made with: the owner and nobody else.
pub(crate) const OURS_ALONE: u32 = 0o600;

/// The file at `at`, opened for appending — made `0600` when it is not there —
/// and believed.
///
/// # Errors
/// [`NotRecorded::ALink`], [`NotRecorded::NotAnAnswersFile`] for what is not a
/// regular file, [`NotRecorded::SomebodyElses`],
/// [`NotRecorded::WritableByOthers`], and [`NotRecorded::NotWritten`] for
/// everything else the machine said — a missing folder among them.
pub(crate) fn opened_to_add_to(at: &Path) -> Result<File, NotRecorded> {
    let mut options = OpenOptions::new();
    options
        .read(true)
        .append(true)
        .create(true)
        .mode(OURS_ALONE)
        .custom_flags(refusing_links_and_waits());
    believed_open(at, &options, Missing::IsAFailure, |why| {
        NotRecorded::NotWritten {
            at: at.to_owned(),
            why,
        }
    })
}

/// A new file at `at`, made `0600` for reading and appending, to be renamed
/// over the answers file once it holds everything it replaces it with.
///
/// Made **new**: whatever is at `at` — a shortening the machine interrupted, or
/// a link somebody put there — is refused rather than written through, and the
/// caller removes an interrupted one first.
///
/// # Errors
/// [`NotRecorded::ALink`], and [`NotRecorded::NotWritten`] for everything else
/// the machine said, something already at `at` among them.
pub(crate) fn made_to_replace(at: &Path) -> Result<File, NotRecorded> {
    let mut options = OpenOptions::new();
    options
        .read(true)
        .append(true)
        .create_new(true)
        .mode(OURS_ALONE)
        .custom_flags(refusing_links_and_waits());
    believed_open(at, &options, Missing::IsAFailure, |why| {
        NotRecorded::NotWritten {
            at: at.to_owned(),
            why,
        }
    })
}

/// The file at `at`, opened to be read, and believed.
///
/// # Errors
/// [`NotRecorded::NotThere`] when there is no file; otherwise as
/// [`opened_to_add_to`], with [`NotRecorded::NotRead`] for what the machine
/// said.
pub(crate) fn opened_to_read(at: &Path) -> Result<File, NotRecorded> {
    let mut options = OpenOptions::new();
    options.read(true).custom_flags(refusing_links_and_waits());
    believed_open(at, &options, Missing::IsNotThere, |why| {
        NotRecorded::NotRead {
            at: at.to_owned(),
            why,
        }
    })
}

/// What a path that is not there means to whoever is opening it.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Missing {
    /// Nothing has been answered there yet: a reader's answer.
    IsNotThere,
    /// A folder that is not there, since the file itself would have been made:
    /// a writer's failure.
    IsAFailure,
}

/// `at`, opened with `options`, refused unless it is a regular file only this
/// login could have written.
fn believed_open(
    at: &Path,
    options: &OpenOptions,
    missing: Missing,
    otherwise: impl Fn(String) -> NotRecorded,
) -> Result<File, NotRecorded> {
    let file = match options.open(at) {
        Ok(file) => file,
        Err(why)
            if why.kind() == std::io::ErrorKind::NotFound && missing == Missing::IsNotThere =>
        {
            return Err(NotRecorded::NotThere { at: at.to_owned() });
        }
        // `ELOOP` is what `O_NOFOLLOW` answers a link with; the named
        // `ErrorKind` for it is not yet stable, so the number is compared.
        Err(why) if why.raw_os_error() == Some(rustix::io::Errno::LOOP.raw_os_error()) => {
            return Err(NotRecorded::ALink { at: at.to_owned() });
        }
        Err(why) => return Err(otherwise(why.to_string())),
    };
    let seen = file.metadata().map_err(|why| otherwise(why.to_string()))?;
    if !seen.is_file() {
        return Err(NotRecorded::NotAnAnswersFile { at: at.to_owned() });
    }
    believed(at, seen.uid(), seen.mode(), us())?;
    Ok(file)
}

/// Whether a file with this owner and mode is one to believe, as a rule of its
/// own so every branch of it is testable without root.
pub(crate) fn believed(at: &Path, owner: u32, mode: u32, us: u32) -> Result<(), NotRecorded> {
    if owner != us {
        return Err(NotRecorded::SomebodyElses {
            at: at.to_owned(),
            owner,
        });
    }
    if mode & OTHERS_MAY_WRITE != 0 {
        return Err(NotRecorded::WritableByOthers {
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

/// The flags that refuse to open a symbolic link (`O_NOFOLLOW`) and refuse to
/// wait (`O_NONBLOCK`), as `OpenOptions` takes them.
///
/// Not waiting matters for what is not a regular file: a named pipe put where
/// the file should be would otherwise hold the backend at `open` until
/// somebody wrote into it, and it is refused a moment later as not a file. A
/// regular file reads and writes the same either way.
#[expect(
    clippy::cast_possible_wrap,
    reason = "open flags are a bit pattern; the kernel reads them as bits either way"
)]
fn refusing_links_and_waits() -> i32 {
    (rustix::fs::OFlags::NOFOLLOW | rustix::fs::OFlags::NONBLOCK).bits() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **The ownership rule, every branch** — a test cannot chown a file to
    /// somebody else without root, so the rule is a function and this walks it.
    ///
    /// **Root's own file is refused too**, which is the change ADR 0052 made: in
    /// a person's own state directory, a file owned by root is not this login's
    /// record.
    #[test]
    fn only_this_logins_own_file_is_believed() {
        let at = Path::new("/home/ada/.local/state/alo/portal-answers.jsonl");
        assert!(matches!(
            believed(at, 0, OURS_ALONE, 1000),
            Err(NotRecorded::SomebodyElses { owner: 0, .. })
        ));
        assert!(believed(at, 1000, OURS_ALONE, 1000).is_ok());
        assert!(believed(at, 0, OURS_ALONE, 0).is_ok());
        assert!(matches!(
            believed(at, 1001, OURS_ALONE, 1000),
            Err(NotRecorded::SomebodyElses { owner: 1001, .. })
        ));
        assert!(matches!(
            believed(at, 1000, 0o620, 1000),
            Err(NotRecorded::WritableByOthers { .. })
        ));
        assert!(matches!(
            believed(at, 1000, 0o602, 1000),
            Err(NotRecorded::WritableByOthers { .. })
        ));
    }
}
