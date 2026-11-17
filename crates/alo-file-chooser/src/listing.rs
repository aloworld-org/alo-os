//! What is inside a folder, as the file chooser shows it.
//!
//! [`Folders`] is the one road to a disk, so every rule the chooser holds is
//! tested on a folder written down in a test as well as on a real one
//! (`crate::on_this_disk`).
//!
//! # Names, never paths
//!
//! A [`Listing`] carries **names** — what each thing is called in the folder
//! listed — and the chooser joins a name onto where it stands. A name that is
//! not exactly one ordinary name (`alo_acting::is_a_name`) is counted and not
//! shown, so `..` and `/etc` are never rows.
//!
//! # Folders first, then files, and no hidden names
//!
//! Folders come before files, each in the order of their names with case set
//! aside, so *Documents* and *downloads* sit together as a person reads them. A
//! name beginning with a dot is **hidden**, as every Linux desktop hides it: it
//! is counted, so a surface can say something is there, and not shown. A person
//! can still save under such a name by typing it.

use std::path::Path;

/// The most things one listing shows, folders and files together.
///
/// Far past what a person reads and short of what a folder can hold: a sheet
/// that tried to show a million rows would be one nobody can use, built by a
/// machine nobody can interrupt. What is left out is said
/// ([`Listing::there_are_more`]).
pub const MOST_SHOWN: usize = 1000;

/// What a row is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// The folder this one is in: going there is going up.
    Above,
    /// A folder in this one.
    Folder,
    /// A file in this one.
    File,
}

/// What is inside one folder.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Listing {
    /// The folders in it, in order.
    folders: Vec<String>,
    /// The files in it, in order.
    files: Vec<String>,
    /// How many were hidden or could not be shown.
    not_shown: usize,
    /// Whether more than [`MOST_SHOWN`] were there.
    there_are_more: bool,
}

impl Listing {
    /// The listing of a folder holding these folders and these files, every
    /// rule above applied.
    #[must_use]
    pub fn of(folders: Vec<String>, files: Vec<String>) -> Self {
        let mut not_shown = 0_usize;
        let mut kept = |names: Vec<String>| {
            let mut kept: Vec<String> = names
                .into_iter()
                .filter(|name| {
                    let shown = alo_acting::is_a_name(name) && !name.starts_with('.');
                    if !shown {
                        not_shown = not_shown.saturating_add(1);
                    }
                    shown
                })
                .collect();
            kept.sort_by(|one, other| {
                one.to_lowercase()
                    .cmp(&other.to_lowercase())
                    .then_with(|| one.cmp(other))
            });
            kept.dedup();
            kept
        };
        let mut folders = kept(folders);
        let mut files = kept(files);
        let there_are_more = folders.len() + files.len() > MOST_SHOWN;
        folders.truncate(MOST_SHOWN);
        files.truncate(MOST_SHOWN - folders.len());
        Self {
            folders,
            files,
            not_shown,
            there_are_more,
        }
    }

    /// The same, and this many more could not be named at all.
    #[must_use]
    pub const fn and_could_not_be_named(mut self, how_many: usize) -> Self {
        self.not_shown = self.not_shown.saturating_add(how_many);
        self
    }

    /// The folders in it, in order.
    #[must_use]
    pub fn folders(&self) -> &[String] {
        &self.folders
    }

    /// The files in it, in order.
    #[must_use]
    pub fn files(&self) -> &[String] {
        &self.files
    }

    /// How many things in it are hidden or could not be shown.
    #[must_use]
    pub const fn not_shown(&self) -> usize {
        self.not_shown
    }

    /// Whether more were there than [`MOST_SHOWN`].
    #[must_use]
    pub const fn there_are_more(&self) -> bool {
        self.there_are_more
    }
}

/// Why a folder could not be listed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotListed {
    /// It is not there any more, or it is not a folder.
    NotThere,
    /// The machine would not open it for this person.
    WouldNotBeRead,
}

/// The one road to a disk.
pub trait Folders {
    /// What is inside `folder`.
    ///
    /// # Errors
    /// [`NotListed`] when it is not a folder this person can read.
    fn listed(&self, folder: &Path) -> Result<Listing, NotListed>;

    /// Whether `folder` holds something called `name` now, and what — asked
    /// of the disk itself rather than of a [`Listing`], because a listing does
    /// not show a hidden name or one past [`MOST_SHOWN`], and a name the sheet
    /// cannot see must still not be replaced without the person being told.
    /// [`Kind::Folder`] or [`Kind::File`], never [`Kind::Above`].
    fn holds(&self, folder: &Path, name: &str) -> Option<Kind>;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(these: &[&str]) -> Vec<String> {
        these.iter().map(|one| (*one).to_owned()).collect()
    }

    /// **Folders first, then files, case set aside, and hidden names
    /// counted rather than shown.**
    #[test]
    fn folders_come_first_and_hidden_names_are_counted() {
        let listing = Listing::of(
            names(&["music", "Documents", ".config", "downloads"]),
            names(&["b.txt", ".bashrc", "A.txt", "..", "a/b"]),
        );
        assert_eq!(
            listing.folders(),
            names(&["Documents", "downloads", "music"])
        );
        assert_eq!(listing.files(), names(&["A.txt", "b.txt"]));
        assert_eq!(listing.not_shown(), 4);
        assert!(!listing.there_are_more());
    }

    /// **No more than [`MOST_SHOWN`] are shown, and that more were there is
    /// said.**
    #[test]
    fn a_great_many_are_shown_up_to_the_most_and_said() {
        let folders: Vec<String> = (0..600).map(|n| format!("f{n:04}")).collect();
        let files: Vec<String> = (0..600).map(|n| format!("g{n:04}")).collect();
        let listing = Listing::of(folders, files);
        assert_eq!(listing.folders().len(), 600);
        assert_eq!(listing.files().len(), MOST_SHOWN - 600);
        assert!(listing.there_are_more());
    }
}
