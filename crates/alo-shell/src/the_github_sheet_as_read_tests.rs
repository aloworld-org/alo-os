//! The GitHub sign-in sheet as a reader is told about it: in front, read in
//! the order drawn, the button the keyboard is on holding the keyboard, and
//! every press held to this sheet — in both states.

use alo_access::{Surface, TurnedOn};

use super::*;
use crate::asking_github::GitHubAnswered;
use crate::the_github_sheet::{GitHubSheetKey, TheGitHubSheet};

/// Settings' words and the sheet's.
fn words() -> Strings {
    let mut vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    crate::the_settings_words::declare_the_settings_words(&mut vocabulary).unwrap();
    crate::the_github_sheets_words::declare_the_github_sheets_words(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A sheet up with the code arrived.
fn with_the_code() -> TheGitHubSheet {
    let mut sheet = TheGitHubSheet::default();
    assert!(sheet.opened());
    assert!(sheet.answered(&GitHubAnswered::TheCodeIs {
        code: "4F2A-9C1D".to_owned(),
        page: "https://github.com/login/device".to_owned(),
    }));
    sheet
}

/// The desktop's tree with `sheet` on it.
fn read(strings: &Strings, sheet: &TheGitHubSheet) -> ReadAloudTree {
    ReadAloudTree::of(strings, &[Surface::Desktop], &TurnedOn::nothing())
        .with_the_github_sheet(strings, sheet.shows())
}

/// The roles and names under the sheet's own node, in order.
fn read_in_order(tree: &ReadAloudTree) -> Vec<(u32, String)> {
    let in_front = tree.in_front().unwrap();
    tree.nodes()[in_front.at]
        .children
        .iter()
        .map(|child| {
            let node = &tree.nodes()[*child];
            (node.role, node.name.clone())
        })
        .collect()
}

/// A role's number.
fn role(of: alo_access::Role) -> u32 {
    access_roles::number_of(of)
}

/// **Signing in is read as it is drawn**: the title, the sentence, the code,
/// Copy, what it risks, Cancel and Open — with the keyboard on Open.
#[test]
fn signing_in_is_read_as_it_is_drawn() {
    let strings = words();
    let sheet = with_the_code();
    let tree = read(&strings, &sheet);
    let in_front = tree.in_front().unwrap();
    let dialogue = &tree.nodes()[in_front.at];
    assert_eq!(dialogue.role, role(alo_access::Role::Dialogue));
    assert_eq!(dialogue.name, "Sign in to GitHub");
    assert_eq!(
        read_in_order(&tree),
        [
            (
                role(alo_access::Role::Label),
                "GitHub will ask for this code. Enter it on the page that opens in your browser."
                    .to_owned()
            ),
            (role(alo_access::Role::Label), "4F2A-9C1D".to_owned()),
            (role(alo_access::Role::Button), "Copy".to_owned()),
            (role(alo_access::Role::Label), "What it risks".to_owned()),
            (role(alo_access::Role::Button), "Cancel".to_owned()),
            (
                role(alo_access::Role::Button),
                "Open github.com/login/device".to_owned()
            ),
        ]
    );
    let risk = tree.nodes()[in_front.at].children[3];
    assert!(
        tree.nodes()[risk]
            .description
            .starts_with("This sign-in lets GitHub's own command, gh,")
    );
    assert_eq!(
        tree.nodes()[in_front.keyboard].name,
        "Open github.com/login/device"
    );
}

/// **While waiting there is nothing to copy or open**: *Checking…* and Cancel.
#[test]
fn waiting_is_read_as_it_is_drawn() {
    let strings = words();
    let mut sheet = TheGitHubSheet::default();
    assert!(sheet.opened());
    let tree = read(&strings, &sheet);
    let names: Vec<String> = read_in_order(&tree)
        .into_iter()
        .map(|(_, name)| name)
        .collect();
    assert_eq!(
        names,
        [
            "GitHub will ask for this code. Enter it on the page that opens in your browser.",
            "Checking…",
            "What it risks",
            "Cancel",
        ]
    );
    assert_eq!(
        tree.nodes()[tree.in_front().unwrap().keyboard].name,
        "Cancel"
    );
}

/// **Signed in is read with the login**, and Done holds the keyboard.
#[test]
fn signed_in_is_read_with_the_login() {
    let strings = words();
    let mut sheet = with_the_code();
    assert!(!sheet.answered(&GitHubAnswered::SignedIn("anna".to_owned())));
    let tree = read(&strings, &sheet);
    let in_front = tree.in_front().unwrap();
    assert_eq!(
        tree.nodes()[in_front.at].name,
        "Signed in to GitHub as anna"
    );
    let names: Vec<String> = read_in_order(&tree)
        .into_iter()
        .map(|(_, name)| name)
        .collect();
    assert_eq!(
        names,
        [
            "Signed in. You can sign out in Settings → Accounts.",
            "What it risks",
            "Done",
        ]
    );
    assert_eq!(tree.nodes()[in_front.keyboard].name, "Done");
}

/// **The keyboard follows Tab**, and only one thing on the tree holds it.
#[test]
fn the_keyboard_follows_tab() {
    let strings = words();
    let mut sheet = with_the_code();
    for expected in ["Copy", "Cancel", "Open github.com/login/device"] {
        let _ = sheet.pressed(GitHubSheetKey::Next);
        let tree = read(&strings, &sheet);
        let focused: Vec<&str> = tree
            .nodes()
            .iter()
            .filter(|node| access_roles::has_the_keyboard(node.states))
            .map(|node| node.name.as_str())
            .collect();
        assert_eq!(focused, [expected]);
    }
}

/// **Every press is this sheet's, and only the buttons are pressable.**
#[test]
fn only_the_buttons_are_pressed_and_each_is_held_to_this_sheet() {
    let strings = words();
    let sheet = with_the_code();
    let number = sheet.shows().unwrap().number;
    let tree = read(&strings, &sheet);
    let on: Vec<OnTheGitHubSheet> = tree
        .what_can_be_pressed()
        .into_iter()
        .filter_map(|pressed| match pressed {
            Pressed::OnTheGitHubSheet { sheet, on } => {
                assert_eq!(sheet, number, "held to the sheet it is heard on");
                Some(on)
            }
            _ => None,
        })
        .collect();
    assert_eq!(
        on,
        [
            OnTheGitHubSheet::Copy,
            OnTheGitHubSheet::Cancel,
            OnTheGitHubSheet::OpenThePage,
        ]
    );
}

/// **No sheet, no change.**
#[test]
fn with_no_sheet_the_tree_is_as_it_was() {
    let strings = words();
    let plain = ReadAloudTree::of(&strings, &[Surface::Desktop], &TurnedOn::nothing());
    let tree = plain.clone().with_the_github_sheet(&strings, None);
    assert_eq!(tree, plain);
}
