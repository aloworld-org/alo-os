//! The road as the exact runs it is: which rented tool, and which arguments.
//!
//! [`crate::THE_ROAD`] names the six steps an enrolment takes.
//! [ADR 0056](../../../docs/decisions/0056-a-sealed-disks-promise-is-shown-on-a-machine-with-a-chip.md)
//! is where they became runs of a program, and its point 5 is the shape of this
//! file: **the two roads differ by exactly one invocation**, so there is one
//! sequence whose fifth step is either of two rather than two sequences that
//! will drift apart. Five of the six runs are the same on a machine with a chip
//! and on a machine without one.
//!
//! # This file builds arguments and runs nothing
//!
//! There is no program started anywhere in this crate, and there cannot be: it
//! depends on nothing, opens no file and starts no process, and
//! `tests/the_key_is_never_kept_on_the_disk_it_recovers.rs` reads the source to
//! hold that. What is here is a description of what is to be run, which is what
//! the installer plan is handed and what
//! `tests/the_sequence_against_a_virtual_disk.rs` runs against a real LUKS2
//! volume in the pinned base.
//!
//! # Why no argument is a free string
//!
//! Every argument is one of three things: a flag written out in this file, the
//! volume's own path ([`crate::TheVolume`], which is a disk's udev identity and
//! a number), or the path of one of the four named secrets
//! ([`crate::ASecretOnItsWay`]). There is no constructor here that takes text,
//! so there is no road from something a person or a model typed into the
//! argument list of a program that runs as root, and
//! `tests/every_argument_is_one_of_three_things.rs` reads every run of every
//! sequence to say so.
//!
//! # What is measured rather than assumed
//!
//! - `systemd-cryptenroll --recovery-key` writes the key to **standard output**
//!   and its English to standard error, so the key is read off stdout and the
//!   sentences a person meets stay ours (ADR 0054, measurement 4).
//! - It has **no option for the secret being enrolled**, so a passphrase is
//!   enrolled with `cryptsetup luksAddKey` and `systemd-cryptenroll` is used for
//!   the recovery key and the chip and nothing else (ADR 0056, measurement 6 and
//!   point 6).
//! - Both tools live under `/usr/sbin` in the pinned base, and are named by
//!   their whole path so that nothing here depends on what `PATH` happened to
//!   be.

use crate::handing_over::ASecretOnItsWay;
use crate::road::Step;
use crate::unlocking::{THE_PCRS_IT_IS_SEALED_AGAINST, WhatToAskFor};
use crate::volume::{IT_OPENS_AS, TheVolume};

/// A rented tool, and there are two.
///
/// Rented and never patched ([ADR 0011](../../../docs/decisions/0011-the-base-is-rented-and-the-image-is-a-container.md)):
/// what this crate holds is which of them is run and with what, never a line of
/// either.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TheTool {
    /// `cryptsetup`: what makes a volume, adds a secret to one, takes one away,
    /// and opens and closes it.
    TheVolumesOwnTool,
    /// `systemd-cryptenroll`: what makes the recovery key, and what seals a key
    /// to the machine's chip.
    TheEnrollingTool,
}

impl TheTool {
    /// Where the program is in the pinned base.
    #[must_use]
    pub const fn where_it_is(self) -> &'static str {
        match self {
            Self::TheVolumesOwnTool => "/usr/sbin/cryptsetup",
            Self::TheEnrollingTool => "/usr/sbin/systemd-cryptenroll",
        }
    }
}

/// Where on the road a run belongs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnTheRoad {
    /// One of the six steps of the enrolment that happens during an install.
    AStepOfTheEnrolment(Step),
    /// Opening the volume with a secret.
    OpeningIt,
    /// Closing it again.
    ClosingIt,
    /// Replacing what the person opens the machine with.
    ChangingWhatThePersonUnlocksWith,
}

/// One run of one rented tool.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    /// Where on the road it belongs.
    at: OnTheRoad,
    /// Which tool.
    tool: TheTool,
    /// Its arguments, in order, each one a flag written out in this file, the
    /// volume's path, or a named secret's path.
    arguments: Vec<String>,
    /// The secret this run has to be given, where it needs one, so that a
    /// caller knows which file to have written before it starts.
    given: Option<ASecretOnItsWay>,
}

