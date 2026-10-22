//! Every refusal, said to the person who asked.
//!
//! One `match` with no wildcard arm, so a variant added to
//! [`crate::NotGoingBack`] without a sentence does not compile. That is the
//! whole reason this is a file of its own rather than an `impl` beside the
//! enum: the mapping is the thing worth protecting, and a reader checking that
//! nothing can be refused silently should have one place to look.
//!
//! # A path is filled in, never translated
//!
//! Every fill here is a path on this machine or a sentence from a program, and
//! each one's note in [`crate::words`] says so, because a translator who does
//! not know that will translate it.

use alo_strings::{Filling, Said, Strings};

use crate::putting_back::NotGoingBack;
use crate::words;

impl NotGoingBack {
    /// What the person who asked is told.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let (word, filling) = match self {
            Self::NotInTheirHome { folder, .. } => (
                words::NOT_IN_YOUR_HOME,
                Filling::of("folder", folder.display().to_string()),
            ),
            Self::TheWholeHome { .. } => (words::THE_WHOLE_HOME, Filling::nothing()),
            Self::NotAFolder { folder } => (
                words::NOT_A_FOLDER,
                Filling::of("folder", folder.display().to_string()),
            ),
            Self::NotInThatPlace { folder } => (
                words::NOT_IN_THAT_PLACE,
                Filling::of("folder", folder.display().to_string()),
            ),
            Self::ALinkRatherThanAFolder { at } => (
                words::A_LINK_RATHER_THAN_A_FOLDER,
                Filling::of("at", at.display().to_string()),
            ),
            Self::OneIsHalfDone { at } => (
                words::ONE_IS_HALF_DONE,
                Filling::of("at", at.display().to_string()),
            ),
            Self::NowhereToSetAsideWhatIsThere { at } => (
                words::NOWHERE_TO_SET_ASIDE,
                Filling::of("at", at.display().to_string()),
            ),
            Self::NotCopied { why } => (words::NOT_COPIED, Filling::of("why", why.clone())),
            Self::NotSwapped { why } => (words::NOT_SWAPPED, Filling::of("why", why.clone())),
            Self::SetAsideUnderTheWorkingName { folder, at, why } => (
                words::SET_ASIDE_UNDER_THE_WORKING_NAME,
                Filling::of("folder", folder.display().to_string())
                    .and("at", at.display().to_string())
                    .and("why", why.clone()),
            ),
        };
        strings.say(&word.key(), &filling)
    }
}

impl NotGoingBack {
    /// Whether the folder is as it was despite this.
    ///
    /// True for exactly one refusal:
    /// [`NotGoingBack::SetAsideUnderTheWorkingName`], where the swap happened
    /// and only the last rename did not. A caller that reported every `Err` as
    /// *nothing changed* would be telling a person the opposite of what their
    /// disk says — which is the shape of mistake the gate's *the refusal path
    /// tested as carefully as the happy path* is about.
    #[must_use]
    pub const fn the_folder_went_back(&self) -> bool {
        matches!(self, Self::SetAsideUnderTheWorkingName { .. })
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected Err is the failure being reported"
)]
#[path = "said_tests.rs"]
mod tests;
