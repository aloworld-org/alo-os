//! The acceptance of task 11, and of
//! [ADR 0052](../../../docs/decisions/0052-what-a-persons-applications-asked-for-is-the-persons-record.md):
//! **a person's record of what their applications asked for is theirs, and a
//! login cannot read another login's.**
//!
//! Until this, the answers file was one machine-wide path in a folder the image
//! makes. On a machine with two people that file would have held both people's
//! applications' requests — every time somebody's calendar asked for their
//! microphone, with the times — each readable by the other. Nobody chose that;
//! it was the shape the first version happened to have, and tasks 8 to 10 left
//! *whose it is* open on purpose.
//!
//! So this walks two logins on one machine and asks the two questions that
//! matter:
//!
//! 1. **two people, two files** — what one person's backend writes is in that
//!    person's own state, and the other person's backend, reading its own place,
//!    finds nothing of it;
//! 2. **and the filesystem holds it, not a filter of ours** — a file owned by
//!    somebody else is refused before a byte of it is read, whatever it says
//!    inside.
//!
//! The second is the half that needed the ownership rule to change. It runs only
//! where this machine can give a file to another login — which needs root, and
//! is the machine the gate runs on — and says what it skipped where it cannot.

#![cfg(unix)]
#![expect(
    clippy::expect_used,
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use alo_capability::Applicant;
use alo_portals::not_recorded::NotRecorded;
use alo_portals::recording::Recording;
use alo_portals::{Answered, AnswersFile, Outcome, Portal, ThePlace, Unanswered};

/// A login's home directory, made for this test.
fn a_login(named: &str) -> PathBuf {
    use std::sync::atomic::{AtomicU32, Ordering};
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let home = std::env::temp_dir().join(format!(
        "alo-portals-{named}-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&home).expect("a home for this test");
    home
}

/// One answer, so that a file has something in it worth not sharing.
fn an_answer(application: &str, at: SystemTime) -> Answered {
    Answered::new(
        at,
        Some(Applicant::named(application)),
        Portal::Secret,
        Outcome::Unanswered(Unanswered::KeyringUnavailable),
    )
}

/// **Two people on one machine keep two files, and neither is the other's.**
#[test]
fn what_one_login_was_asked_is_not_in_another_logins_record() {
    let noon = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);

    let adas_home = a_login("ada");
    let bos_home = a_login("bo");
    let ada = ThePlace::for_this_login(None, Some(adas_home.as_os_str()))
        .expect("a login with a home has somewhere");
    let bo = ThePlace::for_this_login(None, Some(bos_home.as_os_str()))
        .expect("a login with a home has somewhere");

    assert_ne!(ada.file(), bo.file(), "two logins were given one file");

    // Ada's backend keeps an answer, in Ada's own state.
    let at = ada.made().expect("Ada's folder was made");
    let record = AnswersFile::opened(at).expect("Ada's record opened");
    record
        .keep(an_answer("org.gnome.Calendar", noon))
        .expect("kept");

    let hers = AnswersFile::read_back(ada.file()).expect("Ada reads her own");
    assert_eq!(hers.answers.len(), 1);
    assert!(
        hers.answers
            .first()
            .and_then(alo_portals::kept_answer::KeptAnswer::application)
            == Some("org.gnome.Calendar"),
        "Ada's own answer did not read back"
    );

    // Bo's backend, reading Bo's place, finds nothing — not an empty list
    // standing in for a file that is not there, but the refusal that says so.
    let bos = AnswersFile::read_back(bo.file());
    assert!(
        matches!(bos, Err(NotRecorded::NotThere { .. })),
        "Bo's machine answered something other than *nothing has been answered here yet*: {bos:?}"
    );

    // And nothing of Ada's is anywhere under Bo's.
    assert!(
        !bo.folder().exists(),
        "Ada's backend made a folder in Bo's state"
    );
    let ada_text = std::fs::read_to_string(ada.file()).expect("Ada's file is there");
    assert!(ada_text.contains("org.gnome.Calendar"));
}

/// **A file that belongs to somebody else is refused before it is read**, which
/// is what makes *a person's record is their own* a promise the filesystem keeps
/// rather than one our own code filters for.
#[test]
fn a_file_belonging_to_another_login_is_refused_rather_than_read() {
    let noon = SystemTime::UNIX_EPOCH + Duration::from_secs(1_760_000_000);
    let home = a_login("ada-and-a-stranger");
    let place = ThePlace::for_this_login(None, Some(home.as_os_str())).expect("somewhere");
    let at = place.made().expect("the folder was made");
    let record = AnswersFile::opened(at).expect("opened");
    record
        .keep(an_answer("org.example.Stranger", noon))
        .unwrap();
    drop(record);

    // It reads while it is ours.
    assert!(AnswersFile::read_back(place.file()).is_ok());

    // Give it to somebody else. Nothing but root can, and where this machine
    // will not, the rule is still held by `believed_file`'s own tests.
    let Some(somebody_else) = given_to_somebody_else(place.file()) else {
        eprintln!(
            "skipped the second half: this machine will not give a file to another login, so \
             only the first half — two logins, two files — was taken here"
        );
        return;
    };

    let why = AnswersFile::read_back(place.file())
        .expect_err("a file that is not this login's is not this login's record");
    assert!(
        matches!(why, NotRecorded::SomebodyElses { owner, .. } if owner == somebody_else),
        "a file owned by {somebody_else} was read as this login's: {why:?}"
    );

    // And it is not written to either: an answer is never added to a record
    // this login cannot vouch for.
    let opened = AnswersFile::opened(place.file());
    assert!(
        matches!(opened, Err(NotRecorded::SomebodyElses { .. })),
        "the backend added an answer to somebody else's file: {opened:?}"
    );
}

/// **A login with nowhere to keep a record refuses the request**, as task 8's
/// *an answer that is not kept is not sent* has it — rather than answering an
/// application and forgetting what it was told.
#[test]
fn a_login_with_nowhere_to_keep_a_record_refuses_rather_than_answering() {
    // A state directory that is not there: a typo, or a login this machine is
    // not set up for.
    let nowhere = a_login("nowhere").join("not-a-state-directory");
    let place = ThePlace::for_this_login(Some(nowhere.as_os_str()), None).expect("a path");
    let why = place
        .made()
        .expect_err("there is no such directory to keep a record in");
    assert!(matches!(why, NotRecorded::NotWritten { .. }));
    assert!(!place.folder().exists(), "a folder was made under nothing");

    // And a login with neither variable has nowhere at all, which is said
    // rather than guessed at.
    assert_eq!(ThePlace::for_this_login(None, None), None);
    assert_eq!(
        ThePlace::for_this_login(Some(OsStr::new("state")), Some(OsStr::new("home"))),
        None,
        "a relative path is not a place, and `/` is not somebody's state directory"
    );
}

/// Give this file to another login, and say which — or [`None`] where this
/// machine will not.
fn given_to_somebody_else(at: &Path) -> Option<u32> {
    /// The login every Unix has for *nobody in particular*, which is exactly
    /// what is wanted: a file owned by somebody who is not this login.
    const NOBODY: u32 = 65_534;

    let us = rustix::process::geteuid().as_raw();
    let somebody_else = if us == NOBODY { NOBODY - 1 } else { NOBODY };
    let owner = rustix::fs::Uid::from_raw(somebody_else);
    rustix::fs::chown(at, Some(owner), None).ok()?;
    Some(somebody_else)
}
