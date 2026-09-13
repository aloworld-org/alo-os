//! Walking a folder, without ever walking out of it.
//!
//! Two verbs need everything under a folder rather than everything in it:
//! `find_in_folder`, because a person means the folder and the folders inside
//! it, and `archive_folder`, because an archive of a folder that stopped at the
//! first level would not be an archive of it. And one measurement needs it —
//! *what is filling the disk*, in `alo-measuring` — which is why the walk is
//! public: a second walker would be two opinions about what a link is, and
//! this crate exists because there is one.
//!
//! # A link is never followed
//!
//! [`crate::Touching`] resolves the paths a call **names**. Nothing resolves
//! the paths a walk **finds**, because a walk that followed a link would leave
//! the granted folder by a door the grants were never asked about — the escape
//! the whole crate exists to stop, arriving from inside. So a link found on the
//! way is stepped over, at every depth, and what a verb does about the count of
//! them is the verb's business.
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
//! nobody mentioned. And it says **which folders it never finished**, so that
//! a measurement can mark them rather than report a half-counted folder as a
//! whole one.
//!
//! # Searching and measuring are two policies over one walk
//!
//! A search must be complete or fail: a folder on the way that cannot be read
//! stops the walk, because "it is not here" about a folder nobody looked in is
//! a false answer. A measurement must be complete or honest: the same folder
//! is a node that says it could not be read, and the walk goes on, because a
//! person asking what is filling their disk wants the answer for everything
//! that *could* be counted and a note on what could not. The walk is the same
//! code, deciding the same things about a link and a name, and [`Walking`]
//! carries the three differences between the two: whether a link is kept as a
//! step with its own size, whether an unreadable folder is noted or fatal, and
//! whether a folder on another filesystem is entered.

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
pub const MOST_WALKED: usize = 20_000;

/// One thing a walk found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    /// Where it is.
    pub at: PathBuf,
    /// Where it is, relative to the folder the walk started at, spelled with
    /// the separators of this machine.
    pub below: PathBuf,
    /// What it is, decided without following anything.
    pub kind: Kind,
    /// How many bytes it holds, for a file — or, for a link a measuring walk
    /// kept, the bytes of the link itself and never of what it points at.
    pub bytes: u64,
    /// When it was last written, as far as this machine will say.
    pub when: SystemTime,
}

/// What a walk found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Walked {
    /// What is under the folder, each folder before the things inside it.
    pub things: Vec<Step>,
    /// How many links were found: stepped over by a searching walk, kept as
    /// steps by a measuring one.
    pub links: usize,
    /// How many things were left out because their names could not be shown.
    pub could_not_be_named: usize,
    /// Whether the walk stopped before it had been everywhere.
    pub cut_short: bool,
    /// Folders the machine would not let the walk read, each with what it
    /// said. Always empty for a searching walk, which fails instead.
    pub unread: Vec<Unread>,
    /// Folders, relative to where the walk started, that are on another
    /// filesystem and were not entered. Always empty for a searching walk,
    /// which enters them.
    pub elsewhere: Vec<PathBuf>,
    /// Folders, relative to where the walk started, whose contents the walk
    /// had not finished listing when it stopped at its bound — the one it was
    /// in, and every one it had found and not yet entered. Empty unless
    /// [`Self::cut_short`].
    pub not_entered: Vec<PathBuf>,
}

/// A folder a measuring walk found and could not read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unread {
    /// Where it is, relative to the folder the walk started at.
    pub below: PathBuf,
    /// What the machine said when asked to list it, as a sentence.
    pub why: String,
}

/// How a walk treats the things a walk has to decide about.
///
/// Two policies, named for the two callers, and made here rather than by the
/// callers so that a verb cannot pick a combination nobody argued for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Walking {
    /// The most things this walk looks at.
    most: usize,
    /// Whether a link is kept as a step, with its own size, rather than only
    /// counted.
    links_kept: bool,
    /// Whether a folder the machine would not read is noted and stepped over
    /// rather than ending the walk.
    unreadable_noted: bool,
    /// Whether a folder on another filesystem is noted and not entered.
    one_filesystem: bool,
}

impl Walking {
    /// The walk a search or an archive makes: links counted and stepped over,
    /// an unreadable folder fatal, every filesystem entered.
    #[must_use]
    pub const fn searching(most: usize) -> Self {
        Self {
            most,
            links_kept: false,
            unreadable_noted: false,
            one_filesystem: false,
        }
    }

