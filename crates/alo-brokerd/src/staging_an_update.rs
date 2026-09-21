//! What `alo-applying-an-update.service` does: read what the broker handed it,
//! decide the instruction through `alo-keeping-up`, and give the base exactly
//! that.
//!
//! This is the privileged half of `updates.apply`
//! ([ADR 0053](../../../docs/decisions/0053-an-update-is-carried-out-by-a-unit-the-broker-starts-never-by-the-broker.md),
//! option B). It runs in a unit of its own with a fixed command line and the
//! capabilities the base needs, rather than in the broker, which holds none.
//!
//! In this order, and the order is the guarantee:
//!
//! 1. **What the broker handed over is read** from a folder only root can
//!    reach ([`crate::for_the_unit`]) — never from the folder the person
//!    writes in, whose contents can change after they were checked.
//! 2. **It is two builds and nothing else** ([`crate::approved::AnUpdate`]).
//! 3. **The machine is read now** — its builds and the repository its booted
//!    build came from, out of one answer ([`crate::TheMachineNow`]).
//! 4. **`alo_keeping_up::Staging::approved` decides the instruction**, and
//!    refuses before anything runs when the machine has moved on since the
//!    person approved, when the build is already waiting, when the two builds
//!    are one, or when the machine runs no build this can name.
//! 5. **The base is told**, with exactly the arguments that decision wrote.
//!
//! # It cannot restart the machine, and that is held in three places
//!
//! `Staging::approved` takes no *when* and always decides *at the next
//! restart*, so the instruction it writes never carries `--apply`
//! (`alo-keeping-up`'s own tests). Nothing here adds an argument to it. And
//! the unit's capability set does not include `CAP_SYS_BOOT`
//! (`alo-applying-an-update.service`). *Restart now and apply it* is the
//! person's restart through their own session, afterwards.
//!
//! # Nothing here decides what staging means
//!
//! The instruction is `alo-keeping-up`'s, element for element, through the one
//! constructor that crate opened for this road. A second place that assembled
//! the base's arguments from two digests would be a second answer to *what
//! staging an update is*, and one of the two would drift.

use alo_keeping_up::{NotStaged, Staging};
use alo_updating::{Base, NotAnswered, refused_for_its_signature};

use crate::approved::{AnUpdate, LONGEST_HANDED_OVER, NotAnUpdate};
use crate::for_the_unit::{self, Handing, NotHanded};
use crate::the_machine_now::{self, TheMachineNow};
use crate::the_road_out::NoRoad;

/// Why an update was not staged. **On every one of these the machine is as it
/// was, and the next restart starts the build running now.**
#[derive(Debug)]
pub enum NotStagedByTheUnit {
    /// The broker handed nothing over, or what it handed over is not a file
    /// of root's.
    NotHanded(NotHanded),
    /// What was handed over is not a pair of builds.
    NotAnUpdate(NotAnUpdate),
    /// The machine could not be read, so nothing was decided about it.
    NotRead(the_machine_now::NotRead),
    /// Refused before anything ran, by the crate that decides what an update
    /// is.
    Refused(NotStaged),
    /// No road out could be decided, so nothing was fetched.
    NoRoad(NoRoad),
    /// The base was told and refused, or failed part way.
    TheBaseDidNotStageIt(NotAnswered),
    /// The base refused the build because its signature policy would not have
    /// it: this machine could not confirm the build is alo OS.
    TheBuildWasNotGenuine(NotAnswered),
}

impl std::fmt::Display for NotStagedByTheUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotHanded(why) => write!(f, "nothing was handed over to stage: {why}"),
            Self::NotAnUpdate(why) => write!(f, "{why}"),
            Self::NotRead(why) => write!(f, "{why}"),
            Self::Refused(why) => write!(
                f,
                "the update a person approved is not one this machine can stage now: {why:?}"
            ),
            Self::NoRoad(why) => write!(f, "{why}"),
            Self::TheBaseDidNotStageIt(why) => {
                write!(f, "the base did not stage the build: {why:?}")
            }
            Self::TheBuildWasNotGenuine(why) => write!(
                f,
                "this machine could not confirm the build came from alo OS, so it was not \
                 staged: {why:?}"
            ),
        }
    }
}

impl std::error::Error for NotStagedByTheUnit {}

