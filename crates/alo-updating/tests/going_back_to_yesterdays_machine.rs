//! Yesterday's machine named, going back to it, and the record saying the
//! machine went back — against a stand-in for the base that writes down every
//! question, so each refusal can be shown to have told the base nothing.
//!
//! What a real base does with the same instruction on a real boot is
//! `tests/back_to_yesterdays_machine.rs`, in a virtual machine.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use alo_keeping::{Reading, Writing};
use alo_keeping_up::{CannotGoBack, Digest, Since, WhenItApplies};
use alo_record::{Entry, Happened};
use alo_saying::everything_this_machine_can_say;
use alo_strings::Strings;
use alo_updating::{
    AcrossRestarts, Base, NotAnswered, NotGoneBack, NotRecorded, after_a_restart, go_back,
    yesterday,
};

/// What the stand-in reports: a build booted, maybe one waiting, maybe one
/// before, and whether the one before starts next.
#[derive(Clone)]
struct Disk {
    booted: String,
    staged: Option<String>,
    rollback: Option<String>,
    queued: bool,
}

/// A stand-in for the base: answers its status from the disk it holds, sets
/// going back when told `rollback` (as the base does, setting aside anything
/// waiting), and writes down every question.
struct TheBaseStandingIn {
    disk: RefCell<Disk>,
    /// Whether `rollback` succeeds.
    goes_back: bool,
    asked: RefCell<Vec<Vec<String>>>,
}

impl TheBaseStandingIn {
    fn with(booted: &str, staged: Option<&str>, rollback: Option<&str>) -> Self {
        Self {
            disk: RefCell::new(Disk {
                booted: build(booted),
                staged: staged.map(build),
                rollback: rollback.map(build),
                queued: false,
            }),
            goes_back: true,
            asked: RefCell::new(Vec::new()),
        }
    }

    /// The person restarts: the base starts the build before if going back is
    /// set, and keeps the one it left as the build before.
    fn restarted(&self) {
        let mut disk = self.disk.borrow_mut();
        if disk.queued {
            let left = disk.booted.clone();
            disk.booted = disk.rollback.take().unwrap();
            disk.rollback = Some(left);
            disk.queued = false;
        }
    }

    fn rollbacks(&self) -> usize {
        self.asked
            .borrow()
            .iter()
            .filter(|asked| asked.first().is_some_and(|verb| verb == "rollback"))
            .count()
    }
}

impl Base for TheBaseStandingIn {
    fn asked(&self, arguments: &[String]) -> Result<Vec<u8>, NotAnswered> {
        self.asked.borrow_mut().push(arguments.to_vec());
        match arguments.first().map(String::as_str) {
            Some("status") => Ok(a_status(&self.disk.borrow()).into_bytes()),
            Some("rollback") if self.goes_back => {
                let mut disk = self.disk.borrow_mut();
                disk.queued = true;
                disk.staged = None;
                Ok(Vec::new())
            }
            _ => Err(NotAnswered::SaidNo {
                program: "bootc".to_owned(),
                code: Some(1),
                said: "error: No rollback available".to_owned(),
            }),
        }
    }
}

fn a_status(disk: &Disk) -> String {
    let entry = |digest: &String| {
        format!(
            r#"{{"image":{{"image":{{"image":"ghcr.io/aloworld-org/alo-os"}},"imageDigest":"{digest}"}}}}"#
        )
    };
    let or_null =
        |digest: &Option<String>| digest.as_ref().map_or_else(|| "null".to_owned(), entry);
    format!(
        r#"{{"status":{{"booted":{},"staged":{},"rollback":{},"rollbackQueued":{}}}}}"#,
        entry(&disk.booted),
        or_null(&disk.staged),
        or_null(&disk.rollback),
        disk.queued
    )
}

fn build(pair: &str) -> String {
    format!("sha256:{}", pair.repeat(32))
}

fn digest(pair: &str) -> Digest {
    Digest::read(&build(pair)).unwrap()
}

fn at(hours: u64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(60 * 60 * hours)
}

fn the_machine() -> Strings {
    Strings::of(everything_this_machine_can_say().unwrap())
}