impl Run {
    /// Where on the road this run belongs.
    #[must_use]
    pub const fn at(&self) -> OnTheRoad {
        self.at
    }

    /// Which tool is run.
    #[must_use]
    pub const fn tool(&self) -> TheTool {
        self.tool
    }

    /// Its arguments, in order.
    #[must_use]
    pub fn arguments(&self) -> &[String] {
        &self.arguments
    }

    /// The secret that must be in its file before this runs, where there is
    /// one.
    #[must_use]
    pub const fn given(&self) -> Option<ASecretOnItsWay> {
        self.given
    }

    /// Whether what this run prints to standard output is a secret.
    ///
    /// True for exactly one run on the whole road — the one that makes the
    /// recovery key — and it is said here rather than remembered by whoever
    /// reads the output, because the difference between the two is a key in a
    /// log file.
    #[must_use]
    pub const fn what_it_prints_is_a_secret(&self) -> bool {
        matches!(
            self.at,
            OnTheRoad::AStepOfTheEnrolment(Step::TheRecoveryKeyIsMade)
        )
    }

    /// One run, from its pieces.
    fn of(
        at: OnTheRoad,
        tool: TheTool,
        arguments: Vec<String>,
        given: Option<ASecretOnItsWay>,
    ) -> Self {
        Self {
            at,
            tool,
            arguments,
            given,
        }
    }
}

/// The runs that carry out one thing asked of an encrypted disk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TheSequence {
    /// The runs, in the order they happen.
    runs: Vec<Run>,
}

impl TheSequence {
    /// The enrolment that happens once, during an install, in
    /// [`crate::THE_ROAD`]'s order.
    ///
    /// Two of the six steps are a person rather than a program — the key is
    /// shown once, and the person types it back — so there are four runs and not
    /// six, and [`Run::at`] says which step each of them is.
    ///
    /// On [`WhatToAskFor::APin`] the fifth run seals the key to the machine's
    /// chip and asks for the PIN at the machine, which is the one run of the six
    /// that a virtual disk cannot show (ADR 0056, and the plan's task 9).
    #[must_use]
    pub fn enrolling_at_install(volume: &TheVolume, road: WhatToAskFor) -> Self {
        let first = ASecretOnItsWay::TheInstallersFirstKey;
        let mut runs = vec![
            Run::of(
                OnTheRoad::AStepOfTheEnrolment(Step::TheDiskIsMadeIntoAVolume),
                TheTool::TheVolumesOwnTool,
                vec![
                    "luksFormat".to_owned(),
                    "--type".to_owned(),
                    "luks2".to_owned(),
                    "--batch-mode".to_owned(),
                    keyed("--key-file", first.where_it_is()),
                    volume.where_it_is().to_owned(),
                ],
                Some(first),
            ),
            Run::of(
                OnTheRoad::AStepOfTheEnrolment(Step::TheRecoveryKeyIsMade),
                TheTool::TheEnrollingTool,
                vec![
                    "--recovery-key".to_owned(),
                    keyed("--unlock-key-file", first.where_it_is()),
                    volume.where_it_is().to_owned(),
                ],
                Some(first),
            ),
        ];
        runs.push(the_way_the_person_unlocks(volume, road));
        runs.push(Run::of(
            OnTheRoad::AStepOfTheEnrolment(Step::TheInstallersFirstKeyIsWipedAway),
            TheTool::TheVolumesOwnTool,
            vec![
                "luksRemoveKey".to_owned(),
                "--batch-mode".to_owned(),
                keyed("--key-file", first.where_it_is()),
                volume.where_it_is().to_owned(),
            ],
            Some(first),
        ));
        Self { runs }
    }

    /// Opening the volume with one of the secrets.
    ///
    /// The secret is named rather than typed: opening with
    /// [`ASecretOnItsWay::TheRecoveryKey`] is *recovering the machine* and
    /// opening with [`ASecretOnItsWay::ThePersonsSecret`] is an ordinary start,
    /// and the two are the same run with a different file because that is what
    /// LUKS makes them.
    ///
    /// Opening on the chip road needs no secret from us at all — the chip
    /// releases the key when the PIN is typed — and that is the first of the
    /// three promises only a machine with a chip can keep.
    #[must_use]
    pub fn opening(volume: &TheVolume, with: ASecretOnItsWay) -> Self {
        Self {
            runs: vec![Run::of(
                OnTheRoad::OpeningIt,
                TheTool::TheVolumesOwnTool,
                vec![
                    "open".to_owned(),
                    keyed("--key-file", with.where_it_is()),
                    volume.where_it_is().to_owned(),
                    IT_OPENS_AS.to_owned(),
                ],
                Some(with),
            )],
        }
    }

