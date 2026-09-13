//! One number, and where it came from — or why there is no number.
//!
//! The promise this crate keeps is the word *plain*: a number a person can
//! check against the machine. That is only possible if the number says where
//! it was read, so a [`Source`] travels with every one: the file under `/proc`
//! and the line or field in it. `cat` the file, find the line, and it is the
//! same number.
//!
//! # A number that is not there is not a zero
//!
//! The kernel does not always give one. It withholds a file that belongs to
//! another person's process; it keeps no resident memory for a thread of its
//! own; and a process that began after the earlier of two readings has nothing
//! to be a rate over. Each of those written as `0` would be the machine
//! deriving a number and hoping it is close, which is the one thing this crate
//! is for not doing. [`Number`] says which it is, and each has a sentence in
//! the reader's language for the window to show in the number's place.

use std::path::{Path, PathBuf};

use alo_strings::{Filling, Said, Strings};

/// Where a number was read: the file, and the line or field in it.
///
/// The file is a path under `/proc`; the field is the kernel's own name for
/// the line — `VmRSS`, `rchar` — or, where a number is made from two lines,
/// both names. A person reading the file finds the number under that name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    /// The file the number is in.
    pub file: PathBuf,
    /// The line or field of that file it is on, by the kernel's own name.
    pub field: &'static str,
}

impl Source {
    /// A source, from the file and the field.
    #[must_use]
    pub fn of(file: impl Into<PathBuf>, field: &'static str) -> Self {
        Self {
            file: file.into(),
            field,
        }
    }

    /// The file the number is in.
    #[must_use]
    pub fn file(&self) -> &Path {
        &self.file
    }
}

/// A number the kernel gave, and where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Known {
    /// The number.
    pub value: u64,
    /// Where it was read.
    pub from: Source,
}

/// One number a window shows — or the reason it shows a sentence instead.
///
/// Every arm carries its [`Source`], including the ones with no number in
/// them: *withheld* names the file the kernel would not open, and *not said*
/// names the file that has no such line, so that what is missing can be
/// checked as carefully as what is there.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Number {
    /// The kernel gave a number.
    Known(Known),

    /// The kernel keeps this number and would not show it.
    ///
    /// Another person's process, usually: `/proc/<pid>/io` is readable only
    /// by the process's owner or by someone allowed to trace it. Never a
    /// zero, because the process may be doing a great deal.
    Withheld {
        /// The file the kernel would not open.
        from: Source,
        /// What the operating system said when asked, as a sentence.
        why: String,
    },

    /// The kernel's file has no such line for this process.
    ///
    /// A thread of the kernel's own has no `VmRSS` because it has no memory
    /// of its own to count. Not a fault, and not a zero either: the kernel
    /// did not say.
    NotSaid {
        /// The file without the line.
        from: Source,
    },

    /// A rate that has no earlier reading to be a rate over.
    ///
    /// The process began after the earlier of two readings. The next pair
    /// will have a number. Never in a [`crate::Reading`], which holds totals;
    /// only in a [`crate::Running`], which holds rates.
    NotYet {
        /// The file the rate will be read from.
        from: Source,
    },
}

impl Number {
    /// A known number, from a file and a field.
    #[must_use]
    pub fn known(value: u64, from: Source) -> Self {
        Self::Known(Known { value, from })
    }

    /// The number, if the kernel gave one.
    #[must_use]
    pub fn value(&self) -> Option<u64> {
        match self {
            Self::Known(known) => Some(known.value),
            Self::Withheld { .. } | Self::NotSaid { .. } | Self::NotYet { .. } => None,
        }
    }

    /// Where the number was read, or would have been.
    #[must_use]
    pub fn from(&self) -> &Source {
        match self {
            Self::Known(known) => &known.from,
            Self::Withheld { from, .. } | Self::NotSaid { from } | Self::NotYet { from } => from,
        }
    }

    /// What a window shows in place of the number, in the reader's language.
    ///
    /// [`None`] for a known number: a number is shown as itself and needs no
    /// sentence. The other three each have one, declared in [`crate::words`].
    #[must_use]
    pub fn instead(&self, strings: &Strings) -> Option<Said> {
        let word = match self {
            Self::Known(_) => return None,
            Self::Withheld { .. } => &crate::words::WITHHELD,
            Self::NotSaid { .. } => &crate::words::NOT_SAID,
            Self::NotYet { .. } => &crate::words::NOT_YET,
        };
        Some(strings.say(&word.key(), &Filling::nothing()))
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// English, with nothing translated.
    fn in_english() -> Strings {
        Strings::of(crate::measuring_words().unwrap())
    }

    /// A source for these tests.
    fn a_source() -> Source {
        Source::of("/proc/42/io", "rchar")
    }

    /// **A known number is a number and needs no sentence**; each of the
    /// other three is a sentence and has no number. Nothing here is ever both,
    /// and nothing is ever a zero standing in for a sentence.
    #[test]
    fn a_number_is_a_value_or_a_sentence_and_never_a_zero_standing_in() {
        let known = Number::known(7, a_source());
        assert_eq!(known.value(), Some(7));
        assert!(known.instead(&in_english()).is_none());

        let withheld = Number::Withheld {
            from: a_source(),
            why: "permission denied".to_owned(),
        };
        let not_said = Number::NotSaid { from: a_source() };
        let not_yet = Number::NotYet { from: a_source() };
        for (number, says) in [
            (&withheld, "The kernel did not show this."),
            (&not_said, "Not counted for this process."),
            (&not_yet, "Started since the last reading."),
        ] {
            assert_eq!(number.value(), None);
            let said = number.instead(&in_english()).unwrap();
            assert_eq!(said.text(), says);
            assert!(!said.is_a_bug(), "{said}");
        }
    }

    /// **Every arm names its file**, so what is missing is checkable too.
    #[test]
    fn every_number_names_the_file_it_came_from_even_when_there_is_no_number() {
        let from = a_source();
        for number in [
            Number::known(1, from.clone()),
            Number::Withheld {
                from: from.clone(),
                why: String::new(),
            },
            Number::NotSaid { from: from.clone() },
            Number::NotYet { from: from.clone() },
        ] {
            assert_eq!(number.from(), &from);
            assert_eq!(number.from().file(), Path::new("/proc/42/io"));
            assert_eq!(number.from().field, "rchar");
        }
    }
}
