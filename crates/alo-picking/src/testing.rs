//! The strings this crate's own tests are written against, and the filesystem
//! they walk.
//!
//! Two fixtures, both here rather than beside the file that happens to need
//! them first: every file in this crate that says something has the same two
//! questions to answer — *what does this say on a machine with no
//! translations* and *what does it say when somebody has translated it* — and
//! every file that navigates needs a folder tree that exists on every platform
//! the tests run on.
//!
//! Nothing here is compiled into the crate: it exists under `cfg(test)` only.
//!
//! # The vocabulary is this crate's and `alo-capability`'s
//!
//! A pick becomes a grant, and a grant that cannot be made is refused in
//! `alo-capability`'s own words. A fixture holding only this crate's list would
//! answer those with a key in guillemets and the tests would still pass, which
//! is a fixture proving the tests rather than the code. So both lists are
//! declared, in the order `alo-saying` collects them.

#![expect(
    clippy::unwrap_used,
    reason = "in a fixture, a panic on an unexpected None or Err is the failure being reported"
)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use alo_strings::{Language, Strings, Translation, Vocabulary};

use crate::folders::{Folders, Inside, NotShown};
use crate::words::{Word, declare_into};

/// Everything this crate says, and everything it puts beside what it says.
fn its_own_and_what_it_quotes() -> Vocabulary {
    let mut vocabulary = Vocabulary::empty();
    declare_into(&mut vocabulary).unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    vocabulary
}

/// This crate's own words, with nothing translated: what a machine that has no
/// translations of them shows, which is what most of these tests are about.
pub(crate) fn in_english() -> Strings {
    Strings::of(its_own_and_what_it_quotes())
}

/// The same, with some of these words translated into German and German
/// preferred — German because it is the language the rest of this repository
/// tests translation with, so a translator's file exercised here looks like the
/// one exercised everywhere else.
pub(crate) fn translated(words: &[(Word, &str)]) -> Strings {
    let vocabulary = its_own_and_what_it_quotes();
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

/// A filesystem written down rather than created: which folders hold which
/// folders, and which of them the machine refuses.
///
/// The whole of what this crate learns about a disk arrives through
/// [`Folders`], so a tree in a `BTreeMap` exercises every rule
/// [`crate::Picker`] holds — on every platform, without a privilege, and
/// including the two states a real disk will not produce on demand: a folder
/// the machine will not open, and one that goes away between two questions.
#[derive(Debug, Default)]
pub(crate) struct WrittenDown {
    /// Folder to the folders inside it. A path absent from here is not there.
    holds: BTreeMap<PathBuf, Vec<String>>,
    /// The folders this machine refuses to open.
    refuses: BTreeMap<PathBuf, NotShown>,
}

impl WrittenDown {
    /// Nothing at all: a machine with not one folder on it.
    pub(crate) fn nothing() -> Self {
        Self::default()
    }

    /// This folder holds these folders.
    pub(crate) fn holding(mut self, folder: &str, names: &[&str]) -> Self {
        self.holds.insert(
            PathBuf::from(folder),
            names.iter().map(|name| (*name).to_owned()).collect(),
        );
        self
    }

    /// This folder is there and the machine answers about it like this.
    pub(crate) fn refusing(mut self, folder: &str, why: NotShown) -> Self {
        self.refuses.insert(PathBuf::from(folder), why);
        self
    }

    /// This folder is gone, as of now: the state a picker meets when somebody
    /// deletes a folder while it is being looked at.
    pub(crate) fn took_away(&mut self, folder: &str) {
        self.holds.remove(Path::new(folder));
        self.refuses.remove(Path::new(folder));
    }

    /// The ordinary tree these tests walk: a home folder with three folders
    /// in it, one of which holds another.
    pub(crate) fn a_home_folder() -> Self {
        Self::nothing()
            .holding("/", &["home", "etc"])
            .holding("/home", &["anna"])
            .holding("/home/anna", &["Invoices", "Photos", "Work"])
            .holding("/home/anna/Invoices", &["2026"])
            .holding("/home/anna/Invoices/2026", &[])
            .holding("/home/anna/Photos", &[])
            .holding("/home/anna/Work", &[])
    }
}

impl Folders for WrittenDown {
    fn inside(&self, folder: &Path) -> Result<Inside, NotShown> {
        if let Some(why) = self.refuses.get(folder) {
            return Err(*why);
        }
        match self.holds.get(folder) {
            Some(names) => Ok(Inside::these(names.clone())),
            None => Err(NotShown::WentAway),
        }
    }
}
