//! The words in a file, or why the index has none — and, for a file with
//! more different words than an index keeps, that not all of them were kept.
//!
//! # Bounded, and said
//!
//! [`crate::InHand`] holds every index for as long as a file manager is open, so
//! what one entry holds is memory the shell gives a search. The words of a
//! file are kept once each, never once per occurrence, and at most
//! [`crate::MOST_WORDS`] of them: a file with more different words than that
//! — a log, a table of numbers, a dump of identifiers — keeps the first ones
//! it says, in the file's own order, and is [`Contents::NotAllKept`], carrying
//! how many it had that were not kept. Nothing is left out quietly: the entry
//! says it, and a search by words that does not find such a file lists it
//! beside the answer as a file whose words were not all searched.
//!
//! # Spelled the way `made` was added
//!
//! In the index file, a file whose words were not all kept is still
//! `"were":"read"`, with its kept words, and one more field, `unkept`, that
//! is left out when it would be nothing. A reader from before this field
//! existed ignores it and reads the kept words, as the contract says a later
//! field is read; `format` stays `1`; and an entry whose words were all kept
//! is written byte for byte as it was before.

use alo_strings::{Filling, Said, Strings};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::kept_words::KeptWords;
use crate::wording;
use crate::words;

/// The words in a file, or why the index has none.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Contents {
    /// The file is text, and these are its words: each once, lower case,
    /// sorted.
    Read {
        /// The words.
        words: KeptWords,
    },
    /// The file is text with more different words than an index keeps:
    /// these are the first [`crate::MOST_WORDS`] of them in the order the
    /// file says them, each once, lower case, sorted — and `unkept` more were
    /// not kept, so a search by words may not find this file, and says so.
    NotAllKept {
        /// The words that were kept.
        words: KeptWords,
        /// How many different words the file had that were not kept.
        unkept: usize,
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
    /// The words of a text file: [`Contents::Read`] when none were left
    /// out, and [`Contents::NotAllKept`] when `unkept` were. The list is held
    /// with no room to spare, so what an entry holds is what it says.
    pub(crate) fn of_words(words: Vec<String>, unkept: usize) -> Self {
        let words = KeptWords::of(&words);
        if unkept == 0 {
            Self::Read { words }
        } else {
            Self::NotAllKept { words, unkept }
        }
    }

    /// The words this file kept, held in one piece — or nothing, for a file
    /// with no words to keep.
    ///
    /// The way to ask since 2026-09-19. [`Contents::words`] is the old way and
    /// still answers, at the cost of building the list it used to hold.
    #[must_use]
    pub const fn kept(&self) -> Option<&KeptWords> {
        match self {
            Self::Read { words } | Self::NotAllKept { words, .. } => Some(words),
            Self::NotText | Self::NotRead { .. } | Self::TooBig { .. } | Self::NotAFile => None,
        }
    }

    /// The words, if the file had any — every one that was kept.
    #[must_use]
    #[deprecated(
        since = "0.0.2",
        note = "the words are held in one piece now; ask `kept()`, which hands them \
                over without building a list. This still answers, and allocates one \
                `String` per word to do it."
    )]
    pub fn words(&self) -> Vec<String> {
        self.kept().map(KeptWords::to_vec).unwrap_or_default()
    }

    /// Whether the file's kept words hold this one, which is already lower
    /// case. `false` for a file whose words were not all kept says only that
    /// no kept word is this one; [`Contents::all_kept`] says whether that is
    /// the whole of the file.
    #[must_use]
    pub fn say(&self, word: &str) -> bool {
        self.kept().is_some_and(|words| words.says(word))
    }

    /// Whether every word the file had was kept — `false` only for
    /// [`Contents::NotAllKept`]. A file with no words to keep has none left
    /// out.
    #[must_use]
    pub fn all_kept(&self) -> bool {
        !matches!(self, Self::NotAllKept { .. })
    }

    /// The sentence a window shows in place of words, or beside them, in the
    /// language the person reads — or nothing, for a file whose words were
    /// all kept and for something that is not a file.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Option<Said> {
        let (word, filling) = match self {
            Self::Read { .. } | Self::NotAFile => return None,
            Self::NotAllKept { .. } => (
                &words::NOT_ALL_KEPT,
                Filling::of("most", wording::MOST_WORDS.to_string()),
            ),
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

/// Contents as the index file spells them, borrowed, on the way out.
#[derive(Serialize)]
#[serde(tag = "were", rename_all = "kebab-case")]
enum Writing<'a> {
    /// Both kinds of words: `unkept` left out when it is nothing.
    Read {
        /// The kept words, written as the list they have always been written
        /// as — see `KeptWords`'s own `Serialize`, which is why an index file
        /// is byte for byte what it was.
        words: &'a KeptWords,
        /// How many were not kept.
        #[serde(skip_serializing_if = "is_nothing")]
        unkept: usize,
    },
    /// No reader.
    NotText,
    /// Not read, and why.
    NotRead {
        /// What the machine said.
        why: &'a str,
    },
    /// Too big, and how big.
    TooBig {
        /// Its size.
        bytes: u64,
    },
    /// Not a file.
    NotAFile,
}

