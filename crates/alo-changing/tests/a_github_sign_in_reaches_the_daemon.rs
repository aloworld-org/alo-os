//! A GitHub sign-in reaches the daemon as the one line the contract
//! documents, and every way it is not kept is told apart from being kept.
//!
//! A thread stands where `alo-agentd` stands, on a real Unix socket: it reads
//! the one line [`TheDaemonsDoor::keep_a_github_sign_in`] sends, holds it to
//! being `github-sign-in` with exactly what was asked, and answers the way the
//! daemon does — or does not answer at all.

#![cfg(unix)]
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;

use alo_changing::{AtGitHub, GitHubSignInNotKept, TheDaemonsDoor, WhyNotSignedIn};
use alo_protocol::{FromAPerson, ToAPerson};
use alo_strings::{Filling, Strings};

/// A folder of this test's own, empty, on the real disk.
fn a_folder_of_our_own(what: &str) -> PathBuf {
    let folder = std::env::temp_dir()
        .join("alo-changing-github-sign-in-tests")
        .join(format!("{}-{what}", std::process::id()));
    drop(std::fs::remove_dir_all(&folder));
    std::fs::create_dir_all(&folder).unwrap();
    folder
}

/// A thread standing where the daemon stands: it reads one line, holds it to
/// being `expected`, and answers with `answer` — or closes without a word when
/// there is none.
fn a_daemon_at(socket: &Path, expected: FromAPerson, answer: Option<ToAPerson>) -> JoinHandle<()> {
    let listener = UnixListener::bind(socket).unwrap();
    std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut line = String::new();
        BufReader::new(&stream).read_line(&mut line).unwrap();
        assert_eq!(
            FromAPerson::read(line.trim_end()).unwrap(),
            expected,
            "the desktop asked something other than the contract documents"
        );
        if let Some(answer) = answer {
            let written = answer.written().unwrap();
            (&stream).write_all(written.as_bytes()).unwrap();
            (&stream).write_all(b"\n").unwrap();
        }
    })
}

/// Anna signed in.
fn anna_signed_in() -> AtGitHub {
    AtGitHub::signed_in("anna").unwrap()
}

/// **What the daemon says it kept is kept**, for every kind the desktop can
/// report, and the line that went out was exactly that kind.
#[test]
fn a_sign_in_the_daemon_kept_is_kept() {
    for (named, what) in [
        ("in", anna_signed_in()),
        ("out", AtGitHub::signed_out(Some("anna")).unwrap()),
        (
            "no-gh",
            AtGitHub::did_not_sign_in(WhyNotSignedIn::GhIsNotHere),
        ),
        (
            "gh-failed",
            AtGitHub::did_not_sign_in(WhyNotSignedIn::gh_failed(Some("HTTP 502"))),
        ),
        (
            "no-code",
            AtGitHub::did_not_sign_in(WhyNotSignedIn::NoCodeArrived),
        ),
        (
            "cancelled",
            AtGitHub::did_not_sign_in(WhyNotSignedIn::TheyCancelled),
        ),
        (
            "out-failed",
            AtGitHub::did_not_sign_out(Some("not logged in")),
        ),
    ] {
        let folder = a_folder_of_our_own(named);
        let socket = folder.join("agentd.sock");
        let daemon = a_daemon_at(
            &socket,
            FromAPerson::GitHubSignIn(what.clone()),
            Some(ToAPerson::GitHubSignInKept),
        );
        let kept = TheDaemonsDoor::at(&socket).keep_a_github_sign_in(&what);
        daemon.join().unwrap();
        assert_eq!(kept, Ok(()), "{named}");
    }
}

/// **A refusal is the daemon's own words, and never read as kept.**
#[test]
fn a_sign_in_the_daemon_refused_is_told_in_its_words() {
    let folder = a_folder_of_our_own("refused");
    let socket = folder.join("agentd.sock");
    let words = Strings::of(alo_saying::everything_this_machine_can_say().unwrap()).say(
        &alo_protocol::words::NOT_READABLE.key(),
        &Filling::nothing(),
    );
    let daemon = a_daemon_at(
        &socket,
        FromAPerson::GitHubSignIn(anna_signed_in()),
        Some(ToAPerson::refused(&words)),
    );
    let kept = TheDaemonsDoor::at(&socket).keep_a_github_sign_in(&anna_signed_in());
    daemon.join().unwrap();
    assert_eq!(
        kept,
        Err(GitHubSignInNotKept::Refused {
            told: words.text().to_owned()
        })
    );
}

/// **With nobody at the door, nothing was kept, and that is what is said.**
#[test]
fn with_nobody_at_the_door_nothing_is_kept() {
    let folder = a_folder_of_our_own("nobody");
    let kept = TheDaemonsDoor::at(&folder.join("nobody-listens.sock"))
        .keep_a_github_sign_in(&anna_signed_in());
    assert_eq!(kept, Err(GitHubSignInNotKept::NobodyThere));
}

/// **A daemon that hears it and never answers has not said it kept it.**
#[test]
fn a_daemon_that_never_answers_has_not_kept_it() {
    let folder = a_folder_of_our_own("silent");
    let socket = folder.join("agentd.sock");
    let daemon = a_daemon_at(&socket, FromAPerson::GitHubSignIn(anna_signed_in()), None);
    let kept = TheDaemonsDoor::at(&socket).keep_a_github_sign_in(&anna_signed_in());
    daemon.join().unwrap();
    assert_eq!(kept, Err(GitHubSignInNotKept::NotAnswered));
}

/// **An answer this request cannot earn is not a sign of it being kept** — a
/// daemon answering `declined` has said nothing about a sign-in.
#[test]
fn an_answer_to_another_question_is_not_read_as_kept() {
    let folder = a_folder_of_our_own("wrong-answer");
    let socket = folder.join("agentd.sock");
    let daemon = a_daemon_at(
        &socket,
        FromAPerson::GitHubSignIn(anna_signed_in()),
        Some(ToAPerson::Declined),
    );
    let kept = TheDaemonsDoor::at(&socket).keep_a_github_sign_in(&anna_signed_in());
    daemon.join().unwrap();
    assert_eq!(kept, Err(GitHubSignInNotKept::NotAnswered));
}
