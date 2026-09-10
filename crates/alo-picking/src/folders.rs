//! What is inside a folder, as the picker is allowed to learn it.
//!
//! One port, one answer and three ways of having none. [`Folders`] is the only
//! road between this crate and a disk: [`crate::Picker`] never opens anything
//! itself, which is what lets every rule it holds be tested on a filesystem
//! written down in a test, on every platform, rather than only where a
//! developer happens to be able to create the folder a test needs.
//!
//! # Names, and not paths
//!
//! [`Inside`] carries **names** — one folder's name, as it is written in the
//! folder above it — and never a path. The picker joins a name onto where it
//! is standing, so where it stands is always somewhere it walked to from where
//! it started. An implementation that answered with paths could put the picker
//! anywhere on the machine, and the grant that came out would be over a folder
//! nobody navigated to; a name cannot do that, and [`Inside::these`] refuses
//! any name that is not exactly one ordinary component so that `..` and
//! `/etc` are not names either.
//!
//! # What is bounded here, and why it is bounded here rather than in a shell
//!
//! A folder can hold a great many folders, and a picker that tried to show all
//! of them would be a surface nobody can use, made by a machine nobody can
//! interrupt. So [`MOST_SHOWN`] is applied by [`Inside::these`] — in the one
//! place every implementation passes through — rather than by whichever
//! implementation remembered to. What is left over is not hidden:
//! [`Inside::there_are_more`] says so, and `picking.more-than-shown` is the
//! sentence for it.

use std::path::{Component, Path};

/// The most folders one listing shows.
///
/// A thousand is far past what a person reads and far short of what a
/// filesystem can hold. It is this crate's own number rather than
/// `alo-files`': that one bounds what an **agent** may be answered with, and
/// this one bounds what a **person** is shown — the same figure today, and two
/// different questions that must be free to move apart.
pub const MOST_SHOWN: usize = 1000;

/// What is inside one folder: the folders in it, and what was left out.
///
/// Never constructed by a picker and never by a shell: an implementation of
/// [`Folders`] makes one out of what the machine said, and every rule about
/// what may be in it is applied by [`Inside::these`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Inside {
    /// The names of the folders in it, sorted, each exactly one component.
    names: Vec<String>,
    /// How many things in it could not be named at all.
    could_not_be_named: usize,
    /// Whether there were more than [`MOST_SHOWN`] of them.
    there_are_more: bool,
}

impl Inside {
    /// The folders in it, by name.
    ///
    /// Anything that is not exactly one ordinary component of a path is not a
    /// name in a folder: it is counted as something that could not be named
    /// and is dropped, because the alternative is showing a person a row they
    /// cannot open. The rest are sorted, so that two machines listing one
    /// folder show it in the same order.
    #[must_use]
    pub fn these(names: Vec<String>) -> Self {
        let mut could_not_be_named = 0_usize;
        let mut kept: Vec<String> = Vec::with_capacity(names.len());
        for name in names {
            if is_one_name(&name) {
                kept.push(name);
            } else {
                could_not_be_named = could_not_be_named.saturating_add(1);
            }
        }
        kept.sort();
        kept.dedup();
        let there_are_more = kept.len() > MOST_SHOWN;
        kept.truncate(MOST_SHOWN);
        Self {
            names: kept,
            could_not_be_named,
            there_are_more,
        }
    }

    /// The same, and this many things in that folder could not be named or
    /// read at all.
    ///
    /// A machine can hold a name that is not text — bytes no language decodes
    /// — and a directory entry the machine will not describe. Neither can be
    /// shown and neither is silently forgotten: the count is what a surface
    /// can say *something* is here that this list does not show.
    #[must_use]
    pub fn and_could_not_be_named(mut self, how_many: usize) -> Self {
        self.could_not_be_named = self.could_not_be_named.saturating_add(how_many);
        self
    }

    /// The names of the folders in it, sorted.
    #[must_use]
    pub fn names(&self) -> &[String] {
        &self.names
    }

    /// Whether one of these is a folder in it, matched exactly.
    ///
    /// Exactly, with no case folding, for `alo-capability`'s reason: matching
    /// loosely means matching *more* than the person picked, and on this side
    /// of the system every widening is a security bug.
    #[must_use]
    pub fn holds(&self, name: &str) -> bool {
        self.names.iter().any(|held| held == name)
    }

    /// How many things in it could not be named.
    #[must_use]
    pub fn could_not_be_named(&self) -> usize {
        self.could_not_be_named
    }

    /// Whether there were more folders in it than are shown.
    #[must_use]
    pub fn there_are_more(&self) -> bool {
        self.there_are_more
    }
}

/// Whether a name is one folder's name, rather than a path wearing one.
fn is_one_name(name: &str) -> bool {
    let mut components = Path::new(name).components();
    let Some(Component::Normal(only)) = components.next() else {
        return false;
    };
    components.next().is_none() && only == std::ffi::OsStr::new(name)
}

