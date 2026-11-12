//! The password sheet as a reader is told about it: a dialogue named by its
//! heading, the command whole, and the one field with the keyboard named by
//! the line under it — and **nothing about the password**, ever.

use std::sync::mpsc::{Receiver, channel};

use alo_access::{Surface, TurnedOn};
use alo_strings::{Strings, Word};

use super::*;
use crate::SignInKey;
use crate::the_password_sheet::{
    FromTheSheet, PasswordAsked, ThePasswordSheet, ToTheSheet, WhatItIsFor, WhyNotChecked,
};

/// The machine's words, the sheet's among them.
fn words() -> Strings {
    let mut vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    crate::the_password_sheets_words::declare_the_password_sheets_words(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// One word, said.
fn say(strings: &Strings, word: Word) -> String {
    strings
        .say(&word.key(), &alo_strings::Filling::nothing())
        .text()
        .to_owned()
}

/// The desktop's tree with nothing on it but the desktop.
fn plain(strings: &Strings) -> ReadAloudTree {
    ReadAloudTree::of(strings, &[Surface::Desktop], &TurnedOn::nothing())
}

/// A sheet asking for `what_for`, with `typed` typed.
fn a_sheet(what_for: WhatItIsFor, typed: &str) -> (ThePasswordSheet, Receiver<FromTheSheet>) {
    let (answer, answered) = channel();
    let mut sheet = ThePasswordSheet::none();
    sheet.told(ToTheSheet::Ask(PasswordAsked {
        cookie: "c1".to_owned(),
        what_for,
        answer,
    }));
    for letter in typed.chars() {
        sheet.pressed(SignInKey::Letter(letter));
    }
    (sheet, answered)
}

/// The sheet's own node and what hangs under it.
fn the_sheet(tree: &ReadAloudTree) -> (&Node, Vec<&Node>) {
    let sheet = tree.in_front().unwrap();
    let node = &tree.nodes()[sheet.at];
    let under = node.children.iter().map(|at| &tree.nodes()[*at]).collect();
    (node, under)
}

/// **No question, no sheet**: the tree is exactly the desktop's, and nothing
/// is in front.
#[test]
fn with_no_question_the_tree_is_as_it_was() {
    let strings = words();
    let tree = plain(&strings).with_the_password_sheet(&strings, None);
    assert_eq!(tree, plain(&strings));
    assert!(tree.in_front().is_none());
}

/// **Read as the sheet is drawn**: a dialogue named by the heading, under the
/// shell itself; the command whole; and the password field, named by how to
/// answer, with the keyboard on it.
#[test]
fn the_sheet_is_read_as_it_is_drawn() {
    let strings = words();
    let command = "dnf install --assumeyes vlc";
    let (sheet, _answered) = a_sheet(WhatItIsFor::ACommandAloWasAsked(command.to_owned()), "");
    let tree = plain(&strings).with_the_password_sheet(&strings, sheet.shows());
    let (dialogue, under) = the_sheet(&tree);
    assert_eq!(
        dialogue.role,
        access_roles::number_of(alo_access::Role::Dialogue)
    );
    assert_eq!(dialogue.name, say(&strings, words::ALO_ASKS_TO_RUN_THIS));
    assert_eq!(
        dialogue.parent, 0,
        "the sheet is the shell's own, not a window's"
    );
    assert!(access_roles::is_showing(dialogue.states));
    let read: Vec<(u32, &str)> = under.iter().map(|it| (it.role, it.name.as_str())).collect();
    assert_eq!(
        read,
        [
            (access_roles::number_of(alo_access::Role::Label), command),
            (
                access_roles::number_of(alo_access::Role::PasswordEntry),
                say(&strings, words::HOW_TO_ANSWER).as_str()
            ),
        ]
    );
    let field = under[1];
    assert!(access_roles::has_the_keyboard(field.states));
    assert_eq!(
        tree.nodes()[tree.in_front().unwrap().keyboard].path,
        field.path,
        "the field is where the keyboard is said to be"
    );
}

/// **Something else asking is headed as something else**, with polkit's own
/// sentence whole under it.
#[test]
fn something_else_asking_is_said_so() {
    let strings = words();
    let message = "Authentication is required to change the system time";
    let (sheet, _answered) = a_sheet(WhatItIsFor::SomethingElse(message.to_owned()), "");
    let tree = plain(&strings).with_the_password_sheet(&strings, sheet.shows());
    let (dialogue, under) = the_sheet(&tree);
    assert_eq!(dialogue.name, say(&strings, words::SOMETHING_ASKS));
    assert_eq!(under[0].name, message);
}

/// **What went wrong is the field's name**, so a reader told the field was
/// renamed hears it: not right, or could not be checked — never the one in
/// place of the other.
#[test]
fn what_went_wrong_names_the_field() {
    let strings = words();
    for (told, word) in [
        (
            ToTheSheet::NotRight {
                cookie: "c1".to_owned(),
            },
            words::NOT_RIGHT,
        ),
        (
            ToTheSheet::CannotBeChecked {
                cookie: "c1".to_owned(),
                why: WhyNotChecked::NotInStep,
            },
            words::NOT_IN_STEP,
        ),
        (
            ToTheSheet::CannotBeChecked {
                cookie: "c1".to_owned(),
                why: WhyNotChecked::NoChecker,
            },
            words::NO_CHECKER,
        ),
    ] {
        let (mut sheet, _answered) =
            a_sheet(WhatItIsFor::ACommandAloWasAsked("reboot".to_owned()), "x");
        sheet.pressed(SignInKey::Enter);
        sheet.told(told);
        let tree = plain(&strings).with_the_password_sheet(&strings, sheet.shows());
        let (_, under) = the_sheet(&tree);
        assert_eq!(under[1].name, say(&strings, word));
    }
}

/// **Nothing about the password crosses** — not what was typed, not even that
/// anything was: the tree with letters typed is the tree with none, nothing on
/// the sheet can be pressed, and the field is a password field and never an
/// ordinary one.
#[test]
fn nothing_about_the_password_crosses() {
    let strings = words();
    let what_for = || WhatItIsFor::ACommandAloWasAsked("reboot".to_owned());
    let (typed, _typed) = a_sheet(what_for(), "hunter2");
    let (empty, _empty) = a_sheet(what_for(), "");
    let with = plain(&strings).with_the_password_sheet(&strings, typed.shows());
    let without = plain(&strings).with_the_password_sheet(&strings, empty.shows());
    assert_eq!(with.nodes(), without.nodes());
    assert!(
        with.nodes()
            .iter()
            .all(|node| !node.name.contains("hunter2") && !node.description.contains("hunter2"))
    );
    let (_, under) = the_sheet(&with);
    assert_ne!(
        under[1].role,
        access_roles::number_of(alo_access::Role::Entry)
    );
    assert!(under.iter().all(|node| node.press.is_none()));
}

/// **The keyboard is on one thing, and only while the sheet is up.** Without
/// a sheet nothing on the desktop's tree reads as focused (ADR 0001); with
/// one, exactly the field does.
#[test]
fn the_keyboard_is_on_the_field_and_nothing_else() {
    let strings = words();
    let focused = |tree: &ReadAloudTree| {
        tree.nodes()
            .iter()
            .filter(|node| access_roles::has_the_keyboard(node.states))
            .count()
    };
    assert_eq!(focused(&plain(&strings)), 0);
    let (sheet, _answered) = a_sheet(WhatItIsFor::ACommandAloWasAsked("reboot".to_owned()), "");
    let tree = plain(&strings).with_the_password_sheet(&strings, sheet.shows());
    assert_eq!(focused(&tree), 1);
}

/// **A second question is a new sheet to a reader**, even worded the same.
#[test]
fn each_question_is_its_own() {
    let (mut sheet, _answered) = a_sheet(WhatItIsFor::ACommandAloWasAsked("reboot".to_owned()), "");
    let first = sheet.shows().unwrap().asked;
    let (answer, _second) = channel();
    sheet.told(ToTheSheet::Ask(PasswordAsked {
        cookie: "c2".to_owned(),
        what_for: WhatItIsFor::ACommandAloWasAsked("reboot".to_owned()),
        answer,
    }));
    assert_eq!(
        sheet.shows().unwrap().asked,
        first,
        "the one waiting is not in front yet"
    );
    sheet.told(ToTheSheet::Over {
        cookie: "c1".to_owned(),
    });
    assert_ne!(sheet.shows().unwrap().asked, first);
}
