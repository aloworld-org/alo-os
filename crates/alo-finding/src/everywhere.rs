//! One search over every indexed folder: one answer per folder, in the
//! list's order, each saying which folder it is of and how old it is.
//!
//! A person's search box is not a folder's. The list says which folders are
//! indexed; this puts one [`Query`] to every one of them in one call, and
//! hands back one [`OfFolder`] each — an answer held apart from its index,
//! or the refusal that stood where the answer would be. A file manager that
//! stitched the answers together itself would be a search whose honesty
//! depended on the file manager: one that skipped a folder whose index file
//! would not read, and said nothing, has answered *nothing matched* about a
//! folder it never looked at. Here that folder is a named refusal
//! **beside** the other answers, and the search over the rest goes on.
//!
//! # From the index files alone, and the query checked first
//!
//! Each folder's index is read from its file, the way [`crate::Indexed::index_of`]
//! reads it, and never by walking the folder. The query is checked once,
//! before the first index file is opened: a query that is not one is refused
//! with a [`NotAsked`] and nothing on the disk is touched, which a test holds
//! to the kernel's own count of the thread's reads.
//!
//! # Nothing ranks
//!
//! The order is the list's, which is the order the person asked for the
//! folders in; within a folder the order is the index's own, which is the
//! walk's. Nothing the person did not ask for moves a folder or an entry up
//! or down, and there is no score.
//!
//! # Not a verb
//!
//! An agent's `search_files` still names one granted folder and takes one
//! index through `searched.rs`: a search across every indexed folder under
//! one grant would be a search of folders nobody granted. This is the
//! person's search, from the file manager's box, and it takes no grant
//! because the person is not asking anybody.
//!
//! # From the disk every time, or from hand
//!
//! [`crate::Indexed::answer`] reads every index file on every query, and
//! [`crate::InHand::answer`] answers from indexes read once and held; both
//! put each folder's index, or the refusal standing in for it, through
//! `of_folder` below, so that an answer from hand is the answer from the disk
//! in every respect but where the index came from.

use std::path::{Path, PathBuf};

use crate::asking::{self, NotAsked};
use crate::held::Held;
use crate::index::Index;
use crate::indexed::Indexed;
use crate::query::Query;
use crate::refusing::NotIndexed;
use crate::searching;

/// One query's answers from every indexed folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Everywhere {
    /// One per folder on the list, in the list's order.
    pub answers: Vec<OfFolder>,
}

/// What one folder answered — or why it could not.
///
/// `PartialEq` compares the answer's timing too: two searches timed
/// separately are two measurements, and a test comparing what was found
/// compares that.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfFolder {
    /// The folder, as it is on the list.
    pub folder: PathBuf,
    /// The answer, held apart from the index it came from, or the refusal
    /// standing where it would be: the index file could not be read, is not
    /// an index, or is an index of another folder.
    pub answered: Result<Held, NotIndexed>,
}

impl Everywhere {
    /// Every folder that answered, with its answer, in the list's order.
    pub fn answered(&self) -> impl Iterator<Item = (&PathBuf, &Held)> {
        self.answers
            .iter()
            .filter_map(|of| of.answered.as_ref().ok().map(|held| (&of.folder, held)))
    }

    /// Every folder that could not answer, with why, in the list's order.
    pub fn refused(&self) -> impl Iterator<Item = (&PathBuf, &NotIndexed)> {
        self.answers
            .iter()
            .filter_map(|of| of.answered.as_ref().err().map(|why| (&of.folder, why)))
    }

    /// Whether every folder on the list answered.
    #[must_use]
    pub fn every_folder_answered(&self) -> bool {
        self.refused().next().is_none()
    }
}

/// This query, put to every folder on this list.
///
/// # Errors
///
/// [`NotAsked`] for a query that is not one, before any index file is
/// opened.
pub(crate) fn answered(indexed: &Indexed, query: &Query) -> Result<Everywhere, NotAsked> {
    asking::checked(query)?;
    let answers = indexed
        .folders()
        .iter()
        .map(|folder| {
            let read = Index::read_from(&indexed.where_index_of(folder), folder);
            of_folder(folder, read.as_ref(), query)
        })
        .collect();
    Ok(Everywhere { answers })
}

/// This folder's answer to this query — from its index, held apart from it,
/// or the refusal that stands where the index would be.
pub(crate) fn of_folder(
    folder: &Path,
    index: Result<&Index, &NotIndexed>,
    query: &Query,
) -> OfFolder {
    OfFolder {
        folder: folder.to_path_buf(),
        answered: index
            .map(|index| Held::of(&searching::searched(index, query)))
            .map_err(NotIndexed::clone),
    }
}
