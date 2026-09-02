//! What a file verb answers with.
//!
//! Six verbs, six answers, and every one of them is a shape rather than a
//! sentence. The answer goes to a model, which will act on it, and to a person,
//! who will read it; text formatted here would be text both of them had to
//! interpret, and it would hardcode English into the layer furthest from
//! anybody who could translate it.
//!
//! # An answer says what it left out
//!
//! Three of these carry a count or a flag of what is missing —
//! [`Listing::could_not_be_named`], [`Listing::was_cut_short`],
//! [`Search::was_cut_short`], [`Archived::left_out`]. That is the point of
//! them. Every bound in this crate exists so that one folder cannot fill a
//! person's screen or a model's context, and **a bounded answer that does not
//! say it was bounded is a lie** — it reads exactly like a complete one, and
//! the reader would go on to conclude that a file is not there.

use std::path::{Path, PathBuf};

use crate::named::Named;

/// What one of the six answered with.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    /// What is in a folder.
    Listed(Listing),
    /// What is in a file, as text.
    Read(String),
    /// The files a search found.
    Found(Search),
    /// Where the renamed file now is.
    Renamed(PathBuf),
    /// Where the moved file now is.
    Moved(PathBuf),
    /// The archive that was made.
    Archived(Archived),
}

impl Answer {
    /// What is in the folder, when a folder was listed.
    #[must_use]
    pub fn listed(&self) -> Option<&Listing> {
        match self {
            Self::Listed(listing) => Some(listing),
            _ => None,
        }
    }

    /// What is in the file, when a file was read.
    #[must_use]
    pub fn read(&self) -> Option<&str> {
        match self {
            Self::Read(text) => Some(text),
            _ => None,
        }
    }

    /// What was found, when a search was made.
    #[must_use]
    pub fn found(&self) -> Option<&Search> {
        match self {
            Self::Found(search) => Some(search),
            _ => None,
        }
    }

    /// Where the file now is, when it was renamed or moved.
    ///
    /// One question for both, because *where is it now* is one question — which
    /// of the two verbs put it there is [`alo_capability::Authorised::verb`]'s
    /// to answer, and it is answered there for every verb rather than here for
    /// two.
    #[must_use]
    pub fn now_at(&self) -> Option<&Path> {
        match self {
            Self::Renamed(at) | Self::Moved(at) => Some(at),
            _ => None,
        }
    }

    /// The archive, when one was made.
    #[must_use]
    pub fn archived(&self) -> Option<&Archived> {
        match self {
            Self::Archived(archived) => Some(archived),
            _ => None,
        }
    }
}

/// What is in a folder.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Listing {
    /// What is in it, by name, in order.
    things: Vec<Named>,
    /// How many things were left out because their names could not be shown.
    could_not_be_named: usize,
    /// Whether there was more in the folder than one answer carries.
    cut_short: bool,
}

impl Listing {
    /// A listing of these things.
    pub(crate) fn of(things: Vec<Named>, could_not_be_named: usize, cut_short: bool) -> Self {
        Self {
            things,
            could_not_be_named,
            cut_short,
        }
    }

    /// What is in the folder, in the order a person would read it.
    #[must_use]
    pub fn things(&self) -> &[Named] {
        &self.things
    }

    /// How many things are not in this listing because their names could not
    /// be shown — see [`crate::named`].
    #[must_use]
    pub fn could_not_be_named(&self) -> usize {
        self.could_not_be_named
    }

    /// Whether the folder holds more than this answer carries.
    #[must_use]
    pub fn was_cut_short(&self) -> bool {
        self.cut_short
    }
}

/// What a search found.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Search {
    /// The files, by where they are.
    files: Vec<PathBuf>,
    /// Whether the search stopped before it had looked everywhere.
    cut_short: bool,
}

impl Search {
    /// A search that found these files.
    pub(crate) fn of(files: Vec<PathBuf>, cut_short: bool) -> Self {
        Self { files, cut_short }
    }

    /// The files it found.
    #[must_use]
    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }

    /// Whether the search stopped early — either because it had found as many
    /// as it was asked for, or because it had looked at as much of the folder
    /// as one search may. Either way there may be more.
    #[must_use]
    pub fn was_cut_short(&self) -> bool {
        self.cut_short
    }
}

/// An archive that was made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Archived {
    /// Where the archive is.
    at: PathBuf,
    /// How many things went into it.
    things: usize,
    /// How many were left out because they were links.
    left_out: usize,
    /// How many bytes the archive holds.
    bytes: u64,
}

impl Archived {
    /// An archive of this many things, at this path.
    pub(crate) fn of(at: PathBuf, things: usize, left_out: usize, bytes: u64) -> Self {
        Self {
            at,
            things,
            left_out,
            bytes,
        }
    }

    /// Where the archive is.
    #[must_use]
    pub fn at(&self) -> &Path {
        &self.at
    }

    /// How many things went in.
    #[must_use]
    pub fn things(&self) -> usize {
        self.things
    }

    /// How many things were left out because they were links rather than
    /// things.
    ///
    /// An archive that followed a link would copy whatever it points at —
    /// possibly somewhere nobody granted — into a file the agent may then move.
    /// So links are left where they are, and counted, because a person told
    /// *twelve things were archived* should also be told that one was not.
    #[must_use]
    pub fn left_out(&self) -> usize {
        self.left_out
    }

    /// How big the archive is.
    #[must_use]
    pub fn bytes(&self) -> u64 {
        self.bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// An answer that was bounded says so. A listing that was cut short and
    /// did not say it had been would read exactly like a complete one, and
    /// somebody would conclude from it that a file is not there.
    #[test]
    fn an_answer_that_left_something_out_says_that_it_did() {
        let complete = Listing::of(Vec::new(), 0, false);
        assert!(!complete.was_cut_short());
        assert_eq!(complete.could_not_be_named(), 0);
        assert!(complete.things().is_empty());

        let bounded = Listing::of(Vec::new(), 3, true);
        assert!(bounded.was_cut_short());
        assert_eq!(bounded.could_not_be_named(), 3);

        let search = Search::of(vec![PathBuf::from("/home/anna/Invoices/march.pdf")], true);
        assert!(search.was_cut_short());
        assert_eq!(search.files().len(), 1);

        let archive = Archived::of(PathBuf::from("/home/anna/Archive/2026.zip"), 12, 1, 4096);
        assert_eq!(archive.at(), Path::new("/home/anna/Archive/2026.zip"));
        assert_eq!(archive.things(), 12);
        assert_eq!(archive.left_out(), 1);
        assert_eq!(archive.bytes(), 4096);
    }

    /// Asking an answer for something it is not answers nothing, rather than
    /// something that happens to be the right shape.
    #[test]
    fn an_answer_is_only_the_answer_it_is() {
        let listed = Answer::Listed(Listing::default());
        assert!(listed.listed().is_some());
        assert!(listed.read().is_none());
        assert!(listed.found().is_none());
        assert!(listed.now_at().is_none());
        assert!(listed.archived().is_none());

        let moved = Answer::Moved(PathBuf::from("/home/anna/Archive/march.pdf"));
        assert_eq!(
            moved.now_at(),
            Some(Path::new("/home/anna/Archive/march.pdf"))
        );
        let renamed = Answer::Renamed(PathBuf::from("/home/anna/Invoices/march-2026.pdf"));
        assert!(renamed.now_at().is_some());

        assert_eq!(Answer::Read("hello".to_owned()).read(), Some("hello"));
        assert!(Answer::Found(Search::default()).found().is_some());
    }
}
