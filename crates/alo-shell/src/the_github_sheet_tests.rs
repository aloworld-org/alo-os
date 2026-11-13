//! The GitHub sign-in sheet as the person moves through it: up waiting, the
//! code arriving, Copy and Open asking without closing, Cancel and Escape
//! stopping the sign-in, Done closing it once signed in — and every way a
//! sign-in does not work taking it down, with nothing asked of gh twice.

use super::*;
use crate::asking_github::WhyGitHubDidNotWork;

/// The code gh printed in these tests.
const CODE: &str = "4F2A-9C1D";
/// The page gh printed in these tests.
const PAGE: &str = "https://github.com/login/device";

/// A sheet up, with the code arrived.
fn with_the_code() -> TheGitHubSheet {
    let mut sheet = TheGitHubSheet::default();
    assert!(sheet.opened());
    assert!(sheet.answered(&GitHubAnswered::TheCodeIs {
        code: CODE.to_owned(),
        page: PAGE.to_owned(),
    }));
    sheet
}

/// What the sheet shows now, or [`None`] when it is down.
fn is(sheet: &TheGitHubSheet) -> Option<(TheSheetIs, OnTheGitHubSheet)> {
    sheet.shows().map(|shows| (shows.is.clone(), shows.focus))
}

/// **It goes up waiting for the code**, with *Cancel* alone, and a second
/// sign-in while it is up opens nothing more.
#[test]
fn it_opens_waiting_for_the_code_and_only_once() {
    let mut sheet = TheGitHubSheet::default();
    assert!(!sheet.is_up());
    assert!(sheet.opened());
    assert_eq!(
        is(&sheet),
        Some((TheSheetIs::WaitingForTheCode, OnTheGitHubSheet::Cancel))
    );
    assert_eq!(
        sheet.shows().map(|shows| shows.is.buttons()),
        Some(&[OnTheGitHubSheet::Cancel][..])
    );
    assert!(!sheet.opened(), "one sheet, one sign-in");
}

/// **The code fills the card** and the keyboard goes to the navy button.
#[test]
fn the_code_arrives() {
    let sheet = with_the_code();
    assert_eq!(
        is(&sheet),
        Some((
            TheSheetIs::TheCodeIs {
                code: CODE.to_owned(),
                page: PAGE.to_owned(),
            },
            OnTheGitHubSheet::OpenThePage
        ))
    );
}

/// **A code nobody is waiting for is not taken**: before the sheet is up, or
/// a second code once one has arrived.
#[test]
fn a_code_nobody_waits_for_is_not_taken() {
    let code = GitHubAnswered::TheCodeIs {
        code: "AAAA-BBBB".to_owned(),
        page: PAGE.to_owned(),
    };
    let mut down = TheGitHubSheet::default();
    assert!(!down.answered(&code));
    assert!(!down.is_up());
    let mut sheet = with_the_code();
    assert!(!sheet.answered(&code));
    assert!(matches!(
        is(&sheet),
        Some((TheSheetIs::TheCodeIs { code, .. }, _)) if code == CODE
    ));
}

/// **Copy asks for the code on the clipboard, Open for the page**, and
/// neither closes the sheet.
#[test]
fn copy_and_open_ask_and_keep_the_sheet_up() {
    let mut sheet = with_the_code();
    assert_eq!(
        sheet.press(OnTheGitHubSheet::Copy),
        Some(TheSheetAsks::Copy(CODE.to_owned()))
    );
    assert!(sheet.is_up());
    assert_eq!(
        sheet.pressed(GitHubSheetKey::Last),
        None,
        "moving asks nothing"
    );
    assert_eq!(
        sheet.pressed(GitHubSheetKey::Press),
        Some(TheSheetAsks::OpenThePage(PAGE.to_owned()))
    );
    assert!(sheet.is_up());
}

