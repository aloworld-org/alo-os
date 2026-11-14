//! **A command that needs a GitHub sign-in crosses the person's door as one
//! empty answer, and only to a desktop that asked** (the owner's approval of
//! the sign-in sheet, 2026-11-13).

use crate::{FromAPerson, FromAnAgent, NotUnderstood, ToAPerson, ToAnAgent};

/// **The request and both answers read back, spelled as the contract spells
/// them** — the request, the answer that the desktop is now watching, and the
/// one that a command needs a sign-in.
#[test]
fn the_request_and_both_answers_read_back() {
    let asked = FromAPerson::WatchWhatNeedsAGitHubSignIn.written().unwrap();
    assert_eq!(
        asked,
        r#"{"format":1,"asks":{"watch-what-needs-a-github-sign-in":{}}}"#
    );
    assert_eq!(
        FromAPerson::read(&asked).unwrap(),
        FromAPerson::WatchWhatNeedsAGitHubSignIn
    );
    assert_eq!(FromAPerson::WatchWhatNeedsAGitHubSignIn.number(), None);

    let watching = ToAPerson::WatchingWhatNeedsAGitHubSignIn.written().unwrap();
    assert_eq!(
        watching,
        r#"{"format":1,"tells":{"watching-what-needs-a-github-sign-in":{}}}"#
    );
    assert_eq!(
        ToAPerson::read(&watching).unwrap(),
        ToAPerson::WatchingWhatNeedsAGitHubSignIn
    );

    let needed = ToAPerson::AGitHubSignInIsNeeded.written().unwrap();
    assert_eq!(
        needed,
        r#"{"format":1,"tells":{"a-github-sign-in-is-needed":{}}}"#
    );
    assert_eq!(
        ToAPerson::read(&needed).unwrap(),
        ToAPerson::AGitHubSignInIsNeeded
    );
}

/// **It carries nothing, and a field put in it makes it unreadable** — so no
/// command, no output and no agent's words can ride it onto the screen.
#[test]
fn a_field_put_in_it_is_refused() {
    for line in [
        r#"{"format":1,"tells":{"a-github-sign-in-is-needed":{"said":"fatal: could not read Username"}}}"#,
        r#"{"format":1,"tells":{"a-github-sign-in-is-needed":{"command":"git push"}}}"#,
        r#"{"format":1,"asks":{"watch-what-needs-a-github-sign-in":{"agent":"@alo"}}}"#,
    ] {
        let told = ToAPerson::read(line);
        let asked = FromAPerson::read(line);
        assert!(told.is_err() && asked.is_err(), "{line} was read");
    }
}

/// **A reader of what leaves is not handed it as a list.** The indicator reads
/// every line on its connection as `leaving` and ends on anything else; this
/// answer is never a list, so a reader that met it could not mistake it for
/// *nothing is leaving*.
#[test]
fn it_is_never_read_as_what_leaves() {
    assert_eq!(ToAPerson::AGitHubSignInIsNeeded.lines_leaving(), None);
    assert_eq!(
        ToAPerson::WatchingWhatNeedsAGitHubSignIn.lines_leaving(),
        None
    );
}

/// **An agent may not watch for it, and is never told it**: an agent that
/// could would learn when another agent's command failed, and the sheet is
/// the person's.
#[test]
fn an_agent_may_not_watch_and_is_never_told() {
    assert_eq!(
        FromAnAgent::read(r#"{"format":1,"asks":{"watch-what-needs-a-github-sign-in":{}}}"#),
        Err(NotUnderstood::NotForAnAgent)
    );
    for told in [
        ToAPerson::AGitHubSignInIsNeeded,
        ToAPerson::WatchingWhatNeedsAGitHubSignIn,
    ] {
        let written = told.written().unwrap();
        assert_eq!(
            ToAnAgent::read(&written),
            Err(NotUnderstood::NotAnAnswerForAnAgent),
            "{written}"
        );
    }
}

/// **The lines `docs/contracts/daemon-protocol.md` shows are the ones this
/// crate writes**, so the example a third party copies cannot drift from the
/// wire.
#[test]
fn the_contract_shows_the_lines_this_crate_writes() {
    let contract = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/contracts/daemon-protocol.md"),
    )
    .unwrap();
    for shown in [
        r#"{"watch-what-needs-a-github-sign-in":{}}"#,
        r#"{"watching-what-needs-a-github-sign-in":{}}"#,
        r#"{"a-github-sign-in-is-needed":{}}"#,
    ] {
        assert!(
            contract.contains(shown),
            "the contract does not show {shown}"
        );
    }
}
