//! A GitHub sign-in, on the wire and at both doors.
//!
//! The refusals are the tests that matter: an agent's door refuses it, and a
//! login that is not one, or a field nobody declared, is not a request.

#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None or Err is the failure being reported"
)]

use alo_record::{AtGitHub, WhyNotSignedIn};

use crate::agent::FromAnAgent;
use crate::person::FromAPerson;
use crate::refusing::NotUnderstood;
use crate::to_a_person::ToAPerson;
use crate::to_an_agent::ToAnAgent;

fn every_kind() -> Vec<AtGitHub> {
    vec![
        AtGitHub::signed_in("anna").unwrap(),
        AtGitHub::signed_out(Some("anna")).unwrap(),
        AtGitHub::signed_out(None).unwrap(),
        AtGitHub::did_not_sign_in(WhyNotSignedIn::GhIsNotHere),
        AtGitHub::did_not_sign_in(WhyNotSignedIn::gh_failed(Some(
            "error connecting to api.github.com",
        ))),
        AtGitHub::did_not_sign_in(WhyNotSignedIn::gh_failed(None)),
        AtGitHub::did_not_sign_in(WhyNotSignedIn::NoCodeArrived),
        AtGitHub::did_not_sign_in(WhyNotSignedIn::TheyCancelled),
        AtGitHub::did_not_sign_out(Some("not logged in to any hosts")),
        AtGitHub::did_not_sign_out(None),
    ]
}

/// **Every kind goes out and comes back as itself**, on the person's door.
#[test]
fn every_kind_goes_out_and_comes_back_as_itself() {
    for what in every_kind() {
        let asked = FromAPerson::GitHubSignIn(what);
        let line = asked.written().unwrap();
        assert_eq!(FromAPerson::read(&line).unwrap(), asked, "{line}");
    }
}

/// The shape on the wire, written out so the contract and the code cannot
/// drift apart without one of them failing.
#[test]
fn the_wire_spells_it_as_the_contract_does() {
    for (line, what) in [
        (
            r#"{"format":1,"asks":{"github-sign-in":{"signed-in":{"login":"anna"}}}}"#,
            AtGitHub::signed_in("anna").unwrap(),
        ),
        (
            r#"{"format":1,"asks":{"github-sign-in":{"signed-out":{}}}}"#,
            AtGitHub::signed_out(None).unwrap(),
        ),
        (
            r#"{"format":1,"asks":{"github-sign-in":{"did-not-sign-in":{"why":"gh-is-not-here"}}}}"#,
            AtGitHub::did_not_sign_in(WhyNotSignedIn::GhIsNotHere),
        ),
        (
            r#"{"format":1,"asks":{"github-sign-in":{"did-not-sign-in":{"why":{"gh-failed":{"said":"HTTP 502"}}}}}}"#,
            AtGitHub::did_not_sign_in(WhyNotSignedIn::gh_failed(Some("HTTP 502"))),
        ),
        (
            r#"{"format":1,"asks":{"github-sign-in":{"did-not-sign-in":{"why":"they-cancelled"}}}}"#,
            AtGitHub::did_not_sign_in(WhyNotSignedIn::TheyCancelled),
        ),
        (
            r#"{"format":1,"asks":{"github-sign-in":{"did-not-sign-out":{"said":"not logged in"}}}}"#,
            AtGitHub::did_not_sign_out(Some("not logged in")),
        ),
    ] {
        assert_eq!(
            FromAPerson::read(line).unwrap(),
            FromAPerson::GitHubSignIn(what),
            "{line}"
        );
    }
}

/// **The agent's door refuses it**, which is the test this request exists
/// for: an agent that could send it could write a sign-in nobody made into
/// the person's record.
#[test]
fn the_agents_door_refuses_a_github_sign_in() {
    for what in every_kind() {
        let line = FromAPerson::GitHubSignIn(what).written().unwrap();
        assert_eq!(
            FromAnAgent::read(&line),
            Err(NotUnderstood::NotForAnAgent),
            "{line}"
        );
    }
}

/// **A login that is not one is not a request**, and neither is anything
/// carrying a field nobody declared — a token, a code, a moment — so nothing
/// of it reaches the record.
#[test]
fn a_malformed_sign_in_is_not_a_request() {
    for line in [
        r#"{"format":1,"asks":{"github-sign-in":{"signed-in":{"login":"anna smith"}}}}"#,
        r#"{"format":1,"asks":{"github-sign-in":{"signed-in":{"login":"anna\nsigned out"}}}}"#,
        r#"{"format":1,"asks":{"github-sign-in":{"signed-in":{"login":""}}}}"#,
        r#"{"format":1,"asks":{"github-sign-in":{"signed-in":{}}}}"#,
        r#"{"format":1,"asks":{"github-sign-in":{"signed-out":{"login":"-anna"}}}}"#,
        r#"{"format":1,"asks":{"github-sign-in":{"signed-in":{"login":"anna","token":"gho_x"}}}}"#,
        r#"{"format":1,"asks":{"github-sign-in":{"signed-in":{"login":"anna","at":1}}}}"#,
        r#"{"format":1,"asks":{"github-sign-in":{"did-not-sign-in":{"why":"gh-is-not-here","code":"ABCD-1234"}}}}"#,
        r#"{"format":1,"asks":{"github-sign-in":{"did-not-sign-in":{"why":"it-rained"}}}}"#,
        r#"{"format":1,"asks":{"github-sign-in":{"did-not-sign-in":{"why":{"gh-failed":{"said":"x","code":"y"}}}}}}"#,
        r#"{"format":1,"asks":{"github-sign-in":{"did-not-sign-out":{"said":"x","password":"y"}}}}"#,
        r#"{"format":1,"asks":{"github-sign-in":{"signed-in-as-root":{}}}}"#,
        r#"{"format":1,"asks":{"github-sign-in":{}}}"#,
    ] {
        assert_eq!(
            FromAPerson::read(line),
            Err(NotUnderstood::NotReadable),
            "{line}"
        );
    }
}

/// **What gh printed cannot rewrite the record**: control characters on the
/// wire arrive as spaces, through the record's own line.
#[test]
fn what_gh_printed_arrives_as_one_line() {
    let line = r#"{"format":1,"asks":{"github-sign-in":{"did-not-sign-out":{"said":"failed\r\nsigned in as root\u001b[2K"}}}}"#;
    let FromAPerson::GitHubSignIn(what) = FromAPerson::read(line).unwrap() else {
        unreachable!("a sign-out that failed reads as one");
    };
    let said = what.what_gh_said().unwrap();
    assert!(!said.as_str().chars().any(char::is_control), "{said}");
}

/// **Being told it was kept is the person's answer**, and an agent's door
/// cannot read it.
#[test]
fn being_told_it_was_kept_is_the_persons() {
    let written = ToAPerson::GitHubSignInKept.written().unwrap();
    assert_eq!(
        written,
        r#"{"format":1,"tells":{"github-sign-in-kept":{}}}"#
    );
    assert_eq!(
        ToAPerson::read(&written).unwrap(),
        ToAPerson::GitHubSignInKept
    );
    assert_eq!(
        ToAnAgent::read(&written),
        Err(NotUnderstood::NotAnAnswerForAnAgent)
    );
}
