//! What the sheet asks, carried out: a sign-in asks gh once and brings the
//! sheet up; Cancel and Escape ask gh to stop; Open opens the page by the road
//! the code's own page took; and a sign-in that did not work leaves the sheet
//! down and the reason said on Settings → Accounts.

use crate::asking_github::{AskingGitHub, GitHubAnswered, WhyGitHubDidNotWork};
use crate::the_github_sheet::{GitHubSheetKey, OnTheGitHubSheet};

/// A server with Settings open, a browser and a mail app installed.
#[cfg(target_os = "linux")]
fn a_server(
    named: &str,
) -> (
    tempfile::TempDir,
    crate::settings_testing::Machine,
    crate::Server,
) {
    use std::os::unix::fs::PermissionsExt as _;
    let Ok(directory) = tempfile::tempdir() else {
        unreachable!("no private directory");
    };
    assert!(
        std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700)).is_ok()
    );
    let Ok(mut server) = crate::Server::bind(directory.path(), named) else {
        unreachable!("no bound socket");
    };
    let machine = crate::settings_testing::a_persons_machine(named);
    let mut window = machine.opened();
    window.the_applications_are(crate::settings_testing::a_browser_and_a_mail_app());
    server.settings = window;
    (directory, machine, server)
}

/// The code arriving, as the desktop answers it.
fn the_code() -> GitHubAnswered {
    GitHubAnswered::TheCodeIs {
        code: "4F2A-9C1D".to_owned(),
        page: "https://github.com/login/device".to_owned(),
    }
}

/// **Sign in brings the sheet up and asks gh once**; a second press while it
/// is up asks nothing more.
#[cfg(target_os = "linux")]
#[test]
fn the_sheet_opens_on_sign_in_and_asks_once() {
    let (_directory, _machine, mut server) = a_server("github-sheet-opens");
    server.a_github_sign_in_begins();
    assert!(server.github_sheet.is_up());
    server.a_github_sign_in_begins();
    assert_eq!(server.the_github_asks_to_tell(), [AskingGitHub::SignIn]);
}

/// **Cancel asks gh to stop and closes the sheet; Escape is the same.**
#[cfg(target_os = "linux")]
#[test]
fn cancel_and_escape_ask_gh_to_stop() {
    let strings = crate::settings_testing::settings_words();
    let (_directory, _machine, mut server) = a_server("github-sheet-cancel");
    for leaving in [true, false] {
        server.a_github_sign_in_begins();
        server.github_answered(&the_code(), &strings);
        let _ = server.the_github_asks_to_tell();
        if leaving {
            server.the_github_sheet_pressed(OnTheGitHubSheet::Cancel);
        } else {
            server.the_github_sheet_key(GitHubSheetKey::Escape);
        }
        assert!(!server.github_sheet.is_up());
        assert_eq!(
            server.the_github_asks_to_tell(),
            [AskingGitHub::StopSigningIn]
        );
    }
}

/// **Open opens the page again**, the way the code's own page was opened.
#[cfg(target_os = "linux")]
#[test]
fn open_opens_the_page_again() {
    let strings = crate::settings_testing::settings_words();
    let (_directory, _machine, mut server) = a_server("github-sheet-open");
    server.a_github_sign_in_begins();
    server.github_answered(&the_code(), &strings);
    assert_eq!(server.asked_to_open_a_link.len(), 1, "opened when it came");
    server.the_github_sheet_key(GitHubSheetKey::Press);
    let opened: Vec<&str> = server
        .asked_to_open_a_link
        .iter()
        .map(|(_, link)| link.as_str())
        .collect();
    assert_eq!(
        opened,
        [
            "https://github.com/login/device",
            "https://github.com/login/device"
        ]
    );
    assert!(server.github_sheet.is_up(), "Open does not close the sheet");
}

/// **Signed in shows the login; Done closes** and asks nothing of gh.
#[cfg(target_os = "linux")]
#[test]
fn signed_in_then_done() {
    let strings = crate::settings_testing::settings_words();
    let (_directory, _machine, mut server) = a_server("github-sheet-done");
    server.a_github_sign_in_begins();
    server.github_answered(&the_code(), &strings);
    server.github_answered(&GitHubAnswered::SignedIn("anna".to_owned()), &strings);
    let _ = server.the_github_asks_to_tell();
    assert!(matches!(
        server.github_sheet.shows().map(|shows| shows.is.clone()),
        Some(crate::the_github_sheet::TheSheetIs::SignedIn(login)) if login == "anna"
    ));
    server.the_github_sheet_pressed(OnTheGitHubSheet::Done);
    assert!(!server.github_sheet.is_up());
    assert!(server.the_github_asks_to_tell().is_empty());
}

/// **Every failure leaves the sheet down and the reason said** on the
/// Accounts page — gh missing, gh failing, no code, the person cancelling.
#[cfg(target_os = "linux")]
#[test]
fn every_failure_leaves_the_sheet_down_and_the_reason_said() {
    let strings = crate::settings_testing::settings_words();
    let (_directory, _machine, mut server) = a_server("github-sheet-failures");
    for why in [
        WhyGitHubDidNotWork::GhIsNotHere,
        WhyGitHubDidNotWork::GhFailed(Some("HTTP 502".to_owned())),
        WhyGitHubDidNotWork::NoCodeArrived,
        WhyGitHubDidNotWork::Cancelled,
    ] {
        server.a_github_sign_in_begins();
        let failed = GitHubAnswered::DidNotWork {
            asked: AskingGitHub::SignIn,
            why: why.clone(),
        };
        server.github_answered(&failed, &strings);
        assert!(!server.github_sheet.is_up(), "{why:?}");
        let told = server
            .settings
            .open()
            .map(|open| open.github.told.clone())
            .unwrap_or_default();
        assert_eq!(Some(told), failed.said(&strings), "{why:?}");
    }
}

/// **A press heard on a sheet that has gone does nothing.**
#[cfg(target_os = "linux")]
#[test]
fn a_readers_press_on_a_sheet_gone_does_nothing() {
    let (_directory, _machine, mut server) = a_server("github-sheet-reader");
    server.a_github_sign_in_begins();
    let number = server.github_sheet.shows().map_or(0, |shows| shows.number);
    let _ = server.the_github_asks_to_tell();
    assert!(server.a_reader_pressed_on_the_github_sheet(number, OnTheGitHubSheet::Cancel));
    assert_eq!(
        server.the_github_asks_to_tell(),
        [AskingGitHub::StopSigningIn]
    );
    assert!(!server.a_reader_pressed_on_the_github_sheet(number, OnTheGitHubSheet::Cancel));
    assert!(server.the_github_asks_to_tell().is_empty());
}

/// **A sheet no display could draw stops the sign-in.**
#[cfg(target_os = "linux")]
#[test]
fn a_sheet_no_display_could_draw_stops_the_sign_in() {
    let (_directory, _machine, mut server) = a_server("github-sheet-undrawn");
    server.a_github_sign_in_begins();
    let _ = server.the_github_asks_to_tell();
    server.the_github_sheet_could_not_be_drawn();
    assert!(!server.github_sheet.is_up());
    assert_eq!(
        server.the_github_asks_to_tell(),
        [AskingGitHub::StopSigningIn]
    );
}
