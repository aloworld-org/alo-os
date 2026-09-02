//! Walking a folder, without ever walking out of it.
//!
//! Two verbs need everything under a folder rather than everything in it:
//! `find_in_folder`, because a person means the folder and the folders inside
//! it, and `archive_folder`, because an archive of a folder that stopped at the
//! first level would not be an archive of it.
//!
//! # A link is never followed
//!
//! [`crate::Touching`] resolves the paths a call **names**. Nothing resolves
//! the paths a walk **finds**, because a walk that followed a link would leave
//! the granted folder by a door the grants were never asked about — the escape
//! the whole crate exists to stop, arriving from inside. So a link found on the
//! way is counted and stepped over, at every depth, and what a verb does about
//! that count is the verb's business.
//!
//! Not following links is also what makes a walk finish: there are no cycles in
//! a tree of real folders, so nothing here needs to remember where it has been.
//!
//! # It is bounded, and it says when the bound was reached
//!
//! A granted folder can hold a million things. An answer that tried to carry
//! them would fill a person's screen and a model's context, and an archive of
//! them would be one file nobody can open. So a walk stops at [`MOST_WALKED`]
//! and says that it stopped; `find_in_folder` reports it as a search cut short,
//! and `archive_folder` refuses rather than making an archive missing the half
//! nobody mentioned.

use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::failed::Failed;
use crate::named::{Kind, can_be_shown};

/// The most things one walk looks at.
///
/// Big enough for the folders people keep documents in, and small enough that
/// the answer is still one a person can be shown. Under the 65,535 an archive's
/// own format can hold, so a walk that finishes is always a walk that can be
/// archived.
pub(crate) const MOST_WALKED: usize = 20_000;

/// One thing a walk found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Step {
    /// Where it is.
    pub(crate) at: PathBuf,
    /// Where it is, relative to the folder the walk started at, spelled with
    /// the separators of this machine.
    pub(crate) below: PathBuf,
    /// What it is, decided without following anything.
    pub(crate) kind: Kind,
    /// How many bytes it holds, for a file.
    pub(crate) bytes: u64,
    /// When it was last written, as far as this machine will say.
    pub(crate) when: SystemTime,
}

/// What a walk found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Walked {
    /// What is under the folder, each folder before the things inside it.
    pub(crate) things: Vec<Step>,
    /// How many links were stepped over.
    pub(crate) links: usize,
    /// How many things were left out because their names could not be shown.
    pub(crate) could_not_be_named: usize,
    /// Whether the walk stopped before it had been everywhere.
    pub(crate) cut_short: bool,
}

