//! **The power profiles, which are the rented daemon's and not ours.**
//!
//! Three names, a closed set, and alo OS invents none of them: *saver*,
//! *balanced*, *performance* are what the daemon every Linux machine runs has,
//! and a fourth of our own would be a profile no driver implements.
//!
//! # A machine that has two has two
//!
//! The daemon reports which profiles this machine's hardware can actually do.
//! Where *performance* is not among them it is **absent**, not greyed out: a
//! switch a person cannot move is a thing to wonder about every time they open
//! the page, and *why is this grey* is a question the machine has no answer to.

use serde::{Deserialize, Serialize};

use crate::words::{self, Word};

/// **A power profile**, as the rented daemon names them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Profile {
    /// Quieter, cooler, slower.
    Saver,
    /// What a machine does when nobody has said otherwise.
    Balanced,
    /// Everything it has, for as long as it lasts.
    Performance,
}

impl Profile {
    /// All three, in the order a person meets them.
    pub const EVERY: [Self; 3] = [Self::Saver, Self::Balanced, Self::Performance];

    /// What the daemon calls this one.
    #[must_use]
    pub const fn named(self) -> &'static str {
        match self {
            Self::Saver => "power-saver",
            Self::Balanced => "balanced",
            Self::Performance => "performance",
        }
    }

    /// The profile the daemon means by this name, or [`None`] for a name no
    /// profile has — which this crate passes over rather than guessing at.
    #[must_use]
    pub fn by_name(named: &str) -> Option<Self> {
        Self::EVERY
            .into_iter()
            .find(|profile| profile.named() == named.trim())
    }

    /// What a person reads.
    #[must_use]
    pub const fn word(self) -> Word {
        match self {
            Self::Saver => words::PROFILE_SAVER,
            Self::Balanced => words::PROFILE_BALANCED,
            Self::Performance => words::PROFILE_PERFORMANCE,
        }
    }
}

/// **What this machine can actually do**, and which is on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheProfiles {
    /// The ones the daemon says this machine has, in its order.
    offered: Vec<Profile>,
    /// The one it is in.
    on: Profile,
}

impl TheProfiles {
    /// What the daemon reported.
    ///
    /// # Errors
    /// [`NotAProfile::NoneOffered`] where the daemon offered none at all: a
    /// machine with no profiles is a machine this page has nothing to show, and
    /// an empty list would be a page with three switches that do nothing.
    pub fn reported(offered: Vec<Profile>, on: Profile) -> Result<Self, NotAProfile> {
        if offered.is_empty() {
            return Err(NotAProfile::NoneOffered);
        }
        Ok(Self { offered, on })
    }

    /// The ones this machine has.
    #[must_use]
    pub fn offered(&self) -> &[Profile] {
        &self.offered
    }

    /// Whether this machine has this one at all.
    #[must_use]
    pub fn offers(&self, profile: Profile) -> bool {
        self.offered.contains(&profile)
    }

    /// The one it is in.
    #[must_use]
    pub const fn on(&self) -> Profile {
        self.on
    }

    /// **Whether a person may choose this one here**, which they may not when
    /// this machine's hardware does not have it.
    ///
    /// # Errors
    /// [`NotAProfile::NotOnThisMachine`], which a surface reads as *do not show
    /// it* rather than *show it and refuse*.
    pub fn may_choose(&self, profile: Profile) -> Result<(), NotAProfile> {
        if self.offers(profile) {
            return Ok(());
        }
        Err(NotAProfile::NotOnThisMachine { profile })
    }
}

/// Why a profile is not one this machine has.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NotAProfile {
    /// This machine's hardware does not do it.
    #[error("this machine does not have the {profile:?} profile, so it is not shown at all")]
    NotOnThisMachine {
        /// Which one.
        profile: Profile,
    },
    /// The daemon offered none.
    #[error("this machine's power daemon offered no profiles at all")]
    NoneOffered,
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;

    /// **The names are the daemon's**, read back and forth without a spelling
    /// of our own.
    #[test]
    fn the_names_are_the_daemons_own() {
        for profile in Profile::EVERY {
            assert_eq!(Profile::by_name(profile.named()), Some(profile));
        }
        assert_eq!(Profile::by_name("power-saver"), Some(Profile::Saver));
        assert_eq!(Profile::by_name(" balanced\n"), Some(Profile::Balanced));
        assert_eq!(
            Profile::by_name("quiet"),
            None,
            "a name nothing has is not guessed at"
        );
    }

    /// **A machine that has two has two**, and the third is absent rather than
    /// shown and refused.
    #[test]
    fn a_profile_this_machine_does_not_have_is_not_offered_at_all() {
        let two = TheProfiles::reported(vec![Profile::Saver, Profile::Balanced], Profile::Balanced)
            .expect("two profiles");
        assert!(two.offers(Profile::Saver));
        assert!(!two.offers(Profile::Performance));
        assert_eq!(
            two.may_choose(Profile::Performance),
            Err(NotAProfile::NotOnThisMachine {
                profile: Profile::Performance
            })
        );
        assert!(two.may_choose(Profile::Saver).is_ok());
        assert_eq!(two.on(), Profile::Balanced);
    }

    /// **A daemon offering nothing is a refusal**, not a page of switches that
    /// do nothing.
    #[test]
    fn a_machine_with_no_profiles_says_so() {
        assert_eq!(
            TheProfiles::reported(Vec::new(), Profile::Balanced),
            Err(NotAProfile::NoneOffered)
        );
    }
}
