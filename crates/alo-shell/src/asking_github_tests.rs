//! **An ask is handed to the desktop once and in order, and an answer that
//! worked says nothing** — the refusal paths of what is said live with the
//! page (`crate::settings_accounts`).

use super::*;

/// **Every ask is handed over once, in the order it was made**, and nothing
/// is handed where nothing was asked.
#[cfg(target_os = "linux")]
#[test]
fn every_ask_is_handed_to_the_desktop_once_in_order() {
    use std::os::unix::fs::PermissionsExt as _;
    let Ok(directory) = tempfile::tempdir() else {
        unreachable!("no private directory");
    };
    assert!(
        std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700)).is_ok()
    );
    let Ok(mut server) = crate::Server::bind(directory.path(), "asking-github") else {
        unreachable!("no bound socket");
    };
    assert!(server.the_github_asks_to_tell().is_empty());
    server.github_is_asked(AskingGitHub::SignIn);
    server.github_is_asked(AskingGitHub::StopSigningIn);
    assert_eq!(
        server.the_github_asks_to_tell(),
        [AskingGitHub::SignIn, AskingGitHub::StopSigningIn]
    );
    assert!(server.the_github_asks_to_tell().is_empty(), "handed once");
}

/// **Only a page on github.com is opened**, whatever gh's output says, and
/// nothing is opened where nothing opens web links.
#[cfg(target_os = "linux")]
#[test]
fn only_a_page_on_github_is_opened() {
    use std::os::unix::fs::PermissionsExt as _;
    let Ok(directory) = tempfile::tempdir() else {
        unreachable!("no private directory");
    };
    assert!(
        std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700)).is_ok()
    );
    let Ok(mut server) = crate::Server::bind(directory.path(), "github-page") else {
        unreachable!("no bound socket");
    };
    let machine = crate::settings_testing::a_persons_machine("github-page");
    let mut window = machine.opened();
    window.the_applications_are(crate::settings_testing::a_browser_and_a_mail_app());
    server.settings = window;
    let strings = crate::settings_testing::settings_words();
    for elsewhere in [
        "https://github.com.example.org/login/device",
        "http://github.com/login/device",
        "https://example.org/https://github.com/",
    ] {
        server.github_answered(
            &GitHubAnswered::TheCodeIs {
                code: "4F2A-9C1D".to_owned(),
                page: elsewhere.to_owned(),
            },
            &strings,
        );
        assert!(server.asked_to_open_a_link.is_empty(), "{elsewhere}");
    }
    server.github_answered(
        &GitHubAnswered::TheCodeIs {
            code: "4F2A-9C1D".to_owned(),
            page: "https://github.com/login/device".to_owned(),
        },
        &strings,
    );
    let opened: Vec<&str> = server
        .asked_to_open_a_link
        .iter()
        .map(|(_, link)| link.as_str())
        .collect();
    assert_eq!(opened, ["https://github.com/login/device"]);
}

/// **What worked says nothing**: a reading, a code, a sign-in and a sign-out
/// are the page's to show, not a sentence about a failure.
#[test]
fn what_worked_says_nothing() {
    let strings = crate::settings_testing::settings_words();
    for answer in [
        GitHubAnswered::SignedInAs(None),
        GitHubAnswered::SignedInAs(Some("anna".to_owned())),
        GitHubAnswered::TheCodeIs {
            code: "4F2A-9C1D".to_owned(),
            page: "https://github.com/login/device".to_owned(),
        },
        GitHubAnswered::SignInWorked("anna".to_owned()),
        GitHubAnswered::SignedOut,
    ] {
        assert_eq!(answer.said(&strings), None, "{answer:?}");
    }
}

/// **gh's own line is said only where it has words**, so a blank last line
/// is not drawn as an empty second line.
#[test]
fn a_blank_line_from_gh_is_not_said() {
    let strings = crate::settings_testing::settings_words();
    let said = GitHubAnswered::DidNotWork {
        asked: AskingGitHub::SignIn,
        why: WhyGitHubDidNotWork::GhFailed(Some("   ".to_owned())),
    }
    .said(&strings)
    .unwrap_or_default();
    assert_eq!(said.len(), 1, "{said:?}");
}
