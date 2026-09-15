//! Going back to the build before: whether it can be offered, and the sentence
//! a person approves when it is.
//!
//! The base keeps the previous build and starts it again with one command (ADR
//! 0011); that is the inheritance. What is ours is everything that makes it a
//! promise rather than a command nobody has heard of — above all that **going
//! back that cannot be done says so before it is offered**, rather than being
//! offered and failing halfway. [`GoingBack::offered`] is that decision, and
//! each [`CannotGoBack`] is a sentence in the vocabulary.
//!
//! # What the person approves
//!
//! [`GoingBack::said`], and the sentence is honest about the one thing the base
//! does not carry back. The person's files and their own settings live under
//! `/var` and are not part of any build, so going back leaves them exactly as
//! they are. **The machine's own configuration under `/etc` is kept per build**:
//! the base gives each new build a copy merged from the one before, and going
//! back starts the earlier build with the earlier copy (`docs/quirks.md`). So
//! an account made, or a password changed, since the update does not come back
//! with it, and the sentence says so — a person who learned it afterwards would
//! have approved something they were not told.
//!
//! When an update is already waiting for the next restart, the base sets it
//! aside to go back. That is said too, in its own sentence, because it undoes a
//! choice the person made.

use alo_strings::{Filling, Said, Strings};

use crate::before::{Before, Changed};
use crate::deployments::Deployments;
use crate::digest::Digest;
use crate::words::{self, Word};

/// Going back to the build before, decided and ready to offer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoingBack {
    /// The build before, and the build running now.
    before: Before,
    /// An update waiting for the next restart, which going back sets aside.
    sets_aside: Option<Digest>,
}

/// Why going back is not offered — each decided before anything runs, and
/// each said to the person instead of an offer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CannotGoBack {
    /// The base reports no build this machine booted from an image.
    NotRunningABuild,
    /// Nothing is known to have come before the build running.
    NothingBefore,
    /// The build before is named, and the base no longer keeps it.
    NoLongerKept {
        /// The build that is gone.
        build: Digest,
    },
    /// Going back is already set for the next restart; asking again would be a
    /// second execution of one approval.
    AlreadyGoingBack,
    /// The machine is not as it was when going back was offered — another
    /// build runs, the build before changed, or an update started waiting — so
    /// what the person approved is not what would happen.
    ChangedSinceItWasOffered,
}

impl GoingBack {
    /// Whether going back can be offered on a machine whose base reports
    /// `deployments`, and whose record last says `last_changed`.
    ///
    /// # Errors
    /// [`CannotGoBack`], every one of them before anything is offered.
    pub fn offered(
        deployments: &Deployments,
        last_changed: Option<&Changed>,
    ) -> Result<Self, CannotGoBack> {
        let before = Before::of(deployments, last_changed)
            .map_err(|_| CannotGoBack::NotRunningABuild)?
            .ok_or(CannotGoBack::NothingBefore)?;
        if !before.is_kept() {
            return Err(CannotGoBack::NoLongerKept {
                build: before.build().clone(),
            });
        }
        if deployments.is_going_back() {
            return Err(CannotGoBack::AlreadyGoingBack);
        }
        Ok(Self {
            sets_aside: deployments.staged().cloned(),
            before,
        })
    }

    /// The build before, which going back returns to.
    #[must_use]
    pub fn before(&self) -> &Before {
        &self.before
    }

    /// The build running now, which going back leaves.
    #[must_use]
    pub fn from(&self) -> &Digest {
        self.before.running()
    }

    /// The build going back returns to.
    #[must_use]
    pub fn to(&self) -> &Digest {
        self.before.build()
    }

    /// The update waiting for the next restart that going back sets aside, if
    /// there is one.
    #[must_use]
    pub fn sets_aside(&self) -> Option<&Digest> {
        self.sets_aside.as_ref()
    }

    /// The sentence the person approves.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let word = if self.sets_aside.is_some() {
            words::GOING_BACK_SETS_ASIDE_AN_UPDATE
        } else {
            words::GOING_BACK_OFFERED
        };
        strings.say(&word.key(), &Filling::nothing())
    }

    /// The words a person chooses when it happens by.
    #[must_use]
    pub fn when_word(when: crate::WhenItApplies) -> Word {
        match when {
            crate::WhenItApplies::AtTheNextRestart => words::GO_BACK_AT_THE_NEXT_RESTART,
            crate::WhenItApplies::NowBecauseThePersonAsked => words::RESTART_AND_GO_BACK_NOW,
        }
    }
}

