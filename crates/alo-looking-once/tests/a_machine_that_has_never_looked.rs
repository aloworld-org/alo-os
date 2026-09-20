//! A machine that has never looked — the acceptance of task 10 of
//! `docs/autonomy/v0-5-the-machine-keeps-itself-plan.md`, one criterion at a
//! time.
//!
//! | The acceptance | The test |
//! |---|---|
//! | `Because::ThisMachineStarted` has exactly one caller on a booted machine | [`there_is_exactly_one_caller_of_a_check_at_a_start`] |
//! | it happens once per start and never again — no timer, no thread, no repetition | [`nothing_here_could_check_a_second_time`] |
//! | where it runs and under whose privilege is decided and written down | [`the_unit_runs_as_the_person_at_a_start_and_holds_nothing`] |
//! | the check is on the indicator for the whole of it and in the record afterwards | [`the_check_is_shown_while_it_happens_and_written_down_afterwards`] |
//! | a machine with no way out says so once, which `SaidOnce` decides and this must not undo | [`a_machine_with_no_way_out_says_so_once_and_keeps_nothing`] |
//! | a check at a start never delays a sign-in, takes focus or asks anything | [`nothing_here_waits_for_anybody_or_asks_them_anything`] |
//! | nothing downloads a build, and no setting turns checking off | [`nothing_here_fetches_a_build_or_turns_checking_off`] |
//!
//! The last criterion — the whole of it measured on a booted machine, started
//! twice, with the kept answer, the record and the departures counted at the
//! network boundary — is not a test in this suite: it needs a machine that
//! boots. It is in `docs/autonomy/updates/a-machine-that-has-never-looked.md`.
//!
//! # The refusal paths are here beside the answers
//!
//! Every road out of [`alo_looking_once::AtAStart::look_once`] is exercised: a
//! base that will not say what is running, a record that cannot be opened, a
//! record that is not a record, a place that refuses each of its five ways, and
//! an answer that cannot be kept. A check is the one thing on this machine that
//! happens with nobody watching, and the roads where it fails are the ones
//! nobody would otherwise see.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use alo_image::{Service, Unit};
use alo_keeping_up::Digest;
use alo_looking::{Because, NoAnswer, Place, Release, SaidOnce, ThePlace};
use alo_looking_once::{
    AtAStart, DidNotLook, THE_ANSWER, THE_PROGRAM, THE_RECORD, THE_UNIT, THE_UNITS_NAME,
};
use alo_record::{Asking, Only};
use alo_updating::{Base, NotAnswered};

/// Where this crate's own source is, for the tests that read it.
const THE_SOURCE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/src");

/// A moment. The program reads a clock once; nothing being tested does.
fn a_moment() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(60 * 60 * 12)
}

/// A whole build, from one repeated pair.
fn build(pair: &str) -> Digest {
    Digest::read(&format!("sha256:{}", pair.repeat(32))).unwrap()
}

