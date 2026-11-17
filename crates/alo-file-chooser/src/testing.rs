//! A disk written down in a test.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::listing::{Folders, Kind, Listing, NotListed};

/// Folders by path, each with its folders and files; anything else is not
/// there, and a path in `unreadable` would not be read.
#[derive(Debug, Default)]
pub(crate) struct WrittenDown {
    /// Each folder's folders and files.
    folders: BTreeMap<PathBuf, (Vec<String>, Vec<String>)>,
    /// Folders the machine would not open.
    unreadable: Vec<PathBuf>,
}

impl WrittenDown {
    /// `at` holds these folders and files.
    pub(crate) fn with(mut self, at: &str, folders: &[&str], files: &[&str]) -> Self {
        let names = |these: &[&str]| these.iter().map(|one| (*one).to_owned()).collect();
        self.folders
            .insert(PathBuf::from(at), (names(folders), names(files)));
        self
    }

    /// `at` would not be read.
    pub(crate) fn unreadable(mut self, at: &str) -> Self {
        self.unreadable.push(PathBuf::from(at));
        self
    }
}

impl Folders for WrittenDown {
    fn listed(&self, folder: &Path) -> Result<Listing, NotListed> {
        if self.unreadable.iter().any(|one| one == folder) {
            return Err(NotListed::WouldNotBeRead);
        }
        self.folders
            .get(folder)
            .map(|(folders, files)| Listing::of(folders.clone(), files.clone()))
            .ok_or(NotListed::NotThere)
    }

    fn holds(&self, folder: &Path, name: &str) -> Option<Kind> {
        let (folders, files) = self.folders.get(folder)?;
        if folders.iter().any(|one| one == name) {
            Some(Kind::Folder)
        } else {
            files.iter().any(|one| one == name).then_some(Kind::File)
        }
    }
}
