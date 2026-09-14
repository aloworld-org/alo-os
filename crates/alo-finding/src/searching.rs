//! Answering a query: refused if it is not one, timed, and honest about
//! what it could not be held against.
//!
//! One pass over the entries. Each is either held against every part of the
//! query, or set aside in [`NotSearched`] because a part could not be held
//! against it — a file whose bytes were never seen cannot answer *is it a
//! PDF*, and a file whose words the index does not have cannot answer *does
//! it say contract*. What the walk itself never reached comes from the
//! index's [`crate::Covered`], read from the index and never from the disk.
//!
//! Nothing here ranks. The answer is in the index's own order, which is the
//! walk's: folder by folder, name by name, and nothing the person did not ask
//! for moves an entry up or down.

use std::time::Instant;

use crate::answer::{Answer, NotSearched};
use crate::asking::{self, NotAsked};
use crate::entry::Contents;
use crate::index::Index;
use crate::kind::Kind;
use crate::query::Query;

/// This index's answer to this query.
///
/// # Errors
///
/// [`NotAsked`] for a query that is not one, before anything is searched.
pub(crate) fn answered<'a>(index: &'a Index, query: &Query) -> Result<Answer<'a>, NotAsked> {
    asking::checked(query)?;
    Ok(searched(index, query))
}

