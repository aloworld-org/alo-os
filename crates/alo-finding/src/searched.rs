//! The one door from *this may search the index* to *this is what it found*.
//!
//! [`alo_files::Touching`] is the end of everything the capability model can
//! decide about a call that names a folder: validated, permitted, and asked
//! about again — at the moment it would run — where the folder really leads.
//! A [`Searched`] is what happens next for `search_files`, and it is the only
//! thing in this crate that an agent's authority ever reaches.
//!
//! # The index is handed in, and it has to be the folder's
//!
//! Nothing here reads the environment to find where an index is kept, and
//! nothing here walks a folder to make one: whoever holds the indexes hands
//! this the index of the granted folder, and [`Searched::of`] checks that it
//! **is** that folder's — the real folder, as the grants were asked about it
//! — before asking it anything. An index of some other folder answers
//! [`NotAnswered::Indexed`], because an index that answered for a folder it
//! is not of would be a search of somewhere nobody granted.
//!
//! # Two ways of not happening, and they are different facts
//!
//! [`Searched::of`] cannot refuse: a refusal by the grants is
//! `alo_capability::Refused`, made by [`alo_files::Touching::of`] before
//! anything here is reached. What it can do is come back with a
//! [`NotAnswered`] — the query was not one, the index was another folder's —
//! and the authorisation comes back either way, because either way something
//! is written down. A call that was permitted and attempted is a thing that
//! happened, and is recorded as one.
//!
//! # A person needs none of this
//!
//! [`crate::Index::answer`] is the search, and it takes an index and a query.
//! This file is the road an **agent** takes to the same answer — under a
//! grant, recorded — and a person in the file manager takes none of it. The
//! answer is the same because it is the same function.

use alo_capability::{Authorised, Value};
use alo_files::Touching;

use crate::answer::Answer;
use crate::index::Index;
use crate::query::Query;
use crate::refusing::NotIndexed;
use crate::unanswered::NotAnswered;

/// The name the search verb is declared under.
const SEARCH_FILES: &str = "search_files";

/// What happened when a permitted search was put to the index.
///
/// Not `Clone`, like everything else on this journey: a thing that happened
/// is not a thing that can happen again.
#[derive(Debug)]
pub struct Searched<'a> {
    /// What ran, and the authority it ran under.
    authorised: Authorised,
    /// What the index answered, or why it could not.
    outcome: Result<Answer<'a>, NotAnswered>,
}

impl<'a> Searched<'a> {
    /// Put a permitted search to this index.
    ///
    /// The folder the call named has been made real by [`Touching::of`], and
    /// the index has to be of exactly that folder. The query is built here
    /// from the call's one name, and nothing the model sent reaches the index
    /// any other way.
    #[must_use]
    pub fn of(touching: Touching, index: &'a Index) -> Self {
        let outcome = answered(&touching, index);
        Self {
            authorised: touching.into_authorised(),
            outcome,
        }
    }

    /// What ran, and the authority it ran under — what the record is written
    /// from.
    #[must_use]
    pub fn authorised(&self) -> &Authorised {
        &self.authorised
    }

    /// What the index answered, when it did.
    #[must_use]
    pub fn answer(&self) -> Option<&Answer<'a>> {
        self.outcome.as_ref().ok()
    }

    /// Why the index did not answer, when it did not.
    #[must_use]
    pub fn not_answered(&self) -> Option<&NotAnswered> {
        self.outcome.as_ref().err()
    }

    /// The authority and the outcome, taken.
    ///
    /// The authorisation comes back whether or not the index answered,
    /// because a call that was permitted and attempted is a thing that
    /// happened and is recorded as one.
    pub fn into_parts(self) -> (Authorised, Result<Answer<'a>, NotAnswered>) {
        (self.authorised, self.outcome)
    }
}

/// The search, put to the index.
fn answered<'a>(touching: &Touching, index: &'a Index) -> Result<Answer<'a>, NotAnswered> {
    let verb = touching.verb();
    if verb != SEARCH_FILES {
        return Err(NotAnswered::NotThisCrates {
            verb: verb.to_owned(),
        });
    }
    let folder = touching
        .real("folder")
        .ok_or_else(|| missing(verb, "folder"))?;
    let named = match touching.call().value("named") {
        Some(Value::Name(named)) => named,
        _ => return Err(missing(verb, "named")),
    };
    if index.of != folder.as_path() {
        return Err(NotAnswered::Indexed(NotIndexed::NotTheSame {
            asked: folder.as_path().to_path_buf(),
            indexed: index.of.clone(),
        }));
    }
    Ok(index.answer(&Query::named(named))?)
}

/// An argument the verb declares and the call did not carry.
fn missing(verb: &str, argument: &str) -> NotAnswered {
    NotAnswered::Missing {
        verb: verb.to_owned(),
        argument: argument.to_owned(),
    }
}
