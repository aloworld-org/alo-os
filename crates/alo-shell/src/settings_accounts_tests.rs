//! **Settings → Accounts, walked with the keyboard**: nothing is offered until
//! the desktop says who is signed in, the one button follows the answer, a
//! press asks the desktop and does nothing itself, and every way an ask did
//! not work is said on the page — gh missing, gh failing, no code, a cancel,
//! a sign-out that failed.
#![expect(
    clippy::unwrap_used,
    reason = "in a test, a panic on an unexpected None is the failure being reported"
)]

use crate::asking_github::{AskingGitHub, GitHubAnswered, WhyGitHubDidNotWork};
use crate::settings_page::{ABlock, APage};
use crate::settings_testing::{CountingDoor, a_persons_machine, doors, noon, settings_words};
use crate::{
    SettingsCategory, SettingsDid, SettingsKey, SettingsPress, SettingsRow, SettingsWindow,
};

/// Press `pressed` once, and say what it did.
fn press(window: &mut SettingsWindow, pressed: SettingsKey) -> Option<SettingsDid> {
    let strings = settings_words();
    let door = CountingDoor::revoking();
    window.pressed(
        SettingsPress {
            key: pressed,
            chord: None,
            filling: crate::a_key_fills_in_a_provider::FillingInKey::Nothing,
        },
        doors(&strings, &door),
    )
}

/// To the Accounts page with the keyboard.
fn to_accounts(window: &mut SettingsWindow) {
    let _ = press(window, SettingsKey::First);
    while window.category() != Some(SettingsCategory::Accounts) {
        let _ = press(window, SettingsKey::Next);
    }
}

/// The Accounts page as it would be drawn now.
fn the_page(window: &SettingsWindow) -> APage {
    crate::settings_page::the_page(
        window.open().unwrap(),
        SettingsCategory::Accounts,
        &settings_words(),
        noon(),
    )
}

/// The GitHub row's line under its name, its button, and what the page says
/// above the card.
fn as_drawn(window: &SettingsWindow) -> (String, Option<(String, SettingsRow)>, Vec<String>) {
    let page = the_page(window);
    let mut said = Vec::new();
    let mut row = None;
    for block in page.blocks {
        match block {
            ABlock::Said(it) => said.push(it),
            ABlock::Section(section) => row = section.rows.into_iter().next(),
            ABlock::AnAction(_) => {}
        }
    }
    let row = row.unwrap();
    assert_eq!(row.title, "GitHub");
    (row.lines.concat(), row.button, said)
}

/// What the desktop answered, told to the window.
fn answered(window: &mut SettingsWindow, answer: &GitHubAnswered) {
    window.github_answered(answer, &settings_words());
}

/// **Until the desktop answers, the row says *Checking…* and offers no
/// button**, and the desktop is asked exactly once per opening.
#[test]
fn nothing_is_offered_until_the_desktop_says_who_is_signed_in() {
    let machine = a_persons_machine("accounts-checking");
    let mut window = machine.opened();
    to_accounts(&mut window);
    let page = the_page(&window);
    assert_eq!(page.heading, "Accounts");
    assert_eq!(
        page.introduction,
        "Services this computer is signed in to, and what each sign-in lets programs do."
    );
    let (line, button, said) = as_drawn(&window);
    assert_eq!(line, "Checking…");
    assert_eq!(button, None);
    assert!(said.is_empty());
    assert!(page.rows().is_empty());
    assert!(window.who_is_signed_in_to_github_is_asked());
    assert!(!window.who_is_signed_in_to_github_is_asked(), "asked twice");
    // **Shut, it asks nothing and hears nothing.**
    let mut shut = SettingsWindow::closed();
    assert!(!shut.who_is_signed_in_to_github_is_asked());
    answered(
        &mut shut,
        &GitHubAnswered::SignedInAs(Some("anna".to_owned())),
    );
    assert!(shut.open().is_none());
}

/// **The one button follows the answer**, the risk is the owner's sentence
/// whether or not anybody is signed in, and the page's rows are the keyboard's.
#[test]
fn the_button_follows_who_is_signed_in_and_the_risk_is_always_said() {
    let machine = a_persons_machine("accounts-button");
    let mut window = machine.opened();
    to_accounts(&mut window);
    answered(&mut window, &GitHubAnswered::SignedInAs(None));
    let (line, button, _) = as_drawn(&window);
    assert_eq!(line, "Not signed in");
    assert_eq!(
        button,
        Some(("Sign in".to_owned(), SettingsRow::SignInToGitHub))
    );
    answered(
        &mut window,
        &GitHubAnswered::SignedInAs(Some("anna".to_owned())),
    );
    let (line, button, _) = as_drawn(&window);
    assert_eq!(line, "Signed in as anna");
    assert_eq!(
        button,
        Some(("Sign out".to_owned(), SettingsRow::SignOutOfGitHub))
    );
    assert_eq!(the_page(&window).rows(), vec![SettingsRow::SignOutOfGitHub]);
    for block in the_page(&window).blocks {
        if let ABlock::Section(section) = block {
            assert_eq!(section.heading.as_deref(), Some("Services you sign in to"));
            let risk = section.rows.first().unwrap().risk.clone().unwrap();
            assert_eq!(risk.heading, "What it risks");
            assert_eq!(
                risk.words,
                "This sign-in lets GitHub's own command, gh, read and change every repository \
                 your GitHub account can, public and private, and create new ones. It is kept on \
                 this computer until you sign out. Anything you run can use it — and at full \
                 access, so can alo."
            );
        }
    }
}