    /// Closing it again.
    #[must_use]
    pub fn closing() -> Self {
        Self {
            runs: vec![Run::of(
                OnTheRoad::ClosingIt,
                TheTool::TheVolumesOwnTool,
                vec!["close".to_owned(), IT_OPENS_AS.to_owned()],
                None,
            )],
        }
    }

    /// Replacing what the person opens the machine with.
    ///
    /// On the passphrase road it is two runs and not one, and the order is the
    /// same argument [`crate::THE_ROAD`] makes: the new secret is put in before
    /// the old one is taken out, so a machine interrupted between them opens
    /// with either rather than with neither.
    ///
    /// On the chip road it is one run, because `systemd-cryptenroll` replaces
    /// the chip's slot in place — and it is task 9's, because it needs a chip.
    #[must_use]
    pub fn changing_what_the_person_unlocks_with(volume: &TheVolume, road: WhatToAskFor) -> Self {
        let old = ASecretOnItsWay::ThePersonsSecret;
        let new = ASecretOnItsWay::ThePersonsNewSecret;
        let runs = match road {
            WhatToAskFor::APin => vec![Run::of(
                OnTheRoad::ChangingWhatThePersonUnlocksWith,
                TheTool::TheEnrollingTool,
                vec![
                    "--wipe-slot=tpm2".to_owned(),
                    "--tpm2-device=auto".to_owned(),
                    "--tpm2-with-pin=yes".to_owned(),
                    sealed_against(),
                    "--unlock-tpm2-device=auto".to_owned(),
                    volume.where_it_is().to_owned(),
                ],
                None,
            )],
            WhatToAskFor::APassphrase => vec![
                Run::of(
                    OnTheRoad::ChangingWhatThePersonUnlocksWith,
                    TheTool::TheVolumesOwnTool,
                    vec![
                        "luksAddKey".to_owned(),
                        "--batch-mode".to_owned(),
                        keyed("--key-file", old.where_it_is()),
                        keyed("--new-keyfile", new.where_it_is()),
                        volume.where_it_is().to_owned(),
                    ],
                    Some(new),
                ),
                Run::of(
                    OnTheRoad::ChangingWhatThePersonUnlocksWith,
                    TheTool::TheVolumesOwnTool,
                    vec![
                        "luksRemoveKey".to_owned(),
                        "--batch-mode".to_owned(),
                        keyed("--key-file", old.where_it_is()),
                        volume.where_it_is().to_owned(),
                    ],
                    Some(old),
                ),
            ],
        };
        Self { runs }
    }

    /// The runs, in the order they happen.
    #[must_use]
    pub fn runs(&self) -> &[Run] {
        &self.runs
    }
}

/// The one run of the six that differs between the two roads.
fn the_way_the_person_unlocks(volume: &TheVolume, road: WhatToAskFor) -> Run {
    let at = OnTheRoad::AStepOfTheEnrolment(Step::TheWayThePersonUnlocksIsEnrolled);
    let first = ASecretOnItsWay::TheInstallersFirstKey;
    match road {
        WhatToAskFor::APin => Run::of(
            at,
            TheTool::TheEnrollingTool,
            vec![
                "--tpm2-device=auto".to_owned(),
                "--tpm2-with-pin=yes".to_owned(),
                sealed_against(),
                keyed("--unlock-key-file", first.where_it_is()),
                volume.where_it_is().to_owned(),
            ],
            Some(first),
        ),
        WhatToAskFor::APassphrase => Run::of(
            at,
            TheTool::TheVolumesOwnTool,
            vec![
                "luksAddKey".to_owned(),
                "--batch-mode".to_owned(),
                keyed("--key-file", first.where_it_is()),
                keyed(
                    "--new-keyfile",
                    ASecretOnItsWay::ThePersonsSecret.where_it_is(),
                ),
                volume.where_it_is().to_owned(),
            ],
            Some(ASecretOnItsWay::ThePersonsSecret),
        ),
    }
}