/// Why a folder could not be shown.
///
/// Three facts about the machine, and nothing about the person's request: a
/// picker asking about a folder it is standing in or has just been asked to
/// open cannot ask a malformed question, because [`crate::Picker`] has already
/// refused everything malformed before any implementation is reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotShown {
    /// Something is there and it is not a folder — a file, or a link to
    /// somewhere else.
    NotAFolder,
    /// Nothing is there: it was moved or deleted, possibly since it was listed
    /// a moment ago.
    WentAway,
    /// It is there, and this machine would not let it be opened.
    WouldNotBeRead,
}

/// How the picker learns what is inside a folder.
///
/// Implemented by [`crate::OnThisDisk`] for a real machine, and by a test for
/// a filesystem that is written down rather than created. It is deliberately
/// the *whole* of what a picker may learn: there is no method here for asking
/// whether a path exists, and none for reading a file, because a folder
/// chooser that could answer either would be a way to learn about a disk
/// before anything at all had been granted.
pub trait Folders {
    /// What folders are inside this one.
    ///
    /// # Errors
    /// [`NotShown`], which is a fact about the machine rather than about the
    /// question.
    fn inside(&self, folder: &Path) -> Result<Inside, NotShown>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The ordinary case: names come back sorted, and nothing is missing.
    #[test]
    fn the_folders_in_it_come_back_sorted() {
        let inside = Inside::these(vec![
            "Photos".to_owned(),
            "Invoices".to_owned(),
            "Archive".to_owned(),
        ]);
        assert_eq!(inside.names(), ["Archive", "Invoices", "Photos"]);
        assert_eq!(inside.could_not_be_named(), 0);
        assert!(!inside.there_are_more());
    }

    /// **A name that is a path is not a name.** This is the refusal that keeps
    /// the picker inside the tree it walked: an implementation answering with
    /// `..`, `/etc` or `a/b` cannot move it anywhere, because none of those is
    /// ever offered to a person to open.
    #[test]
    fn a_name_that_is_really_a_path_is_not_a_folder_in_here() {
        let inside = Inside::these(vec![
            "Invoices".to_owned(),
            "..".to_owned(),
            ".".to_owned(),
            "/etc".to_owned(),
            "a/b".to_owned(),
            String::new(),
        ]);
        assert_eq!(inside.names(), ["Invoices"]);
        assert_eq!(inside.could_not_be_named(), 5);
        assert!(!inside.holds(".."));
        assert!(!inside.holds("/etc"));
    }

    /// A name is matched exactly, so a folder that differs only in capital
    /// letters is a different folder.
    #[test]
    fn a_name_is_matched_exactly() {
        let inside = Inside::these(vec!["Invoices".to_owned()]);
        assert!(inside.holds("Invoices"));
        assert!(!inside.holds("invoices"));
        assert!(!inside.holds("Invoices "));
    }

    /// **A folder with more in it than can be shown says so**, rather than
    /// showing a truncated list that reads as the whole of it.
    #[test]
    fn more_folders_than_can_be_shown_are_reported_rather_than_dropped_quietly() {
        let many: Vec<String> = (0..=MOST_SHOWN)
            .map(|which| format!("folder-{which:05}"))
            .collect();
        let inside = Inside::these(many);
        assert_eq!(inside.names().len(), MOST_SHOWN);
        assert!(inside.there_are_more());

        let exactly = Inside::these(
            (0..MOST_SHOWN)
                .map(|which| format!("f{which:05}"))
                .collect(),
        );
        assert_eq!(exactly.names().len(), MOST_SHOWN);
        assert!(!exactly.there_are_more());
    }

    /// Two entries with one name are one row: a list that showed the same
    /// folder twice would be a list a person cannot trust.
    #[test]
    fn one_name_is_one_row() {
        let inside = Inside::these(vec!["Invoices".to_owned(), "Invoices".to_owned()]);
        assert_eq!(inside.names(), ["Invoices"]);
    }

    /// What could not be named is counted from both roads — the names that
    /// were not names, and whatever the machine could not describe at all.
    #[test]
    fn what_could_not_be_named_is_counted_from_both_roads() {
        let inside =
            Inside::these(vec!["Invoices".to_owned(), "..".to_owned()]).and_could_not_be_named(2);
        assert_eq!(inside.could_not_be_named(), 3);
        assert_eq!(inside.names(), ["Invoices"]);
    }

    /// An empty folder is an empty listing rather than a refusal: there is
    /// nothing to open in it, and it is still a folder somebody may pick.
    #[test]
    fn an_empty_folder_is_a_listing_with_nothing_in_it() {
        let inside = Inside::these(Vec::new());
        assert!(inside.names().is_empty());
        assert!(!inside.there_are_more());
        assert_eq!(inside, Inside::default());
    }
}
