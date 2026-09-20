//! Applying an update, reading what runs, and writing down that the machine
//! updated — against a stand-in for the base that writes down every question it
//! is asked, so each refusal can be shown to have told the base nothing.
//!
//! What a real base does with the same instructions on a real boot is
//! `tests/an_update_keeps_the_persons_things.rs`, in a virtual machine.

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use alo_egress::{Destination, Indicator};
use alo_keeping::{Reading, Writing};
use alo_keeping_up::{
    Digest, NotStaged, Offered, Ready, Running, Since, Source, Standing, Vouching, WhenItApplies,
    a_check_at,
};
use alo_record::{Entry, Happened};
use alo_saying::everything_this_machine_can_say;
use alo_strings::Strings;
use alo_updating::{
    AcrossRestarts, Base, NotAnswered, NotApplied, NotRead, NotRecorded, after_a_restart, apply,
    deployments, running,
};

/// A stand-in for the base: answers its status from what it is holding, and a
/// `switch` with what it was told to, and writes down every question.
struct TheBaseStandingIn {
    /// What `status` answers.
    status: RefCell<String>,
    /// Whether `switch` succeeds.
    stages: bool,
    /// Every question asked, in order.
    asked: RefCell<Vec<Vec<String>>>,
}

impl TheBaseStandingIn {
    fn running(booted: &str, staged: Option<&str>) -> Self {
        Self {
            status: RefCell::new(a_status(booted, staged)),
            stages: true,
            asked: RefCell::new(Vec::new()),
        }
    }

    fn now_running(&self, booted: &str) {
        *self.status.borrow_mut() = a_status(booted, None);
    }

    fn switches(&self) -> Vec<Vec<String>> {
        self.asked
            .borrow()
            .iter()
            .filter(|asked| asked.first().is_some_and(|verb| verb == "switch"))
            .cloned()
            .collect()
    }
}

impl Base for TheBaseStandingIn {
    fn asked(&self, arguments: &[String]) -> Result<Vec<u8>, NotAnswered> {
        self.asked.borrow_mut().push(arguments.to_vec());
        match arguments.first().map(String::as_str) {
            Some("status") => Ok(self.status.borrow().clone().into_bytes()),
            Some("switch") if self.stages => Ok(Vec::new()),
            _ => Err(NotAnswered::SaidNo {
                program: "bootc".to_owned(),
                code: Some(1),
                // Deliberately **not** a signature refusal: that is its own
                // member and its own sentence now
                // (`tests/an_offer_a_person_can_act_on.rs`), and a stand-in
                // whose only failure was one would have made every test here
                // about the wrong refusal.
                said: "error: Pulling: reading blob sha256:26cdc51f: unexpected EOF".to_owned(),
            }),
        }
    }
}

/// The base's status document, with a build booted and maybe one waiting.
fn a_status(booted: &str, staged: Option<&str>) -> String {
    let entry = |digest: &str| {
        format!(
            r#"{{"image":{{"image":{{"image":"ghcr.io/aloworld-org/alo-os","transport":"registry"}},"imageDigest":"{digest}"}},"pinned":false}}"#
        )
    };
    format!(
        r#"{{"apiVersion":"org.containers.bootc/v1","kind":"BootcHost","status":{{"booted":{},"staged":{},"rollback":null}}}}"#,
        entry(booted),
        staged.map_or_else(|| "null".to_owned(), entry)
    )
}

fn build(pair: &str) -> String {
    format!("sha256:{}", pair.repeat(32))
}

fn digest(pair: &str) -> Digest {
    Digest::read(&build(pair)).unwrap()
}

fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(60 * 60 * 12)
}

fn the_source() -> Source {
    Source::named("ghcr.io/aloworld-org/alo-os").unwrap()
}