/// This index's answer to a query already checked to be one.
///
/// The pass itself, apart from the check, so that a search over every
/// indexed folder can check the query once and put it to each index without
/// a second refusal that could never happen having to be handled anyway.
pub(crate) fn searched<'a>(index: &'a Index, query: &Query) -> Answer<'a> {
    let started = Instant::now();

    let by_words = !query.words().is_empty();
    // Asking for the files that could not be read is asking for exactly
    // those, so they are not *unsearched* for that one kind.
    let by_kind = query.kind_asked().is_some_and(|kind| kind != Kind::Unread);

    let covered = &index.covered;
    let mut not_searched = NotSearched {
        outside: &index.of,
        folders_unread: covered.unread.iter().collect(),
        elsewhere: covered.elsewhere.iter().map(String::as_str).collect(),
        not_entered: covered.not_entered.iter().map(String::as_str).collect(),
        unnamed: covered.unnamed,
        files_unread: Vec::new(),
        no_reader: Vec::new(),
        too_big: Vec::new(),
    };
    let mut found = Vec::new();
    for entry in &index.entries {
        match &entry.contents {
            Contents::NotRead { .. } if by_words || by_kind => {
                not_searched.files_unread.push(entry);
                continue;
            }
            Contents::NotText if by_words => {
                not_searched.no_reader.push(entry);
                continue;
            }
            Contents::TooBig { .. } if by_words => {
                not_searched.too_big.push(entry);
                continue;
            }
            Contents::Read { .. }
            | Contents::NotText
            | Contents::NotRead { .. }
            | Contents::TooBig { .. }
            | Contents::NotAFile => {}
        }
        if query.matches(entry) {
            found.push(entry);
        }
    }

    Answer {
        found,
        not_searched,
        made: index.made,
        took: started.elapsed(),
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::covered::{Covered, Unread};
    use crate::entry::{Entry, Moment};

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

    /// An index of a folder with one of everything a search cannot hold a
    /// query against: a folder the machine would not read, a folder on
    /// another disk, a file that could not be opened, a PDF, a file too
    /// large — and two text files that were read.
    fn an_index() -> Index {
        Index {
            of: PathBuf::from("/home/ada/Documents"),
            made: Some(Moment {
                secs: 1_760_000_000,
                nanos: 0,
            }),
            covered: Covered {
                whole: true,
                most: 20_000,
                unread: vec![Unread {
                    below: "private".to_owned(),
                    why: "permission denied".to_owned(),
                }],
                elsewhere: vec!["mnt".to_owned()],
                not_entered: Vec::new(),
                unnamed: 0,
            },
            entries: vec![
                an_entry("2026", Kind::Folder, Contents::NotAFile),
                an_entry("2026/march.pdf", Kind::Pdf, Contents::NotText),
                an_entry(
                    "2026/locked.txt",
                    Kind::Unread,
                    Contents::NotRead {
                        why: "permission denied".to_owned(),
                    },
                ),
                an_entry(
                    "2026/notes.txt",
                    Kind::Text,
                    Contents::Read {
                        words: vec!["anna".to_owned(), "contract".to_owned()],
                    },
                ),
                an_entry("big.txt", Kind::Text, Contents::TooBig { bytes: 9 }),
                an_entry(
                    "letter.txt",
                    Kind::Text,
                    Contents::Read {
                        words: vec!["dear".to_owned(), "anna".to_owned()],
                    },
                ),
            ],
            opened: 0,
        }
    }

    /// The paths below the folder of these entries.
    fn below(entries: &[&Entry]) -> Vec<String> {
        entries.iter().map(|entry| entry.below.clone()).collect()
    }

    /// **What was not searched is listed beside what was found, and depends
    /// on what was asked.** By name, every file was searched, and only what
    /// the walk never reached is listed; by words, the files whose words
    /// the index does not have are listed too; by kind, the file whose
    /// bytes were never seen. And what the walk never reached is listed
    /// whatever was asked.
    #[test]
    fn what_was_not_searched_depends_on_what_was_asked() {
        let index = an_index();

        let by_name = answered(&index, &Query::named("txt")).unwrap();
        assert_eq!(
            below(&by_name.found),
            ["2026/locked.txt", "2026/notes.txt", "big.txt", "letter.txt"],
            "a name is searched on every file, read or not"
        );
        assert!(by_name.not_searched.files_unread.is_empty());
        assert!(by_name.not_searched.no_reader.is_empty());
        assert!(by_name.not_searched.too_big.is_empty());
        assert_eq!(by_name.not_searched.folders_unread.len(), 1);
        assert_eq!(by_name.not_searched.elsewhere, ["mnt"]);
        assert_eq!(by_name.not_searched.outside, index.of);
        assert!(!by_name.not_searched.is_nothing());
        assert_eq!(
            by_name.made, index.made,
            "the answer says when its index was made"
        );

        let by_words = answered(&index, &Query::saying("Anna")).unwrap();
        assert_eq!(below(&by_words.found), ["2026/notes.txt", "letter.txt"]);
        assert_eq!(
            below(&by_words.not_searched.files_unread),
            ["2026/locked.txt"]
        );
        assert_eq!(below(&by_words.not_searched.no_reader), ["2026/march.pdf"]);
        assert_eq!(below(&by_words.not_searched.too_big), ["big.txt"]);

        let by_kind = answered(&index, &Query::of_kind(Kind::Text)).unwrap();
        assert_eq!(
            below(&by_kind.found),
            ["2026/notes.txt", "big.txt", "letter.txt"]
        );
        assert_eq!(
            below(&by_kind.not_searched.files_unread),
            ["2026/locked.txt"]
        );
        assert!(by_kind.not_searched.no_reader.is_empty());
        assert!(by_kind.not_searched.too_big.is_empty());

        let the_unread = answered(&index, &Query::of_kind(Kind::Unread)).unwrap();
        assert_eq!(below(&the_unread.found), ["2026/locked.txt"]);
        assert!(
            the_unread.not_searched.files_unread.is_empty(),
            "asking for the unread files is asking for exactly those"
        );
    }

    /// **An empty answer is *nothing matched*, and says so apart from
    /// *nothing was looked at*.** Over an index that reached everything, an
    /// empty answer's list is empty; over one that did not, the same empty
    /// answer names what it did not look at.
    #[test]
    fn an_empty_answer_is_nothing_matched_and_never_nothing_was_looked_at() {
        let mut index = an_index();
        let empty = answered(&index, &Query::named("invoice")).unwrap();
        assert!(empty.found.is_empty());
        assert!(!empty.not_searched.is_nothing());

        index.covered.unread.clear();
        index.covered.elsewhere.clear();
        let empty = answered(&index, &Query::named("invoice")).unwrap();
        assert!(empty.found.is_empty());
        assert!(empty.not_searched.is_nothing(), "{:?}", empty.not_searched);
        let by_words = answered(&index, &Query::saying("invoice")).unwrap();
        assert!(by_words.found.is_empty());
        assert!(!by_words.not_searched.is_nothing());
        assert_eq!(by_words.not_searched.no_reader.len(), 1);
    }

    /// **A query that is not one is refused before anything is searched**,
    /// and a bound was not walked, because the answer is a refusal and not
    /// an answer with everything in it.
    #[test]
    fn a_query_that_is_not_one_is_refused_and_not_answered_with_everything() {
        let index = an_index();
        assert_eq!(
            answered(&index, &Query::named("")).unwrap_err(),
            NotAsked::Nothing
        );
        let forty = (0..40)
            .map(|n| format!("w{n}"))
            .collect::<Vec<_>>()
            .join(" ");
        assert_eq!(
            answered(&index, &Query::saying(&forty)).unwrap_err(),
            NotAsked::MoreThanASentence {
                words: 40,
                most: asking::A_SENTENCE
            }
        );
        assert_eq!(
            answered(&index, &Query::named(&"a".repeat(256))).unwrap_err(),
            NotAsked::LongerThanAName {
                chars: 256,
                most: asking::A_NAME
            }
        );
    }

    /// The answer carries how long it took, and the time is around the
    /// search alone: an index with nothing in it answers in no time.
    #[test]
    fn the_answer_says_how_long_it_took() {
        let index = an_index();
        let answer = answered(&index, &Query::named("txt")).unwrap();
        assert!(
            answer.took < std::time::Duration::from_secs(1),
            "{:?}",
            answer.took
        );
    }
}
