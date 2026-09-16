//! Writing the picture into the folder the person chose.
//!
//! One rule, and everything here follows from it: **nothing that was already in
//! that folder is touched.** A picture of a screen is written under a name made
//! from the moment it was taken, and if a file of that name is there the next
//! name is tried — so a screenshot tool cannot overwrite the invoice somebody
//! keeps in the folder they also keep pictures in.
//!
//! # Created, never opened
//!
//! The file is opened with `create_new`, which is the kernel's `O_CREAT |
//! O_EXCL`: it makes the file or it fails, and it fails on a name that is
//! already anything at all — including a symbolic link pointing at somebody
//! else's file. A check followed by a write would be a race with whatever is
//! also writing in that folder; asking the kernel to create it is one operation
//! and there is no gap to win.
//!
//! # And it is the person's own picture
//!
//! On a machine with people on it, the file is made readable and writable by
//! its owner and by nobody else. A picture of a screen is exactly the kind of
//! file that holds somebody's bank balance, and a folder that happens to be
//! readable by everybody should not decide that.
//!
//! # Nothing here decides where
//!
//! The folder arrives as a [`Folder`] and the name as one
//! [`crate::naming::name_for`] made. This file joins them, tries, and answers.

use std::io::{ErrorKind, Write};

use std::path::PathBuf;

use crate::folder::Folder;
use crate::naming;
use crate::on_this_day::OnThisDay;
use crate::picture::Picture;
use crate::refusing::NotTaken;

/// Write this picture into that folder, under the moment it was taken.
///
/// Answers where it ended up.
///
/// # Errors
/// [`NotTaken::NoRoomForAName`] when every name that moment can make is already
/// a file there, and [`NotTaken::NotWritten`] when the folder would not take
/// it — it is gone, it is not the person's to write in, or the disk is full.
pub(crate) fn write(picture: &Picture, into: &Folder, at: OnThisDay) -> Result<PathBuf, NotTaken> {
    let mut already_there = 0;
    loop {
        let Some(named) = naming::name_for(at, already_there) else {
            return Err(NotTaken::NoRoomForAName);
        };
        let where_it_would_go = into.holding(&named);
        match make(&where_it_would_go) {
            Ok(mut file) => {
                return file
                    .write_all(picture.bytes())
                    .and_then(|()| file.sync_all())
                    .map(|()| where_it_would_go)
                    .map_err(|why| NotTaken::NotWritten {
                        said: format!("the picture could not be written: {why}"),
                    });
            }
            Err(why) if why.kind() == ErrorKind::AlreadyExists => {
                already_there = already_there.saturating_add(1);
            }
            Err(why) => {
                return Err(NotTaken::NotWritten {
                    said: format!("{} could not be made: {why}", where_it_would_go.display()),
                });
            }
        }
    }
}

/// Make this file, and fail if anything is there already.
#[cfg(unix)]
fn make(at: &std::path::Path) -> std::io::Result<std::fs::File> {
    use std::os::unix::fs::OpenOptionsExt;

    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        // The person's own, and nobody else's: a picture of a screen holds
        // whatever was on it.
        .mode(0o600)
        .open(at)
}

/// Make this file, and fail if anything is there already.
///
/// Whoever may write in the folder may read what is written there; a Windows
/// host is where this workspace is developed and not where alo OS runs, and the
/// permissions a released machine sets are the Unix ones above.
#[cfg(not(unix))]
fn make(at: &std::path::Path) -> std::io::Result<std::fs::File> {
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(at)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_moment, a_picture, an_empty_folder};

    /// **A picture is written under the moment it was taken**, in the folder
    /// somebody chose, with its bytes exactly as they arrived.
    #[test]
    fn a_picture_is_written_under_the_moment_it_was_taken() {
        let folder = an_empty_folder("written");
        let at = write(&a_picture(), &folder, a_moment()).unwrap();

        assert_eq!(at.parent(), Some(folder.at()));
        assert_eq!(
            at.file_name().and_then(|name| name.to_str()),
            Some("2026-09-16-120000.png")
        );
        assert_eq!(std::fs::read(&at).unwrap(), a_picture().bytes());
    }

    /// **A second picture in the same second does not overwrite the first.**
    /// Both are somebody's, and a tool that lost one of them because a clock
    /// had not ticked would be losing it silently.
    #[test]
    fn a_second_picture_in_one_second_does_not_overwrite_the_first() {
        let folder = an_empty_folder("twice");
        let first = write(&a_picture(), &folder, a_moment()).unwrap();
        let second = write(&a_picture(), &folder, a_moment()).unwrap();

        assert_ne!(first, second);
        assert_eq!(
            second.file_name().and_then(|name| name.to_str()),
            Some("2026-09-16-120000-2.png")
        );
        assert!(first.exists() && second.exists());
    }

    /// **A file already in the folder is never written over**, whatever it is.
    /// Somebody's invoice under a name that happens to look like a picture's is
    /// still their invoice.
    #[test]
    fn nothing_already_in_the_folder_is_written_over() {
        let folder = an_empty_folder("theirs");
        let theirs = folder.holding("2026-09-16-120000.png");
        std::fs::write(&theirs, b"this is somebody's own file").unwrap();

        let at = write(&a_picture(), &folder, a_moment()).unwrap();
        assert_ne!(at, theirs);
        assert_eq!(
            std::fs::read(&theirs).unwrap(),
            b"this is somebody's own file"
        );
    }

    /// **A folder that is not there is a refusal with what the disk said**,
    /// rather than a folder made on somebody's behalf: a picture appearing in a
    /// folder alo OS invented is a picture nobody can find.
    #[test]
    fn a_folder_that_is_not_there_is_refused_and_not_made() {
        let folder = an_empty_folder("gone");
        let missing = Folder::chosen(&folder.at().join("not-there")).unwrap();

        let refused = write(&a_picture(), &missing, a_moment()).unwrap_err();
        assert!(
            matches!(refused, NotTaken::NotWritten { .. }),
            "{refused:?}"
        );
        assert!(refused.diagnosis().is_some_and(|said| !said.is_empty()));
        assert!(!missing.at().exists(), "a folder was made on the way past");
    }

    /// **When every name that moment can make is taken, it says so** rather
    /// than trying for ever while the machine looks busy.
    #[test]
    fn a_folder_with_every_name_already_in_it_says_so() {
        let folder = an_empty_folder("full");
        for already_there in 0..naming::HOW_MANY_ONE_MOMENT_MAKES {
            let named = naming::name_for(a_moment(), already_there).unwrap();
            std::fs::write(folder.holding(&named), b"taken").unwrap();
        }

        let refused = write(&a_picture(), &folder, a_moment()).unwrap_err();
        assert_eq!(refused, NotTaken::NoRoomForAName);
    }

    /// **The picture is the person's own**, readable and writable by them and
    /// by nobody else — because a picture of a screen holds whatever was on it.
    #[cfg(unix)]
    #[test]
    fn the_picture_belongs_to_the_person_and_to_nobody_else() {
        use std::os::unix::fs::PermissionsExt;

        let folder = an_empty_folder("theirs-alone");
        let at = write(&a_picture(), &folder, a_moment()).unwrap();
        let mode = std::fs::metadata(&at).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600, "{mode:o}");
    }
}
