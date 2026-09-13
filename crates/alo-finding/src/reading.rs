//! The one look inside each file, bounded, and handed in so it can be
//! counted.
//!
//! Indexing is incremental — a file unchanged since last time is not read
//! again — and the only honest test of that counts reads. So the read is a
//! trait, [`Reading`], and the assembly in `indexing.rs` takes one: the
//! disk's, [`Disk`], for the crate's public way in, and a counting one in
//! that file's tests. The public [`crate::Index::opened`] is the same count
//! for a test that goes through the public way in.
//!
//! # The bound is `alo-files`'
//!
//! A file larger than [`alo_files::MOST_READ`] — a megabyte, the most a file
//! verb reads — has its kind read from its first bytes and its words left
//! unread, and the entry says so. One bound in the repository for *how much
//! of a file is read* rather than two that could disagree.
//!
//! # A name that has become a link is not read
//!
//! The walk found a file and stepped over every link; between the walk and
//! this look the name could have been replaced by a link out of the folder.
//! [`Disk`] looks at the name again, without following, before it opens
//! anything, and a link is refused as unread. The window between the look and
//! the open is the ordinary one every reader by name has; this crate indexes
//! a person's own folder for that person, and the verb an agent reaches it
//! through is checked against a grant on the way in.

use std::io::Read;
use std::path::Path;

use alo_files::MOST_READ;

use crate::kind::SNIFFED;

/// What a look inside a file found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Looked {
    /// The whole file, within the bound.
    Whole(Vec<u8>),
    /// A file over the bound: how large, and its first bytes for its kind.
    TooBig {
        /// How large it is.
        bytes: u64,
        /// Its first [`SNIFFED`] bytes.
        first: Vec<u8>,
    },
    /// A file the machine would not let this reader open, or a name that is
    /// a link now.
    NotRead {
        /// What the machine said, as a sentence.
        why: String,
    },
}

/// Whatever can look inside a file.
pub(crate) trait Reading {
    /// Look inside the file at this path.
    fn look(&mut self, at: &Path) -> Looked;
}

/// The disk.
pub(crate) struct Disk;

impl Reading for Disk {
    fn look(&mut self, at: &Path) -> Looked {
        match std::fs::symlink_metadata(at) {
            Ok(about) if about.file_type().is_symlink() => {
                return Looked::NotRead {
                    why: "it is a link now, and a link is never followed".to_owned(),
                };
            }
            Ok(_) => {}
            Err(why) => {
                return Looked::NotRead {
                    why: why.to_string(),
                };
            }
        }
        let opened = match std::fs::File::open(at) {
            Ok(opened) => opened,
            Err(why) => {
                return Looked::NotRead {
                    why: why.to_string(),
                };
            }
        };
        // One byte past the bound, so that a file which is exactly at it is
        // read whole and one over it is known to be over it.
        let mut held = Vec::new();
        if let Err(why) = (&opened).take(MOST_READ + 1).read_to_end(&mut held) {
            return Looked::NotRead {
                why: why.to_string(),
            };
        }
        if held.len() as u64 > MOST_READ {
            let bytes = opened
                .metadata()
                .map_or(held.len() as u64, |about| about.len());
            held.truncate(SNIFFED);
            return Looked::TooBig { bytes, first: held };
        }
        Looked::Whole(held)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A folder of this test's own, under this machine's temporary directory.
    fn a_folder_of_our_own(what: &str) -> std::path::PathBuf {
        let folder =
            std::env::temp_dir().join(format!("alo-finding-reading-{}-{what}", std::process::id()));
        let _ = std::fs::remove_dir_all(&folder);
        std::fs::create_dir_all(&folder).unwrap();
        folder
    }

    /// A file within the bound is read whole; one over it is its size and
    /// its first bytes; one that is not there says what the machine said.
    #[test]
    fn a_look_is_whole_or_too_big_or_not_read() {
        let folder = a_folder_of_our_own("bound");
        let small = folder.join("small.txt");
        std::fs::write(&small, b"a contract").unwrap();
        assert_eq!(Disk.look(&small), Looked::Whole(b"a contract".to_vec()));

        let exactly = folder.join("exactly.txt");
        std::fs::write(&exactly, vec![b'x'; MOST_READ as usize]).unwrap();
        assert!(
            matches!(Disk.look(&exactly), Looked::Whole(held) if held.len() as u64 == MOST_READ)
        );

        let big = folder.join("big.bin");
        let mut over = b"%PDF-1.7 ".to_vec();
        over.resize(MOST_READ as usize + 1, b'x');
        std::fs::write(&big, &over).unwrap();
        match Disk.look(&big) {
            Looked::TooBig { bytes, first } => {
                assert_eq!(bytes, MOST_READ + 1);
                assert_eq!(first.len(), SNIFFED);
                assert!(first.starts_with(b"%PDF-"));
            }
            other => panic!("{other:?}"),
        }

        assert!(matches!(
            Disk.look(&folder.join("gone.txt")),
            Looked::NotRead { .. }
        ));
        let _ = std::fs::remove_dir_all(&folder);
    }

    /// **A name that is a link is not read**, even when what it points at
    /// could be.
    #[cfg(unix)]
    #[test]
    fn a_name_that_is_a_link_is_not_read() {
        let folder = a_folder_of_our_own("link");
        let elsewhere = folder.join("elsewhere.txt");
        std::fs::write(&elsewhere, b"not for the index").unwrap();
        let link = folder.join("looks-like-a-file.txt");
        std::os::unix::fs::symlink(&elsewhere, &link).unwrap();
        match Disk.look(&link) {
            Looked::NotRead { why } => assert!(why.contains("link"), "{why}"),
            other => panic!("{other:?}"),
        }
        let _ = std::fs::remove_dir_all(&folder);
    }
}
