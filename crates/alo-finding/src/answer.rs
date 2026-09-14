//! What a search answers: what matched, beside what was not searched.
//!
//! An empty answer has to be *nothing matched* and never *nothing was looked
//! at*, and the difference is the second list. [`Answer::found`] is every
//! entry that answered the query, in the index's own order; [`Answer::not_searched`]
//! is everything the query could not be held against — a folder the machine
//! would not read, a folder on another disk, a folder the index stopped in, a
//! file that could not be opened, a file of a kind with no reader, a file
//! larger than an index reads — and the folder outside which nothing was
//! looked at, because an index is of one folder a person named.
//!
//! What counts as *not searched* depends on what was asked. A search by
//! name or by date holds against every entry, because a name and a date come
//! from the walk and not from the bytes; a PDF searched by name was searched.
//! A search by words cannot be held against a file whose words the index
//! does not have, and a search by kind cannot be held against a file whose
//! bytes were never seen — and those are the files [`NotSearched`] lists.
//!
//! # An answer says how old it is
//!
//! [`Answer::made`] is the moment the index it answered from was made, as
//! the caller said it when the index was made, so that a window can put *as
//! of Tuesday* beside the results: a search over a folder indexed last week
//! answers about last week's folder, and the person should be able to tell.
//! How a moment is written for the reader is the window's, in the reader's
//! own calendar; `alo-strings` formats no dates, by its own design, and this
//! crate hands the moment over as two numbers rather than as English.

use std::path::{Path, PathBuf};
use std::time::Duration;

use alo_strings::{Counting, Filling, Said, Strings};

use crate::covered::Unread;
use crate::entry::{Entry, Moment};
use crate::words;

/// What a search answered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer<'a> {
    /// Every entry that answered, in the index's own order.
    pub found: Vec<&'a Entry>,
    /// Everything the query could not be held against.
    pub not_searched: NotSearched<'a>,
    /// The moment the index this answered from was made, as the caller said
    /// it then — so a window can say *as of Tuesday* beside the results.
    /// [`None`] for an index read back from a file written before the moment
    /// was kept.
    pub made: Option<Moment>,
    /// How long the answer took, measured around the search alone.
    pub took: Duration,
}

/// Everything a search did not look at, so that an empty answer can say
/// which of the two things it means.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NotSearched<'a> {
    /// The folder the index is of: nothing outside it was searched.
    pub outside: &'a Path,
    /// Folders the machine would not let the index read, with what it said.
    pub folders_unread: Vec<&'a Unread>,
    /// Folders on another filesystem, which the index did not enter.
    pub elsewhere: Vec<&'a str>,
    /// Folders no walk could list to their end, each holding more than one
    /// walk looks at, at one level.
    pub not_entered: Vec<&'a str>,
    /// How many things were left out of the index because their names
    /// cannot be shown.
    pub unnamed: usize,
    /// Files the machine would not let the index open, so neither their
    /// kind nor their words could be searched. Empty for a search by name
    /// or date alone.
    pub files_unread: Vec<&'a Entry>,
    /// Files of a kind whose words the index cannot read. Empty unless the
    /// search was by words.
    pub no_reader: Vec<&'a Entry>,
    /// Files larger than an index reads, whose words it did not. Empty
    /// unless the search was by words.
    pub too_big: Vec<&'a Entry>,
}