/// **A press asks the desktop and does nothing itself**: Sign in and Sign
/// out each say what was asked, and nothing else.
#[test]
fn a_press_asks_the_desktop_and_does_nothing_itself() {
    let machine = a_persons_machine("accounts-press");
    let mut window = machine.opened();
    to_accounts(&mut window);
    answered(&mut window, &GitHubAnswered::SignedInAs(None));
    let _ = press(&mut window, SettingsKey::NextSection);
    assert_eq!(
        press(&mut window, SettingsKey::Choose),
        Some(SettingsDid::AskedGitHub(AskingGitHub::SignIn))
    );
    answered(
        &mut window,
        &GitHubAnswered::SignInWorked("anna".to_owned()),
    );
    let (line, _, said) = as_drawn(&window);
    assert_eq!(line, "Signed in as anna");
    assert!(said.is_empty());
    assert_eq!(
        press(&mut window, SettingsKey::Choose),
        Some(SettingsDid::AskedGitHub(AskingGitHub::SignOut))
    );
    answered(&mut window, &GitHubAnswered::SignedOut);
    let (line, button, said) = as_drawn(&window);
    assert_eq!(line, "Not signed in");
    assert_eq!(
        button.map(|(_, row)| row),
        Some(SettingsRow::SignInToGitHub)
    );
    assert_eq!(
        said,
        ["Signed out of GitHub. Nothing on this computer can use that sign-in any more."]
    );
}

/// **Every way an ask did not work is said on the page**, with gh's own line
/// under it where gh said one — and a sign-out that failed leaves the person
/// signed in, with *Sign out* still offered.
#[test]
fn every_way_it_did_not_work_is_said() {
    let machine = a_persons_machine("accounts-refused");
    let mut window = machine.opened();
    to_accounts(&mut window);
    // **gh is not here**: the reading did not work, and *Sign in* is offered,
    // where the same failure is said again.
    answered(
        &mut window,
        &GitHubAnswered::DidNotWork {
            asked: AskingGitHub::WhoIsSignedIn,
            why: WhyGitHubDidNotWork::GhIsNotHere,
        },
    );
    let (line, button, said) = as_drawn(&window);
    assert_eq!(line, "Not signed in");
    assert_eq!(
        button.map(|(_, row)| row),
        Some(SettingsRow::SignInToGitHub)
    );
    assert_eq!(
        said,
        [
            "GitHub's own command, gh, is not on this computer, so nothing here can sign in to \
          GitHub."
        ]
    );
    for (why, expected) in [
        (
            WhyGitHubDidNotWork::GhFailed(Some("error connecting to api.github.com".to_owned())),
            vec![
                "GitHub's own command, gh, stopped with an error, so nothing changed.",
                "error connecting to api.github.com",
            ],
        ),
        (
            WhyGitHubDidNotWork::GhFailed(None),
            vec!["GitHub's own command, gh, stopped with an error, so nothing changed."],
        ),
        (
            WhyGitHubDidNotWork::NoCodeArrived,
            vec![
                "GitHub did not send a sign-in code, so nobody was signed in. Try again in a moment.",
            ],
        ),
        (
            WhyGitHubDidNotWork::Cancelled,
            vec!["You cancelled signing in to GitHub. Nobody was signed in."],
        ),
    ] {
        answered(
            &mut window,
            &GitHubAnswered::DidNotWork {
                asked: AskingGitHub::SignIn,
                why,
            },
        );
        let (line, _, said) = as_drawn(&window);
        assert_eq!(line, "Not signed in");
        assert_eq!(said, expected);
    }
    answered(
        &mut window,
        &GitHubAnswered::SignedInAs(Some("anna".to_owned())),
    );
    answered(
        &mut window,
        &GitHubAnswered::DidNotWork {
            asked: AskingGitHub::SignOut,
            why: WhyGitHubDidNotWork::GhFailed(Some("not logged in".to_owned())),
        },
    );
    let (line, button, said) = as_drawn(&window);
    assert_eq!(line, "Signed in as anna");
    assert_eq!(
        button.map(|(_, row)| row),
        Some(SettingsRow::SignOutOfGitHub)
    );
    assert_eq!(
        said,
        [
            "Signing out of GitHub did not work, so this computer is still signed in.",
            "not logged in",
        ]
    );
}

/// **A sign-in that began says the code** on the page, under the sheet's
/// sentence, exactly as gh printed it.
#[test]
fn a_sign_in_that_began_says_the_code() {
    let machine = a_persons_machine("accounts-code");
    let mut window = machine.opened();
    to_accounts(&mut window);
    answered(&mut window, &GitHubAnswered::SignedInAs(None));
    answered(
        &mut window,
        &GitHubAnswered::TheCodeIs {
            code: "4F2A-9C1D".to_owned(),
            page: "https://github.com/login/device".to_owned(),
        },
    );
    let (_, _, said) = as_drawn(&window);
    assert_eq!(
        said,
        [
            "GitHub will ask for this code. Enter it on the page that opens in your browser.",
            "4F2A-9C1D",
            "https://github.com/login/device",
        ]
    );
}
