//! Turning a walk into an index, reading only what changed.
//!
//! The walk is `alo-files`' measuring policy, walked on from every folder one
//! walk left unentered by `walking_on.rs` until the folder is gathered whole:
//! a link is a step with its own bytes and is never followed, a folder the
//! machine would not read is noted and stepped over rather than ending the
//! walk, and a folder on another filesystem is noted and not entered. A
//! search must be honest rather than complete — *this folder could not be
//! read* beside the answer is a true answer, and a whole index refused
//! because one subfolder belongs to somebody else is a person locked out of
//! the rest of their documents.
//!
//! # Unchanged means the same size and the same time
//!
//! A file is read only when the previous index has no entry for it, or has
//! one with a different size or a different modification time. That is the
//! rule `make` and every backup tool use, and its known gap is the same: a
//! file rewritten with the same bytes count within the filesystem's clock
//! resolution is not seen. The report says so.
//!
//! # Nothing recurses, and the previous index is read once
//!
//! The gathering is flat, in the walks' order, and the previous entries go
//! into a map by path before it is read; each step is then one lookup.

use std::collections::HashMap;
use std::path::Path;
use std::time::SystemTime;

use crate::covered::{Covered, Unread};
use crate::entry::{Contents, Entry, Moment};
use crate::index::Index;
use crate::kind::{Kind, SNIFFED};
use crate::reading::{Looked, Reading};
use crate::refusing::NotIndexed;
use crate::walking_on::{Gathered, everything_under};
use crate::wording;

/// The index of this folder, made at `made` as the caller says it, reading a
/// file only where `previous` cannot vouch for it, gathered by walks of at
/// most `most` things each — [`alo_files::MOST_WALKED`] from the crate's
/// public way in, and smaller in a test that wants a small folder to take
/// more than one walk.
///
/// # Errors
///
/// [`NotIndexed::NotAbsolute`] for a folder named from somewhere other than
/// the root, and [`NotIndexed::NotWalked`] when the folder itself is not
/// there, is a file, or could not be read. `previous` is always an index of
/// the same folder, because the only caller that hands one in is
/// [`Index::again`] and it hands in itself.
pub(crate) fn assembled(
    folder: &Path,
    previous: Option<&Index>,
    made: SystemTime,
    reading: &mut dyn Reading,
    most: usize,
) -> Result<Index, NotIndexed> {
    if !folder.has_root() {
        return Err(NotIndexed::NotAbsolute {
            at: folder.to_path_buf(),
        });
    }
    let walked = everything_under(folder, most).map_err(|why| NotIndexed::NotWalked {
        at: folder.to_path_buf(),
        why,
    })?;
    let vouched: HashMap<&str, &Entry> = previous
        .map(|previous| {
            previous
                .entries
                .iter()
                .map(|entry| (entry.below.as_str(), entry))
                .collect()
        })
        .unwrap_or_default();

    let mut entries = Vec::with_capacity(walked.things.len());
    let mut opened = 0;
    for step in &walked.things {
        let below = spelled(&step.below);
        let modified = Moment::of(step.when);
        let (kind, contents) = match Kind::of_thing(step.kind) {
            Some(kind) => (kind, Contents::NotAFile),
            None => match vouched.get(below.as_str()) {
                Some(kept)
                    if kept.kind.is_a_file()
                        && kept.bytes == step.bytes
                        && kept.modified == modified =>
                {
                    (kept.kind, kept.contents.clone())
                }
                _ => {
                    opened += 1;
                    looked(reading.look(&step.at))
                }
            },
        };
        entries.push(Entry {
            below,
            kind,
            bytes: step.bytes,
            modified,
            contents,
        });
    }
    Ok(Index {
        of: folder.to_path_buf(),
        made: Some(Moment::of(made)),
        covered: covered_from(&walked, most),
        entries,
        opened,
    })
}

/// What a look inside a file says about its kind and its words.
fn looked(looked: Looked) -> (Kind, Contents) {
    match looked {
        Looked::Whole(bytes) => {
            let kind = Kind::of_bytes(bytes.get(..SNIFFED).unwrap_or(&bytes));
            let contents = if kind.has_words() {
                Contents::Read {
                    words: wording::words_of(&String::from_utf8_lossy(&bytes)),
                }
            } else {
                Contents::NotText
            };
            (kind, contents)
        }
        Looked::TooBig { bytes, first } => (Kind::of_bytes(&first), Contents::TooBig { bytes }),
        Looked::NotRead { why } => (Kind::Unread, Contents::NotRead { why }),
    }
}

