//! An answer held on its own, apart from the index it came from.
//!
//! An [`Answer`] borrows from its [`crate::Index`]: every matched entry is a
//! reference into the index's own list, which is the right shape for a
//! caller holding an index and asking it. A search over every indexed folder
//! reads each index from its file and answers from it inside one call, and
//! the index is gone when the call returns — so what it hands back has to be
//! an answer that stands on its own. [`Held`] is that: the same five things
//! an [`Answer`] says, owned, and [`Held::answer`] gives the borrowing view
//! back so that every sentence [`NotSearched::said`] can say, and every
//! question [`NotSearched::is_nothing`] answers, is said and answered by one
//! piece of code rather than two.
//!
//! Nothing is added on the way: a held answer is the answer it was made
//! from, and a unit test holds the round trip to equality.

use std::path::PathBuf;
use std::time::Duration;

use crate::answer::{Answer, NotSearched};
use crate::covered::Unread;
use crate::entry::{Entry, Moment};

/// What one search answered, held apart from its index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Held {
    /// Every entry that answered, in the index's own order.
    pub found: Vec<Entry>,
    /// Everything the query could not be held against.
    pub not_searched: Unsearched,
    /// The moment the index this answered from was made, as the caller said
    /// it then; [`None`] for an index read back from a file written before
    /// the moment was kept.
    pub made: Option<Moment>,
    /// How long the answer took, measured around the search alone.
    pub took: Duration,
}

/// Everything a search did not look at, held apart from its index.
///
/// Field for field what [`NotSearched`] is, owned.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unsearched {
    /// The folder the index is of: nothing outside it was searched.
    pub outside: PathBuf,
    /// Folders the machine would not let the index read, with what it said.
    pub folders_unread: Vec<Unread>,
    /// Folders on another filesystem, which the index did not enter.
    pub elsewhere: Vec<String>,
    /// Folders no walk could list to their end, each holding more than one
    /// walk looks at, at one level.
    pub not_entered: Vec<String>,
    /// How many things were left out of the index because their names
    /// cannot be shown.
    pub unnamed: usize,
    /// Files the machine would not let the index open.
    pub files_unread: Vec<Entry>,
    /// Files of a kind whose words the index cannot read.
    pub no_reader: Vec<Entry>,
    /// Files larger than an index reads.
    pub too_big: Vec<Entry>,
    /// Files whose words were not all kept, and not all searched.
    pub not_all_kept: Vec<Entry>,
}

impl Held {
    /// This answer, held: every borrowed entry copied out of its index.
    #[must_use]
    pub fn of(answer: &Answer<'_>) -> Self {
        let not = &answer.not_searched;
        Self {
            found: copied(&answer.found),
            not_searched: Unsearched {
                outside: not.outside.to_path_buf(),
                folders_unread: not.folders_unread.iter().map(|&u| u.clone()).collect(),
                elsewhere: not.elsewhere.iter().map(|&s| s.to_owned()).collect(),
                not_entered: not.not_entered.iter().map(|&s| s.to_owned()).collect(),
                unnamed: not.unnamed,
                files_unread: copied(&not.files_unread),
                no_reader: copied(&not.no_reader),
                too_big: copied(&not.too_big),
                not_all_kept: copied(&not.not_all_kept),
            },
            made: answer.made,
            took: answer.took,
        }
    }

    /// The borrowing view of this answer, so that what an [`Answer`] can say
    /// a held one can say, by the same code.
    #[must_use]
    pub fn answer(&self) -> Answer<'_> {
        Answer {
            found: self.found.iter().collect(),
            not_searched: self.not_searched.borrowed(),
            made: self.made,
            took: self.took,
        }
    }
}

impl Unsearched {
    /// The borrowing view, for [`NotSearched::said`] and
    /// [`NotSearched::is_nothing`].
    #[must_use]
    pub fn borrowed(&self) -> NotSearched<'_> {
        NotSearched {
            outside: &self.outside,
            folders_unread: self.folders_unread.iter().collect(),
            elsewhere: self.elsewhere.iter().map(String::as_str).collect(),
            not_entered: self.not_entered.iter().map(String::as_str).collect(),
            unnamed: self.unnamed,
            files_unread: self.files_unread.iter().collect(),
            no_reader: self.no_reader.iter().collect(),
            too_big: self.too_big.iter().collect(),
            not_all_kept: self.not_all_kept.iter().collect(),
        }
    }
}