/// A flag and the file it names, written the way both tools take one.
fn keyed(flag: &str, file: &str) -> String {
    format!("{flag}={file}")
}

/// Which registers the key is sealed against, written as the enrolling tool
/// takes them.
///
/// Built from [`THE_PCRS_IT_IS_SEALED_AGAINST`] rather than typed out again, so
/// that the decision and the argument cannot disagree.
fn sealed_against() -> String {
    let registers: Vec<String> = THE_PCRS_IT_IS_SEALED_AGAINST
        .iter()
        .map(u8::to_string)
        .collect();
    format!("--tpm2-pcrs={}", registers.join("+"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::volume::TheDisk;

    /// The volume every test here is about.
    fn a_volume() -> TheVolume {
        let Ok(disk) = TheDisk::named("virtio-alo-target") else {
            unreachable!("that is a disk's own name")
        };
        match TheVolume::the_partition_of(&disk, 4) {
            Ok(volume) => volume,
            Err(why) => unreachable!("4 is a partition: {why}"),
        }
    }

    /// **The enrolment is four runs in the road's order**, and the two steps
    /// that are a person rather than a program are not runs at all.
    #[test]
    fn the_enrolment_is_the_roads_steps_in_the_roads_order() {
        for road in [WhatToAskFor::APassphrase, WhatToAskFor::APin] {
            let sequence = TheSequence::enrolling_at_install(&a_volume(), road);
            let steps: Vec<OnTheRoad> = sequence.runs().iter().map(Run::at).collect();
            assert_eq!(
                steps,
                [
                    OnTheRoad::AStepOfTheEnrolment(Step::TheDiskIsMadeIntoAVolume),
                    OnTheRoad::AStepOfTheEnrolment(Step::TheRecoveryKeyIsMade),
                    OnTheRoad::AStepOfTheEnrolment(Step::TheWayThePersonUnlocksIsEnrolled),
                    OnTheRoad::AStepOfTheEnrolment(Step::TheInstallersFirstKeyIsWipedAway),
                ],
                "{road:?}"
            );
        }
    }

    /// **The two roads differ by exactly one run**, which is ADR 0056's point 5
    /// held by the code rather than by a paragraph.
    #[test]
    fn the_two_roads_differ_by_exactly_one_run() {
        let volume = a_volume();
        let chip = TheSequence::enrolling_at_install(&volume, WhatToAskFor::APin);
        let typed = TheSequence::enrolling_at_install(&volume, WhatToAskFor::APassphrase);
        let differ: Vec<usize> = chip
            .runs()
            .iter()
            .zip(typed.runs())
            .enumerate()
            .filter(|(_, (one, other))| one != other)
            .map(|(which, _)| which)
            .collect();
        assert_eq!(differ, [2], "the roads differ somewhere else as well");
        assert_eq!(
            chip.runs().get(2).map(Run::at),
            Some(OnTheRoad::AStepOfTheEnrolment(
                Step::TheWayThePersonUnlocksIsEnrolled
            ))
        );
    }

    /// **The recovery key is made before the way the person unlocks is
    /// enrolled**, and the installer's own key is wiped last — the road's one
    /// decision, held again where it turns into runs.
    #[test]
    fn the_recovery_key_is_made_before_anything_the_person_unlocks_with() {
        let sequence = TheSequence::enrolling_at_install(&a_volume(), WhatToAskFor::APassphrase);
        let made = sequence
            .runs()
            .iter()
            .position(|run| run.at() == OnTheRoad::AStepOfTheEnrolment(Step::TheRecoveryKeyIsMade));
        let enrolled = sequence.runs().iter().position(|run| {
            run.at() == OnTheRoad::AStepOfTheEnrolment(Step::TheWayThePersonUnlocksIsEnrolled)
        });
        assert!(made < enrolled, "{made:?} {enrolled:?}");
        assert_eq!(
            sequence.runs().last().map(Run::at),
            Some(OnTheRoad::AStepOfTheEnrolment(
                Step::TheInstallersFirstKeyIsWipedAway
            ))
        );
    }

    /// **Exactly one run on the whole road prints a secret**, and it is the one
    /// that makes the recovery key.
    #[test]
    fn one_run_prints_a_secret_and_the_rest_print_nothing_of_the_kind() {
        let volume = a_volume();
        let mut printing = 0;
        for sequence in [
            TheSequence::enrolling_at_install(&volume, WhatToAskFor::APassphrase),
            TheSequence::enrolling_at_install(&volume, WhatToAskFor::APin),
            TheSequence::opening(&volume, ASecretOnItsWay::ThePersonsSecret),
            TheSequence::closing(),
            TheSequence::changing_what_the_person_unlocks_with(&volume, WhatToAskFor::APassphrase),
        ] {
            for run in sequence.runs() {
                if run.what_it_prints_is_a_secret() {
                    printing += 1;
                    assert_eq!(
                        run.at(),
                        OnTheRoad::AStepOfTheEnrolment(Step::TheRecoveryKeyIsMade)
                    );
                }
            }
        }
        assert_eq!(
            printing, 2,
            "once on each of the two roads, and nowhere else"
        );
    }

    /// **The registers in the argument are the registers in the decision.**
    #[test]
    fn the_argument_names_the_register_the_decision_names() {
        assert_eq!(sealed_against(), "--tpm2-pcrs=7");
        let sealed = TheSequence::enrolling_at_install(&a_volume(), WhatToAskFor::APin);
        let Some(run) = sealed.runs().get(2) else {
            unreachable!("the enrolment has four runs")
        };
        assert!(
            run.arguments().contains(&"--tpm2-pcrs=7".to_owned()),
            "{:?}",
            run.arguments()
        );
        assert!(
            run.arguments().contains(&"--tpm2-with-pin=yes".to_owned()),
            "a chip road with no PIN is not ADR 0054's road"
        );
    }

    /// **The passphrase road never names the enrolling tool for the secret
    /// being enrolled**, which is ADR 0056's measurement 6: that tool has no
    /// option for one.
    #[test]
    fn a_passphrase_is_enrolled_by_the_volumes_own_tool() {
        let sequence = TheSequence::enrolling_at_install(&a_volume(), WhatToAskFor::APassphrase);
        let Some(run) = sequence.runs().get(2) else {
            unreachable!("the enrolment has four runs")
        };
        assert_eq!(run.tool(), TheTool::TheVolumesOwnTool);
        assert_eq!(run.given(), Some(ASecretOnItsWay::ThePersonsSecret));
        assert!(run.arguments().contains(&"luksAddKey".to_owned()));
    }

    /// **Changing a passphrase puts the new secret in before it takes the old
    /// one out**, so an interruption leaves a machine that opens.
    #[test]
    fn the_new_secret_goes_in_before_the_old_one_comes_out() {
        let sequence = TheSequence::changing_what_the_person_unlocks_with(
            &a_volume(),
            WhatToAskFor::APassphrase,
        );
        let what: Vec<Option<&String>> = sequence
            .runs()
            .iter()
            .map(|run| run.arguments().first())
            .collect();
        assert_eq!(
            what,
            [
                Some(&"luksAddKey".to_owned()),
                Some(&"luksRemoveKey".to_owned())
            ]
        );
    }

    /// Opening names the secret it opens with, and closing needs none.
    #[test]
    fn opening_names_its_secret_and_closing_needs_none() {
        let volume = a_volume();
        for secret in [
            ASecretOnItsWay::ThePersonsSecret,
            ASecretOnItsWay::TheRecoveryKey,
        ] {
            let sequence = TheSequence::opening(&volume, secret);
            let Some(run) = sequence.runs().first() else {
                unreachable!("opening is one run")
            };
            assert_eq!(run.given(), Some(secret));
            assert!(
                !run.arguments().contains(&secret.where_it_is().to_owned()),
                "a bare path where a flag was meant"
            );
            assert!(
                run.arguments()
                    .contains(&format!("--key-file={}", secret.where_it_is()))
            );
        }
        let Some(closing) = TheSequence::closing().runs().first().cloned() else {
            unreachable!("closing is one run")
        };
        assert_eq!(closing.given(), None);
        assert_eq!(
            closing.arguments(),
            ["close".to_owned(), IT_OPENS_AS.to_owned()]
        );
    }

    /// Each tool is named by its whole path, so nothing depends on `PATH`.
    #[test]
    fn each_tool_is_named_by_its_whole_path() {
        for tool in [TheTool::TheVolumesOwnTool, TheTool::TheEnrollingTool] {
            assert!(tool.where_it_is().starts_with("/usr/sbin/"), "{tool:?}");
        }
    }
}
