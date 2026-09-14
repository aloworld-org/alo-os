//! The list's indexes read from their files once and held in hand, to be
//! asked many times.
//!
//! [`crate::Indexed::answer`] reads every index file from the disk on every
//! query, which is the disk's word every time and the right default. A
//! person typing *contract* into a search box fires a query per keystroke,
//! and three folders of ten thousand entries are three files parsed per
//! keystroke; a file manager that decided to keep the indexes between
//! keystrokes would otherwise write its own set of them, its own refresh of
//! one by name, and its own memory of which index file would not read.
//! [`InHand`] is that set, written once here: every folder on the list, its
//! index read from its file at the moment the caller asked for the set, and
//! held — or the refusal that stood where the index would be, held in the
//! same place — answering any number of queries from memory in the shape
//! [`crate::Indexed::answer`] answers in.
//!
//! # The caller's choice, for the caller's lifetime
//!
//! Holding the indexes in hand is the caller's decision, and so is when to
//! read them again: nothing here caches across processes, writes anything
//! new to the disk on its own, or decides when the disk should be asked
//! again — no watcher, no thread, no timer, no clock. A set in hand is the
//! list and the indexes as they were when it was read; a caller who wants
//! the disk's word reads another with [`InHand::of`], or keeps asking
//! [`crate::Indexed::answer`], which is unchanged. A folder whose index
//! file would not read stays refused in hand, with the same refusal, until
//! the caller reads again or brings that folder up to date by its name.
//!
//! # Brought up to date by its name, in hand and on the disk
//!
//! [`InHand::again`] is [`crate::Indexed::again`] on the list this set was
//! read from — the kept index read, the folder indexed again reading only
//! what changed, the result kept whole — with the fresh index put in the
//! set's own place for that folder. Nothing else in the set is read again,
//! which a test holds to the kernel's per-thread read count. A refusal
//! leaves the set as it was, the way it leaves the disk as it was.
//!
//! # Nothing ranks, and this is not a verb
//!
//! The answers are in the list's order and each folder's in its index's
//! own, as they are from the disk. An agent's `search_files` still names
//! one granted folder and takes one index through `searched.rs`; the list
//! is still not a grant.

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::asking::{self, NotAsked};
use crate::everywhere::{self, Everywhere};
use crate::index::Index;
use crate::indexed::Indexed;
use crate::query::Query;
use crate::refusing::NotIndexed;

/// Every index on the list, read from its file once and held.
///
/// Made by [`Self::of`] or [`Indexed::in_hand`]. What it holds is what the
/// disk said at that moment, folder by folder in the list's order: an index,
/// or the refusal that stood where one would be. It goes on saying that
/// until the caller reads again or refreshes a folder by its name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InHand {
    /// The list as it was when this set was read from it.
    indexed: Indexed,
    /// One per folder on the list, in the list's order: the index read from
    /// its file, or why it could not be.
    held: Vec<Result<Index, NotIndexed>>,
}

impl InHand {
    /// The indexes of every folder on this list, read from their files now
    /// and held from here on.
    ///
    /// Each folder's index is read the way [`Indexed::index_of`] reads it —
    /// from its file, and never by walking the folder — and a folder whose
    /// file could not be read, is not an index, or is another folder's is
    /// held as that refusal in its place rather than left out. An empty
    /// list is an empty set.
    #[must_use]
    pub fn of(indexed: &Indexed) -> Self {
        let held = indexed
            .folders()
            .iter()
            .map(|folder| Index::read_from(&indexed.where_index_of(folder), folder))
            .collect();
        Self {
            indexed: indexed.clone(),
            held,
        }
    }

    /// The folders this set holds, in the list's order — the list as it was
    /// when the set was read.
    #[must_use]
    pub fn folders(&self) -> &[PathBuf] {
        self.indexed.folders()
    }

    /// The list this set was read from, as it was then.
    #[must_use]
    pub fn indexed(&self) -> &Indexed {
        &self.indexed
    }

    /// Every folder with what is held for it, in the list's order: its
    /// index, or the refusal standing where it would be.
    pub fn each(&self) -> impl Iterator<Item = (&PathBuf, Result<&Index, &NotIndexed>)> {
        self.folders()
            .iter()
            .zip(&self.held)
            .map(|(folder, held)| (folder, held.as_ref()))
    }

    /// The index of this folder, from hand.
    ///
    /// Nothing on the disk is opened, whether the answer is the index or
    /// the refusal that was held in its place.
    ///
    /// # Errors
    ///
    /// [`NotIndexed::NotAbsolute`] for a folder not named from the root,
    /// [`NotIndexed::NeverIndexed`] for one not on the list this set was
    /// read from, and the refusal held for a folder whose index file could
    /// not be read when the set was.
    pub fn index_of(&self, folder: &Path) -> Result<&Index, NotIndexed> {
        listed(folder)?;
        let held = self
            .each()
            .find(|(listed, _)| listed.as_path() == folder)
            .map(|(_, held)| held)
            .ok_or_else(|| never_indexed(folder))?;
        held.map_err(NotIndexed::clone)
    }

    /// This query, put to every folder in hand: one answer per folder, in
    /// the list's order, in the shape [`Indexed::answer`] answers in — and
    /// from memory, with no index file opened.
    ///
    /// A folder held as a refusal answers with that refusal, beside the
    /// others, every time it is asked, until the caller reads again or
    /// brings it up to date by its name. Nothing ranks.
    ///
    /// # Errors
    ///
    /// [`NotAsked`] for a query that is not one, refused before anything is
    /// searched.
    pub fn answer(&self, query: &Query) -> Result<Everywhere, NotAsked> {
        asking::checked(query)?;
        let answers = self
            .each()
            .map(|(folder, held)| everywhere::of_folder(folder, held, query))
            .collect();
        Ok(Everywhere { answers })
    }