/// An update ready from `running` to `offered`, found the only way one can be:
/// during a check that was on the indicator.
fn ready(running: &str, offered: &str) -> Ready {
    let mut indicator = Indicator::default();
    let underway =
        indicator.beginning_on_its_own(a_check_at(Destination::at("ghcr.io").unwrap()), noon());
    let offer = Offered::heard(&underway, digest(offered), Vouching::ThePlaceVouchesForIt).unwrap();
    assert!(indicator.ended_on_its_own(underway));
    match Standing::between(&Running::reported(digest(running)), &offer) {
        Standing::Ready(ready) => ready,
        Standing::UpToDate => panic!("{running} and {offered} are the same build"),
    }
}

fn the_machine() -> Strings {
    Strings::of(everything_this_machine_can_say().unwrap())
}

/// A folder of this test's own, emptied first.
fn a_folder(named: &str) -> PathBuf {
    let folder = std::env::temp_dir().join(format!("alo-updating-{named}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&folder);
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

fn entries(record: &Path) -> Vec<Entry> {
    Reading::at(record)
        .unwrap()
        .record()
        .everything()
        .cloned()
        .collect()
}

/// **An update the person chose is staged by digest**, with the signature
/// policy enforced, after one reading of the machine as it is — and nothing
/// else is asked.
#[test]
fn an_update_the_person_chose_is_staged_by_digest_and_nothing_else_is_asked() {
    let base = TheBaseStandingIn::running(&build("aa"), None);
    let staging = apply(
        &base,
        &ready("aa", "bb"),
        &the_source(),
        WhenItApplies::AtTheNextRestart,
    )
    .unwrap();
    assert_eq!(staging.to(), &digest("bb"));
    assert_eq!(
        *base.asked.borrow(),
        [
            vec!["status", "--format", "json", "--format-version", "1"],
            vec![
                "switch",
                "--enforce-container-sigpolicy",
                &format!("ghcr.io/aloworld-org/alo-os@{}", build("bb")),
            ],
        ]
        .map(|asked| asked.into_iter().map(str::to_owned).collect::<Vec<_>>())
    );
    let said = staging.said(&the_machine());
    assert!(!said.is_a_bug(), "{said}");
}

/// **A machine that moved on since the update was found is refused, and the
/// base is never told to stage anything.**
#[test]
fn a_machine_that_moved_on_is_refused_and_the_base_is_never_told_to_stage() {
    let base = TheBaseStandingIn::running(&build("cc"), None);
    let refused = apply(
        &base,
        &ready("aa", "bb"),
        &the_source(),
        WhenItApplies::NowBecauseThePersonAsked,
    )
    .unwrap_err();
    assert!(
        matches!(
            refused,
            NotApplied::Refused(NotStaged::TheMachineMovedOn { .. })
        ),
        "{refused:?}"
    );
    assert!(base.switches().is_empty(), "{:?}", base.asked.borrow());
    let said = refused.said(&the_machine());
    assert!(!said.is_a_bug(), "{said}");
}

/// **One approval, one execution**: an update already waiting is not staged a
/// second time.
#[test]
fn an_update_already_waiting_is_not_staged_a_second_time() {
    let base = TheBaseStandingIn::running(&build("aa"), None);
    let chosen = ready("aa", "bb");
    apply(
        &base,
        &chosen,
        &the_source(),
        WhenItApplies::AtTheNextRestart,
    )
    .unwrap();
    *base.status.borrow_mut() = a_status(&build("aa"), Some(&build("bb")));

    let again = apply(
        &base,
        &chosen,
        &the_source(),
        WhenItApplies::AtTheNextRestart,
    );
    assert!(
        matches!(again, Err(NotApplied::Refused(NotStaged::AlreadyWaiting))),
        "{again:?}"
    );
    assert_eq!(base.switches().len(), 1);
}

/// **A base that will not stage the build leaves the machine as it was**, and
/// the person is told so in the vocabulary rather than in the base's words.
#[test]
fn a_base_that_will_not_stage_the_build_is_said_as_nothing_changed() {
    let base = TheBaseStandingIn {
        stages: false,
        ..TheBaseStandingIn::running(&build("aa"), None)
    };
    let refused = apply(
        &base,
        &ready("aa", "bb"),
        &the_source(),
        WhenItApplies::AtTheNextRestart,
    )
    .unwrap_err();
    assert!(
        matches!(refused, NotApplied::TheBaseDidNotStageIt(_)),
        "{refused:?}"
    );
    let said = refused.said(&the_machine());
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("nothing was changed"), "{said}");
    assert!(!said.text().contains("signature"), "{said}");
}

/// **A base that cannot say what is running refuses the update before it is
/// told anything.**
#[test]
fn a_machine_whose_running_build_cannot_be_read_is_not_updated() {
    let base = TheBaseStandingIn::running(&build("aa"), None);
    *base.status.borrow_mut() = r#"{"status":{"booted":{"image":null}}}"#.to_owned();
    let refused = apply(
        &base,
        &ready("aa", "bb"),
        &the_source(),
        WhenItApplies::AtTheNextRestart,
    )
    .unwrap_err();
    assert!(
        matches!(refused, NotApplied::Refused(NotStaged::NotRunningABuild)),
        "{refused:?}"
    );
    *base.status.borrow_mut() = "half an answer".to_owned();
    let refused = apply(
        &base,
        &ready("aa", "bb"),
        &the_source(),
        WhenItApplies::AtTheNextRestart,
    )
    .unwrap_err();
    assert!(
        matches!(refused, NotApplied::NotRead(NotRead::NotUnderstood { .. })),
        "{refused:?}"
    );
    assert!(base.switches().is_empty());
    assert!(!refused.said(&the_machine()).is_a_bug());
}

/// **What is running is asked of the base every time**, so the answer after an
/// update is the new build and never a remembered one.
#[test]
fn the_running_build_is_the_bases_answer_at_the_moment_it_is_asked() {
    let base = TheBaseStandingIn::running(&build("aa"), Some(&build("bb")));
    assert_eq!(running(&base).unwrap().digest(), &digest("aa"));
    assert_eq!(deployments(&base).unwrap().staged(), Some(&digest("bb")));
    base.now_running(&build("bb"));
    assert_eq!(running(&base).unwrap().digest(), &digest("bb"));
    assert_eq!(base.asked.borrow().len(), 3);
}

/// **The record says the machine updated, from which build to which, with no
/// agent** — once, at the first start on the new build, and never at the first
/// start of all or on the same build again.
#[test]
fn the_record_says_the_machine_updated_from_which_build_to_which_with_no_agent() {
    let folder = a_folder("record");
    let record_at = folder.join("record.jsonl");
    let kept = AcrossRestarts::in_folder(&folder);
    let mut record = Writing::opening(&record_at).unwrap();
    let base = TheBaseStandingIn::running(&build("aa"), None);

    let first = after_a_restart(&base, &kept, &mut record, noon()).unwrap();
    assert_eq!(first, Since::FirstKnown(digest("aa")));
    assert!(
        entries(&record_at).is_empty(),
        "the first start wrote an update"
    );

    let again = after_a_restart(&base, &kept, &mut record, noon()).unwrap();
    assert_eq!(again, Since::Unchanged(digest("aa")));
    assert!(entries(&record_at).is_empty());

    base.now_running(&build("bb"));
    let updated = after_a_restart(&base, &kept, &mut record, noon()).unwrap();
    assert_eq!(
        updated,
        Since::Updated {
            from: digest("aa"),
            to: digest("bb"),
        }
    );
    let written = entries(&record_at);
    assert_eq!(written.len(), 1);
    let entry = written.first().unwrap();
    assert_eq!(entry.agent(), None);
    assert_eq!(entry.at(), noon());
    assert!(!entry.happened().caused_egress());
    assert!(
        matches!(
            entry.happened(),
            Happened::Updated { from, to } if from.is(&build("aa")) && to.is(&build("bb"))
        ),
        "{entry:?}"
    );

    assert_eq!(
        after_a_restart(&base, &kept, &mut record, noon()).unwrap(),
        Since::Unchanged(digest("bb"))
    );
    assert_eq!(
        entries(&record_at).len(),
        1,
        "one update became two entries"
    );
    let _ = std::fs::remove_dir_all(&folder);
}

/// **A last known build that cannot be read is refused**, rather than read as
/// nothing known — which would write no entry for an update that happened —
/// and nothing is written over it.
#[test]
fn a_last_known_build_that_cannot_be_read_is_refused_and_not_written_over() {
    let folder = a_folder("unreadable");
    let record_at = folder.join("record.jsonl");
    let kept = AcrossRestarts::in_folder(&folder);
    let last_known_at = kept.last_known().to_path_buf();
    std::fs::write(&last_known_at, "sha256:beef\n").unwrap();
    let mut record = Writing::opening(&record_at).unwrap();
    let base = TheBaseStandingIn::running(&build("bb"), None);

    let refused = after_a_restart(&base, &kept, &mut record, noon()).unwrap_err();
    assert!(
        matches!(refused, NotRecorded::LastKnownNotRead { .. }),
        "{refused:?}"
    );
    assert_eq!(
        std::fs::read_to_string(&last_known_at).unwrap(),
        "sha256:beef\n"
    );
    assert!(entries(&record_at).is_empty());
    let said = refused.said(&the_machine());
    assert!(!said.is_a_bug(), "{said}");
    let _ = std::fs::remove_dir_all(&folder);
}

/// **The entry comes first, and the last known build second**: a machine that
/// cannot keep the new build writes the update again at its next start rather
/// than losing it.
#[test]
fn an_update_whose_build_could_not_be_kept_is_written_again_rather_than_lost() {
    let folder = a_folder("order");
    let record_at = folder.join("record.jsonl");
    let kept = AcrossRestarts::in_folder(&folder);
    let last_known_at = kept.last_known().to_path_buf();
    std::fs::write(&last_known_at, format!("{}\n", build("aa"))).unwrap();
    let mut record = Writing::opening(&record_at).unwrap();
    let base = TheBaseStandingIn::running(&build("bb"), None);

    // A folder where the new value would be written makes keeping it fail.
    let beside = folder.join("last-known-build.new");
    std::fs::create_dir(&beside).unwrap();
    let refused = after_a_restart(&base, &kept, &mut record, noon()).unwrap_err();
    assert!(
        matches!(refused, NotRecorded::LastKnownNotKept { .. }),
        "{refused:?}"
    );
    assert_eq!(
        entries(&record_at).len(),
        1,
        "the update was not written first"
    );
    assert_eq!(
        std::fs::read_to_string(&last_known_at).unwrap(),
        format!("{}\n", build("aa"))
    );

    std::fs::remove_dir(&beside).unwrap();
    let since = after_a_restart(&base, &kept, &mut record, noon()).unwrap();
    assert!(matches!(since, Since::Updated { .. }), "{since:?}");
    assert_eq!(entries(&record_at).len(), 2);
    assert_eq!(
        after_a_restart(&base, &kept, &mut record, noon()).unwrap(),
        Since::Unchanged(digest("bb"))
    );
    let _ = std::fs::remove_dir_all(&folder);
}

/// **A base that cannot be asked writes nothing down.**
#[test]
fn a_base_that_cannot_be_asked_writes_nothing_down() {
    let folder = a_folder("no-base");
    let record_at = folder.join("record.jsonl");
    let kept = AcrossRestarts::in_folder(&folder);
    let last_known_at = kept.last_known().to_path_buf();
    let mut record = Writing::opening(&record_at).unwrap();
    let base = alo_updating::TheBase::at(&folder.join("no-bootc-here"));
    let refused = after_a_restart(&base, &kept, &mut record, noon()).unwrap_err();
    assert!(
        matches!(refused, NotRecorded::NotRead(NotRead::NotAnswered(_))),
        "{refused:?}"
    );
    assert!(!last_known_at.exists());
    assert!(entries(&record_at).is_empty());
    assert!(!refused.said(&the_machine()).is_a_bug());
    let _ = std::fs::remove_dir_all(&folder);
}
