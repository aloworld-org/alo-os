//! The index of one folder: made, asked, kept and read back.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use alo_files::MOST_WALKED;

use crate::answer::Answer;
use crate::asking::NotAsked;
use crate::covered::Covered;
use crate::entry::{Entry, Moment};
use crate::format;
use crate::indexing;
use crate::keeping;
use crate::place;
use crate::query::Query;
use crate::reading::Disk;
use crate::refusing::NotIndexed;

/// The index of one folder.
///
/// Everything under the folder, one [`Entry`] each, in the walk's order:
/// each folder before the things inside it, and names in order within a
/// folder. Asking it is [`Self::find`], which never touches the disk.
///
/// # When it was made is the caller's to say
///
/// [`Self::made`] is the moment the caller handed [`Self::of`] or
/// [`Self::again`], never one this crate read from a clock: when an index is
/// made is the file manager's, the daemon's or the person's decision, and a
/// crate that looked at the clock itself would be a step from looking at the
/// disk itself. It is [`None`] only for an index read back from a file written
/// before the moment was kept, which the format allows and the contract says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Index {
    /// The folder this is the index of, as it was named — from the root.
    pub of: PathBuf,
    /// The moment this index was made, as the caller said it, so that an
    /// answer from it can say how old it is; [`None`] for one read back from
    /// a file written before the moment was kept.
    pub made: Option<Moment>,
    /// What the walk under it could not reach.
    pub covered: Covered,
    /// Everything it could.
    pub entries: Vec<Entry>,
    /// How many files were opened to make this index: every file, for a
    /// first index; only the changed ones, for one made again. Zero for an
    /// index read back from its file. About the indexing, not the index, so
    /// it is not written into the file.
    pub opened: usize,
}

impl Index {
    /// This folder, indexed at this moment, every file read.
    ///
    /// `made` is written into the index as the moment it was made, so that
    /// an answer from it can say how old it is. It is an argument rather than
    /// a reading of the clock, for the reasons on [`Self::made`].
    ///
    /// # A folder larger than one walk
    ///
    /// One walk looks at [`alo_files::MOST_WALKED`] things. A folder holding
    /// more is walked on from every folder that walk had found and not
    /// entered — the same walker, under the same bound, from a folder it
    /// named — until nothing is left unentered, and the index is whole. The
    /// one folder that cannot be made whole is one holding more than the
    /// bound at a single level, which no walk can list to its end; it stays
    /// in [`Covered::not_entered`], [`Covered::whole`] is false, and every
    /// answer says it was not searched.
    ///
    /// # Errors
    ///
    /// [`NotIndexed::NotAbsolute`] for a folder not named from the root, and
    /// [`NotIndexed::NotWalked`] for one that is not there, is a file, or
    /// could not be read.
    pub fn of(folder: &Path, made: SystemTime) -> Result<Self, NotIndexed> {
        indexing::assembled(folder, None, made, &mut Disk, MOST_WALKED)
    }

    /// The same folder, indexed again at this moment: walked again — and on,
    /// as [`Self::of`] walks — and a file read only if its size or its time
    /// has changed since this index. An index cut short at one walk's bound
    /// by an earlier version is made whole here, reading only the files it
    /// had not reached.
    ///
    /// # Errors
    ///
    /// As [`Self::of`]. This index is as it was, whatever the answer.
    pub fn again(&self, made: SystemTime) -> Result<Self, NotIndexed> {
        indexing::assembled(&self.of, Some(self), made, &mut Disk, MOST_WALKED)
    }

    /// Everything that answers this query, from the index alone and in the
    /// index's own order.
    ///
    /// The filter and nothing else: [`Self::answer`] is the search a person
    /// or an agent is given, which refuses a query that is not one and says
    /// what it did not look at.
    #[must_use]
    pub fn find(&self, query: &Query) -> Vec<&Entry> {
        self.entries
            .iter()
            .filter(|entry| query.matches(entry))
            .collect()
    }

