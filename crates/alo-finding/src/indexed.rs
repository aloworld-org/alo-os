//! Which folders are indexed, and the index for a folder found by its name.
//!
//! An [`crate::Index`] is one folder's; this is the list of the folders a
//! person asked to have one, kept beside the indexes under
//! `$XDG_DATA_HOME/alo/finding/` in the file `docs/contracts/file-index.md`
//! describes. A daemon carrying `search_files` and a file manager offering
//! a search box need the same answer — *is this folder indexed, and where is
//! its index?* — and this is the one place that answers it, so that neither
//! has to remember the folders it indexed and work out where each is kept.
//!
//! # The list is the authority, and the disk is never walked to answer
//!
//! [`Indexed::index_of`] answers from the list and the index file, and from
//! nothing else: a folder not on the list is [`crate::NotIndexed::NeverIndexed`]
//! whether or not it exists, and a folder on the list is read back from its
//! file. Nothing here walks a folder, reads a file inside one, or lists the
//! directory the indexes are in — the list says what is indexed, and a file
//! in that directory the list does not name is nothing's.
//!
//! # Written whole or not at all, like an index
//!
//! The list goes down the way an index does, through `keeping.rs`: a sibling
//! written, synced and renamed over the real file. [`Indexed::keep`] writes
//! the index first and the list second, so a list never names a folder whose
//! index was not written; [`Indexed::forget`] removes the index file first
//! and the list second, so a person who asked for a folder to be forgotten
//! has its words gone from the disk before anything else — an index file the
//! list still names and that is not there is read as *could not be read*,
//! and the folder is simply indexed again or forgotten again.
//!
//! # The list is not a grant
//!
//! A folder being on the list says nothing about whether an agent may search
//! it. A grant is `alo-capability`'s, asked at the door in `searched.rs`;
//! this file never names one, and
//! `tests/which_folders_are_indexed_and_the_index_for_one.rs` indexes a
//! folder no grant covers and shows the verb still refused.
//!
//! # Nothing here reads the environment
//!
//! `$XDG_DATA_HOME` and `$HOME` are passed in, as [`crate::Index::where_kept`]
//! takes them, for the reasons `place.rs` gives.

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::index::Index;
use crate::keeping;
use crate::listed;
use crate::place;
use crate::refusing::NotIndexed;

/// The folders a person asked to have indexed, and the directory their
/// indexes are in.
///
/// Read from the disk by [`Self::read_from`]; changed by [`Self::keep`] and
/// [`Self::forget`], each of which writes the list whole before returning.
/// Two of these read at different moments can disagree the way two readings
/// of any file can; whoever holds one across a change reads it again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Indexed {
    /// The directory the list and the indexes are in.
    under: PathBuf,
    /// The folders, in the order they were asked for.
    folders: Vec<PathBuf>,
}

impl Indexed {
    /// The list as it is on the disk, given what the session says.
    ///
    /// `data_home` is `$XDG_DATA_HOME` and `home` is `$HOME`, each as the
    /// process really has it, unset arriving as [`None`]. A list that is not
    /// there yet is an empty one: nothing has been asked for, and the first
    /// [`Self::keep`] writes it.
    ///
    /// # Errors
    ///
    /// [`NotIndexed::NowhereToKeepIt`] for a session with no home directory,
    /// [`NotIndexed::ListNotRead`] when the file is there and could not be
    /// read, and [`NotIndexed::NotAList`] when what was read is not a list
    /// this version reads.
    pub fn read_from(data_home: Option<&OsStr>, home: Option<&OsStr>) -> Result<Self, NotIndexed> {
        let under = place::the_directory(data_home, home).ok_or(NotIndexed::NowhereToKeepIt)?;
        let at = place::list_under(&under);
        let folders = match std::fs::read_to_string(&at) {
            Ok(text) => listed::read(&text).map_err(|why| NotIndexed::NotAList {
                at: at.clone(),
                why,
            })?,
            Err(why) if why.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(why) => {
                return Err(NotIndexed::ListNotRead {
                    at,
                    why: why.to_string(),
                });
            }
        };
        Ok(Self { under, folders })
    }

    /// Where the list is kept, given what the session says.
    ///
    /// # Errors
    ///
    /// [`NotIndexed::NowhereToKeepIt`] for a session with no home directory.
    pub fn where_kept(
        data_home: Option<&OsStr>,
        home: Option<&OsStr>,
    ) -> Result<PathBuf, NotIndexed> {
        place::the_directory(data_home, home)
            .map(|under| place::list_under(&under))
            .ok_or(NotIndexed::NowhereToKeepIt)
    }

