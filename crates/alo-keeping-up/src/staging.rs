//! Applying an update: the one instruction the base is given, decided as a
//! value before anything runs.
//!
//! The mechanism is the base's (ADR 0011): a new build is staged beside the one
//! running and the machine boots into it at the next restart. **What is ours is
//! what that instruction may say**, and [`Staging`] is all of it — so what the
//! doing crate hands a program is a list this file wrote and a test read, and
//! never text assembled where it runs.
//!
//! # What the instruction is
//!
//! `switch --enforce-container-sigpolicy <source>@<digest>`, and `--apply`
//! after it only when the person asked to restart now:
//!
//! - **`switch` to a digest, never `upgrade` to a tag.** The build staged is
//!   exactly the one the offer named. A tag is resolved wherever it is fetched,
//!   and by then it may name a different build from the one the person was told
//!   is ready.
//! - **`--enforce-container-sigpolicy` always.** Whether a build may be staged
//!   at all is the machine's signature policy's answer (ADR 0036), and the base
//!   refuses to stage anything under a policy that would accept an unsigned
//!   build. There is no instruction here without it, so no choice of the
//!   person's and no offer can turn it off.
//! - **Nothing names a path.** The instruction has no argument that reaches
//!   `/var`, `/etc` or a person's folder; what survives an update is the base's
//!   promise about a deployment, and the test in a virtual machine is what holds
//!   it to that promise for the files alo OS keeps.
//! - **`--apply` only for [`WhenItApplies::NowBecauseThePersonAsked`].**
//!   Without it the build waits for a restart the person makes, which is
//!   [`THE_RULE`](crate::THE_RULE) as an argument list.
//!
//! # What is refused before anything runs
//!
//! [`NotStaged`]: the machine is not running a build this can name; it is no
//! longer running the build the offer was compared against — something
//! changed since the person was told, so what they chose is not this; or the
//! offered build is already waiting, so a second approval would be a second
//! execution of one change.

use alo_strings::{Filling, Said, Strings};

use crate::deployments::Deployments;
use crate::digest::Digest;
use crate::never::Cause;
use crate::source::Source;
use crate::standing::Ready;
use crate::when::WhenItApplies;
use crate::words;

/// The base's verb for staging a build by reference.
const SWITCH: &str = "switch";

/// The base's flag refusing to stage under a policy that accepts anything.
const ENFORCE_THE_SIGNATURE_POLICY: &str = "--enforce-container-sigpolicy";

/// The base's flag restarting into the staged build at once.
const APPLY: &str = "--apply";

/// An update, decided and ready to hand to the base.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Staging {
    /// The build running when this was decided.
    from: Digest,
    /// The build to stage.
    to: Digest,
    /// The build, as the base fetches it.
    reference: String,
    /// When the person chose it applies.
    when: WhenItApplies,
}

/// Why an update was not staged, before anything ran.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NotStaged {
    /// The base reports no build this machine booted from an image.
    NotRunningABuild,
    /// The machine is running a different build from the one the update was
    /// found against.
    TheMachineMovedOn {
        /// The build the offer was compared against.
        expected: Digest,
        /// The build the base reports running now.
        running: Digest,
    },
    /// The offered build is already staged for the next restart.
    AlreadyWaiting,
}

impl Staging {
    /// The instruction staging `ready`'s offered build from `source`, applied
    /// `when` the person chose.
    ///
    /// `deployments` is what the base reports **now**, read after the person
    /// chose and before this is decided, so the refusals below are about the
    /// machine as it is rather than as it was when the update was found.
    ///
    /// # Errors
    /// [`NotStaged`].
    pub fn of(
        ready: &Ready,
        deployments: &Deployments,
        source: &Source,
        when: WhenItApplies,
    ) -> Result<Self, NotStaged> {
        let running = deployments
            .running()
            .map_err(|_| NotStaged::NotRunningABuild)?;
        if running.digest() != ready.running() {
            return Err(NotStaged::TheMachineMovedOn {
                expected: ready.running().clone(),
                running: running.digest().clone(),
            });
        }
        if deployments.staged() == Some(ready.offered()) {
            return Err(NotStaged::AlreadyWaiting);
        }
        Ok(Self {
            from: ready.running().clone(),
            to: ready.offered().clone(),
            reference: source.at(ready.offered()),
            when,
        })
    }

    /// Every argument handed to the base, in order.
    #[must_use]
    pub fn arguments(&self) -> Vec<String> {
        let mut arguments = vec![
            SWITCH.to_owned(),
            ENFORCE_THE_SIGNATURE_POLICY.to_owned(),
            self.reference.clone(),
        ];
        if self.when == WhenItApplies::NowBecauseThePersonAsked {
            arguments.push(APPLY.to_owned());
        }
        arguments
    }

    /// The build running when this was decided.
    #[must_use]
    pub fn from(&self) -> &Digest {
        &self.from
    }

    /// The build staged.
    #[must_use]
    pub fn to(&self) -> &Digest {
        &self.to
    }

    /// When it applies.
    #[must_use]
    pub fn when(&self) -> WhenItApplies {
        self.when
    }

    /// Who causes the restart that applies it: always the person.
    #[must_use]
    pub fn cause_of_the_restart(&self) -> Cause {
        self.when.cause_of_the_restart()
    }