fn a_folder(named: &str) -> PathBuf {
    let folder =
        std::env::temp_dir().join(format!("alo-going-back-{named}-{}", std::process::id()));
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

/// A machine that updated from `aa` to `bb` at noon, as the record and the
/// last known build say, with `aa` still kept.
fn after_an_update(named: &str) -> (PathBuf, AcrossRestarts, Writing, TheBaseStandingIn) {
    let folder = a_folder(named);
    let record_at = folder.join("record.jsonl");
    let kept = AcrossRestarts::in_folder(&folder);
    let mut record = Writing::opening(&record_at).unwrap();
    let base = TheBaseStandingIn::with("aa", None, None);
    after_a_restart(&base, &kept, &mut record, at(1)).unwrap();
    *base.disk.borrow_mut() = Disk {
        booted: build("bb"),
        staged: None,
        rollback: Some(build("aa")),
        queued: false,
    };
    assert!(matches!(
        after_a_restart(&base, &kept, &mut record, at(12)).unwrap(),
        Since::Updated { .. }
    ));
    (record_at, kept, record, base)
}

/// **After an update, the build before is named, dated and kept; going back to
/// it is one instruction; and the first start on it is recorded as a return,
/// with no agent, once.**
#[test]
fn going_back_after_an_update_is_named_set_with_one_instruction_and_recorded_once() {
    let (record_at, kept, mut record, base) = after_an_update("legitimate");

    let yesterday_now = yesterday(&base, &record_at).unwrap();
    assert_eq!(yesterday_now.running(), &digest("bb"));
    let before = yesterday_now.before().unwrap();
    assert_eq!(before.build(), &digest("aa"));
    assert!(before.is_kept());
    assert_eq!(yesterday_now.replaced_at(), Some(at(12)));
    assert_eq!(yesterday_now.record_not_read(), None);
    let offer = yesterday_now.going_back().unwrap().clone();
    let said = offer.said(&the_machine());
    assert!(!said.is_a_bug(), "{said}");

    base.asked.borrow_mut().clear();
    let returning = go_back(&base, &offer, &kept, WhenItApplies::AtTheNextRestart).unwrap();
    assert_eq!(returning.to(), &digest("aa"));
    assert_eq!(
        *base.asked.borrow(),
        [
            vec!["status", "--format", "json", "--format-version", "1"],
            vec!["rollback"],
        ]
        .map(|asked| asked.into_iter().map(str::to_owned).collect::<Vec<_>>())
    );
    assert_eq!(
        std::fs::read_to_string(kept.going_back_to()).unwrap(),
        format!("{}\n", build("aa"))
    );
    assert!(!returning.said(&the_machine()).is_a_bug());

    // Before the restart nothing has happened, and the note waits.
    assert_eq!(
        after_a_restart(&base, &kept, &mut record, at(13)).unwrap(),
        Since::Unchanged(digest("bb"))
    );
    assert!(kept.going_back_to().exists());
    assert_eq!(entries(&record_at).len(), 1);

    base.restarted();
    assert_eq!(
        after_a_restart(&base, &kept, &mut record, at(14)).unwrap(),
        Since::RolledBack {
            from: digest("bb"),
            to: digest("aa"),
        }
    );
    let written = entries(&record_at);
    assert_eq!(written.len(), 2);
    let entry = written.last().unwrap();
    assert_eq!(entry.agent(), None);
    assert_eq!(entry.at(), at(14));
    assert!(!entry.happened().caused_egress());
    assert!(
        matches!(
            entry.happened(),
            Happened::RolledBack { from, to } if from.is(&build("bb")) && to.is(&build("aa"))
        ),
        "{entry:?}"
    );
    assert!(
        !kept.going_back_to().exists(),
        "the note outlived the return"
    );

    assert_eq!(
        after_a_restart(&base, &kept, &mut record, at(15)).unwrap(),
        Since::Unchanged(digest("aa"))
    );
    assert_eq!(
        entries(&record_at).len(),
        2,
        "one return became two entries"
    );

    // And yesterday is now the build it left, replaced when it went back.
    let yesterday_after = yesterday(&base, &record_at).unwrap();
    assert_eq!(yesterday_after.before().unwrap().build(), &digest("bb"));
    assert_eq!(yesterday_after.replaced_at(), Some(at(14)));
    assert!(yesterday_after.going_back().is_ok());
}

/// **Restarting now adds `--apply`, and nothing else.**
#[test]
fn going_back_now_is_the_one_instruction_with_apply() {
    let (record_at, kept, _record, base) = after_an_update("now");
    let offer = yesterday(&base, &record_at)
        .unwrap()
        .going_back()
        .unwrap()
        .clone();
    base.asked.borrow_mut().clear();
    go_back(
        &base,
        &offer,
        &kept,
        WhenItApplies::NowBecauseThePersonAsked,
    )
    .unwrap();
    assert_eq!(
        base.asked.borrow().last().unwrap(),
        &["rollback".to_owned(), "--apply".to_owned()]
    );
}

/// **One approval, one execution**: going back already set is refused, and the
/// base is not turned round a second time.
#[test]
fn going_back_already_set_is_refused_and_the_base_is_told_once() {
    let (record_at, kept, _record, base) = after_an_update("twice");
    let offer = yesterday(&base, &record_at)
        .unwrap()
        .going_back()
        .unwrap()
        .clone();
    go_back(&base, &offer, &kept, WhenItApplies::AtTheNextRestart).unwrap();
    let again = go_back(&base, &offer, &kept, WhenItApplies::AtTheNextRestart);
    assert_eq!(
        again,
        Err(NotGoneBack::Refused(CannotGoBack::AlreadyGoingBack))
    );
    assert_eq!(base.rollbacks(), 1);
    assert_eq!(
        yesterday(&base, &record_at).unwrap().going_back(),
        Err(&CannotGoBack::AlreadyGoingBack)
    );
    assert!(!again.unwrap_err().said(&the_machine()).is_a_bug());
}

/// **A machine that changed since going back was offered is refused**, nothing
/// is noted, and the base is told nothing.
#[test]
fn a_machine_that_changed_since_the_offer_is_refused_and_nothing_is_noted() {
    let (record_at, kept, _record, base) = after_an_update("changed");
    let offer = yesterday(&base, &record_at)
        .unwrap()
        .going_back()
        .unwrap()
        .clone();
    // An update started waiting after the offer: going back would set it
    // aside without the person having been told.
    base.disk.borrow_mut().staged = Some(build("cc"));
    let refused = go_back(&base, &offer, &kept, WhenItApplies::AtTheNextRestart).unwrap_err();
    assert_eq!(
        refused,
        NotGoneBack::Refused(CannotGoBack::ChangedSinceItWasOffered)
    );
    assert_eq!(base.rollbacks(), 0);
    assert!(!kept.going_back_to().exists());
    let said = refused.said(&the_machine());
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("nothing was changed"), "{said}");

    // Offered again, the sentence now says the waiting update will not apply.
    let offer_again = yesterday(&base, &record_at).unwrap();
    let said = offer_again.going_back().unwrap().said(&the_machine());
    assert!(said.text().contains("will not apply"), "{said}");
}

