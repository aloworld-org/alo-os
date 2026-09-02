//! Making one archive file out of a folder.
//!
//! "Archive" here means *make an archive*, not *move it to the archive folder*
//! — the contract says so, and the second is `move_file` under another name.
//! What the format is, and why it is a stored zip, is [`crate::zip`]'s; this
//! file decides **what goes in**, which is a question about grants rather than
//! about bytes.
//!
//! # Three things never go in
//!
//! **A link does not.** An archive of a granted folder that followed a link
//! inside it would copy whatever the link points at — a key, another person's
//! folder, a disk nobody granted — into a file the agent may then move
//! anywhere it has a grant. That is the escape this crate exists to stop,
//! wearing a different hat, and [`crate::walking`] is where it is stepped over.
//! The count comes back in the answer, because a person told *twelve things
//! were archived* should also be told that one was left where it was.
//!
//! **Nothing that is not a file or a folder does.** A socket or a device is not
//! a thing an archive can hold, and a copy of one would be a file that lies
//! about what it is.
//!
//! **The archive itself does not.** An archive written inside the folder it is
//! an archive of is refused outright rather than skipped, because *"make an
//! archive of Invoices, in Invoices"* is a sentence whose answer changes the
//! folder it is about while it is being read.
//!
//! # It is bounded, and a bound is a refusal rather than a shorter archive
//!
//! An archive missing the half nobody mentioned is worse than no archive: the
//! person keeps it, deletes the folder, and finds out later. So a folder with
//! more in it than one archive holds is refused, in words that say to archive
//! one of the folders inside it instead.

use std::fs;
use std::path::Path;

use crate::answer::{Answer, Archived};
use crate::failed::Failed;
use crate::named::Kind;
use crate::real::Real;
use crate::walking::{MOST_WALKED, walk};
use crate::zip::Archive;

/// The most bytes one archive holds.
///
/// Two gigabytes, which is well inside what the format can address without the
/// extensions this crate does not write, and far more than the folders of
/// documents this verb exists for.
const MOST_ARCHIVED: u64 = 2 * 1024 * 1024 * 1024;

/// The ending an archive's name has to have.
const A_ZIP: &str = ".zip";

/// Whether a name says what the archive actually is.
///
/// A name is a person's word for the file they are about to have. Writing a
/// zip called `invoices.tar.gz` would hand somebody a file whose name lies
/// about what is in it, and appending `.zip` to a name they approved would hand
/// them a file with a name they did not agree to. So it is a refusal, and it is
/// made before anything is written.
pub(crate) fn is_an_archive_name(name: &str) -> bool {
    name.to_lowercase().ends_with(A_ZIP) && name.chars().count() > A_ZIP.chars().count()
}

/// Make one archive file out of a folder.
///
/// # Errors
/// [`Failed`] if the folder is not a folder, holds more than one archive holds,
/// the archive would go inside it, something is already at the archive's name,
/// or the machine would not read or write.
pub(crate) fn archive(folder: &Real, into: &Real, at: &Path) -> Result<Answer, Failed> {
    let from = folder.as_path();
    let what = fs::symlink_metadata(from).map_err(|why| Failed::machine(from, "archived", &why))?;
    if !what.is_dir() {
        return Err(Failed::NotAFolder {
            path: from.display().to_string(),
        });
    }
    let holding = into.as_path();
    let what =
        fs::symlink_metadata(holding).map_err(|why| Failed::machine(holding, "written", &why))?;
    if !what.is_dir() {
        return Err(Failed::NotAFolder {
            path: holding.display().to_string(),
        });
    }
    // Both are resolved, so this is a comparison of two real places rather than
    // of two spellings.
    if holding == from || holding.starts_with(from) {
        return Err(Failed::IntoItself {
            folder: from.display().to_string(),
        });
    }

    let walked = walk(from, MOST_WALKED)?;
    if walked.cut_short {
        return Err(Failed::TooMany {
            folder: from.display().to_string(),
            most: MOST_WALKED,
        });
    }
    let held: u64 = walked.things.iter().map(|step| step.bytes).sum();
    if held > MOST_ARCHIVED {
        return Err(Failed::TooMuch {
            folder: from.display().to_string(),
            most: MOST_ARCHIVED,
        });
    }

    let left_out = walked.links + walked.could_not_be_named;
    // Made before anything is written, and outside what is cleaned up below:
    // this fails when something is already at that name, and **what was already
    // there is not this call's to remove.**
    let archive = Archive::beginning(at, MOST_ARCHIVED)?;
    match write(archive, &walked.things) {
        Ok((things, bytes)) => Ok(Answer::Archived(Archived::of(
            at.to_owned(),
            things,
            left_out,
            bytes,
        ))),
        Err(why) => {
            // A half-written archive is a file somebody would later open and
            // find half of their folder in. It goes away with the attempt that
            // made it — and only ever the one this call created.
            let _ = fs::remove_file(at);
            Err(why)
        }
    }
}