/// Contents as the index file spells them, on the way in.
#[derive(Deserialize)]
#[serde(tag = "were", rename_all = "kebab-case")]
enum Reading {
    /// Words, with `unkept` nothing when a file from before does not say it.
    Read {
        /// The kept words.
        words: Vec<String>,
        /// How many were not kept.
        #[serde(default)]
        unkept: usize,
    },
    /// No reader.
    NotText,
    /// Not read, and why.
    NotRead {
        /// What the machine said.
        why: String,
    },
    /// Too big, and how big.
    TooBig {
        /// Its size.
        bytes: u64,
    },
    /// Not a file.
    NotAFile,
}

/// Whether a count is nothing, so the field is left out. By reference,
/// because that is how `skip_serializing_if` hands it over.
fn is_nothing(count: &usize) -> bool {
    *count == 0
}

impl Serialize for Contents {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::Read { words } => Writing::Read { words, unkept: 0 },
            Self::NotAllKept { words, unkept } => Writing::Read {
                words,
                unkept: *unkept,
            },
            Self::NotText => Writing::NotText,
            Self::NotRead { why } => Writing::NotRead { why },
            Self::TooBig { bytes } => Writing::TooBig { bytes: *bytes },
            Self::NotAFile => Writing::NotAFile,
        }
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Contents {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(match Reading::deserialize(deserializer)? {
            Reading::Read { words, unkept } => Self::of_words(words, unkept),
            Reading::NotText => Self::NotText,
            Reading::NotRead { why } => Self::NotRead { why },
            Reading::TooBig { bytes } => Self::TooBig { bytes },
            Reading::NotAFile => Self::NotAFile,
        })
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// Contents are spelled with a tag, so a reader can tell them apart
    /// without knowing which fields each has — and words all kept are
    /// spelled exactly as they were before `unkept` existed.
    #[test]
    fn contents_are_tagged_in_the_file() {
        let spelled = |contents: &Contents| serde_json::to_string(contents).unwrap();
        assert_eq!(
            spelled(&Contents::Read {
                words: KeptWords::from(["a"])
            }),
            r#"{"were":"read","words":["a"]}"#
        );
        assert_eq!(
            spelled(&Contents::NotAllKept {
                words: KeptWords::from(["a"]),
                unkept: 3
            }),
            r#"{"were":"read","words":["a"],"unkept":3}"#
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

    /// **Every kind of contents comes back as it went**, and words with
    /// nothing left out come back as [`Contents::Read`] whether the file
    /// said `"unkept":0` or, as a file from before says it, nothing at all.
    #[test]
    fn contents_come_back_as_they_went_and_nothing_unkept_is_all_read() {
        for contents in [
            Contents::Read {
                words: KeptWords::from(["a", "b"]),
            },
            Contents::NotAllKept {
                words: KeptWords::from(["a"]),
                unkept: 7,
            },
            Contents::NotText,
            Contents::NotRead {
                why: "no".to_owned(),
            },
            Contents::TooBig { bytes: 5 },
            Contents::NotAFile,
        ] {
            let back: Contents =
                serde_json::from_str(&serde_json::to_string(&contents).unwrap()).unwrap();
            assert_eq!(back, contents);
        }
        let zero: Contents =
            serde_json::from_str(r#"{"were":"read","words":["a"],"unkept":0}"#).unwrap();
        assert_eq!(
            zero,
            Contents::Read {
                words: KeptWords::from(["a"])
            }
        );
    }

    /// **A count that is not a count is refused**, rather than read as
    /// nothing left out: a negative number, a fraction, a word.
    #[test]
    fn an_unkept_that_is_not_a_count_is_refused() {
        for torn in [
            r#"{"were":"read","words":["a"],"unkept":-1}"#,
            r#"{"were":"read","words":["a"],"unkept":1.5}"#,
            r#"{"were":"read","words":["a"],"unkept":"many"}"#,
            r#"{"were":"read","unkept":2}"#,
        ] {
            assert!(serde_json::from_str::<Contents>(torn).is_err(), "{torn}");
        }
    }

    /// **Words not all kept still answer from what was kept, and say they
    /// are not all there**; every other kind of contents has nothing left
    /// out.
    #[test]
    fn words_not_all_kept_answer_from_what_was_kept_and_say_so() {
        let bounded = Contents::of_words(vec!["alpha".to_owned(), "beta".to_owned()], 2);
        assert!(matches!(bounded, Contents::NotAllKept { unkept: 2, .. }));
        assert!(bounded.say("beta"));
        assert!(!bounded.say("gamma"));
        assert!(!bounded.all_kept());
        let whole = Contents::of_words(vec!["alpha".to_owned()], 0);
        assert!(matches!(whole, Contents::Read { .. }));
        assert!(whole.all_kept());
        assert!(Contents::NotText.all_kept());

        let strings = Strings::of(crate::finding_words().unwrap());
        let said = bounded.said(&strings).unwrap();
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.unfilled().is_empty(), "{said}");
        assert!(said.text().contains("50000"), "{said}");
        assert!(whole.said(&strings).is_none());
    }
}