/// **A return that cannot be done says so before it is offered**: nothing
/// before, or the build before no longer on the disk — each named in the
/// vocabulary, and the base is asked only its status.
#[test]
fn a_return_that_cannot_be_done_is_said_before_it_is_offered() {
    let folder = a_folder("cannot");
    let record_at = folder.join("record.jsonl");
    let mut record = Writing::opening(&record_at).unwrap();

    let first = TheBaseStandingIn::with("aa", None, None);
    let nothing = yesterday(&first, &record_at).unwrap();
    assert_eq!(nothing.before(), None);
    assert_eq!(nothing.going_back(), Err(&CannotGoBack::NothingBefore));

    record
        .keep(&Entry::updated(&build("aa"), &build("bb"), at(12)))
        .unwrap();
    let gone = TheBaseStandingIn::with("bb", None, None);
    let named = yesterday(&gone, &record_at).unwrap();
    let before = named.before().unwrap();
    assert_eq!(before.build(), &digest("aa"));
    assert!(!before.is_kept());
    assert_eq!(named.replaced_at(), Some(at(12)));
    let refused = named.going_back().unwrap_err();
    assert_eq!(
        refused,
        &CannotGoBack::NoLongerKept {
            build: digest("aa")
        }
    );
    let said = refused.said(&the_machine());
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("no longer kept"), "{said}");

    for base in [first, gone] {
        assert_eq!(base.rollbacks(), 0);
    }
    let _ = std::fs::remove_dir_all(&folder);
}