/// Write the things a walk found into an archive that has been begun, and say
/// how many went in and how big it came out.
fn write(mut archive: Archive, things: &[crate::walking::Step]) -> Result<(usize, u64), Failed> {
    let mut went = 0;
    for step in things {
        let name = step.below.to_string_lossy();
        match step.kind {
            Kind::Folder => archive.folder(&name, step.when)?,
            Kind::File => archive.file(&name, &step.at, step.when)?,
            // A walk never answers with a link, and a socket is not a thing an
            // archive can hold. Neither is an error; both are left where they
            // are, and counted.
            Kind::Link | Kind::Other => continue,
        }
        went += 1;
    }
    let bytes = archive.finish()?;
    Ok((went, bytes))
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{a_folder_of_our_own, really};

    /// A name that does not say what the file is, is not a name this verb
    /// accepts.
    #[test]
    fn an_archive_is_called_something_that_says_what_it_is() {
        for good in ["invoices.zip", "Invoices 2026.ZIP", "a.zip"] {
            assert!(is_an_archive_name(good), "{good}");
        }
        for bad in [
            "invoices",
            "invoices.tar.gz",
            "invoices.zip.pdf",
            ".zip",
            "",
        ] {
            assert!(!is_an_archive_name(bad), "{bad}");
        }
    }

    /// The ordinary day: a folder goes in, an archive comes out, and what came
    /// out is a zip that says how many things are in it.
    #[test]
    fn a_folder_becomes_one_archive_file() {
        let root = a_folder_of_our_own("archive");
        let invoices = root.join("Invoices");
        let keep = root.join("Archive");
        fs::create_dir_all(invoices.join("2026")).unwrap();
        fs::create_dir_all(&keep).unwrap();
        fs::write(invoices.join("march.pdf"), b"an invoice").unwrap();
        fs::write(invoices.join("2026/april.pdf"), b"another invoice").unwrap();

        let at = keep.join("invoices.zip");
        let answer = archive(&really(&invoices), &really(&keep), &at).unwrap();
        let archived = answer.archived().unwrap();
        assert_eq!(archived.at(), at);
        assert_eq!(archived.things(), 3, "two files and the folder they are in");
        assert_eq!(archived.left_out(), 0);

        let bytes = fs::read(&at).unwrap();
        assert_eq!(bytes.len() as u64, archived.bytes());
        assert_eq!(bytes.get(..4), Some([0x50, 0x4b, 0x03, 0x04].as_slice()));
        assert!(
            String::from_utf8_lossy(&bytes).contains("2026/april.pdf"),
            "the names inside an archive are spelled the way the format spells them"
        );

        let _ = fs::remove_dir_all(&root);
    }

    /// **An archive is never written inside the folder it is an archive of.**
    /// Otherwise the answer to the sentence changes the folder the sentence is
    /// about while it is being written.
    #[test]
    fn an_archive_is_refused_when_it_would_go_inside_what_it_is_an_archive_of() {
        let root = a_folder_of_our_own("itself");
        let invoices = root.join("Invoices");
        let inside = invoices.join("2026");
        fs::create_dir_all(&inside).unwrap();
        fs::write(invoices.join("march.pdf"), b"an invoice").unwrap();

        let into_itself = archive(
            &really(&invoices),
            &really(&invoices),
            &invoices.join("invoices.zip"),
        )
        .unwrap_err();
        assert!(
            matches!(into_itself, Failed::IntoItself { .. }),
            "{into_itself:?}"
        );

        let deeper = archive(
            &really(&invoices),
            &really(&inside),
            &inside.join("invoices.zip"),
        )
        .unwrap_err();
        assert!(matches!(deeper, Failed::IntoItself { .. }), "{deeper:?}");
        assert!(!invoices.join("invoices.zip").exists());

        let _ = fs::remove_dir_all(&root);
    }

    /// **A link is left where it is, and counted.** An archive that followed
    /// one would copy whatever it points at into a file the agent may then move
    /// anywhere it has a grant.
    #[cfg(unix)]
    #[test]
    fn a_link_inside_the_folder_is_left_out_and_counted() {
        let root = a_folder_of_our_own("archive-links");
        let invoices = root.join("Invoices");
        let elsewhere = root.join("Elsewhere");
        let keep = root.join("Archive");
        fs::create_dir_all(&invoices).unwrap();
        fs::create_dir_all(&elsewhere).unwrap();
        fs::create_dir_all(&keep).unwrap();
        fs::write(invoices.join("march.pdf"), b"an invoice").unwrap();
        let secret = elsewhere.join("secret.txt");
        fs::write(&secret, b"not an invoice").unwrap();
        std::os::unix::fs::symlink(&secret, invoices.join("keys.txt")).unwrap();

        let at = keep.join("invoices.zip");
        let answer = archive(&really(&invoices), &really(&keep), &at).unwrap();
        let archived = answer.archived().unwrap();
        assert_eq!(archived.things(), 1);
        assert_eq!(archived.left_out(), 1);

        let bytes = fs::read(&at).unwrap();
        let inside = String::from_utf8_lossy(&bytes);
        assert!(!inside.contains("not an invoice"), "a link was followed");
        assert!(!inside.contains("keys.txt"), "a link was archived");

        let _ = fs::remove_dir_all(&root);
    }

    /// Nothing is written over. An archive whose name is taken is refused, and
    /// what was there is untouched.
    #[test]
    fn an_archive_never_replaces_what_is_already_there() {
        let root = a_folder_of_our_own("archive-taken");
        let invoices = root.join("Invoices");
        let keep = root.join("Archive");
        fs::create_dir_all(&invoices).unwrap();
        fs::create_dir_all(&keep).unwrap();
        fs::write(invoices.join("march.pdf"), b"an invoice").unwrap();
        let at = keep.join("invoices.zip");
        fs::write(&at, b"somebody else's archive").unwrap();

        let refused = archive(&really(&invoices), &really(&keep), &at).unwrap_err();
        assert!(
            matches!(refused, Failed::AlreadyThere { .. }),
            "{refused:?}"
        );
        assert_eq!(fs::read(&at).unwrap(), b"somebody else's archive");

        let _ = fs::remove_dir_all(&root);
    }

    /// An empty folder is an archive of nothing rather than a failure: a person
    /// who archives an empty folder gets an empty archive, which is what they
    /// asked for.
    #[test]
    fn an_empty_folder_is_an_empty_archive() {
        let root = a_folder_of_our_own("archive-empty");
        let invoices = root.join("Invoices");
        let keep = root.join("Archive");
        fs::create_dir_all(&invoices).unwrap();
        fs::create_dir_all(&keep).unwrap();

        let at = keep.join("invoices.zip");
        let answer = archive(&really(&invoices), &really(&keep), &at).unwrap();
        let archived = answer.archived().unwrap();
        assert_eq!(archived.things(), 0);
        assert_eq!(
            archived.bytes(),
            22,
            "the record at the end, and nothing else"
        );

        let _ = fs::remove_dir_all(&root);
    }
}