/// **Tab goes through the buttons in the order drawn, and round again**;
/// Shift+Tab the other way.
#[test]
fn tab_goes_round_the_buttons() {
    let mut sheet = with_the_code();
    let mut went = Vec::new();
    for _ in 0..4 {
        assert_eq!(sheet.pressed(GitHubSheetKey::Next), None);
        went.push(sheet.shows().map(|shows| shows.focus));
    }
    assert_eq!(
        went,
        [
            Some(OnTheGitHubSheet::Copy),
            Some(OnTheGitHubSheet::Cancel),
            Some(OnTheGitHubSheet::OpenThePage),
            Some(OnTheGitHubSheet::Copy),
        ]
    );
    assert_eq!(sheet.pressed(GitHubSheetKey::Previous), None);
    assert_eq!(
        sheet.shows().map(|shows| shows.focus),
        Some(OnTheGitHubSheet::OpenThePage)
    );
    assert_eq!(sheet.pressed(GitHubSheetKey::First), None);
    assert_eq!(
        sheet.shows().map(|shows| shows.focus),
        Some(OnTheGitHubSheet::Copy)
    );
}

/// **Cancel asks gh to stop and closes the sheet**, waiting or with the code.
#[test]
fn cancel_stops_the_sign_in_and_closes() {
    let mut waiting = TheGitHubSheet::default();
    assert!(waiting.opened());
    assert_eq!(
        waiting.press(OnTheGitHubSheet::Cancel),
        Some(TheSheetAsks::StopSigningIn)
    );
    assert!(!waiting.is_up());
    let mut sheet = with_the_code();
    assert_eq!(
        sheet.press(OnTheGitHubSheet::Cancel),
        Some(TheSheetAsks::StopSigningIn)
    );
    assert!(!sheet.is_up());
    assert_eq!(sheet.press(OnTheGitHubSheet::Cancel), None, "asked once");
}

/// **Escape is Cancel** while signing in.
#[test]
fn escape_is_cancel() {
    let mut sheet = with_the_code();
    assert_eq!(
        sheet.pressed(GitHubSheetKey::Escape),
        Some(TheSheetAsks::StopSigningIn)
    );
    assert!(!sheet.is_up());
}

/// **Signed in shows the login, and Done closes** — Escape too, and neither
/// asks gh to stop a sign-in that worked.
#[test]
fn signed_in_shows_the_login_and_done_closes() {
    for leaving in [GitHubSheetKey::Press, GitHubSheetKey::Escape] {
        let mut sheet = with_the_code();
        assert!(!sheet.answered(&GitHubAnswered::SignedIn("anna".to_owned())));
        assert_eq!(
            is(&sheet),
            Some((
                TheSheetIs::SignedInAs("anna".to_owned()),
                OnTheGitHubSheet::Done
            ))
        );
        assert_eq!(sheet.press(OnTheGitHubSheet::Cancel), None, "not drawn");
        assert!(sheet.is_up());
        assert_eq!(sheet.pressed(leaving), None);
        assert!(!sheet.is_up(), "{leaving:?}");
    }
}

/// **Every way a sign-in does not work takes the sheet down**: gh missing, gh
/// failing, no code arriving, and the person cancelling — whether or not the
/// code had arrived. Why is the Accounts page's to say.
#[test]
fn every_failure_takes_the_sheet_down() {
    for why in [
        WhyGitHubDidNotWork::GhIsNotHere,
        WhyGitHubDidNotWork::GhFailed(Some("HTTP 502".to_owned())),
        WhyGitHubDidNotWork::NoCodeArrived,
        WhyGitHubDidNotWork::Cancelled,
    ] {
        for code_arrived in [false, true] {
            let mut sheet = if code_arrived {
                with_the_code()
            } else {
                let mut sheet = TheGitHubSheet::default();
                assert!(sheet.opened());
                sheet
            };
            assert!(!sheet.answered(&GitHubAnswered::DidNotWork {
                asked: AskingGitHub::SignIn,
                why: why.clone(),
            }));
            assert!(!sheet.is_up(), "{why:?}, code arrived: {code_arrived}");
        }
    }
}

