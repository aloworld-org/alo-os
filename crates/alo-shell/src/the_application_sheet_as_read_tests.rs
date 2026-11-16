//! The sheet as a reader is told about it: in front, read in the order drawn,
//! the length the keyboard is on holding the keyboard, and every press held
//! to this sheet.

use std::time::Instant;

use alo_access::{Surface, TurnedOn};

use super::*;
use crate::the_application_sheet::{AnApplicationWanted, ApplicationSheetKey, TheApplicationSheet};
use crate::the_tasks_a_person_opened::WhichTask;

/// Everything this machine says, and the sheet's words.
fn words() -> Strings {
    let mut vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    crate::the_application_sheets_words::declare_the_application_sheets_words(&mut vocabulary)
        .unwrap();
    Strings::of(vocabulary)
}

/// A sheet up for VSCodium.
fn a_sheet() -> TheApplicationSheet {
    let mut sheet = TheApplicationSheet::default();
    let _ = sheet.wanted(
        AnApplicationWanted::by(WhichTask::numbered(0), "com.vscodium.codium", "VSCodium"),
        Instant::now(),
    );
    sheet
}

/// The desktop's tree with `sheet` on it.
fn read(strings: &Strings, sheet: &TheApplicationSheet) -> ReadAloudTree {
    ReadAloudTree::of(strings, &[Surface::Desktop], &TurnedOn::nothing())
        .with_the_application_sheet(strings, sheet.shows())
}

/// A role's number.
fn role(of: alo_access::Role) -> u32 {
    access_roles::number_of(of)
}

/// **It is read as it is drawn**: the heading, the line, the three lengths
/// with their lines, the note, *Not now* and the navy button — the keyboard
/// on *This task*.
#[test]
fn it_is_read_as_it_is_drawn() {
    let strings = words();
    let sheet = a_sheet();
    let tree = read(&strings, &sheet);
    let in_front = tree.in_front().unwrap();
    let dialogue = &tree.nodes()[in_front.at];
    assert_eq!(dialogue.role, role(alo_access::Role::Dialogue));
    assert_eq!(dialogue.name, "Let alo use VSCodium?");
    let children: Vec<(u32, String)> = dialogue
        .children
        .iter()
        .map(|child| (tree.nodes()[*child].role, tree.nodes()[*child].name.clone()))
        .collect();
    assert_eq!(
        children,
        [
            (
                role(alo_access::Role::Label),
                "alo asked to use VSCodium. Choose how long.".to_owned()
            ),
            (role(alo_access::Role::List), String::new()),
            (
                role(alo_access::Role::Label),
                "Only VSCodium is included. Press Escape to say not now.".to_owned()
            ),
            (role(alo_access::Role::Button), "Not now".to_owned()),
            (
                role(alo_access::Role::Button),
                "Let alo use VSCodium".to_owned()
            ),
        ]
    );
    let list = &tree.nodes()[dialogue.children[1]];
    let lengths: Vec<(String, String)> = list
        .children
        .iter()
        .map(|child| {
            let node = &tree.nodes()[*child];
            (node.name.clone(), node.description.clone())
        })
        .collect();
    assert_eq!(
        lengths,
        [
            (
                "This task".to_owned(),
                "alo may use it in any task until this task's window closes or you stop it"
                    .to_owned()
            ),
            ("This session".to_owned(), "Until you sign out".to_owned()),
            (
                "Remember".to_owned(),
                "Until you take it back in Settings".to_owned()
            ),
        ]
    );
    assert_eq!(tree.nodes()[in_front.keyboard].name, "This task");
}

/// **The keyboard is heard where it is drawn**, after the arrows move it.
#[test]
fn the_keyboard_is_heard_where_it_moved() {
    let strings = words();
    let mut sheet = a_sheet();
    let _ = sheet.pressed(ApplicationSheetKey::Last, Instant::now());
    let tree = read(&strings, &sheet);
    assert_eq!(
        tree.nodes()[tree.in_front().unwrap().keyboard].name,
        "Remember"
    );
}

/// **Every press is held to this sheet**, and a length's press is a length,
/// never the navy button.
#[test]
fn every_press_is_held_to_this_sheet() {
    let strings = words();
    let sheet = a_sheet();
    let number = sheet.shows().unwrap().number;
    let tree = read(&strings, &sheet);
    let presses: Vec<Pressed> = tree
        .nodes()
        .iter()
        .filter_map(|node| node.press.clone())
        .filter(|press| matches!(press, Pressed::OnTheApplicationSheet { .. }))
        .collect();
    assert_eq!(
        presses,
        [
            OnTheApplicationSheet::For(HowLongForAnApplication::ThisTask),
            OnTheApplicationSheet::For(HowLongForAnApplication::ThisSession),
            OnTheApplicationSheet::For(HowLongForAnApplication::Remember),
            OnTheApplicationSheet::NotNow,
            OnTheApplicationSheet::LetAloUseIt,
        ]
        .map(|on| Pressed::OnTheApplicationSheet { sheet: number, on })
    );
}

/// **With no sheet up the tree is left as it was.**
#[test]
fn with_no_sheet_the_tree_is_left_as_it_was() {
    let strings = words();
    let tree = read(&strings, &TheApplicationSheet::default());
    assert!(tree.in_front().is_none());
}
