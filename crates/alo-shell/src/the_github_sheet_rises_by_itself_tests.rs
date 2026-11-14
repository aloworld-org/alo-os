//! **A command that needed a GitHub sign-in puts the sheet up once**: never
//! two, never over another sheet, never for ever, and never without a knock.

use std::time::{Duration, Instant};

use super::{ASignInWanted, ItGoesUp, THE_LONGEST};
use crate::asking_github::AskingGitHub;

/// **One failure, one sheet**: it goes up on the next frame with no sheet in
/// the way, and the frame after has nothing more to put up.
#[test]
fn one_failure_puts_the_sheet_up_once() {
    let now = Instant::now();
    let mut wanted = ASignInWanted::default();
    wanted.heard(false, now);
    assert_eq!(wanted.this_frame(false, false, now), ItGoesUp::Now);
    assert_eq!(wanted.this_frame(false, false, now), ItGoesUp::NotNow);
}

/// **Nothing heard, nothing goes up** — the sheet opens only for a command
/// the service said needed a sign-in, never on a frame by itself.
#[test]
fn nothing_heard_puts_nothing_up() {
    let now = Instant::now();
    let mut wanted = ASignInWanted::default();
    for _ in 0..3 {
        assert_eq!(wanted.this_frame(false, false, now), ItGoesUp::NotNow);
    }
}

/// **Many failures before a frame are one sheet**, not one each.
#[test]
fn many_failures_before_a_frame_are_one_sheet() {
    let now = Instant::now();
    let mut wanted = ASignInWanted::default();
    for _ in 0..5 {
        wanted.heard(false, now);
    }
    assert_eq!(wanted.this_frame(false, false, now), ItGoesUp::Now);
    assert_eq!(wanted.this_frame(false, false, now), ItGoesUp::NotNow);
}

/// **A failure while the GitHub sheet is up is let go**: the sheet up is
/// already asking it, and nothing goes up after it comes down.
#[test]
fn a_failure_while_the_sheet_is_up_does_not_stack() {
    let now = Instant::now();
    let mut wanted = ASignInWanted::default();
    wanted.heard(true, now);
    assert_eq!(wanted.this_frame(false, false, now), ItGoesUp::NotNow);

    // And one heard just before the sheet went up some other way — from
    // Settings → Accounts — is answered by that sheet.
    wanted.heard(false, now);
    assert_eq!(wanted.this_frame(true, false, now), ItGoesUp::NotNow);
    assert_eq!(wanted.this_frame(false, false, now), ItGoesUp::NotNow);
}

/// **Behind another sheet it waits, and goes up when that one comes down** —
/// the *Open with* sheet's rule.
#[test]
fn behind_another_sheet_it_waits_then_goes_up() {
    let now = Instant::now();
    let mut wanted = ASignInWanted::default();
    wanted.heard(false, now);
    let later = now + Duration::from_secs(30);
    assert_eq!(wanted.this_frame(false, true, now), ItGoesUp::NotNow);
    assert_eq!(wanted.this_frame(false, true, later), ItGoesUp::NotNow);
    assert_eq!(wanted.this_frame(false, false, later), ItGoesUp::Now);
}

/// **Never for ever**: one that waited longer than the *Open with* sheet's
/// bound is let go rather than rising over something the person has moved on
/// from.
#[test]
fn it_waits_no_longer_than_the_open_with_sheet_does() {
    let now = Instant::now();
    let mut wanted = ASignInWanted::default();
    wanted.heard(false, now);
    let too_late = now + THE_LONGEST + Duration::from_secs(1);
    assert_eq!(wanted.this_frame(false, true, too_late), ItGoesUp::NotNow);
    assert_eq!(wanted.this_frame(false, false, too_late), ItGoesUp::NotNow);
}

/// A server with Settings open, as the sheet's own tests stand one up.
#[cfg(target_os = "linux")]
fn a_server(named: &str) -> (tempfile::TempDir, crate::Server) {
    use std::os::unix::fs::PermissionsExt as _;
    let Ok(directory) = tempfile::tempdir() else {
        unreachable!("no private directory");
    };
    assert!(
        std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700)).is_ok()
    );
    let Ok(server) = crate::Server::bind(directory.path(), named) else {
        unreachable!("no bound socket");
    };
    (directory, server)
}