/// **What is not about this sign-in leaves the sheet as it is**: a reading,
/// a sign-out failing, a sign-out.
#[test]
fn what_is_not_about_the_sign_in_leaves_it() {
    let mut sheet = with_the_code();
    let before = is(&sheet);
    for answer in [
        GitHubAnswered::SignedInAs(None),
        GitHubAnswered::SignedOut,
        GitHubAnswered::DidNotWork {
            asked: AskingGitHub::SignOut,
            why: WhyGitHubDidNotWork::GhFailed(None),
        },
        GitHubAnswered::DidNotWork {
            asked: AskingGitHub::WhoIsSignedIn,
            why: WhyGitHubDidNotWork::GhIsNotHere,
        },
    ] {
        assert!(!sheet.answered(&answer));
        assert_eq!(is(&sheet), before, "{answer:?}");
    }
}

/// **A button not drawn does nothing**: no Copy and no Open before the code,
/// no Done before signing in.
#[test]
fn a_button_not_drawn_does_nothing() {
    let mut sheet = TheGitHubSheet::default();
    assert!(sheet.opened());
    for on in [
        OnTheGitHubSheet::Copy,
        OnTheGitHubSheet::OpenThePage,
        OnTheGitHubSheet::Done,
    ] {
        assert_eq!(sheet.press(on), None, "{on:?}");
        assert!(sheet.is_up(), "{on:?}");
    }
}

/// **A reader's press is held to the sheet it was heard on**: a press for an
/// earlier sheet, or for a button not drawn, does nothing.
#[test]
fn a_readers_press_is_held_to_its_sheet() {
    let mut sheet = with_the_code();
    let number = sheet.shows().map_or(0, |shows| shows.number);
    assert_eq!(
        sheet.a_reader_pressed(number + 1, OnTheGitHubSheet::Cancel),
        None
    );
    assert_eq!(sheet.a_reader_pressed(number, OnTheGitHubSheet::Done), None);
    assert!(sheet.is_up());
    assert_eq!(
        sheet.a_reader_pressed(number, OnTheGitHubSheet::Copy),
        Some(Some(TheSheetAsks::Copy(CODE.to_owned())))
    );
    assert_eq!(
        sheet.a_reader_pressed(number, OnTheGitHubSheet::Cancel),
        Some(Some(TheSheetAsks::StopSigningIn))
    );
    assert!(!sheet.is_up());
    assert!(sheet.opened());
    assert_eq!(
        sheet.a_reader_pressed(number, OnTheGitHubSheet::Cancel),
        None,
        "a press heard on the last sheet never acts on this one"
    );
}

/// **A sheet no display can draw stops a sign-in still waiting on the
/// person**, and once signed in only comes down.
#[test]
fn a_sheet_that_cannot_be_drawn_stops_the_sign_in() {
    let mut sheet = with_the_code();
    assert_eq!(
        sheet.could_not_be_drawn(),
        Some(TheSheetAsks::StopSigningIn)
    );
    assert!(!sheet.is_up());
    let mut signed_in = with_the_code();
    assert!(!signed_in.answered(&GitHubAnswered::SignedIn("anna".to_owned())));
    assert_eq!(signed_in.could_not_be_drawn(), None);
    assert!(!signed_in.is_up());
    assert_eq!(TheGitHubSheet::default().could_not_be_drawn(), None);
}

/// **Keys are read as Settings reads them**: Tab and the arrows move, Enter
/// presses, Escape leaves, and anything else does nothing.
#[test]
fn keys_are_read_as_settings_reads_them() {
    use crate::SettingsKey as K;
    for (key, means) in [
        (K::NextSection, GitHubSheetKey::Next),
        (K::Next, GitHubSheetKey::Next),
        (K::More, GitHubSheetKey::Next),
        (K::PreviousSection, GitHubSheetKey::Previous),
        (K::Previous, GitHubSheetKey::Previous),
        (K::Less, GitHubSheetKey::Previous),
        (K::Choose, GitHubSheetKey::Press),
        (K::Close, GitHubSheetKey::Escape),
        (K::PutBackThisOne, GitHubSheetKey::Nothing),
        (K::Nothing, GitHubSheetKey::Nothing),
    ] {
        assert_eq!(GitHubSheetKey::of(key), means, "{key:?}");
    }
}
