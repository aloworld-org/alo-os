//! **Signing in to GitHub and out, against a `gh` written down here** — a
//! script that prints what gh prints — so every road is a test on any Linux
//! machine: who is signed in, a sign-in that finishes, and each way one does
//! not — gh missing, gh failing, no code arriving, the person cancelling, a
//! sign-out that fails. Each sign-in, sign-out and failure is recorded; a
//! reading is not.

use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use alo_changing::{AtGitHub, WhyNotSignedIn};
use alo_shell::{AskingGitHub, GitHubAnswered, WhyGitHubDidNotWork};

use super::TheGitHubSignIn;

/// A `gh` at `folder`, answering each command as `script` says.
fn a_gh(folder: &Path, script: &str) -> PathBuf {
    let gh = folder.join("gh");
    std::fs::write(&gh, format!("#!/bin/sh\n{script}\n")).expect("the script is written");
    std::fs::set_permissions(&gh, std::fs::Permissions::from_mode(0o755)).expect("the script runs");
    gh
}

/// What a gh that works says: anna is signed in, a sign-in prints its code
/// and page and finishes as anna, and signing out works.
const A_GH_THAT_WORKS: &str = r#"case "$1 $2" in
"config get") echo anna ;;
"auth login")
  echo "! First copy your one-time code: 4F2A-9C1D" >&2
  echo "Open this URL to continue in your web browser: https://github.com/login/device" >&2
  sleep 0.5
  echo "✓ Authentication complete." >&2
  echo "✓ Logged in as anna" >&2 ;;
"auth logout") echo "✓ Logged out of github.com account anna" >&2 ;;
esac"#;

/// A worker on `gh`, and what it recorded.
fn running(gh: PathBuf, no_code_within: Duration) -> (TheGitHubSignIn, Arc<Mutex<Vec<AtGitHub>>>) {
    let kept = Arc::new(Mutex::new(Vec::new()));
    let into = Arc::clone(&kept);
    let worker = TheGitHubSignIn::waiting_for_a_code(
        gh,
        Box::new(move |what: &AtGitHub| {
            into.lock().expect("the record list").push(what.clone());
        }),
        no_code_within,
    );
    (worker, kept)
}

/// The next answer, waiting up to ten seconds for it.
fn the_next(worker: &mut TheGitHubSignIn) -> GitHubAnswered {
    let began = Instant::now();
    loop {
        if let Some(answer) = worker.answered() {
            return answer;
        }
        assert!(
            began.elapsed() < Duration::from_secs(10),
            "nothing was answered"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
}

/// What was recorded, so far.
fn recorded(kept: &Arc<Mutex<Vec<AtGitHub>>>) -> Vec<AtGitHub> {
    kept.lock().expect("the record list").clone()
}

/// **A sign-in that finishes**: the code and the page first, then signed in
/// as the login gh named — recorded once, and the reading before it not at all.
#[test]
fn a_sign_in_hands_over_the_code_and_finishes_recorded() {
    let folder = tempfile::tempdir().expect("a folder");
    let (mut worker, kept) = running(a_gh(folder.path(), A_GH_THAT_WORKS), Duration::from_secs(5));
    worker.asked(AskingGitHub::WhoIsSignedIn);
    assert_eq!(
        the_next(&mut worker),
        GitHubAnswered::SignedInAs(Some("anna".to_owned()))
    );
    worker.asked(AskingGitHub::SignIn);
    assert_eq!(
        the_next(&mut worker),
        GitHubAnswered::TheCodeIs {
            code: "4F2A-9C1D".to_owned(),
            page: "https://github.com/login/device".to_owned(),
        }
    );
    assert_eq!(
        the_next(&mut worker),
        GitHubAnswered::SignInWorked("anna".to_owned())
    );
    assert_eq!(
        recorded(&kept),
        [AtGitHub::signed_in("anna").expect("a login")]
    );
}

/// **Signing out works, and is recorded with who it was.**
#[test]
fn signing_out_is_recorded() {
    let folder = tempfile::tempdir().expect("a folder");
    let (mut worker, kept) = running(a_gh(folder.path(), A_GH_THAT_WORKS), Duration::from_secs(5));
    worker.asked(AskingGitHub::SignOut);
    assert_eq!(the_next(&mut worker), GitHubAnswered::SignedOut);
    assert_eq!(
        recorded(&kept),
        [AtGitHub::signed_out(Some("anna")).expect("a login")]
    );
}

/// **gh is not on this computer**: a reading says so and records nothing; a
/// sign-in and a sign-out say so and are recorded as not done.
#[test]
fn gh_missing_is_said_and_recorded() {
    let folder = tempfile::tempdir().expect("a folder");
    let (mut worker, kept) = running(folder.path().join("no-gh-here"), Duration::from_secs(5));
    for asked in [
        AskingGitHub::WhoIsSignedIn,
        AskingGitHub::SignIn,
        AskingGitHub::SignOut,
    ] {
        worker.asked(asked);
        assert_eq!(
            the_next(&mut worker),
            GitHubAnswered::DidNotWork {
                asked,
                why: WhyGitHubDidNotWork::GhIsNotHere,
            }
        );
    }
    assert_eq!(
        recorded(&kept),
        [
            AtGitHub::did_not_sign_in(WhyNotSignedIn::GhIsNotHere),
            AtGitHub::did_not_sign_out(None),
        ]
    );
}

/// **gh fails**: the sign-in says gh's own last line, and is recorded with it;
/// a reading that fails is nobody signed in.
#[test]
fn gh_failing_is_said_with_its_own_line() {
    let folder = tempfile::tempdir().expect("a folder");
    let script = r#"case "$1 $2" in
"config get") exit 1 ;;
"auth login") echo "error connecting to api.github.com" >&2; exit 1 ;;
esac"#;
    let (mut worker, kept) = running(a_gh(folder.path(), script), Duration::from_secs(5));
    worker.asked(AskingGitHub::WhoIsSignedIn);
    assert_eq!(the_next(&mut worker), GitHubAnswered::SignedInAs(None));
    worker.asked(AskingGitHub::SignIn);
    assert_eq!(
        the_next(&mut worker),
        GitHubAnswered::DidNotWork {
            asked: AskingGitHub::SignIn,
            why: WhyGitHubDidNotWork::GhFailed(Some(
                "error connecting to api.github.com".to_owned()
            )),
        }
    );
    assert_eq!(
        recorded(&kept),
        [AtGitHub::did_not_sign_in(WhyNotSignedIn::gh_failed(Some(
            "error connecting to api.github.com"
        )))]
    );
}