/// **A build to go back to that cannot be noted stops going back before the
/// base is told**, because without the note the return would be recorded as
/// an update.
#[test]
fn going_back_that_cannot_be_noted_tells_the_base_nothing() {
    let (record_at, kept, _record, base) = after_an_update("not-noted");
    let offer = yesterday(&base, &record_at)
        .unwrap()
        .going_back()
        .unwrap()
        .clone();
    let mut beside = kept.going_back_to().as_os_str().to_owned();
    beside.push(".new");
    std::fs::create_dir(PathBuf::from(beside)).unwrap();
    let refused = go_back(&base, &offer, &kept, WhenItApplies::AtTheNextRestart).unwrap_err();
    assert!(
        matches!(refused, NotGoneBack::NotNoted { .. }),
        "{refused:?}"
    );
    assert_eq!(base.rollbacks(), 0);
    let said = refused.said(&the_machine());
    assert!(!said.is_a_bug(), "{said}");
    assert!(said.text().contains("nothing was changed"), "{said}");
}

/// **A base that will not go back leaves the machine as it was**, said in the
/// vocabulary; the note waits harmlessly, and the next change clears it without
/// calling that change a return.
#[test]
fn a_base_that_will_not_go_back_is_said_as_nothing_changed_and_the_note_does_no_harm() {
    let (record_at, kept, mut record, mut base) = after_an_update("base-refuses");
    base.goes_back = false;
    let offer = yesterday(&base, &record_at)
        .unwrap()
        .going_back()
        .unwrap()
        .clone();
    let refused = go_back(&base, &offer, &kept, WhenItApplies::AtTheNextRestart).unwrap_err();
    assert!(
        matches!(refused, NotGoneBack::TheBaseDidNotSetIt(_)),
        "{refused:?}"
    );
    let said = refused.said(&the_machine());
    assert!(!said.is_a_bug(), "{said}");
    assert!(!said.text().contains("rollback"), "{said}");

    base.restarted();
    assert_eq!(
        after_a_restart(&base, &kept, &mut record, at(13)).unwrap(),
        Since::Unchanged(digest("bb"))
    );
    *base.disk.borrow_mut() = Disk {
        booted: build("cc"),
        staged: None,
        rollback: Some(build("bb")),
        queued: false,
    };
    assert_eq!(
        after_a_restart(&base, &kept, &mut record, at(14)).unwrap(),
        Since::Updated {
            from: digest("bb"),
            to: digest("cc"),
        }
    );
    assert!(!kept.going_back_to().exists());
}

/// **A note that cannot be read is refused**, rather than read as no return
/// chosen — which would write a return as an update — and nothing is written.
#[test]
fn a_note_that_cannot_be_read_writes_nothing_down() {
    let (record_at, kept, mut record, base) = after_an_update("unreadable-note");
    std::fs::write(kept.going_back_to(), "sha256:beef\n").unwrap();
    *base.disk.borrow_mut() = Disk {
        booted: build("aa"),
        staged: None,
        rollback: Some(build("bb")),
        queued: false,
    };
    let refused = after_a_restart(&base, &kept, &mut record, at(14)).unwrap_err();
    assert!(
        matches!(refused, NotRecorded::GoingBackNotRead { .. }),
        "{refused:?}"
    );
    assert_eq!(entries(&record_at).len(), 1);
    assert_eq!(
        std::fs::read_to_string(kept.last_known()).unwrap(),
        format!("{}\n", build("bb"))
    );
    assert!(!refused.said(&the_machine()).is_a_bug());
}

/// **A record that cannot be read leaves only *when* unknown**: the build
/// before is still named from the disk, and going back is still offered.
#[test]
fn a_record_that_cannot_be_read_leaves_only_when_unknown() {
    let folder = a_folder("no-record");
    let base = TheBaseStandingIn::with("bb", None, Some("aa"));
    let looked = yesterday(&base, &folder.join("record.jsonl")).unwrap();
    assert_eq!(looked.before().unwrap().build(), &digest("aa"));
    assert_eq!(looked.replaced_at(), None);
    assert!(looked.record_not_read().is_some());
    assert!(looked.going_back().is_ok());
    let _ = std::fs::remove_dir_all(&folder);
}