    /// The answer to this query: what matched, beside what was not searched,
    /// and how long it took — from the index alone, and in the index's own
    /// order.
    ///
    /// # Errors
    ///
    /// [`NotAsked`] for a query that is not one — empty, or longer than a
    /// sentence — refused before anything is searched, because the answer to
    /// no question is not everything.
    pub fn answer(&self, query: &Query) -> Result<Answer<'_>, NotAsked> {
        crate::searching::answered(self, query)
    }

    /// Where this entry is on the disk.
    #[must_use]
    pub fn where_is(&self, entry: &Entry) -> PathBuf {
        let mut at = self.of.clone();
        for part in entry.below.split('/') {
            at.push(part);
        }
        at
    }

    /// This index, written whole to this file — replaced whole or not at all,
    /// readable by its owner alone where the host has modes.
    ///
    /// # Errors
    ///
    /// [`NotIndexed::NotKept`], with what the machine said; the file at `at`
    /// is as it was.
    pub fn kept_at(&self, at: &Path) -> Result<(), NotIndexed> {
        let text = format::written(self).map_err(|why| NotIndexed::NotKept {
            at: at.to_path_buf(),
            why: why.to_string(),
        })?;
        keeping::kept(at, &text).map_err(|why| NotIndexed::NotKept {
            at: at.to_path_buf(),
            why: why.to_string(),
        })
    }

    /// The index of this folder, in this file.
    ///
    /// The folder is asked for as well as the file because the file's name is
    /// a hash of the folder's path and its first line names the folder: a
    /// file that names another folder — copied, or a hash that collided — is
    /// refused rather than searched as if it were this one's.
    ///
    /// # Errors
    ///
    /// [`NotIndexed::NotOpened`] when the file could not be read,
    /// [`NotIndexed::NotAnIndex`] when what was read is not an index this
    /// version reads, and [`NotIndexed::NotTheSame`] when it is an index of
    /// another folder.
    pub fn read_from(at: &Path, of: &Path) -> Result<Self, NotIndexed> {
        let text = std::fs::read_to_string(at).map_err(|why| NotIndexed::NotOpened {
            at: at.to_path_buf(),
            why: why.to_string(),
        })?;
        let index = format::read(&text).map_err(|why| NotIndexed::NotAnIndex {
            at: at.to_path_buf(),
            why,
        })?;
        if index.of != of {
            return Err(NotIndexed::NotTheSame {
                asked: of.to_path_buf(),
                indexed: index.of,
            });
        }
        Ok(index)
    }

    /// Where the index of this folder is kept, given what the session says.
    ///
    /// `data_home` is `$XDG_DATA_HOME` and `home` is `$HOME`, each as the
    /// process really has it, unset arriving as [`None`]. Nothing here reads
    /// the environment; `docs/contracts/file-index.md` says where the answer
    /// is.
    ///
    /// # Errors
    ///
    /// [`NotIndexed::NotAbsolute`] for a folder not named from the root, and
    /// [`NotIndexed::NowhereToKeepIt`] for a session with no home directory.
    pub fn where_kept(
        data_home: Option<&OsStr>,
        home: Option<&OsStr>,
        folder: &Path,
    ) -> Result<PathBuf, NotIndexed> {
        if !folder.has_root() {
            return Err(NotIndexed::NotAbsolute {
                at: folder.to_path_buf(),
            });
        }
        place::where_it_is(data_home, home, folder).ok_or(NotIndexed::NowhereToKeepIt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A relative folder has no place to be kept, and a session with no home
    /// directory has nowhere to keep one.
    #[test]
    fn where_an_index_is_kept_is_refused_for_a_relative_folder_or_no_home() {
        assert!(matches!(
            Index::where_kept(None, Some(OsStr::new("/home/ada")), Path::new("Documents")),
            Err(NotIndexed::NotAbsolute { .. })
        ));
        assert!(matches!(
            Index::where_kept(None, None, Path::new("/home/ada/Documents")),
            Err(NotIndexed::NowhereToKeepIt)
        ));
        assert!(
            Index::where_kept(
                Some(OsStr::new("/data")),
                None,
                Path::new("/home/ada/Documents")
            )
            .is_ok()
        );
    }

    /// An entry's place on the disk is the folder and the parts below it,
    /// joined the way this host joins them.
    #[test]
    fn where_an_entry_is_joins_the_parts_the_way_this_host_does() {
        let index = Index {
            of: PathBuf::from("/home/ada/Documents"),
            made: None,
            covered: Covered {
                whole: true,
                most: 0,
                unread: Vec::new(),
                elsewhere: Vec::new(),
                not_entered: Vec::new(),
                unnamed: 0,
            },
            entries: Vec::new(),
            opened: 0,
        };
        let entry = Entry {
            below: "2026/March/march.pdf".to_owned(),
            kind: crate::Kind::Pdf,
            bytes: 0,
            modified: crate::Moment { secs: 0, nanos: 0 },
            contents: crate::Contents::NotText,
        };
        assert_eq!(
            index.where_is(&entry),
            Path::new("/home/ada/Documents")
                .join("2026")
                .join("March")
                .join("march.pdf")
        );
    }
}
