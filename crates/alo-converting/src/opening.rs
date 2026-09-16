//! Opening the original so it cannot be written, and creating the copy so
//! nothing is replaced.
//!
//! ADR 0039 §3: **the copy is never the original.** The original is opened
//! read-only, so nothing downstream of this file holds a handle that could
//! write it. The copy is created in the folder the person chose with `O_EXCL`,
//! so a name already there is a refusal and never an overwrite or a quiet
//! rename. A copy half-written when converting fails is removed by the verb that
//! created it — and only if the name still leads to the file this created, so
//! a file somebody put there in the meantime is never the one removed.
//!
//! # On Linux, no link is followed
//!
//! The original and the folder are opened with `openat2` and
//! `RESOLVE_NO_SYMLINKS`, the way `alo-files` opens what a verb reads: the whole
//! path is resolved in one call that refuses a link at every component, so
//! nothing can be swapped in between the grant's answer and the open. The copy
//! is created relative to the folder's handle rather than by its name. This is
//! the only file here that names `rustix` for it.
//!
//! Elsewhere it is `std`, which resolves each name once more; a machine that
//! converts is a Linux machine, and this half exists so the crate builds.
//!
//! # A file with other names is not read
//!
//! A hard link is a second real name nothing about a path reveals, and one
//! inside a granted folder is a way to read what the grant never covered. Asked
//! of the open handle, as `alo-files` asks it: more than one name and the file
//! is not converted.

use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

/// Why a file could not be opened or created.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotOpened {
    /// Something by that name is already there.
    Taken,
    /// The file has more than one name on this machine.
    MoreThanOneName,
    /// It is not a regular file, or not a folder, as asked.
    NotThatKind,
    /// The machine said no, in its own words.
    TheMachine(io::ErrorKind),
}

/// A folder the copy is created in, held open.
#[derive(Debug)]
pub struct Folder {
    /// The open folder.
    #[cfg(target_os = "linux")]
    handle: std::os::fd::OwnedFd,
    /// Where it is.
    path: PathBuf,
}

/// A copy this created, which is only ever removed by this.
#[derive(Debug)]
pub struct Created {
    /// The copy, open for writing.
    file: File,
    /// Its name in the folder.
    name: OsString,
}

impl Created {
    /// The copy, open for writing.
    #[must_use]
    pub fn file(&self) -> &File {
        &self.file
    }

    /// Its name in the folder.
    #[must_use]
    pub fn name(&self) -> &OsStr {
        &self.name
    }
}

/// Open the original, read-only.
///
/// # Errors
/// [`NotOpened`].
pub fn original(path: &Path) -> Result<File, NotOpened> {
    let opened = platform::read_only(path)?;
    let what = opened
        .metadata()
        .map_err(|why| NotOpened::TheMachine(why.kind()))?;
    if !what.is_file() {
        return Err(NotOpened::NotThatKind);
    }
    if platform::names(&what) > 1 {
        return Err(NotOpened::MoreThanOneName);
    }
    Ok(opened)
}

impl Folder {
    /// Open the folder a copy is created in.
    ///
    /// # Errors
    /// [`NotOpened`].
    pub fn open(path: &Path) -> Result<Self, NotOpened> {
        platform::folder(path)
    }

    /// Where it is.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Create a new, empty file in it, refusing a name already there.
    ///
    /// # Errors
    /// [`NotOpened::Taken`] when anything by that name is there, a link
    /// included; otherwise what the machine said.
    pub fn create(&self, name: &OsStr) -> Result<Created, NotOpened> {
        let file = platform::create(self, name)?;
        Ok(Created {
            file,
            name: name.to_owned(),
        })
    }

    /// Remove a copy this created, if the name still leads to it.
    ///
    /// # Errors
    /// What the machine said. A name that now leads somewhere else is left
    /// alone and is not an error: it is not the copy.
    pub fn remove(&self, created: Created) -> io::Result<()> {
        platform::remove(self, created)
    }
}

#[cfg(target_os = "linux")]
mod platform {
    //! The Linux answers, and the only place `rustix` is named for opening.

    use std::ffi::OsStr;
    use std::fs::{File, Metadata};
    use std::io;
    use std::os::unix::fs::MetadataExt as _;
    use std::path::Path;

    use rustix::fs::{
        AtFlags, CWD, Mode, OFlags, ResolveFlags, fstat, openat, openat2, statat, unlinkat,
    };

    use super::{Created, Folder, NotOpened};

    /// The machine's refusal, as a [`NotOpened`].
    fn refused(why: rustix::io::Errno) -> NotOpened {
        if why == rustix::io::Errno::EXIST {
            NotOpened::Taken
        } else if why == rustix::io::Errno::NOTDIR {
            NotOpened::NotThatKind
        } else {
            NotOpened::TheMachine(io::Error::from(why).kind())
        }
    }

    /// Open a path read-only, following no link anywhere on it.
    pub(super) fn read_only(path: &Path) -> Result<File, NotOpened> {
        openat2(
            CWD,
            path,
            OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOCTTY,
            Mode::empty(),
            ResolveFlags::NO_SYMLINKS,
        )
        .map(File::from)
        .map_err(refused)
    }

    /// How many names a file has.
    pub(super) fn names(what: &Metadata) -> u64 {
        what.nlink()
    }

    /// Open a folder, following no link anywhere on it.
    pub(super) fn folder(path: &Path) -> Result<Folder, NotOpened> {
        let handle = openat2(
            CWD,
            path,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
            ResolveFlags::NO_SYMLINKS,
        )
        .map_err(refused)?;
        Ok(Folder {
            handle,
            path: path.to_owned(),
        })
    }