/// A folder of this machine's own, emptied first.
fn a_folder(named: &str) -> PathBuf {
    let folder =
        std::env::temp_dir().join(format!("alo-looking-once-{named}-{}", std::process::id()));
    drop(std::fs::remove_dir_all(&folder));
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

/// The place this machine's build was pinned to come from.
fn the_place() -> Place {
    Place::on_this_machine().expect("the pin this repository ships")
}

/// Every file of this crate's source, as text.
fn every_source_file() -> Vec<(PathBuf, String)> {
    let mut read = Vec::new();
    for entry in std::fs::read_dir(THE_SOURCE).expect("this crate's source") {
        let path = entry.expect("a source file").path();
        let text = std::fs::read_to_string(&path).expect("a source file");
        read.push((path, text));
    }
    assert!(
        read.len() >= 5,
        "only {} source files were read",
        read.len()
    );
    read
}

/// The unit this crate hands over, read with the reader the image's own units
/// are read with.
fn the_unit() -> Service {
    Service::of(THE_UNITS_NAME, Unit::read(THE_UNIT).expect("a unit")).expect("a service")
}

/// The record this check wrote, read back.
fn what_the_record_says(at: &AtAStart) -> alo_record::Record {
    alo_keeping::Reading::at(at.record())
        .expect("the record this check wrote")
        .record()
        .clone()
}

/// A base that says this machine is running a build, or will not say at all.
struct ABaseThatSays {
    /// What it answers with, where it answers.
    running: Option<Digest>,
}

impl ABaseThatSays {
    /// A base reporting this build as the one booted.
    fn running(build: &Digest) -> Self {
        Self {
            running: Some(build.clone()),
        }
    }

    /// A base that answers nothing at all — a machine that did not boot from an
    /// image, or one where the base is not there.
    fn nothing() -> Self {
        Self { running: None }
    }
}

impl Base for ABaseThatSays {
    fn asked(&self, _: &[String]) -> Result<Vec<u8>, NotAnswered> {
        match &self.running {
            Some(build) => Ok(a_status_naming(build).into_bytes()),
            None => Err(NotAnswered::NotStarted {
                program: "a base that is not there".to_owned(),
                why: "there is no base on this machine".to_owned(),
            }),
        }
    }
}

/// A place that answers whatever a test needs it to.
struct APlaceThatAnswers {
    /// Every name it holds.
    names: Vec<String>,
    /// What it says each release is.
    builds: Vec<(String, String)>,
    /// What it refuses with instead of answering at all.
    refusing: Option<NoAnswer>,
    /// How often it has been asked anything.
    asked: Cell<usize>,
}

impl APlaceThatAnswers {
    /// A place offering this release as this build.
    fn offering(release: &str, build: &Digest) -> Self {
        Self {
            names: vec![release.to_owned()],
            builds: vec![(release.to_owned(), build.as_str().to_owned())],
            refusing: None,
            asked: Cell::new(0),
        }
    }

    /// A place that refuses in this way.
    fn that_answers_with(refusal: NoAnswer) -> Self {
        Self {
            names: Vec::new(),
            builds: Vec::new(),
            refusing: Some(refusal),
            asked: Cell::new(0),
        }
    }

    /// How often it has been asked anything.
    fn how_often_it_was_asked(&self) -> usize {
        self.asked.get()
    }
}

impl ThePlace for APlaceThatAnswers {
    fn every_name(&self) -> Result<Vec<String>, NoAnswer> {
        self.asked.set(self.asked.get() + 1);
        self.refusing.map_or_else(|| Ok(self.names.clone()), Err)
    }

    fn the_build_of(&self, release: &Release) -> Result<String, NoAnswer> {
        self.asked.set(self.asked.get() + 1);
        if let Some(refusal) = self.refusing {
            return Err(refusal);
        }
        self.builds
            .iter()
            .find(|(named, _)| named == release.named_as())
            .map(|(_, build)| build.clone())
            .ok_or(NoAnswer::ItRefused)
    }
}

/// A place offering the release this repository is pinned at, as a newer build.
fn a_place_with_something_newer(place: &Place) -> APlaceThatAnswers {
    APlaceThatAnswers::offering(place.not_before().named_as(), &build("bb"))
}

/// **`Because::ThisMachineStarted` has exactly one caller**, and this crate is
/// it.
///
/// Read out of the source rather than asserted in prose: *the machine started*
/// is the one occasion nobody is watching, and a second caller added anywhere
/// would be a second departure at every boot that nobody would notice for
/// months.
#[test]
fn there_is_exactly_one_caller_of_a_check_at_a_start() {
    let mut naming = Vec::new();
    for (path, text) in every_source_file() {
        let how_many = text.matches("Because::ThisMachineStarted").count();
        if how_many > 0 {
            naming.push((path, how_many));
        }
        assert!(
            !text.contains("Because::ThePersonAsked"),
            "a start is not a person asking"
        );
    }
    assert_eq!(naming.len(), 1, "{naming:?}");
    let (where_, how_many) = naming.first().expect("the one file naming it");
    assert_eq!(*how_many, 1, "{naming:?}");
    assert!(
        where_.ends_with("once.rs"),
        "{where_:?} is not where the one act is"
    );

    // And both occasions still exist, so that a crate which quietly became the
    // only way to check would fail here rather than pass.
    assert_eq!(Because::EVERY.len(), 2);
    assert!(!Because::ThisMachineStarted.somebody_is_waiting());
}

/// **Nothing here could check a second time.** No thread, no timer, no loop, no
/// retry — and the unit that runs it says the same thing from the other end.
#[test]
fn nothing_here_could_check_a_second_time() {
    for (path, text) in every_source_file() {
        for watching in [
            "Instant::now",
            "std::thread",
            "thread::spawn",
            "::sleep(",
            "set_timeout",
            "Interval",
            "async fn",
            ".await",
            "std::sync::mpsc",
            "loop {",
            "while ",
        ] {
            assert!(
                !text.contains(watching),
                "{} names {watching}, which is the first line of a watcher",
                path.display()
            );
        }
    }

    // The unit cannot repeat it either: there is no timer beside it, nothing
    // restarts it, and a machine that has looked stays looked.
    let unit = the_unit();
    assert_eq!(
        unit.kind(),
        Some("exec"),
        "a oneshot's start job is not finished until it has exited, and a unit wanted by a target \
         is ordered before it — which puts a check on the boot's critical path"
    );
    assert!(unit.stays_after_exiting());
    for repeating in ["[Timer]", "OnCalendar", "OnUnitActiveSec", "\nRestart="] {
        assert!(!THE_UNIT.contains(repeating), "the unit names {repeating}");
    }
}

/// **Where it runs and under whose privilege**, read off the unit rather than
/// out of a paragraph.
///
/// The person's login, at a start, holding nothing. The whole argument is in
/// the unit's own comments and in this crate's; this is the half a change can
/// be caught by.
#[test]
fn the_unit_runs_as_the_person_at_a_start_and_holds_nothing() {
    let unit = the_unit();

    assert_eq!(unit.runs(), THE_PROGRAM);
    assert_eq!(
        unit.as_login(),
        Some("alo"),
        "it writes two files that are the person's, in a folder that is 0700 theirs"
    );
    assert_ne!(unit.as_login(), Some(alo_image::ROOT));
    assert_eq!(unit.in_group(), Some("alo"));
    assert!(
        unit.holds_nothing(),
        "asking a registry a question needs no privilege at all"
    );
    assert!(unit.bounded_to().is_empty());
    assert!(unit.given().is_empty());

    // Started at boot, not at a sign-in: the answer it keeps is one per machine
    // and a session is not one per machine.
    assert!(unit.wanted_by().contains(&"multi-user.target"));
    assert!(
        !THE_UNIT.contains("user@"),
        "a session hook would check once per sign-in"
    );

    // And where it writes is where the image already makes a folder for the
    // person, rather than somewhere this crate invented.
    assert!(THE_ANSWER.starts_with("/var/lib/alo/"));
    assert!(THE_RECORD.starts_with("/var/lib/alo/"));
    assert_ne!(
        THE_RECORD, "/var/lib/alo/record.jsonl",
        "the person's record already has one writer"
    );
    assert!(THE_UNIT.contains("ReadWritePaths=/var/lib/alo"));
    assert!(THE_PROGRAM.starts_with("/usr/libexec/"));
}

/// **A check at a start never delays a person's sign-in, takes focus or asks
/// them anything.** `THE_RULE` is about an update and this is the thing that
/// finds one, so the same promise holds here.
#[test]
fn nothing_here_waits_for_anybody_or_asks_them_anything() {
    // Nothing is ordered before this unit's start job, so nothing in the boot
    // waits for the answer — and its own ordering names nothing a person is
    // waiting at. Read off the parsed unit rather than out of its text, because
    // its comments name `graphical.target` in the course of saying why the
    // check is deliberately not on the way to it.
    let unit = the_unit();
    assert!(
        unit.before().is_empty(),
        "a unit ordered before something is a unit something waits for: {:?}",
        unit.before()
    );
    for waiting in ["systemd-user-sessions.service", "display-manager.service"] {
        assert!(
            !unit.after().contains(&waiting),
            "the unit is ordered after {waiting}, which is where a person waits"
        );
    }
    // And a oneshot here would put the check itself on the boot's critical
    // chain, whatever the ordering says. Measured; see the unit's own comments.
    assert_ne!(unit.kind(), Some("oneshot"));

    // And nothing in the program reads from anybody.
    for (path, text) in every_source_file() {
        for asking in ["stdin()", "read_line", "prompt"] {
            assert!(
                !text.contains(asking),
                "{} names {asking}, and a check asks nobody anything",
                path.display()
            );
        }
    }
}

/// **Nothing downloads a build, and no setting turns checking off.** Task 6's
/// constraint and task 1's, held here as well as where they were made — this is
/// the crate that could most easily break either without anybody noticing.
#[test]
fn nothing_here_fetches_a_build_or_turns_checking_off() {
    for (path, text) in every_source_file() {
        for fetching in ["blobs", "oci-archive", "podman", "skopeo", "Staging::"] {
            assert!(
                !text.contains(fetching),
                "{} names {fetching}, and a check fetches an answer and never a build",
                path.display()
            );
        }
        for turning_off in ["disable", "opt_out", "turned off"] {
            assert!(
                !text.contains(turning_off),
                "{} names {turning_off}, and there is no setting that turns checking off",
                path.display()
            );
        }
    }
    assert!(
        !THE_UNIT.contains("ConditionPathExists"),
        "a file that switches the unit off is a setting that turns checking off"
    );
}

/// **The whole act: shown while it happens, written down afterwards, kept where
/// a surface reads it back.**
#[test]
fn the_check_is_shown_while_it_happens_and_written_down_afterwards() {
    let folder = a_folder("whole");
    let at = AtAStart::in_folder(&folder);
    let place = the_place();
    let asked = a_place_with_something_newer(&place);

    let found = at
        .look_once(
            &ABaseThatSays::running(&build("aa")),
            &place,
            &asked,
            &mut SaidOnce::new(),
            a_moment(),
        )
        .unwrap();

    assert!(found.is_ready());
    assert_eq!(found.because(), Because::ThisMachineStarted);
    assert_eq!(
        asked.how_often_it_was_asked(),
        2,
        "one check, two questions"
    );

    // The answer is there to be read back without asking the place again.
    let kept = at.answer().read().unwrap().unwrap();
    assert_eq!(kept.because(), Because::ThisMachineStarted);
    assert_eq!(kept.about(), &build("aa"));

    // And the one departure is in the record, as an errand nobody was behind.
    let record = what_the_record_says(&at);
    assert_eq!(
        record
            .answering(&Asking::anything().only(Only::OnItsOwn))
            .count(),
        1
    );
    let entry = record.everything().next().expect("the one entry");
    assert_eq!(
        entry.happened().errand(),
        Some(alo_egress::Errand::CheckingForAnUpdate)
    );
    assert_eq!(entry.at(), a_moment());
    assert_eq!(
        record.answering(&Asking::anything().by("alo OS")).count(),
        0
    );

    drop(std::fs::remove_dir_all(&folder));
}

/// **A refused check is written down too**, and keeps nothing.
///
/// A record with a gap exactly where a check failed is the shape law 1 cannot
/// afford: a person asking *what did this machine do on its own?* after a week
/// of failed checks would be told *nothing*.
#[test]
fn a_refused_check_is_written_down_and_keeps_nothing() {
    for refusal in NoAnswer::EVERY {
        let folder = a_folder("refused");
        let at = AtAStart::in_folder(&folder);
        let refused = at
            .look_once(
                &ABaseThatSays::running(&build("aa")),
                &the_place(),
                &APlaceThatAnswers::that_answers_with(refusal),
                &mut SaidOnce::new(),
                a_moment(),
            )
            .unwrap_err();

        assert!(
            matches!(refused, DidNotLook::NothingFoundOut(said) if said == refusal),
            "{refused:?}"
        );
        assert!(!refused.nothing_left_this_machine());
        assert_eq!(
            at.answer().read().unwrap(),
            None,
            "{refusal:?} kept an answer"
        );
        assert_eq!(
            what_the_record_says(&at)
                .answering(&Asking::anything().only(Only::Egress))
                .count(),
            1,
            "{refusal:?} left no record"
        );
        drop(std::fs::remove_dir_all(&folder));
    }
}

/// **A machine with no way out says so once**, and a second start says it again
/// — because `SaidOnce` deliberately does not remember across one.
///
/// This is the criterion that must not be *undone*, and the way to undo it
/// would be to write the silence to a disk: a machine that remembered would
/// stay quiet about a network that was down last week. So the memory lasts as
/// long as whatever holds it, and a start keeps nothing about a refusal for a
/// surface to repeat at anybody.
#[test]
fn a_machine_with_no_way_out_says_so_once_and_keeps_nothing() {
    let folder = a_folder("no-way-out");
    let at = AtAStart::in_folder(&folder);
    let place = the_place();
    let nowhere = APlaceThatAnswers::that_answers_with(NoAnswer::NoWayOut);
    let mut said = SaidOnce::new();

    let first = at
        .look_once(
            &ABaseThatSays::running(&build("aa")),
            &place,
            &nowhere,
            &mut said,
            a_moment(),
        )
        .unwrap_err();
    assert!(
        matches!(first, DidNotLook::NothingFoundOut(NoAnswer::NoWayOut)),
        "{first:?}"
    );

    // Asked again while the same memory is held, there is nothing left to word.
    let again = at
        .look_once(
            &ABaseThatSays::running(&build("aa")),
            &place,
            &nowhere,
            &mut said,
            a_moment(),
        )
        .unwrap_err();
    assert!(matches!(again, DidNotLook::SaidAlready), "{again:?}");
    assert!(said.has_said_it());

    // Nothing was kept either time, so no surface has a stale line to put in
    // front of anybody at the next start.
    assert_eq!(at.answer().read().unwrap(), None);
    drop(std::fs::remove_dir_all(&folder));
}

/// **A machine whose base will not say what it is running asks nothing**, and
/// nothing leaves it.
#[test]
fn a_machine_whose_base_will_not_say_what_it_runs_asks_nothing() {
    let folder = a_folder("no-base");
    let at = AtAStart::in_folder(&folder);
    let place = the_place();
    let asked = a_place_with_something_newer(&place);

    let refused = at
        .look_once(
            &ABaseThatSays::nothing(),
            &place,
            &asked,
            &mut SaidOnce::new(),
            a_moment(),
        )
        .unwrap_err();

    assert!(
        matches!(refused, DidNotLook::TheBaseWouldNotSay(_)),
        "{refused:?}"
    );
    assert!(refused.nothing_left_this_machine());
    assert_eq!(asked.how_often_it_was_asked(), 0, "the place was asked");
    assert!(!at.record().exists(), "a record was opened for no check");
    drop(std::fs::remove_dir_all(&folder));
}

/// **A machine that cannot write down what it is about to do does not do it.**
///
/// The record is opened before the first question, so there is no road through
/// this crate that reaches the network without somewhere to account for it
/// afterwards.
#[test]
fn a_machine_that_cannot_open_its_record_asks_nothing() {
    let at = AtAStart::in_folder(Path::new("/nowhere/at/all/alo-looking-once"));
    let place = the_place();
    let asked = a_place_with_something_newer(&place);

    let refused = at
        .look_once(
            &ABaseThatSays::running(&build("aa")),
            &place,
            &asked,
            &mut SaidOnce::new(),
            a_moment(),
        )
        .unwrap_err();

    assert!(
        matches!(refused, DidNotLook::NoRecordToWriteIn(_)),
        "{refused:?}"
    );
    assert!(refused.nothing_left_this_machine());
    assert_eq!(asked.how_often_it_was_asked(), 0, "the place was asked");
}

/// **Something there that is not a record is refused**, rather than added to —
/// and then nothing is asked at all.
#[test]
fn a_file_that_is_not_a_record_is_refused_before_anything_is_asked() {
    let folder = a_folder("not-a-record");
    let at = AtAStart::in_folder(&folder);
    std::fs::write(at.record(), "this is not a record\n").unwrap();
    let place = the_place();
    let asked = a_place_with_something_newer(&place);

    let refused = at
        .look_once(
            &ABaseThatSays::running(&build("aa")),
            &place,
            &asked,
            &mut SaidOnce::new(),
            a_moment(),
        )
        .unwrap_err();

    assert!(
        matches!(refused, DidNotLook::NoRecordToWriteIn(_)),
        "{refused:?}"
    );
    assert_eq!(asked.how_often_it_was_asked(), 0);
    assert_eq!(at.answer().read().unwrap(), None);
    drop(std::fs::remove_dir_all(&folder));
}

/// **An answer that cannot be kept is said rather than swallowed.** The check
/// happened, it is in the record, and no surface will read it back — which is a
/// different thing to tell somebody from *nothing was found out*.
#[test]
fn an_answer_that_cannot_be_kept_is_its_own_refusal() {
    let folder = a_folder("unkeepable");
    let at = AtAStart::in_folder(&folder);
    // A directory where the answer goes: the record beside it still opens, the
    // check still happens, and the answer has nowhere to be written.
    std::fs::create_dir_all(at.answer().path()).unwrap();
    let place = the_place();

    let refused = at
        .look_once(
            &ABaseThatSays::running(&build("aa")),
            &place,
            &a_place_with_something_newer(&place),
            &mut SaidOnce::new(),
            a_moment(),
        )
        .unwrap_err();

    assert!(matches!(refused, DidNotLook::NotKept(_)), "{refused:?}");
    assert!(!refused.nothing_left_this_machine());
    assert_eq!(
        what_the_record_says(&at)
            .answering(&Asking::anything().only(Only::Egress))
            .count(),
        1,
        "the check left and was not written down"
    );
    drop(std::fs::remove_dir_all(&folder));
}

/// **The two files a check writes are the only two.** A start leaves the
/// machine otherwise exactly as it was.
#[test]
fn a_check_writes_two_files_and_changes_nothing_else() {
    let folder = a_folder("two-files");
    let at = AtAStart::in_folder(&folder);
    let place = the_place();

    at.look_once(
        &ABaseThatSays::running(&build("aa")),
        &place,
        &a_place_with_something_newer(&place),
        &mut SaidOnce::new(),
        a_moment(),
    )
    .unwrap();

    let mut left: Vec<String> = std::fs::read_dir(&folder)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    left.sort();
    assert_eq!(left, ["an-update-was-found", "checking-for-updates.jsonl"]);
    drop(std::fs::remove_dir_all(&folder));
}

/// A base's status naming this build as the one booted.
///
/// The shape `alo_keeping_up::Deployments` reads, written here rather than
/// taken from that crate's own fixtures: what this file is testing is the act
/// around the base, and a status it writes itself is one fewer thing that has
/// to be true for a refusal here to mean what it says.
fn a_status_naming(build: &Digest) -> String {
    format!(
        "{{\"status\":{{\"booted\":{{\"image\":{{\"imageDigest\":\"{}\"}}}}}}}}",
        build.as_str()
    )
}
