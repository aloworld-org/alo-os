//! The kernel, as the three things this crate asks of it.
//!
//! Everything here is read out of files under `/proc`, and a reading is the
//! same set of reads whether the files are the kernel's or ones a test wrote
//! into a directory of its own. So the reads go through this trait: [`Disk`]
//! is the real thing, and a test's kernel is whatever the test says — a
//! directory it built, a file it made unreadable, a process it removes between
//! the listing and the reading.
//!
//! Three operations, and no fourth: list a directory, read a file, read a
//! link. Nothing here writes, and nothing here can.

use std::io;
use std::path::{Path, PathBuf};

/// What this crate asks of a kernel: to list, to read, and to say where a link
/// points. Nothing else.
pub trait Kernel {
    /// The names in a directory, in no particular order.
    ///
    /// # Errors
    ///
    /// Whatever the operating system said.
    fn list(&self, directory: &Path) -> io::Result<Vec<String>>;

    /// A file's text.
    ///
    /// # Errors
    ///
    /// Whatever the operating system said. A file of a process that ended is
    /// [`io::ErrorKind::NotFound`] or `ESRCH`; a file the caller may not open
    /// is [`io::ErrorKind::PermissionDenied`].
    fn read(&self, file: &Path) -> io::Result<String>;

    /// Where a symbolic link points.
    ///
    /// # Errors
    ///
    /// Whatever the operating system said.
    fn link(&self, link: &Path) -> io::Result<PathBuf>;
}

/// The kernel that is actually running: the filesystem.
#[derive(Debug, Clone, Copy, Default)]
pub struct Disk;

impl Kernel for Disk {
    fn list(&self, directory: &Path) -> io::Result<Vec<String>> {
        let mut names = Vec::new();
        for entry in std::fs::read_dir(directory)? {
            names.push(entry?.file_name().to_string_lossy().into_owned());
        }
        Ok(names)
    }

    fn read(&self, file: &Path) -> io::Result<String> {
        std::fs::read_to_string(file)
    }

    fn link(&self, link: &Path) -> io::Result<PathBuf> {
        std::fs::read_link(link)
    }
}

/// Whether a read failed because the process is no longer there.
///
/// The kernel says `ENOENT` for a directory that has gone and `ESRCH` — *no
/// such process* — for a file inside one whose process ended while it was
/// open. Both mean the same thing here, and the standard library has no kind
/// for the second, so it is matched by number.
#[must_use]
pub(crate) fn is_gone(why: &io::Error) -> bool {
    /// `ESRCH`, on Linux.
    const NO_SUCH_PROCESS: i32 = 3;
    why.kind() == io::ErrorKind::NotFound || why.raw_os_error() == Some(NO_SUCH_PROCESS)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// The real kernel lists, reads and follows a link in a directory this
    /// test made, and the three are the three the trait names.
    #[test]
    fn the_disk_lists_reads_and_follows() {
        let ours = std::env::temp_dir().join(format!("alo-measuring-disk-{}", std::process::id()));
        drop(std::fs::remove_dir_all(&ours));
        std::fs::create_dir_all(&ours).unwrap();
        std::fs::write(ours.join("a"), "one\n").unwrap();
        std::fs::write(ours.join("b"), "two\n").unwrap();

        let mut listed = Disk.list(&ours).unwrap();
        listed.sort();
        assert_eq!(listed, ["a", "b"]);
        assert_eq!(Disk.read(&ours.join("a")).unwrap(), "one\n");
        let missing = Disk.read(&ours.join("c")).unwrap_err();
        assert!(is_gone(&missing), "{missing}");
        assert!(Disk.link(&ours.join("a")).is_err(), "a file is not a link");

        drop(std::fs::remove_dir_all(&ours));
    }

    /// `ESRCH` is gone, `ENOENT` is gone, and a refusal is not.
    #[test]
    fn gone_is_no_such_file_or_no_such_process_and_nothing_else() {
        assert!(is_gone(&io::Error::from_raw_os_error(3)));
        assert!(is_gone(&io::Error::from(io::ErrorKind::NotFound)));
        assert!(!is_gone(&io::Error::from(io::ErrorKind::PermissionDenied)));
    }
}