/// What the walks could not reach, as the index keeps it; `most` is the
/// bound each walk was under.
pub(crate) fn covered_from(walked: &Gathered, most: usize) -> Covered {
    Covered {
        whole: walked.whole,
        most,
        unread: walked
            .unread
            .iter()
            .map(|unread| Unread {
                below: spelled(&unread.below),
                why: unread.why.clone(),
            })
            .collect(),
        elsewhere: walked
            .elsewhere
            .iter()
            .map(|below| spelled(below))
            .collect(),
        not_entered: walked
            .not_entered
            .iter()
            .map(|below| spelled(below))
            .collect(),
        unnamed: walked.could_not_be_named,
    }
}

/// A path below the folder, with `/` between the parts on every host.
///
/// The walk only keeps names that can be shown, so nothing here is lossy in
/// practice; the conversion is spelled as lossy because the type allows a
/// name that is not text, and refusing the whole index over one such name
/// would be the wrong trade.
fn spelled(below: &Path) -> String {
    below
        .iter()
        .map(|part| part.to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::reading::Disk;

    /// A fixed moment for every index these tests make.
    fn noon() -> SystemTime {
        SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_760_000_000)
    }

    /// A reader that counts, and looks with the disk.
    struct Counting {
        /// How many looks so far.
        looks: usize,
        /// The paths looked at, in order.
        at: Vec<PathBuf>,
    }

    impl Reading for Counting {
        fn look(&mut self, at: &Path) -> Looked {
            self.looks += 1;
            self.at.push(at.to_path_buf());
            Disk.look(at)
        }
    }

    /// The entries of an index that are files.
    fn files_of(index: &Index) -> Vec<&Entry> {
        index
            .entries
            .iter()
            .filter(|entry| entry.kind.is_a_file())
            .collect()
    }

    /// A folder of this test's own, under this machine's temporary directory.
    fn a_folder_of_our_own(what: &str) -> PathBuf {
        let folder = std::env::temp_dir().join(format!(
            "alo-finding-indexing-{}-{what}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).unwrap();
        folder.canonicalize().unwrap()
    }

    /// **A file unchanged since last time is not read again, counted by the
    /// reader itself.** The first index reads every file; the second, given
    /// the first, reads none; a file that changed is read once, and a file
    /// that is new is read once.
    #[test]
    fn only_what_changed_is_read_and_the_reader_counts() {
        let folder = a_folder_of_our_own("incremental");
        std::fs::create_dir_all(folder.join("2026")).unwrap();
        std::fs::write(folder.join("notes.txt"), b"a note").unwrap();
        std::fs::write(folder.join("2026/march.pdf"), b"%PDF-1.7").unwrap();

        let mut first = Counting {
            looks: 0,
            at: Vec::new(),
        };
        let index = assembled(&folder, None, noon(), &mut first, alo_files::MOST_WALKED).unwrap();
        assert_eq!(first.looks, 2, "{:?}", first.at);
        assert_eq!(index.opened, 2);
        assert_eq!(
            index.made,
            Some(Moment::of(noon())),
            "made when the caller said"
        );
        assert_eq!(index.entries.len(), 3, "a folder and two files");

        let mut second = Counting {
            looks: 0,
            at: Vec::new(),
        };
        let again = assembled(
            &folder,
            Some(&index),
            noon(),
            &mut second,
            alo_files::MOST_WALKED,
        )
        .unwrap();
        assert_eq!(second.looks, 0, "{:?}", second.at);
        assert_eq!(again.opened, 0);
        // The files are the same entries, read from the earlier index. A
        // folder's own time is not compared: on NTFS it settles a moment
        // after a write inside it, and a folder has no contents to vouch for.
        assert_eq!(files_of(&again), files_of(&index));
        assert_eq!(again.entries.len(), index.entries.len());

        std::fs::write(folder.join("notes.txt"), b"a longer note").unwrap();
        std::fs::write(folder.join("2026/april.txt"), b"new").unwrap();
        let mut third = Counting {
            looks: 0,
            at: Vec::new(),
        };
        let changed = assembled(
            &folder,
            Some(&again),
            noon(),
            &mut third,
            alo_files::MOST_WALKED,
        )
        .unwrap();
        assert_eq!(third.looks, 2, "{:?}", third.at);
        assert!(third.at.iter().any(|at| at.ends_with("notes.txt")));
        assert!(third.at.iter().any(|at| at.ends_with("april.txt")));
        assert_eq!(changed.opened, 2);
        let _ = std::fs::remove_dir_all(&folder);
    }

    /// **A folder larger than one walk is one index, whole, and indexed again
    /// it reads nothing** — with the bound made small here, so that the
    /// folder is a few dozen files and the walking on is still counted by the
    /// reader: every file read exactly once the first time, none the second,
    /// and the index the same as one made with a bound the folder fits under.
    #[test]
    fn a_folder_larger_than_one_walk_is_indexed_whole_and_again_reads_nothing() {
        let folder = a_folder_of_our_own("larger-than-one-walk");
        let mut files = 0;
        for f in 0..5 {
            let sub = folder.join(format!("f-{f}")).join("deeper");
            std::fs::create_dir_all(&sub).unwrap();
            for m in 0..4 {
                std::fs::write(sub.join(format!("{f}-{m}.txt")), b"a word each").unwrap();
                std::fs::write(
                    sub.parent().unwrap().join(format!("{f}-{m}.txt")),
                    b"another",
                )
                .unwrap();
                files += 2;
            }
        }
        let things = files + 10;

        let mut in_walks = Counting {
            looks: 0,
            at: Vec::new(),
        };
        let index = assembled(&folder, None, noon(), &mut in_walks, 7).unwrap();
        assert!(index.covered.whole, "{:?}", index.covered);
        assert!(index.covered.not_entered.is_empty());
        assert_eq!(index.covered.most, 7, "the bound each walk was under");
        assert_eq!(index.entries.len(), things);
        assert_eq!(index.opened, files);
        assert_eq!(
            in_walks.looks, files,
            "each file read once: {:?}",
            in_walks.at
        );
        let mut sorted = in_walks.at.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), files, "no file read twice");

        let mut in_one = Counting {
            looks: 0,
            at: Vec::new(),
        };
        let at_once = assembled(&folder, None, noon(), &mut in_one, things).unwrap();
        assert!(at_once.covered.whole);
        let mut ours: Vec<&str> = index.entries.iter().map(|e| e.below.as_str()).collect();
        ours.sort_unstable();
        let mut theirs: Vec<&str> = at_once.entries.iter().map(|e| e.below.as_str()).collect();
        theirs.sort_unstable();
        assert_eq!(ours, theirs);

        let mut none = Counting {
            looks: 0,
            at: Vec::new(),
        };
        let again = assembled(&folder, Some(&index), noon(), &mut none, 7).unwrap();
        assert_eq!(none.looks, 0, "{:?}", none.at);
        assert_eq!(again.opened, 0);
        assert!(again.covered.whole);
        assert_eq!(files_of(&again), files_of(&index));
        let _ = std::fs::remove_dir_all(&folder);
    }

    /// **What the walks could not read, enter or finish is kept beside the
    /// index**, spelled with `/` whatever the host, the root included.
    #[test]
    fn what_the_walk_could_not_reach_is_kept_beside_the_index() {
        let walked = Gathered {
            things: Vec::new(),
            links: 0,
            could_not_be_named: 2,
            whole: false,
            unread: vec![alo_files::Unread {
                below: PathBuf::from("Private").join("Theirs"),
                why: "permission denied".to_owned(),
            }],
            elsewhere: vec![PathBuf::from("Drive")],
            not_entered: vec![PathBuf::new(), PathBuf::from("Later")],
        };
        let covered = covered_from(&walked, alo_files::MOST_WALKED);
        assert_eq!(
            covered,
            Covered {
                whole: false,
                most: alo_files::MOST_WALKED,
                unread: vec![Unread {
                    below: "Private/Theirs".to_owned(),
                    why: "permission denied".to_owned(),
                }],
                elsewhere: vec!["Drive".to_owned()],
                not_entered: vec![String::new(), "Later".to_owned()],
                unnamed: 2,
            }
        );
        assert!(!covered.is_everything());
    }

    /// A look is turned into a kind and contents: words for text, none for a
    /// kind with no reader, the size for a file over the bound, the reason
    /// for one that could not be read.
    #[test]
    fn a_look_becomes_a_kind_and_contents() {
        assert_eq!(
            looked(Looked::Whole(b"An Invoice, an invoice".to_vec())),
            (
                Kind::Text,
                Contents::Read {
                    words: vec!["an".to_owned(), "invoice".to_owned()]
                }
            )
        );
        assert_eq!(
            looked(Looked::Whole(b"%PDF-1.7 words inside".to_vec())),
            (Kind::Pdf, Contents::NotText)
        );
        assert_eq!(
            looked(Looked::TooBig {
                bytes: 9,
                first: b"hello".to_vec()
            }),
            (Kind::Text, Contents::TooBig { bytes: 9 })
        );
        assert_eq!(
            looked(Looked::NotRead {
                why: "no".to_owned()
            }),
            (
                Kind::Unread,
                Contents::NotRead {
                    why: "no".to_owned()
                }
            )
        );
    }
}
