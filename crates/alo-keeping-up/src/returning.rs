//! Going back: the one instruction the base is given, decided against the
//! machine as it is at the moment the person approved.
//!
//! `rollback`, and `--apply` after it only when the person asked to restart
//! now. **Nothing else, and nothing that names a build or a path**: the base
//! starts the deployment it kept, and there is no argument by which this could
//! point it anywhere else. What makes the instruction safe to give is decided
//! before it — [`Returning::of`] reads the offer the person approved against
//! the deployments reported *now*, and refuses when they no longer agree.
//!
//! **Why agreement is checked and not assumed.** The base's command reorders
//! what it has; it does not take a target. If the build before changed after
//! the offer, or going back was already set, the same command would start a
//! different build from the one in the sentence the person approved — or turn
//! the machine round again. So each of those is
//! [`CannotGoBack::ChangedSinceItWasOffered`] or
//! [`CannotGoBack::AlreadyGoingBack`], and the base is told nothing.

use alo_strings::{Filling, Said, Strings};

use crate::deployments::Deployments;
use crate::digest::Digest;
use crate::going_back::{CannotGoBack, GoingBack};
use crate::never::Cause;
use crate::when::WhenItApplies;
use crate::words;

/// The base's verb for starting the deployment it kept.
const ROLLBACK: &str = "rollback";

/// The base's flag restarting into it at once.
const APPLY: &str = "--apply";

/// Going back, decided and ready to hand to the base.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Returning {
    /// The build being left.
    from: Digest,
    /// The build returned to.
    to: Digest,
    /// When the person chose it happens.
    when: WhenItApplies,
}

impl Returning {
    /// The instruction for going back as `offer` described it, `when` the
    /// person chose, on a machine whose base reports `deployments` now.
    ///
    /// # Errors
    /// [`CannotGoBack::NotRunningABuild`], [`CannotGoBack::AlreadyGoingBack`],
    /// and [`CannotGoBack::ChangedSinceItWasOffered`] when the build running,
    /// the build kept or the update waiting is not what the offer said.
    pub fn of(
        offer: &GoingBack,
        deployments: &Deployments,
        when: WhenItApplies,
    ) -> Result<Self, CannotGoBack> {
        let running = deployments
            .running()
            .map_err(|_| CannotGoBack::NotRunningABuild)?;
        if deployments.is_going_back() {
            return Err(CannotGoBack::AlreadyGoingBack);
        }
        if running.digest() != offer.from()
            || deployments.rollback() != Some(offer.to())
            || deployments.staged() != offer.sets_aside()
        {
            return Err(CannotGoBack::ChangedSinceItWasOffered);
        }
        Ok(Self {
            from: offer.from().clone(),
            to: offer.to().clone(),
            when,
        })
    }

    /// Every argument handed to the base, in order.
    #[must_use]
    pub fn arguments(&self) -> Vec<String> {
        let mut arguments = vec![ROLLBACK.to_owned()];
        if self.when == WhenItApplies::NowBecauseThePersonAsked {
            arguments.push(APPLY.to_owned());
        }
        arguments
    }

    /// The build being left.
    #[must_use]
    pub fn from(&self) -> &Digest {
        &self.from
    }

    /// The build returned to.
    #[must_use]
    pub fn to(&self) -> &Digest {
        &self.to
    }

    /// When it happens.
    #[must_use]
    pub fn when(&self) -> WhenItApplies {
        self.when
    }

    /// Who causes the restart that returns the machine: always the person.
    #[must_use]
    pub fn cause_of_the_restart(&self) -> Cause {
        self.when.cause_of_the_restart()
    }

    /// What a person is told once the base has set it.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&words::GOING_BACK_AT_THE_RESTART.key(), &Filling::nothing())
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use super::*;
    use crate::before::Changed;
    use crate::testing::{in_english, whole};

    fn after_an_update() -> Deployments {
        Deployments::reported(Some(whole("bb")), None, Some(whole("aa")))
    }

    fn the_offer() -> GoingBack {
        GoingBack::offered(
            &after_an_update(),
            Some(&Changed::between(whole("aa"), whole("bb"))),
        )
        .unwrap()
    }

    /// **At the next restart the instruction is one word, and it names no build
    /// and no path.**
    #[test]
    fn at_the_next_restart_the_base_is_told_to_go_back_and_nothing_else() {
        let returning = Returning::of(
            &the_offer(),
            &after_an_update(),
            WhenItApplies::AtTheNextRestart,
        )
        .unwrap();
        assert_eq!(returning.arguments(), ["rollback".to_owned()]);
        assert_eq!(returning.from(), &whole("bb"));
        assert_eq!(returning.to(), &whole("aa"));
        assert_eq!(returning.cause_of_the_restart(), Cause::ThePerson);
        assert!(!returning.said(&in_english()).is_a_bug());
    }

    /// **Restarting now is the one choice that adds `--apply`, and it is the
    /// person's.**
    #[test]
    fn restarting_now_adds_apply_and_nothing_else() {
        let returning = Returning::of(
            &the_offer(),
            &after_an_update(),
            WhenItApplies::NowBecauseThePersonAsked,
        )
        .unwrap();
        assert_eq!(
            returning.arguments(),
            ["rollback".to_owned(), "--apply".to_owned()]
        );
        for when in WhenItApplies::EVERY {
            let arguments = Returning::of(&the_offer(), &after_an_update(), when)
                .unwrap()
                .arguments();
            assert!(
                arguments
                    .iter()
                    .all(|a| !a.contains('/') && !a.contains("sha256"))
            );
        }
    }

    /// **A machine that changed since the offer is refused**: another build
    /// running, another build kept, an update that started waiting, or the
    /// build before gone.
    #[test]
    fn a_machine_that_changed_since_the_offer_is_refused() {
        for changed in [
            Deployments::reported(Some(whole("cc")), None, Some(whole("aa"))),
            Deployments::reported(Some(whole("bb")), None, Some(whole("cc"))),
            Deployments::reported(Some(whole("bb")), Some(whole("dd")), Some(whole("aa"))),
            Deployments::reported(Some(whole("bb")), None, None),
        ] {
            let refused =
                Returning::of(&the_offer(), &changed, WhenItApplies::AtTheNextRestart).unwrap_err();
            assert_eq!(
                refused,
                CannotGoBack::ChangedSinceItWasOffered,
                "{changed:?}"
            );
            let said = refused.said(&in_english());
            assert!(!said.is_a_bug(), "{said}");
            assert!(said.text().contains("nothing was changed"), "{said}");
        }
    }

    /// **Going back already set is refused**, so one approval is one
    /// execution and the machine is never turned round again.
    #[test]
    fn going_back_already_set_is_not_set_a_second_time() {
        let refused = Returning::of(
            &the_offer(),
            &after_an_update().going_back_at_the_next_restart(),
            WhenItApplies::AtTheNextRestart,
        )
        .unwrap_err();
        assert_eq!(refused, CannotGoBack::AlreadyGoingBack);
    }

    /// **A machine whose running build cannot be named is refused.**
    #[test]
    fn a_machine_running_no_build_is_not_told_to_go_back() {
        let refused = Returning::of(
            &the_offer(),
            &Deployments::reported(None, None, Some(whole("aa"))),
            WhenItApplies::AtTheNextRestart,
        )
        .unwrap_err();
        assert_eq!(refused, CannotGoBack::NotRunningABuild);
    }
}