/// These borrowed entries, owned.
fn copied(entries: &[&Entry]) -> Vec<Entry> {
    entries.iter().map(|&entry| entry.clone()).collect()
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::covered::Covered;
    use crate::entry::Contents;
    use crate::index::Index;
    use crate::kept_words::KeptWords;
    use crate::kind::Kind;
    use crate::query::Query;

    /// An entry for these tests.
    fn an_entry(below: &str, kind: Kind, contents: Contents) -> Entry {
        Entry {
            below: below.to_owned(),
            kind,
            bytes: 1,
            modified: Moment { secs: 1, nanos: 0 },
            contents,
        }
    }

    /// An index with one of everything an answer can carry.
    fn an_index() -> Index {
        Index {
            of: PathBuf::from("/home/ada/Documents"),
            made: Some(Moment {
                secs: 1_760_000_000,
                nanos: 0,
            }),
            covered: Covered {
                whole: false,
                most: 3,
                unread: vec![Unread {
                    below: "private".to_owned(),
                    why: "permission denied".to_owned(),
                }],
                elsewhere: vec!["mnt".to_owned()],
                not_entered: vec!["late".to_owned()],
                unnamed: 2,
            },
            entries: vec![
                an_entry("march.pdf", Kind::Pdf, Contents::NotText),
                an_entry(
                    "locked.txt",
                    Kind::Unread,
                    Contents::NotRead {
                        why: "permission denied".to_owned(),
                    },
                ),
                an_entry(
                    "notes.txt",
                    Kind::Text,
                    Contents::Read {
                        words: KeptWords::from(["contract"]),
                    },
                ),
                an_entry("big.txt", Kind::Text, Contents::TooBig { bytes: 9 }),
            ],
            opened: 0,
        }
    }

    /// **A held answer is the answer it was made from**, nothing added and
    /// nothing dropped: the view given back is equal to the original, with
    /// every list of what was not searched filled and carried across, and
    /// the sentences said from it are the same sentences.
    #[test]
    fn a_held_answer_is_the_answer_it_was_made_from() {
        let index = an_index();
        let answer = index.answer(&Query::saying("contract")).unwrap();
        assert_eq!(answer.found.len(), 1);
        assert_eq!(answer.not_searched.files_unread.len(), 1);
        assert_eq!(answer.not_searched.no_reader.len(), 1);
        assert_eq!(answer.not_searched.too_big.len(), 1);
        assert!(!answer.not_searched.is_nothing());

        let held = Held::of(&answer);
        assert_eq!(held.answer(), answer);
        assert_eq!(held.made, index.made);
        assert_eq!(held.took, answer.took);
        assert_eq!(held.not_searched.outside, Path::new("/home/ada/Documents"));
        assert_eq!(held.not_searched.unnamed, 2);
        assert_eq!(held.not_searched.not_entered, ["late"]);

        let strings = alo_strings::Strings::of(crate::finding_words().unwrap());
        let said: Vec<String> = answer
            .not_searched
            .said(&strings)
            .into_iter()
            .map(alo_strings::Said::into_text)
            .collect();
        let said_held: Vec<String> = held
            .not_searched
            .borrowed()
            .said(&strings)
            .into_iter()
            .map(alo_strings::Said::into_text)
            .collect();
        assert_eq!(said, said_held);
    }

    /// An answer over an index that reached everything, held, still says
    /// nothing was left out.
    #[test]
    fn a_held_answer_over_everything_still_says_nothing_was_left_out() {
        let mut index = an_index();
        index.covered = Covered {
            whole: true,
            most: 3,
            unread: Vec::new(),
            elsewhere: Vec::new(),
            not_entered: Vec::new(),
            unnamed: 0,
        };
        let answer = index.answer(&Query::named("txt")).unwrap();
        let held = Held::of(&answer);
        assert!(held.not_searched.borrowed().is_nothing());
        assert_eq!(held.found.len(), 3);
        assert_eq!(held.answer(), answer);
    }
}