/// What the sheet up on `server` shows, if one is.
#[cfg(target_os = "linux")]
fn showing(server: &crate::Server) -> Option<crate::the_github_sheet::TheSheetIs> {
    server.github_sheet.shows().map(|shows| shows.is.clone())
}

/// **Nothing is started before the press** — the owner, 2026-11-13: *"Wait
/// for my press"*. The sheet rises saying what failed, gh is asked nothing,
/// and a second failure while it is up adds nothing. Only *Sign in* asks gh,
/// once, and the sheet then waits for the code.
#[cfg(target_os = "linux")]
#[test]
fn on_the_server_nothing_is_started_before_the_press() {
    use crate::the_github_sheet::{OnTheGitHubSheet, TheSheetIs};
    let (_directory, mut server) = a_server("github-wanted-press");
    let now = Instant::now();
    server.a_command_needed_a_github_sign_in(now);
    server.a_wanted_github_sign_in_goes_up(false, now);
    assert_eq!(showing(&server), Some(TheSheetIs::ACommandNeededIt));
    assert!(
        server.the_github_asks_to_tell().is_empty(),
        "gh was asked before the person pressed anything"
    );

    server.a_command_needed_a_github_sign_in(now);
    server.a_wanted_github_sign_in_goes_up(false, now);
    assert!(server.the_github_asks_to_tell().is_empty());

    server.the_github_sheet_pressed(OnTheGitHubSheet::SignIn);
    assert_eq!(showing(&server), Some(TheSheetIs::WaitingForTheCode));
    assert_eq!(server.the_github_asks_to_tell(), [AskingGitHub::SignIn]);
}

/// **Escape at the first state starts nothing**: the sheet comes down and gh
/// is asked nothing — not even to stop, because nothing began, so nothing is
/// written down as a sign-in the person cancelled. *Cancel* is the same.
#[cfg(target_os = "linux")]
#[test]
fn on_the_server_escape_at_the_first_state_starts_nothing() {
    use crate::the_github_sheet::{GitHubSheetKey, OnTheGitHubSheet};
    let (_directory, mut server) = a_server("github-wanted-escape");
    let now = Instant::now();
    server.a_command_needed_a_github_sign_in(now);
    server.a_wanted_github_sign_in_goes_up(false, now);
    assert!(server.github_sheet.is_up());
    server.the_github_sheet_key(GitHubSheetKey::Escape);
    assert!(!server.github_sheet.is_up());
    assert!(server.the_github_asks_to_tell().is_empty());

    server.a_command_needed_a_github_sign_in(now);
    server.a_wanted_github_sign_in_goes_up(false, now);
    server.the_github_sheet_pressed(OnTheGitHubSheet::Cancel);
    assert!(!server.github_sheet.is_up());
    assert!(server.the_github_asks_to_tell().is_empty());
}

/// **From Settings → Accounts a press was made already**, so that road still
/// starts at once, waiting for the code.
#[cfg(target_os = "linux")]
#[test]
fn from_settings_it_still_starts_at_once() {
    use crate::the_github_sheet::TheSheetIs;
    let (_directory, mut server) = a_server("github-wanted-settings");
    server.a_github_sign_in_begins();
    assert_eq!(showing(&server), Some(TheSheetIs::WaitingForTheCode));
    assert_eq!(server.the_github_asks_to_tell(), [AskingGitHub::SignIn]);
}

/// **On the server, nothing goes up while another sheet is**, and the sheet
/// goes up the frame that one comes down.
#[cfg(target_os = "linux")]
#[test]
fn on_the_server_it_waits_behind_another_sheet() {
    let (_directory, mut server) = a_server("github-wanted-waits");
    let now = Instant::now();
    server.a_command_needed_a_github_sign_in(now);
    server.a_wanted_github_sign_in_goes_up(true, now);
    assert!(!server.github_sheet.is_up());
    assert!(server.the_github_asks_to_tell().is_empty());

    server.a_wanted_github_sign_in_goes_up(false, now);
    assert!(server.github_sheet.is_up());
    assert!(server.the_github_asks_to_tell().is_empty());
}