    /// What a person is told once the base has staged it.
    ///
    /// Only said for [`WhenItApplies::AtTheNextRestart`] in practice — the
    /// other choice restarts the machine as it was asked to — and worded for
    /// both, so nothing that shows it has to decide which.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        strings.say(&words::WAITING_FOR_THE_RESTART.key(), &Filling::nothing())
    }
}

impl NotStaged {
    /// What a person reads, in the vocabulary.
    #[must_use]
    pub fn said(&self, strings: &Strings) -> Said {
        let word = match self {
            Self::NotRunningABuild => words::RUNNING_NOT_KNOWN,
            Self::TheMachineMovedOn { .. } => words::CHANGED_SINCE_IT_WAS_FOUND,
            Self::AlreadyWaiting => words::ALREADY_WAITING,
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
    use crate::testing::{in_english, ready_between, whole};

    fn the_source() -> Source {
        Source::named("ghcr.io/aloworld-org/alo-os").unwrap()
    }

    fn running(pair: &str) -> Deployments {
        Deployments::reported(Some(whole(pair)), None, None)
    }

    /// **At the next restart, the instruction is three arguments and none of
    /// them restarts anything.**
    #[test]
    fn at_the_next_restart_the_base_is_told_to_stage_and_nothing_else() {
        let staging = Staging::of(
            &ready_between("aa", "bb"),
            &running("aa"),
            &the_source(),
            WhenItApplies::AtTheNextRestart,
        )
        .unwrap();
        assert_eq!(
            staging.arguments(),
            [
                "switch".to_owned(),
                "--enforce-container-sigpolicy".to_owned(),
                format!("ghcr.io/aloworld-org/alo-os@{}", whole("bb").as_str()),
            ]
        );
        assert_eq!(staging.from(), &whole("aa"));
        assert_eq!(staging.to(), &whole("bb"));
        assert_eq!(staging.cause_of_the_restart(), Cause::ThePerson);
    }

    /// **Restarting now is the one choice that adds `--apply`, and it is the
    /// person's.**
    #[test]
    fn restarting_now_adds_apply_and_is_caused_by_the_person() {
        let staging = Staging::of(
            &ready_between("aa", "bb"),
            &running("aa"),
            &the_source(),
            WhenItApplies::NowBecauseThePersonAsked,
        )
        .unwrap();
        assert_eq!(staging.arguments().last().unwrap(), "--apply");
        assert_eq!(staging.cause_of_the_restart(), Cause::ThePerson);
    }

    /// **The signature policy is enforced under every choice, and no argument
    /// names a path.**
    #[test]
    fn every_instruction_enforces_the_signature_policy_and_names_no_path() {
        for when in WhenItApplies::EVERY {
            let arguments = Staging::of(
                &ready_between("aa", "bb"),
                &running("aa"),
                &the_source(),
                when,
            )
            .unwrap()
            .arguments();
            assert!(arguments.contains(&"--enforce-container-sigpolicy".to_owned()));
            for argument in &arguments {
                assert!(!argument.starts_with('/'), "{argument}");
                assert!(!argument.contains("/var"), "{argument}");
                assert!(!argument.contains("/etc"), "{argument}");
            }
            let flags: Vec<&String> = arguments.iter().filter(|a| a.starts_with("--")).collect();
            assert!(
                flags
                    .iter()
                    .all(|flag| ["--enforce-container-sigpolicy", "--apply"]
                        .contains(&flag.as_str())),
                "{flags:?}"
            );
        }
    }

    /// **A machine that moved on since the update was found is not staged.**
    #[test]
    fn a_machine_running_another_build_than_the_offer_was_found_against_is_refused() {
        let refused = Staging::of(
            &ready_between("aa", "bb"),
            &running("cc"),
            &the_source(),
            WhenItApplies::AtTheNextRestart,
        )
        .unwrap_err();
        assert_eq!(
            refused,
            NotStaged::TheMachineMovedOn {
                expected: whole("aa"),
                running: whole("cc"),
            }
        );
        let said = refused.said(&in_english());
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("nothing was changed"), "{said}");
    }

    /// **The same build staged twice is one change, not two.**
    #[test]
    fn an_update_already_waiting_is_not_staged_again() {
        let refused = Staging::of(
            &ready_between("aa", "bb"),
            &Deployments::reported(Some(whole("aa")), Some(whole("bb")), None),
            &the_source(),
            WhenItApplies::AtTheNextRestart,
        )
        .unwrap_err();
        assert_eq!(refused, NotStaged::AlreadyWaiting);
        assert!(!refused.said(&in_english()).is_a_bug());
    }

    /// **A different build waiting is replaced by the one chosen**, because
    /// the person chose this one.
    #[test]
    fn a_different_build_waiting_is_replaced_by_the_one_chosen() {
        assert!(
            Staging::of(
                &ready_between("aa", "bb"),
                &Deployments::reported(Some(whole("aa")), Some(whole("cc")), None),
                &the_source(),
                WhenItApplies::AtTheNextRestart,
            )
            .is_ok()
        );
    }

    /// **A machine running no nameable build is not staged.**
    #[test]
    fn a_machine_running_no_build_is_not_staged() {
        let refused = Staging::of(
            &ready_between("aa", "bb"),
            &Deployments::reported(None, None, None),
            &the_source(),
            WhenItApplies::AtTheNextRestart,
        )
        .unwrap_err();
        assert_eq!(refused, NotStaged::NotRunningABuild);
        assert!(!refused.said(&in_english()).is_a_bug());
    }
}