/// Everything under this folder, to any depth, following nothing.
///
/// The folder itself is not in the answer: a walk answers what is *in* a
/// folder, and the folder is what was asked about.
///
/// # Errors
/// [`Failed`] if the folder is not there, is not a folder, or the machine
/// would not read it. A folder found on the way that cannot be read stops the
/// walk rather than being skipped, because a search that quietly leaves out
/// half a folder answers "it is not here" about something that is.
pub(crate) fn walk(folder: &Path, most: usize) -> Result<Walked, Failed> {
    let mut things = Vec::new();
    let mut links = 0;
    let mut could_not_be_named = 0;
    let mut cut_short = false;
    let mut pending = VecDeque::new();
    pending.push_back(PathBuf::new());

    while let Some(below) = pending.pop_front() {
        let here = folder.join(&below);
        let mut here_holds = Vec::new();
        let reading = fs::read_dir(&here).map_err(|why| Failed::machine(&here, "listed", &why))?;
        for entry in reading {
            let entry = entry.map_err(|why| Failed::machine(&here, "listed", &why))?;
            let name = entry.file_name();
            let Some(name) = name.to_str().filter(|name| can_be_shown(name)) else {
                could_not_be_named += 1;
                continue;
            };
            let what = entry
                .file_type()
                .map_err(|why| Failed::machine(&entry.path(), "looked at", &why))?;
            let kind = Kind::of(what);
            if kind == Kind::Link {
                links += 1;
                continue;
            }
            let about = entry.metadata().ok();
            here_holds.push(Step {
                at: entry.path(),
                below: below.join(name),
                kind,
                bytes: about.as_ref().map(fs::Metadata::len).unwrap_or(0),
                when: about
                    .and_then(|about| about.modified().ok())
                    .unwrap_or(SystemTime::UNIX_EPOCH),
            });
        }
        here_holds.sort_by(|one, other| one.below.cmp(&other.below));
        for step in here_holds {
            if things.len() >= most {
                cut_short = true;
                break;
            }
            if step.kind == Kind::Folder {
                pending.push_back(step.below.clone());
            }
            things.push(step);
        }
        if cut_short {
            break;
        }
    }

    Ok(Walked {
        things,
        links,
        could_not_be_named,
        cut_short,
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::a_folder_of_our_own;

    /// A walk goes all the way down, in an order a person could read, with
    /// every folder before the things inside it — which is also the order an
    /// archive has to be written in.
    #[test]
    fn a_walk_goes_all_the_way_down_in_an_order_that_can_be_read() {
        let folder = a_folder_of_our_own("walk");
        fs::create_dir_all(folder.join("2026/March")).unwrap();
        fs::write(folder.join("2026/March/march.pdf"), b"an invoice").unwrap();
        fs::write(folder.join("notes.txt"), b"hello").unwrap();

        let walked = walk(&folder, MOST_WALKED).unwrap();
        let found: Vec<_> = walked
            .things
            .iter()
            .map(|step| step.below.to_string_lossy().replace('\\', "/"))
            .collect();
        assert_eq!(
            found,
            ["2026", "notes.txt", "2026/March", "2026/March/march.pdf"]
        );
        assert!(!walked.cut_short);
        assert_eq!(walked.links, 0);

        let invoice = walked
            .things
            .iter()
            .find(|step| step.below.ends_with("march.pdf"))
            .unwrap();
        assert_eq!(invoice.kind, Kind::File);
        assert_eq!(invoice.bytes, 10);

        let _ = fs::remove_dir_all(&folder);
    }

    /// **A walk that reached its bound says so.** A search that answered "it is
    /// not here" having stopped looking would be worse than one that answered
    /// nothing at all.
    #[test]
    fn a_walk_that_stops_early_says_that_it_stopped() {
        let folder = a_folder_of_our_own("bounded");
        for which in 0..5 {
            fs::write(folder.join(format!("{which}.txt")), b"x").unwrap();
        }

        let walked = walk(&folder, 3).unwrap();
        assert_eq!(walked.things.len(), 3);
        assert!(walked.cut_short);

        let all = walk(&folder, 5).unwrap();
        assert_eq!(all.things.len(), 5);
        assert!(!all.cut_short);

        let _ = fs::remove_dir_all(&folder);
    }

    /// **The escape, arriving from inside.** A link inside the folder is
    /// counted and stepped over — following it would leave the granted folder
    /// by a door the grants were never asked about.
    #[cfg(unix)]
    #[test]
    fn a_link_found_on_the_way_is_counted_and_not_followed() {
        let root = a_folder_of_our_own("links");
        let invoices = root.join("Invoices");
        let elsewhere = root.join("Elsewhere");
        fs::create_dir_all(&invoices).unwrap();
        fs::create_dir_all(&elsewhere).unwrap();
        fs::write(elsewhere.join("secret.txt"), b"not an invoice").unwrap();
        std::os::unix::fs::symlink(&elsewhere, invoices.join("everything")).unwrap();
        std::os::unix::fs::symlink(elsewhere.join("secret.txt"), invoices.join("march.pdf"))
            .unwrap();

        let walked = walk(&invoices, MOST_WALKED).unwrap();
        assert!(walked.things.is_empty(), "{:?}", walked.things);
        assert_eq!(walked.links, 2);

        let _ = fs::remove_dir_all(&root);
    }

    /// A folder that is not there, and a folder that is a file, are answered in
    /// words rather than as an error number.
    #[test]
    fn what_is_not_a_folder_is_answered_in_words() {
        let folder = a_folder_of_our_own("nothing");
        let gone = walk(&folder.join("Taxes"), MOST_WALKED).unwrap_err();
        assert!(matches!(gone, Failed::Gone { .. }), "{gone:?}");

        fs::write(folder.join("march.pdf"), b"an invoice").unwrap();
        let not_a_folder = walk(&folder.join("march.pdf"), MOST_WALKED).unwrap_err();
        assert!(
            !matches!(not_a_folder, Failed::Gone { .. }),
            "{not_a_folder:?}"
        );

        let _ = fs::remove_dir_all(&folder);
    }
}
