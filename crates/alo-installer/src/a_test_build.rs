//! Whether this programme is a release or a test build, and the one line that
//! says so.
//!
//! [ADR 0096](../../../docs/decisions/0096-a-workflow-may-publish-a-candidate-and-a-candidate-announces-itself.md)
//! control five: **the programme announces itself, with the date it was
//! built**, on its first line on screen, compiled in by the candidate workflow
//! the way `ALO_INSTALLER_ENVIRONMENT_SHA256` already is.
//!
//! # The person this protects is not looking at the release page
//!
//! Every other control a candidate carries lives on the page it was downloaded
//! from. Deleting a pre-release removes that page and not the file on somebody's
//! disk, so a person who kept a candidate and double-clicks it next month has
//! no page to consult and no network. **A filename does not reach them** —
//! nobody reads a filename and a zip gets extracted. The notes do not reach
//! them — that is the page they left.
//!
//! One line on screen does, and the date in it is what makes a kept candidate
//! say how old it is without anything to look up.
//!
//! # It informs and deliberately does not refuse
//!
//! A test build that stopped working after some number of days could bite
//! mid-walk, and that failure would look exactly like the loader failure this
//! whole road exists to eliminate: a programme that prints nothing and does not
//! start. It would also be the installer deciding for the person, which the
//! fifth law forbids — *every protection is a default they can change, not a
//! wall*. Telling somebody plainly what they are holding restricts nothing.
//!
//! # Why the date is written as the machine wrote it
//!
//! ADR 0096 illustrates the sentence with *7 October 2026*. The date is filled
//! in here as `2026-10-07`, which is the form this crate already instructs its
//! translators to use: `words::HOW_MUCH_AND_HOW_RECENT`'s own note says
//! *{newest} a date as year-month-day, for example 2026-09-28*. Month names in
//! prose would contradict a written instruction one file away, and would mean
//! translating twelve month names into twenty-four languages for one sentence.
//! *In the person's own language* governs the sentence; the date is a filled
//! gap.

use alo_strings::{Filling, Word};

use crate::environment::THE_CANDIDATE_BUILT;
use crate::what_replacing_destroys::Day;
use crate::words;

/// What kind of programme this is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThisBuild {
    /// A release. It says nothing about itself, because every release is one.
    ARelease,
    /// A test build, made on this day.
    ATestBuildFrom(Day),
    /// A test build whose day is not a day this can read.
    ///
    /// **The warning survives a broken date, and that is the whole of why this
    /// variant exists.** *It is a test build* is the half that protects
    /// somebody; the day is the improvement. A single sentence with a bad gap
    /// would either put whatever was compiled in front of a person, or be
    /// suppressed and take the warning with it.
    ATestBuild,
}

/// What was built, from what the workflow compiled in.
///
/// Separated from [`this_build`] so the other branches can be reached at all:
/// `option_env!` is a compile-time value, so a test binary has exactly one of
/// these and the rest would never be exercised. The same reason
/// `crate::fast_startup::FastStartup::read` takes what the machine printed.
#[must_use]
pub fn what_was_built(built: Option<&str>) -> ThisBuild {
    match built {
        None => ThisBuild::ARelease,
        Some(built) => Day::of(built).map_or(ThisBuild::ATestBuild, ThisBuild::ATestBuildFrom),
    }
}

/// What this programme is, as the workflow that built it said.
#[must_use]
pub fn this_build() -> ThisBuild {
    what_was_built(THE_CANDIDATE_BUILT)
}

impl ThisBuild {
    /// The sentence this is said in and what fills it, or [`None`] for a
    /// release, which says nothing.
    #[must_use]
    pub fn said_as(&self) -> Option<(Word, Filling)> {
        match self {
            Self::ARelease => None,
            Self::ATestBuildFrom(day) => {
                Some((words::A_TEST_BUILD_FROM, Filling::of("day", day.as_str())))
            }
            Self::ATestBuild => Some((words::A_TEST_BUILD, Filling::nothing())),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]
mod tests {
    use alo_strings::Strings;

    use super::*;

    /// This installer's own words.
    fn source() -> Strings {
        Strings::of(words::installer_words().unwrap())
    }

    /// **A release says nothing about being one.**
    ///
    /// Nothing compiled in is the ordinary case and the common one: every
    /// release, every local build, and every test binary. A programme that
    /// announced itself as a release would be adding a line to the one road
    /// that does not need it.
    #[test]
    fn a_release_says_nothing_about_itself() {
        assert_eq!(what_was_built(None), ThisBuild::ARelease);
        assert_eq!(what_was_built(None).said_as(), None);
    }

    /// **A test build says so, with the day it was built.**
    #[test]
    fn a_test_build_says_so_with_its_day() {
        let built = what_was_built(Some("2026-10-07"));
        assert_eq!(
            built,
            ThisBuild::ATestBuildFrom(Day::of("2026-10-07").unwrap())
        );

        let (word, filling) = built.said_as().unwrap();
        let said = source().say(&word.key(), &filling).into_text();
        assert!(said.contains("2026-10-07"), "{said}");
        assert!(said.contains("not a release"), "{said}");
    }

    /// **A day that is not a day loses the day and keeps the warning.**
    ///
    /// Anything the workflow could compile in that `Day` refuses: an empty
    /// value, a time, a month that is not a month, a date in somebody else's
    /// order. Every one of them still tells the person what they are holding,
    /// which is the half that matters.
    #[test]
    fn a_day_that_is_not_a_day_still_warns() {
        for built in [
            "",
            "   ",
            "today",
            "2026-10-07T01:02:03Z",
            "2026-13-01",
            "2026-10-32",
            "07-10-2026",
            "20261007",
        ] {
            assert_eq!(
                what_was_built(Some(built)),
                ThisBuild::ATestBuild,
                "{built:?} was read as a day"
            );
            let (word, filling) = what_was_built(Some(built)).said_as().unwrap();
            let said = source().say(&word.key(), &filling).into_text();
            assert!(
                said.contains("not a release"),
                "{built:?} lost the warning: {said}"
            );
        }
    }

    /// **Neither sentence leaves a gap unfilled**, which for the dated one is
    /// the thing that would put `{day}` in front of a person.
    #[test]
    fn both_sentences_are_said_whole() {
        for built in [Some("2026-10-07"), Some("not a day")] {
            let (word, filling) = what_was_built(built).said_as().unwrap();
            let said = source().say(&word.key(), &filling).into_text();
            assert!(!said.contains('{'), "{built:?} left a gap: {said}");
            assert!(!said.contains('}'), "{built:?} left a gap: {said}");
        }
    }
}
