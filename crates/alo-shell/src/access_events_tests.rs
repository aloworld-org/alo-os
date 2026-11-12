//! What a reader is told, moment to moment: a sheet coming up, the keyboard
//! moving on it, the field renamed — and nothing at all otherwise.

use std::sync::mpsc::{Receiver, channel};

use alo_access::{Surface, TurnedOn};
use alo_strings::Strings;

use super::*;
use crate::SignInKey;
use crate::the_password_sheet::{
    FromTheSheet, PasswordAsked, ThePasswordSheet, ToTheSheet, WhatItIsFor,
};

/// The machine's words, the password sheet's among them.
fn words() -> Strings {
    let mut vocabulary = alo_saying::everything_this_machine_can_say().unwrap();
    crate::the_password_sheets_words::declare_the_password_sheets_words(&mut vocabulary).unwrap();
    Strings::of(vocabulary)
}

/// A password question up, named `cookie`.
fn asked(sheet: &mut ThePasswordSheet, cookie: &str) -> Receiver<FromTheSheet> {
    let (answer, answered) = channel();
    sheet.told(ToTheSheet::Ask(PasswordAsked {
        cookie: cookie.to_owned(),
        what_for: WhatItIsFor::ACommandAloWasAsked("reboot".to_owned()),
        answer,
    }));
    answered
}

/// The desktop's tree, with the password sheet as it is.
fn tree(strings: &Strings, sheet: &ThePasswordSheet) -> ReadAloudTree {
    ReadAloudTree::of(strings, &[Surface::Desktop], &TurnedOn::nothing())
        .with_the_password_sheet(strings, sheet.shows())
}

/// **No sheet, nothing said** — the desktop behaves as it did before any of
/// this, and nothing is remembered.
#[test]
fn with_no_sheet_nothing_is_said() {
    let strings = words();
    let (told, keyboard) = what_happened(None, &tree(&strings, &ThePasswordSheet::none()));
    assert!(told.is_empty());
    assert!(keyboard.is_none());
}

/// **A sheet coming up is said to be in front, then where its keys go**, and
/// the same sheet drawn again says nothing more.
#[test]
fn a_sheet_coming_up_is_said_once() {
    let strings = words();
    let mut sheet = ThePasswordSheet::none();
    let _answered = asked(&mut sheet, "c1");
    let now = tree(&strings, &sheet);
    let (told, keyboard) = what_happened(None, &now);
    let in_front = now.in_front().unwrap();
    assert_eq!(
        told,
        [
            AnEvent::InFront {
                path: now.nodes()[in_front.at].path.clone(),
                name: now.nodes()[in_front.at].name.clone(),
            },
            AnEvent::TheKeyboardIsHere {
                path: now.nodes()[in_front.keyboard].path.clone(),
            },
        ]
    );
    let (again, _) = what_happened(keyboard.as_ref(), &tree(&strings, &sheet));
    assert!(
        again.is_empty(),
        "the same sheet redrawn was announced again"
    );
}

/// **The password not being right renames the field**, and that is all that
/// is said — the sheet did not come up again.
#[test]
fn not_right_is_said_as_the_fields_new_name() {
    let strings = words();
    let mut sheet = ThePasswordSheet::none();
    let _answered = asked(&mut sheet, "c1");
    let (_, keyboard) = what_happened(None, &tree(&strings, &sheet));
    sheet.pressed(SignInKey::Letter('x'));
    sheet.pressed(SignInKey::Enter);
    let (checking, keyboard) = what_happened(keyboard.as_ref(), &tree(&strings, &sheet));
    assert!(matches!(checking.as_slice(), [AnEvent::NowCalled { .. }]));
    sheet.told(ToTheSheet::NotRight {
        cookie: "c1".to_owned(),
    });
    let now = tree(&strings, &sheet);
    let (told, _) = what_happened(keyboard.as_ref(), &now);
    let field = &now.nodes()[now.in_front().unwrap().keyboard];
    assert_eq!(
        told,
        [AnEvent::NowCalled {
            path: field.path.clone(),
            name: field.name.clone(),
        }]
    );
}

/// **A new question is a new sheet**, even worded exactly as the last.
#[test]
fn a_new_question_comes_up_again() {
    let strings = words();
    let mut sheet = ThePasswordSheet::none();
    let _first = asked(&mut sheet, "c1");
    let _second = asked(&mut sheet, "c2");
    let (_, keyboard) = what_happened(None, &tree(&strings, &sheet));
    sheet.told(ToTheSheet::Over {
        cookie: "c1".to_owned(),
    });
    let (told, _) = what_happened(keyboard.as_ref(), &tree(&strings, &sheet));
    assert!(matches!(
        told.as_slice(),
        [AnEvent::InFront { .. }, AnEvent::TheKeyboardIsHere { .. }]
    ));
}

/// **A sheet going down says nothing, and forgets where the keyboard was**,
/// so the next one is said to come up.
#[test]
fn a_sheet_going_down_says_nothing() {
    let strings = words();
    let mut sheet = ThePasswordSheet::none();
    let _answered = asked(&mut sheet, "c1");
    let (_, keyboard) = what_happened(None, &tree(&strings, &sheet));
    sheet.pressed(SignInKey::Clear);
    let (told, keyboard) = what_happened(keyboard.as_ref(), &tree(&strings, &sheet));
    assert!(told.is_empty());
    assert!(keyboard.is_none());
}

/// **A window opening behind the sheet moves its paths, and the keyboard is
/// told again** — a reader holding the old path would be holding something
/// else.
#[test]
fn a_sheet_renumbered_says_where_the_keyboard_is_again() {
    let strings = words();
    let mut sheet = ThePasswordSheet::none();
    let _answered = asked(&mut sheet, "c1");
    let (_, keyboard) = what_happened(None, &tree(&strings, &sheet));
    let moved = ReadAloudTree::of(&strings, &[Surface::Desktop], &TurnedOn::nothing())
        .with_the_frames_open(
            &strings,
            &[(
                crate::frame_name::FrameName::Given("Ledger for March".to_owned()),
                crate::Decorations::TheShellDraws,
            )],
            &TurnedOn::nothing(),
        )
        .with_the_password_sheet(&strings, sheet.shows());
    let (told, _) = what_happened(keyboard.as_ref(), &moved);
    assert!(matches!(
        told.as_slice(),
        [AnEvent::TheKeyboardIsHere { .. }]
    ));
}

/// **The control with the keyboard being set is said**, once, and being unset
/// again is said too — a switch pressed on Settings, as a reader hears it.
#[test]
fn the_keyboards_control_being_set_is_said() {
    let strings = words();
    let mut sheet = ThePasswordSheet::none();
    let _answered = asked(&mut sheet, "c1");
    let before = tree(&strings, &sheet);
    let (_, keyboard) = what_happened(None, &before);
    let at = before.in_front().unwrap().keyboard;
    let mut set = before.clone();
    let node = set.node_mut(at).unwrap();
    node.states = [node.states[0] | (1 << crate::access_roles::CHECKED), node.states[1]];
    let path = node.path.clone();
    let (told, keyboard) = what_happened(keyboard.as_ref(), &set);
    assert_eq!(
        told,
        [AnEvent::NowChecked {
            path: path.clone(),
            checked: true,
        }]
    );
    let (again, keyboard) = what_happened(keyboard.as_ref(), &set);
    assert!(again.is_empty(), "the same state was said twice");
    let (unset, _) = what_happened(keyboard.as_ref(), &before);
    assert_eq!(unset, [AnEvent::NowChecked { path, checked: false }]);
}