    /// The folders a person asked to have indexed, in the order they asked.
    #[must_use]
    pub fn folders(&self) -> &[PathBuf] {
        &self.folders
    }

    /// Whether this folder is one of them.
    #[must_use]
    pub fn holds(&self, folder: &Path) -> bool {
        self.folders.iter().any(|listed| listed == folder)
    }

    /// Where this folder's index is kept, whether or not it is on the list.
    #[must_use]
    pub fn where_index_of(&self, folder: &Path) -> PathBuf {
        place::index_under(&self.under, folder)
    }

    /// The index of this folder, read from the disk.
    ///
    /// From the list and the index file alone: the folder itself is never
    /// walked, whether or not it is there.
    ///
    /// # Errors
    ///
    /// [`NotIndexed::NotAbsolute`] for a folder not named from the root,
    /// [`NotIndexed::NeverIndexed`] for a folder that is not on the list, and
    /// whatever [`Index::read_from`] says about the file of one that is.
    pub fn index_of(&self, folder: &Path) -> Result<Index, NotIndexed> {
        self.listed(folder)?;
        Index::read_from(&self.where_index_of(folder), folder)
    }

    /// Keep this index: written to its file, and its folder put on the list
    /// if it was not there already.
    ///
    /// The index first and the list second, so the list never names a folder
    /// whose index was not written. A folder already on the list has its
    /// index replaced and the list left as it was.
    ///
    /// # Errors
    ///
    /// [`NotIndexed::NotAbsolute`] for an index of a folder not named from
    /// the root, [`NotIndexed::NotKept`] when the index could not be written,
    /// and [`NotIndexed::ListNotKept`] when it was and the list could not be;
    /// in every one the list on the disk and in hand are as they were.
    pub fn keep(&mut self, index: &Index) -> Result<(), NotIndexed> {
        if !index.of.has_root() {
            return Err(NotIndexed::NotAbsolute {
                at: index.of.clone(),
            });
        }
        index.kept_at(&self.where_index_of(&index.of))?;
        if self.holds(&index.of) {
            return Ok(());
        }
        let mut folders = self.folders.clone();
        folders.push(index.of.clone());
        self.written(&folders)?;
        self.folders = folders;
        Ok(())
    }

    /// Forget this folder: its index file removed, and the folder taken off
    /// the list.
    ///
    /// The file first and the list second, so the words of a folder a person
    /// asked to have forgotten are off the disk before anything else. An
    /// index file that is already gone is not a refusal.
    ///
    /// # Errors
    ///
    /// [`NotIndexed::NotAbsolute`] for a folder not named from the root,
    /// [`NotIndexed::NeverIndexed`] for one not on the list,
    /// [`NotIndexed::NotRemoved`] when its index file could not be removed,
    /// and [`NotIndexed::ListNotKept`] when it was and the list could not be
    /// written — in which case the list still names the folder, and
    /// [`Self::index_of`] says its file could not be read.
    pub fn forget(&mut self, folder: &Path) -> Result<(), NotIndexed> {
        self.listed(folder)?;
        let at = self.where_index_of(folder);
        if let Err(why) = std::fs::remove_file(&at)
            && why.kind() != std::io::ErrorKind::NotFound
        {
            return Err(NotIndexed::NotRemoved {
                at,
                why: why.to_string(),
            });
        }
        let folders: Vec<PathBuf> = self
            .folders
            .iter()
            .filter(|listed| listed.as_path() != folder)
            .cloned()
            .collect();
        self.written(&folders)?;
        self.folders = folders;
        Ok(())
    }

    /// That this folder is named from the root and is on the list.
    fn listed(&self, folder: &Path) -> Result<(), NotIndexed> {
        if !folder.has_root() {
            return Err(NotIndexed::NotAbsolute {
                at: folder.to_path_buf(),
            });
        }
        if !self.holds(folder) {
            return Err(NotIndexed::NeverIndexed {
                at: folder.to_path_buf(),
            });
        }
        Ok(())
    }

    /// These folders, written whole as the list on the disk.
    fn written(&self, folders: &[PathBuf]) -> Result<(), NotIndexed> {
        let at = place::list_under(&self.under);
        let text = listed::written(folders).map_err(|why| NotIndexed::ListNotKept {
            at: at.clone(),
            why: why.to_string(),
        })?;
        keeping::kept(&at, &text).map_err(|why| NotIndexed::ListNotKept {
            at,
            why: why.to_string(),
        })
    }
}