    /// This folder's index brought up to date at this moment, in hand and
    /// on the disk, by its name: [`Indexed::again`] on the list this set was
    /// read from, and the fresh index put in this set's place for the
    /// folder — whether that place held an index or a refusal. No other
    /// folder's index file is read.
    ///
    /// # Errors
    ///
    /// Whatever [`Indexed::again`] refuses with — a folder not named from
    /// the root, one not on the list, one whose kept index file would not
    /// read, one that is gone since it was indexed, or one whose fresh index
    /// could not be written. The set is as it was, as the disk is.
    pub fn again(&mut self, folder: &Path, made: SystemTime) -> Result<&Index, NotIndexed> {
        let fresh = self.indexed.again(folder, made)?;
        let place = self
            .indexed
            .folders()
            .iter()
            .zip(self.held.iter_mut())
            .find(|(listed, _)| listed.as_path() == folder)
            .map(|(_, held)| held)
            .ok_or_else(|| never_indexed(folder))?;
        *place = Ok(fresh);
        place.as_ref().map_err(NotIndexed::clone)
    }
}

/// That this folder is named from the root.
fn listed(folder: &Path) -> Result<(), NotIndexed> {
    if folder.has_root() {
        Ok(())
    } else {
        Err(NotIndexed::NotAbsolute {
            at: folder.to_path_buf(),
        })
    }
}

/// The refusal for a folder that is not on the list this set was read from.
fn never_indexed(folder: &Path) -> NotIndexed {
    NotIndexed::NeverIndexed {
        at: folder.to_path_buf(),
    }
}

impl Indexed {
    /// The indexes of every folder on this list, read from their files once
    /// and held in hand from here on, to be asked many times.
    ///
    /// [`InHand::of`], as a method: the same reads [`Self::answer`] makes on
    /// every query, made once. Holding them is the caller's choice for the
    /// caller's lifetime — nothing here decides when to read again.
    #[must_use]
    pub fn in_hand(&self) -> InHand {
        InHand::of(self)
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A list of these folders, written to a data home of this test's own
    /// with no index file beside it, so that every index is a refusal the
    /// set has to hold in place.
    fn a_list_of(what: &str, folders: &[&str]) -> Indexed {
        let data_home =
            std::env::temp_dir().join(format!("alo-finding-in-hand-{}-{what}", std::process::id()));
        let _ = std::fs::remove_dir_all(&data_home);
        let under = crate::place::the_directory(Some(data_home.as_os_str()), None).unwrap();
        std::fs::create_dir_all(&under).unwrap();
        let folders: Vec<PathBuf> = folders.iter().map(PathBuf::from).collect();
        crate::keeping::kept(
            &crate::place::list_under(&under),
            &crate::listed::written(&folders).unwrap(),
        )
        .unwrap();
        let list = Indexed::read_from(Some(data_home.as_os_str()), None).unwrap();
        assert_eq!(list.folders(), folders.as_slice());
        let _ = std::fs::remove_dir_all(&data_home);
        list
    }

    /// A set read from a list whose index files are not there holds one
    /// refusal per folder, in the list's order, naming each file; asks it
    /// with no file opened; and a folder not on the list is refused as
    /// never indexed without the set changing.
    #[test]
    fn a_set_with_no_files_to_read_holds_a_refusal_in_every_place() {
        let list = a_list_of("two", &["/home/ada/Documents", "/home/ada/Pictures"]);
        let in_hand = list.in_hand();
        assert_eq!(in_hand.folders(), list.folders());
        assert_eq!(in_hand.indexed(), &list);
        for (folder, held) in in_hand.each() {
            let refused = held.unwrap_err();
            assert!(
                matches!(refused, NotIndexed::NotOpened { at, .. } if at == &list.where_index_of(folder)),
                "{refused}"
            );
        }
        let refused = in_hand
            .index_of(Path::new("/home/ada/Pictures"))
            .unwrap_err();
        assert!(matches!(refused, NotIndexed::NotOpened { .. }), "{refused}");
        assert!(matches!(
            in_hand.index_of(Path::new("/home/ada/Music")).unwrap_err(),
            NotIndexed::NeverIndexed { at } if at == Path::new("/home/ada/Music")
        ));
        assert!(matches!(
            in_hand.index_of(Path::new("Documents")).unwrap_err(),
            NotIndexed::NotAbsolute { at } if at == Path::new("Documents")
        ));

        let everywhere = in_hand.answer(&Query::named("march")).unwrap();
        assert_eq!(everywhere.answers.len(), 2);
        assert_eq!(everywhere.refused().count(), 2);
        assert_eq!(
            in_hand.answer(&Query::named("")).unwrap_err(),
            NotAsked::Nothing
        );
    }

    /// An empty list is an empty set, which answers with no folders and no
    /// refusal.
    #[test]
    fn an_empty_list_is_an_empty_set() {
        let in_hand = a_list_of("none", &[]).in_hand();
        assert!(in_hand.folders().is_empty());
        assert_eq!(in_hand.each().count(), 0);
        let everywhere = in_hand.answer(&Query::named("march")).unwrap();
        assert!(everywhere.answers.is_empty());
        assert!(everywhere.every_folder_answered());
    }
}
