//! One thing under the folder, and what the index knows about it.

use std::time::{Duration, SystemTime};

use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Serialize};

use crate::kind::Kind;
use crate::wording;
use crate::words;

/// One thing the walk found under the folder.
///
/// `docs/contracts/file-index.md` describes how it is written down; every
/// field here is one there, by the same name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    /// Where it is below the folder, with `/` between the parts on every
    /// host, so that an index is the same file whichever machine reads it.
    pub below: String,
    /// What it is, from its bytes.
    pub kind: Kind,
    /// How many bytes it holds — for a link, the bytes of the link itself.
    pub bytes: u64,
    /// When it was last written, as the filesystem says.
    pub modified: Moment,
    /// Its words, or why there are none.
    pub contents: Contents,
}

impl Entry {
    /// Its name: the last part of where it is.
    #[must_use]
    pub fn name(&self) -> &str {
        self.below
            .rsplit_once('/')
            .map_or(self.below.as_str(), |(_, name)| name)
    }
}

/// A moment, as seconds and nanoseconds since the Unix epoch.
///
/// Written down as two numbers rather than a formatted date, because a reader
/// that is not alo OS then needs no calendar to compare two of them, and a
/// moment before 1970 does not happen on a filesystem this crate walks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Moment {
    /// Whole seconds since the epoch.
    pub secs: u64,
    /// Nanoseconds past those seconds.
    pub nanos: u32,
}

impl Moment {
    /// This time, as a moment. A time before the epoch, which a filesystem
    /// reports for a file whose time it does not know, is the epoch.
    #[must_use]
    pub fn of(when: SystemTime) -> Self {
        let since = when
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or(Duration::ZERO);
        Self {
            secs: since.as_secs(),
            nanos: since.subsec_nanos(),
        }
    }

    /// This moment, as a time.
    #[must_use]
    pub fn as_time(self) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::new(self.secs, self.nanos)
    }
}

/// The words in a file, or why the index has none.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "were", rename_all = "kebab-case")]
#[non_exhaustive]
pub enum Contents {
    /// The file is text, and these are its words: each once, lower case,
    /// sorted.
    Read {
        /// The words.
        words: Vec<String>,
    },
    /// The file is a kind this crate has no reader for, so it has no words
    /// here — a search by contents will not find it, and says so.
    NotText,
    /// The file could not be read, and this is what the machine said.
    NotRead {
        /// What the machine said, as a sentence.
        why: String,
    },
    /// The file is larger than an index reads, so its words were not read.
    TooBig {
        /// How large it is.
        bytes: u64,
    },
    /// A folder, a link or a device: not a file, so nothing to read.
    NotAFile,
}

impl Contents {
    /// The words, if the file had any.
    #[must_use]
    pub fn words(&self) -> &[String] {
        match self {
            Self::Read { words } => words,
            Self::NotText | Self::NotRead { .. } | Self::TooBig { .. } | Self::NotAFile => &[],
        }
    }

    /// Whether the file's words hold this one, which is already lower case.
    #[must_use]
    pub fn say(&self, word: &str) -> bool {
        wording::says(self.words(), word)
    }

    /// The sentence a window shows in place of words, in the language the
    /// person reads — or nothing, for a file whose words were read and for
    /// something that is not a file.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Option<Said> {
        let (word, filling) = match self {
            Self::Read { .. } | Self::NotAFile => return None,
            Self::NotText => (&words::NOT_TEXT, Filling::nothing()),
            Self::NotRead { why } => (&words::NOT_READ, Filling::of("why", why.clone())),
            Self::TooBig { .. } => (
                &words::TOO_BIG,
                Filling::of("most", alo_files::MOST_READ.to_string()),
            ),
        };
        Some(strings.say(&word.key(), &filling))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// A moment survives the trip to two numbers and back, and a time before
    /// the epoch is the epoch rather than a refusal. The nanoseconds are a
    /// multiple of a hundred because Windows keeps time in hundreds of them.
    #[test]
    fn a_moment_is_two_numbers_and_comes_back_whole() {
        let when = SystemTime::UNIX_EPOCH + Duration::new(1_760_000_000, 4200);
        let moment = Moment::of(when);
        assert_eq!(
            moment,
            Moment {
                secs: 1_760_000_000,
                nanos: 4200
            }
        );
        assert_eq!(moment.as_time(), when);
        assert_eq!(
            Moment::of(SystemTime::UNIX_EPOCH - Duration::from_secs(1)),
            Moment { secs: 0, nanos: 0 }
        );
        let spelled = serde_json::to_string(&moment).unwrap();
        assert_eq!(spelled, r#"{"secs":1760000000,"nanos":4200}"#);
    }

    /// The name is the last part, and the words answer only for a file that
    /// was read.
    #[test]
    fn an_entry_is_named_by_its_last_part_and_answers_from_its_words() {
        let entry = Entry {
            below: "2026/March/march.pdf".to_owned(),
            kind: Kind::Text,
            bytes: 10,
            modified: Moment { secs: 1, nanos: 0 },
            contents: Contents::Read {
                words: vec!["an".to_owned(), "invoice".to_owned()],
            },
        };
        assert_eq!(entry.name(), "march.pdf");
        assert!(entry.contents.say("invoice"));
        assert!(!entry.contents.say("contract"));
        let at_the_top = Entry {
            below: "notes.txt".to_owned(),
            contents: Contents::NotText,
            ..entry
        };
        assert_eq!(at_the_top.name(), "notes.txt");
        assert!(!at_the_top.contents.say("invoice"));
        assert!(at_the_top.contents.words().is_empty());
    }

    /// Contents are spelled with a tag, so a reader can tell the five apart
    /// without knowing which fields each has.
    #[test]
    fn contents_are_tagged_in_the_file() {
        let spelled = |contents: &Contents| serde_json::to_string(contents).unwrap();
        assert_eq!(
            spelled(&Contents::Read {
                words: vec!["a".to_owned()]
            }),
            r#"{"were":"read","words":["a"]}"#
        );
        assert_eq!(spelled(&Contents::NotText), r#"{"were":"not-text"}"#);
        assert_eq!(
            spelled(&Contents::NotRead {
                why: "no".to_owned()
            }),
            r#"{"were":"not-read","why":"no"}"#
        );
        assert_eq!(
            spelled(&Contents::TooBig { bytes: 5 }),
            r#"{"were":"too-big","bytes":5}"#
        );
        assert_eq!(spelled(&Contents::NotAFile), r#"{"were":"not-a-file"}"#);
    }
}
