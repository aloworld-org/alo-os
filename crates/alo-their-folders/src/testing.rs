//! The strings this crate's own tests are written against, and the disk they
//! pretend to write to.
//!
//! Two fixtures, because every file here has the same questions to answer:
//! *what does this do on a machine with no translations*, *what does it do
//! when somebody has translated a folder name* — including translated it into
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

use alo_strings::{Language, Strings, Translation, Vocabulary};

use crate::making::Disk;
use crate::words::{Word, declare_into};

/// Everything this crate says.
fn its_own() -> Vocabulary {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary).unwrap();
    vocabulary
}

/// This crate's own words, with nothing translated: what a machine with no
/// translations makes, which is the English folder names.
pub(crate) fn in_english() -> Strings {
    Strings::of(its_own())
}

/// The same, with some words translated into German and German preferred —
/// German because it is the language the rest of this repository tests
/// translation with, so a translator's file exercised here looks like the one
/// exercised everywhere else.
pub(crate) fn translated(words: &[(Word, &str)]) -> Strings {
    let vocabulary = its_own();
    let mut german = Translation::into_language(german_language());
    for (word, says) in words {
        german = german.says(word.key(), *says);
    }
    let speaking = vocabulary.check(german).unwrap();
    let mut strings = Strings::of(vocabulary);
    strings.speaks(speaking).unwrap();
    strings.prefers(&[german_language()]);
    strings
}

/// German, as `alo-strings` names a language.
pub(crate) fn german_language() -> Language {
    Language::written("de").unwrap()
}

/// Whether `alo-strings` would refuse this translation outright.
///
/// [`translated`] cannot carry a translation the vocabulary rejects — it would
/// panic building the fixture rather than exercising anything — so a rule that
/// `alo-strings` already holds is tested here, at the door it is held at,
/// rather than pretended to be `crate::naming`'s.
pub(crate) fn would_not_translate(word: Word, says: &str) -> bool {
    let vocabulary = its_own();
    let german = Translation::into_language(german_language()).says(word.key(), says);
    vocabulary.check(german).is_err()
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
        }
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
}
