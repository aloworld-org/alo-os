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
//! # Two doors, one instruction
//!
//! [`Staging::of`] is the road a person takes through the shell: a [`Ready`]
//! from a check **this** machine made, carrying the build it was running when
//! it looked and the build the place offered.
//! [`Staging::approved`] is the other road, opened for
//! [ADR 0053](../../../docs/decisions/0053-an-update-is-carried-out-by-a-unit-the-broker-starts-never-by-the-broker.md):
//! what crosses a process boundary there is an **approval**, a `{from, to}`
//! pair a person approved, with no `Ready` behind it and nothing of this
//! crate's in it.
//!
//! Both go through one private decision, so there is exactly one place that
//! knows what staging an update means. A second place would assemble the
//! base's arguments from two digests itself, and one of the two would drift.
//!
//! # What is refused before anything runs
//!
//! [`NotStaged`]: the machine is not running a build this can name; it is no
//! longer running the build the offer was compared against — something
//! changed since the person was told, so what they chose is not this; the
//! offered build is already waiting, so a second approval would be a second
//! execution of one change; or the two builds named are the same build, which
//! is not an update at all. The last of those is unreachable through
//! [`Staging::of`] — `Standing::between` answers *up to date* rather than
//! making a `Ready` from one build twice — and reachable through
//! [`Staging::approved`], because an approval made elsewhere is only two
//! digests and can hold that.

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
    /// The build to change from and the build to change to are one build, so
    /// there is no update to stage.
    ///
    /// Only [`Staging::approved`] can meet it: a [`Ready`] cannot hold one
    /// build twice.
    NotAnUpdate,
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
        Self::decided(ready.running(), ready.offered(), deployments, source, when)
    }

    /// The instruction staging `to` on a machine running `from`, decided from
    /// an approval that arrived from somewhere else.
    ///
    /// The road [ADR 0053](../../../docs/decisions/0053-an-update-is-carried-out-by-a-unit-the-broker-starts-never-by-the-broker.md)
    /// needs: what crosses into the program that runs the base is a `{from,
    /// to}` pair a person approved, and there is no [`Ready`] on that side of
    /// the boundary to make one from. The instruction it decides is
    /// [`Staging::of`]'s, element for element, because both go through one
    /// private decision.
    ///
    /// `deployments` is what the base reports **now** — read after the
    /// approval and before the instruction is made — so every refusal below is
    /// about the machine as it is rather than as it was when somebody approved.
    ///
    /// # It applies at the next restart, and there is no other choice here
    ///
    /// An approval made elsewhere carries two builds and nothing else. *Restart
    /// now and apply it* is not in it, so it is not decided from it: this
    /// constructor takes no [`WhenItApplies`] and always decides
    /// [`WhenItApplies::AtTheNextRestart`], and the instruction it writes
    /// therefore never carries `--apply`. A restart the person did not approve
    /// is exactly what [`THE_RULE`](crate::THE_RULE) forbids, and this is that
    /// promise held by construction rather than by the caller remembering it.
    ///
    /// # What it is not
    ///
    /// It reads nothing, starts nothing and names no program. It does not ask
    /// a registry anything, and it does not decide whether a build is vouched
    /// for — that is [`crate::Vouching`]'s, answered where the check happens,
    /// and the machine's signature policy answers the question that matters at
    /// the moment the base is told.
    ///
    /// # Errors
    /// [`NotStaged`], including [`NotStaged::NotAnUpdate`], which
    /// [`Staging::of`] cannot meet. An approval whose two builds arrived the
    /// wrong way round is refused as well, by
    /// [`NotStaged::TheMachineMovedOn`]: the machine is not running the build
    /// such an approval says it should be changing to.
    pub fn approved(
        from: &Digest,
        to: &Digest,
        deployments: &Deployments,
        source: &Source,
    ) -> Result<Self, NotStaged> {
        Self::decided(
            from,
            to,
            deployments,
            source,
            WhenItApplies::AtTheNextRestart,
        )
    }

    /// The one decision both doors go through.
    fn decided(
        from: &Digest,
        to: &Digest,
        deployments: &Deployments,
        source: &Source,
        when: WhenItApplies,
    ) -> Result<Self, NotStaged> {
        if from == to {
            return Err(NotStaged::NotAnUpdate);
        }
        let running = deployments
            .running()
            .map_err(|_| NotStaged::NotRunningABuild)?;
        if running.digest() != from {
            return Err(NotStaged::TheMachineMovedOn {
                expected: from.clone(),
                running: running.digest().clone(),
            });
        }
        if deployments.staged() == Some(to) {
            return Err(NotStaged::AlreadyWaiting);
        }
        Ok(Self {
            from: from.clone(),
            to: to.clone(),
            reference: source.at(to),
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
            Self::NotAnUpdate => words::NOT_AN_UPDATE,
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

    /// **The two doors decide one instruction**, element for element.
    ///
    /// One [`Ready`], staged both ways — through [`Staging::of`] and through
    /// [`Staging::approved`] given that `Ready`'s own `{from, to}` — against
    /// the same deployments and the same source. This is what holds the broker's
    /// road to this crate's decision rather than to a second one that resembles
    /// it (ADR 0053).
    #[test]
    fn an_approval_and_a_ready_decide_the_same_instruction_element_for_element() {
        let ready = ready_between("aa", "bb");
        let deployments = running("aa");
        let through_a_ready = Staging::of(
            &ready,
            &deployments,
            &the_source(),
            WhenItApplies::AtTheNextRestart,
        )
        .unwrap();
        let through_an_approval = Staging::approved(
            ready.running(),
            ready.offered(),
            &deployments,
            &the_source(),
        )
        .unwrap();

        assert_eq!(
            through_an_approval.arguments(),
            through_a_ready.arguments(),
            "the two doors wrote different instructions"
        );
        for (from_an_approval, from_a_ready) in through_an_approval
            .arguments()
            .iter()
            .zip(through_a_ready.arguments().iter())
        {
            assert_eq!(from_an_approval, from_a_ready);
        }
        assert_eq!(through_an_approval, through_a_ready);
        assert_eq!(through_an_approval.from(), through_a_ready.from());
        assert_eq!(through_an_approval.to(), through_a_ready.to());
        assert_eq!(through_an_approval.when(), through_a_ready.when());
    }

    /// **An approval never restarts the machine.** It carries two builds and
    /// no choice about restarting, so the instruction it decides has no
    /// `--apply` in it and waits for a restart the person makes.
    #[test]
    fn an_approval_stages_for_the_next_restart_and_never_applies_at_once() {
        let staging =
            Staging::approved(&whole("aa"), &whole("bb"), &running("aa"), &the_source()).unwrap();
        assert_eq!(staging.when(), WhenItApplies::AtTheNextRestart);
        assert!(!staging.arguments().contains(&"--apply".to_owned()));
        assert_eq!(staging.cause_of_the_restart(), Cause::ThePerson);
        assert!(
            staging
                .arguments()
                .contains(&"--enforce-container-sigpolicy".to_owned())
        );
    }

    /// **One build named twice is not an update**, and is refused rather than
    /// staged into a restart that would change nothing.
    #[test]
    fn an_approval_naming_one_build_twice_is_not_an_update() {
        let refused = Staging::approved(&whole("aa"), &whole("aa"), &running("aa"), &the_source())
            .unwrap_err();
        assert_eq!(refused, NotStaged::NotAnUpdate);
        let said = refused.said(&in_english());
        assert!(!said.is_a_bug(), "{said}");
        assert!(said.text().contains("nothing was changed"), "{said}");

        // Decided before the machine is read, so a machine reporting nothing
        // does not turn it into some other refusal.
        assert_eq!(
            Staging::approved(
                &whole("aa"),
                &whole("aa"),
                &Deployments::reported(None, None, None),
                &the_source(),
            ),
            Err(NotStaged::NotAnUpdate)
        );
    }

    /// **An approval about a machine that has moved on since is refused**, and
    /// the refusal carries both builds so what moved can be said.
    #[test]
    fn an_approval_for_a_build_this_machine_no_longer_runs_is_refused_with_both_builds() {
        let refused = Staging::approved(&whole("aa"), &whole("bb"), &running("cc"), &the_source())
            .unwrap_err();
        assert_eq!(
            refused,
            NotStaged::TheMachineMovedOn {
                expected: whole("aa"),
                running: whole("cc"),
            }
        );
        assert!(!refused.said(&in_english()).is_a_bug());
    }

    /// **An approval that arrived the wrong way round is refused**, because
    /// the machine is not running the build it says to change to.
    #[test]
    fn an_approval_whose_two_builds_arrived_swapped_is_refused() {
        assert_eq!(
            Staging::approved(&whole("bb"), &whole("aa"), &running("aa"), &the_source()),
            Err(NotStaged::TheMachineMovedOn {
                expected: whole("bb"),
                running: whole("aa"),
            })
        );
    }

    /// **An approval for a build already waiting is one change, not two.**
    #[test]
    fn an_approval_for_a_build_already_waiting_is_not_staged_again() {
        let refused = Staging::approved(
            &whole("aa"),
            &whole("bb"),
            &Deployments::reported(Some(whole("aa")), Some(whole("bb")), None),
            &the_source(),
        )
        .unwrap_err();
        assert_eq!(refused, NotStaged::AlreadyWaiting);
        assert!(!refused.said(&in_english()).is_a_bug());
    }

    /// **An approval reaching a machine running no nameable build is refused.**
    #[test]
    fn an_approval_on_a_machine_running_no_build_is_not_staged() {
        let refused = Staging::approved(
            &whole("aa"),
            &whole("bb"),
            &Deployments::reported(None, None, None),
            &the_source(),
        )
        .unwrap_err();
        assert_eq!(refused, NotStaged::NotRunningABuild);
        assert!(!refused.said(&in_english()).is_a_bug());
    }

    /// **Every refusal an approval can meet reads differently from the others**,
    /// so a person is never told one thing for two situations.
    #[test]
    fn no_two_refusals_an_approval_can_meet_read_alike() {
        let strings = in_english();
        let refusals = [
            NotStaged::NotRunningABuild,
            NotStaged::TheMachineMovedOn {
                expected: whole("aa"),
                running: whole("cc"),
            },
            NotStaged::AlreadyWaiting,
            NotStaged::NotAnUpdate,
        ];
        let mut texts = std::collections::BTreeSet::new();
        for refusal in &refusals {
            let said = refusal.said(&strings);
            assert!(!said.is_a_bug(), "{said}");
            assert!(said.unfilled().is_empty(), "{said}");
            texts.insert(said.into_text());
        }
        assert_eq!(texts.len(), refusals.len(), "two refusals read the same");
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
