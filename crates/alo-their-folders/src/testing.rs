//! The catalogues this crate's own tests are written against, and the disk
//! they pretend to write to.
//!
//! Two fixtures, because every file here has the same questions to answer:
//! *what does this do in a language with no catalogue*, *what does it do
//! when a catalogue names a folder* — including translated it into
//! something that cannot be a folder, which is the case this crate exists to
//! refuse rather than obey — and *what does it do when the disk says no*.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.
//!
//! # The disk that says no is the point of [`ADisk`]
//!
//! A real filesystem is very hard to make refuse one directory and allow the
//! next, and refusing is the behaviour that matters: a person whose disk is
//! full should still get the folders that fit. So the refusals are tested
//! here, against a disk that is a pair of maps, and `crate::on_this_disk` is
//! tested against a real one for the things only a real one can answer.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::folders::Folder;
use crate::making::Disk;
use alo_gettext::{Catalogue, written};

/// **An `xdg-user-dirs` catalogue** naming these folders — German in name,
/// because it is the language the rest of this repository tests with —
/// written as GNU `msgfmt` writes one and read back the way the machine's is.
pub(crate) fn in_german(names: &[(Folder, &str)]) -> Catalogue {
    let pairs: Vec<(&str, &str)> = names
        .iter()
        .map(|(folder, name)| (folder.xdg_name(), *name))
        .collect();
    Catalogue::read(&written(&pairs)).unwrap()
}

/// A disk written down rather than written to: what was asked of it, and what
/// it was told to refuse.
pub(crate) struct ADisk {
    /// Every directory it was asked to make and did.
    made: RefCell<BTreeSet<PathBuf>>,
    /// Every file it was asked to write, and what went in it.
    written: RefCell<BTreeMap<PathBuf, String>>,
    /// Directories it refuses by name.
    refusing: BTreeSet<PathBuf>,
    /// Whether it refuses every directory.
    refuses_everything: bool,
    /// Whether it refuses to write the record.
    refuses_to_write: bool,
    /// Files already there, and what is in them.
    holding: BTreeMap<PathBuf, String>,
    /// Files that are there and will not be read.
    unreadable: BTreeSet<PathBuf>,
}

impl ADisk {
    /// An ordinary disk with room on it.
    pub(crate) fn that_works() -> Self {
        Self {
            made: RefCell::new(BTreeSet::new()),
            written: RefCell::new(BTreeMap::new()),
            refusing: BTreeSet::new(),
            refuses_everything: false,
            refuses_to_write: false,
            holding: BTreeMap::new(),
            unreadable: BTreeSet::new(),
        }
    }

    /// The same disk, with this file already on it.
    pub(crate) fn holding(mut self, at: &Path, contents: &str) -> Self {
        self.holding.insert(at.to_owned(), contents.to_owned());
        self
    }

    /// The same disk, with a file here it will not read.
    pub(crate) fn unreadable(mut self, at: &Path) -> Self {
        self.unreadable.insert(at.to_owned());
        self
    }

    /// A disk that will not make any directory at all — a full one, or a home
    /// folder that belongs to somebody else.
    pub(crate) fn that_refuses_everything() -> Self {
        Self {
            refuses_everything: true,
            ..Self::that_works()
        }
    }

    /// The same disk, refusing this one directory.
    pub(crate) fn refusing(mut self, at: &Path) -> Self {
        self.refusing.insert(at.to_owned());
        self
    }

    /// The same disk, making directories and refusing to write a file.
    pub(crate) fn refusing_to_write(mut self) -> Self {
        self.refuses_to_write = true;
        self
    }

    /// Whether it was asked for this directory and made it.
    pub(crate) fn made(&self, at: &Path) -> bool {
        self.made.borrow().contains(at)
    }

    /// What went into this file, if anything did.
    pub(crate) fn written(&self, at: &Path) -> Option<String> {
        self.written.borrow().get(at).cloned()
    }
}

impl Disk for ADisk {
    fn make_folder(&self, at: &Path) -> Result<(), String> {
        if self.refuses_everything || self.refusing.contains(at) {
            return Err(format!("no room for {}", at.display()));
        }
        self.made.borrow_mut().insert(at.to_owned());
        Ok(())
    }

    fn write(&self, at: &Path, contents: &str) -> Result<(), String> {
        if self.refuses_to_write {
            return Err(format!("would not write {}", at.display()));
        }
        self.written
            .borrow_mut()
            .insert(at.to_owned(), contents.to_owned());
        Ok(())
    }

    fn read(&self, at: &Path) -> Result<Option<String>, String> {
        if self.unreadable.contains(at) {
            return Err(format!("would not read {}", at.display()));
        }
        Ok(self.holding.get(at).cloned())
    }
}
