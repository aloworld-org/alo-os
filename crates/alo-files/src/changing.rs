//! The two verbs that move a file: rename it where it is, and move it into
//! another folder.
//!
//! Both are the same act to a filesystem — a file gets a different full name —
//! and they are two verbs because they are two things to a person. What they
//! share is here, once, so that the rules below cannot be true of one of them
//! and not the other.
//!
//! # Nothing is replaced that was not named
//!
//! A person approved *move march.pdf into Archive*. They did not approve
//! *and overwrite the march.pdf that is already there*, and on most systems
//! renaming over an existing file is exactly what that sentence would silently
//! do. So a destination that already holds anything — a file, a folder, or a
//! link to somewhere else entirely — is refused, and the person is told the
//! name is taken.
//!
//! There is a gap between asking whether something is there and moving
//! something onto it, and `docs/quirks.md` records both what closes it
//! (`renameat2` with `RENAME_NOREPLACE` on Linux, from a directory handle) and
//! why the portable half cannot. What it does not do is pretend: the check is
//! made, the residual race is written down, and the code that closes it is a
//! queue item rather than a comment.
//!
//! # Half a move is not a smaller move
//!
//! Every refusal here happens before anything is touched, so a call that is
//! refused has moved nothing. There is no state in which the file is in two
//! places, or in neither.

use std::fs;
use std::path::Path;

use crate::answer::Answer;
use crate::failed::Failed;
use crate::real::Real;

/// Give a file a different name, where it already is.
///
/// # Errors
/// [`Failed`] if it is not a file, if something is already called that, or if
/// the machine would not move it.
pub(crate) fn rename(file: &Real, to: &Path) -> Result<Answer, Failed> {
    moving(file, to, "renamed")?;
    Ok(Answer::Renamed(to.to_owned()))
}

/// Move a file into a folder.
///
/// # Errors
/// [`Failed`] if the file is not a file, the folder is not a folder, the file
/// is already in it, something in it is already called that, or the machine
/// would not move it.
pub(crate) fn move_into(file: &Real, into: &Real, to: &Path) -> Result<Answer, Failed> {
    let folder = into.as_path();
    let what =
        fs::symlink_metadata(folder).map_err(|why| Failed::machine(folder, "moved", &why))?;
    if !what.is_dir() {
        return Err(Failed::NotAFolder {
            path: folder.display().to_string(),
        });
    }
    if file.as_path().parent() == Some(folder) {
        return Err(Failed::AlreadyIn {
            path: file.as_path().display().to_string(),
        });
    }
    moving(file, to, "moved")?;
    Ok(Answer::Moved(to.to_owned()))
}