/// **No code arrives**: a gh that prints nothing is ended after the wait, and
/// that is said and recorded.
#[test]
fn no_code_arriving_ends_the_sign_in() {
    let folder = tempfile::tempdir().expect("a folder");
    let script = r#"case "$1 $2" in
"auth login") sleep 30 ;;
esac"#;
    let (mut worker, kept) = running(a_gh(folder.path(), script), Duration::from_millis(300));
    let began = Instant::now();
    worker.asked(AskingGitHub::SignIn);
    assert_eq!(
        the_next(&mut worker),
        GitHubAnswered::DidNotWork {
            asked: AskingGitHub::SignIn,
            why: WhyGitHubDidNotWork::NoCodeArrived,
        }
    );
    assert!(
        began.elapsed() < Duration::from_secs(10),
        "gh was waited out"
    );
    assert_eq!(
        recorded(&kept),
        [AtGitHub::did_not_sign_in(WhyNotSignedIn::NoCodeArrived)]
    );
}

/// **The person cancels** while gh waits for the code to be typed: gh is
/// ended, and that is said and recorded. A cancel with nothing waiting is
/// nothing.
#[test]
fn a_cancel_ends_a_waiting_sign_in() {
    let folder = tempfile::tempdir().expect("a folder");
    let script = r#"case "$1 $2" in
"auth login") echo "! First copy your one-time code: 4F2A-9C1D" >&2; sleep 30 ;;
esac"#;
    let (mut worker, kept) = running(a_gh(folder.path(), script), Duration::from_secs(5));
    worker.asked(AskingGitHub::StopSigningIn);
    worker.asked(AskingGitHub::SignIn);
    assert_eq!(
        the_next(&mut worker),
        GitHubAnswered::TheCodeIs {
            code: "4F2A-9C1D".to_owned(),
            page: "https://github.com/login/device".to_owned(),
        }
    );
    worker.asked(AskingGitHub::StopSigningIn);
    assert_eq!(
        the_next(&mut worker),
        GitHubAnswered::DidNotWork {
            asked: AskingGitHub::SignIn,
            why: WhyGitHubDidNotWork::Cancelled,
        }
    );
    assert_eq!(
        recorded(&kept),
        [AtGitHub::did_not_sign_in(WhyNotSignedIn::TheyCancelled)]
    );
}

/// **A sign-out that fails** says gh's own last line, and is recorded with it.
#[test]
fn a_sign_out_that_fails_is_said_and_recorded() {
    let folder = tempfile::tempdir().expect("a folder");
    let script = r#"case "$1 $2" in
"config get") echo anna ;;
"auth logout") echo "could not remove the token from the keyring" >&2; exit 1 ;;
esac"#;
    let (mut worker, kept) = running(a_gh(folder.path(), script), Duration::from_secs(5));
    worker.asked(AskingGitHub::SignOut);
    assert_eq!(
        the_next(&mut worker),
        GitHubAnswered::DidNotWork {
            asked: AskingGitHub::SignOut,
            why: WhyGitHubDidNotWork::GhFailed(Some(
                "could not remove the token from the keyring".to_owned()
            )),
        }
    );
    assert_eq!(
        recorded(&kept),
        [AtGitHub::did_not_sign_out(Some(
            "could not remove the token from the keyring"
        ))]
    );
}
