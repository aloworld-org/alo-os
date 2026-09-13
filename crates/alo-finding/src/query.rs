//! A question over the index: by name, kind, date and contents.
//!
//! A [`Query`] is one or more of the four, and an entry answers when it
//! matches every part given — a filter, not a ranking. Nothing here orders
//! the answer by anything the person did not ask for: [`crate::Index::find`]
//! answers in the index's own order, which is the walk's, folder by folder and
//! name by name.
//!
//! A query is built from one part and grows by `and_`: there is no empty
//! query to construct, so *answered with everything* is not a thing this
//! type can do by accident. A part that is empty — a name of no letters, a
//! sentence of no words — matches every entry on that axis, which is what a
//! filter with no condition does; refusing a person's empty *question* is the
//! search's job, in front of this, and the plan's next task.

use std::time::SystemTime;

use crate::entry::Entry;
use crate::kind::Kind;
use crate::wording;

/// A question over the index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Query {
    /// Part of a name, in lower case, or nothing.
    named: Option<String>,
    /// A kind, or nothing.
    kind: Option<Kind>,
    /// Modified at or after this moment, or nothing.
    since: Option<SystemTime>,
    /// Modified before this moment, or nothing.
    before: Option<SystemTime>,
    /// Words the file must hold, every one, in lower case, or nothing.
    saying: Option<Vec<String>>,
}

impl Query {
    /// Everything whose name holds this, whichever case either is in.
    #[must_use]
    pub fn named(part: &str) -> Self {
        Self::nothing().and_named(part)
    }

    /// Everything of this kind.
    #[must_use]
    pub fn of_kind(kind: Kind) -> Self {
        Self::nothing().and_of_kind(kind)
    }

    /// Everything last written at or after this moment.
    #[must_use]
    pub fn changed_since(when: SystemTime) -> Self {
        Self::nothing().and_changed_since(when)
    }

    /// Everything last written before this moment.
    #[must_use]
    pub fn changed_before(when: SystemTime) -> Self {
        Self::nothing().and_changed_before(when)
    }

    /// Everything whose words hold every word of this, whichever case either
    /// is in.
    #[must_use]
    pub fn saying(words: &str) -> Self {
        Self::nothing().and_saying(words)
    }

    /// This, and the name must hold this too.
    #[must_use]
    pub fn and_named(mut self, part: &str) -> Self {
        self.named = Some(part.to_lowercase());
        self
    }

    /// This, and of this kind.
    #[must_use]
    pub fn and_of_kind(mut self, kind: Kind) -> Self {
        self.kind = Some(kind);
        self
    }

    /// This, and last written at or after this moment.
    #[must_use]
    pub fn and_changed_since(mut self, when: SystemTime) -> Self {
        self.since = Some(when);
        self
    }

    /// This, and last written before this moment.
    #[must_use]
    pub fn and_changed_before(mut self, when: SystemTime) -> Self {
        self.before = Some(when);
        self
    }

    /// This, and holding every word of this too.
    #[must_use]
    pub fn and_saying(mut self, words: &str) -> Self {
        self.saying = Some(wording::words_of(words));
        self
    }

    /// The words this query asks for, in the form they are looked up in.
    #[must_use]
    pub fn words(&self) -> &[String] {
        self.saying.as_deref().unwrap_or(&[])
    }

    /// Whether this entry answers the query: every part given matches.
    #[must_use]
    pub fn matches(&self, entry: &Entry) -> bool {
        if let Some(named) = &self.named
            && !entry.name().to_lowercase().contains(named.as_str())
        {
            return false;
        }
        if let Some(kind) = self.kind
            && entry.kind != kind
        {
            return false;
        }
        let modified = entry.modified.as_time();
        if let Some(since) = self.since
            && modified < since
        {
            return false;
        }
        if let Some(before) = self.before
            && modified >= before
        {
            return false;
        }
        if let Some(saying) = &self.saying
            && !saying.iter().all(|word| entry.contents.say(word))
        {
            return false;
        }
        true
    }

    /// No part at all — never public, because a query with nothing in it is
    /// the one thing this type must not be.
    fn nothing() -> Self {
        Self {
            named: None,
            kind: None,
            since: None,
            before: None,
            saying: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::entry::{Contents, Moment};

    /// An entry for these tests: a text file with three words, written at
    /// second one thousand.
    fn the_contract() -> Entry {
        Entry {
            below: "2026/Contract-Anna.txt".to_owned(),
            kind: Kind::Text,
            bytes: 30,
            modified: Moment {
                secs: 1000,
                nanos: 0,
            },
            contents: Contents::Read {
                words: vec![
                    "anna".to_owned(),
                    "contract".to_owned(),
                    "summer".to_owned(),
                ],
            },
        }
    }

    /// A moment, as a time.
    fn at(secs: u64) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(secs)
    }

    /// **Each of the four parts matches on its own and refuses on its own**,
    /// and a name is matched whichever case either side is in.
    #[test]
    fn each_part_answers_and_each_part_refuses() {
        let entry = the_contract();
        assert!(Query::named("contract").matches(&entry));
        assert!(Query::named("ANNA").matches(&entry));
        assert!(!Query::named("invoice").matches(&entry));
        assert!(
            !Query::named("2026").matches(&entry),
            "the name, not the path"
        );

        assert!(Query::of_kind(Kind::Text).matches(&entry));
        assert!(!Query::of_kind(Kind::Pdf).matches(&entry));

        assert!(Query::changed_since(at(1000)).matches(&entry));
        assert!(!Query::changed_since(at(1001)).matches(&entry));
        assert!(Query::changed_before(at(1001)).matches(&entry));
        assert!(!Query::changed_before(at(1000)).matches(&entry));

        assert!(Query::saying("Contract, Summer").matches(&entry));
        assert!(!Query::saying("contract winter").matches(&entry));
        assert_eq!(
            Query::saying("Summer contract").words(),
            ["contract", "summer"]
        );
    }

    /// Parts combine as *and*: every one given has to match.
    #[test]
    fn parts_combine_and_every_one_has_to_match() {
        let entry = the_contract();
        let both = Query::named("anna")
            .and_of_kind(Kind::Text)
            .and_saying("summer");
        assert!(both.matches(&entry));
        assert!(!both.clone().and_changed_since(at(2000)).matches(&entry));
        assert!(!both.and_of_kind(Kind::Pdf).matches(&entry));
    }

    /// A file whose words were not read does not answer by contents, even to
    /// a word that would be in it.
    #[test]
    fn a_file_with_no_words_read_does_not_answer_by_contents() {
        let entry = Entry {
            contents: Contents::NotText,
            kind: Kind::Pdf,
            ..the_contract()
        };
        assert!(Query::of_kind(Kind::Pdf).matches(&entry));
        assert!(!Query::saying("contract").matches(&entry));
        assert!(
            Query::saying("").matches(&entry),
            "no words asked, none needed"
        );
    }
}
