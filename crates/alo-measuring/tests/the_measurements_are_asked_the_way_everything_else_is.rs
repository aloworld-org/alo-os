//! *What is running* and *what is filling* are asked the way everything else
//! is asked — and are reachable without asking anybody.
//!
//! The plan's acceptance for task 5, on this crate's half: *two read verbs,
//! declared in the shape `alo-files` declares its own, so no approval is
//! asked and the record's entry has no approval to name; each evaluated
//! against a grant naming the directory it reads, refused outside it, and
//! recorded as having run; and the same answers reachable by a caller with
//! no agent and no grant at all.*
//!
//! The real verbs, the real grants, the real resolver and the real record,
//! walked the way a daemon would walk them. *What is running* is read from a
//! kernel this test writes out into a directory of its own — the files
//! `/proc` has, with numbers easy to check by eye — so the whole road runs
//! on every host; *what is filling* opens the real disk and its size is
//! checked to the byte where the host can count one.
//!
//! | The acceptance | The test |
//! |---|---|
//! | what is running: a read under a grant over the kernel's directory, answered inside the turn, recorded with no approval to name | [`what_is_running_under_a_grant_over_proc_answers_inside_the_turn_and_is_recorded`] |
//! | what is filling: a read under a grant over the folder, answered inside the turn, recorded with no approval to name | [`what_is_filling_under_a_grant_over_the_folder_answers_inside_the_turn_and_is_recorded`] |
//! | refused outside the grant, before anything is read, and the refusal is recorded | [`a_directory_outside_the_grant_is_refused_before_anything_is_read_and_the_refusal_is_recorded`] |
//! | a grant revoked after the authorisation, or expired, still stops it | [`a_grant_taken_away_or_run_out_stops_the_measurement`] |
//! | a call that is not one never reaches the grants or the kernel | [`a_call_that_is_not_one_never_reaches_the_grants_or_the_kernel`] |
//! | the same numbers, with no agent and no grant | [`a_person_with_no_agent_and_no_grant_gets_the_same_numbers`] |
//! | only the two measurements are this crate's to answer | [`only_the_two_measurements_are_this_crates_to_answer`] |

#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime};

use alo_capability::{
    Authorised, Call, CallError, Given, Grant, Grantee, Grants, NotAuthorised, Reach, Refused,
};
use alo_files::{OnThisMachine, Resolving, Touching};
use alo_measuring::{
    Disk, Measured, Measurement, NotMeasured, Reading, Running, measuring_verbs, measuring_words,
};
use alo_record::{Asking, Entry, Only, Record};
use alo_strings::Strings;

/// A fixed moment, so that expiry is arithmetic rather than a wait.
fn noon() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000)
}

/// How long the grants here last.
fn hour() -> Duration {
    Duration::from_secs(60 * 60)
}

/// How far apart the two readings are said to be.
fn two_seconds() -> Duration {
    Duration::from_secs(2)
}

/// The agent everything here is granted to.
fn agent() -> Grantee {
    Grantee::named("@measuring")
}

