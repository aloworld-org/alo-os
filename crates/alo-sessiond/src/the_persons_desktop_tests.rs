#![expect(
    clippy::expect_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::path::{Path, PathBuf};

use super::{
    NotStarted, StartsADesktop, THE_DESKTOP, TheDesktopToStart, WHERE_THE_SOCKET_GOES,
    the_desktop_for,
};

/// The runtime path `CreateSession` answers with for account 1000.
fn the_sessions_runtime() -> PathBuf {
    PathBuf::from("/run/user/1000")
}

/// **The program is a constant and nothing a caller says can change it.**
///
/// This is law 2 written as a test rather than as a comment: a caller supplies
/// an account number and a name, and neither reaches the program that runs.
#[test]
fn nothing_a_caller_supplies_changes_which_program_starts() {
    let ordinary = the_desktop_for(1000, Path::new("/home/disan"), &the_sessions_runtime());
    assert_eq!(ordinary.program(), Path::new(THE_DESKTOP));

    // A name that would be an injection if a name were ever concatenated into
    // a command. It is not, and this is what says so.
    let awkward = the_desktop_for(
        1000,
        Path::new("/home/; /bin/sh -c id"),
        &the_sessions_runtime(),
    );
    assert_eq!(
        awkward.program(),
        Path::new(THE_DESKTOP),
        "the program must not vary with the account's name"
    );
    assert_eq!(
        ordinary.program(),
        awkward.program(),
        "two very different names produced two different programs, which means a name reaches \
         the thing that runs"
    );
}

/// The desktop runs as the person the session was opened for, and the account
/// number is carried rather than looked up again.
#[test]
fn it_runs_as_the_person_the_session_was_opened_for() {
    assert_eq!(
        the_desktop_for(1000, Path::new("/home/disan"), &the_sessions_runtime()).person(),
        1000
    );
    assert_eq!(
        the_desktop_for(1001, Path::new("/home/mac"), &the_sessions_runtime()).person(),
        1001
    );
}

/// Their folder is theirs, under the one place this machine keeps folders.
#[test]
fn the_folder_is_the_persons_own() {
    let what = the_desktop_for(1000, Path::new("/home/disan"), &the_sessions_runtime());
    assert_eq!(what.home(), Path::new("/home/disan"));
    assert!(
        what.home().is_absolute(),
        "a folder read from the machine's user database is an absolute path, and anything else \n         would make `HOME` depend on where this process happened to be started"
    );
}

/// **The socket goes in the session's runtime directory, not in a machine-wide
/// one.**
///
/// `alo-desktop.service` made `/run/alo-desktop`, which outlives the session
/// and is the same path for whoever signs in next. The session's own directory
/// is the person's and `logind` takes it away with their last session.
#[test]
fn the_socket_lives_inside_the_session_that_was_opened() {
    let what = the_desktop_for(1000, Path::new("/home/disan"), &the_sessions_runtime());
    assert_eq!(what.runtime(), Path::new("/run/user/1000/alo-desktop"));
    assert!(
        what.runtime().starts_with(the_sessions_runtime()),
        "the socket directory must be inside the session's own runtime directory"
    );
    assert!(
        !what.runtime().starts_with("/run/alo-desktop"),
        "this is the machine-wide directory the unit file made, which is what this moved away \
         from"
    );
    assert!(what.runtime().ends_with(WHERE_THE_SOCKET_GOES));
}

/// A second person gets a second runtime directory, with nothing shared.
#[test]
fn two_people_share_no_directory() {
    let one = the_desktop_for(1000, Path::new("/home/disan"), Path::new("/run/user/1000"));
    let other = the_desktop_for(1001, Path::new("/home/mac"), Path::new("/run/user/1001"));
    assert_ne!(one.home(), other.home());
    assert_ne!(one.runtime(), other.runtime());
    assert_ne!(one.person(), other.person());
}

/// Something that refuses everything, to exercise the reporting path.
struct AMachineThatRefuses;

impl StartsADesktop for AMachineThatRefuses {
    fn start(&mut self, what: &TheDesktopToStart) -> Result<(), NotStarted> {
        Err(NotStarted::TheMachineRefused {
            program: what.program().display().to_string(),
            person: what.person(),
            why: "No such file or directory (os error 2)".to_owned(),
        })
    }
}

/// **The refusal says which program, which account, and what the machine
/// said** — tested as carefully as the happy path, because a desktop that did
/// not start is the case a person is looking at when they read this.
#[test]
fn a_refusal_names_the_program_the_account_and_the_reason() {
    let what = the_desktop_for(1000, Path::new("/home/disan"), &the_sessions_runtime());
    let refused = AMachineThatRefuses
        .start(&what)
        .expect_err("this machine refuses everything");

    let said = refused.to_string();
    assert!(
        said.contains(THE_DESKTOP),
        "the sentence must name the program: {said}"
    );
    assert!(
        said.contains("1000"),
        "the sentence must name the account: {said}"
    );
    assert!(
        said.contains("No such file or directory"),
        "the machine's own words must survive into the sentence: {said}"
    );
}
