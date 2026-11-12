//! The *Open with* sheet as a reader is told about it: the title, the line
//! under it, each application by its name with its line as description, the
//! one the arrows are on selected and holding the keyboard, then Cancel and
//! Open — each pressable, and every press held to this sheet.

use std::os::unix::net::UnixStream;
use std::time::Instant;

use alo_access::{Surface, TurnedOn};
use alo_acting::ToChooseALink;
use alo_applications::{Application, Link};

use super::*;
use crate::the_open_with_sheet::{AskedToChoose, SheetKey, TheOpenWithSheet};

/// The machine's words, the sheet's among them.
fn words() -> Strings {
    let mut vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    crate::the_open_with_sheets_words::declare_the_open_with_sheets_words(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// One word, said.
fn say(strings: &Strings, word: Word) -> String {
    strings.say(&word.key(), &Filling::nothing()).into_text()
}

/// What each application is called here: Firefox has a name, the rest none.
fn called(identifier: &str) -> String {
    if identifier == "org.mozilla.firefox" {
        "Firefox".to_owned()
    } else {
        identifier.to_owned()
    }
}

/// A sheet up for `link`, offering three, with its portal's end held.
fn a_sheet(link: &str) -> (TheOpenWithSheet, UnixStream) {
    let (portal, desktop) = UnixStream::pair().unwrap();
    desktop.set_nonblocking(true).unwrap();
    let app = |id: &str| Application::identified(id).unwrap();
    let asked = ToChooseALink::of(
        app("org.gnome.Papers"),
        Link::checked(link).unwrap(),
        vec![
            app("org.mozilla.firefox"),
            app("io.gitlab.librewolf-community"),
            app("org.chromium.Chromium"),
        ],
    )
    .unwrap();
    let mut sheet = TheOpenWithSheet::none();
    sheet.asked(
        vec![AskedToChoose::of(asked, desktop)],
        true,
        Instant::now(),
    );
    (sheet, portal)
}

/// The desktop's tree with `sheet` on it.
fn read(strings: &Strings, sheet: &TheOpenWithSheet) -> ReadAloudTree {
    ReadAloudTree::of(strings, &[Surface::Desktop], &TurnedOn::nothing()).with_the_open_with_sheet(
        strings,
        sheet.shows(),
        &called,
    )
}

/// What hangs under `at`.
fn under(tree: &ReadAloudTree, at: usize) -> Vec<&Node> {
    tree.nodes()[at]
        .children
        .iter()
        .map(|child| &tree.nodes()[*child])
        .collect()
}

/// **Read as the frame is drawn**, in the order it is drawn.
#[test]
fn the_sheet_is_read_as_it_is_drawn() {
    let strings = words();
    let (sheet, _portal) = a_sheet("https://example.com/invoice");
    let tree = read(&strings, &sheet);
    let in_front = tree.in_front().unwrap();
    let dialogue = &tree.nodes()[in_front.at];
    assert_eq!(
        dialogue.role,
        access_roles::number_of(alo_access::Role::Dialogue)
    );
    assert_eq!(dialogue.name, say(&strings, words::OPEN_WITH));
    assert_eq!(dialogue.parent, 0);
    let parts = under(&tree, in_front.at);
    let roles: Vec<u32> = parts.iter().map(|it| it.role).collect();
    assert_eq!(
        roles,
        [
            access_roles::number_of(alo_access::Role::Label),
            access_roles::number_of(alo_access::Role::List),
            access_roles::number_of(alo_access::Role::Button),
            access_roles::number_of(alo_access::Role::Button),
        ]
    );
    assert_eq!(
        parts[0].name,
        strings
            .say(
                &words::ASKED_TO_OPEN.key(),
                &Filling::of("application", "org.gnome.Papers")
                    .and("address", "https://example.com/invoice"),
            )
            .into_text()
    );
    assert_eq!(parts[2].name, say(&strings, words::CANCEL));
    assert_eq!(parts[3].name, say(&strings, words::OPEN));
    let list = tree.nodes()[in_front.at].children[1];
    let rows: Vec<(&str, &str)> = under(&tree, list)
        .iter()
        .map(|row| (row.name.as_str(), row.description.as_str()))
        .collect();
    let installed = say(&strings, words::INSTALLED);
    assert_eq!(
        rows,
        [
            (
                "Firefox",
                say(&strings, words::YOUR_DEFAULT_FOR_WEB_LINKS).as_str()
            ),
            ("io.gitlab.librewolf-community", installed.as_str()),
            ("org.chromium.Chromium", installed.as_str()),
        ]
    );
}

/// **An email address is said to be one**, under the person's default.
#[test]
fn the_default_for_an_email_address_is_said_so() {
    let strings = words();
    let (sheet, _portal) = a_sheet("mailto:someone@example.com");
    let tree = read(&strings, &sheet);
    let keyboard = &tree.nodes()[tree.in_front().unwrap().keyboard];
    assert_eq!(
        keyboard.description,
        say(&strings, words::YOUR_DEFAULT_FOR_EMAIL_ADDRESSES)
    );
}

/// **The keyboard starts on the default and follows the arrows**: the row it
/// is on is the one selected and the only thing focused, on the whole tree.
#[test]
fn the_keyboard_is_on_the_row_the_arrows_are_on() {
    let strings = words();
    let (mut sheet, _portal) = a_sheet("https://example.com/invoice");
    for (expected, then) in [
        ("Firefox", SheetKey::Next),
        ("io.gitlab.librewolf-community", SheetKey::Last),
    ] {
        let tree = read(&strings, &sheet);
        let focused: Vec<&str> = tree
            .nodes()
            .iter()
            .filter(|node| access_roles::has_the_keyboard(node.states))
            .map(|node| node.name.as_str())
            .collect();
        assert_eq!(focused, [expected]);
        let selected = tree
            .nodes()
            .iter()
            .filter(|node| node.states[0] & (1 << access_roles::SELECTED) != 0)
            .count();
        assert_eq!(
            selected, 1,
            "one row is selected, the one with the keyboard"
        );
        assert_eq!(
            tree.nodes()[tree.in_front().unwrap().keyboard].name,
            expected
        );
        sheet.pressed(then, Instant::now());
    }
}

/// **Every press is this sheet's**, and only what a click lands on is
/// pressable: each row, Cancel and Open — not the title, the line or the list.
#[test]
fn what_can_be_pressed_is_what_a_click_lands_on() {
    let strings = words();
    let (sheet, _portal) = a_sheet("https://example.com/invoice");
    let number = sheet.shows().unwrap().number;
    let tree = read(&strings, &sheet);
    let on: Vec<OnTheOpenWithSheet> = tree
        .what_can_be_pressed()
        .into_iter()
        .map(|pressed| match pressed {
            Pressed::OnTheOpenWithSheet { sheet, on } => {
                assert_eq!(sheet, number, "held to the sheet it is heard on");
                on
            }
            other => panic!("{other:?} is not the sheet's"),
        })
        .collect();
    assert_eq!(
        on,
        [
            OnTheOpenWithSheet::TheApplication(0),
            OnTheOpenWithSheet::TheApplication(1),
            OnTheOpenWithSheet::TheApplication(2),
            OnTheOpenWithSheet::Cancel,
            OnTheOpenWithSheet::Open,
        ]
    );
}

/// **No sheet, no change.**
#[test]
fn with_no_sheet_the_tree_is_as_it_was() {
    let strings = words();
    let plain = ReadAloudTree::of(&strings, &[Surface::Desktop], &TurnedOn::nothing());
    let tree = plain
        .clone()
        .with_the_open_with_sheet(&strings, None, &called);
    assert_eq!(tree, plain);
}