impl CannotGoBack {
    /// What a person reads instead of an offer, in the vocabulary.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let word = match self {
            Self::NotRunningABuild => words::RUNNING_NOT_KNOWN,
            Self::NothingBefore => words::NOTHING_TO_GO_BACK_TO,
            Self::NoLongerKept { .. } => words::NO_LONGER_KEPT,
            Self::AlreadyGoingBack => words::ALREADY_GOING_BACK,
            Self::ChangedSinceItWasOffered => words::CHANGED_SINCE_GOING_BACK_WAS_OFFERED,
        };
        strings.say(&word.key(), &Filling::nothing())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::testing::{in_english, whole};

    fn after_an_update() -> Deployments {
        Deployments::reported(Some(whole("bb")), None, Some(whole("aa")))
    }

    fn the_update() -> Changed {
        Changed::between(whole("aa"), whole("bb"))
    }

    /// **After an update, going back is offered to the build before**, in a
    /// sentence that names what stays and what does not.
    #[test]
    fn after_an_update_going_back_to_the_build_before_is_offered() {
        let offer = GoingBack::offered(&after_an_update(), Some(&the_update())).unwrap();
        assert_eq!(offer.from(), &whole("bb"));
        assert_eq!(offer.to(), &whole("aa"));
        assert_eq!(offer.sets_aside(), None);
        let said = offer.said(&in_english());
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("Your files"), "{said}");
        assert!(said.text().contains("passwords"), "{said}");
    }

    /// **An update waiting is set aside by going back, and the sentence says
    /// so**, because it undoes something the person chose.
    #[test]
    fn going_back_with_an_update_waiting_says_the_update_will_not_apply() {
        let deployments =
            Deployments::reported(Some(whole("bb")), Some(whole("cc")), Some(whole("aa")));
        let offer = GoingBack::offered(&deployments, Some(&the_update())).unwrap();
        assert_eq!(offer.sets_aside(), Some(&whole("cc")));
        let said = offer.said(&in_english());
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("will not apply"), "{said}");
    }

    /// **A machine with nothing before is told so, and offered nothing.**
    #[test]
    fn a_machine_with_nothing_before_is_not_offered_going_back() {
        let refused =
            GoingBack::offered(&Deployments::reported(Some(whole("aa")), None, None), None)
                .unwrap_err();
        assert_eq!(refused, CannotGoBack::NothingBefore);
        let said = refused.said(&in_english());
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("no earlier version"), "{said}");
    }

    /// **A build before that is no longer on the disk is named as gone, before
    /// anything is offered** — never offered and then failing halfway.
    #[test]
    fn a_build_before_that_is_no_longer_kept_is_refused_before_it_is_offered() {
        let refused = GoingBack::offered(
            &Deployments::reported(Some(whole("bb")), None, None),
            Some(&the_update()),
        )
        .unwrap_err();
        assert_eq!(refused, CannotGoBack::NoLongerKept { build: whole("aa") });
        let said = refused.said(&in_english());
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("no longer kept"), "{said}");
    }

    /// **Going back already set is not offered again**: the base would turn it
    /// round, and one approval is one execution.
    #[test]
    fn going_back_already_set_for_the_restart_is_not_offered_again() {
        let refused = GoingBack::offered(
            &after_an_update().going_back_at_the_next_restart(),
            Some(&the_update()),
        )
        .unwrap_err();
        assert_eq!(refused, CannotGoBack::AlreadyGoingBack);
        assert!(!refused.said(&in_english()).is_a_bug());
    }

    /// **A machine running no nameable build is offered nothing.**
    #[test]
    fn a_machine_running_no_build_is_not_offered_going_back() {
        let refused = GoingBack::offered(
            &Deployments::reported(None, None, Some(whole("aa"))),
            Some(&the_update()),
        )
        .unwrap_err();
        assert_eq!(refused, CannotGoBack::NotRunningABuild);
        assert!(!refused.said(&in_english()).is_a_bug());
    }

    /// Each choice of when has going back's own words, not an update's.
    #[test]
    fn each_choice_of_when_has_going_backs_own_words() {
        for when in crate::WhenItApplies::EVERY {
            let word = GoingBack::when_word(when);
            assert_ne!(word.named(), when.word().named());
            assert!(
                word.says().to_lowercase().contains("go back"),
                "{}",
                word.says()
            );
        }
    }
}