impl NotSearched<'_> {
    /// Whether everything under the folder was searched — so that an empty
    /// answer means nothing matched.
    #[must_use]
    pub fn is_nothing(&self) -> bool {
        self.folders_unread.is_empty()
            && self.elsewhere.is_empty()
            && self.not_entered.is_empty()
            && self.unnamed == 0
            && self.files_unread.is_empty()
            && self.no_reader.is_empty()
            && self.too_big.is_empty()
    }

    /// Every sentence a window shows beside the answer, in the language the
    /// person reads: one saying nothing outside the folder was searched,
    /// then one per folder and per unread file, then one counting the files
    /// of a kind with no reader and one counting the files too large — each
    /// of the last two only when there are any.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Vec<Said> {
        let mut said = vec![strings.say(
            &words::NOT_SEARCHED_OUTSIDE.key(),
            &Filling::of("folder", self.outside.display().to_string()),
        )];
        for unread in &self.folders_unread {
            said.push(strings.say(
                &words::NOT_SEARCHED_FOLDER_UNREAD.key(),
                &Filling::of("below", self.where_is(&unread.below)).and("why", unread.why.clone()),
            ));
        }
        for below in &self.elsewhere {
            said.push(strings.say(
                &words::NOT_SEARCHED_ELSEWHERE.key(),
                &Filling::of("below", self.where_is(below)),
            ));
        }
        for below in &self.not_entered {
            said.push(strings.say(
                &words::NOT_SEARCHED_NOT_ENTERED.key(),
                &Filling::of("below", self.where_is(below)),
            ));
        }
        if self.unnamed > 0 {
            said.push(strings.count(
                &words::UNNAMED.key(),
                &Counting::of(self.unnamed as u64),
                &Filling::of("unnamed", self.unnamed.to_string()),
            ));
        }
        for entry in &self.files_unread {
            let why = match &entry.contents {
                crate::entry::Contents::NotRead { why } => why.clone(),
                _ => String::new(),
            };
            said.push(strings.say(
                &words::NOT_SEARCHED_FILE_UNREAD.key(),
                &Filling::of("below", self.where_is(&entry.below)).and("why", why),
            ));
        }
        for (counted, files) in [
            (&words::NOT_SEARCHED_NO_READER, &self.no_reader),
            (&words::NOT_SEARCHED_TOO_BIG, &self.too_big),
        ] {
            if !files.is_empty() {
                said.push(strings.count(
                    &counted.key(),
                    &Counting::of(files.len() as u64),
                    &Filling::of("files", files.len().to_string()),
                ));
            }
        }
        said
    }

    /// Where something below the folder is on the disk, as a person reads
    /// it: the folder itself for the empty path.
    fn where_is(&self, below: &str) -> String {
        let mut at: PathBuf = self.outside.to_path_buf();
        for part in below.split('/').filter(|part| !part.is_empty()) {
            at.push(part);
        }
        at.display().to_string()
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::entry::{Contents, Moment};
    use crate::kind::Kind;

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

    /// **A search that looked at everything says only where it looked**, and
    /// one that did not says each thing it did not look at, as a path a
    /// person can open — the folder itself for the folder itself.
    #[test]
    fn what_was_not_searched_is_said_as_paths_and_counts() {
        let strings = Strings::of(crate::finding_words().unwrap());
        let of = Path::new("/home/ada/Documents");
        let everything = NotSearched {
            outside: of,
            folders_unread: Vec::new(),
            elsewhere: Vec::new(),
            not_entered: Vec::new(),
            unnamed: 0,
            files_unread: Vec::new(),
            no_reader: Vec::new(),
            too_big: Vec::new(),
        };
        assert!(everything.is_nothing());
        let said = everything.said(&strings);
        let texts: Vec<&str> = said.iter().map(Said::text).collect();
        assert_eq!(texts, ["Nothing outside /home/ada/Documents was searched."]);

        let unread = Unread {
            below: "private".to_owned(),
            why: "permission denied".to_owned(),
        };
        let locked = an_entry(
            "2026/locked.txt",
            Kind::Unread,
            Contents::NotRead {
                why: "permission denied".to_owned(),
            },
        );
        let pdf = an_entry("2026/march.pdf", Kind::Pdf, Contents::NotText);
        let big = an_entry("big.txt", Kind::Text, Contents::TooBig { bytes: 9 });
        let not = NotSearched {
            outside: of,
            folders_unread: vec![&unread],
            elsewhere: vec!["mnt"],
            not_entered: vec![""],
            unnamed: 2,
            files_unread: vec![&locked],
            no_reader: vec![&pdf, &pdf],
            too_big: vec![&big],
        };
        assert!(!not.is_nothing());
        let said = not.said(&strings);
        for said in &said {
            assert!(!said.is_a_bug(), "{said}");
            assert!(said.unfilled().is_empty(), "{said}");
        }
        let texts: Vec<&str> = said.iter().map(Said::text).collect();
        let at = |below: &str| {
            let mut at = Path::new("/home/ada/Documents").to_path_buf();
            at.extend(below.split('/'));
            at.display().to_string()
        };
        assert_eq!(
            texts,
            [
                "Nothing outside /home/ada/Documents was searched.".to_owned(),
                format!(
                    "{} could not be read, so nothing in it was searched: permission denied",
                    at("private")
                ),
                format!(
                    "{} is on another disk, so nothing in it was searched.",
                    at("mnt")
                ),
                "The index stopped before it had finished /home/ada/Documents, so not all of it \
                 was searched."
                    .to_owned(),
                "2 things whose names cannot be shown are not indexed.".to_owned(),
                format!(
                    "{} could not be read, so it was not searched: permission denied",
                    at("2026/locked.txt")
                ),
                "2 files are of kinds whose words cannot be read, so they were not searched by \
                 their words."
                    .to_owned(),
                "One file is larger than an index reads, so it was not searched by its words."
                    .to_owned(),
            ]
        );
    }
}