/// The words this machine reads: this crate's beside the file half's and the
/// capability model's, which is the arrangement a shell has.
fn in_english() -> Strings {
    let mut vocabulary = measuring_words().unwrap();
    alo_files::words::declare_into(&mut vocabulary).unwrap();
    alo_capability::declare_into(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A folder of this test's own, **resolved**, so that what is granted and
/// what is asked about are spelled the way this machine spells them — a
/// grant is over a real place, and on Windows a resolved path carries a
/// prefix the path it was typed from does not.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let folder = std::env::temp_dir().join(format!(
        "alo-measuring-verb-{}-{what}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).unwrap();
    OnThisMachine.real(&folder).unwrap().into_path_buf()
}

/// A kernel written out into a directory: the machine's `stat` and
/// `meminfo`, and one process, `cat`, pid 42, with every file the crate
/// reads and these totals.
fn a_written_kernel(proc: &Path, machine_ticks: u64, ticks: u64, written: u64) {
    fs::write(
        proc.join("stat"),
        format!("cpu  {machine_ticks} 0 0 0 0 0 0 0 0 0\ncpu0 1 0 0 0 0 0 0 0 0 0\n"),
    )
    .unwrap();
    fs::write(
        proc.join("meminfo"),
        "MemTotal:       16000000 kB\nMemAvailable:   12000000 kB\n",
    )
    .unwrap();
    let process = proc.join("42");
    fs::create_dir_all(process.join("net")).unwrap();
    fs::create_dir_all(process.join("ns")).unwrap();
    fs::write(
        process.join("stat"),
        format!("42 (cat) S 1 1 1 0 -1 4194304 0 0 0 0 {ticks} 0 0 0 20 0 1 0 500 0 0 0"),
    )
    .unwrap();
    fs::write(process.join("status"), "Name:\tcat\nVmRSS:\t2000 kB\n").unwrap();
    fs::write(
        process.join("io"),
        format!("rchar: 3000\nwchar: {written}\n"),
    )
    .unwrap();
    fs::write(
        process.join("net").join("dev"),
        "Inter-| Receive | Transmit\n face |bytes packets errs drop fifo frame compressed \
         multicast|bytes packets errs drop fifo colls carrier compressed\n    lo: 5 0 0 0 0 0 0 0 \
         5 0 0 0 0 0 0 0\n  eth0: 5000 0 0 0 0 0 0 0 6000 0 0 0 0 0 0 0\n",
    )
    .unwrap();
    // Not a link: a namespace this host cannot say, which the crate reports
    // as none rather than as a process that is gone.
    fs::write(process.join("ns").join("net"), "").unwrap();
}

/// The written kernel, later: the machine has ticked and `cat` has written
/// more.
fn the_kernel_later(proc: &Path) {
    a_written_kernel(proc, 1400, 150, 4000 + 8000);
}

/// A folder of known bytes: ten in one file, twenty-five in a file one
/// level down.
fn a_folder_of_known_bytes(root: &Path) {
    fs::write(root.join("a.txt"), b"0123456789").unwrap();
    fs::create_dir_all(root.join("sub")).unwrap();
    fs::write(root.join("sub").join("b.txt"), b"0123456789012345678901234").unwrap();
}

/// A path, as a call arrives with it.
fn as_given(path: &Path) -> Given {
    Given::text(path.to_string_lossy().into_owned())
}

/// Grants to the agent over these folders, made at noon and lasting an hour.
fn granting(folders: &[&Path]) -> Grants {
    let mut grants = Grants::default();
    for folder in folders {
        grants.grant(
            Grant::checked(
                agent().as_str(),
                Reach::Folder(folder.to_path_buf()),
                noon(),
                hour(),
            )
            .unwrap(),
        );
    }
    grants
}

/// A call of `what_is_running` over this kernel directory.
fn running_from(proc: &Path) -> Result<Call, CallError> {
    measuring_verbs()
        .unwrap()
        .call("what_is_running", &[("proc", as_given(proc))])
}

/// A call of `what_is_filling` over this folder.
fn filling_of(folder: &Path) -> Result<Call, CallError> {
    measuring_verbs()
        .unwrap()
        .call("what_is_filling", &[("folder", as_given(folder))])
}

/// What a refusal says on this machine.
fn refusal(refused: &Refused) -> String {
    refused.said(&in_english()).into_text()
}

/// The whole road from a call to a permitted, resolved call, under these
/// grants, at noon.
fn permitted(call: &Call, grants: &Grants) -> Touching {
    let authorised = Authorised::read(call, &agent(), grants, noon()).unwrap();
    assert!(
        authorised.from_approval().is_none(),
        "a read needs no approval"
    );
    Touching::of(authorised, grants, &OnThisMachine, &in_english()).unwrap()
}

/// **What is running, all the way.** A read is authorised with no proposal
/// and no approval, the kernel's directory is made real and asked about
/// again, two readings are taken with the test's kernel changing between
/// them, and the record keeps all four of ADR 0001 §7's answers — with
/// *from which approval* answered by none.
#[test]
fn what_is_running_under_a_grant_over_proc_answers_inside_the_turn_and_is_recorded() {
    let strings = in_english();
    let proc = a_folder_of_our_own("proc");
    a_written_kernel(&proc, 1000, 100, 4000);
    let grants = granting(&[&proc]);

    let call = running_from(&proc).unwrap();
    assert!(!call.waits_for_approval(), "a measurement is a read");
    let touching = permitted(&call, &grants);
    assert_eq!(touching.real("proc").unwrap().as_path(), proc);

    let measured = Measured::of(touching, two_seconds(), |waited| {
        assert_eq!(waited, two_seconds());
        the_kernel_later(&proc);
    });
    let Some(Measurement::Running(running)) = measured.measurement() else {
        panic!("{:?}", measured.not_measured());
    };
    assert_eq!(running.interval(), two_seconds());
    let cat = running.process(42).unwrap();
    assert_eq!(cat.name, "cat");
    assert_eq!(cat.memory.value(), Some(2000 * 1024));
    assert_eq!(cat.written.value(), Some(8000 / 2));
    assert_eq!(cat.processor.value(), Some(50 * 1000 / 400));
    assert!(running.gone().is_empty());

    let (authorised, outcome) = measured.into_parts();
    assert!(outcome.is_ok());
    let mut record = Record::default();
    record.keep(Entry::ran(&authorised, &strings));
    let entry = record.everything().next().unwrap();
    assert!(entry.happened().ran());
    assert_eq!(entry.happened().from_approval(), None);
    assert_eq!(entry.happened().against().len(), 1);
    let what = entry.happened().what().unwrap();
    assert!(what.verb().is("what_is_running"));
    assert!(
        what.sentence()
            .as_str()
            .starts_with("list what is running and what it is using, read from "),
        "{}",
        what.sentence().as_str()
    );

    let _ = fs::remove_dir_all(&proc);
}

/// **What is filling, all the way.** The same road over a folder of known
/// bytes: on a host that can count, the tree is true to the byte; on any
/// other, the call still ran under its grant and is recorded as having run,
/// and what came back beside it says the host could not count.
#[test]
fn what_is_filling_under_a_grant_over_the_folder_answers_inside_the_turn_and_is_recorded() {
    let strings = in_english();
    let folder = a_folder_of_our_own("filling");
    a_folder_of_known_bytes(&folder);
    let grants = granting(&[&folder]);

    let call = filling_of(&folder).unwrap();
    assert!(!call.waits_for_approval(), "a count is a read");
    let touching = permitted(&call, &grants);
    let measured = Measured::of(touching, two_seconds(), |_| {
        panic!("a count is one look at the disk and waits for nothing")
    });

    #[cfg(target_os = "linux")]
    {
        let Some(Measurement::Filling(holding)) = measured.measurement() else {
            panic!("{:?}", measured.not_measured());
        };
        assert_eq!(holding.folder, folder);
        assert_eq!(holding.tree.size, 10 + 25);
        assert!(holding.finished);
        let sub = holding
            .tree
            .children
            .iter()
            .find(|child| child.name == "sub")
            .unwrap();
        assert_eq!(sub.size, 25);
    }
    #[cfg(not(target_os = "linux"))]
    {
        assert_eq!(measured.not_measured(), Some(&NotMeasured::NotOnThisHost));
        let said = measured.not_measured().unwrap().said(&strings);
        assert!(!said.is_a_bug(), "{said}");
    }

    let (authorised, _) = measured.into_parts();
    let mut record = Record::default();
    record.keep(Entry::ran(&authorised, &strings));
    let entry = record.everything().next().unwrap();
    assert!(entry.happened().ran());
    assert_eq!(entry.happened().from_approval(), None);
    let what = entry.happened().what().unwrap();
    assert!(what.verb().is("what_is_filling"));
    assert!(
        what.sentence()
            .as_str()
            .starts_with("count what is filling "),
        "{}",
        what.sentence().as_str()
    );

    let _ = fs::remove_dir_all(&folder);
}

/// **A directory nobody granted is refused before anything is read**, by the
/// grants and in their words — the refusal says nothing about whether the
/// directory exists, because nothing looked — and it is written down beside
/// every other refusal.
#[test]
fn a_directory_outside_the_grant_is_refused_before_anything_is_read_and_the_refusal_is_recorded() {
    let strings = in_english();
    let granted = a_folder_of_our_own("granted-not-this");
    let grants = granting(&[&granted]);
    let mut record = Record::default();

    // A kernel directory that is not there at all: refused for the grant,
    // and the refusal does not say it is missing.
    let nowhere = granted.join("no-such-proc");
    let call = running_from(&nowhere).unwrap();
    let elsewhere = a_folder_of_our_own("not-granted");
    let grants_elsewhere = granting(&[&elsewhere]);
    let refused = Authorised::read(&call, &agent(), &grants_elsewhere, noon()).unwrap_err();
    assert!(matches!(refused.why(), NotAuthorised::NotGranted(_)));
    assert!(
        refusal(&refused).contains("has not been granted"),
        "{}",
        refusal(&refused)
    );
    assert!(
        !refusal(&refused).contains("nothing at"),
        "{}",
        refusal(&refused)
    );
    record.keep(Entry::refused(&refused, &agent(), &strings, noon()));

    // And a folder to count, outside the grant.
    let call = filling_of(&elsewhere).unwrap();
    let refused = Authorised::read(&call, &agent(), &grants, noon()).unwrap_err();
    assert!(
        refusal(&refused).contains(&elsewhere.to_string_lossy().into_owned()),
        "{}",
        refusal(&refused)
    );
    assert_eq!(refused.call(), &call, "the record needs what was refused");
    record.keep(Entry::refused(&refused, &agent(), &strings, noon()));

    let refusals_only = Asking::anything().only(Only::Refusals);
    let stopped: Vec<_> = record.answering(&refusals_only).collect();
    assert_eq!(stopped.len(), 2);
    for entry in stopped {
        assert!(entry.happened().was_stopped());
        assert!(!entry.happened().ran());
        assert!(
            entry
                .happened()
                .why_stopped()
                .unwrap()
                .as_str()
                .contains("has not been granted")
        );
    }

    let _ = fs::remove_dir_all(&granted);
    let _ = fs::remove_dir_all(&elsewhere);
}

/// The grants are asked again at the door to the kernel, so a grant taken
/// away between the authorisation and the reading still stops it — and a
/// grant that has run out authorises nothing in the first place.
#[test]
fn a_grant_taken_away_or_run_out_stops_the_measurement() {
    let strings = in_english();
    let proc = a_folder_of_our_own("revoked");
    a_written_kernel(&proc, 1000, 100, 4000);
    let mut grants = granting(&[&proc]);
    let call = running_from(&proc).unwrap();

    let authorised = Authorised::read(&call, &agent(), &grants, noon()).unwrap();
    assert_eq!(grants.revoke_everything_for(&agent()), 1);
    let refused = Touching::of(authorised, &grants, &OnThisMachine, &strings).unwrap_err();
    assert!(
        refusal(&refused).contains("has not been granted"),
        "{}",
        refusal(&refused)
    );

    let grants = granting(&[&proc]);
    let expired = Authorised::read(&call, &agent(), &grants, noon() + hour()).unwrap_err();
    assert!(
        refusal(&expired).contains("has expired"),
        "{}",
        refusal(&expired)
    );

    let _ = fs::remove_dir_all(&proc);
}

/// A call that does not survive the door never becomes a call, so nothing
/// asks the grants and nothing reads the kernel: a relative directory, one
/// that steps upwards, a missing argument, an interval nobody offered.
#[test]
fn a_call_that_is_not_one_never_reaches_the_grants_or_the_kernel() {
    let verbs = measuring_verbs().unwrap();
    for (verb, argument, given) in [
        ("what_is_running", "proc", "proc"),
        ("what_is_running", "proc", "/proc/../etc"),
        ("what_is_filling", "folder", "Pictures"),
        ("what_is_filling", "folder", "/home/anna/../root"),
    ] {
        let err = verbs
            .call(verb, &[(argument, Given::text(given))])
            .unwrap_err();
        assert!(matches!(err, CallError::Argument(_)), "{given}: {err:?}");
    }
    assert!(matches!(
        verbs.call("what_is_filling", &[]).unwrap_err(),
        CallError::Missing { .. }
    ));
    assert!(matches!(
        verbs
            .call(
                "what_is_running",
                &[("proc", Given::text("/proc")), ("over", Given::number(60))]
            )
            .unwrap_err(),
        CallError::NoSuchArgument { .. }
    ));
}

/// **A person asking why it is slow is not an agent and is not asking
/// anybody.** The kernel's directory is all it takes — no grantee, no
/// grants, no authorisation — and the numbers are the ones the verb gave,
/// because they are the same functions. The shape says the same thing:
/// nothing a person calls takes an argument through which they and an agent
/// could be told apart.
#[test]
fn a_person_with_no_agent_and_no_grant_gets_the_same_numbers() {
    let proc = a_folder_of_our_own("by-hand");
    a_written_kernel(&proc, 1000, 100, 4000);

    // Nobody: no grants were made, and nothing asks for any.
    let earlier = Reading::of_kernel(&Disk, &proc).unwrap();
    the_kernel_later(&proc);
    let later = Reading::of_kernel(&Disk, &proc).unwrap();
    let by_hand = later.since(&earlier, two_seconds()).unwrap();

    // The agent, under a grant, through the verb, over the same two moments.
    a_written_kernel(&proc, 1000, 100, 4000);
    let grants = granting(&[&proc]);
    let touching = permitted(&running_from(&proc).unwrap(), &grants);
    let measured = Measured::of(touching, two_seconds(), |_| the_kernel_later(&proc));
    let Some(Measurement::Running(by_the_verb)) = measured.measurement() else {
        panic!("{:?}", measured.not_measured());
    };
    assert_eq!(by_the_verb, &by_hand);

    let now: fn() -> Result<Reading, NotMeasured> = Reading::now;
    let since: fn(&Reading, &Reading, Duration) -> Result<Running, NotMeasured> = Reading::since;
    let of: fn(&Path) -> Result<alo_measuring::Holding, NotMeasured> = alo_measuring::Holding::of;
    let _ = (now, since, of);

    #[cfg(target_os = "linux")]
    {
        let folder = a_folder_of_our_own("by-hand-filling");
        a_folder_of_known_bytes(&folder);
        let by_hand = alo_measuring::Holding::of(&folder).unwrap();
        let grants = granting(&[&folder]);
        let touching = permitted(&filling_of(&folder).unwrap(), &grants);
        let measured = Measured::of(touching, two_seconds(), |_| {});
        let Some(Measurement::Filling(by_the_verb)) = measured.measurement() else {
            panic!("{:?}", measured.not_measured());
        };
        assert_eq!(by_the_verb, &by_hand);
        let _ = fs::remove_dir_all(&folder);
    }

    let _ = fs::remove_dir_all(&proc);
}

/// A permitted call of some other verb handed to this door is not answered
/// by it — this crate measures, and anything else is somewhere else's to do.
#[test]
fn only_the_two_measurements_are_this_crates_to_answer() {
    let strings = in_english();
    let folder = a_folder_of_our_own("not-ours");
    let grants = granting(&[&folder]);
    let listing = alo_files::file_verbs()
        .unwrap()
        .call("list_folder", &[("folder", as_given(&folder))])
        .unwrap();
    let touching = permitted(&listing, &grants);
    let measured = Measured::of(touching, two_seconds(), |_| {
        panic!("nothing was measured, so nothing waited")
    });
    match measured.not_measured().unwrap() {
        NotMeasured::NotThisCrates { verb } => assert_eq!(verb, "list_folder"),
        other => panic!("{other:?}"),
    }
    assert!(
        measured
            .not_measured()
            .unwrap()
            .said(&strings)
            .text()
            .contains("list_folder")
    );
    let _ = fs::remove_dir_all(&folder);
}