    /// Create a file in a folder by its handle, refusing anything there.
    pub(super) fn create(folder: &Folder, name: &OsStr) -> Result<File, NotOpened> {
        openat(
            &folder.handle,
            name,
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::from_raw_mode(0o644),
        )
        .map(File::from)
        .map_err(refused)
    }

    /// Remove a created file, if its name still leads to it.
    pub(super) fn remove(folder: &Folder, created: Created) -> io::Result<()> {
        let ours = fstat(&created.file)?;
        let there = match statat(&folder.handle, created.name(), AtFlags::SYMLINK_NOFOLLOW) {
            Ok(there) => there,
            Err(rustix::io::Errno::NOENT) => return Ok(()),
            Err(why) => return Err(why.into()),
        };
        if there.st_dev != ours.st_dev || there.st_ino != ours.st_ino {
            return Ok(());
        }
        unlinkat(&folder.handle, created.name(), AtFlags::empty())?;
        Ok(())
    }
}

#[cfg(not(target_os = "linux"))]
mod platform {
    //! The portable answers, which resolve each name once more.

    use std::ffi::OsStr;
    use std::fs::{File, Metadata};
    use std::io;
    use std::path::Path;

    use super::{Created, Folder, NotOpened};

    /// Open a path read-only.
    pub(super) fn read_only(path: &Path) -> Result<File, NotOpened> {
        File::open(path).map_err(|why| NotOpened::TheMachine(why.kind()))
    }

    /// How many names a file has, which `std` cannot say here.
    pub(super) fn names(_: &Metadata) -> u64 {
        1
    }

    /// A folder, by its name.
    pub(super) fn folder(path: &Path) -> Result<Folder, NotOpened> {
        if path.is_dir() {
            Ok(Folder {
                path: path.to_owned(),
            })
        } else {
            Err(NotOpened::NotThatKind)
        }
    }

    /// Create a file, refusing anything there.
    pub(super) fn create(folder: &Folder, name: &OsStr) -> Result<File, NotOpened> {
        File::options()
            .write(true)
            .create_new(true)
            .open(folder.path.join(name))
            .map_err(|why| match why.kind() {
                io::ErrorKind::AlreadyExists => NotOpened::Taken,
                other => NotOpened::TheMachine(other),
            })
    }

    /// Remove a created file by its name.
    pub(super) fn remove(folder: &Folder, created: Created) -> io::Result<()> {
        let at = folder.path.join(created.name());
        drop(created);
        std::fs::remove_file(at)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use std::io::Write;

    /// A folder of its own for one test.
    fn a_folder(named: &str) -> PathBuf {
        let at = std::env::temp_dir().join(format!(
            "alo-converting-opening-{named}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&at);
        std::fs::create_dir_all(&at).unwrap();
        at
    }

    /// **A name already there is refused, and what is there is untouched.**
    #[test]
    fn a_name_already_there_is_never_replaced() {
        let at = a_folder("taken");
        std::fs::write(at.join("report.pdf"), b"somebody's own").unwrap();
        let folder = Folder::open(&at).unwrap();
        assert_eq!(
            folder.create(OsStr::new("report.pdf")).err(),
            Some(NotOpened::Taken)
        );
        assert_eq!(
            std::fs::read(at.join("report.pdf")).unwrap(),
            b"somebody's own"
        );
        std::fs::remove_dir_all(&at).unwrap();
    }

    /// **A copy is removed only while its name still leads to it.**
    #[test]
    fn a_copy_is_removed_only_while_its_name_leads_to_it() {
        let at = a_folder("removing");
        let folder = Folder::open(&at).unwrap();
        let created = folder.create(OsStr::new("copy.pdf")).unwrap();
        created.file().write_all(b"half").unwrap();
        folder.remove(created).unwrap();
        assert!(!at.join("copy.pdf").exists());

        #[cfg(target_os = "linux")]
        {
            let created = folder.create(OsStr::new("copy.pdf")).unwrap();
            std::fs::remove_file(at.join("copy.pdf")).unwrap();
            std::fs::write(at.join("copy.pdf"), b"somebody else's").unwrap();
            folder.remove(created).unwrap();
            assert_eq!(
                std::fs::read(at.join("copy.pdf")).unwrap(),
                b"somebody else's"
            );
        }
        std::fs::remove_dir_all(&at).unwrap();
    }

    /// **The original is opened so it cannot be written**, and a folder is not
    /// an original.
    #[test]
    fn the_original_cannot_be_written_through_its_handle() {
        let at = a_folder("original");
        std::fs::write(at.join("sent.docx"), b"as sent").unwrap();
        let mut opened = original(&at.join("sent.docx")).unwrap();
        assert!(opened.write_all(b"changed").is_err());
        assert_eq!(std::fs::read(at.join("sent.docx")).unwrap(), b"as sent");
        assert!(original(&at).is_err());
        std::fs::remove_dir_all(&at).unwrap();
    }

    /// **A link on the way, or a second name, is not followed or read.**
    #[cfg(target_os = "linux")]
    #[test]
    fn a_link_or_a_second_name_is_not_read() {
        let at = a_folder("links");
        std::fs::write(at.join("real.docx"), b"real").unwrap();
        std::os::unix::fs::symlink(at.join("real.docx"), at.join("link.docx")).unwrap();
        assert!(matches!(
            original(&at.join("link.docx")),
            Err(NotOpened::TheMachine(_))
        ));
        std::os::unix::fs::symlink(&at, at.join("through")).unwrap();
        assert!(Folder::open(&at.join("through")).is_err());
        std::fs::hard_link(at.join("real.docx"), at.join("second.docx")).unwrap();
        assert_eq!(
            original(&at.join("real.docx")).err(),
            Some(NotOpened::MoreThanOneName)
        );
        std::fs::remove_dir_all(&at).unwrap();
    }
}