    /// The walk a measurement makes: a link is a step holding the bytes of
    /// the link itself, an unreadable folder is noted and stepped over, and a
    /// folder on another filesystem is noted and not entered.
    ///
    /// Where this machine cannot say which filesystem a folder is on, every
    /// folder is entered and nothing is noted as elsewhere; on Linux, which is
    /// where `alo-measuring` measures, it can.
    #[must_use]
    pub const fn measuring(most: usize) -> Self {
        Self {
            most,
            links_kept: true,
            unreadable_noted: true,
            one_filesystem: true,
        }
    }

    /// Everything under this folder, to any depth, following nothing.
    ///
    /// The folder itself is not in the answer: a walk answers what is *in* a
    /// folder, and the folder is what was asked about.
    ///
    /// # Errors
    /// [`Failed`] if the folder is not there, is not a folder, or the machine
    /// would not read it. For a searching walk, a folder found on the way that
    /// cannot be read stops the walk too, because a search that quietly leaves
    /// out half a folder answers "it is not here" about something that is; a
    /// measuring walk notes it in [`Walked::unread`] and goes on.
    pub fn through(&self, folder: &Path) -> Result<Walked, Failed> {
        let mut walked = Walked {
            things: Vec::new(),
            links: 0,
            could_not_be_named: 0,
            cut_short: false,
            unread: Vec::new(),
            elsewhere: Vec::new(),
            not_entered: Vec::new(),
        };
        let filesystem = if self.one_filesystem {
            Some(filesystem_of(folder).map_err(|why| Failed::machine(folder, "listed", &why))?)
        } else {
            None
        };
        let mut pending = VecDeque::new();
        pending.push_back(PathBuf::new());

        while let Some(below) = pending.pop_front() {
            let here = folder.join(&below);
            let is_the_root = below.as_os_str().is_empty();
            if let Some(started_on) = filesystem
                && !is_the_root
            {
                match filesystem_of(&here) {
                    Ok(on) if on == started_on => {}
                    Ok(_) => {
                        walked.elsewhere.push(below);
                        continue;
                    }
                    Err(why) => {
                        walked.unread.push(Unread {
                            below,
                            why: why.to_string(),
                        });
                        continue;
                    }
                }
            }
            let reading = match fs::read_dir(&here) {
                Ok(reading) => reading,
                Err(why) if self.unreadable_noted && !is_the_root => {
                    walked.unread.push(Unread {
                        below,
                        why: why.to_string(),
                    });
                    continue;
                }
                Err(why) => return Err(Failed::machine(&here, "listed", &why)),
            };
            let mut here_holds = Vec::new();
            for entry in reading {
                let entry = entry.map_err(|why| Failed::machine(&here, "listed", &why))?;
                let name = entry.file_name();
                let Some(name) = name.to_str().filter(|name| can_be_shown(name)) else {
                    walked.could_not_be_named += 1;
                    continue;
                };
                let what = entry
                    .file_type()
                    .map_err(|why| Failed::machine(&entry.path(), "looked at", &why))?;
                let kind = Kind::of(what);
                if kind == Kind::Link {
                    walked.links += 1;
                    if !self.links_kept {
                        continue;
                    }
                }
                // `DirEntry::metadata` does not follow a link, so for one this
                // is the size of the link itself.
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
                if walked.things.len() >= self.most {
                    walked.cut_short = true;
                    break;
                }
                if step.kind == Kind::Folder {
                    pending.push_back(step.below.clone());
                }
                walked.things.push(step);
            }
            if walked.cut_short {
                walked.not_entered.push(below);
                walked.not_entered.extend(pending.drain(..));
                break;
            }
        }

        Ok(walked)
    }
}

/// Everything under this folder, the way a search or an archive walks it.
///
/// # Errors
/// As [`Walking::through`], for a searching walk.
pub(crate) fn walk(folder: &Path, most: usize) -> Result<Walked, Failed> {
    Walking::searching(most).through(folder)
}

/// Which filesystem a folder is on, as a number two folders can be compared
/// by, read without following a link.
#[cfg(unix)]
fn filesystem_of(at: &Path) -> std::io::Result<u64> {
    use std::os::unix::fs::MetadataExt;
    fs::symlink_metadata(at).map(|about| about.dev())
}