/// What both of them do: check that there is a file to move and nothing where
/// it is going, and then move it.
///
/// `doing` is the word the failure uses, which is the only thing the two verbs
/// differ by once the machine is involved.
fn moving(file: &Real, to: &Path, doing: &str) -> Result<(), Failed> {
    let from = file.as_path();
    let what = fs::symlink_metadata(from).map_err(|why| Failed::machine(from, doing, &why))?;
    if !what.is_file() {
        return Err(Failed::NotAFile {
            path: from.display().to_string(),
        });
    }
    // Anything at all, including a link: `symlink_metadata` answers about the
    // name rather than about what it leads to, which is the question being
    // asked here. A link is something, and moving onto it would replace it.
    if fs::symlink_metadata(to).is_ok() {
        return Err(Failed::AlreadyThere {
            path: to.display().to_string(),
        });
    }
    fs::rename(from, to).map_err(|why| Failed::machine(from, doing, &why))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_folder_of_our_own, really, said};

    /// The ordinary day: a file gets a different name, where it was.
    #[test]
    fn a_file_can_be_given_a_different_name_where_it_is() {
        let folder = a_folder_of_our_own("rename");
        let file = folder.join("march.pdf");
        fs::write(&file, b"an invoice").unwrap();
        let to = folder.join("march-2026.pdf");

        let answer = rename(&really(&file), &to).unwrap();
        assert_eq!(answer.now_at(), Some(to.as_path()));
        assert!(!file.exists());
        assert_eq!(fs::read(&to).unwrap(), b"an invoice");

        let _ = fs::remove_dir_all(&folder);
    }

    /// **Nothing is replaced that was not named.** The person approved a
    /// sentence about one file; the file that was already called that is not in
    /// it.
    #[test]
    fn a_name_that_is_taken_is_not_quietly_replaced() {
        let folder = a_folder_of_our_own("taken");
        let file = folder.join("march.pdf");
        let taken = folder.join("march-2026.pdf");
        fs::write(&file, b"an invoice").unwrap();
        fs::write(&taken, b"somebody else's invoice").unwrap();

        let refused = rename(&really(&file), &taken).unwrap_err();
        assert!(
            matches!(refused, Failed::AlreadyThere { .. }),
            "{refused:?}"
        );
        assert!(said(&refused).contains("choose another name"));
        // And nothing moved: both files are exactly as they were.
        assert_eq!(fs::read(&file).unwrap(), b"an invoice");
        assert_eq!(fs::read(&taken).unwrap(), b"somebody else's invoice");

        let _ = fs::remove_dir_all(&folder);
    }

    /// A move puts the file in the folder, and takes it out of the one it was
    /// in — which is the difference between a move and a copy.
    #[test]
    fn a_file_moves_into_a_folder_and_leaves_the_one_it_was_in() {
        let root = a_folder_of_our_own("move");
        let invoices = root.join("Invoices");
        let archive = root.join("Archive");
        fs::create_dir_all(&invoices).unwrap();
        fs::create_dir_all(&archive).unwrap();
        let file = invoices.join("march.pdf");
        fs::write(&file, b"an invoice").unwrap();
        let to = archive.join("march.pdf");

        let answer = move_into(&really(&file), &really(&archive), &to).unwrap();
        assert_eq!(answer.now_at(), Some(to.as_path()));
        assert!(!file.exists());
        assert_eq!(fs::read(&to).unwrap(), b"an invoice");

        let _ = fs::remove_dir_all(&root);
    }

    /// Moving a file into the folder it is already in is answered rather than
    /// done: it would be a change that changes nothing, reported as a change.
    #[test]
    fn moving_a_file_into_the_folder_it_is_in_says_so() {
        let folder = a_folder_of_our_own("already-in");
        let file = folder.join("march.pdf");
        fs::write(&file, b"an invoice").unwrap();

        let refused =
            move_into(&really(&file), &really(&folder), &folder.join("march.pdf")).unwrap_err();
        assert!(matches!(refused, Failed::AlreadyIn { .. }), "{refused:?}");
        assert!(file.exists());

        let _ = fs::remove_dir_all(&folder);
    }

    /// A folder is not a file, and moving one is not this verb — said plainly,
    /// rather than by moving a folder and calling it a file.
    #[test]
    fn a_folder_is_not_a_file_and_is_not_moved_by_these() {
        let root = a_folder_of_our_own("folders");
        let invoices = root.join("Invoices");
        let archive = root.join("Archive");
        fs::create_dir_all(&invoices).unwrap();
        fs::create_dir_all(&archive).unwrap();

        let refused = rename(&really(&invoices), &root.join("Taxes")).unwrap_err();
        assert!(matches!(refused, Failed::NotAFile { .. }), "{refused:?}");
        assert!(invoices.exists());

        let moving_one = move_into(
            &really(&invoices),
            &really(&archive),
            &archive.join("Invoices"),
        )
        .unwrap_err();
        assert!(
            matches!(moving_one, Failed::NotAFile { .. }),
            "{moving_one:?}"
        );
        assert!(invoices.exists());

        // And a folder is not somewhere a file can be moved *to* either.
        let file = invoices.join("march.pdf");
        fs::write(&file, b"an invoice").unwrap();
        let into_a_file = move_into(
            &really(&file),
            &really(&file),
            &invoices.join("march-2026.pdf"),
        )
        .unwrap_err();
        assert!(
            matches!(into_a_file, Failed::NotAFolder { .. }),
            "{into_a_file:?}"
        );

        let _ = fs::remove_dir_all(&root);
    }

    /// A file that went away between being resolved and being moved is
    /// answered as gone, and nothing else is attempted.
    #[test]
    fn a_file_that_went_away_is_answered_as_gone() {
        let folder = a_folder_of_our_own("gone");
        let file = folder.join("march.pdf");
        fs::write(&file, b"an invoice").unwrap();
        let real = really(&file);
        fs::remove_file(&file).unwrap();

        let refused = rename(&real, &folder.join("march-2026.pdf")).unwrap_err();
        assert!(matches!(refused, Failed::Gone { .. }), "{refused:?}");

        let _ = fs::remove_dir_all(&folder);
    }
}