/// Carry `updates.apply` out: stage the build the broker handed over.
///
/// `reading` is the base as it is asked about this machine, which reaches no
/// network. `fetching` is handed the host the build comes from and gives back
/// the base as it is told to pull one — which is where the machine's own proxy
/// goes, because that is the call that leaves the machine.
///
/// # Errors
/// [`NotStagedByTheUnit`], on every one of which the machine is unchanged.
pub fn carried_out<B: Base>(
    reading: &impl Base,
    handing: &Handing,
    fetching: impl FnOnce(&str) -> Result<B, NoRoad>,
) -> Result<Staging, NotStagedByTheUnit> {
    let bytes = for_the_unit::handed(&handing.update(), LONGEST_HANDED_OVER)
        .map_err(NotStagedByTheUnit::NotHanded)?;
    let update = AnUpdate::read(&bytes).map_err(NotStagedByTheUnit::NotAnUpdate)?;
    let now = TheMachineNow::read(reading).map_err(NotStagedByTheUnit::NotRead)?;
    let staging = Staging::approved(update.from(), update.to(), now.deployments(), now.source())
        .map_err(NotStagedByTheUnit::Refused)?;
    let host = now
        .source()
        .as_str()
        .split('/')
        .next()
        .unwrap_or_else(|| now.source().as_str());
    let base = fetching(host).map_err(NotStagedByTheUnit::NoRoad)?;
    base.asked(&staging.arguments()).map_err(|why| {
        // Which refusal it was. The instruction always carries
        // `--enforce-container-sigpolicy`, so a build the machine's signature
        // policy will not have is refused by the base before anything is
        // staged — a different thing to tell a person than a download that
        // stopped half way.
        if refused_for_its_signature(&why) {
            NotStagedByTheUnit::TheBuildWasNotGenuine(why)
        } else {
            NotStagedByTheUnit::TheBaseDidNotStageIt(why)
        }
    })?;
    Ok(staging)
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use alo_keeping_up::Digest;
    use alo_updating::THE_STATUS;

    use super::*;

    /// A whole digest made of one repeated pair.
    fn whole(pair: &str) -> Digest {
        Digest::read(&format!("sha256:{}", pair.repeat(32))).unwrap()
    }

    /// Where this machine's builds come from, in the answers below.
    const THE_REPOSITORY: &str = "ghcr.io/aloworld-org/alo-os";

    /// What `bootc status` answers on a machine booted on `booted`, with
    /// `staged` waiting.
    fn an_answer(booted: &Digest, staged: Option<&Digest>) -> Vec<u8> {
        let waiting = staged.map_or_else(|| "null".to_owned(), deployment);
        format!(
            r#"{{"apiVersion":"org.containers.bootc/v1","kind":"BootcHost",
            "spec":{{"image":{{"image":"{THE_REPOSITORY}","transport":"registry"}}}},
            "status":{{"staged":{waiting},"rollback":null,"rollbackQueued":false,
              "booted":{},"type":"bootcHost"}}}}"#,
            deployment(booted)
        )
        .into_bytes()
    }

    /// One deployment in that answer.
    fn deployment(digest: &Digest) -> String {
        format!(
            r#"{{"image":{{"image":{{"image":"{THE_REPOSITORY}@{}","transport":"registry"}},
            "version":"0.0.5","imageDigest":"{}","architecture":"amd64"}},
            "ostree":{{"checksum":"abc","deploySerial":0,"stateroot":"default"}}}}"#,
            digest.as_str(),
            digest.as_str()
        )
    }

    /// What one stand-in base answers and remembers, shared by every copy of
    /// it — so the base handed to `fetching` and the base asked for the status
    /// are one machine.
    #[derive(Debug)]
    struct Answers {
        /// What it answers `status` with.
        status: Vec<u8>,
        /// Whether the instruction is refused, and with what it said.
        refusing: Option<String>,
        /// Every argument list it was given that was not the status.
        told: RefCell<Vec<Vec<String>>>,
    }

    /// A base that answers the status with what it was given and writes down
    /// every other thing it was asked.
    #[derive(Debug, Clone)]
    struct ABase(Rc<Answers>);

    impl ABase {
        fn answering(status: Vec<u8>) -> Self {
            Self(Rc::new(Answers {
                status,
                refusing: None,
                told: RefCell::new(Vec::new()),
            }))
        }

        fn refusing(status: Vec<u8>, said: &str) -> Self {
            Self(Rc::new(Answers {
                status,
                refusing: Some(said.to_owned()),
                told: RefCell::new(Vec::new()),
            }))
        }

        fn told(&self) -> Vec<Vec<String>> {
            self.0.told.borrow().clone()
        }
    }

    impl Base for ABase {
        fn asked(&self, arguments: &[String]) -> Result<Vec<u8>, NotAnswered> {
            if arguments == THE_STATUS {
                return Ok(self.0.status.clone());
            }
            self.0.told.borrow_mut().push(arguments.to_vec());
            match &self.0.refusing {
                None => Ok(Vec::new()),
                Some(said) => Err(NotAnswered::SaidNo {
                    program: "/usr/bin/bootc".to_owned(),
                    code: Some(1),
                    said: said.clone(),
                }),
            }
        }
    }

    /// A folder of this test's own, holding what the broker handed over.
    fn handed(named: &str, bytes: &[u8]) -> Handing {
        let at = std::env::temp_dir().join(format!(
            "alo-brokerd-staging-{}-{named}",
            std::process::id()
        ));
        drop(std::fs::remove_dir_all(&at));
        std::fs::create_dir_all(&at).unwrap();
        let handing = Handing::at(&at);
        for_the_unit::put(&handing.update(), bytes).unwrap();
        handing
    }

    /// A folder with nothing handed over in it.
    fn handed_nothing(named: &str) -> Handing {
        let at = std::env::temp_dir().join(format!(
            "alo-brokerd-staging-{}-{named}",
            std::process::id()
        ));
        drop(std::fs::remove_dir_all(&at));
        std::fs::create_dir_all(&at).unwrap();
        Handing::at(&at)
    }

    /// The road out a test gives: the same machine, straight out, whatever the
    /// host.
    fn straight_out(base: &ABase) -> impl FnOnce(&str) -> Result<ABase, NoRoad> {
        let base = base.clone();
        move |_| Ok(base)
    }

    /// **The build a person approved is staged, with exactly the instruction
    /// `alo-keeping-up` wrote** — and the repository comes from the machine
    /// rather than from anything handed over.
    #[test]
    fn the_build_approved_is_staged_with_the_instruction_the_crate_decided() {
        let update = AnUpdate::between(whole("aa"), whole("bb"));
        let handing = handed("carried", &update.written());
        let base = ABase::answering(an_answer(&whole("aa"), None));

        let staging = carried_out(&base, &handing, straight_out(&base)).unwrap();
        assert_eq!(staging.to(), &whole("bb"));
        assert_eq!(
            base.told(),
            vec![vec![
                "switch".to_owned(),
                "--enforce-container-sigpolicy".to_owned(),
                format!("{THE_REPOSITORY}@{}", whole("bb").as_str()),
            ]]
        );
    }

    /// **Nothing the unit causes carries `--apply`.** ADR 0053's first
    /// promise, measured at the argument list the base actually received.
    #[test]
    fn nothing_the_unit_tells_the_base_restarts_the_machine() {
        let update = AnUpdate::between(whole("aa"), whole("bb"));
        let handing = handed("no-apply", &update.written());
        let base = ABase::answering(an_answer(&whole("aa"), None));
        carried_out(&base, &handing, straight_out(&base)).unwrap();
        for told in base.told() {
            assert!(!told.contains(&"--apply".to_owned()), "{told:?}");
            assert!(!told.iter().any(|word| word.contains("reboot")), "{told:?}");
        }
    }

    /// **A machine that moved on since the person approved is refused, and the
    /// base is told nothing.**
    #[test]
    fn a_machine_that_moved_on_since_the_approval_is_refused_before_the_base_is_told() {
        let update = AnUpdate::between(whole("aa"), whole("bb"));
        let handing = handed("moved-on", &update.written());
        let base = ABase::answering(an_answer(&whole("cc"), None));
        let refused = carried_out(&base, &handing, straight_out(&base)).unwrap_err();
        assert!(
            matches!(
                refused,
                NotStagedByTheUnit::Refused(NotStaged::TheMachineMovedOn { .. })
            ),
            "{refused}"
        );
        assert!(base.told().is_empty(), "the base was told something");
    }

    /// **A build already waiting is not staged a second time**: one approval
    /// is one execution.
    #[test]
    fn a_build_already_waiting_is_not_staged_again() {
        let update = AnUpdate::between(whole("aa"), whole("bb"));
        let handing = handed("waiting", &update.written());
        let base = ABase::answering(an_answer(&whole("aa"), Some(&whole("bb"))));
        let refused = carried_out(&base, &handing, straight_out(&base)).unwrap_err();
        assert!(
            matches!(
                refused,
                NotStagedByTheUnit::Refused(NotStaged::AlreadyWaiting)
            ),
            "{refused}"
        );
        assert!(base.told().is_empty());
    }

    /// **Two builds that are one build is not an update**, and nothing runs.
    #[test]
    fn one_build_named_twice_is_not_staged() {
        let update = AnUpdate::between(whole("aa"), whole("aa"));
        let handing = handed("not-an-update", &update.written());
        let base = ABase::answering(an_answer(&whole("aa"), None));
        let refused = carried_out(&base, &handing, straight_out(&base)).unwrap_err();
        assert!(
            matches!(refused, NotStagedByTheUnit::Refused(NotStaged::NotAnUpdate)),
            "{refused}"
        );
        assert!(base.told().is_empty());
    }

    /// **A unit started with nothing handed over stages nothing**, which is
    /// what somebody starting the unit by hand meets.
    #[test]
    fn a_unit_started_with_nothing_handed_over_stages_nothing() {
        let handing = handed_nothing("nothing");
        let base = ABase::answering(an_answer(&whole("aa"), None));
        let refused = carried_out(&base, &handing, straight_out(&base)).unwrap_err();
        assert!(
            matches!(refused, NotStagedByTheUnit::NotHanded(_)),
            "{refused}"
        );
        assert!(base.told().is_empty());
    }

    /// **Something that is not a pair of builds stages nothing.**
    #[test]
    fn something_that_is_not_a_pair_of_builds_stages_nothing() {
        let handing = handed("not-a-pair", b"switch --apply\n");
        let base = ABase::answering(an_answer(&whole("aa"), None));
        let refused = carried_out(&base, &handing, straight_out(&base)).unwrap_err();
        assert!(
            matches!(refused, NotStagedByTheUnit::NotAnUpdate(_)),
            "{refused}"
        );
        assert!(base.told().is_empty());
    }

    /// **A build the machine's signature policy refuses is its own refusal**,
    /// told apart from a download that stopped half way.
    #[test]
    fn a_build_the_signature_policy_refuses_is_its_own_refusal() {
        let update = AnUpdate::between(whole("aa"), whole("bb"));
        let handing = handed("not-genuine", &update.written());
        let base = ABase::refusing(
            an_answer(&whole("aa"), None),
            "Error: Fetching image: Source image rejected: Signature for identity \
             ghcr.io/aloworld-org/alo-os is not accepted",
        );
        let refused = carried_out(&base, &handing, straight_out(&base)).unwrap_err();
        assert!(
            matches!(refused, NotStagedByTheUnit::TheBuildWasNotGenuine(_)),
            "{refused}"
        );

        let handing = handed("stopped", &update.written());
        let base = ABase::refusing(an_answer(&whole("aa"), None), "error: connection reset");
        let refused = carried_out(&base, &handing, straight_out(&base)).unwrap_err();
        assert!(
            matches!(refused, NotStagedByTheUnit::TheBaseDidNotStageIt(_)),
            "{refused}"
        );
    }

    /// **A machine with no road out stages nothing**, and the base is never
    /// told to fetch anything.
    #[test]
    fn a_machine_with_no_road_out_stages_nothing() {
        let update = AnUpdate::between(whole("aa"), whole("bb"));
        let handing = handed("no-road", &update.written());
        let base = ABase::answering(an_answer(&whole("aa"), None));
        let refused = carried_out(&base, &handing, |_| {
            Err::<ABase, NoRoad>(NoRoad::NotRead {
                path: "/etc/alo-proxy/proxy.json".to_owned(),
                why: "it is not a file this machine wrote".to_owned(),
            })
        })
        .unwrap_err();
        assert!(
            matches!(refused, NotStagedByTheUnit::NoRoad(_)),
            "{refused}"
        );
        assert!(base.told().is_empty());
    }
}