/// Which filesystem a folder is on — which this machine cannot say, so every
/// folder is on the one the walk started on.
#[cfg(not(unix))]
fn filesystem_of(at: &Path) -> std::io::Result<u64> {
    fs::symlink_metadata(at).map(|_| 0)
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
        assert!(walked.not_entered.is_empty());
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

    /// **A walk that reached its bound says so**, and says which folders it
    /// had not finished. A search that answered "it is not here" having
    /// stopped looking would be worse than one that answered nothing at all,
    /// and a size for a folder the walk never entered would be a size that is
    /// silently too small.
    #[test]
    fn a_walk_that_stops_early_says_that_it_stopped_and_where() {
        let folder = a_folder_of_our_own("bounded");
        fs::create_dir_all(folder.join("Later")).unwrap();
        fs::write(folder.join("Later/z.txt"), b"x").unwrap();
        for which in 0..5 {
            fs::write(folder.join(format!("z{which}.txt")), b"x").unwrap();
        }

        let walked = walk(&folder, 3).unwrap();
        assert_eq!(walked.things.len(), 3);
        assert!(walked.cut_short);
        // The root was being listed, and `Later` — sorted before the files —
        // had been found and not entered.
        assert_eq!(
            walked.not_entered,
            [PathBuf::new(), PathBuf::from("Later")],
            "{:?}",
            walked.not_entered
        );

        let all = walk(&folder, 7).unwrap();
        assert_eq!(all.things.len(), 7);
        assert!(!all.cut_short);
        assert!(all.not_entered.is_empty());

        let _ = fs::remove_dir_all(&folder);
    }

    /// **The escape, arriving from inside.** A link inside the folder is
    /// counted and stepped over by a search — following it would leave the
    /// granted folder by a door the grants were never asked about.
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

    /// **A measuring walk keeps a link as a step and still never follows it.**
    /// Its bytes are the bytes of the link itself — the length of the name it
    /// points at — and nothing behind it is walked, so a link to somewhere
    /// large outside the folder cannot make the folder look large.
    #[cfg(unix)]
    #[test]
    fn a_measuring_walk_keeps_a_link_as_its_own_bytes_and_never_follows_it() {
        let root = a_folder_of_our_own("measured-links");
        let invoices = root.join("Invoices");
        let elsewhere = root.join("Elsewhere");
        fs::create_dir_all(&invoices).unwrap();
        fs::create_dir_all(&elsewhere).unwrap();
        fs::write(elsewhere.join("big.bin"), vec![0; 100_000]).unwrap();
        std::os::unix::fs::symlink(&elsewhere, invoices.join("everything")).unwrap();
        std::os::unix::fs::symlink(elsewhere.join("big.bin"), invoices.join("big.bin")).unwrap();

        let walked = Walking::measuring(MOST_WALKED).through(&invoices).unwrap();
        assert_eq!(walked.links, 2);
        assert_eq!(walked.things.len(), 2, "{:?}", walked.things);
        for step in &walked.things {
            assert_eq!(step.kind, Kind::Link, "{step:?}");
            let link_itself = fs::read_link(&step.at).unwrap();
            assert_eq!(step.bytes, link_itself.as_os_str().len() as u64, "{step:?}");
        }
        assert!(
            walked
                .things
                .iter()
                .all(|step| step.below.components().count() == 1),
            "nothing behind a link was walked: {:?}",
            walked.things
        );

        let _ = fs::remove_dir_all(&root);
    }

    /// **A folder the machine will not read ends a search and is noted by a
    /// measurement.** Made unreadable by giving it a name the machine cannot
    /// open — a path longer than the kernel accepts — because that refuses the
    /// administrator too, and the supervisor's gates run as one; a folder with
    /// its permissions taken away is refused by the same `read_dir` and noted
    /// by the same line.
    #[cfg(target_os = "linux")]
    #[test]
    fn a_folder_that_cannot_be_read_ends_a_search_and_is_noted_by_a_measurement() {
        let root = a_folder_of_our_own("unreadable");
        fs::write(root.join("counted.txt"), b"seven b").unwrap();
        let deep = a_folder_too_deep_to_open(&root);

        let searched = walk(&root, MOST_WALKED).unwrap_err();
        assert!(
            matches!(searched, Failed::TheMachineSaidNo { .. }),
            "{searched:?}"
        );

        let measured = Walking::measuring(MOST_WALKED).through(&root).unwrap();
        assert_eq!(measured.unread.len(), 1, "{:?}", measured.unread);
        let unread = measured.unread.first().unwrap();
        assert!(unread.below.starts_with("deep"), "{unread:?}");
        assert!(!unread.why.is_empty(), "{unread:?}");
        assert!(deep.starts_with(&root));
        assert!(
            measured
                .things
                .iter()
                .any(|step| step.below == Path::new("counted.txt") && step.bytes == 7),
            "the rest of the folder is still counted"
        );
        assert!(!measured.cut_short);

        let _ = fs::remove_dir_all(&root);
    }

    /// A folder nested so deep that its full path is longer than the kernel
    /// will open, made one level at a time relative to an open handle so
    /// that making it needs no path that long.
    ///
    /// Answers the deepest folder's path, which is longer than `PATH_MAX`.
    #[cfg(target_os = "linux")]
    fn a_folder_too_deep_to_open(root: &Path) -> PathBuf {
        use rustix::fs::{Mode, OFlags, mkdirat, openat};

        let mut at = root.join("deep");
        fs::create_dir(&at).unwrap();
        let mut handle = openat(
            rustix::fs::CWD,
            &at,
            OFlags::PATH | OFlags::DIRECTORY,
            Mode::empty(),
        )
        .unwrap();
        // Each level adds two bytes; the kernel's limit is 4096.
        while at.as_os_str().len() <= 4200 {
            mkdirat(&handle, "d", Mode::RWXU).unwrap();
            handle = openat(
                &handle,
                "d",
                OFlags::PATH | OFlags::DIRECTORY,
                Mode::empty(),
            )
            .unwrap();
            at.push("d");
        }
        at
    }

    /// **A measuring walk stops at another filesystem and says so.** `/proc`
    /// is the one filesystem every Linux machine has that is not the one a
    /// person's files are on, and a walk into it would be counting the
    /// kernel's view of every process as bytes on the disk.
    #[cfg(target_os = "linux")]
    #[test]
    fn a_measuring_walk_does_not_enter_another_filesystem_and_says_which() {
        let root = a_folder_of_our_own("filesystems");
        // A folder on this filesystem that *looks* like a mount point is
        // entered; the real one is what stops the walk.
        fs::create_dir_all(root.join("proc")).unwrap();
        fs::write(root.join("proc/notes.txt"), b"here").unwrap();

        let measured = Walking::measuring(MOST_WALKED).through(&root).unwrap();
        assert!(measured.elsewhere.is_empty(), "{:?}", measured.elsewhere);
        assert!(
            measured
                .things
                .iter()
                .any(|step| step.below.ends_with("notes.txt"))
        );

        // Enough to list every folder directly under the root before the
        // bound: the walk is breadth-first, so `/proc` is reached — and
        // refused — before anything two levels down is.
        let slash = Walking::measuring(5000).through(Path::new("/")).unwrap();
        assert!(
            slash
                .elsewhere
                .iter()
                .any(|below| below == Path::new("proc")),
            "{:?}",
            slash.elsewhere
        );
        assert!(
            !slash
                .things
                .iter()
                .any(|step| step.below.starts_with("proc") && step.below != Path::new("proc")),
            "nothing under /proc was walked"
        );

        let _ = fs::remove_dir_all(&root);
    }

    /// A folder that is not there, and a folder that is a file, are answered in
    /// words rather than as an error number — by both policies, because the
    /// folder the walk was asked about is the one thing a measurement cannot
    /// note and go on from.
    #[test]
    fn what_is_not_a_folder_is_answered_in_words() {
        let folder = a_folder_of_our_own("nothing");
        let gone = walk(&folder.join("Taxes"), MOST_WALKED).unwrap_err();
        assert!(matches!(gone, Failed::Gone { .. }), "{gone:?}");
        let gone = Walking::measuring(MOST_WALKED)
            .through(&folder.join("Taxes"))
            .unwrap_err();
        assert!(matches!(gone, Failed::Gone { .. }), "{gone:?}");

        fs::write(folder.join("march.pdf"), b"an invoice").unwrap();
        let not_a_folder = walk(&folder.join("march.pdf"), MOST_WALKED).unwrap_err();
        assert!(
            !matches!(not_a_folder, Failed::Gone { .. }),
            "{not_a_folder:?}"
        );
        let not_a_folder = Walking::measuring(MOST_WALKED)
            .through(&folder.join("march.pdf"))
            .unwrap_err();
        assert!(
            !matches!(not_a_folder, Failed::Gone { .. }),
            "{not_a_folder:?}"
        );

        let _ = fs::remove_dir_all(&folder);
    }
}
